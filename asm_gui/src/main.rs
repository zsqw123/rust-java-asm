use crate::app::AsmApp;

pub mod app;

fn main() -> eframe::Result {
    let eframe_options = eframe::NativeOptions {
        ..Default::default()
    };
    eframe::run_native(
        "asm-gui",
        eframe_options,
        Box::new(|_| Ok(
            Box::new(AsmApp::default())
        )),
    )
}
