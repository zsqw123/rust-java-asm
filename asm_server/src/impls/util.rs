use std::future::Future;

#[cfg(target_family = "wasm")]
pub fn schedule_task<F: Future<Output=()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

#[cfg(not(target_family = "wasm"))]
pub fn schedule_task<F: Future<Output=()> + Send + 'static>(f: F) {
    std::thread::spawn(move || futures::executor::block_on(f));
}
