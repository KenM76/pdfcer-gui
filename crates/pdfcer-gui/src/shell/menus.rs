//! # shell::menus — pdfcer's context menus, as data
//!
//! [`built_in`] returns every context menu pdfcer defines, as an
//! `egui_shell::menu::Menus` value carried on the same
//! [`egui_shell::Shell`] the ribbon is. [`MenuHost`] is the one place a
//! right-click site turns that data into drawn rows and invoked commands.
//!
//! This module is the third of the three surfaces `RIBBON_IA.md` §5.8
//! names, and it is the one that was missing entirely:
//!
//! | Surface | Answers | Lives | Built |
//! |---|---|---|---|
//! | the contextual **Format** tab | *"what do I change mid-gesture?"* | in the ribbon, on selection | `manifest::format` |
//! | the **properties panel** | *"what *is* this thing?"* | in the dock, always | `panels::properties` |
//! | the **context menu** | *"act on **this**, now"* | at the pointer | **here** |
//!
//! `RIBBON_IA.md` §6 says what its absence cost:
//!
//! > **Context menus** — currently zero in the entire crate
//! > (`grep context_menu` → no hits). Every selection type above needs one,
//! > carrying the same commands as its Format tab section plus
//! > Cut/Copy/Paste/Delete. This is not a ribbon question, but it is **the
//! > other half of making selection meaningful**, and no amount of ribbon
//! > design substitutes for it.
//!
//! and §5.8 says why a third surface is not duplication:
//!
//! > A third surface, the **context menu**, carries the same commands again
//! > for the user who right-clicks. That is not duplication in the P1 sense
//! > — **context menus are not tabs** — and it is the path most users try
//! > after the keyboard.
//!
//! That sentence is why this module holds no vocabulary of its own. A menu
//! item is [`egui_shell::manifest::Item`] — the *same* type a ribbon band
//! holds — so a command id resolves through the *same* registry into the
//! *same* [`egui_shell::HandlerToken`], dispatched at the *same* choke
//! point. "The same commands again" is literally true, and the
//! confirmation gate, the undo entry and the refusal that `format.delete`
//! is subject to are written once and cover both.
//!
//! # ★ The rule that decided what is in each menu: only real commands
//!
//! `RIBBON_IA.md` P3, **no placeholders**, applied here exactly as
//! [`super::manifest`] applies it to the ribbon. §6 asks for
//! *"Cut/Copy/Paste/Delete"* on the selection menu. **Cut, copy and paste
//! do not exist**: there is no object clipboard in this build, which
//! [`super::manifest::PLANNED`] records against `edit.cut`, `edit.copy`,
//! `edit.paste` and `edit.paste_in_place`. So they are **absent from
//! `canvas.object`**, not present and greyed.
//!
//! The distinction matters and the menu engine implements both halves:
//!
//! | Situation | What the operator sees | Why |
//! |---|---|---|
//! | the command **is registered** and its predicate is false | the row, **greyed**, with its tooltip | it exists; it is not applicable *right now* |
//! | the command **is not registered** in this build | nothing at all | it does not exist, and a greyed row for a command that can never be enabled is a promise the build cannot keep |
//!
//! An unregistered id would in fact still be *dropped* by
//! `egui_shell::menu::plan::resolve` and disclosed on the verify channel —
//! but relying on that would be shipping a document that names commands
//! this build does not have and calling the omission a feature.
//! [`tests::every_command_every_menu_names_is_registered`] is what stops
//! that: the check `egui_shell::Shell::validate_against` does **not**
//! perform, because `command_references()` walks tabs, the QAT and the
//! keymap and deliberately not the menus (a command in a menu is not a
//! reachability claim — see [`super::tests`]).
//!
//! # ★ And the rule that decided the shape: a menu with nothing to offer
//! never opens
//!
//! Right-clicking something that has nothing to offer must do **nothing** —
//! not flash an empty box, and not open a menu of greyed rows. The engine
//! takes that decision *before* it asks `egui` for a popup
//! (`egui_shell::menu::plan::offers_anything`), and [`MenuHost::attach`]
//! is the only path this application uses, so every pdfcer right-click
//! inherits it. [`MenuHost::would_open`] answers the same question without
//! drawing, for a caller that wants to know.
//!
//! It is not a theoretical case here. `objects.row` offers `file.properties`,
//! which is gated on `doc.open`; with no document the Objects panel draws no
//! rows at all, but a build that compiled `file.properties` out would leave
//! that menu empty, and the right-click would then correctly do nothing
//! rather than opening a box with a rule in it.
//!
//! # The four menus, and why each holds what it holds
//!
//! | Context id | Right-click site | Items | The reasoning |
//! |---|---|---|---|
//! | [`CANVAS_OBJECT`] | a selected object on the page | `view.zoom_selection`, `format.properties`, `format.select_form`, `format.unshare_form`, `format.delete` | ★ The Items column was **wrong** until 2026-08-28 — it had never been updated for `format.select_form`, added the previous day, which is this project's recurring shape of a prose claim beside the thing it describes decaying while a test pins the truth one screen down. The two form commands arrived with the form-XObject work: `format.select_form` because a click now reaches *inside* a form and the container has to be reachable on purpose, and `format.unshare_form` because O53 forbids a command existing only on the ribbon — and because the operator who needs it is mid-gesture, about to type into a title block, and the pointer is where they are looking. Zoom to selection is here because **SolidWorks and Acrobat both reach it by right-click** and only Inkscape binds a key for it — operator instruction of 2026-08-14 to match those three; see the registration site for why no chord was invented. Then §5.8 lists Delete in **every** selection type's row. It is the one command in that section that exists (see `manifest::DIRECTED`), and it is wired: `PdfcerApp::dispatch_token` reads `SelectionState::deletable_objects_on`, the same rule the Delete key reads. **`format.properties` joined them on 2026-08-18**, with the ce-dimension properties section: a selected ce dimension's group, measurement, style overrides and radius/diameter switch are otherwise reachable only by noticing that a contextual tab appeared or by opening a dock panel by name, and the operator's report was *"I click and can't figure out how to enable some of the basic stuff."* It sits above Delete because the destructive row is last in every menu here. |
//! | [`CANVAS_EMPTY`] | blank page, or the paper beside the drawing | `view.zoom_fit_page`, `view.zoom_fit_width`, `view.zoom_fit_height`, `view.zoom_actual` | The four **named** zoom levels, all of which have a live dispatch arm today. A right-click on paper is about the *view*, because there is no object to be about. |
//! | [`DOCK_TAB`] | a panel tab in the dock | `view.reset_layout` | The only registered command that acts on the dock. The **command** is wired (`PdfcerApp::dispatch_command` calls `Modes::reset` with `ResetScope::All`); the **menu** still cannot be attached — see the warning below. |
//! | [`OBJECTS_ROW`] | a row in the Objects panel | `file.properties` | The Properties panel is *where an object row is described*; right-clicking a row focuses it and this is the command that puts the description on screen — which it now does: `PdfcerApp::show_panel` activates the panel, mounting it first if the operator's arrangement no longer holds it. |
//!
//! ## ★ `dock.tab` is defined and **cannot be attached from this crate**
//!
//! `egui-shell`'s dock draws its own tabs and already owns their secondary
//! click: `crates/egui-shell/src/dock/tabs.rs` calls
//! `response.context_menu(…)` with a hard-coded **Close** button, and
//! `egui_shell::dock::Dock` exposes no seam — no `with_tab_menu`, no tab
//! `Response` handed back — through which an application could attach one
//! of its own. Two context menus on one `Response` would fight over the
//! same popup id.
//!
//! The menu is defined here anyway, and that is a deliberate choice rather
//! than an oversight left lying about:
//!
//! - it is **data**, and a document that describes pdfcer's context menus
//!   with one of the four missing would be wrong about the application even
//!   while it happened to match the wiring;
//! - the operator-customization layer merges against it, so an operator can
//!   already state what they want on a panel tab;
//! - it costs nothing at run time — a menu nobody looks up is a `Vec` entry.
//!
//! What it needs to come alive is a change in `egui-shell`, not here: the
//! dock must either take a context id for its tabs and call
//! `egui_shell::menu::Menu::attach` itself, or hand the tab's `Response`
//! out. `Close` is not a pdfcer command and is not registered
//! (see [`super::manifest::PLANNED`]'s `dock.close_panel` entry), so the
//! hard-coded button is not a duplication of anything in this file.
//!
//! # Where the strings are
//!
//! Nowhere in this module, and nowhere in [`crate::text::menus`] either.
//! Every row's label and tooltip is the **command's**, from
//! [`crate::text::commands`], because a context menu carries the same
//! commands again and a second copy of "Delete" is a second copy that can
//! drift. [`crate::text::menus`]' header carries the full argument and the
//! list of what *would* land there.
//!
//! The context ids below (`"canvas.object"`, `"dock.tab"`, …) are never
//! displayed. They are lookup keys the application chooses and the shell
//! never interprets — the same kind of string a command id is.

use egui_shell::manifest::{Item, Shell};
use egui_shell::menu::{ContextMenu, Menu, Menus};
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
///
/// Two rows: take a copy, and look closer. Its own context rather than a
/// filtered [`CANVAS_OBJECT`], because every other row of that menu edits and
/// R9 says a mode that cannot edit renders nothing rather than a greyed list.
/// `canvas::menus::CanvasMenu::ReadObject` carries the argument.
pub const CANVAS_READ_OBJECT: &str = "canvas.read-object";

/// ★★★ Right-click on the page **with a caret placed in existing text**.
///
/// The third canvas menu, added 2026-08-28 with paragraph reflow. It is keyed
/// on the *caret*, not on a selection, and that is the whole reason it is a
/// separate context: while the operator is editing text there is no selected
/// object, so [`CANVAS_OBJECT`] never resolves and [`CANVAS_EMPTY`] would
/// offer four zoom levels to somebody with a cursor blinking in a paragraph.
///
/// ⇒ **Without this, reflow would be reachable only from the ribbon**, and the
/// standing rule is that anything the engine can do to a thing on the page is
/// reachable by clicking that thing on the page. A paragraph's "click" is the
/// caret; its right-click is this.
pub const CANVAS_TEXT: &str = "canvas.text";

/// ★★★ Right-click on the page **over a form field**.
///
/// The fourth canvas menu, added 2026-08-28. Keyed on `doc.selected_field`,
/// which is neither a `SelectionState` entry nor a caret — a `/Widget` is
/// deliberately not an annotation selection — so none of the other three ever
/// resolved for one, and a right-click on a text box offered *"zoom to fit
/// width"*.
///
/// ⇒ `OPERATOR_REQUESTS.md` **O53**: *"always always always I need objects on
/// the canvas to be clickable and editable as one would expect."* A context
/// menu is the fourth of the five gestures that sentence covers, after click,
/// drag and Delete.
pub const CANVAS_FIELD: &str = "canvas.field";

/// Right-click on a panel tab in the dock.
///
/// Defined but not attachable from this crate — see the module header.
pub const DOCK_TAB: &str = "dock.tab";

/// Right-click on an object row in the Objects panel.
pub const OBJECTS_ROW: &str = "objects.row";

/// Right-click on a page tile in the Pages panel.
///
/// Spelled here **and** in `crate::panels::pages::PAGES_ROW`, which is the one
/// duplication this module tolerates and only because the panel attaches the
/// menu before this file could hand it a constant: the two are asserted equal
/// by that panel's own test, so a rename that touches one fails rather than
/// silently detaching every tile's menu.
pub const PAGES_ROW: &str = "pages.row";

/// **A document tab in the strip under the ribbon** — not a dock tab.
///
/// The two are deliberately different contexts because they name different
/// things: [`DOCK_TAB`] names a *panel*, which is a tool, and this names a
/// *document*, which is an operand. Sharing one menu between them would offer
/// *Reset layout* on a drawing and *Close others* on the Bookmarks panel.
pub const DOCUMENT_TAB: &str = "document.tab";

/// Every context id this module defines, for the sweeps in [`tests`].
///
/// Hand-written, and pinned by
/// [`tests::the_catalog_defines_exactly_the_documented_contexts`] against
/// [`built_in`] itself, so a menu added to the document without an entry
/// here — or an entry here with no menu — fails rather than silently
/// halving a test sweep.
pub const CONTEXTS: &[&str] = &[
    CANVAS_OBJECT,
    CANVAS_READ_OBJECT,
    CANVAS_EMPTY,
    CANVAS_TEXT,
    CANVAS_FIELD,
    DOCK_TAB,
    DOCUMENT_TAB,
    OBJECTS_ROW,
    PAGES_ROW,
];

// ===========================================================================
// The document
// ===========================================================================

/// **Every context menu pdfcer defines.**
///
/// Deterministic and side-effect free, exactly as
/// [`super::manifest::built_in`] is, and called from the same place: once,
/// at start-up, and from the tests. It is the **built-in layer** of
/// `SHELL_FRAMEWORK.md` §4's three-layer merge, so it has to be complete
/// and has to validate — it is what every other layer patches and what an
/// operator gets back when they reset.
///
/// # Order is presentation
///
/// Items appear in the order they are written. Within a menu that order is
/// argued at each site; between menus it does not matter, because a lookup
/// is by key.
#[must_use]
pub fn built_in() -> Menus {
    Menus::new()
        // -------------------------------------------------------------------
        // canvas.object — the selection menu.
        //
        // ★ ONE item, and the three that `RIBBON_IA.md` §6 also asks for are
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
        // ★ **Zoom to selection is here because that is where two of the three
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
            // ★ The right-click route to the Properties panel, added 2026-08-18
            // with the ce-dimension properties section.
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
            // ★ The right-click route to the container, added 2026-08-27 with
            // the form-XObject descent.
            //
            // It matters more here than on the ribbon, and the reason is where
            // the operator's hand is: they have just clicked something inside a
            // form, found they cannot move it, and the next thing they do is
            // right-click it. A control on a contextual tab three inches away
            // is the correct *second* home, not the first one they will find.
            //
            // Greyed rather than absent when the selection is not inside a
            // form, by the same R9 reading the catalog entry argues.
            Item::command("format.select_form"),
            // ★★★ The right-click route to *"give this page its own copy"*,
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
            // ★★ It is also the only surface that can reach them in time. The
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
            // ★★★ **Absent, not greyed, where the engine would refuse it.** The
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
            Item::command("format.delete").shown_when(super::manifest::DELETE_PERMITTED),
        ]))
        // -------------------------------------------------------------------
        // canvas.read-object — a picture, while reading.
        //
        // ★★★ `OPERATOR_REQUESTS.md` O71: *"In read mode the regular pointer
        // should also allow us to select images so we can copy and paste them
        // … outside of the pdfcergui."*
        //
        // Copy became reachable there by chord on 2026-08-31 and by nothing
        // else, which is a feature only somebody who was told about it can
        // use. Acrobat Reader puts *Copy Image* on the right-click and that is
        // where a hand goes.
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
        // ★★ TWO items, and the pair is chosen by what an operator does to a
        // field they have just placed: they check its settings, or they got rid
        // of it. Properties first, destructive last — the ordering rule every
        // menu in this file follows.
        //
        // ★★★ **Rename is absent and its absence is not an oversight.** It
        // lives in the Properties panel as a draft box with an explicit commit,
        // because renaming a field on every keystroke would author one real,
        // separately-undoable rename per character. A menu item cannot ask for
        // text, so `format.properties` IS the rename route — one click further
        // and honest about it, rather than a second half-implemented rename
        // that could disagree with the first.
        //
        // ★ `format.delete` removes THIS BOX, not the whole field. A field with
        // two widgets on two pages is one field selectable from either place,
        // and the panel offers both deletions labelled. See `dispatch::format`.
        //
        // ★★★ **`shown_when` — and its absence here was the second half of the
        // R83 forms defect, left open for a day by the fix that closed the
        // first.**
        //
        // `canvas.object`'s `format.delete` two menus above has carried
        // `DELETE_PERMITTED` since 2026-08-29; this one carried nothing. On an
        // ordinary certified fillable form that made the menu's Delete drawn,
        // live and undimmed over a widget the engine will not remove — and the
        // press cleared `doc.selected_field`, which took away the Properties
        // panel's sentence that had been correctly explaining the refusal. A
        // refused gesture that destroys its own explanation.
        //
        // It is the SAME condition as `canvas.object`'s, deliberately, and it
        // is correct for both because `app::conditions` publishes it from a
        // ladder that asks the forms query when a field is selected and the
        // annotation query otherwise — the same precedence
        // `app::dispatch::format` resolves the command by. One name, one
        // meaning: *deleting what is selected would not be refused*.
        .with(Menu::new(CANVAS_FIELD).with_items([
            Item::command("format.properties"),
            Item::command("format.delete").shown_when(super::manifest::DELETE_PERMITTED),
        ]))
        // -------------------------------------------------------------------
        // canvas.text — the caret's menu.
        //
        // ★★ ONE item, and it is the one that has no other canvas route. Cut,
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
        .with(Menu::new(CANVAS_TEXT).with_items([Item::command("edit.reflow_block")]))
        // -------------------------------------------------------------------
        // dock.tab — a panel tab.
        //
        // ★ Defined, valid, merged and NOT ATTACHED. The dock owns its tabs'
        // secondary click inside `egui-shell` and offers no seam; the module
        // header carries the full account and what would close it.
        //
        // `Close` is deliberately absent. The dock's own hard-coded button
        // closes a tab through `dock::ctx::Intent::Close`, which is a dock
        // mechanism and not a pdfcer command: there is no `dock.close_panel`
        // in the registry, it is listed in `manifest::PLANNED`, and inventing
        // one here would name an id that resolves to nothing.
        //
        // `view.reset_layout` is what is left, and it is not a consolation
        // prize: "put the panels back where they started" is the single most
        // likely thing an operator wants from a right-click on a panel tab
        // after closing it, it is registered, and it is reachable from
        // View ▸ Window as well — which a menu is allowed to mirror, because
        // context menus are not tabs.
        // -------------------------------------------------------------------
        .with(Menu::new(DOCK_TAB).with_items([Item::command("view.reset_layout")]))
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
        // ★ `format.delete` is deliberately NOT here, and the reason is
        // destructive rather than tidy. The Objects panel's focus is **not**
        // the selection — `panels::ObjectTreeUi::focus`'s own docs and
        // `the_panel_focus_has_not_quietly_become_a_selection` defend the
        // distinction — so a Delete on this menu would be enabled by
        // `selection.any`, which describes the CANVAS selection, and would
        // remove objects the operator never pointed at. That is the exact
        // failure that test exists to prevent, arriving through a menu.
        //
        // A Delete that acts on the row belongs here the day the row click
        // becomes a selection gesture and that focus field is deleted, which
        // is the commit `ObjectTreeUi::focus` names.
        // -------------------------------------------------------------------
        // -------------------------------------------------------------------
        // document.tab — the strip under the ribbon.
        //
        // ★ TWO rows, where the conventional menu has three.
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
        // ★ It is also what stops this menu being **empty with one document
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
        // **Move to a new window**, which the operator asked for on 2026-08-20,
        // is absent under R9: a row for a capability the build does not have is
        // a placeholder. It is registered nowhere, so `Menu::attach` cannot
        // draw it even if this list named it.
        .with(Menu::new(DOCUMENT_TAB).with_items([
            Item::command("file.close"),
            Item::command("view.close_other_documents"),
        ]))
        .with(Menu::new(OBJECTS_ROW).with_items([Item::command("file.properties")]))
        // -------------------------------------------------------------------
        // pages.row — a page tile in the Pages panel.
        //
        // ★ Unlike `objects.row`, this menu carries **destructive** verbs, and
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
///
/// Carries the three things every `egui_shell::menu::Menu::attach` call
/// needs — the document to look the context up in, the registry to resolve
/// its ids against, and the conditions to evaluate their predicates
/// against — so a call site names only *which* menu and *what* it was
/// attached to.
///
/// # Why a borrowing struct rather than three arguments
///
/// Because it is passed through two layers that have nothing to do with
/// menus. `canvas::show` and `panels::Panel::show` hand it on to the
/// functions that actually right-click, and threading three parameters
/// through each of them would make every one of those signatures a place
/// the three could be mismatched — a registry from one frame with the
/// conditions from another, say, which produces a menu that is *plausible*
/// and wrong.
///
/// # Why it is `Option` at every call site
///
/// [`crate::app::PdfcerApp::shell`] is `Option<Shell>`: if the built-in
/// manifest ever fails to validate, the ribbon does not render and the
/// application deliberately stays usable for reading. A build in that state
/// has no menus either, and `None` is the honest way to say so — not a
/// stand-in for "menus are not wired yet".
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
    ///
    /// # ★ Why a panel is given this rather than a string of its own
    ///
    /// `crate::panels::tool` names the armed tool, and the only honest name for
    /// it is **the name on the control that armed it**. A second string would
    /// compile, would read identically on the day it was written, and would
    /// drift the first time either was reworded — and the drift is invisible,
    /// because nothing renders both at once.
    ///
    /// `NO_SURFACE.md` §1 records exactly that failure with a colour rather
    /// than a label: a duplicate of a value that already existed, plus a test
    /// that *"asserted the literal triple against a function returning the
    /// literal triple. Two copies of one constant cannot disagree."* Reading
    /// the registry makes the second copy unrepresentable instead of merely
    /// unlikely.
    ///
    /// Returns `None` for an id this build does not register, which is a real
    /// state rather than a defensive one: `SHELL_FRAMEWORK.md` §5b's whole
    /// point is that a capability compiled out loses its command, and a caller
    /// must render **nothing** for it rather than a name with no control behind
    /// it.
    #[must_use]
    pub fn label(&self, id: &str) -> Option<&str> {
        self.registry.get(id).map(|c| c.label.as_str())
    }

    /// **The chord bound to `id` by the operator's own keymap**, if any.
    ///
    /// From the manifest's keymap, inverted, exactly as a menu row's accelerator
    /// hint is — `egui_shell::menu::shortcut::Shortcuts::of`. So an operator who
    /// rebinds a key sees every surface follow, with nothing to keep in step.
    ///
    /// A panel that hard-coded `"Ctrl+E"` would be telling an operator to press
    /// a key their manifest may not bind, which is worse than telling them
    /// nothing: a chord that does not work reads as the *feature* not working.
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

    /// **★ This frame's conditions, with one condition corrected.**
    ///
    /// # The frame-ordering hazard this exists for, in full
    ///
    /// `PdfcerApp::conditions()` is evaluated **once**, at the top of the
    /// frame, before the ribbon is drawn — so its `selection.any` describes
    /// the selection as it stood *at the start of the frame*. The canvas is
    /// composed last, and a right-click over an unselected object **selects
    /// it** (see [`crate::canvas::menus`]). The click and the menu it opens
    /// therefore happen on a frame whose snapshot still says nothing is
    /// selected.
    ///
    /// Left uncorrected the consequence is total, not cosmetic:
    /// `format.delete` is gated on `selection.any`, so it resolves disabled,
    /// so `offers_anything` is false, so **the menu does not open at all** —
    /// and it never opens later either, because `egui`'s popup is opened by
    /// the secondary click and there is no second click. The first
    /// right-click on an object would silently do nothing, which is
    /// precisely the class of defect (`DEFECTS.md` D1) this whole stage
    /// exists to end.
    ///
    /// # Why this is not a second source of truth
    ///
    /// It corrects **one named condition to a value the caller has just
    /// computed**; it does not re-derive the condition set. The rule for
    /// *when* `selection.any` holds still lives in exactly one place —
    /// `PdfcerApp::conditions`, reading `OpenDoc::selection` — and the caller
    /// here passes `!selection.is_empty()` read from the same field, one
    /// frame later. The spelling of the condition comes from
    /// [`super::manifest::SELECTION_ANY`], which is the same constant the
    /// Format tab's `visible_when` and `format.delete`'s `enabled_when` are
    /// built from.
    ///
    /// Returns an owned set, because the borrow it corrects is shared and
    /// the correction lasts exactly as long as the one `attach` call that
    /// wants it.
    #[must_use]
    pub fn with_condition(&self, condition: &str, holds: bool) -> ConditionSet {
        self.with_conditions(&[(condition, holds)])
    }

    /// The same correction, for **several** conditions at once.
    ///
    /// ★★ Added 2026-08-28 with the `canvas.field` menu, and it exists so the
    /// second caller could not be written as a two-step. [`Self::with_condition`]
    /// returns an owned `ConditionSet` rather than a builder, so correcting two
    /// conditions meant taking the result and mutating it — which puts half the
    /// correction inside this type's documented contract and half outside it,
    /// where the next reader will not find the argument above for why the
    /// correction is needed at all.
    ///
    /// ⇒ Both conditions the canvas corrects are *the same fact one frame
    /// later*, and they should arrive by the same route. `with_condition`
    /// delegates here so there is one implementation.
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
    ///
    /// **Nothing is executed.** The returned tokens are *intent*; the
    /// application dispatches them at the one choke point the ribbon
    /// already uses, which is where the confirmation gate and the undo entry
    /// belong.
    ///
    /// A context with no menu, a menu whose every command is missing from
    /// this build, and a menu whose every command is disabled all produce
    /// the same thing: **no popup, and an empty `Vec`**. The engine takes
    /// that decision before `egui` is asked for a popup, and it also closes
    /// an already-open popup whose offer has evaporated — so a menu left
    /// open over a selection that is then deleted vanishes rather than
    /// lingering with a dead Delete in it.
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
        // ★★★ **The rows publish where they were drawn**, since 2026-08-28.
        //
        // This was `Menu::attach(…)` — the convenience constructor that takes
        // *no optional capabilities at all* — so pdfcer's context menus drew
        // rows and told the diagnostic channel nothing about them. The
        // consequence was narrow and total: **no driven check could click a
        // context-menu row**, ever, because there was no coordinate to aim at.
        //
        // `right_clicking_a_form_field_opens_its_menu` is the evidence. It is
        // the first driven context menu in this project's history, it asserts
        // that the right menu *resolved* and that it *offered something*, and
        // it stops there — because the next step, pressing a row, had nothing
        // to press. Its own header records the shape: *"a gesture with no
        // driver is a gesture R1 cannot reach, and the gap left no failing test
        // behind to advertise itself."* This is the same finding one layer
        // down: the driver existed and the target did not.
        //
        // ★★ Why an `egui` popup makes this the ONLY possible answer, rather
        // than the tidiest one. `egui_shell::menu::report`'s header states it:
        // a context menu is drawn at the pointer, and `egui` may flip it to any
        // of several alignments to keep it on screen. There is no fraction of
        // the window it can be hard-coded to and no layout a harness could
        // re-derive. Publishing the rectangle is not the best of three options;
        // it is the only one.
        //
        // ★ The names are `egui_shell::menu::report`'s — `menu.body.<context>`
        // and `menu.item.<context>.<command id>` — and they go through
        // `crate::diag::ui_rect`, the same sink the ribbon, the status bar and
        // the dock already publish to. So a harness filters one channel and one
        // prefix, and nothing here invents a naming scheme.
        //
        // ★ Cost when nobody is listening: `Reporter` does not format a name
        // unless a sink is present, and `crate::diag::ui_rect` is a no-op
        // without `PDFCER_DIAG`. A closure per attach, and nothing else.
        let mut sink = |name: &str, rect: egui::Rect| crate::diag::ui_rect(name, rect);
        ContextMenu::new().reporting_rects_to(&mut sink).attach(
            response,
            self.shell,
            self.registry,
            context_id,
            conditions,
        )
    }

    /// **Whether right-clicking this context would produce a menu at all.**
    ///
    /// Pure and cheap, and the *same* question [`Self::attach`] asks itself
    /// — so a caller that wants to draw a "⋯" affordance beside a row, or a
    /// test that wants to assert the empty-menu rule without opening a
    /// window, gets the answer the operator will actually get.
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
mod tests {
    use super::*;
    use crate::shell::{commands, manifest};
    use egui_shell::manifest::Item;
    use std::collections::BTreeSet;

    /// The shipped shell and a fully populated registry, built the way the
    /// application builds them.
    fn shell_and_registry() -> (Shell, CommandRegistry) {
        let mut registry = CommandRegistry::new();
        commands::register(&mut registry);
        (manifest::built_in(), registry)
    }

    /// Conditions for a document that is open, has pages, and has something
    /// selected — the state in which every menu here is at its liveliest.
    fn everything_open() -> ConditionSet {
        ConditionSet::new()
            .with("doc.open")
            .with("doc.pages")
            .with(manifest::SELECTION_ANY)
            // ★★ 2026-08-28. Without it `canvas.object` stopped opening, and
            // the failure was correct: `format.delete` and `format.properties`
            // moved to the wider `selection.actionable` when a form field
            // became something they can act on, and this fixture's name
            // promises *"everything open"* while naming conditions one at a
            // time.
            //
            // ⇒ A hand-listed "liveliest state" fixture goes stale the moment a
            // command's predicate changes, and it fails on the menu that lost
            // its last enabled item rather than on the condition that moved —
            // which is a true failure pointing at the wrong file. Adding the
            // name here is the whole repair; the alternative, deriving the set
            // from the registry, would make the test assert that the registry
            // agrees with itself.
            .with(manifest::SELECTION_ACTIONABLE)
    }

    /// **★ Every command every menu names is registered.**
    ///
    /// The check nothing else performs. `Shell::validate_against` walks
    /// `command_references()`, which covers tab groups, the QAT and the
    /// keymap and deliberately **not** the menus — so a menu naming a
    /// command this build does not have passes the manifest's own
    /// validation, renders as one row fewer, and discloses the omission on
    /// a channel nobody reads during development.
    ///
    /// `Menus::validate_against` is the engine's own opt-in answer and it
    /// checks both this and the structural rules (no empty context id, no
    /// duplicate context, no command listed twice within one menu), so it
    /// is asked rather than reimplemented. Its error names the menu **and**
    /// the id, which is what makes a failure point at a line rather than at
    /// a file.
    #[test]
    fn every_command_every_menu_names_is_registered() {
        let (shell, registry) = shell_and_registry();
        let menus = shell
            .menus
            .as_ref()
            .expect("the built-in shell must carry its menus");
        menus.validate_against(&registry).expect(
            "every command a context menu names must be registered — an unregistered id \
             is silently dropped at render time, so nothing else would report this",
        );
    }

    /// …and the manifest really carries them, rather than the menus existing
    /// only as a function nothing calls.
    ///
    /// The failure this catches is a one-line omission with no symptom: drop
    /// the `menus` assignment from `manifest::built_in` and every test in
    /// this file that builds `built_in()` directly still passes, while every
    /// right-click in the running application does nothing.
    #[test]
    fn the_shipped_shell_carries_the_menu_document() {
        let shell = manifest::built_in();
        let menus = shell
            .menus
            .as_ref()
            .expect("`manifest::built_in` must set the `menus` field");
        assert_eq!(
            menus.len(),
            built_in().len(),
            "the shell carries a different menu document from the one this module defines"
        );
        for context in CONTEXTS {
            assert!(
                menus.get(context).is_some(),
                "the shipped shell has no menu for `{context}`"
            );
        }
    }

    /// **The catalog and the constant list are the same set.**
    ///
    /// [`CONTEXTS`] is hand-written and every sweep below is only as
    /// complete as it is, which is the classic way a test suite quietly
    /// stops covering something. Checked in both directions.
    #[test]
    fn the_catalog_defines_exactly_the_documented_contexts() {
        let menus = built_in();
        let declared: BTreeSet<&str> = CONTEXTS.iter().copied().collect();
        assert_eq!(
            declared.len(),
            CONTEXTS.len(),
            "CONTEXTS lists a context id twice"
        );
        let defined: BTreeSet<&str> = menus.iter().map(|m| m.context.as_str()).collect();
        assert_eq!(
            defined, declared,
            "the menu document and CONTEXTS disagree; every sweep in this file is scoped \
             by CONTEXTS, so the extra or missing entry is untested"
        );
    }

    /// The document is structurally valid on its own.
    ///
    /// Distinct from the registry check and not implied by it: the built-in
    /// layer is what every customization layer patches and what a reset
    /// restores, so it has to stand up without an application present —
    /// non-empty context ids, no duplicates, no command listed twice in one
    /// menu.
    #[test]
    fn the_built_in_menu_document_is_valid() {
        built_in()
            .validate()
            .expect("the built-in menu layer must satisfy every structural rule");
    }

    /// **★ No menu names a command that does not exist — stated as the
    /// no-placeholders rule, by name.**
    ///
    /// `every_command_every_menu_names_is_registered` proves the positive.
    /// This proves the *specific* negative `RIBBON_IA.md` §6 asks for and
    /// P3 forbids: §6 wants Cut/Copy/Paste on the selection menu, this build
    /// has no object clipboard, and the honest answer is **absence**.
    ///
    /// Asserted against `PLANNED` rather than against a hand-written list of
    /// four ids, so a clipboard command that lands — and is therefore
    /// removed from `PLANNED` — stops being forbidden here automatically
    /// instead of failing a test that had gone stale.
    #[test]
    fn no_menu_offers_a_command_this_build_does_not_have() {
        let planned: BTreeSet<&str> = manifest::PLANNED.iter().map(|(id, _)| *id).collect();
        for menu in built_in().iter() {
            for id in menu.command_ids() {
                assert!(
                    !planned.contains(id),
                    "menu `{}` offers `{id}`, which `manifest::PLANNED` records as absent \
                     from this build. P3: an unavailable capability renders NOTHING — not \
                     a greyed row, which is a promise the build cannot keep.",
                    menu.context
                );
            }
        }
        // ★★★ **A HAND-WRITTEN LIST OF FOUR IDS STOOD HERE UNTIL 2026-09-01,
        // AND THIS TEST'S OWN DOC COMMENT SAID IT DID NOT.**
        //
        // The paragraph above reads *"asserted against `PLANNED` rather than
        // against a hand-written list of four ids, so a clipboard command that
        // lands … stops being forbidden here automatically instead of failing
        // a test that had gone stale."* The sweep above does exactly that. And
        // underneath it sat the list anyway, forbidding `edit.cut`,
        // `edit.copy`, `edit.paste` and `edit.paste_in_place` by name.
        //
        // The object clipboard landed on 2026-08-20. `edit.copy` has a
        // registration, a dispatch arm, a driven check and — as of
        // `OPERATOR_REQUESTS.md` O71 — a right-click row on the reader's
        // picture menu, which is what made this fail. **The test was not
        // protecting an invariant; it was pinning a fact that had stopped
        // being true, in a file whose prose already said it should not.**
        //
        // ⇒ Deleted rather than updated, because updating it would restore the
        // exact mechanism the doc comment argues against. `PLANNED` is the one
        // list, and a command that lands leaves it.
    }

    /// **★ Every menu opens when the application is at its liveliest.**
    ///
    /// The other half of the empty-menu rule, and the half that would
    /// otherwise be satisfied by defining no menus at all. A menu that never
    /// opens is indistinguishable from a right-click that is not wired, and
    /// the operator draws the same conclusion from both.
    ///
    /// `dock.tab` is included: it is not *attached* (see the module header),
    /// but the day the `egui-shell` seam lands it must have something to
    /// offer, and this is what says so.
    #[test]
    fn every_menu_offers_something_when_a_document_is_open_and_selected() {
        let (shell, registry) = shell_and_registry();
        let conditions = everything_open();
        let host = MenuHost::new(&shell, &registry, &conditions);
        for context in CONTEXTS {
            assert!(
                host.would_open(context),
                "`{context}` offers nothing even with a document open, pages present and \
                 something selected — so right-clicking that surface does nothing, ever"
            );
        }
    }

    /// ★★★ **The field menu opens on a field selection ALONE.**
    ///
    /// The state the operator is actually in when they right-click a text box:
    /// `doc.selected_field` is set and `SelectionState` is **empty**, because a
    /// `/Widget` is deliberately not an annotation selection. Every other canvas
    /// menu resolves nothing there.
    ///
    /// ⇒ This is the assertion that would have caught the bug this feature
    /// shipped with for ten minutes: `format.delete` and `format.properties`
    /// were gated on `selection.any`, which is **false** in exactly this state,
    /// so both items resolved disabled, `offers_anything` was false, and the
    /// menu never opened. A right-click on a form field would have done nothing
    /// at all — `DEFECTS.md` D1's shape, arrived at through a new door.
    ///
    /// ★ `everything_open()` is deliberately not used: it sets both conditions
    /// and would pass on a build where the two are confused. The whole point is
    /// that only the wider one holds here.
    #[test]
    fn the_field_menu_opens_with_a_field_selected_and_nothing_else() {
        let (shell, registry) = shell_and_registry();
        let field_only = ConditionSet::new()
            .with("doc.open")
            .with("doc.pages")
            .with(manifest::SELECTION_ACTIONABLE);
        let host = MenuHost::new(&shell, &registry, &field_only);
        assert!(
            host.would_open(CANVAS_FIELD),
            "a selected form field offers no menu, so right-clicking one does nothing"
        );
        // ★★ And the object menu opens here TOO, which is correct and is worth
        // asserting rather than leaving as a surprise: both its items can act
        // on a field, so the menus differ by their CONTEXT ID rather than by
        // what is enabled. `canvas::menus::attach` picks Field first when a
        // field is in play, which is where the distinction is made.
        assert!(host.would_open(CANVAS_OBJECT));
    }

    /// **★ …and an empty menu never opens.**
    ///
    /// The engine's rule 2, asserted through the seam this application
    /// actually uses rather than against the engine's own unit tests.
    /// Three shapes, and all three are reachable:
    ///
    /// 1. **a context with no menu at all** — a right-click site whose id is
    ///    misspelled, or one wired ahead of its menu;
    /// 2. **a menu whose every command is disabled** — `canvas.object` with
    ///    nothing selected, which is what a right-click on paper would find
    ///    if the canvas picked the wrong context id;
    /// 3. **a menu whose every command is unregistered** — the shape a
    ///    build with a capability compiled out produces.
    ///
    /// Shape 2 is the one that matters most in daily use, and it is the one
    /// a naive wiring gets wrong: `format.delete` is registered, so a
    /// `context_menu` closure written by hand would happily draw it greyed
    /// and cost a click to dismiss.
    #[test]
    fn a_menu_with_nothing_to_offer_does_not_open() {
        let (shell, registry) = shell_and_registry();

        // 1. No such context.
        let live = everything_open();
        let host = MenuHost::new(&shell, &registry, &live);
        assert!(
            !host.would_open("canvas.nothing-here"),
            "an unknown context must resolve to no menu, not to an empty one"
        );

        // 2. Every command disabled — nothing is selected, so `format.delete`
        //    is greyed and it is the menu's only item.
        let nothing_selected = ConditionSet::new().with("doc.open").with("doc.pages");
        let host = MenuHost::new(&shell, &registry, &nothing_selected);
        assert!(
            !host.would_open(CANVAS_OBJECT),
            "a menu of nothing but greyed rows is strictly worse than no menu: it costs a \
             click to dismiss and teaches the operator that right-clicking here is useless"
        );
        assert!(
            host.would_open(CANVAS_EMPTY),
            "…while the view menu is still live, which is what makes the canvas's choice \
             of context id the thing that matters"
        );

        // 3. Every command unregistered — the compiled-out build.
        let empty_registry = CommandRegistry::new();
        let host = MenuHost::new(&shell, &empty_registry, &live);
        for context in CONTEXTS {
            assert!(
                !host.would_open(context),
                "`{context}` opened against a registry holding no commands at all"
            );
        }
    }

    /// **★ A corrected condition changes the answer.**
    ///
    /// [`MenuHost::with_condition`] exists for one frame-ordering hazard,
    /// and this is that hazard reduced to two assertions: with the stale
    /// snapshot the selection menu does not open, and with the correction
    /// the canvas just computed it does.
    ///
    /// Without this the first right-click on an object silently does
    /// nothing — the menu is decided before `egui` is asked for a popup, so
    /// there is no later frame on which it can recover.
    #[test]
    fn correcting_the_selection_condition_is_what_opens_the_object_menu() {
        let (shell, registry) = shell_and_registry();
        // The snapshot the frame was composed with: nothing was selected
        // when the ribbon was drawn.
        let stale = ConditionSet::new().with("doc.open").with("doc.pages");
        let host = MenuHost::new(&shell, &registry, &stale);
        assert!(!host.would_open(CANVAS_OBJECT));

        // The canvas has since selected the object under the pointer.
        //
        // ★ BOTH conditions, because `attach` corrects both — see
        // `MenuHost::with_conditions`. Correcting only `selection.any` here
        // would have this test passing on a build where `attach` forgot the
        // second, which is the exact hazard the test exists for one level up.
        let corrected = host.with_conditions(&[
            (manifest::SELECTION_ANY, true),
            (manifest::SELECTION_ACTIONABLE, true),
        ]);
        assert!(
            host.would_open_with(CANVAS_OBJECT, &corrected),
            "the right-click selected an object and the menu still refused to open"
        );

        // …and the correction goes both ways, so a menu cannot be opened by
        // a condition the caller has just found to be false.
        //
        // ★★ BOTH have to be cleared, and the reason is worth a sentence
        // because the first version of this line cleared only `selection.any`
        // and the assertion failed. `canvas.object`'s two items now take
        // `selection.actionable`, so clearing the narrower condition alone
        // leaves them enabled and the menu opens — correctly.
        //
        // ⇒ A "goes both ways" assertion has to clear **every** condition the
        // forward direction set, or it is asserting about a state the forward
        // direction never produces.
        let cleared = MenuHost::new(&shell, &registry, &corrected).with_conditions(&[
            (manifest::SELECTION_ANY, false),
            (manifest::SELECTION_ACTIONABLE, false),
        ]);
        assert!(!host.would_open_with(CANVAS_OBJECT, &cleared));
    }

    /// A command may appear in several menus, and on a tab as well.
    ///
    /// `RIBBON_IA.md` §5.8: the context menu *"carries the same commands
    /// again … that is not duplication in the P1 sense — context menus are
    /// not tabs"*. Every id in this document is also on a ribbon tab, which
    /// is the point and not an oversight; if a future edit extends the
    /// one-command-one-tab rule over menus, this is the test that says no.
    #[test]
    fn every_menu_command_is_also_reachable_from_the_ribbon() {
        let shell = manifest::built_in();
        let on_a_surface: BTreeSet<String> = shell
            .command_references()
            .into_iter()
            .map(|(_, id)| id)
            .collect();
        for menu in built_in().iter() {
            for id in menu.command_ids() {
                assert!(
                    on_a_surface.contains(id),
                    "menu `{}` is the ONLY route to `{id}`. A context menu is a third \
                     surface carrying commands that already have a home, not a home of \
                     its own — a command reachable by right-click alone is undiscoverable.",
                    menu.context
                );
            }
        }
    }

    /// Menus survive a round trip through RON, which is what makes them
    /// customizable.
    ///
    /// The whole value proposition of the shell-as-data design is that an
    /// operator can edit this; `crate::shell::ron` asserts the same thing
    /// for the manifest as a whole. Asserted here as well, on the menu
    /// document alone, because a failure in the shared file says only that
    /// *something* stopped round-tripping.
    #[test]
    fn the_menu_document_round_trips_through_ron() {
        let original = built_in();
        let text = original.to_ron_pretty().expect("serializes");
        assert_eq!(
            Menus::from_ron(&text).expect("the pretty form parses"),
            original
        );
        // And the shapes an operator would search for are legible in it.
        //
        // ★ The command spelling is checked on the COMPACT form. RON's pretty
        // printer breaks a struct variant across three lines, and `Item::Command`
        // became one when `ItemSize` landed — so a `contains` for the one-line
        // spelling fails on a pretty document that is perfectly correct. The
        // context id is still checked on the pretty form, because that is the
        // string an operator scrolling the file actually looks for.
        assert!(text.contains(CANVAS_OBJECT), "{text}");
        let compact = original.to_ron().expect("serializes");
        // ★★ The spelling checked here carries the CONDITION, and it had to
        // change on 2026-08-29: **both** `format.delete` items now do.
        //
        // `canvas.object`'s gained `selection.delete_permitted` with the
        // annotation half of R83; `canvas.field`'s gained the same name with
        // the form half, which is what this assertion's previous bare spelling
        // was silently attesting was still missing. A `contains` for
        // `Command(id:"format.delete")` matched only because no gate was
        // written on that menu at all.
        //
        // ⇒ Asserting the gated spelling rather than deleting the assertion:
        // the point of the check is that an operator scrolling the compact
        // document can find the command, and the visible-condition is the half
        // that decides whether the row is drawn — which is exactly what such an
        // operator is looking for it to say.
        assert!(
            compact.contains(
                "Command(id:\"format.delete\",visible_when:\"selection.delete_permitted\")"
            ),
            "{compact}"
        );
        assert!(
            !compact.contains("Command(id:\"format.delete\")"),
            "an UNGATED `format.delete` is back on some menu. Both of them are \
             gated on `selection.delete_permitted`, because a Delete drawn where \
             the engine refuses it is silently inert — and on `canvas.field` that \
             press also cleared the selection, blanking the Properties panel \
             sentence that explained the refusal: {compact}"
        );
    }

    /// Each menu holds the items this module's header claims it holds.
    ///
    /// A change-detector, and deliberately one: the table in the header is
    /// the specification, and a menu that quietly gains an item has a
    /// specification that quietly became wrong. The failure message names
    /// the menu, so the fix is one line in one of the two places.
    #[test]
    fn each_menu_holds_exactly_the_documented_items() {
        let menus = built_in();
        for (context, expected) in [
            (
                CANVAS_OBJECT,
                &[
                    "view.zoom_selection",
                    "format.properties",
                    "format.select_form",
                    "format.unshare_form",
                    "format.delete",
                ][..],
            ),
            (
                CANVAS_EMPTY,
                &[
                    "view.zoom_fit_page",
                    "view.zoom_fit_width",
                    "view.zoom_fit_height",
                    "view.zoom_actual",
                ][..],
            ),
            (DOCK_TAB, &["view.reset_layout"][..]),
            (OBJECTS_ROW, &["file.properties"][..]),
        ] {
            let menu = menus.get(context).expect("defined");
            let ids: Vec<&str> = menu.command_ids().collect();
            assert_eq!(
                ids, expected,
                "menu `{context}` no longer matches the table in this module's header"
            );
            // Nothing in this document is a custom item or a separator yet,
            // and a document of pure commands is what makes the sweeps above
            // total rather than approximate.
            assert!(
                menu.items()
                    .iter()
                    .all(|i| matches!(i, Item::Command { .. })),
                "menu `{context}` holds a non-command item; the sweeps in this file walk \
                 `command_ids()` and would not see it"
            );
        }
    }
}
