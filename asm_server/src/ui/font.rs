use fontdb::Style;
use log::warn;
use std::collections::HashMap;

pub struct FontFallbacks {
    families: HashMap<String, fontdb::ID>,
}

/// font family name and owned font data.
pub type FontData = (&'static str, Vec<u8>);

impl FontFallbacks {
    #[cfg(not(target_os = "macos"))]
    pub const MONO: &'static str = "Consolas";

    #[cfg(target_os = "macos")]
    pub const MONO: &'static str = "Menlo";

    #[cfg(not(target_os = "macos"))]
    pub const FALLBACKS: &'static [&'static str] = &[
        // some special characters.
        "Segoe UI Emoji", "Segoe UI Symbol", "Segoe UI Historic",
    ];

    #[cfg(target_os = "macos")]
    pub const FALLBACKS: &'static [&'static str] = &[
        // some special characters.
        "Apple Color Emoji",
    ];

    #[cfg(not(target_os = "macos"))]
    pub const NORMAL: &'static [&'static str] = &[
        "Segoe UI",
        // CJK
        "Microsoft YaHei UI", "Microsoft JhengHei UI", "Yu Gothic UI", "Malgun Gothic",
    ];

    #[cfg(target_os = "macos")]
    pub const NORMAL: &'static [&'static str] = &[
        "Arial", ".SF NS", "Menlo", "Geneva", "Arial Unicode MS",
        // CJK
        "PingFang SC", "PingFang HK", "PingFang TC", "Apple SD Gothic Neo", "Hiragino Sans",
    ];

    #[inline]
    fn needed_font(family: &str) -> bool {
        Self::MONO == family ||
            Self::NORMAL.iter().any(|&name| name == family) ||
            Self::FALLBACKS.iter().any(|&name| name == family)
    }

    #[inline]
    pub fn new(db: &fontdb::Database) -> Self {
        Self::from_style(db, Style::Normal)
    }

    pub fn from_style(db: &fontdb::Database, style: Style) -> Self {
        let mut families = HashMap::new();
        for face_info in db.faces() {
            let id = face_info.id;
            let face_style = face_info.style;
            if face_style != style { continue; }
            let families_for_single_face = &face_info.families;
            for (family, _) in families_for_single_face {
                if !Self::needed_font(family) { continue; }
                families.insert(family.to_string(), id);
            }
        }
        FontFallbacks { families }
    }

    pub fn load_mono(&self, db: &fontdb::Database) -> Option<FontData> {
        self.load_font(db, FontFallbacks::MONO)
    }

    pub fn load_all(&self, db: &fontdb::Database) -> Vec<FontData> {
        // load normal
        let mut fonts: Vec<FontData> = FontFallbacks::NORMAL.iter().filter_map(|name| {
            match self.load_font(db, name) {
                None => {
                    warn!("Failed to find system font family: {}", name);
                    None
                }
                Some(data) => Some(data)
            }
        }).collect();
        // load customized
        if cfg!(target_os = "macos") {
            if let Some(emoji) = load_macos_emoji() {
                fonts.push(emoji);
            }
        }
        // load fallbacks
        for name in FontFallbacks::FALLBACKS {
            if let Some(font) = self.load_font(db, name) {
                fonts.push(font);
            } else {
                warn!("Failed to find system font family as a fallback: {}", name);
            }
        }
        fonts
    }

    #[inline]
    fn load_font(&self, db: &fontdb::Database, name: &'static str) -> Option<FontData> {
        let id = self.families.get(name)?;
        let font_data = db.with_face_data(*id, |font_data, _| font_data.to_vec())?;
        Some((name, font_data))
    }
}
fn load_macos_emoji() -> Option<FontData> {
    let bytes = include_bytes!("../fonts/NotoEmoji-Regular.ttf");
    Some(("NotoEmoji", bytes.to_vec()))
}
