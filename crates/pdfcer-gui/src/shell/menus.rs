//! # shell::menus — pdfcer's context menus, as data
//!
//! [`built_in`] returns every context menu pdfcer defines, as an
//! `egui_shell::menu::Menus` value carried on the same
//! [`egui_shell::Shell`] the ribbon is. [`MenuHost`] is the one place a
//! right-click site turns that data into drawn rows and invoked commands.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/shell/menus.md`.

use egui_shell::manifest::{Item, Shell};
use egui_shell::menu::{Menu, Menus};
use egui_shell::{CommandRegistry, ConditionSet, HandlerToken};

// ===========================================================================
// Context ids
// ===========================================================================
//
// Constants rather than literals at the call sites, because a context id is
// used in exactly two places that must agree — the document below, and the
// right-click site in `canvas` or `panels` — and a typo in either produces
// silence rather than an error. `Menu::attach` treats an unknown context as
// "this surface has no menu yet", which is the correct behaviour for a
// surface that genuinely has none and an undebuggable one for a surface
// that was meant to have one.
//
// Dotted lowercase, matching command ids. The shell enforces no shape.

/// Right-click on the page, over an object.
///
/// The **selection** menu of `RIBBON_IA.md` §5.8: act on what is selected.
pub const CANVAS_OBJECT: &str = "canvas.object";

/// Right-click on the page, over nothing.
///
/// The **view** menu: with no object under the pointer there is nothing to
/// act *on*, so the menu is about how the page is shown.
pub const CANVAS_EMPTY: &str = "canvas.empty";

/// **Reading, over a picture** — `OPERATOR_REQUESTS.md` O71.
pub const CANVAS_READ_OBJECT: &str = "canvas.read-object";

/// Right-click on the page **with a caret placed in existing text**.
pub const CANVAS_TEXT: &str = "canvas.text";

/// Right-click on the page **over a form field**.
pub const CANVAS_FIELD: &str = "canvas.field";

/// Right-click on the page **over a selected markup shape**.
pub const CANVAS_MARKUP: &str = "canvas.markup";

/// Right-click on the page **over a selected ce dimension**. Its own menu, not
/// the markup one: a ce dimension reaches a different verb family (R8b rule
/// 15), so the markup rows — node edits, flatten, the markup clipboard — do not
/// apply to it.
pub const CANVAS_DIMENSION: &str = "canvas.dimension";

/// Right-click on the page **inside the snapshot box**.
pub const CANVAS_SNAPSHOT: &str = "canvas.snapshot";

/// **The right-click landed on a segment of a shape that can take a new
/// point** — the `visible_when` of `markup.add_node`.
pub const NODE_INSERT_OFFERED: &str = "markup.node_insert_offered";

/// **…and inserting there would actually be allowed** — the `enabled_when` of
/// `markup.add_node`, carried on the command rather than on the item because
/// `Item` has no enablement field and enablement is the registry's.
///
/// The gap between this and [`NODE_INSERT_OFFERED`] is the greyed row: drawn,
/// unpressable, explaining itself on hover. R9.
pub const NODE_INSERTABLE: &str = "markup.node_insertable";

/// **The right-click landed on an existing point** — the `visible_when` of
/// `markup.remove_node`. [`NODE_INSERT_OFFERED`]'s twin; see it for why these
/// live here and not in `PdfcerApp::conditions`.
pub const NODE_REMOVE_OFFERED: &str = "markup.node_remove_offered";

/// **…and removing it would not breach the shape's vertex floor** — the
/// `enabled_when` of `markup.remove_node`.
pub const NODE_REMOVABLE: &str = "markup.node_removable";

/// **The engine would burn the right-clicked markup into its page** — the
/// `visible_when` of `markup.flatten`, asked from
/// `EditSession::annotation_flatten_refusals` when the menu opens.
pub const FLATTEN_OFFERED: &str = "markup.flatten_offered";

/// The selection is one circular ce dimension showing its radius: the
/// `visible_when` and `enabled_when` of `format.dimension_diameter`.
pub const DIMENSION_DIAMETER_OFFERED: &str = "dimension.diameter_offered";

/// The selection is one circular ce dimension showing its diameter: the
/// `visible_when` and `enabled_when` of `format.dimension_radius`.
pub const DIMENSION_RADIUS_OFFERED: &str = "dimension.radius_offered";

/// The selection is one closed perimeter ce dimension showing its perimeter:
/// the `visible_when` and `enabled_when` of `format.dimension_area`.
pub const DIMENSION_AREA_OFFERED: &str = "dimension.area_offered";

/// The selection is one closed perimeter ce dimension showing its area: the
/// `visible_when` and `enabled_when` of `format.dimension_perimeter`.
pub const DIMENSION_PERIMETER_OFFERED: &str = "dimension.perimeter_offered";

/// **The right-click landed on one line of a MULTI-LINE text object** — the
/// `visible_when` of `format.select_text_line`, and its `enabled_when` too.
pub const RUN_SELECT_OFFERED: &str = "canvas.run_select_offered";

/// **The selection is two or more runs of one page text object** — the
/// `visible_when` of `format.merge_text_runs` on the canvas menu. Published by
/// `PdfcerApp::conditions` and corrected per right-click by
/// [`crate::canvas::menus`], both from [`crate::canvas::runmerge::operand`].
pub const TEXT_MERGE_OFFERED: &str = "selection.text_merge_offered";

/// **The engine's preflight would merge those runs** — the command's
/// `enabled_when`. Offered but not allowed is drawn greyed (R9), because the
/// operator can change the selection and try again.
pub const TEXT_MERGE_ALLOWED: &str = "selection.text_merge_allowed";

/// **The selection is one page text object of two or more lines** — the
/// `visible_when` of `format.split_text_lines` on the canvas menu, from
/// [`crate::canvas::runsplit::operand`], parked per right-click like
/// [`TEXT_MERGE_OFFERED`].
pub const TEXT_SPLIT_OFFERED: &str = "selection.text_split_offered";

/// **No line of that object inherits its position** — the command's
/// `enabled_when`. The rest of the engine's refusals need the content stream,
/// which the object model does not carry, so they reach the status line on
/// the press instead.
pub const TEXT_SPLIT_ALLOWED: &str = "selection.text_split_allowed";

/// **The selection can be moved to a layer** — `format.move_to_layer`'s
/// `enabled_when` and its canvas-menu `shown_when`. Published by
/// `PdfcerApp::conditions` and corrected per right-click by
/// [`crate::canvas::menus`], both from
/// `crate::app::actions::layerassign::offered`.
pub const LAYER_ASSIGNABLE: &str = "selection.layer_assignable";

/// Right-click on a panel tab in the dock.
///
/// Defined but not attachable from this crate — see the module header.
pub const DOCK_TAB: &str = "dock.tab";

/// **The panel under this tab is docked**, so it can be floated.
pub const PANEL_DOCKED: &str = "panel.docked";

/// **The panel under this tab is in a window of its own**, so it can be
/// docked back. The complement of [`PANEL_DOCKED`]; exactly one of the two
/// holds for any panel that is being drawn at all.
pub const PANEL_FLOATING: &str = "panel.floating";

/// Right-click on an object row in the Objects panel.
pub const OBJECTS_ROW: &str = "objects.row";

/// Right-click on a page tile in the Pages panel.
pub const PAGES_ROW: &str = "pages.row";

/// **A document tab in the strip under the ribbon** — not a dock tab.
pub const DOCUMENT_TAB: &str = "document.tab";

/// Every context id this module defines, for the sweeps in [`tests`].
pub const CONTEXTS: &[&str] = &[
    CANVAS_OBJECT,
    CANVAS_READ_OBJECT,
    CANVAS_EMPTY,
    CANVAS_TEXT,
    CANVAS_FIELD,
    CANVAS_MARKUP,
    CANVAS_DIMENSION,
    CANVAS_SNAPSHOT,
    DOCK_TAB,
    DOCUMENT_TAB,
    OBJECTS_ROW,
    PAGES_ROW,
];

// ===========================================================================
// The document
// ===========================================================================

/// **Every context menu pdfcer defines.**
#[must_use]
pub fn built_in() -> Menus {
    Menus::new()
        // -------------------------------------------------------------------
        // canvas.object — the selection menu.
        //
        // ONE item, and the three that `RIBBON_IA.md` §6 also asks for are
        // ABSENT rather than greyed.
        //
        // §6: "carrying the same commands as its Format tab section plus
        // Cut/Copy/Paste/Delete". The Format tab section is `format.delete`
        // and nothing else — every property editor in §5.8's table is **N**
        // and sits in `manifest::PLANNED`, twenty-four entries from that one
        // section. Cut, copy and paste need an **object clipboard**, which
        // this build does not have in any form:
        //
        //   edit.cut   "N — there is no object clipboard. The two text-copy
        //               commands in File ▸ Export are a different mechanism
        //               and do not imply one."
        //
        // So a faithful reading of §6 would produce a menu of one live row
        // and three dead ones. P3 says the dead ones render nothing, and the
        // menu engine's own rule 1 says the same in the other direction: an
        // unregistered id is absent, a registered-but-inapplicable one is
        // greyed. `edit.cut` is not registered. It is absent.
        //
        // No separator: a rule is punctuation between *kinds*, and one item
        // has no kinds to separate. (The engine would collapse a leading or
        // trailing rule anyway — `plan::collapse` — which is exactly why a
        // stale document degrades into a clean menu rather than into two
        // horizontal lines above one row.)
        // -------------------------------------------------------------------
        // **Zoom to selection is here because that is where two of the three
        // reference applications put it.**
        //
        // Operator instruction, 2026-08-14: *"make your best educated guesses
        // to match what inkscape, acrobat, and SolidWorks do."* Applied to the
        // open question of how `view.zoom_selection` is reached:
        //
        // | | how it is reached |
        // |---|---|
        // | **SolidWorks** | right-click ▸ Zoom to Selection. No default chord. |
        // | **Acrobat** | View ▸ Zoom menu, and the marquee-zoom tool. No default chord. |
        // | **Inkscape** | a chord — bare `3`, in its zoom family `1`–`6`. |
        //
        // Two of three reach it from a menu, so it goes in the menu. **No chord
        // is invented**, and the reason is not only the two-to-one split:
        // Inkscape's family is *bare digits*, this shell's manifest chords are
        // `Ctrl`-modified by construction (`app::keyboard::commands` refuses a
        // frame without `command`), and its `Ctrl+1`/`2`/`3` are the
        // Read/Review/Edit selector that `MODES_AND_PANELS.md` Part 1 §6
        // specifies. Transposing Inkscape's `3` onto `Ctrl+4` would match the
        // letter of neither convention and the muscle memory of nobody.
        //
        // This also gives the command a **second reachable route**, which it
        // needed for a reason worth recording: its ribbon control is
        // `enabled_when("selection.bounds")`, so it is greyed exactly when it
        // would decline — see `app::status::decline`. The menu does not change
        // that (a menu on an object implies a selection), so the decline stays
        // reachable only by the race in which bounds evaporate between the
        // frame that drew the enabled control and the frame that applied it.
        // That is now a **decided** outcome rather than an open question: the
        // decline is a race-only safety net, and it is correct for it to be
        // rare.
        //
        // Ordered zoom-then-delete because `RIBBON_IA.md` §5.8's rule for
        // menus is least-destructive first, and Delete is the one entry here
        // that cannot be undone by looking somewhere else.
        .with(Menu::new(CANVAS_OBJECT).with_items([
            Item::command("view.zoom_selection"),
            //
            // It is here because it is where the operator actually looks. A ce
            // dimension's group, its measured value, its eleven inherited-or-
            // overridden settings and its radius/diameter switch are otherwise
            // reachable only by knowing that a contextual **Format** tab
            // appeared, or by opening a dock panel by name — and the operator's
            // report that started this work was *"I click and can't figure out
            // how to enable some of the basic stuff."*
            //
            // Above Delete, which stays last: the destructive row is last in
            // every menu in this file, deliberately.
            Item::command("format.properties"),
            //
            // It matters more here than on the ribbon, and the reason is where
            // the operator's hand is: they have just clicked something inside a
            // form, found they cannot move it, and the next thing they do is
            // right-click it. A control on a contextual tab three inches away
            // is the correct *second* home, not the first one they will find.
            //
            //
            // The operator asked to *"move the individual text blocks"* inside
            // a grouped block of text *"and have the ability to delete them"*.
            // Delete shipped — `EditSession::delete_text_run`, at the Part rung
            // — and then the note filed with it recorded the part that had not:
            // **the rung itself was reachable by exactly one chord-armed
            // gesture and was announced on no surface.** A capability nobody
            // was told about has not shipped.
            //
            // ## Why a row that RE-AIMS is allowed where a row that EDITS is not
            //
            // `DESIGNS.md` §6.2 forbids a context-menu item by name, and that
            // ruling stands: it is about the *move* half, whose premise is
            // "use the conventional interaction, never invent one" — and the
            // conventional way to move a piece of a thing is to drag it, not to
            // pick a row called *Move*. This row performs no edit. It changes
            // what is selected, which is the same act as the two rows below it,
            // for the same reason: the operator's hand is already on the thing.
            //
            // Nor does it breach `canvas::menus`' rule that a right-click never
            // descends. The click still selects the whole object; the descent
            // happens only when somebody reads a row that says so and presses
            // it.
            //
            // ## Placement: the two Select rows read as a pair
            //
            // Directly above `format.select_form`, which is the group's
            // describe → re-aim → detach → destroy order with both re-aims
            // together. Descent above ascent, because the thing the pointer is
            // ON is nearer than the thing it is inside.
            //
            // ## Absent, not greyed
            //
            // On `RUN_SELECT_OFFERED`, which is one condition rather than the
            // offered/enabled pair the node rows carry, for the reason stated
            // at the constant: there is no recoverable state here. See also
            // `crate::canvas::runmenu`'s `of > 1` gate — on a single-run text
            // object this row would descend to a place indistinguishable from
            // where the operator already stands, which is a row that appears to
            // do nothing.
            Item::command("format.select_text_line").shown_when(RUN_SELECT_OFFERED),
            Item::command("format.select_form"),
            // The right-click route to *"give this page its own copy"*,
            // added 2026-08-28 with the form-XObject unshare.
            //
            // **O53's ruling is why it is here at all**: a command must not
            // exist only on the ribbon. That rule is doing more work for this
            // command than for most, because the operator who needs it is by
            // definition mid-gesture — they have just clicked inside a title
            // block, they are about to type into it, and the moment they need
            // to be offered a private copy is *before* that keystroke. A
            // contextual tab three inches away is the correct second home; the
            // pointer is the first.
            //
            // It is also the only surface that can reach them in time. The
            // engine's SHARED CONTENT disclosure fires **after** an edit has
            // fanned out to every sheet; this row is the one place the choice
            // is offered while it is still a choice.
            //
            // Directly under `format.select_form`, matching the ribbon group's
            // order for the reason argued there: describe, re-aim, detach,
            // destroy — and Delete stays last, as it does in every menu in this
            // file.
            //
            // Greyed rather than absent when the selection is not inside a
            // form, by the same R9 reading the catalog entry argues, and on the
            // same `selection.in_form` predicate as the row above it.
            Item::command("format.unshare_form"),
            Item::command("format.merge_text_runs").shown_when(TEXT_MERGE_OFFERED),
            Item::command("format.split_text_lines").shown_when(TEXT_SPLIT_OFFERED),
            // **Mark what was pointed at for redaction**, at the pointer —
            // `OPERATOR_REQUESTS.md` O217, whose requirement is that redaction
            // address *the same unit, by the same gestures* as everything else
            // the operator aims at.
            //
            // ## The unit is already right; the route is what this row supplies
            //
            // `app::actions::redactsel::mark_selection` builds its quads from
            // `SelectionState::outlines()`, and `SelectionState::outline_rect`
            // answers `part_bounds` whenever the entry carries a subpath. So a
            // selection standing at the **Part** rung on one chunk of a text
            // block marks that chunk's box and never the block's, and a set
            // built by shift-click or a rubber band marks one `/Redact` with
            // one quad per held chunk. Nothing about the unit is restated here
            // — the mark is the outline the operator can see, which is the
            // whole of `redactsel`'s "why the bounds and not the shape".
            //
            // O53 is the rest of the argument: **a command must not exist only
            // on the ribbon.** The operator who needs this one has just clicked
            // a chunk and is looking at it; Edit ▸ Protect is the correct
            // *second* home.
            //
            // ## Why a marking row is allowed where `DESIGNS.md` §6.2 bans an
            // editing one
            //
            // That ruling is about inventing a menu row for a gesture the
            // conventional interaction already owns — *Move* instead of a drag.
            // There is no conventional gesture that marks a region for
            // redaction; Acrobat's own route is the right-click on the
            // selection, which is this row.
            //
            // ## Placement: last of the non-destructive rows
            //
            // A `/Redact` annotation **removes nothing** — applying is
            // `edit.redact_apply`, a separate confirmed act — so this is not
            // the destructive row and does not displace Delete, which stays
            // last here as it does in every menu in this file. It sits
            // immediately above it because it is the act that *precedes* the
            // irreversible one, and the group reads describe → re-aim → detach
            // → mark → destroy.
            //
            // ## Greyed, not absent, and no mode gate
            //
            // The catalog's `selection.any` is the only gate, and it cannot be
            // false here: this menu is chosen because an object is under the
            // pointer. No mode predicate is added, and that is not an omission
            // — `canvas::rightclick` computes `reading = !caps.edit_content`
            // and downgrades every mode that cannot edit content to
            // `CANVAS_READ_OBJECT`, so this menu is reachable in Edit alone,
            // which is the same mode the Edit tab's Protect group is shown in.
            // A second statement of that fact here would be a second thing to
            // keep in step.
            Item::command("edit.redact_selection"),
            // **Absent, not greyed, where the engine would refuse it.** The
            // same condition and the same constant the Format tab's Delete
            // carries — `manifest::format::DELETE_VISIBLE_WHEN` — so this menu
            // and that ribbon group cannot disagree about whether the operator
            // is offered a Delete on a certified drawing.
            //
            // ⇒ This is the second of the two live-and-inert routes the
            // `annotation_deletion_refusal` audit found: right-clicking a
            // comment on a certified sheet opened a menu with a working-looking
            // Delete, and pressing it wrote one line to the trace and said
            // nothing. `panels::properties::annotdelete` carries the finding and
            // the sentence that replaces the control.
            Item::command("format.move_to_layer").shown_when(LAYER_ASSIGNABLE),
            Item::command("format.delete").shown_when(super::manifest::DELETE_PERMITTED),
        ]))
        // -------------------------------------------------------------------
        // canvas.read-object — a picture, while reading.
        //
        // `OPERATOR_REQUESTS.md` O71: *"In read mode the regular pointer
        // should also allow us to select images so we can copy and paste them
        // … outside of the pdfcergui."*
        //
        //
        // TWO rows, and the shortness is the design. Every other row of
        // `canvas.object` edits — Delete, unshare, re-aim, the Properties
        // panel's editable fields — and R9's answer to *"this mode cannot"* is
        // to render nothing rather than to grey a list. So this menu offers the
        // two things a reader can genuinely do with a picture: take a copy, and
        // look closer.
        //
        // ORDER: copy first. It is why the menu exists and it is what the
        // operator came for; zoom-to-selection is the useful neighbour, not the
        // headline. Neither is destructive, so the least-destructive-first rule
        // that orders the other menus has nothing to say here.
        .with(Menu::new(CANVAS_READ_OBJECT).with_items([
            Item::command("edit.copy"),
            Item::command("view.zoom_selection"),
        ]))
        // -------------------------------------------------------------------
        // canvas.snapshot — inside the snapshot box: copy it, or save it as a PDF.
        .with(Menu::new(CANVAS_SNAPSHOT).with_items([
            Item::command("edit.copy"),
            Item::command("view.snapshot_save_pdf"),
        ]))
        // -------------------------------------------------------------------
        // canvas.empty — the view menu.
        //
        // Right-clicking paper is not a question about an object, because
        // there is not one; it is a question about the view. The three named
        // zoom levels are the view commands that exist AND have a live
        // dispatch arm in `PdfcerApp::dispatch_token` today, so every row here
        // does something the moment it is clicked.
        //
        // ORDER: fit page, fit width, actual size — deliberately not the View
        // ▸ Zoom band's order (actual, fit page, fit width). The band reads as
        // a scale progression, top to bottom, because it is a band and the
        // eye reads it as a set. A menu at the pointer is read as a list of
        // verbs in likelihood order, and on a drawing sheet the overwhelmingly
        // most-wanted answer to "I have lost my place" is **fit page**.
        //
        // What is NOT here, and why each was considered:
        //
        //   view.show_annotations   A real, wired toggle — but it is a
        //                           *display* setting rather than an act on
        //                           what was pointed at, and a menu that
        //                           starts collecting settings stops being a
        //                           list of verbs. It is one click away on
        //                           View ▸ Display.
        //   view.zoom_selection     N. Zoom to the selection's bounding box —
        //   view.zoom_region        N. Marquee zoom. Both in PLANNED, and both
        //                           are the commands that would most obviously
        //                           belong here when they land.
        //   pages.* / edit.*        Act on the page or its content, which is
        //                           what `canvas.object` is for.
        // -------------------------------------------------------------------
        .with(Menu::new(CANVAS_EMPTY).with_items([
            Item::command("view.zoom_fit_page"),
            Item::command("view.zoom_fit_width"),
            Item::command("view.zoom_fit_height"),
            Item::command("view.zoom_actual"),
        ]))
        // -------------------------------------------------------------------
        // canvas.field — the form field's menu.
        //
        // TWO items, and the pair is chosen by what an operator does to a
        // field they have just placed: they check its settings, or they got rid
        // of it. Properties first, destructive last — the ordering rule every
        // menu in this file follows.
        //
        // **Rename is absent and its absence is not an oversight.** It
        // lives in the Properties panel as a draft box with an explicit commit,
        // because renaming a field on every keystroke would author one real,
        // separately-undoable rename per character. A menu item cannot ask for
        // text, so `format.properties` IS the rename route — one click further
        // and honest about it, rather than a second half-implemented rename
        // that could disagree with the first.
        //
        // `format.delete` removes THIS BOX, not the whole field. A field with
        // two widgets on two pages is one field selectable from either place,
        // and the panel offers both deletions labelled. See `dispatch::format`.
        //
        // **`shown_when` — and its absence here was the second half of the
        // R83 forms defect, left open for a day by the fix that closed the
        // first.**
        //
        //
        // It is the SAME condition as `canvas.object`'s, deliberately, and it
        // is correct for both because `app::conditions` publishes it from a
        // ladder that asks the forms query when a field is selected and the
        // annotation query otherwise — the same precedence
        // `app::dispatch::format` resolves the command by. One name, one
        // meaning: *deleting what is selected would not be refused*.
        .with(Menu::new(CANVAS_FIELD).with_items([
            Item::command("format.properties"),
            Item::command("format.move_to_layer").shown_when(LAYER_ASSIGNABLE),
            Item::command("format.delete").shown_when(super::manifest::DELETE_PERMITTED),
        ]))
        // -------------------------------------------------------------------
        // canvas.markup — a placed markup shape's menu.
        //
        //
        //   "I also can't edit or delete nodes of a markup shape once it is
        //    drawn."
        //
        // Half of that was answered the same day: `canvas::annotnodes` moves,
        // inserts and removes vertices through `Pass 255.0`'s verbs. But insert
        // and remove needed the Points tool armed **plus** `Ctrl` or
        // `Ctrl+Shift`, and nothing on screen said so — a capability only
        // somebody who was told about it can use, which is the same shape as
        // O71's chord-only Copy Image one menu above. The note filed with that
        // work named the fix and named this file as the reason it was not built:
        //
        //   "The natural way to add or remove a corner is a right-click on the
        //    shape — 'add a point here', 'remove this point' — which is how the
        //    engine itself describes these two operations. The chords above are
        //    a stopgap."
        //
        // ## Why not `canvas.object` with two rows added
        //
        // Because that menu is FIVE rows and only three of them mean anything
        // on an annotation. `format.select_form` and `format.unshare_form` are
        // about page content painted from inside a form XObject; a markup
        // annotation is never inside one, so both would draw, resolve and do
        // nothing. R9's answer to a permanently inapplicable control is nothing
        // rather than greying, and `canvas.read-object`'s own note two menus
        // above settled the precedent: when a majority of a menu's rows do not
        // apply to a subject, the subject gets a context, not a filter.
        //
        // `view.zoom_selection` is absent for a sharper reason than taste: it is
        // gated on `selection.bounds`, which `app::conditions` publishes from
        // `canvas::zoom::can_zoom_to_selection` → `SelectionState::outline_union`
        // → the **content** outline map. An annotation selection carries its
        // outline on `AnnotSelection` and puts nothing in that map, so the row
        // would be greyed on every markup shape there has ever been. A
        // permanently greyed row is a promise the build cannot keep; when
        // zoom-to-selection learns to frame an annotation it belongs here, first.
        //
        // ## The order, and the two rules it obeys
        //
        // 1. **Describe, then act, then destroy** — the same progression
        //    `canvas.object` uses and for the same reason.
        // 2. **The destructive row is last in every menu in this file.**
        //
        // So: what is it (`format.properties`) · the two node verbs, which are
        // why this menu exists · the clipboard · Delete.
        //
        //
        // ## The two node rows: `shown_when` AND greying, on one row
        //
        // This is the only pair in the file that uses both halves of R9 at once,
        // and it has to, because the same command is permanently inapplicable on
        // one shape and temporarily unavailable on another:
        //
        // | shape, and where the pointer is | Remove this point |
        // |---|---|
        // | a `/Square`, anywhere | **absent** — it will never have points |
        // | a five-corner polygon, on a corner | live |
        // | a **three**-corner polygon, on a corner | **greyed**, tooltip names the floor |
        // | a five-corner polygon, in its middle | absent — no point was pointed at |
        //
        // Neither answer is hard-coded here or in the canvas. Both come from
        // `EditSession::reshape_annotation_preview`, asked with the exact
        // `VertexEdit` the row would commit, and the **error variant** is what
        // separates the greyed case (`ReshapeWouldBreachVertexFloor` — draw
        // another corner and it comes back) from the absent one. See
        // `crate::canvas::annotnodes::menu`, which is where that is decided and
        // where the two `visible_when` conditions below are set per click.
        //
        // ## The clipboard rows
        //
        //
        // `edit.paste_duplicate` is deliberately absent even though it is
        // registered: over a markup clipboard `dispatch::clipboard` falls it
        // through to plain paste, so the row would be a second Paste under a
        // different name. Its subject is a form field, and `canvas.field` is
        // where it would belong the day that menu grows a clipboard group.
        //
        // `edit.cut` carries no `shown_when` here, unlike `format.delete`
        // below it, and the asymmetry is the registry's rather than this file's:
        // cut is gated by `selection.cut_permitted`, an `Enable::Custom` on the
        // command that clears for the things the clipboard cannot carry, so it
        // GREYS where it would refuse. Delete's refusal is a property of the
        // FILE — a certified or encrypted drawing — which is not temporary, so
        // that one disappears. Two refusals, two mechanisms, one reason each.
        .with(Menu::new(CANVAS_MARKUP).with_items([
            Item::command("format.properties"),
            Item::Separator,
            Item::command("markup.add_node").shown_when(NODE_INSERT_OFFERED),
            Item::command("markup.remove_node").shown_when(NODE_REMOVE_OFFERED),
            Item::Separator,
            Item::command("edit.cut"),
            Item::command("edit.copy"),
            Item::command("edit.paste"),
            Item::Separator,
            // The same condition and the same constant `canvas.object`'s
            // and `canvas.field`'s Deletes carry. `app::conditions` publishes it
            // from a ladder whose annotation rung is guarded by `author_markup`,
            // which is what keeps this row alive in Review — deleting a comment
            // is exactly what Review is for.
            Item::command("markup.flatten").shown_when(FLATTEN_OFFERED),
            Item::command("format.move_to_layer").shown_when(LAYER_ASSIGNABLE),
            Item::command("format.delete").shown_when(super::manifest::DELETE_PERMITTED),
        ]))
        // -------------------------------------------------------------------
        // canvas.text — the caret's menu.
        //
        // ONE item, and it is the one that has no other canvas route. Cut,
        // Copy and Paste are conspicuously absent and their absence is
        // deliberate: `edit.cut`/`edit.copy` act on the OBJECT selection, not
        // on a text draft's selected characters, so offering them here would
        // put three items on the menu of which two act on something other than
        // what the operator is pointing at. That is the `canvas.object`
        // select-first defect in a different costume.
        //
        // ⇒ When a draft-scoped cut and copy exist they belong here, above the
        // reflow, in the order every editor uses. Until then the menu is
        // honest at one item.
        .with(Menu::new(CANVAS_DIMENSION).with_items([
            Item::command("format.properties"),
            // One click where the operator's hand already is, for the switch
            // Properties also carries. At most one of the pair is drawn.
            Item::command("format.dimension_diameter").shown_when(DIMENSION_DIAMETER_OFFERED),
            Item::command("format.dimension_radius").shown_when(DIMENSION_RADIUS_OFFERED),
            Item::command("format.dimension_area").shown_when(DIMENSION_AREA_OFFERED),
            Item::command("format.dimension_perimeter").shown_when(DIMENSION_PERIMETER_OFFERED),
            Item::Separator,
            Item::command("format.move_to_layer").shown_when(LAYER_ASSIGNABLE),
            Item::command("format.delete").shown_when(super::manifest::DELETE_PERMITTED),
        ]))
        .with(Menu::new(CANVAS_TEXT).with_items([Item::command("edit.reflow_block")]))
        // -------------------------------------------------------------------
        // dock.tab — a panel tab.
        //
        // Defined, valid, merged and NOT ATTACHED. The dock owns its tabs'
        // secondary click inside `egui-shell` and offers no seam; the module
        // header carries the full account and what would close it.
        //
        // `Close` is deliberately absent. The dock's own hard-coded button
        // closes a tab through `dock::ctx::Intent::Close`, which is a dock
        // mechanism and not a pdfcer command: there is no `dock.close_panel`
        // in the registry, it is listed in `manifest::PLANNED`, and inventing
        // one here would name an id that resolves to nothing.
        //
        //
        // Each is `shown_when` a condition the tab handler sets PER TAB — see
        // `crate::app::surfaces`, which corrects the frame's condition set
        // once per drawn tab through `MenuHost::with_conditions`. So the menu
        // on a docked panel's tab offers Float and Close; the menu on a
        // floating panel's header strip offers Dock and Close; and neither
        // ever shows a row that would do nothing.
        //
        // `shown_when` and not `enabled_when`, which is R9 exactly: an
        // unavailable capability renders NOTHING. "Dock" on a panel that is
        // already docked is not temporarily unavailable — it is meaningless —
        // and a greyed row would make the operator wonder what they had to do
        // to earn it.
        //
        // Order: the two verbs that MOVE the panel first, the one that
        // takes it away last, and Reset layout below them because its
        // operand is the whole dock rather than this panel. Close is not
        // adjacent to Float, deliberately, so a mis-aimed click on the row
        // above Close costs a window rather than a panel.
        .with(Menu::new(DOCK_TAB).with_items([
            Item::command("view.panel_float").shown_when(PANEL_DOCKED),
            Item::command("view.panel_dock").shown_when(PANEL_FLOATING),
            Item::command("view.panel_close"),
            Item::Separator,
            Item::command("view.reset_layout"),
        ]))
        // -------------------------------------------------------------------
        // objects.row — a row in the Objects panel.
        //
        // The panel's own stated purpose is the operator's: "I'd like to have
        // a layer tree there for the document that I can also click on to
        // select objects. at least that way we can troubleshoot better what I
        // am clicking on in the GUI area." So the row's question is *what is
        // this*, and the Properties panel is the surface that answers it —
        // `file.properties`' own tooltip commissions exactly that: "…and the
        // properties of whatever is selected on the page."
        //
        // `format.delete` is NOT here. It was kept off while an Objects row
        // click wrote a panel-local focus rather than a selection: a Delete
        // gated on `selection.any` would then have removed whatever was
        // selected on the CANVAS rather than the row under the pointer. The row
        // click is a selection gesture now (`panels::objects` raises
        // `SelectionAction::SelectObject`), so that reasoning has expired and
        // the omission is unexamined. `DEFECTS.md` D48.
        // -------------------------------------------------------------------
        // -------------------------------------------------------------------
        // document.tab — the strip under the ribbon.
        //
        // TWO rows, where the conventional menu has three.
        //
        // Every browser and every editor offers *Close*, *Close others* and
        // *Close tabs to the right*.
        //
        // **Close is `file.close` itself**, not a second command — and that is
        // worth reading, because the second command was written first and two
        // gates refused it in the same run. `no_two_commands_share_a_label`
        // caught that it would carry `file.close`'s label, because it does
        // `file.close`'s job; `every_menu_command_is_also_reachable_from_the_ribbon`
        // caught that its only route would have been this right-click. Between
        // them they are right: what differs is not the *meaning* but the
        // **operand**, and an operand that comes from the surface a command was
        // invoked on is `crate::app::PdfcerApp::tab_menu_target`'s whole job.
        // From here it closes the tab you right-clicked; from the ribbon and
        // from `Ctrl+W` it closes the one on screen.
        //
        // It is also what stops this menu being **empty with one document
        // open**. `view.close_other_documents` waits on `docs.multiple`, so a
        // menu of it alone would never open for the commonest state there is —
        // and `every_menu_offers_something_when_a_document_is_open_and_selected`
        // caught exactly that. A surface an operator right-clicks once, gets
        // nothing from, and never tries again is worse than a surface with no
        // menu at all.
        //
        // **Close tabs to the right** is absent because its operand is a
        // *direction* rather than a document, and this application has no other
        // control shaped that way — so it would arrive with its own command,
        // condition and arm, to save a gesture that closing two tabs already
        // covers. It goes in the day somebody has fifteen drawings open and
        // says so.
        //
        .with(Menu::new(DOCUMENT_TAB).with_items([
            Item::command("file.close"),
            Item::command("view.close_other_documents"),
        ]))
        .with(Menu::new(OBJECTS_ROW).with_items([Item::command("file.properties")]))
        // -------------------------------------------------------------------
        // pages.row — a page tile in the Pages panel.
        //
        // Unlike `objects.row`, this menu carries **destructive** verbs, and
        // that is right rather than inconsistent. The distinction is what the
        // right-click is *about*:
        //
        // * an Objects row names a paint-order index the panel has *focused*,
        //   which is deliberately not the canvas selection — so a Delete there
        //   would be enabled by `selection.any` and would remove objects the
        //   operator never pointed at (see that menu's own note);
        // * a Pages tile names **the page selection**, which the panel owns and
        //   which the `pages.*` commands already act on. The verbs below are
        //   the same ones on the Pages tab, reaching the same selection, so a
        //   right-click here cannot act on anything a click on the tab would
        //   not.
        //
        // `RIBBON_IA.md` §5.8's rule — a context menu carries the same commands
        // again, never new ones — is therefore literally true of this list:
        // every id is registered, gated and drawn on Pages ▸ Organise or
        // Pages ▸ Transform.
        //
        // The order is the order of use: move, then extract, then rotate, then
        // the one that cannot be undone. Delete last, and separated from the
        // rest by everything above it, because a menu that puts a destructive
        // verb under the pointer's resting position gets pressed by accident.
        // -------------------------------------------------------------------
        .with(Menu::new(PAGES_ROW).with_items([
            Item::command("pages.move_up"),
            Item::command("pages.move_down"),
            Item::command("pages.extract"),
            Item::command("pages.rotate_left"),
            Item::command("pages.rotate_right"),
            Item::command("pages.delete"),
        ]))
}

// ===========================================================================
// MenuHost
// ===========================================================================

/// **The one seam between a right-click site and the menu engine.**
#[derive(Clone, Copy)]
pub struct MenuHost<'a> {
    /// The document the menus live in. `Shell` implements
    /// `egui_shell::menu::MenuLookup`, and it is also what supplies the
    /// chord hints: a menu row shows the chord **the keymap binds**, so an
    /// operator who rebinds a key sees the menu follow with nothing else to
    /// keep in step.
    shell: &'a Shell,
    /// Every command this build has.
    registry: &'a CommandRegistry,
    /// The conditions the frame was composed with.
    ///
    /// A *snapshot*, taken before any widget was drawn — which is why
    /// [`Self::with_condition`] exists. See its docs; the staleness it
    /// repairs is not hypothetical.
    conditions: &'a ConditionSet,
}

impl<'a> MenuHost<'a> {
    /// Bind the menu document, the registry and this frame's conditions
    /// together.
    #[must_use]
    pub fn new(
        shell: &'a Shell,
        registry: &'a CommandRegistry,
        conditions: &'a ConditionSet,
    ) -> Self {
        Self {
            shell,
            registry,
            conditions,
        }
    }

    /// **The operator-visible label of `id`, from the one registry the ribbon
    /// reads.**
    #[must_use]
    pub fn label(&self, id: &str) -> Option<&str> {
        self.registry.get(id).map(|c| c.label.as_str())
    }

    /// **The chord bound to `id` by the operator's own keymap**, if any.
    #[must_use]
    pub fn chord(&self, id: &str) -> Option<String> {
        egui_shell::menu::shortcut::Shortcuts::of(self.shell)
            .get(id)
            .map(str::to_owned)
    }

    /// The conditions this host evaluates predicates against.
    #[must_use]
    pub fn conditions(&self) -> &ConditionSet {
        self.conditions
    }

    /// **This frame's conditions, with one condition corrected.**
    #[must_use]
    pub fn with_condition(&self, condition: &str, holds: bool) -> ConditionSet {
        self.with_conditions(&[(condition, holds)])
    }

    /// The same correction, for **several** conditions at once.
    #[must_use]
    pub fn with_conditions(&self, pairs: &[(&str, bool)]) -> ConditionSet {
        let mut set = self.conditions.clone();
        for &(condition, holds) in pairs {
            if holds {
                set.set(condition);
            } else {
                set.clear(condition);
            }
        }
        set
    }

    /// Attach the menu for `context_id` to a widget's secondary click, and
    /// report the commands the operator chose.
    #[must_use]
    pub fn attach(&self, response: &egui::Response, context_id: &str) -> Vec<HandlerToken> {
        self.attach_with(response, context_id, self.conditions)
    }

    /// [`Self::attach`], against conditions the caller has corrected.
    ///
    /// The companion to [`Self::with_condition`]; see its docs for the
    /// frame-ordering hazard both exist for.
    #[must_use]
    pub fn attach_with(
        &self,
        response: &egui::Response,
        context_id: &str,
        conditions: &ConditionSet,
    ) -> Vec<HandlerToken> {
        // The two optional capabilities every pdfcer context menu is
        // built with — the rect sink that makes a row clickable by a driven
        // check, and the icon painter that makes a row draw the glyph its
        // command already names — live in [`super::menus_wiring`], with the
        // full account of why each exists and what its absence cost.
        //
        // They are there rather than here because both are properties of
        // the BUILD, identical on every frame and at every call site, while
        // this type exists to bind one frame's document, registry and
        // conditions. Mixing the two put the answer to "why does a menu row
        // have a glyph?" in the middle of a lifetime-juggling struct.
        super::menus_wiring::attach(self.shell, self.registry, response, context_id, conditions)
    }

    /// **Whether right-clicking this context would produce a menu at all.**
    #[must_use]
    pub fn would_open(&self, context_id: &str) -> bool {
        self.would_open_with(context_id, self.conditions)
    }

    /// [`Self::would_open`], against conditions the caller has corrected.
    #[must_use]
    pub fn would_open_with(&self, context_id: &str, conditions: &ConditionSet) -> bool {
        Menu::would_open(self.shell, self.registry, context_id, conditions)
    }
}

impl std::fmt::Debug for MenuHost<'_> {
    /// Deliberately shallow. A `Shell` and a `CommandRegistry` printed in
    /// full are thousands of lines, and a `MenuHost` appears in a trace to
    /// answer "was one supplied at all", never "what is in it".
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuHost")
            .field("menus", &self.shell.menus.as_ref().map_or(0, Menus::len))
            .field("commands", &self.registry.len())
            .field("conditions", &self.conditions.iter().count())
            .finish()
    }
}

#[cfg(test)]
mod tests;
