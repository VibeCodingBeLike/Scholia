pub mod mla_doc;
pub mod works_cited;

#[allow(unused_imports)]
pub use mla_doc::{
    format_current_mla_date, generate_block_id, to_mla_title_case, MlaBlock, MlaDocument,
    MlaHeader,
};
#[allow(unused_imports)]
pub use works_cited::{SourceType, WorksCitedEntry};
