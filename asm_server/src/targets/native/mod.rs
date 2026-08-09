mod runtime;

pub(crate) use runtime::{file_handle_path, reveal_parent, schedule_task};
pub use runtime::{Instant, SystemTime};

use crate::impls::apk_load::{
    report_dex_progress, resolve_dex, DexAccessor, IndexedDex, ProgressReporter,
};
use crate::impls::server::ServerMessage;
use crate::server::OpenFileError;
use futures::channel::oneshot::Receiver;
use log::error;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::mpsc::Sender;

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

pub(crate) async fn read_dex_inputs(
    inputs: Vec<(String, Vec<u8>)>, sender: Sender<ServerMessage>,
) -> Result<DexAccessor, OpenFileError> {
    crate::impls::apk_load::read_dex_inputs(inputs, sender, || async {}).await
}

pub(crate) fn spawn_process_dex<F>(
    _task_index: usize, display_name: String, bytes: Vec<u8>,
    _yield_step: F, reporter: Arc<Mutex<ProgressReporter>>,
) -> Receiver<Option<IndexedDex>>
{
    let (sender, receiver) = futures::channel::oneshot::channel();
    dex_worker_pool().spawn(move || {
        let result = process_dex_sync(
            _task_index, &display_name, bytes, reporter,
        );
        let _ = sender.send(result);
    });
    receiver
}

fn process_dex_sync(
    task_index: usize, display_name: &str, bytes: Vec<u8>,
    reporter: Arc<Mutex<ProgressReporter>>,
) -> Option<IndexedDex> {
    let (file_name, accessor) = resolve_dex(display_name, bytes)?;
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
            let progress = (class_index + 1) as f32 / class_count.max(1) as f32;
            report_dex_progress(
                &reporter, task_index, progress,
                || format!("Indexing {display_name}..."),
            );
        }
    }
    if classes.len() != class_count {
        error!("Some classes in {display_name} could not be indexed");
    }
    Some(IndexedDex { file_name, accessor, classes })
}
