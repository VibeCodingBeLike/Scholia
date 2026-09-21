pub mod mla_doc;
pub mod works_cited;

#[allow(unused_imports)]
pub use mla_doc::{
    clean_superscripts, format_current_mla_date, generate_block_id, num_to_superscript,
    to_mla_title_case, typographical_clean, ExplanatoryNote, MlaBlock, MlaDocument, MlaHeader,
};
#[allow(unused_imports)]
pub use works_cited::{SourceType, WorksCitedEntry};
