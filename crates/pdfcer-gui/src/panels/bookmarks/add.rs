//! # `panels::bookmarks::add` — writing a bookmark, and the `/Count` trap that
//! comes with it
//!
//! ## The gap this closes
//!
//! pdfcer has read bookmarks since the reader passes and had **zero authoring
//! verbs opposite them** — the engine's own words. `EditSession::add_outline_item`
//! shipped 2026-08-19 as `Pass 103.0`, the first ask of this shell's
//! `insert_pages` request, and this is the surface for it.
//!
//! ## ★★ The `/Count` trap, and why nothing here diffs a number
//!
//! The engine flagged it as *"not a footnote … the entire difficulty of the
//! feature"*, and it is the one thing that would produce a wrong disclosure:
//!
//! | | root `/Outlines` | an item |
//! |---|---|---|
//! | `/Count` counts | visible items at **every** level, including top-level | visible **descendants**, excluding itself |
//! | absent means | no open items | the item is a **leaf** |
//!
//! On an item the **sign is the open/closed flag** — §12.3.3 defines no `/Open`
//! key, so the sign is the only carrier. And the consequence:
//!
//! > Adding a bookmark under a **collapsed** ancestor does not change the
//! > document's total, because the new item is not visible.
//!
//! So a surface reporting *"added N bookmarks"* by diffing the root count
//! reports **zero for a correct save**. Nothing here diffs anything: one call
//! adds one bookmark, and that is what is said.
//!
//! ## ★ And the collapsed case is DISCLOSED, not merely survived
//!
//! Getting the count right is the low bar. The operator's actual problem is
//! that they will add a bookmark under a collapsed parent, look at the panel,
//! and **not see it** — because it genuinely is not visible, and the panel is
//! correct to show it that way.
//!
//! `OutlineItem::open` is read from the parent before the add, so the sentence
//! can be said. It is the same posture the ce-dimension group window takes
//! about re-measuring on a move: state the surprising consequence before the
//! press, not after the operator has gone looking.
//!
//! ## Why only the current page, and only `Fit`
//!
//! Because those are the two things the engine authors today and the only ones
//! it authors **without refusing**. `Destination::Named` and `Remote` are
//! refused by name, and `DestView::Unknown` is *"the one that looks writable
//! and is not"* — the reader keeps an extension's fit name and discards its
//! parameters, so re-emitting it writes a view that is not the one the source
//! had.
//!
//! A destination chooser offering fits pdfcer cannot write would be a control
//! whose options are mostly refusals, which is R9 at the level of a combo box.
//! The page the operator is looking at is the destination every other
//! page-scoped surface in this application uses, and it needs no chooser.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::actions::bookmarks::BookmarkAction;
use crate::app::state::OpenDoc;
use crate::text::panels as t;

use super::BookmarksUi;
use super::tree::{display_title, find};

/// The region the title field publishes.
pub const REGION_TITLE: &str = "bookmarks.new_title"; // ui-text-exempt: trace region name, never displayed
/// The region the Add button publishes.
pub const REGION_ADD: &str = "bookmarks.add"; // ui-text-exempt: trace region name, never displayed

/// Draw the add-a-bookmark row.
///
/// `items` is the outline as it currently stands, used for two things and
/// neither of them a count: naming the chosen parent, and reading whether it is
/// **collapsed**.
pub fn show(ui: &mut Ui, doc: &OpenDoc, ui_state: &mut BookmarksUi, actions: &mut Vec<Action>) {
    let page = doc.view.page_index;
    let outline = pdfcer_core::outline::read_outline(&doc.session.view());
    // A parent that has gone — the document was edited, or reloaded — falls
    // back to the top level rather than naming an object that is not there.
    // Reachable through undo, which is the ordinary way an id stops resolving.
    let parent = ui_state.selected.and_then(|id| find(&outline.items, id));
    if ui_state.selected.is_some() && parent.is_none() {
        ui_state.clear_selection();
    }

    ui.separator();
    ui.label(t::bookmark_add_heading());

    // --- where it goes -----------------------------------------------------
    ui.horizontal(|ui| {
        ui.label(match parent {
            Some(item) => t::bookmark_add_under(&display_title(&item.title)),
            None => t::bookmark_add_at_top().to_owned(),
        });
        // Offered only when there is something to clear, for the reason the
        // Rename button in the groups window is: a control whose only possible
        // effect is the state you are already in reads as broken.
        if parent.is_some() && ui.button(t::bookmark_add_to_top_button()).clicked() {
            // Through the state's own verb rather than by poking the field,
            // because clearing the selection must also drop the rename draft
            // held against it - the two are one act, and `clear_selection` is
            // where that is written down once. See `super::BookmarksUi`.
            ui_state.clear_selection();
        }
    });
    ui.weak(t::bookmark_add_parent_hint());
    ui.weak(t::bookmark_add_destination(page.saturating_add(1)));

    // ★ The `/Count` disclosure, said BEFORE the press. See the module header:
    // a bookmark added under a collapsed parent is genuinely not visible, the
    // panel is correct to show it that way, and an operator who was not told
    // goes looking for a bookmark that is there.
    if parent.is_some_and(|item| !item.open) {
        ui.weak(t::bookmark_add_under_collapsed());
    }

    // --- the title and the button -----------------------------------------
    ui.horizontal(|ui| {
        let response = ui.add(
            egui::TextEdit::singleline(&mut ui_state.title)
                .desired_width(160.0)
                .hint_text(t::bookmark_add_title_hint()),
        );
        crate::diag::ui_rect(REGION_TITLE, response.rect);

        let title = ui_state.title.trim().to_owned();
        if title.is_empty() {
            // Greyed WITH an explanation rather than absent: unlike the Rename
            // button next door, this control is the *whole* of the feature, and
            // a row that vanished until you typed would leave an operator
            // looking for where bookmarks are added.
            let disabled = ui.add_enabled(false, egui::Button::new(t::bookmark_add_button()));
            crate::diag::ui_rect(REGION_ADD, disabled.rect);
            // ★★★ **`on_disabled_hover_text`, since 2026-08-31** —
            // `OPERATOR_REQUESTS.md` O77's sweep.
            //
            // This read `on_hover_text`, and in egui 0.35 that builds
            // `Tooltip::for_enabled`, which opens only when
            // `response.enabled()` — so on a response that is already disabled
            // it runs no content and paints nothing. The comment above
            // promised *"greyed WITH an explanation"* and there was no
            // explanation: the control was greyed, silent, and unexplainable
            // by hovering, which is R9 breached by a one-word method name.
            disabled.on_disabled_hover_text(t::bookmark_add_needs_a_title());
        } else {
            let button = ui.button(t::bookmark_add_button());
            crate::diag::ui_rect(REGION_ADD, button.rect);
            if button.clicked() {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed. The
                    // LENGTH, not the text — a bookmark's name is the
                    // operator's own words about their drawing.
                    format!(
                        "bookmark-add page={} under={:?} chars={}",
                        page + 1,
                        ui_state.selected.map(|id| id.num),
                        title.chars().count()
                    )
                });
                actions.push(Action::Bookmark(BookmarkAction::Add {
                    parent: ui_state.selected,
                    title,
                    page,
                }));
                // Cleared so a second press cannot silently make a second
                // bookmark with the same name under the same parent — which
                // the engine would accept, and which would leave two
                // indistinguishable rows.
                ui_state.title.clear();
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The add row's own two strings still say what the module header promises.
    ///
    /// The tree walks that used to be tested here moved to [`super::tree`] on
    /// 2026-08-28, with their fixtures, when `edit` needed them too. What is
    /// left in this file that a test can reach without a `Ui` is the wording of
    /// the two disclosures the row is responsible for - and both of them are
    /// disclosures rather than labels, which is why they are worth pinning.
    #[test]
    fn the_collapsed_disclosure_names_the_consequence_rather_than_the_cause() {
        let said = t::bookmark_add_under_collapsed();
        // The operator's problem is that they will not SEE the bookmark. A
        // sentence about `/Count` would be true and useless.
        assert!(said.contains("expand"), "{said}");
        assert!(
            said.contains("still be in the file"),
            "the add WORKED, and the sentence must not read as a failure: {said}"
        );
    }

    /// An untitled parent is named by the stand-in, not by a gap in a sentence.
    ///
    /// An untitled bookmark is **legal** - `OutlineItem::title`'s own doc says
    /// a file may carry one - so `bookmark_add_under` must never be handed an
    /// empty string. That is [`display_title`]'s job, and this pins the pairing
    /// at the call site's own spelling.
    #[test]
    fn an_untitled_parent_is_still_nameable() {
        let sentence = t::bookmark_add_under(&display_title("   "));
        assert!(sentence.contains(t::bookmark_untitled()), "{sentence}");
        assert!(
            !sentence.ends_with("Under "),
            "a gap where the name should be: {sentence}"
        );
    }
}
