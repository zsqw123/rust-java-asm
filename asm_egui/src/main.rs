pub mod app;
pub(crate) mod file_tree;
pub(crate) mod util;
mod smali;
mod file_tab;
mod top_bar;
mod targets;

fn main() {
    targets::run();
}
