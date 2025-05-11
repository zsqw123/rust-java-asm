use eframe::epaint::text::{FontData, FontDefinitions};
use eframe::CreationContext;
use egui::FontFamily;
use java_asm_server::ui::font::FontFallbacks;
use log::info;
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

pub fn inject_sys_font(context: &CreationContext) -> Option<()> {
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

    let normal_entry = fonts.families.entry(FontFamily::Proportional).or_default();
    for font_name in &normal_font_names {
        normal_entry.push(font_name.to_string());
    }
    let mono_entry = fonts.families.entry(FontFamily::Monospace).or_default();
    if let Some(mono_font) = font_fallbacks.load_mono(&db) {
        insert_font_into_definitions(&mut fonts.font_data, mono_font.0, mono_font.1);
        mono_entry.insert(0, mono_font.0.to_string());
    }
    for font_name in &normal_font_names {
        mono_entry.push(font_name.to_string());
    }
    info!("default font families loaded in {}ms", start.elapsed().as_millis());

    Some(context.egui_ctx.set_fonts(fonts))
}

fn insert_font_into_definitions(
    font_map: &mut BTreeMap<String, Arc<FontData>>, font_name: &str, font_data: Vec<u8>,
) {
    let egui_font_data = FontData::from_owned(font_data);
    font_map.insert(font_name.into(), Arc::new(egui_font_data));
}
