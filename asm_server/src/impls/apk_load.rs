use crate::impls::server::{ProgressMessage, ServerMessage};
use crate::server::OpenFileError;
use crate::{Accessor, ExportableSource};
use futures::stream::{FuturesUnordered, StreamExt};
use java_asm::dex::{ClassDef, DexFile, DexFileAccessor};
use java_asm::smali::{SmaliNode, SmaliToken, stb};
use java_asm::{DescriptorRef, StrRef};
use log::{error, warn};
use std::collections::HashMap;
use std::future::Future;
use std::io::{Cursor, Read};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Sender;
use zip::ZipArchive;

pub struct DexAccessor {
    pub map: HashMap<DescriptorRef, ClassPosition>,
    pub dex_sources: HashMap<StrRef, Arc<DexFileAccessor>>,
}

type ClassPosition = (Arc<DexFileAccessor>, ClassDef);

pub(crate) struct IndexedDex {
    pub(crate) file_name: StrRef,
    pub(crate) accessor: Arc<DexFileAccessor>,
    pub(crate) classes: Vec<(DescriptorRef, ClassDef)>,
}

pub(crate) struct ProgressReporter {
    sender: Sender<ServerMessage>,
    last_percent: i16,
    task_count: usize,
    task_progress: Vec<f32>,
}

const MAX_PARALLEL_DEX_TASKS: usize = 16;
pub(crate) const DEX_COLLECT_PROGRESS_END: f32 = 0.9;

struct DexCollectProgress {
    sender: Sender<ServerMessage>,
    total_entries: usize,
    completed_entries: usize,
    last_percent: i16,
}

impl DexCollectProgress {
    fn new(sender: Sender<ServerMessage>, total_entries: usize) -> Self {
        Self { sender, total_entries, completed_entries: 0, last_percent: -1 }
    }

    fn add_entry(&mut self) -> bool {
        self.completed_entries = self.completed_entries.saturating_add(1);
        let fraction = if self.total_entries == 0 {
            1.0
        } else {
            (self.completed_entries as f32 / self.total_entries as f32).min(1.0)
        };
        let progress = fraction * DEX_COLLECT_PROGRESS_END;
        let percent = (progress * 100.0).floor() as i16;
        if percent <= self.last_percent
            || percent > (DEX_COLLECT_PROGRESS_END * 100.0) as i16
        {
            return false;
        }
        if self.sender.capacity() <= 1
            || !try_send_progress(
            &self.sender,
            progress,
            format!("Collecting DEX files... {percent}%"),
        )
        {
            return false;
        }
        self.last_percent = percent;
        true
    }
}

impl ProgressReporter {
    fn new(sender: Sender<ServerMessage>, task_count: usize) -> Self {
        Self {
            sender,
            last_percent: -1,
            task_count,
            task_progress: vec![0.0; task_count],
        }
    }

    fn report(&mut self, progress: f32, message: impl FnOnce() -> String) -> bool {
        let progress = DEX_COLLECT_PROGRESS_END
            + progress.clamp(0.0, 1.0) * (1.0 - DEX_COLLECT_PROGRESS_END);
        let percent = (progress * 100.0).floor() as i16;
        if percent <= self.last_percent || percent >= 100 {
            return false;
        }
        let message = message();
        // Progress is best-effort. A slow UI consumer must not block a native
        // DEX worker, and a later percentage will retry after the queue drains.
        // Keep one queue slot free for the reliable final completion message.
        if self.sender.capacity() <= 1
            || !try_send_progress(&self.sender, percent as f32 / 100.0, message)
        {
            return false;
        }
        self.last_percent = percent;
        true
    }

    fn report_task(
        &mut self, task_index: usize, message: impl FnOnce() -> String,
    ) -> bool {
        if let Some(progress) = self.task_progress.get_mut(task_index) {
            *progress = 1.0;
        }
        let progress = if self.task_count == 0 {
            1.0
        } else {
            self.task_progress.iter().sum::<f32>() / self.task_count as f32
        };
        self.report(progress, message)
    }

    fn report_task_progress(
        &mut self, task_index: usize, progress: f32, message: impl FnOnce() -> String,
    ) -> bool {
        if let Some(task_progress) = self.task_progress.get_mut(task_index) {
            *task_progress = progress.clamp(0.0, 1.0);
        }
        let progress = if self.task_count == 0 {
            1.0
        } else {
            self.task_progress.iter().sum::<f32>() / self.task_count as f32
        };
        self.report(progress, message)
    }
}

pub(crate) fn try_send_progress(
    sender: &Sender<ServerMessage>, progress: f32, message: impl Into<String>,
) -> bool {
    sender.try_send(ServerMessage::Progress(ProgressMessage {
        progress: progress.clamp(0.0, 1.0),
        in_loading: true,
        message: message.into(),
    })).is_ok()
}

fn report_progress(
    reporter: &Arc<Mutex<ProgressReporter>>,
    progress: f32,
    message: impl FnOnce() -> String,
) -> bool {
    let Ok(mut reporter) = reporter.lock() else {
        return false;
    };
    reporter.report(progress, message)
}

pub(crate) fn report_task_progress(
    reporter: &Arc<Mutex<ProgressReporter>>, task_index: usize,
    message: impl FnOnce() -> String,
) -> bool {
    let Ok(mut reporter) = reporter.lock() else {
        return false;
    };
    reporter.report_task(task_index, message)
}

pub(crate) fn report_dex_progress(
    reporter: &Arc<Mutex<ProgressReporter>>, task_index: usize,
    progress: f32, message: impl FnOnce() -> String,
) -> bool {
    let Ok(mut reporter) = reporter.try_lock() else {
        return false;
    };
    reporter.report_task_progress(task_index, progress, message)
}

async fn finish_dex_task<E>(
    reporter: &Arc<Mutex<ProgressReporter>>,
    indexed_dexes: &mut [Option<IndexedDex>],
    index: usize,
    display_name: String,
    task_result: Result<Option<IndexedDex>, E>,
) -> bool {
    indexed_dexes[index] = task_result.ok().flatten();
    report_task_progress(
        reporter, index,
        || format!("Loading {display_name}..."),
    )
}

// package loading logic is shared for all clients.
pub(crate) async fn read_dex_inputs<F, Fut>(
    inputs: Vec<(String, Vec<u8>)>, sender: Sender<ServerMessage>, yield_step: F,
) -> Result<DexAccessor, OpenFileError>
where
    F: Fn() -> Fut + Clone + 'static,
    Fut: Future<Output=()> + 'static,
    F: CollectYield,
{
    send_progress(&sender, 0.0, "Collecting DEX files...").await;
    yield_step().await;
    let total_entries = inputs.iter()
        .map(|(input_name, bytes)| count_valid_entries(input_name, bytes))
        .try_fold(0usize, |total, count| count.map(|count| total.saturating_add(count)))?;
    let mut collect_progress = DexCollectProgress::new(sender.clone(), total_entries);
    let mut dex_files = Vec::new();
    for (display_name, bytes) in inputs {
        let collected = collect_dex_files(
            &display_name, bytes, Some(display_name.as_str()), 0,
            &mut collect_progress, true, &yield_step,
        ).await?;
        dex_files.extend(collected);
        yield_step().await;
    }
    make_source_names_unique(&mut dex_files);

    // reports
    if dex_files.is_empty() {
        return Err(OpenFileError::Custom(
            "selected inputs contain no DEX files".to_owned(),
        ));
    }
    send_progress(
        &sender, DEX_COLLECT_PROGRESS_END,
        format!("Collected {} DEX file(s)...", dex_files.len()),
    ).await;
    yield_step().await;

    read_dex_sources(dex_files, sender, yield_step).await
}

const MAX_ARCHIVE_DEPTH: usize = 16;

#[cfg(not(target_family = "wasm"))]
type CollectFuture<'a, T> = futures::future::BoxFuture<'a, T>;

#[cfg(target_family = "wasm")]
type CollectFuture<'a, T> = futures::future::LocalBoxFuture<'a, T>;

#[cfg(not(target_family = "wasm"))]
pub(crate) trait CollectYield: Send + Sync {
    type Future: Future<Output=()> + Send;

    fn collect_yield(&self) -> Self::Future;
}

#[cfg(target_family = "wasm")]
pub(crate) trait CollectYield {
    type Future: Future<Output=()>;

    fn collect_yield(&self) -> Self::Future;
}

#[cfg(not(target_family = "wasm"))]
impl<F, Fut> CollectYield for F
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output=()> + Send,
{
    type Future = Fut;

    fn collect_yield(&self) -> Self::Future {
        self()
    }
}

#[cfg(target_family = "wasm")]
impl<F, Fut> CollectYield for F
where
    F: Fn() -> Fut,
    Fut: Future<Output=()>,
{
    type Future = Fut;

    fn collect_yield(&self) -> Self::Future {
        self()
    }
}

fn collect_dex_files<'a, Y>(
    input_name: &'a str, bytes: Vec<u8>, source_prefix: Option<&'a str>, depth: usize,
    progress: &'a mut DexCollectProgress, report_current_archive: bool,
    yield_step: &'a Y,
) -> CollectFuture<'a, Result<Vec<(String, Vec<u8>)>, OpenFileError>>
where
    Y: CollectYield + 'a,
{
    Box::pin(async move {
        if is_dex_bytes(&bytes) {
            if report_current_archive && progress.add_entry() {
                yield_step.collect_yield().await;
            }
            return Ok(vec![(
                source_prefix.unwrap_or(input_name).to_owned(),
                bytes,
            )]);
        }
        if !is_zip_bytes(&bytes) {
            return Err(OpenFileError::Custom(format!(
                "unsupported input: {input_name}"
            )));
        }
        if depth >= MAX_ARCHIVE_DEPTH {
            return Err(OpenFileError::Custom(format!(
                "archive nesting is too deep: {input_name}"
            )));
        }

        let mut archive = ZipArchive::new(Cursor::new(bytes))
            .map_err(OpenFileError::LoadZip)?;
        let mut dex_files = Vec::new();
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).map_err(OpenFileError::LoadZip)?;
            let entry_name = entry.name().to_owned();
            let is_valid_entry = !entry.is_dir() && is_possible_input_entry(&entry_name);
            if !is_valid_entry { continue; }
            let mut nested_entry = None;
            let capacity = entry.size().min(usize::MAX as u64) as usize;
            // APKs contain a lot of unrelated resources. Read only a four-byte
            // header first; fully decompress an entry only if it can be a DEX or
            // another nested package archive.
            if capacity < 4 { continue; }
            let mut header = [0; 4];
            entry.read_exact(&mut header).map_err(OpenFileError::Io)?;
            if !is_dex_bytes(&header) && !is_zip_bytes(&header) { continue; }

            // read entry start
            let mut entry_bytes = Vec::with_capacity(capacity);
            entry_bytes.extend_from_slice(&header);
            entry.read_to_end(&mut entry_bytes).map_err(OpenFileError::Io)?;
            let entry_source = match source_prefix {
                Some(prefix) => format!("{prefix}!{entry_name}"),
                None => entry_name.clone(),
            };
            if is_dex_bytes(&entry_bytes) {
                dex_files.push((entry_source, entry_bytes));
            } else if is_zip_bytes(&entry_bytes) {
                nested_entry = Some((entry_name, entry_bytes, entry_source));
            }
            // read entry ends

            // read nested entry
            if let Some((entry_name, entry_bytes, entry_source)) = nested_entry {
                dex_files.extend(collect_dex_files(
                    &entry_name, entry_bytes, Some(&entry_source), depth + 1,
                    progress, false, yield_step,
                ).await?);
            }
            drop(entry);

            if report_current_archive && progress.add_entry() {
                yield_step.collect_yield().await;
            } else if !report_current_archive {
                yield_step.collect_yield().await;
            }
        }
        Ok(dex_files)
    })
}

fn count_valid_entries(input_name: &str, bytes: &[u8]) -> Result<usize, OpenFileError> {
    if is_dex_bytes(bytes) {
        return Ok(1);
    }
    if !is_zip_bytes(bytes) {
        return Err(OpenFileError::Custom(format!(
            "unsupported input: {input_name}"
        )));
    }
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(OpenFileError::LoadZip)?;
    let mut total = 0usize;
    for index in 0..archive.len() {
        let entry = archive.by_index(index).map_err(OpenFileError::LoadZip)?;
        if !entry.is_dir() && is_possible_input_entry(entry.name()) {
            total = total.saturating_add(1);
        }
    }
    Ok(total)
}

fn make_source_names_unique(dex_files: &mut [(String, Vec<u8>)]) {
    let mut names = HashMap::<String, usize>::new();
    for (name, _) in dex_files {
        let count = names.entry(name.clone()).or_default();
        *count += 1;
        if *count > 1 {
            name.push_str(&format!("#{count}"));
        }
    }
}

fn is_dex_bytes(bytes: &[u8]) -> bool {
    bytes.get(..4) == Some(b"dex\n")
}

fn is_zip_bytes(bytes: &[u8]) -> bool {
    matches!(bytes.get(..4), Some(b"PK\x03\x04" | b"PK\x05\x06" | b"PK\x07\x08"))
}

fn is_possible_input_entry(name: &str) -> bool {
    let Some((_, extension)) = name.rsplit_once('.') else {
        // Keep magic-based detection for entries without an extension.
        return true;
    };
    ["dex", "apk", "apks", "xapk", "aab", "zip", "jar"]
        .iter()
        .any(|candidate| extension.eq_ignore_ascii_case(candidate))
}

async fn read_dex_sources<F, Fut>(
    dex_files: Vec<(String, Vec<u8>)>, sender: Sender<ServerMessage>, yield_step: F,
) -> Result<DexAccessor, OpenFileError>
where
    F: Fn() -> Fut + Clone + 'static,
    Fut: Future<Output=()> + 'static,
{
    let dex_count = dex_files.len();
    let reporter = Arc::new(Mutex::new(ProgressReporter::new(
        sender.clone(), dex_count,
    )));
    if report_progress(&reporter, 0.0, || {
        format!("Loading {dex_count} DEX file(s)...")
    }) {
        yield_step().await;
    }
    let mut pending = FuturesUnordered::new();
    let mut indexed_dexes: Vec<Option<IndexedDex>> = (0..dex_count)
        .map(|_| None)
        .collect();
    let mut active_tasks = 0usize;

    for (index, (display_name, bytes)) in dex_files.into_iter().enumerate() {
        while active_tasks >= MAX_PARALLEL_DEX_TASKS {
            let Some((completed_index, completed_name, task_result)) = pending.next().await else {
                active_tasks = 0;
                break;
            };
            active_tasks -= 1;
            finish_dex_task(
                &reporter,
                &mut indexed_dexes,
                completed_index,
                completed_name,
                task_result,
            ).await;
            yield_step().await;
        }
        let task = crate::targets::spawn_process_dex(
            index, display_name.clone(), bytes, yield_step.clone(),
            Arc::clone(&reporter),
        );
        pending.push(async move { (index, display_name, task.await) });
        active_tasks += 1;
        yield_step().await;
    }

    while let Some((index, display_name, task_result)) = pending.next().await {
        finish_dex_task(
            &reporter,
            &mut indexed_dexes,
            index,
            display_name,
            task_result,
        ).await;
        yield_step().await;
    }
    let accessor = build_accessor(indexed_dexes.into_iter().flatten());
    send_loaded(&sender, format!("Loaded {dex_count} DEX file(s)")).await;
    Ok(accessor)
}

fn build_accessor(indexed_dexes: impl IntoIterator<Item=IndexedDex>) -> DexAccessor {
    let indexed_dexes: Vec<_> = indexed_dexes.into_iter().collect();
    let class_capacity = indexed_dexes.iter()
        .map(|dex| dex.classes.len())
        .sum();
    let mut dex_sources = HashMap::with_capacity(indexed_dexes.len());
    let mut map = HashMap::with_capacity(class_capacity);
    for IndexedDex { file_name, accessor, classes } in indexed_dexes {
        dex_sources.insert(file_name, Arc::clone(&accessor));
        for (class_name, class_def) in classes {
            // Results are merged in collected input/archive order, so the first definition wins.
            map.entry(class_name).or_insert((Arc::clone(&accessor), class_def));
        }
    }
    map.shrink_to_fit();
    DexAccessor { map, dex_sources }
}

pub(crate) fn resolve_dex(
    display_name: &str, bytes: Vec<u8>,
) -> Option<(StrRef, Arc<DexFileAccessor>)> {
    let file_name = StrRef::from(display_name);
    let dex_file = match DexFile::resolve_from_bytes(&bytes) {
        Ok(dex_file) => dex_file,
        Err(err) => {
            error!("Error when resolving {display_name}: {err:?}");
            return None;
        }
    };
    let accessor = Arc::new(DexFileAccessor::new(dex_file, bytes, file_name.clone()));
    Some((file_name, accessor))
}

pub(crate) async fn send_progress(
    sender: &Sender<ServerMessage>, progress: f32, message: impl Into<String>,
) {
    let message = ServerMessage::Progress(ProgressMessage {
        progress,
        in_loading: true,
        message: message.into(),
    });
    sender.send(message).await.unwrap();
}

async fn send_loaded(
    sender: &Sender<ServerMessage>, message: impl Into<String>,
) {
    let message = ServerMessage::Progress(ProgressMessage {
        progress: 1.0,
        in_loading: false,
        message: message.into(),
    });
    sender.send(message).await.unwrap();
}

impl Accessor for DexAccessor {
    fn read_classes(&self) -> Vec<StrRef> {
        self.map.keys().cloned().collect()
    }

    fn exist_class(&self, class_key: &str) -> bool {
        self.map.contains_key(class_key)
    }

    fn read_content(&self, class_key: &str) -> Option<SmaliNode> {
        let class_position = self.map.get(class_key);
        if let Some((accessor, class_def)) = class_position {
            let dex_file_name = accessor.file_name.clone();
            let smali_node = accessor.get_class_smali(*class_def).ok();
            let Some(smali_node) = smali_node else {
                warn!("No class content found for: {}", class_key);
                return None;
            };
            let mut smali_node = smali_node;
            let source_tag_smali = stb().push(SmaliToken::SourceInfo(dex_file_name)).s();
            smali_node.children.insert(0, source_tag_smali);
            Some(smali_node)
        } else {
            warn!("No class content found for: {}", class_key);
            None
        }
    }

    // Source keys are DEX names, optionally containing `!`-separated nested archive paths.
    fn peek_source(&self, source_key: &str) -> Option<ExportableSource> {
        let dex_source = self.dex_sources.get(source_key);
        let Some(dex_source) = dex_source else {
            warn!("No source found for: {source_key} when trying peek source.");
            return None;
        };
        let file_name = dex_source.file_name.clone();
        let exportable_name = file_name
            .rsplit(['!', '/'])
            .next()
            .unwrap_or(&file_name)
            .into();
        let source = dex_source.bytes.clone();
        Some(ExportableSource {
            exportable_name,
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{DexCollectProgress, ProgressReporter, ServerMessage};
    use std::cell::Cell;
    use tokio::sync::mpsc;

    #[test]
    fn collect_progress_counts_entries_equally() {
        let (sender, mut receiver) = mpsc::channel(8);
        let mut reporter = DexCollectProgress::new(sender, 3);
        assert!(reporter.add_entry());
        assert!(reporter.add_entry());
        assert!(reporter.add_entry());

        let mut progress = Vec::new();
        while let Ok(ServerMessage::Progress(message)) = receiver.try_recv() {
            progress.push(message.progress);
        }
        assert_eq!(progress, vec![0.3, 0.6, 0.9]);
    }

    #[test]
    fn progress_reports_once_per_percent() {
        let (sender, mut receiver) = mpsc::channel(8);
        let mut reporter = ProgressReporter::new(sender, 0);
        let message_built = Cell::new(false);
        assert!(reporter.report(0.0, || {
            message_built.set(true);
            "Loading APK...".to_owned()
        }));
        message_built.set(false);
        assert!(!reporter.report(0.005, || {
            message_built.set(true);
            "Loading APK...".to_owned()
        }));
        assert!(!message_built.get());
        assert!(reporter.report(0.11, || {
            message_built.set(true);
            "Loading APK...".to_owned()
        }));
        assert!(message_built.get());
        message_built.set(false);
        assert!(!reporter.report(0.19, || {
            message_built.set(true);
            "Loading APK...".to_owned()
        }));
        assert!(!message_built.get());
        assert!(reporter.report(0.21, || {
            message_built.set(true);
            "Loading APK...".to_owned()
        }));

        let mut progress = Vec::new();
        while let Ok(ServerMessage::Progress(message)) = receiver.try_recv() {
            progress.push(message.progress);
        }
        assert_eq!(progress, vec![0.9, 0.91, 0.92]);
    }

    #[test]
    fn progress_reports_completed_tasks_equally() {
        let (sender, mut receiver) = mpsc::channel(8);
        let mut reporter = ProgressReporter::new(sender, 2);
        assert!(reporter.report(0.0, || "Loading APK...".to_owned()));
        assert!(reporter.report_task(0, || "Loading classes.dex...".to_owned()));
        assert!(!reporter.report_task(1, || "Loading classes2.dex...".to_owned()));

        let mut progress = Vec::new();
        while let Ok(ServerMessage::Progress(message)) = receiver.try_recv() {
            progress.push(message.progress);
        }
        assert_eq!(progress, vec![0.9, 0.95]);
    }
}
