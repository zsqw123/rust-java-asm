use eframe::epaint::text::{FontData, FontDefinitions};
use eframe::CreationContext;
use egui::FontFamily;
use fontdb::Query;
use log::{error, info};

pub fn inject_sys_font(context: &CreationContext) -> Option<()> {
    let mut fonts = FontDefinitions::empty();

    let mut db = fontdb::Database::new();
    db.load_system_fonts();
    let faces = db.faces().collect::<Vec<_>>();
    info!("system fonts loaded: {:?}", &faces);
    
    let loaded = load_sys_font(&mut fonts, &mut db,
                               &[fontdb::Family::SansSerif], "sans-serif");
    if let Some(loaded) = loaded {
        fonts.families.entry(FontFamily::Proportional).or_default()
            .insert(0, loaded);
    }

    let loaded = load_sys_font(&mut fonts, &mut db,
                               &[fontdb::Family::Monospace, fontdb::Family::SansSerif], "monospace");
    if let Some(loaded) = loaded {
        fonts.families.entry(FontFamily::Monospace).or_default()
            .insert(0, loaded);
    }

    Some(context.egui_ctx.set_fonts(fonts))
}

fn load_sys_font(
    font_definitions: &mut FontDefinitions,
    db: &mut fontdb::Database,
    families: &[fontdb::Family],
    font_name: &str,
) -> Option<String> {
    let mut query = Query::default();
    query.families = families;
    let font_data = db.query(&query)
        .and_then(|id| db.with_face_data(id, |font_data, _| font_data.to_vec()));
    match font_data {
        None => {
            error!("Failed to find system font family {font_name}");
            None
        }
        Some(font_data) => {
            font_definitions.font_data.insert(font_name.into(), FontData::from_owned(font_data));
            info!("font {font_name} loaded");
            Some(font_name.into())
        }
    }
}
