use crate::server::OpenFileError;
use crate::Accessor;
use java_asm::dex::{ClassDef, DexFile, DexFileAccessor};
use java_asm::smali::SmaliNode;
use java_asm::{DescriptorRef, StrRef};
use log::{error, warn};
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::rc::Rc;
use zip::ZipArchive;

pub struct ApkAccessor {
    pub map: HashMap<DescriptorRef, ClassPosition>,
}

type ClassPosition = (Rc<DexFileAccessor>, ClassDef);

pub fn read_apk(zip_archive: ZipArchive<impl Read + Seek>) -> Result<ApkAccessor, OpenFileError> {
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
    let dex_files = dex_files.iter().map(|name| {
        let mut file = zip_archive.by_index(*name).map_err(OpenFileError::LoadZip)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(OpenFileError::Io)?;
        let dex_file = DexFile::resolve_from_bytes(&bytes).map_err(OpenFileError::ResolveError)?;
        Ok(Rc::new(DexFileAccessor::new(dex_file, bytes)))
    }).map(|res: Result<Rc<DexFileAccessor>, OpenFileError>| {
        match res {
            Ok(dex_file) => Some(dex_file),
            Err(err) => {
                error!("Error when reading dex file: {:?}", err);
                None
            }
        }
    }).filter_map(|v| v).collect::<Vec<_>>();
    let classes_count = dex_files.iter().map(|dex_file| dex_file.file.class_defs.len()).sum();
    let mut map = HashMap::with_capacity(classes_count);
    for dex_file in dex_files {
        for class_def in dex_file.file.class_defs.iter() {
            let class_idx = class_def.class_idx;
            let class_name = dex_file.get_type(class_idx);
            if let Ok(class_name) = class_name {
                let class_name = Rc::from(class_name);
                let existed = map.get(&class_name);
                if existed.is_none() {
                    map.insert(class_name, (Rc::clone(&dex_file), *class_def));
                }
            } else {
                error!("Error when reading class name {}: {:?}", class_idx, class_name);
            }
        }
    };
    map.shrink_to_fit();
    Ok(ApkAccessor { map })
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
            accessor.get_class_smali(*class_def).ok()
        } else {
            warn!("No class content found for: {}", class_key);
            None
        }
    }
}
