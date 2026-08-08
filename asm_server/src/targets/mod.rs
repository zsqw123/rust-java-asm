// Keep target selection in this module; target implementations stay below the
// native/wasm directories and expose only shared runtime operations.
#[cfg(target_family = "wasm")]
mod wasm;
#[cfg(not(target_family = "wasm"))]
mod native;

#[cfg(target_family = "wasm")]
pub(crate) use wasm::{file_handle_path, read_apk, reveal_parent, schedule_task, spawn_process_dex};
#[cfg(not(target_family = "wasm"))]
pub(crate) use native::{file_handle_path, read_apk, reveal_parent, schedule_task, spawn_process_dex};

#[cfg(target_family = "wasm")]
pub use wasm::{Instant, SystemTime};
#[cfg(not(target_family = "wasm"))]
pub use native::{Instant, SystemTime};
pub use std::time::Duration;
