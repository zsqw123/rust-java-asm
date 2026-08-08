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
use std::io::{Read, Seek};
use std::sync::Arc;
use tokio::sync::{Mutex as AsyncMutex, mpsc::Sender};
use zip::ZipArchive;

pub struct ApkAccessor {
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
    completed_tasks: usize,
}

const MAX_PARALLEL_DEX_TASKS: usize = 16;

impl ProgressReporter {
    fn new(sender: Sender<ServerMessage>, task_count: usize) -> Self {
        Self {
            sender,
            last_percent: -1,
            task_count,
            completed_tasks: 0,
        }
    }

    async fn report(&mut self, progress: f32, message: impl FnOnce() -> String) -> bool {
        let progress = progress.clamp(0.0, 1.0);
        let percent = (progress * 100.0).floor() as i16;
        if percent <= self.last_percent || percent >= 100 {
            return false;
        }
        self.last_percent = percent;
        let message = message();
        send_progress(&self.sender, percent as f32 / 100.0, message).await;
        true
    }

    async fn report_task(&mut self, message: impl FnOnce() -> String) -> bool {
        self.completed_tasks = self.completed_tasks.saturating_add(1);
        let progress = if self.task_count == 0 {
            1.0
        } else {
            self.completed_tasks.min(self.task_count) as f32 / self.task_count as f32
        };
        self.report(progress, message).await
    }
}

async fn report_progress(
    reporter: &Arc<AsyncMutex<ProgressReporter>>,
    progress: f32,
    message: impl FnOnce() -> String,
) -> bool {
    reporter.lock().await.report(progress, message).await
}

pub(crate) async fn report_task_progress(
    reporter: &Arc<AsyncMutex<ProgressReporter>>,
    message: impl FnOnce() -> String,
) -> bool {
    reporter.lock().await.report_task(message).await
}

async fn finish_dex_task<E>(
    reporter: &Arc<AsyncMutex<ProgressReporter>>,
    indexed_dexes: &mut [Option<IndexedDex>],
    index: usize,
    display_name: String,
    task_result: Result<Option<IndexedDex>, E>,
) -> bool {
    indexed_dexes[index] = task_result.ok().flatten();
    report_task_progress(
        reporter,
        || format!("Loading {display_name}..."),
    ).await
}

// Keep APK loading shared. Each target supplies only the scheduling checkpoint
// used to let its runtime process pending UI work.
pub async fn read_apk<F, Fut>(
    zip_archive: ZipArchive<impl Read + Seek>, sender: Sender<ServerMessage>,
    yield_step: F,
) -> Result<ApkAccessor, OpenFileError>
where
    F: Fn() -> Fut + Clone + 'static,
    Fut: Future<Output=()> + 'static,
{
    let mut zip_archive = zip_archive;
    yield_step().await;

    // read dex files
    let mut dex_files = zip_archive
        .file_names()
        .filter(|name|
            // classes dex should be classes.dex or classes*.dex, and not in the sub directory.
            name.starts_with("classes") && name.ends_with(".dex") && !name.contains("/")
        ).collect::<Vec<_>>();
    dex_files.sort_by(|a, b| dex_index(a).cmp(&dex_index(b)));
    
    // read zip entry indices
    let dex_files: Vec<_> = dex_files.into_iter().filter_map(|name| {
        zip_archive.index_for_name(name).map(|index| (index, name.to_owned()))
    }).collect();

    let dex_count = dex_files.len();
    let reporter = Arc::new(AsyncMutex::new(ProgressReporter::new(
        sender.clone(), dex_count,
    )));
    if report_progress(&reporter, 0.0, || "Loading APK...".to_owned()).await {
        yield_step().await;
    }
    let mut pending = FuturesUnordered::new();
    let mut indexed_dexes: Vec<Option<IndexedDex>> = (0..dex_count)
        .map(|_| None)
        .collect();
    let mut active_tasks = 0usize;

    for (index, (entry_index, display_name)) in dex_files.into_iter().enumerate() {
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
        let bytes = match zip_archive.by_index(entry_index) {
            Ok(mut file) => {
                // DEX entries are retained by the accessor, so reserve their final
                // uncompressed size once instead of repeatedly growing the buffer.
                let capacity = file.size().min(usize::MAX as u64) as usize;
                let mut bytes = Vec::with_capacity(capacity);
                match file.read_to_end(&mut bytes) {
                    Ok(_) => Some(bytes),
                    Err(err) => {
                        error!("Error when reading {display_name}: {err:?}");
                        None
                    }
                }
            }
            Err(err) => {
                error!("Error when reading dex entry: {err:?}");
                None
            }
        };
        if let Some(bytes) = bytes {
            let task = crate::targets::spawn_process_dex(
                index,
                display_name.to_owned(),
                bytes,
                yield_step.clone(),
            );
            pending.push(async move { (index, display_name.to_owned(), task.await) });
            active_tasks += 1;
        } else {
            finish_dex_task(
                &reporter,
                &mut indexed_dexes,
                index,
                display_name.to_owned(),
                Ok::<Option<IndexedDex>, ()>(None),
            ).await;
        }
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
    let class_capacity = indexed_dexes.iter()
        .filter_map(Option::as_ref)
        .map(|dex| dex.classes.len())
        .sum();
    let mut dex_sources = HashMap::with_capacity(indexed_dexes.len());
    let mut map = HashMap::with_capacity(class_capacity);
    for indexed_dex in indexed_dexes.into_iter().flatten() {
        let IndexedDex { file_name, accessor, classes } = indexed_dex;
        dex_sources.insert(file_name, Arc::clone(&accessor));
        for (class_name, class_def) in classes {
            // dex index is the priority, the lower the index, the higher the priority.
            // Results are merged in dex filename order, so the first definition wins.
            map.entry(class_name).or_insert((Arc::clone(&accessor), class_def));
        }
    }
    map.shrink_to_fit();
    send_loaded(&sender, "APK loaded").await;
    Ok(ApkAccessor { map, dex_sources })
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

// classes.dex -> 0
// classes2.dex -> 2
#[inline]
fn dex_index(name: &str) -> usize {
    let dex_index_end = name.rfind('.').unwrap_or_default();
    let dex_index_start = 7usize;
    name[dex_index_start..dex_index_end].parse::<usize>().unwrap_or_default()
}


impl Accessor for ApkAccessor {
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

    // source key is the dex file name.
    fn peek_source(&self, source_key: &str) -> Option<ExportableSource> {
        let dex_source = self.dex_sources.get(source_key);
        let Some(dex_source) = dex_source else {
            warn!("No source found for: {source_key} when trying peek source.");
            return None;
        };
        let file_name = dex_source.file_name.clone();
        let source = dex_source.bytes.clone();
        Some(ExportableSource {
            exportable_name: file_name,
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ProgressReporter, ServerMessage};
    use std::cell::Cell;
    use tokio::sync::mpsc;

    #[test]
    fn progress_reports_once_per_percent() {
        let (sender, mut receiver) = mpsc::channel(8);
        futures::executor::block_on(async {
            let mut reporter = ProgressReporter::new(sender, 0);
            let message_built = Cell::new(false);
            assert!(reporter.report(0.0, || {
                message_built.set(true);
                "Loading APK...".to_owned()
            }).await);
            message_built.set(false);
            assert!(!reporter.report(0.005, || {
                message_built.set(true);
                "Loading APK...".to_owned()
            }).await);
            assert!(!message_built.get());
            assert!(reporter.report(0.011, || {
                message_built.set(true);
                "Loading APK...".to_owned()
            }).await);
            assert!(message_built.get());
            message_built.set(false);
            assert!(!reporter.report(0.019, || {
                message_built.set(true);
                "Loading APK...".to_owned()
            }).await);
            assert!(!message_built.get());
            assert!(reporter.report(0.02, || {
                message_built.set(true);
                "Loading APK...".to_owned()
            }).await);
        });

        let mut progress = Vec::new();
        while let Ok(ServerMessage::Progress(message)) = receiver.try_recv() {
            progress.push(message.progress);
        }
        assert_eq!(progress, vec![0.0, 0.01, 0.02]);
    }

    #[test]
    fn progress_reports_completed_tasks_equally() {
        let (sender, mut receiver) = mpsc::channel(8);
        futures::executor::block_on(async {
            let mut reporter = ProgressReporter::new(sender, 2);
            assert!(reporter.report(0.0, || "Loading APK...".to_owned()).await);
            assert!(reporter.report_task(|| "Loading classes.dex...".to_owned()).await);
            assert!(!reporter.report_task(|| "Loading classes2.dex...".to_owned()).await);
        });

        let mut progress = Vec::new();
        while let Ok(ServerMessage::Progress(message)) = receiver.try_recv() {
            progress.push(message.progress);
        }
        assert_eq!(progress, vec![0.0, 0.5]);
    }
}

