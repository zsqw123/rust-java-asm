mod runtime;

pub(crate) use runtime::{file_handle_path, reveal_parent, schedule_task};
pub use runtime::{Instant, SystemTime};

use crate::impls::apk_load::{
    resolve_dex, ApkAccessor, IndexedDex,
};
use crate::impls::server::ServerMessage;
use crate::server::OpenFileError;
use futures::channel::oneshot::Receiver;
use log::error;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::io::{Read, Seek};
use std::sync::OnceLock;
use tokio::sync::mpsc::Sender;
use zip::ZipArchive;

const DEX_WORKER_THREADS: usize = 16;

fn dex_worker_pool() -> &'static ThreadPool {
    static POOL: OnceLock<ThreadPool> = OnceLock::new();
    POOL.get_or_init(|| {
        ThreadPoolBuilder::new()
            .num_threads(DEX_WORKER_THREADS)
            .thread_name(|index| format!("dex-worker-{index}"))
            .build()
            .unwrap_or_else(|error| panic!("failed to create DEX worker pool: {error}"))
    })
}

pub(crate) async fn read_apk(
    zip_archive: ZipArchive<impl Read + Seek>, sender: Sender<ServerMessage>,
) -> Result<ApkAccessor, OpenFileError> {
    crate::impls::apk_load::read_apk(zip_archive, sender, || async {}).await
}

pub(crate) fn spawn_process_dex<F>(
    _task_index: usize, display_name: String, bytes: Vec<u8>,
    _yield_step: F,
) -> Receiver<Option<IndexedDex>>
{
    let (sender, receiver) = futures::channel::oneshot::channel();
    dex_worker_pool().spawn(move || {
        let _ = sender.send(process_dex_sync(&display_name, bytes));
    });
    receiver
}

fn process_dex_sync(
    display_name: &str, bytes: Vec<u8>,
) -> Option<IndexedDex> {
    let (file_name, accessor) = resolve_dex(display_name, bytes)?;
    let class_count = accessor.file.class_defs.len();
    let mut classes = Vec::with_capacity(class_count);
    for class_def in &accessor.file.class_defs {
        let class_idx = class_def.class_idx;
        let class_name = accessor.get_type(class_idx);
        if let Ok(class_name) = class_name {
            classes.push((class_name, *class_def));
        } else {
            error!("Error when reading class name {}: {:?}", class_idx, class_name);
        }
    }
    if classes.len() != class_count {
        error!("Some classes in {display_name} could not be indexed");
    }
    Some(IndexedDex { file_name, accessor, classes })
}
