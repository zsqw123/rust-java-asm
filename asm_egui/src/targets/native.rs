use crate::app::EguiApp;
use eframe::epaint::text::{FontData, FontDefinitions};
use eframe::CreationContext;
use egui::{IconData, ViewportBuilder};
use image::{ImageFormat, ImageReader};
use java_asm_server::ui::font::FontFallbacks;
use log::info;
use std::collections::{BTreeMap, HashSet};
use std::io::Cursor;
use std::sync::Arc;
use std::time::Instant;

fn insert_font_into_definitions(
    font_map: &mut BTreeMap<String, Arc<FontData>>,
    font_name: &str, font_data: Vec<u8>,
) {
    let egui_font_data = FontData::from_owned(font_data);
    font_map.insert(font_name.into(), Arc::new(egui_font_data));
}

pub(crate) fn configure_fonts(context: &CreationContext) {
    let mut fonts = FontDefinitions::empty();

    let mut db = fontdb::Database::new();
    let start = Instant::now();
    db.load_system_fonts();
    let mut families: HashSet<String> = HashSet::new();
    for face_info in db.faces() {
        let families_for_single_face = &face_info.families;
        for (family, _) in families_for_single_face {
            families.insert(family.to_string());
        }
    }
    let families_print_to_str = families.iter().map(|s| s.as_str())
        .collect::<Vec<&str>>().join(", ");
    let families_print_to_str = format!("[{families_print_to_str}]");
    info!("system fonts loaded in {}ms: {families_print_to_str}", start.elapsed().as_millis());

    let start = Instant::now();
    let font_fallbacks = FontFallbacks::new(&db);
    let all_font = font_fallbacks.load_all(&db);

    let mut normal_font_names = Vec::with_capacity(all_font.len());
    for font_data in all_font {
        insert_font_into_definitions(&mut fonts.font_data, font_data.0, font_data.1);
        normal_font_names.push(font_data.0);
    }

    let normal_entry = fonts.families.entry(egui::FontFamily::Proportional).or_default();
    for font_name in &normal_font_names {
        normal_entry.push(font_name.to_string());
    }
    let mono_entry = fonts.families.entry(egui::FontFamily::Monospace).or_default();
    if let Some(mono_font) = font_fallbacks.load_mono(&db) {
        insert_font_into_definitions(&mut fonts.font_data, mono_font.0, mono_font.1);
        mono_entry.insert(0, mono_font.0.to_string());
    }
    for font_name in &normal_font_names {
        mono_entry.push(font_name.to_string());
    }
    info!("default font families loaded in {}ms", start.elapsed().as_millis());

    context.egui_ctx.set_fonts(fonts);
}

pub fn main() -> eframe::Result {
    let mut image = ImageReader::new(
        Cursor::new(include_bytes!("../../../res/icon.png"))
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
