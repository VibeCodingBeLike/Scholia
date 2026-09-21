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
    MoveSentenceLeft,
    MoveSentenceRight,
    ManageWorksCited,
    AddFootnote,
    ConvertToMlaTitleCase,
    OpenPreferences,
    ToggleComplianceCheck,
    ToggleFocusMode,
    Undo,
    Redo,
}

impl Action {
    pub fn display_name(&self) -> &'static str {
        match self {
            Action::NewDocument => "New MLA Document",
            Action::OpenDocument => "Open Document",
            Action::SaveDocument => "Save Document (.mla)",
            Action::ExportDocx => "Export to Word (.docx)",
            Action::ExportHtmlPdf => "Export to Printable HTML / PDF",
            Action::AddParagraph => "Add New Body Paragraph",
            Action::InsertBlockQuote => "Insert MLA Block Quote (>4 lines)",
            Action::InsertHeading1 => "Insert Section Heading 1 (Bold)",
            Action::InsertHeading2 => "Insert Section Heading 2 (Italics)",
            Action::InsertHeading3 => "Insert Section Heading 3 (Centered)",
            Action::InsertCitation => "Insert In-Text Parenthetical Citation",
            Action::DeleteBlock => "Delete Paragraph",
            Action::MoveBlockUp => "Move Active Block Up",
            Action::MoveBlockDown => "Move Active Block Down",
            Action::MoveSentenceLeft => "Move Sentence Left",
            Action::MoveSentenceRight => "Move Sentence Right",
            Action::ManageWorksCited => "Open Works Cited Manager",
            Action::AddFootnote => "Add Note / Definition",
            Action::ConvertToMlaTitleCase => "Format Title to MLA Title Case",
            Action::OpenPreferences => "Theme & Transparency Settings",
            Action::ToggleComplianceCheck => "Run MLA Compliance Inspector",
            Action::ToggleFocusMode => "Toggle Zen / Focus Mode",
            Action::Undo => "Undo",
            Action::Redo => "Redo",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            Action::ManageWorksCited
            | Action::InsertCitation
            | Action::AddFootnote
            | Action::InsertBlockQuote
            | Action::InsertHeading1
            | Action::InsertHeading2
            | Action::InsertHeading3 => "Writing",

            Action::MoveBlockUp
            | Action::MoveBlockDown
            | Action::MoveSentenceLeft
            | Action::MoveSentenceRight
            | Action::DeleteBlock
            | Action::AddParagraph
            | Action::Undo
            | Action::Redo => "Modify",

            Action::NewDocument
            | Action::OpenDocument
            | Action::SaveDocument
            | Action::ExportDocx
            | Action::ExportHtmlPdf => "File",

            Action::ToggleFocusMode
            | Action::OpenPreferences
            | Action::ToggleComplianceCheck
            | Action::ConvertToMlaTitleCase => "Tools",
        }
    }

    pub fn is_in_text_action(&self) -> bool {
        matches!(self, Action::InsertCitation | Action::AddFootnote)
    }

    pub fn in_text_hint(&self) -> Option<&'static str> {
        match self {
            Action::InsertCitation => Some("/cite"),
            Action::AddFootnote => Some("^N"),
            _ => None,
        }
    }

    pub fn all() -> &'static [Action] {
        &[
            Action::NewDocument,
            Action::OpenDocument,
            Action::SaveDocument,
            Action::ExportDocx,
            Action::ExportHtmlPdf,
            Action::Undo,
            Action::Redo,
            Action::AddParagraph,
            Action::InsertBlockQuote,
            Action::InsertHeading1,
            Action::InsertHeading2,
            Action::InsertHeading3,
            Action::InsertCitation,
            Action::DeleteBlock,
            Action::MoveBlockUp,
            Action::MoveBlockDown,
            Action::MoveSentenceLeft,
            Action::MoveSentenceRight,
            Action::ManageWorksCited,
            Action::AddFootnote,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyName {
    None,
    N,
    O,
    S,
    E,
    B,
    C,
    F,
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
    Left,
    Right,
    Z,
    Y,
}

impl KeyName {
    pub fn all() -> &'static [KeyName] {
        &[
            KeyName::None,
            KeyName::N,
            KeyName::O,
            KeyName::S,
            KeyName::E,
            KeyName::B,
            KeyName::C,
            KeyName::F,
            KeyName::L,
            KeyName::W,
            KeyName::T,
            KeyName::V,
            KeyName::P,
            KeyName::Z,
            KeyName::Y,
            KeyName::Comma,
            KeyName::Enter,
            KeyName::F11,
            KeyName::Num1,
            KeyName::Num2,
            KeyName::Num3,
            KeyName::Delete,
            KeyName::Backspace,
            KeyName::Up,
            KeyName::Down,
            KeyName::Left,
            KeyName::Right,
        ]
    }

    pub fn to_egui_key(self) -> Key {
        match self {
            KeyName::None => Key::Escape, // Unused: matches() guards against KeyName::None
            KeyName::N => Key::N,
            KeyName::O => Key::O,
            KeyName::S => Key::S,
            KeyName::E => Key::E,
            KeyName::B => Key::B,
            KeyName::C => Key::C,
            KeyName::F => Key::F,
            KeyName::L => Key::L,
            KeyName::W => Key::W,
            KeyName::T => Key::T,
            KeyName::V => Key::V,
            KeyName::P => Key::P,
            KeyName::Z => Key::Z,
            KeyName::Y => Key::Y,
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
            KeyName::Left => Key::ArrowLeft,
            KeyName::Right => Key::ArrowRight,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            KeyName::None => "None",
            KeyName::N => "N",
            KeyName::O => "O",
            KeyName::S => "S",
            KeyName::E => "E",
            KeyName::B => "B",
            KeyName::C => "C",
            KeyName::F => "F",
            KeyName::L => "L",
            KeyName::W => "W",
            KeyName::T => "T",
            KeyName::V => "V",
            KeyName::P => "P",
            KeyName::Z => "Z",
            KeyName::Y => "Y",
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
            KeyName::Left => "Left",
            KeyName::Right => "Right",
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

    pub fn none() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
            key: KeyName::None,
        }
    }

    pub fn display_string(&self) -> String {
        if self.key == KeyName::None {
            return "None".to_string();
        }
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
        if self.key == KeyName::None {
            return false;
        }
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
    pub move_sentence_left: Shortcut,
    pub move_sentence_right: Shortcut,
    pub manage_works_cited: Shortcut,
    pub add_footnote: Shortcut,
    pub convert_title_case: Shortcut,
    pub open_preferences: Shortcut,
    pub toggle_compliance: Shortcut,
    pub toggle_focus: Shortcut,
    pub undo: Shortcut,
    pub redo: Shortcut,
}

impl Default for KeybindConfig {
    fn default() -> Self {
        Self {
            new_doc: Shortcut::new(true, false, false, KeyName::N),
            open_doc: Shortcut::new(true, false, false, KeyName::O),
            save_doc: Shortcut::new(true, false, false, KeyName::S),
            export_docx: Shortcut::new(true, false, false, KeyName::E),
            export_html: Shortcut::new(true, true, false, KeyName::E),
            undo: Shortcut::new(true, false, false, KeyName::Z),
            redo: Shortcut::new(true, false, false, KeyName::Y),
            add_paragraph: Shortcut::new(true, false, false, KeyName::Enter),
            insert_blockquote: Shortcut::new(true, true, false, KeyName::B),
            insert_h1: Shortcut::new(true, false, true, KeyName::Num1),
            insert_h2: Shortcut::new(true, false, true, KeyName::Num2),
            insert_h3: Shortcut::new(true, false, true, KeyName::Num3),
            insert_citation: Shortcut::none(), // In-text action (/cite) - no keybind
            delete_block: Shortcut::new(true, false, false, KeyName::Backspace),
            move_block_up: Shortcut::new(true, false, true, KeyName::Up),
            move_block_down: Shortcut::new(true, false, true, KeyName::Down),
            move_sentence_left: Shortcut::new(true, false, true, KeyName::Left),
            move_sentence_right: Shortcut::new(true, false, true, KeyName::Right),
            manage_works_cited: Shortcut::new(true, false, false, KeyName::W),
            add_footnote: Shortcut::none(), // In-text action (^N) - no keybind
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
            Action::MoveSentenceLeft => self.move_sentence_left,
            Action::MoveSentenceRight => self.move_sentence_right,
            Action::ManageWorksCited => self.manage_works_cited,
            Action::AddFootnote => self.add_footnote,
            Action::ConvertToMlaTitleCase => self.convert_title_case,
            Action::OpenPreferences => self.open_preferences,
            Action::ToggleComplianceCheck => self.toggle_compliance,
            Action::ToggleFocusMode => self.toggle_focus,
            Action::Undo => self.undo,
            Action::Redo => self.redo,
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
            Action::MoveSentenceLeft => self.move_sentence_left = sc,
            Action::MoveSentenceRight => self.move_sentence_right = sc,
            Action::ManageWorksCited => self.manage_works_cited = sc,
            Action::AddFootnote => self.add_footnote = sc,
            Action::ConvertToMlaTitleCase => self.convert_title_case = sc,
            Action::OpenPreferences => self.open_preferences = sc,
            Action::ToggleComplianceCheck => self.toggle_compliance = sc,
            Action::ToggleFocusMode => self.toggle_focus = sc,
            Action::Undo => self.undo = sc,
            Action::Redo => self.redo = sc,
        }
    }

    pub fn check_action(&self, action: Action, input: &egui::InputState) -> bool {
        match action {
            Action::AddParagraph
            | Action::ExportDocx
            | Action::ExportHtmlPdf
            | Action::InsertCitation
            | Action::AddFootnote => false,
            Action::Undo => {
                self.get_shortcut(action).matches(input)
                    || Shortcut::new(true, false, false, KeyName::Z).matches(input)
            }
            Action::Redo => {
                self.get_shortcut(action).matches(input)
                    || Shortcut::new(true, false, false, KeyName::Y).matches(input)
                    || Shortcut::new(true, true, false, KeyName::Z).matches(input)
                    || Shortcut::new(true, true, false, KeyName::Y).matches(input)
            }
            _ => self.get_shortcut(action).matches(input),
        }
    }
}
