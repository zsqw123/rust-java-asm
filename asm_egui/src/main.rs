use crate::app::EguiApp;
use egui::{IconData, ViewportBuilder};
use image::{ImageFormat, ImageReader};
use std::io::Cursor;

pub mod app;
pub(crate) mod font;
pub(crate) mod file_tree;
pub(crate) mod util;
mod smali;
mod file_tab;
mod top_bar;

fn main() -> eframe::Result {
    let mut image = ImageReader::new(
        Cursor::new(include_bytes!("../../res/icon.png"))
    );
    image.set_format(ImageFormat::Png);
    let image = image.decode().unwrap();
    let width = image.width();
    let height = image.height();
    let decoded = image.to_rgba8().to_vec();
    let icon_data = IconData {
        rgba: decoded,
        width,
        height,
    };
    let viewport = ViewportBuilder::default()
        .with_icon(icon_data);
    let eframe_options = eframe::NativeOptions {
        persist_window: true,
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "asm-gui",
        eframe_options,
        Box::new(|context| Ok(
            Box::new(EguiApp::new(context))
        )),
    )
}
