use egui::{FontData, FontDefinitions, FontFamily, FontId};
use std::path::PathBuf;
use std::sync::Arc;

pub mod icons {
    // Standard Nerd Font Icons (PUA glyphs)
    pub const FILE_NEW: &str = "\u{f016}"; //  nf-fa-file_o
    pub const FOLDER_OPEN: &str = "\u{f07c}"; //  nf-fa-folder_open
    pub const SAVE: &str = "\u{f0c7}"; //  nf-fa-save
    pub const WORD_DOCX: &str = "\u{f1c2}"; //  nf-fa-file_word_o
    pub const HTML_PDF: &str = "\u{f1c1}"; //  nf-fa-file_pdf_o
    pub const TEXT: &str = "\u{f0f6}"; //  nf-fa-file_text_o
    pub const BOOK_CITATIONS: &str = "\u{f02d}"; //  nf-fa-book
    pub const QUOTE: &str = "\u{f10d}"; //  nf-fa-quote_left
    pub const CHECK_COMPLIANCE: &str = "\u{f058}"; //  nf-fa-check_circle
    pub const SETTINGS: &str = "\u{f013}"; //  nf-fa-cog
    pub const FOCUS_MODE: &str = "\u{f06e}"; //  nf-fa-eye
    pub const CALENDAR: &str = "\u{f073}"; //  nf-fa-calendar
    pub const PLUS: &str = "\u{f067}"; //  nf-fa-plus
    pub const EDIT: &str = "\u{f040}"; //  nf-fa-pencil
    pub const TRASH: &str = "\u{f1f8}"; //  nf-fa-trash
    pub const PARAGRAPH: &str = "\u{f1dd}"; //  nf-fa-paragraph
    pub const HEADING: &str = "\u{f1dc}"; //  nf-fa-header
    pub const INFO: &str = "\u{f05a}"; //  nf-fa-info_circle
    pub const SPARKLE: &str = "\u{f005}"; //  nf-fa-star
    pub const WARNING: &str = "\u{f071}"; //  nf-fa-exclamation_triangle
    pub const CHECK: &str = "\u{f00c}"; //  nf-fa-check
    pub const ARROW_UP: &str = "\u{f062}"; //  nf-fa-arrow_up
    pub const ARROW_DOWN: &str = "\u{f063}"; //  nf-fa-arrow_down
    pub const TITLE_CASE: &str = "\u{f031}"; //  nf-fa-font
    pub const TIMES: &str = "\u{f00d}"; //  nf-fa-times
}

pub const MLA_DOC_FONT: &str = "TimesNewRoman";

/// Packaged JetBrains Mono Nerd Font directly bundled inside the application binary
pub const EMBEDDED_NERD_FONT: &[u8] =
    include_bytes!("../assets/fonts/JetBrainsMonoNerdFont-Regular.ttf");

pub fn doc_font_family() -> FontFamily {
    FontFamily::Name(Arc::from(MLA_DOC_FONT))
}

pub fn doc_font(size: f32) -> FontId {
    FontId::new(size, doc_font_family())
}

/// Load packaged Nerd Font and Times New Roman into egui
pub fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    // 1. Packaged JetBrains Mono Nerd Font - Always available and self-contained
    fonts.font_data.insert(
        "NerdFont".to_string(),
        Arc::new(FontData::from_static(EMBEDDED_NERD_FONT)),
    );

    // 2. Locate and load Times New Roman for MLA paper manuscript if available
    let mut times_loaded = false;
    if let Some(times_path) = find_times_new_roman() {
        if let Ok(bytes) = std::fs::read(&times_path) {
            fonts.font_data.insert(
                MLA_DOC_FONT.to_string(),
                Arc::new(FontData::from_owned(bytes)),
            );
            times_loaded = true;
        }
    }

    // 3. Use the Nerd Font for the ENTIRE APP (UI, buttons, menus, dialogs, badges) and icons
    if let Some(prop) = fonts.families.get_mut(&FontFamily::Proportional) {
        prop.insert(0, "NerdFont".to_string());
    }
    if let Some(mono) = fonts.families.get_mut(&FontFamily::Monospace) {
        mono.insert(0, "NerdFont".to_string());
    }

    // 4. Register MLA Document Family (Times New Roman with NerdFont fallback for symbols/icons)
    let mut doc_family_list = Vec::new();
    if times_loaded {
        doc_family_list.push(MLA_DOC_FONT.to_string());
    }
    doc_family_list.push("NerdFont".to_string());
    doc_family_list.push("Hack".to_string());
    fonts.families.insert(doc_font_family(), doc_family_list);

    ctx.set_fonts(fonts);
}

fn find_times_new_roman() -> Option<PathBuf> {
    let mut candidates = vec![
        PathBuf::from(r"C:\Windows\Fonts\times.ttf"),
        PathBuf::from(r"C:\Windows\Fonts\timesi.ttf"),
    ];

    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        candidates.push(PathBuf::from(&local).join(r"Microsoft\Windows\Fonts\times.ttf"));
        candidates.push(
            PathBuf::from(&local).join(r"Microsoft\Windows\Fonts\TimesNewerRoman-Regular.otf"),
        );
    }

    // macOS
    candidates.push(PathBuf::from(
        "/System/Library/Fonts/Supplemental/Times New Roman.ttf",
    ));
    candidates.push(PathBuf::from("/Library/Fonts/Times New Roman.ttf"));

    // Linux
    candidates.push(PathBuf::from(
        "/usr/share/fonts/truetype/msttcorefonts/Times_New_Roman.ttf",
    ));
    candidates.push(PathBuf::from("/usr/share/fonts/TTF/times.ttf"));
    candidates.push(PathBuf::from(
        "/usr/share/fonts/truetype/liberation/LiberationSerif-Regular.ttf",
    ));
    candidates.push(PathBuf::from(
        "/usr/share/fonts/truetype/freefont/FreeSerif.ttf",
    ));

    for path in &candidates {
        if path.exists() && path.is_file() {
            return Some(path.clone());
        }
    }
    None
}
