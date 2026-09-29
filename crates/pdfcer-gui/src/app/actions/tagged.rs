//! # `app::actions::tagged` — a tagged PDF's own structure, for the Word and table exports
//!
//! `layout_from_structure` under `StructureUse::Auto`: the tree's headings,
//! paragraphs, lists and tables when it owns at least half the laid-out text,
//! otherwise the inferred layout, which the call returns itself. Either way
//! the receipt says which source was followed and what the tree could not
//! express (R8b).
//!
//! `read_structure_tree` extracts every page's text whatever the tree holds,
//! so the table export, which may cover a few pages of a long drawing set,
//! asks [`has_tree`] first and skips the read on an untagged file.

use pdfcer_core::block_layout::{DocumentLayout, LayoutOptions, PageGeometry};
use pdfcer_core::page_tree::Page;
use pdfcer_core::structure_tree;
use pdfcer_core::table_detect::{self, Table};
use pdfcer_core::tagged_layout::{
    self, FallbackReason, LayoutSourceUsed, TaggedLayoutOptions, TaggedLayoutReport,
};
use pdfcer_core::text_extract::ExtractOptions;
use pdfcer_core::view::DocumentView;

use crate::text::export_tagged as t;

/// The document laid out, from its tree where the tree qualified.
pub(super) struct Structured {
    /// Blocks per page, in `geometry`'s order.
    pub layout: DocumentLayout,
    /// One entry per page of `layout.pages`.
    pub geometry: Vec<PageGeometry>,
    /// The tree's tables; empty when the tree was not followed, in which
    /// case the caller runs its own table detection.
    pub tables: Vec<Table>,
    pub report: TaggedLayoutReport,
}

impl Structured {
    /// Whether the tree was followed.
    pub fn followed(&self) -> bool {
        self.report.source == LayoutSourceUsed::StructureTree
    }
}

/// Whether the catalog names a `/StructTreeRoot`. Reads one key.
pub(super) fn has_tree(view: &DocumentView<'_>) -> bool {
    use pdfcer_core::graph::ObjectGraph;
    view.catalog_dict()
        .is_some_and(|catalog| catalog.get(b"StructTreeRoot").is_some())
}

/// Lays the whole document out. With `keep`, the result is cut to those page
/// indices and sorted into their order.
pub(super) fn lay_out(
    view: &DocumentView<'_>,
    pages: &[Page],
    options: &ExtractOptions,
    keep: Option<&[usize]>,
) -> Result<Structured, String> {
    let tree = structure_tree::read_structure_tree(view, options).map_err(|e| e.to_string())?;
    let geometry_of = |indices: &mut dyn Iterator<Item = usize>| -> Vec<PageGeometry> {
        indices
            .map(|i| {
                pages.get(i).map_or(
                    PageGeometry::new(
                        pdfcer_core::page_tree::Rect::from_corners(0.0, 0.0, 612.0, 792.0),
                        0,
                    ),
                    |page| PageGeometry::new(page.crop_box, page.rotate),
                )
            })
            .collect()
    };
    let all = geometry_of(&mut tree.text.pages.iter().map(|p| p.page_index));
    let mut tagged = tagged_layout::layout_from_structure(
        &tree,
        &all,
        &LayoutOptions::default(),
        &TaggedLayoutOptions::default(),
    );
    if let Some(keep) = keep {
        tagged.retain_pages(keep);
        let rank = |page: usize| keep.iter().position(|&i| i == page);
        tagged.layout.pages.sort_by_key(|p| rank(p.page_index));
        tagged.tables.sort_by_key(|t| rank(t.page_index));
    }
    let geometry = geometry_of(&mut tagged.layout.pages.iter().map(|p| p.page_index));
    let tables = table_detect::tables_from_structure(&tagged.tables, &tagged.layout.text);
    Ok(Structured {
        layout: tagged.layout,
        geometry,
        tables,
        report: tagged.report,
    })
}

/// The receipt's sentences about the tree. Silent on an untagged document:
/// there was nothing to follow, and the inference notes already say so.
pub(super) fn notes(report: &TaggedLayoutReport) -> Vec<String> {
    let percent = percent(report.coverage);
    let mut notes = Vec::new();
    match report.fallback {
        None => notes.push(t::followed(percent)),
        Some(FallbackReason::LowCoverage) => notes.push(t::too_little(percent)),
        Some(FallbackReason::NoTextClaimed) => notes.push(t::owns_nothing().to_owned()),
        Some(_) => return notes,
    }
    if report.fallback.is_some() {
        return notes;
    }
    let paragraphs = report
        .non_standard_as_paragraph
        .saturating_add(report.untyped_as_paragraph);
    if paragraphs > 0 {
        notes.push(t::as_paragraphs(paragraphs));
    }
    if report.nested_tables_flattened > 0 {
        notes.push(t::nested_tables(report.nested_tables_flattened));
    }
    if report.stray_table_content > 0 {
        notes.push(t::stray_table_content(report.stray_table_content));
    }
    if report.broken_references > 0 {
        notes.push(t::broken_references(report.broken_references));
    }
    notes
}

/// The trace fields every export using this module appends.
pub(super) fn trace_fields(report: &TaggedLayoutReport) -> String {
    let fallback = match report.fallback {
        None => "none",
        Some(FallbackReason::Disabled) => "disabled",
        Some(FallbackReason::NoStructureTree) => "no-tree",
        Some(FallbackReason::NoTextClaimed) => "no-text-claimed",
        Some(FallbackReason::LowCoverage) => "low-coverage",
        Some(_) => "other",
    };
    // ui-text-exempt: diagnostic trace fields, never displayed
    format!(
        "structure={} structure_fallback={fallback} structure_coverage={} structure_blocks={} \
         structure_tables={} broken_references={}",
        if report.fallback.is_none() {
            "tree"
        } else {
            "layout"
        },
        percent(report.coverage),
        report.structure_blocks,
        report.tables,
        report.broken_references,
    )
}

/// A coverage fraction as a whole percentage, clamped to 0–100.
fn percent(coverage: f32) -> u32 {
    // Clamped to 0..=100 first, so the cast cannot truncate or wrap.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let p = (coverage.clamp(0.0, 1.0) * 100.0).round() as u32;
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_untagged_document_adds_no_sentence() {
        let mut report = TaggedLayoutReport::default();
        report.fallback = Some(FallbackReason::NoStructureTree);
        assert!(notes(&report).is_empty());
    }

    #[test]
    fn a_followed_tree_names_its_coverage_and_its_losses() {
        let mut report = TaggedLayoutReport::default();
        report.source = LayoutSourceUsed::StructureTree;
        report.coverage = 0.874;
        report.untyped_as_paragraph = 2;
        report.broken_references = 1;
        let notes = notes(&report);
        assert!(notes[0].contains("87%"), "{notes:?}");
        assert_eq!(notes.len(), 3, "{notes:?}");
    }

    #[test]
    fn low_coverage_says_the_page_was_judged_instead() {
        let mut report = TaggedLayoutReport::default();
        report.fallback = Some(FallbackReason::LowCoverage);
        report.coverage = 0.2;
        report.broken_references = 4;
        let notes = notes(&report);
        assert_eq!(notes.len(), 1, "{notes:?}");
        assert!(notes[0].contains("20%"), "{notes:?}");
    }
}
