//! # `shell::manifest::rail` — what is in the left rail
//!
//! `OPERATOR_REQUESTS.md` **O123** part 7, his words:
//!
//! > *"What I'd also added in the bar at the left side that we are adding: the
//! > navigate selectors and some other related selection controls (lasso tool
//! > when we implement one, etc) and these will fold up into a drop down arrow
//! > if space becomes scarce."*
//!
//! and **O126**'s addendum, same conversation:
//!
//! > *"also add rotate pages to that area, and those should be available in
//! > every mode including read."*
//!
//! `mockups/pdfcer-shell.html` draws this and he approved it. This file is the
//! list; [`egui_shell::dock::rail`] is the geometry and the fold ladder.
//!
//! ## The three groups, and the one that is deliberately last
//!
//! | group | fold | why |
//! |---|---|---|
//! | the five **panel tabs** | [`RailFold::Never`] | *"all five panels one click away"* is the rail's entire argument for existing |
//! | **navigate** | [`RailFold::PinArmed`] | four mutually exclusive modal tools; at the floor the strip must still say which one you are holding |
//! | **select** | [`RailFold::Whole`] | the specialist gestures; nothing in it is reached by habit |
//! | **rotate** | [`RailFold::Whole`] | O126, and see the ⚠ below about Read |
//!
//! ## ⚠⚠ THIS LETS READ MODE DIRTY A DOCUMENT, AND THAT IS HIS CALL
//!
//! `pages.rotate_left` and `pages.rotate_right` write `/Rotate` into the page
//! dictionary. **That is a document edit, not a view transform** — the file is
//! modified, the undo log gains an entry, and the title bar gains its dirty
//! marker. Both commands are `enabled_when("doc.pages")` with **no mode gate**,
//! so they already worked in every mode; what this file adds is their
//! *placement*, on a strip that is on screen in Read.
//!
//! ⇒ So after this change **Read mode can dirty a document**, which cuts
//! against the standing *"Read authors nothing"* invariant that shapes every
//! other gate in this manifest.
//!
//! **It is built this way because he asked for it, in those words, and the
//! record should say so rather than let a later reader find it and file it as
//! an oversight.** The alternative that was considered and rejected in O126 is
//! a view-only rotation in Read and a real one elsewhere — *"two behaviours
//! wearing one button, which is worse than either"*. If he wants view-only
//! rotation, that is a different control and it should say so on its face.
//!
//! ## ★★ The lasso is NOT here, and the empty-group question was decided
//!
//! He named the lasso himself — *"lasso tool when we implement one"* — and it
//! does not exist: no command, no handler, no icon asset. The mockup draws it
//! dashed with the freehand pen's borrowed art, which is a thing **a mock may
//! do and the product may not**: R9 says an unavailable capability renders
//! *nothing*.
//!
//! Two shapes were available and the choice matters for what happens next:
//!
//! 1. a `select` group **present but empty**, as a placeholder; or
//! 2. a `select` group carrying the selection controls that **do** exist.
//!
//! (1) is refused. An empty group draws no caption and no rule — the planner
//! skips it — so it would be data that renders nothing, and the next reader
//! would have to run the code to discover that. (2) is what ships, and it is
//! possible only because of the second finding below.
//!
//! ## ★★★ `edit.select_all` — the objection was re-checked and it has lapsed
//!
//! The mockup's legend says a command with no icon cannot live in an icon
//! rail, and names `edit.select_all` as disqualified on exactly that ground:
//! *"the rail's rows are a picture with an optional word under it — at the
//! Tight rung and below there is no word left, and a row with neither is a
//! blank."* The reasoning is sound and the premise is stale. **`select-all`
//! was adopted as a glyph on 2026-09-04**, hours before this file was written,
//! after Ken pointed out that the refusal had never been his: *"add a
//! select-all glyph. I didn't refuse that."*
//!
//! So the objection is discharged by the fact rather than argued away, and
//! `edit.select_all` goes in the `select` group — which is also where it
//! belongs by meaning: he asked for *"some other related selection controls"*,
//! and Select all is one.
//!
//! ⇒ The lasso's seam is therefore a **one-line** change in a group that is
//! already on screen: add `Item::command("edit.lasso")` beside Select all when
//! the tool exists. It is marked in the code below.

use egui_shell::manifest::{Item, RailFold, RailGroup};

use crate::text::ribbon as t;

/// The rail's groups, top to bottom.
///
/// Consumed by [`super::build`] through `Shell::with_rail`, so this list is
/// **manifest data**: it merges, it validates, it serializes, and an operator
/// overlay can reorder it. `SHELL_FRAMEWORK.md`'s reason, in one line — *"a
/// rail that only `pdfcer-gui` knows about breaks it quietly."*
#[must_use]
pub fn groups() -> Vec<RailGroup> {
    vec![
        // -------------------------------------------------------------------
        // GROUP 1 — THE FIVE PANEL TABS. The floor.
        //
        // ★★ `RailFold::Never`, and it is the load-bearing choice on this
        // strip. O123 part 5 put Pages, Bookmarks, Layers, Signatures and
        // Fonts into ONE dock; the rail is what makes all five simultaneously
        // one click away, which a 280 pt horizontal tab bar cannot do — three
        // fit and two go behind a chevron. A rail that folded these would be
        // strictly worse than the tab stack it replaced, and at that point the
        // honest move is to switch arrangements rather than keep shrinking.
        //
        // No caption. A heading over the first group in the strip reads as a
        // heading for the whole rail, which is a different claim.
        //
        // These are the same `view.panel_*` commands the View tab registers,
        // and that is permitted for the QAT's reason (`RIBBON_IA.md` P1a): a
        // shortcut to a known home is not a second place to hunt.
        // -------------------------------------------------------------------
        RailGroup::new(
            "tabs",
            [
                Item::command("view.panel_pages"),
                Item::command("view.panel_bookmarks"),
                Item::command("view.panel_layers"),
                Item::command("view.panel_signatures"),
                // ★ `file.fonts`, NOT `view.panel_fonts` — the Fonts panel's command
                // is registered on the File tab and there is no second id for
                // it. `crate::panels::Panel::command_id` is the source of
                // truth, and inventing a symmetric-looking id here would give
                // the rail a tab that opens nothing.
                Item::command("file.fonts"),
            ],
        ),
        // -------------------------------------------------------------------
        // GROUP 2 — NAVIGATE. The four modal tools.
        //
        // The order is the order a tool palette is always in — the arrow, the
        // white arrow, the type tool, the hand — copied verbatim from
        // `super::view`'s Navigate group so the operator's eye finds the same
        // sequence on both surfaces. Two orders for one set of tools would be
        // worse than either.
        //
        // ★ `view.tool_node` carries `shown_when("mode.edit_content")`, the
        // same condition the ribbon item carries and for the same reason (O69,
        // R9): the Points tool edits the nodes of a path and Read cannot, so
        // in Read it renders NOTHING rather than greying. On a permanent
        // surface that matters more than on a tab — a control that is wrong
        // for the mode would be wrong on screen for the whole session rather
        // than for one click.
        //
        // ★ `view.smart_select` is deliberately absent. It is a TOGGLE that
        // changes what the arrow selects, not a tool you can be holding, so it
        // has no armed state for `PinArmed` to pin and it would be the one row
        // in this group that does not answer *"what does a drag do?"*.
        //
        // ★★★ `RailFold::PinArmed`: at the floor of the ladder this group
        // becomes ONE row showing whatever is armed — including a tool armed
        // from a ribbon tab that is not open, because the pinning reads
        // `selected:<id>`, which is application state rather than a property
        // of the visible tab. A rail that cannot say what you are holding has
        // given up the job `crate::app::toolstatus` handed it.
        // -------------------------------------------------------------------
        RailGroup::new(
            "navigate",
            [
                Item::command("view.tool_select"),
                Item::command("view.tool_node").shown_when("mode.edit_content"),
                Item::command("view.tool_text"),
                Item::command("view.tool_hand"),
            ],
        )
        .with_caption(t::group_view_navigate())
        .with_fold(RailFold::PinArmed),
        // -------------------------------------------------------------------
        // GROUP 3 — SELECT. His *"other related selection controls"*.
        //
        // ★★★ THE LASSO GOES HERE, AND NOWHERE ELSE, WHEN IT EXISTS.
        //
        //     Item::command("edit.lasso"),
        //
        // One line, in this list, once `edit.lasso` is a registered command
        // with a handler and an icon of its own. Nothing else changes: the
        // group already exists, already carries a caption, already folds as a
        // unit, and is already covered by the fold-ladder tests. Until then it
        // is absent rather than drawn dashed — see the module header on why a
        // mock may show an unbuilt control and a product may not (R9).
        //
        // ⚠ The lasso will need `shown_when("mode.edit_content")` when it
        // lands: a lasso picks page CONTENT, which is the one thing Read does
        // not do, and the mockup's own legend says so. `edit.select_all` needs
        // no such gate — it selects whatever the current surface offers and is
        // gated on `doc.pages` alone, at the command.
        //
        // `RailFold::Whole`: this is the group that folds FIRST, because
        // nothing in it is reached by habit or by chord in the way Hand and
        // Select are.
        // -------------------------------------------------------------------
        RailGroup::new("select", [Item::command("edit.select_all")])
            .with_caption(t::group_rail_select())
            .with_fold(RailFold::Whole),
        // -------------------------------------------------------------------
        // GROUP 4 — ROTATE. O126, and read the module header's ⚠ before
        // touching it: this is what puts a real document edit in Read mode.
        //
        // Its own group rather than two more rows under `select`, because a
        // rotation is an act performed on the document and a selection tool is
        // a mode the pointer is in. One caption cannot be true of both.
        //
        // `RailFold::Whole`, so the pair folds together — half a rotate group
        // is a control whose partner the operator has to go and find.
        // -------------------------------------------------------------------
        RailGroup::new(
            "rotate",
            [
                Item::command("pages.rotate_left"),
                Item::command("pages.rotate_right"),
            ],
        )
        .with_caption(t::group_rail_rotate())
        .with_fold(RailFold::Whole),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shipped registry. Built here rather than borrowed from
    /// `shell::commands`' own test module, which is private to that module.
    fn catalog() -> egui_shell::CommandRegistry {
        let mut reg = egui_shell::CommandRegistry::new();
        crate::shell::commands::register(&mut reg);
        reg
    }

    /// ★★ Every id in the rail is a registered command.
    ///
    /// `Shell::validate` enforces this at start-up and would refuse the whole
    /// manifest; this test says so at `cargo test` time instead, because a
    /// rail typo's symptom — a hole in permanent chrome — is one an operator
    /// meets before a developer does.
    #[test]
    fn every_rail_id_is_a_registered_command() {
        let registry = catalog();
        for group in groups() {
            for item in &group.items {
                if let Item::Command { id, .. } = item {
                    assert!(
                        registry.get(id).is_some(),
                        "rail group `{}` names unregistered command `{id}`",
                        group.id
                    );
                }
            }
        }
    }

    /// ★★★ Every id in the rail names an icon.
    ///
    /// The rail's row is a picture with an *optional* word under it, and at
    /// `Rung::Tight` and below there is no word left. A command with no icon
    /// would draw a blank rectangle there — which is the mockup legend's own
    /// objection, and the reason `edit.select_all` could not be in the rail
    /// until 2026-09-04.
    ///
    /// ⚠ This test is what stops that objection from lapsing in the other
    /// direction: an icon *removed* from any rail command turns a row into a
    /// blank, and nothing else in the build would notice.
    #[test]
    fn every_rail_id_names_an_icon() {
        let registry = catalog();
        for group in groups() {
            for item in &group.items {
                if let Item::Command { id, .. } = item {
                    let command = registry.get(id).expect("registered");
                    assert!(
                        command.icon.is_some(),
                        "rail group `{}` command `{id}` has no icon, and a rail row \
                         with neither picture nor word is a blank",
                        group.id
                    );
                }
            }
        }
    }

    /// ★ The panel-tab group never folds, and it is the only one that does not.
    ///
    /// Pinned as data rather than trusted to the planner, because the planner
    /// honours whatever this file declares — a `fold` typo here would be a
    /// silent downgrade of the rail's only structural promise.
    #[test]
    fn only_the_panel_tabs_are_marked_never_folding() {
        let groups = groups();
        assert_eq!(groups[0].id, "tabs");
        assert_eq!(groups[0].fold, RailFold::Never);
        assert_eq!(groups[0].items.len(), 5, "all five panels, one click away");
        for group in &groups[1..] {
            assert_ne!(
                group.fold,
                RailFold::Never,
                "group `{}` claims the floor, and there is only one floor",
                group.id
            );
        }
    }

    /// ⚠ **Rotate is in the rail with no mode gate**, which is what lets Read
    /// dirty a document. His call — O126. Pinned so that removing the gate's
    /// absence is a deliberate act rather than a drive-by.
    #[test]
    fn rotate_is_ungated_and_therefore_reachable_in_read() {
        let group = groups()
            .into_iter()
            .find(|g| g.id == "rotate")
            .expect("the rotate group");
        for item in &group.items {
            assert_eq!(
                item.visible_condition(),
                None,
                "rotate is available in every mode including Read — O126"
            );
        }
    }

    /// The Points tool is withheld outside Edit, on the rail as on the ribbon.
    #[test]
    fn the_points_tool_is_withheld_outside_edit() {
        let group = groups()
            .into_iter()
            .find(|g| g.id == "navigate")
            .expect("the navigate group");
        let node = group
            .items
            .iter()
            .find(|i| matches!(i, Item::Command { id, .. } if id == "view.tool_node"))
            .expect("the points tool");
        assert_eq!(node.visible_condition(), Some("mode.edit_content"));
    }

    /// ★ The lasso is absent, and this test is the tripwire for the day it is
    /// added: R9 forbids drawing a capability the build does not have, so a
    /// `edit.lasso` id appearing here before the command is registered would
    /// otherwise be caught only by `Shell::validate` at start-up.
    #[test]
    fn the_lasso_is_absent_until_the_command_exists() {
        let registry = catalog();
        let in_rail = groups().iter().any(|g| {
            g.items
                .iter()
                .any(|i| matches!(i, Item::Command { id, .. } if id == "edit.lasso"))
        });
        assert_eq!(
            in_rail,
            registry.get("edit.lasso").is_some(),
            "the rail draws the lasso exactly when the lasso exists"
        );
    }
}
