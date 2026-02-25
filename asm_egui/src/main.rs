pub mod app;
pub(crate) mod font;
pub(crate) mod file_tree;
pub(crate) mod util;
mod smali;
mod file_tab;
mod top_bar;
mod targets;

fn main() {
    #[cfg(target_family = "wasm")]
    targets::wasm::main();

    #[cfg(not(target_family = "wasm"))]
    targets::native::main().ok();
}

