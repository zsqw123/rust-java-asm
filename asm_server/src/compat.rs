//! Small platform adapters shared by the server's native and browser paths.
//!
//! Keep platform checks here so parsing, loading, and UI state code can use a
//! single API without dropping native behavior just to make WASM compile.

use std::future::Future;
use std::path::Path;

#[cfg(not(target_family = "wasm"))]
pub(crate) use std::time::Instant;
#[cfg(target_family = "wasm")]
pub(crate) use web_time::Instant;

#[cfg(target_family = "wasm")]
pub(crate) fn schedule_task<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

#[cfg(not(target_family = "wasm"))]
pub(crate) fn schedule_task<F: Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || futures::executor::block_on(f));
}

/// Give browser rendering a chance between CPU-heavy loading phases.
#[cfg(target_family = "wasm")]
pub(crate) async fn yield_to_browser() {
    use js_sys::Promise;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::window;

    let promise = Promise::new(&mut |resolve, _reject| {
        let window = window().expect("No window");
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0)
            .expect("failed to schedule browser yield");
    });
    let _ = JsFuture::from(promise).await;
}

#[cfg(not(target_family = "wasm"))]
pub(crate) async fn yield_to_browser() {}

pub(crate) fn file_handle_path(file_handle: &rfd::FileHandle) -> String {
    #[cfg(target_family = "wasm")]
    {
        file_handle.file_name()
    }
    #[cfg(not(target_family = "wasm"))]
    {
        file_handle.path().display().to_string()
    }
}

pub(crate) fn reveal_parent(path: &Path) {
    #[cfg(not(target_family = "wasm"))]
    {
        let parent_path = path.parent().unwrap_or(path);
        if !parent_path.exists() {
            return;
        }
        open::that_in_background(parent_path);
    }
    #[cfg(target_family = "wasm")]
    {
        let _ = path;
    }
}
