use crate::app::AsmApp;

pub mod app;
pub(crate) mod font;
pub(crate) mod file_tree;
pub(crate) mod util;

fn main() -> eframe::Result {
    let eframe_options = eframe::NativeOptions {
        ..Default::default()
    };
    eframe::run_native(
        "asm-gui",
        eframe_options,
        Box::new(|context| Ok(
            Box::new(AsmApp::new(context))
        )),
    )
}
