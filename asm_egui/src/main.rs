pub mod app;
mod bottom_panel;
pub(crate) mod file_tree;
pub(crate) mod util;
mod smali;
mod file_tab;
mod top_bar;
mod targets;
mod toast;

fn main() {
    targets::run();
}
