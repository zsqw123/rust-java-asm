use std::fs::read;
use eframe::CreationContext;
use eframe::epaint::FontFamily;
use eframe::epaint::text::{FontData, FontDefinitions};
use font_kit::family_name::FamilyName;
use font_kit::handle::Handle;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use log::error;

pub fn inject_sys_font(context: &CreationContext) -> Option<()> {
    let mut fonts = FontDefinitions::empty();
    let sans_font = load_sys_font(FamilyName::SansSerif);
    let mono_font = load_sys_font(FamilyName::Monospace)
        .or_else(|| sans_font.clone());

    let sans_name = "sans-serif";
    let mono_name = "monospace";
    fonts.font_data.insert(sans_name.to_owned(), FontData::from_owned(sans_font?));
    fonts.font_data.insert(mono_name.to_owned(), FontData::from_owned(mono_font?));

    fonts.families.entry(FontFamily::Proportional).or_default()
        .insert(0, sans_name.to_owned());
    fonts.families.entry(FontFamily::Monospace).or_default()
        .insert(0, mono_name.to_owned());
    Some(context.egui_ctx.set_fonts(fonts))
}

fn load_sys_font(family_name: FamilyName) -> Option<Vec<u8>> {
    let cloned_name = family_name.clone();
    let handle = SystemSource::new()
        .select_best_match(&[family_name], &Properties::new())
        .map(|h| match h {
            Handle::Path { path, .. } => { read(path) }
            Handle::Memory { bytes, .. } => Ok(bytes.to_vec())
        });

    match handle {
        Ok(res) => match res {
            Ok(res) => Some(res),
            Err(e) => {
                error!("Failed to load system font family {cloned_name:?} due to io error: {e}");
                None
            }
        },
        Err(e) => {
            error!("Failed to find system font family {cloned_name:?} with font-kit: {e}");
            None
        }
    }
}
