//! # `dialogs::insert_pages` — which pages, and where they land
//!
//! The second half of `pages.insert_from_file`. The picker asks *which file*;
//! this asks the two questions a picker cannot.
//!
//! ## ★ Why this exists, which is a lesson rather than a feature note
//!
//! Insert shipped earlier the same day **without** it: it took every page of
//! the chosen file and put them after the current one. That satisfied the
//! sentence *"add insert from file"* and it is not the feature. The operator,
//! 2026-08-18:
//!
//! > *"when I ask for something, my expectation is usually that everything
//! > surrounding that request is also done to where it would match the
//! > behaviour a user would expect. Otherwise I am left typing out every little
//! > missing detail."*
//!
//! `HANDOFF.md` §3 instruction 0 is that, written down. The test it asks for is
//! *"what would a competent user reach for next, within this same gesture?"*,
//! and for an insert the answers are immediate: **how many pages am I about to
//! add**, **do I want all of them**, and **where do they go**. Acrobat's own
//! Insert Pages dialog asks the last two and shows the first.
//!
//! ## The four positions are the engine's own vocabulary
//!
//! `pdfcer_core::pageops::InsertPosition` is `Start` / `End` / `Before(n)` /
//! `After(n)`, and this dialog produces one directly rather than mapping
//! through a local enum. A second vocabulary for the same four choices would
//! be a second place for "before" and "after" to drift apart — and the drift
//! would be silent, because both spellings compile and both insert *somewhere*.
//!
//! ## The range grammar is the print dialog's, deliberately
//!
//! [`crate::dialogs::print::tabs::parse_page_range`] parses `3`, `1-4`,
//! `5,1-2`, and its own header carries the argument for why there is exactly
//! one of it:
//!
//! > *"Two range parsers would eventually disagree about something like
//! > `5,1-2` — whether it reorders, whether it deduplicates — and an operator
//! > moving between the GUI and a script would have no way to know which one
//! > they were talking to."*
//!
//! That argument was made about the GUI and the CLI. It is the same argument
//! between two GUI surfaces, and stronger: an operator who learns the range
//! syntax on Print is entitled to it working here.
//!
//! ★ **And the order-preserving, non-deduplicating behaviour is a feature
//! here.** `5,1-2` inserts source page 5 first, then 1 and 2 — which is a
//! reorder an operator can ask for in one gesture. `1,1` inserts page 1 twice,
//! which is also legitimate. Both fall out of treating the text as a sequence,
//! and both match Print and the CLI.
//!
//! ## Rule 4: nothing here is drawn on the page
//!
//! The dialog states what it will do and does it. The one inference pdfcer makes
//! on the operator's behalf is the **refusal** of an unparseable range, and
//! that is disclosed in words with the reason, never by silently inserting a
//! guess — the same posture the print dialog takes with the same parser.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::actions::pages::PageAction;
use crate::text::pages as t;

/// The dialog body's published region, for `ui-verify`.
const REGION_BODY: &str = "insert-pages.body";

/// The commit button's region.
const REGION_INSERT: &str = "insert-pages.insert";

/// Where the pages land, as the four radios offer it.
///
/// A local enum **only** for the radio state, converted to
/// `pdfcer_core::pageops::InsertPosition` at the point of use — because two of
/// the four need the current page index, which the radio does not carry and the
/// dialog does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Where {
    /// Before the page the operator is looking at.
    BeforeCurrent,
    /// After it. **The default**, because it is what "insert here" means to
    /// somebody who navigated to a sheet first.
    AfterCurrent,
    /// Before every existing page.
    Start,
    /// After every existing page.
    End,
}

/// The insert dialog's state.
pub struct InsertPagesDialog {
    /// The file the picker chose.
    path: std::path::PathBuf,
    /// Its name, for the sentence. Held rather than derived per frame.
    name: String,
    /// How many pages it has — read **once**, when the dialog opened.
    ///
    /// Reading it here rather than in the apply arm is what lets the dialog say
    /// *"4 pages"* before the operator commits to anything, which is the first
    /// of the three questions this dialog exists to answer.
    source_pages: usize,
    /// The page the operator was looking at when they asked. Frozen at open:
    /// the dialog is modal in spirit and the page behind it does not move, and
    /// a position that re-read the view every frame would mean *"after page 7"*
    /// silently becoming *"after page 9"*.
    current_page: usize,
    /// Take every page of the source.
    all: bool,
    /// The typed range, live even while [`Self::all`] is set, so switching away
    /// and back does not lose it — the same reasoning as the print dialog's
    /// `range_text`.
    range_text: String,
    /// Which of the four positions.
    position: Where,
    /// Set by the commit button, consumed after the window closure returns.
    ///
    /// Deferred by one statement for the print dialog's reason: the action
    /// replaces most of the document's derived state, and doing that inside
    /// `Window::show`'s closure runs it while egui is part-way through laying
    /// this window out.
    insert_requested: bool,
    /// Set by Cancel.
    close_requested: bool,
}

impl InsertPagesDialog {
    /// Open it for `path`, having already counted its pages.
    #[must_use]
    pub fn open(path: std::path::PathBuf, source_pages: usize, current_page: usize) -> Self {
        let name = path.file_name().map_or_else(
            || path.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        );
        Self {
            path,
            name,
            source_pages,
            current_page,
            all: true,
            // Seeded with the whole document rather than empty. An empty field
            // beside an unselected radio is a control with no example in it,
            // and the operator has to guess the syntax; `1-4` teaches it.
            range_text: if source_pages > 0 {
                format!("1-{source_pages}")
            } else {
                String::new()
            },
            position: Where::AfterCurrent,
            insert_requested: false,
            close_requested: false,
        }
    }

    /// The source pages this dialog currently names, or `None` if the typed
    /// range is unparseable.
    ///
    /// `None` is what disables the commit button *and* draws the refusal — one
    /// derivation feeding both, so the button cannot be live while the sentence
    /// says the range is bad.
    fn chosen(&self) -> Option<Vec<usize>> {
        if self.all {
            return Some((0..self.source_pages).collect());
        }
        crate::dialogs::print::tabs::parse_page_range(&self.range_text, self.source_pages)
            .filter(|pages| !pages.is_empty())
    }

    /// The engine's position for the selected radio.
    const fn position(&self) -> pdfcer_core::pageops::InsertPosition {
        use pdfcer_core::pageops::InsertPosition;
        match self.position {
            Where::BeforeCurrent => InsertPosition::Before(self.current_page),
            Where::AfterCurrent => InsertPosition::After(self.current_page),
            Where::Start => InsertPosition::Start,
            Where::End => InsertPosition::End,
        }
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        // ★ ITS OWN OS WINDOW as of 2026-08-21. Size is an opening bid; see
        // [`crate::dialogs::host::Host::fit`].
        let (frame, ()) = crate::dialogs::host::Host::new(
            "insert-pages", // ui-text-exempt: a viewport key, never displayed.
            t::insert_window_title(),
            egui::vec2(480.0, 560.0),
            egui::vec2(380.0, 300.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.insert_requested)
            && let Some(pages) = self.chosen()
        {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "insert-pages-requested path={:?} n={} of={} position={:?}",
                    self.path,
                    pages.len(),
                    self.source_pages,
                    self.position(),
                )
            });
            actions.push(Action::Page(PageAction::InsertPagesFromFile {
                path: self.path.clone(),
                pages,
                position: self.position(),
            }));
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// The two questions, the sentence, and the two buttons.
    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::insert_source(&self.name, self.source_pages));
        ui.add_space(8.0);

        ui.label(t::insert_which_heading());
        if ui
            .radio(self.all, t::insert_all(self.source_pages))
            .clicked()
        {
            self.all = true;
        }
        ui.horizontal(|ui| {
            if ui.radio(!self.all, t::insert_range()).clicked() {
                self.all = false;
            }
            // Typing in the field selects the radio, which is the behaviour
            // every range control in this application has: an operator who
            // clicks into a text box has said what they want, and making them
            // also press the radio beside it is a second statement of one
            // intent.
            if ui
                .add(egui::TextEdit::singleline(&mut self.range_text).desired_width(140.0))
                .changed()
            {
                self.all = false;
            }
        });
        if !self.all {
            ui.label(egui::RichText::new(t::insert_range_hint()).small().weak());
        }

        ui.add_space(8.0);
        ui.label(t::insert_where_heading());
        // The current page is named in the label rather than left as "here",
        // because the dialog is centred over a document the operator may have
        // scrolled: the number is what makes the choice checkable.
        let page = self.current_page.saturating_add(1);
        for (option, label) in [
            (Where::AfterCurrent, t::insert_after_page(page)),
            (Where::BeforeCurrent, t::insert_before_page(page)),
            (Where::Start, t::insert_at_start().to_owned()),
            (Where::End, t::insert_at_end().to_owned()),
        ] {
            if ui.radio(self.position == option, label).clicked() {
                self.position = option;
            }
        }

        ui.add_space(8.0);
        let chosen = self.chosen();
        match &chosen {
            Some(pages) => {
                ui.label(
                    egui::RichText::new(t::insert_summary(pages.len()))
                        .small()
                        .weak(),
                );
            }
            None => {
                ui.label(
                    egui::RichText::new(t::insert_range_unparsable())
                        .small()
                        .color(ui.visuals().error_fg_color),
                );
            }
        }

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(t::insert_cancel()).clicked() {
                self.close_requested = true;
            }
            // ★ ABSENT rather than greyed while the range is unparseable, on
            // the standing rule: the refusal is already on screen immediately
            // above, naming what is wrong, so a greyed button would be a
            // second and quieter statement of a fact already made loudly.
            // Same reasoning as the print dialog's commit button.
            if let Some(pages) = chosen {
                let button = ui.button(t::insert_commit(pages.len()));
                crate::diag::ui_rect(REGION_INSERT, button.rect);
                if button.clicked() {
                    self.insert_requested = true;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::pageops::InsertPosition;

    fn dialog() -> InsertPagesDialog {
        InsertPagesDialog::open(std::path::PathBuf::from("x.pdf"), 4, 6)
    }

    /// ★ The four radios produce the engine's four positions, and the two that
    /// need a page carry the RIGHT one.
    ///
    /// The failure this catches is an off-by-one between "after page 7" as the
    /// operator reads it and `After(6)` as the engine takes it — invisible in
    /// any test that only checks that *a* position was produced, and visible to
    /// an operator as pages landing one sheet away from where they asked.
    #[test]
    fn each_position_maps_to_the_engines_own() {
        let mut d = dialog();
        d.position = Where::AfterCurrent;
        assert_eq!(d.position(), InsertPosition::After(6));
        d.position = Where::BeforeCurrent;
        assert_eq!(d.position(), InsertPosition::Before(6));
        d.position = Where::Start;
        assert_eq!(d.position(), InsertPosition::Start);
        d.position = Where::End;
        assert_eq!(d.position(), InsertPosition::End);
    }

    /// ★ An unparseable range names NO pages, which is what hides the button.
    ///
    /// Both halves matter: a bad range must not fall back to "all" — that would
    /// insert a document the operator did not ask for — and an empty result
    /// must be `None` rather than `Some(vec![])`, or the button would be drawn
    /// over a selection of nothing.
    #[test]
    fn a_bad_range_names_nothing_and_does_not_fall_back_to_all() {
        let mut d = dialog();
        d.all = false;
        for spec in ["", "9", "abc", "3-1", "0"] {
            d.range_text = spec.to_owned();
            assert!(d.chosen().is_none(), "{spec:?} must name no pages");
        }
        d.range_text = "1-4".to_owned();
        assert_eq!(d.chosen(), Some(vec![0, 1, 2, 3]));
    }

    /// The default is every page, after the page the operator was on.
    ///
    /// Pinned because it is the behaviour the first version of this feature had
    /// with no dialog at all, and an operator who liked it should be able to
    /// press Insert twice and get it.
    #[test]
    fn it_opens_on_every_page_after_the_current_one() {
        let d = dialog();
        assert!(d.all);
        assert_eq!(d.chosen(), Some(vec![0, 1, 2, 3]));
        assert_eq!(d.position(), InsertPosition::After(6));
    }

    /// ★ The range grammar is the print dialog's, including the two surprises.
    ///
    /// Order is preserved and duplicates are kept, because the text is a
    /// SEQUENCE the operator wrote. Here that is not a quirk to tolerate — it
    /// is how an operator inserts pages in a different order, or twice, in one
    /// gesture. Asserted so that a later "tidy-up" into a sorted set has to
    /// argue with a test rather than with nothing.
    #[test]
    fn the_range_is_a_sequence_not_a_set() {
        let mut d = dialog();
        d.all = false;
        d.range_text = "3,1-2".to_owned();
        assert_eq!(d.chosen(), Some(vec![2, 0, 1]), "order is the operator's");
        d.range_text = "1,1".to_owned();
        assert_eq!(d.chosen(), Some(vec![0, 0]), "a page may be inserted twice");
    }
}
