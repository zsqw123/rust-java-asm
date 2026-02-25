use rfd::{AsyncFileDialog, FileHandle};
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

pub enum ReadAccess {
    FileHandleBased {
        handle: FileHandle,
    },
    PathBased {
        path: PathBuf,
    },
    Raw {
        name: String,
        data: Arc<[u8]>,
    },
}

#[derive(Debug)]
pub enum ReadError {
    IOError {
        err: std::io::Error,
    },
    None,
}

pub struct WriteAccess {
    handle: FileHandle,
}

#[cfg(target_family = "wasm")]
fn guess_path_for_handle(file_handle: &FileHandle) -> String {
    file_handle.file_name()
}

#[cfg(not(target_family = "wasm"))]
fn guess_path_for_handle(file_handle: &FileHandle) -> String {
    file_handle.path().display().to_string()
}

impl ReadAccess {
    pub fn name(&self) -> String {
        match self {
            Self::FileHandleBased { handle } => handle.file_name(),
            Self::Raw { name, .. } => name.clone(),
            Self::PathBased { path } => {
                match path.file_name() {
                    Some(name) => name.display().to_string(),
                    None => path.display().to_string(),
                }
            }
        }
    }

    // in wasm, it will returns name directly rather than path because it's not available.
    pub fn guess_path(&self) -> String {
        match self {
            Self::FileHandleBased { handle } => guess_path_for_handle(handle),
            Self::PathBased { path } => path.display().to_string(),
            _ => self.name(),
        }
    }

    pub async fn new(dialog: AsyncFileDialog) -> Option<Self> {
        let handle = dialog.pick_file().await?;
        Some(Self::FileHandleBased { handle })
    }

    pub fn from_raw(name: String, data: Arc<[u8]>) -> Self {
        Self::Raw { name, data }
    }

    pub fn from_path(path: &PathBuf) -> Self {
        let path = path.clone();
        Self::PathBased { path }
    }

    pub async fn read(&self) -> Result<Arc<[u8]>, ReadError> {
        match self {
            Self::FileHandleBased { handle } => {
                let file_content = handle.read().await;
                Ok(Arc::from(file_content.into_boxed_slice()))
            }
            Self::Raw { data, .. } => {
                Ok(Arc::clone(data))
            }
            Self::PathBased { path } => {
                let mut buf = Vec::new();
                match std::fs::File::open(path) {
                    Ok(mut file) => {
                        match file.read_to_end(&mut buf) {
                            Ok(_) => Ok(Arc::from(buf.into_boxed_slice())),
                            Err(err) => Err(ReadError::IOError { err }),
                        }
                    }
                    Err(err) => Err(ReadError::IOError { err })
                }
            }
        }
    }
}

impl WriteAccess {
    pub fn name(&self) -> String {
        self.handle.file_name()
    }

    // in wasm, it will returns name directly rather than path because it's not available.
    pub fn guess_path(&self) -> PathBuf {
        let path_str = guess_path_for_handle(&self.handle);
        PathBuf::from(path_str)
    }

    pub async fn new(dialog: AsyncFileDialog) -> Option<Self> {
        let handle = dialog.save_file().await?;
        Some(Self { handle })
    }

    pub async fn write(&self, data: &[u8]) -> std::io::Result<()> {
        self.handle.write(data).await
    }
}
