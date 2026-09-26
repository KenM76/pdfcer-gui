//! # `railmanifest` — what is in the left rail
//!
//! `OPERATOR_REQUESTS.md` **O123** part 7, his words:
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/railmanifest.md`.

use egui_shell::manifest::{Item, RailFold, RailGroup};

use crate::text::ribbon as t;

/// The rail's groups, top to bottom.
#[must_use]
pub fn groups() -> Vec<RailGroup> {
    vec![
        // -------------------------------------------------------------------
        // GROUP 1 — THE PANEL TABS. The floor.
        //
        // `RailFold::Never`, and it is the load-bearing choice on this
        // strip. O123 part 5 put Pages, Bookmarks, Layers, Signatures and
        // Fonts into ONE dock, and Comments joined them here; the rail is what
        // makes every one of them simultaneously one click away, which a 280 pt
        // horizontal tab bar cannot do — three fit and the rest go behind a
        // chevron. A rail that folded these would be
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
                // **COMMENTS — this row is Read's only route to the
                // comment list.**
                //
                // The Comments panel's single command is `markup.comments`,
                // which lives on the **Markup** tab, and Read's mode table is
                // `["file", "view"]` alone. Without this row a reader can
                // neither open the comment list nor reopen it after closing
                // it. That posture is backwards: Acrobat *Reader* is a
                // read-only product and reading comments is its entire
                // purpose. Read's stance is about **authorship**, not about
                // information — a mode that may not write a comment may
                // certainly read one somebody else wrote.
                //
                // ⚠⚠ **It is on the RAIL and not on View ▸ Panels, and that
                // is forced, not preferred.** `RIBBON_IA.md` P1 —
                // *one command appears on at most one tab* — is enforced by
                // `Shell::validate`, so adding this id to the View tab beside
                // the other panel toggles is a **validation failure**, not a
                // duplicate control. And a manifest that fails to validate
                // does not merely lose the item: `Capabilities::for_mode`
                // falls back to `FULL` when the shell is absent, so the whole
                // build silently gains every authoring capability in every
                // mode. **The failure is not local to the item being added**,
                // which is why the trap is written here rather than left to be
                // met.
                //
                // The rail is not a tab, so P1 does not reach it — the same
                // permission the four toggles above rely on, recorded at the
                // head of this group: *a shortcut to a known home is not a
                // second place to hunt.* And unlike a tab, the rail is present
                // in every mode, which is precisely the property this needs.
                //
                // ⬜ A rename to `view.panel_comments` would let the id match
                // its family and put the tab-side toggle with the other panel
                // toggles. It touches `Panel::command_id`, the dispatch arm,
                // the catalog, the token block, the ladder and the mock. The
                // rail placement here is wanted either way; the rename would
                // move only the second, tab-side control.
                Item::command("markup.comments"),
                // `file.fonts`, NOT `view.panel_fonts` — the Fonts panel's command
                // is registered on the File tab and there is no second id for
                // it. `pdfcer_gui::panels::Panel::command_id` is the source of
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
        // `view.tool_node` carries `shown_when("mode.edit_content")`, the
        // same condition the ribbon item carries and for the same reason (O69,
        // R9): the Points tool edits the nodes of a path and Read cannot, so
        // in Read it renders NOTHING rather than greying. On a permanent
        // surface that matters more than on a tab — a control that is wrong
        // for the mode would be wrong on screen for the whole session rather
        // than for one click.
        //
        // `view.smart_select` IS a member of this group, on the
        // operator's instruction: *"our smart selector should be visible with
        // the other navigate controls in our left rail."*
        //
        // ⚠ **Unpinnable is not the same as absent, and the two are easy to
        // conflate.** It is a TOGGLE that changes what the arrow selects
        // rather than a tool you can be holding, so it has no armed state
        // worth pinning at the floor of the ladder — but that is a fact about
        // the PIN, not about MEMBERSHIP. Read as membership it would exclude
        // `edit.select_all` too, and that is in the rail. `super::view`'s
        // Navigate group carries this exact toggle beside these exact four
        // tools, arguing the placement: *"it changes what the arrow at the
        // head of this row selects when you click with it… this changes what
        // a gesture MEANS."* The rail mirrors that group row for row; leaving
        // one member out would make the two surfaces disagree, which is the
        // thing the order note above this list exists to prevent.
        //
        // **WHAT HAPPENS AT THE FOLD, stated rather than discovered.**
        // [`RailFold::PinArmed`] pins the row whose `selected:<id>` condition
        // holds, taking the FIRST such row in list order. `app::conditions::
        // armed` sets `selected:view.smart_select` whenever the preference is
        // on, so at `Rung::Cramped` with the arrow armed and smart select on
        // there are two candidates — and the tool wins, because it is listed
        // first. **That is the decision:** the pinned row answers *"what does
        // a drag do?"*, a toggle cannot answer that, so a toggle never takes
        // the pin.
        // The smart selector then joins `folded` with the rest of the group
        // and is one click away behind the chevron — it does not vanish, and
        // its state is still legible on View ▸ Navigate, which is on screen in
        // every mode that shows the row at all.
        //
        // It carries `shown_when("mode.edit_content")`, the same gate the
        // ribbon item carries, for `view.tool_node`'s reason two paragraphs
        // up: the command also carries `enabled_when("mode.edit_content")`, so
        // a rail row without the gate would be a permanently greyed control on
        // a permanent surface in Read.
        //
        // `RailFold::PinArmed`: at the floor of the ladder this group
        // becomes ONE row showing whatever is armed — including a tool armed
        // from a ribbon tab that is not open, because the pinning reads
        // `selected:<id>`, which is application state rather than a property
        // of the visible tab. A rail that cannot say what you are holding has
        // given up the job `pdfcer_gui::app::toolstatus` handed it.
        // -------------------------------------------------------------------
        RailGroup::new(
            "navigate",
            [
                Item::command("view.tool_select"),
                Item::command("view.tool_node").shown_when("mode.edit_content"),
                Item::command("view.tool_text"),
                Item::command("view.tool_hand"),
                Item::command("view.smart_select").shown_when("mode.edit_content"),
                Item::command("view.text_chunks").shown_when("mode.edit_content"),
            ],
        )
        .with_caption(t::group_view_navigate())
        .with_fold(RailFold::PinArmed),
        // -------------------------------------------------------------------
        // GROUP 3 — SELECT. His *"other related selection controls"*.
        //
        // THE LASSO GOES HERE, AND NOWHERE ELSE, WHEN IT EXISTS.
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
