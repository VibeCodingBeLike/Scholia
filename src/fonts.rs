use egui::{FontData, FontDefinitions, FontFamily, FontId};
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
    pub const ARROW_LEFT: &str = "\u{f060}"; //  nf-fa-arrow_left
    pub const ARROW_RIGHT: &str = "\u{f061}"; //  nf-fa-arrow_right
    pub const TITLE_CASE: &str = "\u{f031}"; //  nf-fa-font
    pub const TIMES: &str = "\u{f00d}"; //  nf-fa-times
    pub const UNDO: &str = "\u{f0e2}"; //  nf-fa-undo
    pub const REDO: &str = "\u{f01e}"; //  nf-fa-repeat
}

pub const MLA_DOC_FONT: &str = "TimesNewRoman";

/// Packaged JetBrains Mono Nerd Font directly bundled inside the application binary
pub const EMBEDDED_NERD_FONT: &[u8] =
    include_bytes!("../assets/fonts/JetBrainsMonoNerdFont-Regular.ttf");

/// Times New Roman bundled directly — ensures consistent MLA/PDF output across all machines
pub const EMBEDDED_TIMES_NEW_ROMAN: &[u8] =
    include_bytes!("../assets/fonts/TimesNewRoman-Regular.ttf");

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

    // 2. Bundled Times New Roman - always available, guarantees consistent MLA/PDF rendering
    fonts.font_data.insert(
        MLA_DOC_FONT.to_string(),
        Arc::new(FontData::from_static(EMBEDDED_TIMES_NEW_ROMAN)),
    );

    // 3. Use the Nerd Font for the ENTIRE APP (UI, buttons, menus, dialogs, badges) and icons
    if let Some(prop) = fonts.families.get_mut(&FontFamily::Proportional) {
        prop.insert(0, "NerdFont".to_string());
    }
    if let Some(mono) = fonts.families.get_mut(&FontFamily::Monospace) {
        mono.insert(0, "NerdFont".to_string());
    }

    // 4. Register MLA Document Family: bundled Times New Roman + NerdFont for icon fallback
    let doc_family_list = vec![
        MLA_DOC_FONT.to_string(),
        "NerdFont".to_string(),
    ];
    fonts.families.insert(doc_font_family(), doc_family_list);

    ctx.set_fonts(fonts);
}
