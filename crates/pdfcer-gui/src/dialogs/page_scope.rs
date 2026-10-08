//! # `dialogs::page_scope` — "which pages": the scope group shared by
//! Recognise text and Remove OCR text
//!
//! Contract: [`PageScope`] holds the page the dialog opened on and the rail's
//! picked pages, both **captured at open** — the operator can page the
//! document or work the rail while the window is up, and a run that read
//! either live would act on a set they had stopped thinking about.
//! [`PageScope::pages`] resolves the answer to zero-based ascending indices,
//! `None` when it names no page (the caller greys its button: R9's
//! *temporarily* unavailable). [`PageScope::show`] draws the radios, publishes
//! the group's rect under the caller's region name, and traces
//! `<tag> pages= first= last=` on change.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/page_scope.md`.

use crate::text::ocr as t;

/// Which pages a run covers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Scope {
    /// Every page of the document. The default.
    All,
    /// Only the page the operator was looking at when the dialog opened.
    CurrentPage,
    /// The pages picked in the thumbnail rail (`OPERATOR_REQUESTS.md` O79).
    /// The rail's selection is already the operand for delete, extract,
    /// rotate and the page clipboard; a typed range would make him read
    /// page numbers off the rail and retype them.
    Picked,
    /// The pages named in [`PageScope::range`].
    Range,
}

impl Scope {
    /// The pages this scope names, zero-based and in order.
    pub(super) fn pages(
        self,
        current: usize,
        count: usize,
        range: &str,
        picked: &[usize],
    ) -> Option<Vec<usize>> {
        match self {
            Self::All => (count > 0).then(|| (0..count).collect()),
            Self::CurrentPage => (current < count).then(|| vec![current]),
            // Filtered against the page count: the selection was captured at
            // open and a page deleted since would be an index past the end.
            Self::Picked => {
                let pages: Vec<usize> = picked.iter().copied().filter(|p| *p < count).collect();
                (!pages.is_empty()).then_some(pages)
            }
            // The PRINT dialog's parser, so every page-range field in the
            // program accepts the same text.
            Self::Range => crate::dialogs::print::tabs::parse_page_range(range, count)
                .filter(|pages| !pages.is_empty()),
        }
    }
}

/// The scope group's state.
#[derive(Debug)]
pub(super) struct PageScope {
    /// The page the dialog opened on, zero-based.
    pub(super) page_index: usize,
    /// The rail's selection at open: zero-based, ascending, possibly empty —
    /// empty means [`Scope::Picked`] is not offered (R9).
    picked: Vec<usize>,
    pub(super) scope: Scope,
    /// Kept as text so a half-typed `1-` is a state the field can hold.
    range: String,
    /// The page list last traced, so the trace fires on a change.
    traced: Vec<usize>,
}

impl PageScope {
    /// All pages by default — what every surveyed tool defaults to.
    pub(super) fn new(page_index: usize, picked: Vec<usize>) -> Self {
        Self {
            page_index,
            picked,
            scope: Scope::All,
            range: String::new(),
            traced: Vec::new(),
        }
    }

    /// The pages the current answer names, for a document of `count` pages.
    pub(super) fn pages(&self, count: usize) -> Option<Vec<usize>> {
        self.scope
            .pages(self.page_index, count, &self.range, &self.picked)
    }

    /// Draw the group. `region` names its rect and prefixes each radio's
    /// (`.all`, `.current`, `.picked`); `tag` names its trace line.
    pub(super) fn show(&mut self, ui: &mut egui::Ui, count: usize, region: &str, tag: &str) {
        ui.label(t::scope_heading());
        ui.add_space(4.0);
        let group = ui
            .vertical(|ui| {
                let all = ui.radio_value(&mut self.scope, Scope::All, t::scope_all());
                crate::diag::ui_rect(&format!("{region}.all"), all.rect);
                let current = ui.radio_value(
                    &mut self.scope,
                    Scope::CurrentPage,
                    t::scope_current(self.page_index + 1),
                );
                crate::diag::ui_rect(&format!("{region}.current"), current.rect);
                // Third: the order of how much the operator had to do to
                // express the operand — nothing, one page, a picked set, a
                // typed set.
                if !self.picked.is_empty() {
                    let picked = ui.radio_value(
                        &mut self.scope,
                        Scope::Picked,
                        t::scope_picked(self.picked.len()),
                    );
                    crate::diag::ui_rect(&format!("{region}.picked"), picked.rect);
                }
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.scope, Scope::Range, t::scope_range());
                    let field = ui.add(
                        // escape-disposition: dialog-cancels — `dialogs::host` owns the key for
                        // every field in this window: the first press leaves the box, the second
                        // cancels.
                        egui::TextEdit::singleline(&mut self.range)
                            .desired_width(140.0)
                            .hint_text(t::scope_range_hint()),
                    );
                    // Typing IS the choice.
                    if field.gained_focus() || field.changed() {
                        self.scope = Scope::Range;
                    }
                });
            })
            .response;
        crate::diag::ui_rect(region, group.rect);
        let resolved = self.pages(count).unwrap_or_default();
        if resolved != self.traced {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "{tag} pages={} first={:?} last={:?}",
                    resolved.len(),
                    resolved.first(),
                    resolved.last()
                )
            });
            self.traced = resolved;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// All pages means all of them, in order, zero-based.
    #[test]
    fn all_pages_is_every_page_in_order() {
        assert_eq!(
            Scope::All.pages(3, 5, "", &[]),
            Some(vec![0, 1, 2, 3, 4]),
            "the current page has no bearing on All"
        );
    }

    /// **This page only means the page the dialog OPENED on.**
    #[test]
    fn this_page_only_is_the_captured_page() {
        assert_eq!(Scope::CurrentPage.pages(2, 5, "", &[]), Some(vec![2]));
    }

    /// **The rail's picked pages are the operand** —
    /// `OPERATOR_REQUESTS.md` O79.
    #[test]
    fn the_picked_pages_are_the_pages_picked() {
        assert_eq!(
            Scope::Picked.pages(0, 36, "", &[3, 7, 11, 12]),
            Some(vec![3, 7, 11, 12]),
            "the rail's selection is the operand, verbatim"
        );
        assert_eq!(
            Scope::Picked.pages(0, 36, "1-4", &[9]),
            Some(vec![9]),
            "a typed range in the field has no bearing on the picked scope"
        );
    }

    /// **A picked page the document no longer has is dropped**, and an
    /// empty result resolves to nothing.
    #[test]
    fn a_picked_page_the_document_lost_is_dropped() {
        assert_eq!(
            Scope::Picked.pages(0, 5, "", &[1, 4, 9, 20]),
            Some(vec![1, 4]),
            "indices past the end are dropped, the rest stand"
        );
        assert_eq!(
            Scope::Picked.pages(0, 5, "", &[9, 20]),
            None,
            "nothing left is nothing to run, which greys the button by the existing path"
        );
        assert_eq!(
            Scope::Picked.pages(0, 5, "", &[]),
            None,
            "an empty rail selection names no page"
        );
    }

    /// A scope naming no page resolves to nothing, which the dialog renders as
    /// an unavailable button rather than as an error.
    #[test]
    fn a_scope_that_names_no_page_resolves_to_nothing() {
        assert_eq!(Scope::Range.pages(0, 5, "", &[]), None, "an empty range");
        assert_eq!(Scope::Range.pages(0, 5, "  ", &[]), None, "whitespace");
        assert_eq!(Scope::Range.pages(0, 5, "9-12", &[]), None, "past the end");
        assert_eq!(
            Scope::All.pages(0, 0, "", &[]),
            None,
            "a document with no pages"
        );
        assert_eq!(
            Scope::CurrentPage.pages(7, 5, "", &[]),
            None,
            "a captured index the document no longer has"
        );
    }

    /// **The range field speaks the PRINT dialog's dialect, not its own.**
    #[test]
    fn the_range_is_parsed_by_the_print_dialogs_parser() {
        for input in ["1-3", "2,4", "1-2, 5", "3"] {
            assert_eq!(
                Scope::Range.pages(0, 5, input, &[]),
                crate::dialogs::print::tabs::parse_page_range(input, 5).filter(|p| !p.is_empty()),
                "the two must agree on {input:?}"
            );
        }
    }
}
