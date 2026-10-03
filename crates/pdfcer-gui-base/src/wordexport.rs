//! # `wordexport` — what File ▸ Export ▸ Word document… was asked to write
//!
//! The window's decisions, resolved before the press: the pages, and the
//! engine's three choices — `DocxOptions::page_breaks`, `DocxOptions::tables`
//! and where headings and tables come from. `pdfcer_gui::app::actions::export_word`
//! carries them out.

pub use crate::taggedexport::StructureSource;

/// Everything the Word-export window decided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordExportPlan {
    /// 0-based page indices, ascending and unique.
    pub pages: Vec<usize>,
    /// A page break between PDF pages.
    pub page_breaks: bool,
    /// Tables written as Word tables; `false` writes their text as paragraphs.
    pub tables: bool,
    /// Where headings, paragraphs and tables come from.
    pub structure: StructureSource,
}

impl WordExportPlan {
    /// The engine's defaults over `pages`, which are sorted and deduplicated.
    #[must_use]
    pub fn new(mut pages: Vec<usize>) -> Self {
        pages.sort_unstable();
        pages.dedup();
        Self {
            pages,
            page_breaks: true,
            tables: true,
            structure: StructureSource::Auto,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_are_put_in_document_order_once_each() {
        assert_eq!(WordExportPlan::new(vec![2, 0, 1, 1]).pages, vec![0, 1, 2]);
    }

    #[test]
    fn the_defaults_are_the_engines() {
        let plan = WordExportPlan::new(vec![0]);
        let engine = pdfcer_core::export::docx::DocxOptions::default();
        assert_eq!(plan.page_breaks, engine.page_breaks);
        assert_eq!(plan.tables, engine.tables);
        assert_eq!(plan.structure, StructureSource::Auto);
    }
}
