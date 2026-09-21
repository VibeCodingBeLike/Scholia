pub mod mla_doc;
pub mod works_cited;

#[allow(unused_imports)]
pub use mla_doc::{
    clean_superscripts, format_current_mla_date, generate_block_id, num_to_superscript,
    reorder_sentences, render_text_with_note_tags, split_sentences, to_mla_title_case,
    try_parse_and_apply_note_shortcut, typographical_clean, ExplanatoryNote, MlaBlock, MlaDocument,
    MlaHeader, NoteShortcutResult, NoteTag, SentenceSpan,
};
#[allow(unused_imports)]
pub use works_cited::{SourceType, WorksCitedEntry};
