use crate::app::AsmApp;

pub mod app;
mod font;

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
