use crate::impls::server::{ProgressMessage, ServerMessage};
use crate::server::OpenFileError;
use crate::{Accessor, ExportableSource};
use java_asm::dex::{ClassDef, DexFile, DexFileAccessor};
use java_asm::smali::{stb, SmaliNode, SmaliToken};
use java_asm::{DescriptorRef, StrRef};
use log::{error, warn};
use std::collections::HashMap;
use std::future::Future;
use std::io::{Read, Seek};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
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

// Keep APK loading shared. Each target supplies only the scheduling checkpoint
// used to let its runtime process pending UI work.
pub async fn read_apk<F, Fut>(
    zip_archive: ZipArchive<impl Read + Seek>, sender: Sender<ServerMessage>,
    mut yield_step: F,
) -> Result<ApkAccessor, OpenFileError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = ()>,
{
    let mut zip_archive = zip_archive;
    send_progress(&sender, 0.0, "Scanning APK...").await;
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

    let dex_file_count = dex_files.len();
    let mut dex_inputs = Vec::with_capacity(dex_file_count);

    for (index, (entry_index, display_name)) in dex_files.into_iter().enumerate() {
        let dex_start = index as f32 / dex_file_count.max(1) as f32;
        let dex_span = 1.0 / dex_file_count.max(1) as f32;

        send_progress(
            &sender,
            dex_start + dex_span * 0.05,
            format!("Reading {display_name}..."),
        ).await;
        yield_step().await;

        let bytes = {
            let mut file = match zip_archive.by_index(entry_index) {
                Ok(file) => file,
                Err(err) => {
                    error!("Error when reading dex entry: {err:?}");
                    continue;
                }
            };
            // DEX entries are retained by the accessor, so reserve their final
            // uncompressed size once instead of repeatedly growing the buffer.
            let capacity = file.size().min(usize::MAX as u64) as usize;
            let mut bytes = Vec::with_capacity(capacity);
            if let Err(err) = file.read_to_end(&mut bytes) {
                error!("Error when reading {display_name}: {err:?}");
                continue;
            }
            bytes
        };
        dex_inputs.push((display_name.to_owned(), bytes));
        yield_step().await;
    }

    let indexed_dexes = process_dexes(dex_inputs, &sender, &mut yield_step).await;
    let class_capacity = indexed_dexes.iter()
        .map(|dex| dex.classes.len())
        .sum();
    let mut dex_sources = HashMap::with_capacity(indexed_dexes.len());
    let mut map = HashMap::with_capacity(class_capacity);
    for indexed_dex in indexed_dexes {
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

async fn process_dexes<F, Fut>(
    inputs: Vec<(String, Vec<u8>)>, sender: &Sender<ServerMessage>,
    yield_step: &mut F,
) -> Vec<IndexedDex>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = ()>,
{
    let dex_count = inputs.len().max(1);
    let mut indexed_dexes = Vec::with_capacity(inputs.len());
    for (index, (display_name, bytes)) in inputs.into_iter().enumerate() {
        send_progress(
            sender, 0.25 + index as f32 / dex_count as f32 * 0.70,
            format!("Parsing {display_name}..."),
        ).await;
        yield_step().await;

        let Some((file_name, accessor)) = resolve_dex(&display_name, bytes) else {
            continue;
        };
        let class_start = 0.25 + index as f32 / dex_count as f32 * 0.70;
        let class_span = 0.70 / dex_count as f32;
        let classes = index_classes(
            &display_name, &accessor, sender, class_start, class_span, yield_step,
        ).await;
        indexed_dexes.push(IndexedDex { file_name, accessor, classes });
        send_progress(
            sender, 0.25 + (index + 1) as f32 / dex_count as f32 * 0.70,
            format!("Indexed {display_name}..."),
        ).await;
        yield_step().await;
    }
    indexed_dexes
}

async fn index_classes<F, Fut>(
    display_name: &str, accessor: &Arc<DexFileAccessor>, sender: &Sender<ServerMessage>,
    progress_start: f32, progress_span: f32, yield_step: &mut F,
) -> Vec<(DescriptorRef, ClassDef)>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = ()>,
{
    let class_count = accessor.file.class_defs.len();
    let report_step = (class_count / 16).max(64);
    let mut classes = Vec::with_capacity(class_count);
    for (class_index, class_def) in accessor.file.class_defs.iter().enumerate() {
        let class_idx = class_def.class_idx;
        let class_name = accessor.get_type(class_idx);
        if let Ok(class_name) = class_name {
            classes.push((class_name, *class_def));
        } else {
            error!("Error when reading class name {}: {:?}", class_idx, class_name);
        }

        let is_last_class = class_index + 1 == class_count;
        if is_last_class || (class_index + 1) % report_step == 0 {
            let class_progress = (class_index + 1) as f32 / class_count.max(1) as f32;
            send_progress(
                sender,
                progress_start + progress_span * class_progress,
                format!("Indexing {display_name}..."),
            ).await;
            yield_step().await;
        }
    }
    if classes.len() != class_count {
        error!("Some classes in {display_name} could not be indexed");
    }
    classes
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
