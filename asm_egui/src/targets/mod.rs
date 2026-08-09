#[cfg(target_family = "wasm")]
pub mod wasm;
#[cfg(not(target_family = "wasm"))]
pub mod native;

#[cfg(target_family = "wasm")]
pub(crate) use wasm::{configure_fonts, open_dropped_file};
#[cfg(not(target_family = "wasm"))]
pub(crate) use native::{configure_fonts, open_dropped_file};

#[cfg(target_family = "wasm")]
pub(crate) fn run() {
    wasm::main();
}

#[cfg(not(target_family = "wasm"))]
pub(crate) fn run() {
    native::main().ok();
}
