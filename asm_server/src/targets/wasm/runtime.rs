use js_sys::Promise;
use std::future::Future;
use std::path::Path;
use wasm_bindgen_futures::JsFuture;

pub(crate) type Instant = web_time::Instant;

pub(crate) fn schedule_task<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

pub(crate) fn file_handle_path(file_handle: &rfd::FileHandle) -> String {
    file_handle.file_name()
}

pub(crate) fn reveal_parent(_path: &Path) {}

pub(super) async fn yield_to_browser() {
    let promise = Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("No window");
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0)
            .expect("failed to schedule browser yield");
    });
    let _ = JsFuture::from(promise).await;
}
