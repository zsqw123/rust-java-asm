use crate::impls::server::{ProgressMessage, ServerMessage};
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
    // read dex files
    let mut dex_files = zip_archive
        .file_names()
        .filter(|name|
            // classes dex should be classes.dex or classes*.dex, and not in the sub directory.
            name.starts_with("classes") && name.ends_with(".dex") && !name.contains("/")
        ).collect::<Vec<_>>();
    dex_files.sort_by(|a, b| dex_index(a).cmp(&dex_index(b)));
    
    // read zip entry indices
    let dex_files: Vec<_> = dex_files.iter().map(|s| {
        zip_archive.index_for_name(s)
    }).filter_map(|v|v).collect();
    
    // put dex files
    let dex_file_count = dex_files.len();
    let mut dex_sources = HashMap::new();
    let dex_files = dex_files.iter().map(|entry_index| {
        let mut file = zip_archive.by_index(*entry_index).map_err(OpenFileError::LoadZip)?;
        let file_name = StrRef::from(file.name());
        let file_name_for_dex_sources = file_name.clone();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(OpenFileError::Io)?;
        let dex_file = DexFile::resolve_from_bytes(&bytes).map_err(OpenFileError::ResolveError)?;
        let dex_file_accessor = Arc::new(DexFileAccessor::new(dex_file, bytes, file_name));

        dex_sources.insert(file_name_for_dex_sources, dex_file_accessor.clone());
        Ok(dex_file_accessor)
    }).map(|res: Result<Arc<DexFileAccessor>, OpenFileError>| {
        match res {
            Ok(dex_file) => Some(dex_file),
            Err(err) => {
                error!("Error when reading dex file: {:?}", err);
                None
            }
        }
    }).filter_map(|v| v);
    let mut map = HashMap::new();
    for (index, dex_file) in dex_files.enumerate() {
        for class_def in dex_file.file.class_defs.iter() {
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
        }
        send_progress(&sender, index + 1, dex_file_count).await;
    };
    map.shrink_to_fit();
    send_loaded(&sender).await;
    Ok(ApkAccessor { map, dex_sources })
}

async fn send_progress(
    sender: &Sender<ServerMessage>, current: usize, total: usize,
) {
    let progress = current as f32 / total as f32;
    let message = ServerMessage::Progress(ProgressMessage {
        progress,
        in_loading: true,
    });
    sender.send(message).await.unwrap();
}

async fn send_loaded(
    sender: &Sender<ServerMessage>,
) {
    let message = ServerMessage::Progress(ProgressMessage {
        progress: 1.0,
        in_loading: false,
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
