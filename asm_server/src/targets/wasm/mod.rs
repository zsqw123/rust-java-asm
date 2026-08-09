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
use std::future::Future;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Sender;

pub(crate) async fn read_dex_inputs(
    inputs: Vec<(String, Vec<u8>)>, sender: Sender<ServerMessage>,
) -> Result<DexAccessor, OpenFileError> {
    crate::impls::apk_load::read_dex_inputs(
        inputs, sender, runtime::yield_to_browser,
    ).await
}

pub(crate) fn spawn_process_dex<F, Fut>(
    _task_index: usize, display_name: String, bytes: Vec<u8>,
    yield_step: F, reporter: Arc<Mutex<ProgressReporter>>,
) -> Receiver<Option<IndexedDex>>
where
    F: Fn() -> Fut + Clone + 'static,
    Fut: Future<Output = ()> + 'static,
{
    let (sender, receiver) = futures::channel::oneshot::channel();
    wasm_bindgen_futures::spawn_local(async move {
        let result = process_dex(
            _task_index, display_name, bytes, yield_step, reporter,
        ).await;
        let _ = sender.send(result);
    });
    receiver
}

async fn process_dex<F, Fut>(
    task_index: usize, display_name: String, bytes: Vec<u8>,
    yield_step: F, reporter: Arc<Mutex<ProgressReporter>>,
) -> Option<IndexedDex>
where
    F: Fn() -> Fut + Clone,
    Fut: Future<Output = ()>,
{
    let (file_name, accessor) = resolve_dex(&display_name, bytes)?;
    yield_step().await;

    let class_count = accessor.file.class_defs.len();
    let report_step = (class_count / 100).max(1);
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
            yield_step().await;
        }
    }
    if classes.len() != class_count {
        error!("Some classes in {display_name} could not be indexed");
    }
    Some(IndexedDex { file_name, accessor, classes })
}
