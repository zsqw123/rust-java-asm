#[cfg(target_family = "wasm")]
pub mod wasm;
#[cfg(not(target_family = "wasm"))]
pub mod native;
