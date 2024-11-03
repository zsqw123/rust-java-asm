use fontdb::Style;
use log::warn;
use std::collections::HashMap;

pub struct FontFallbacks {
    families: HashMap<String, fontdb::ID>,
    style: Style,
}

pub type FontData = (&'static str, Vec<u8>);

impl FontFallbacks {
    pub const MONO: &'static str = "Consolas";
    pub const FAST: &'static str = "Segoe UI";

    pub const ALL: &'static [&'static str] = &[
        FontFallbacks::FAST,
        // CJK
        "Microsoft YaHei UI", "Microsoft JhengHei UI", "Yu Gothic UI", "Malgun Gothic",
        // some special characters.
        "Segoe UI Emoji", "Segoe UI Symbol", "Segoe UI Historic",
    ];

    #[inline]
    fn needed_font(family: &str) -> bool {
        Self::MONO == family ||
            Self::ALL.iter().any(|&name| name == family)
    }

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
        FontFallbacks { families, style }
    }

    pub fn load_mono(&self, db: &fontdb::Database) -> Option<FontData> {
        self.load_font(db, FontFallbacks::MONO)
    }

    pub fn load_fast(&self, db: &fontdb::Database) -> Option<FontData> {
        self.load_font(db, FontFallbacks::FAST)
    }

    pub fn load_all(&self, db: &fontdb::Database) -> Vec<FontData> {
        FontFallbacks::ALL.iter().filter_map(|name| {
            match self.load_font(db, name) {
                None => {
                    warn!("Failed to find system font family: {}", name);
                    None
                }
                Some(data) => Some(data)
            }
        }).collect()
    }

    #[inline]
    fn load_font(&self, db: &fontdb::Database, name: &'static str) -> Option<FontData> {
        let id = self.families.get(name)?;
        let font_data = db.with_face_data(*id, |font_data, _| font_data.to_vec())?;
        Some((name, font_data))
    }
}
