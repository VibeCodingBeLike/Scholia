use egui::Key;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    NewDocument,
    OpenDocument,
    SaveDocument,
    ExportDocx,
    ExportHtmlPdf,
    AddParagraph,
    InsertBlockQuote,
    InsertHeading1,
    InsertHeading2,
    InsertHeading3,
    InsertCitation,
    DeleteBlock,
    MoveBlockUp,
    MoveBlockDown,
    ManageWorksCited,
    ConvertToMlaTitleCase,
    OpenPreferences,
    ToggleComplianceCheck,
    ToggleFocusMode,
}

impl Action {
    pub fn display_name(&self) -> &'static str {
        match self {
            Action::NewDocument => "New MLA Document",
            Action::OpenDocument => "Open Document",
            Action::SaveDocument => "Save Document (.mladoc)",
            Action::ExportDocx => "Export to Word (.docx)",
            Action::ExportHtmlPdf => "Export to Printable HTML / PDF",
            Action::AddParagraph => "Add New Body Paragraph",
            Action::InsertBlockQuote => "Insert MLA Block Quote (>4 lines)",
            Action::InsertHeading1 => "Insert Section Heading 1 (Bold)",
            Action::InsertHeading2 => "Insert Section Heading 2 (Italics)",
            Action::InsertHeading3 => "Insert Section Heading 3 (Centered)",
            Action::InsertCitation => "Insert In-Text Parenthetical Citation",
            Action::DeleteBlock => "Delete Active Block",
            Action::MoveBlockUp => "Move Active Block Up",
            Action::MoveBlockDown => "Move Active Block Down",
            Action::ManageWorksCited => "Open Works Cited Manager",
            Action::ConvertToMlaTitleCase => "Format Title to MLA Title Case",
            Action::OpenPreferences => "Theme & Transparency Settings",
            Action::ToggleComplianceCheck => "Run MLA Compliance Inspector",
            Action::ToggleFocusMode => "Toggle Zen / Focus Mode",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            Action::NewDocument
            | Action::OpenDocument
            | Action::SaveDocument
            | Action::ExportDocx
            | Action::ExportHtmlPdf => "File & Export",
            Action::AddParagraph
            | Action::InsertBlockQuote
            | Action::InsertHeading1
            | Action::InsertHeading2
            | Action::InsertHeading3
            | Action::InsertCitation
            | Action::DeleteBlock
            | Action::MoveBlockUp
            | Action::MoveBlockDown
            | Action::ConvertToMlaTitleCase => "Editing & MLA Blocks",
            Action::ManageWorksCited => "Works Cited",
            Action::OpenPreferences | Action::ToggleComplianceCheck | Action::ToggleFocusMode => {
                "View & Tools"
            }
        }
    }

    pub fn all() -> &'static [Action] {
        &[
            Action::NewDocument,
            Action::OpenDocument,
            Action::SaveDocument,
            Action::ExportDocx,
            Action::ExportHtmlPdf,
            Action::AddParagraph,
            Action::InsertBlockQuote,
            Action::InsertHeading1,
            Action::InsertHeading2,
            Action::InsertHeading3,
            Action::InsertCitation,
            Action::DeleteBlock,
            Action::MoveBlockUp,
            Action::MoveBlockDown,
            Action::ManageWorksCited,
            Action::ConvertToMlaTitleCase,
            Action::OpenPreferences,
            Action::ToggleComplianceCheck,
            Action::ToggleFocusMode,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shortcut {
    pub ctrl: bool, // or Cmd on mac
    pub shift: bool,
    pub alt: bool,
    pub key: KeyName,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyName {
    N,
    O,
    S,
    E,
    B,
    C,
    L,
    W,
    T,
    V,
    P,
    Comma,
    Enter,
    F11,
    Num1,
    Num2,
    Num3,
    Delete,
    Backspace,
    Up,
    Down,
}

impl KeyName {
    pub fn to_egui_key(self) -> Key {
        match self {
            KeyName::N => Key::N,
            KeyName::O => Key::O,
            KeyName::S => Key::S,
            KeyName::E => Key::E,
            KeyName::B => Key::B,
            KeyName::C => Key::C,
            KeyName::L => Key::L,
            KeyName::W => Key::W,
            KeyName::T => Key::T,
            KeyName::V => Key::V,
            KeyName::P => Key::P,
            KeyName::Comma => Key::Comma,
            KeyName::Enter => Key::Enter,
            KeyName::F11 => Key::F11,
            KeyName::Num1 => Key::Num1,
            KeyName::Num2 => Key::Num2,
            KeyName::Num3 => Key::Num3,
            KeyName::Delete => Key::Delete,
            KeyName::Backspace => Key::Backspace,
            KeyName::Up => Key::ArrowUp,
            KeyName::Down => Key::ArrowDown,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            KeyName::N => "N",
            KeyName::O => "O",
            KeyName::S => "S",
            KeyName::E => "E",
            KeyName::B => "B",
            KeyName::C => "C",
            KeyName::L => "L",
            KeyName::W => "W",
            KeyName::T => "T",
            KeyName::V => "V",
            KeyName::P => "P",
            KeyName::Comma => ",",
            KeyName::Enter => "Enter",
            KeyName::F11 => "F11",
            KeyName::Num1 => "1",
            KeyName::Num2 => "2",
            KeyName::Num3 => "3",
            KeyName::Delete => "Del",
            KeyName::Backspace => "Backspace",
            KeyName::Up => "Up",
            KeyName::Down => "Down",
        }
    }
}

impl Shortcut {
    pub fn new(ctrl: bool, shift: bool, alt: bool, key: KeyName) -> Self {
        Self {
            ctrl,
            shift,
            alt,
            key,
        }
    }

    pub fn display_string(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            #[cfg(target_os = "macos")]
            parts.push("Cmd");
            #[cfg(not(target_os = "macos"))]
            parts.push("Ctrl");
        }
        if self.alt {
            #[cfg(target_os = "macos")]
            parts.push("Option");
            #[cfg(not(target_os = "macos"))]
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        parts.push(self.key.label());
        parts.join("+")
    }

    pub fn matches(&self, input: &egui::InputState) -> bool {
        let modifier_ctrl = input.modifiers.command || input.modifiers.ctrl;
        let modifier_shift = input.modifiers.shift;
        let modifier_alt = input.modifiers.alt;

        let ctrl_match = if self.ctrl {
            modifier_ctrl
        } else {
            !modifier_ctrl
        };
        let shift_match = if self.shift {
            modifier_shift
        } else {
            !modifier_shift
        };
        let alt_match = if self.alt {
            modifier_alt
        } else {
            !modifier_alt
        };

        let key_pressed = input.key_pressed(self.key.to_egui_key());

        ctrl_match && shift_match && alt_match && key_pressed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindConfig {
    pub new_doc: Shortcut,
    pub open_doc: Shortcut,
    pub save_doc: Shortcut,
    pub export_docx: Shortcut,
    pub export_html: Shortcut,
    pub add_paragraph: Shortcut,
    pub insert_blockquote: Shortcut,
    pub insert_h1: Shortcut,
    pub insert_h2: Shortcut,
    pub insert_h3: Shortcut,
    pub insert_citation: Shortcut,
    pub delete_block: Shortcut,
    pub move_block_up: Shortcut,
    pub move_block_down: Shortcut,
    pub manage_works_cited: Shortcut,
    pub convert_title_case: Shortcut,
    pub open_preferences: Shortcut,
    pub toggle_compliance: Shortcut,
    pub toggle_focus: Shortcut,
}

impl Default for KeybindConfig {
    fn default() -> Self {
        Self {
            new_doc: Shortcut::new(true, false, false, KeyName::N),
            open_doc: Shortcut::new(true, false, false, KeyName::O),
            save_doc: Shortcut::new(true, false, false, KeyName::S),
            export_docx: Shortcut::new(true, false, false, KeyName::E),
            export_html: Shortcut::new(true, true, false, KeyName::E),
            add_paragraph: Shortcut::new(true, false, false, KeyName::Enter),
            insert_blockquote: Shortcut::new(true, true, false, KeyName::B),
            insert_h1: Shortcut::new(true, false, true, KeyName::Num1),
            insert_h2: Shortcut::new(true, false, true, KeyName::Num2),
            insert_h3: Shortcut::new(true, false, true, KeyName::Num3),
            insert_citation: Shortcut::new(true, true, false, KeyName::C),
            delete_block: Shortcut::new(true, false, false, KeyName::Backspace),
            move_block_up: Shortcut::new(false, false, true, KeyName::Up),
            move_block_down: Shortcut::new(false, false, true, KeyName::Down),
            manage_works_cited: Shortcut::new(true, false, false, KeyName::W),
            convert_title_case: Shortcut::new(true, true, false, KeyName::T),
            open_preferences: Shortcut::new(true, false, false, KeyName::Comma),
            toggle_compliance: Shortcut::new(true, false, false, KeyName::L),
            toggle_focus: Shortcut::new(false, false, false, KeyName::F11),
        }
    }
}

impl KeybindConfig {
    pub fn get_shortcut(&self, action: Action) -> Shortcut {
        match action {
            Action::NewDocument => self.new_doc,
            Action::OpenDocument => self.open_doc,
            Action::SaveDocument => self.save_doc,
            Action::ExportDocx => self.export_docx,
            Action::ExportHtmlPdf => self.export_html,
            Action::AddParagraph => self.add_paragraph,
            Action::InsertBlockQuote => self.insert_blockquote,
            Action::InsertHeading1 => self.insert_h1,
            Action::InsertHeading2 => self.insert_h2,
            Action::InsertHeading3 => self.insert_h3,
            Action::InsertCitation => self.insert_citation,
            Action::DeleteBlock => self.delete_block,
            Action::MoveBlockUp => self.move_block_up,
            Action::MoveBlockDown => self.move_block_down,
            Action::ManageWorksCited => self.manage_works_cited,
            Action::ConvertToMlaTitleCase => self.convert_title_case,
            Action::OpenPreferences => self.open_preferences,
            Action::ToggleComplianceCheck => self.toggle_compliance,
            Action::ToggleFocusMode => self.toggle_focus,
        }
    }

    pub fn set_shortcut(&mut self, action: Action, sc: Shortcut) {
        match action {
            Action::NewDocument => self.new_doc = sc,
            Action::OpenDocument => self.open_doc = sc,
            Action::SaveDocument => self.save_doc = sc,
            Action::ExportDocx => self.export_docx = sc,
            Action::ExportHtmlPdf => self.export_html = sc,
            Action::AddParagraph => self.add_paragraph = sc,
            Action::InsertBlockQuote => self.insert_blockquote = sc,
            Action::InsertHeading1 => self.insert_h1 = sc,
            Action::InsertHeading2 => self.insert_h2 = sc,
            Action::InsertHeading3 => self.insert_h3 = sc,
            Action::InsertCitation => self.insert_citation = sc,
            Action::DeleteBlock => self.delete_block = sc,
            Action::MoveBlockUp => self.move_block_up = sc,
            Action::MoveBlockDown => self.move_block_down = sc,
            Action::ManageWorksCited => self.manage_works_cited = sc,
            Action::ConvertToMlaTitleCase => self.convert_title_case = sc,
            Action::OpenPreferences => self.open_preferences = sc,
            Action::ToggleComplianceCheck => self.toggle_compliance = sc,
            Action::ToggleFocusMode => self.toggle_focus = sc,
        }
    }

    pub fn check_action(&self, action: Action, input: &egui::InputState) -> bool {
        match action {
            Action::AddParagraph | Action::ExportDocx | Action::ExportHtmlPdf => false,
            _ => self.get_shortcut(action).matches(input),
        }
    }
}
