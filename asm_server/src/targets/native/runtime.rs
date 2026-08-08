use std::future::Future;
use std::path::Path;

pub(crate) type Instant = std::time::Instant;

pub(crate) fn schedule_task<F: Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || futures::executor::block_on(f));
}

pub(crate) fn file_handle_path(file_handle: &rfd::FileHandle) -> String {
    file_handle.path().display().to_string()
}

pub(crate) fn reveal_parent(path: &Path) {
    let parent_path = path.parent().unwrap_or(path);
    if !parent_path.exists() {
        return;
    }
    open::that_in_background(parent_path);
}
