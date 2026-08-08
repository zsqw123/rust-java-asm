use crate::impls::server::{ProgressMessage, ServerMessage};
use crate::compat::yield_to_browser;
use crate::server::OpenFileError;
use crate::{Accessor, ExportableSource};
use java_asm::dex::{ClassDef, DexFile, DexFileAccessor};
use java_asm::smali::{stb, SmaliNode, SmaliToken};
use java_asm::{DescriptorRef, StrRef};
use log::{error, warn};
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use zip::ZipArchive;

pub struct ApkAccessor {
    pub map: HashMap<DescriptorRef, ClassPosition>,
    pub dex_sources: HashMap<StrRef, Arc<DexFileAccessor>>,
}

type ClassPosition = (Arc<DexFileAccessor>, ClassDef);

pub async fn read_apk(
    zip_archive: ZipArchive<impl Read + Seek>, sender: Sender<ServerMessage>,
) -> Result<ApkAccessor, OpenFileError> {
    let mut zip_archive = zip_archive;
    send_progress(&sender, 0.0, "Scanning APK...").await;
    yield_to_browser().await;

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
    let mut dex_sources = HashMap::new();
    let mut map = HashMap::new();

    for (index, (entry_index, display_name)) in dex_files.into_iter().enumerate() {
        let dex_start = index as f32 / dex_file_count.max(1) as f32;
        let dex_span = 1.0 / dex_file_count.max(1) as f32;

        send_progress(
            &sender,
            dex_start + dex_span * 0.05,
            format!("Reading {display_name}..."),
        ).await;
        yield_to_browser().await;

        let (file_name, bytes) = {
            let mut file = match zip_archive.by_index(entry_index) {
                Ok(file) => file,
                Err(err) => {
                    error!("Error when reading dex entry: {err:?}");
                    continue;
                }
            };
            let file_name = StrRef::from(file.name());
            let mut bytes = Vec::new();
            if let Err(err) = file.read_to_end(&mut bytes) {
                error!("Error when reading {file_name}: {err:?}");
                continue;
            }
            (file_name, bytes)
        };
        let file_name_for_dex_sources = file_name.clone();

        send_progress(
            &sender,
            dex_start + dex_span * 0.25,
            format!("Parsing {display_name}..."),
        ).await;
        yield_to_browser().await;

        let dex_file = match DexFile::resolve_from_bytes(&bytes) {
            Ok(dex_file) => dex_file,
            Err(err) => {
                error!("Error when resolving {display_name}: {err:?}");
                continue;
            }
        };
        let dex_file = Arc::new(DexFileAccessor::new(dex_file, bytes, file_name));
        dex_sources.insert(file_name_for_dex_sources, dex_file.clone());

        send_progress(
            &sender,
            dex_start + dex_span * 0.60,
            format!("Indexing {display_name}..."),
        ).await;
        yield_to_browser().await;

        let class_count = dex_file.file.class_defs.len().max(1);
        let report_step = (class_count / 16).max(64);
        for (class_index, class_def) in dex_file.file.class_defs.iter().enumerate() {
            let class_idx = class_def.class_idx;
            let class_name = dex_file.get_type(class_idx);
            if let Ok(class_name) = class_name {
                let class_name = Arc::from(class_name);
                let existed = map.get(&class_name);
                if existed.is_none() {
                    // dex index is the priority, the lower the index, the higher the priority.
                    // if two classes have the same name, the one with the lower index will be kept.
                    map.insert(class_name, (Arc::clone(&dex_file), *class_def));
                }
            } else {
                error!("Error when reading class name {}: {:?}", class_idx, class_name);
            }

            let is_last_class = class_index + 1 == class_count;
            if is_last_class || class_index % report_step == 0 {
                let class_progress = (class_index + 1) as f32 / class_count as f32;
                send_progress(
                    &sender,
                    dex_start + dex_span * (0.60 + class_progress * 0.35),
                    format!("Indexing {display_name}..."),
                ).await;
                yield_to_browser().await;
            }
        }
    };
    map.shrink_to_fit();
    send_loaded(&sender, "APK loaded").await;
    Ok(ApkAccessor { map, dex_sources })
}

async fn send_progress(
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
