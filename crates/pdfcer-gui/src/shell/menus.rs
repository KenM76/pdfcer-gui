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
//! # The menus, and why each holds what it holds
//!
//! ⚠ This heading read *"The four menus"* until 2026-09-06 and the table under
//! it listed four while [`built_in`] returned eight. That is this project's
//! recurring shape — **a prose count beside the thing it counts, decaying while
//! a test pins the truth one screen down** — and it is why the heading no
//! longer carries a number at all. [`CONTEXTS`] is the count, and
//! `tests::the_catalog_defines_exactly_the_documented_contexts` is what makes
//! it true.
//!
//! | Context id | Right-click site | Items | The reasoning |
//! |---|---|---|---|
//! | [`CANVAS_OBJECT`] | a selected object on the page | `view.zoom_selection`, `format.properties`, `format.select_form`, `format.unshare_form`, `format.delete` | ★ The Items column was **wrong** until 2026-08-28 — it had never been updated for `format.select_form`, added the previous day, which is this project's recurring shape of a prose claim beside the thing it describes decaying while a test pins the truth one screen down. The two form commands arrived with the form-XObject work: `format.select_form` because a click now reaches *inside* a form and the container has to be reachable on purpose, and `format.unshare_form` because O53 forbids a command existing only on the ribbon — and because the operator who needs it is mid-gesture, about to type into a title block, and the pointer is where they are looking. Zoom to selection is here because **SolidWorks and Acrobat both reach it by right-click** and only Inkscape binds a key for it — operator instruction of 2026-08-14 to match those three; see the registration site for why no chord was invented. Then §5.8 lists Delete in **every** selection type's row. It is the one command in that section that exists (see `manifest::DIRECTED`), and it is wired: `PdfcerApp::dispatch_token` reads `SelectionState::deletable_objects_on`, the same rule the Delete key reads. **`format.properties` joined them on 2026-08-18**, with the ce-dimension properties section: a selected ce dimension's group, measurement, style overrides and radius/diameter switch are otherwise reachable only by noticing that a contextual tab appeared or by opening a dock panel by name, and the operator's report was *"I click and can't figure out how to enable some of the basic stuff."* It sits above Delete because the destructive row is last in every menu here. |
//! | [`CANVAS_MARKUP`] | a selected markup shape on the page | `format.properties`, `markup.add_node`, `markup.remove_node`, `edit.cut`, `edit.copy`, `edit.paste`, `format.delete` | ★★★ **The sixth canvas context, 2026-09-06, and the reason it is not [`CANVAS_OBJECT`] is that four of that menu's five rows are meaningless on an annotation.** `format.select_form` and `format.unshare_form` are about page content inside a form XObject; a markup annotation is not page content and is never inside one, so both would resolve, draw and do nothing — the *live and silently inert* class this project's `DEFECTS.md` is made of. What replaces them is the pair the operator asked for by name: *"I also can't edit or delete nodes of a markup shape once it is drawn."* See the block comment at the registration for the order, and [`crate::canvas::annotnodes::menu`] for why one of them can be greyed and the other absent on the very same shape. |
//! | [`CANVAS_EMPTY`] | blank page, or the paper beside the drawing | `view.zoom_fit_page`, `view.zoom_fit_width`, `view.zoom_fit_height`, `view.zoom_actual` | The four **named** zoom levels, all of which have a live dispatch arm today. A right-click on paper is about the *view*, because there is no object to be about. |
//! | [`DOCK_TAB`] | a panel tab in the dock | `view.reset_layout` | The only registered command that acts on the dock. The **command** is wired (`PdfcerApp::dispatch_command` calls `Modes::reset` with `ResetScope::All`); the **menu** still cannot be attached — see the warning below. |
//! | [`OBJECTS_ROW`] | a row in the Objects panel | `file.properties` | The Properties panel is *where an object row is described*; right-clicking a row focuses it and this is the command that puts the description on screen — which it now does: `PdfcerApp::show_panel` activates the panel, mounting it first if the operator's arrangement no longer holds it. |
//!
//! ## ★ `dock.tab` — the note below described a gap that CLOSED
//!
//! ⚠ **The heading and the paragraph under it were true until the
//! tab-menu seam landed, and are kept only for the record.**
//! `egui_shell::dock::Dock::with_tab_menu` exists,
//! `crate::app::surfaces::docks` supplies a handler, and this menu is
//! attached on every drawn panel tab — with its conditions corrected per
//! tab, so `view.panel_float` and `view.panel_dock` are never both offered.
//! It is additionally attached to a **floating** panel's header strip by
//! `crate::app::surfaces::floating_panels`, which is why one menu
//! definition serves two surfaces.
//!
//! The original note, unedited:
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

/// ★★★ Right-click on the page **over a selected markup shape**.
///
/// The sixth canvas menu, added 2026-09-06. Keyed on the **annotation
/// selection** — `SelectionState::annot` with
/// [`AnnotKind::Markup`](crate::canvas::selection::AnnotKind::Markup) — which is
/// neither a content selection nor a caret nor a field, so none of the other
/// five ever resolved for one.
///
/// # ★★ Why not just widen [`CANVAS_OBJECT`]
///
/// Because four of that menu's five rows are about **page content**, and an
/// annotation is not page content:
///
/// | that menu's row | on a markup shape |
/// |---|---|
/// | `format.select_form` | meaningless — an annotation is never inside a form XObject |
/// | `format.unshare_form` | meaningless, same reason |
/// | `view.zoom_selection` | works, and is kept |
/// | `format.properties` | works, and is the route to the Properties panel's markup section |
/// | `format.delete` | works, and stays last |
///
/// Two rows that resolve, draw and do nothing is the *live and silently inert*
/// class `DEFECTS.md` is made of, and R9's answer to *"this cannot apply"* is
/// nothing rather than greying — the shape will not become page content while
/// the operator looks at it.
///
/// ⇒ So a context of its own, carrying what a placed markup can actually
/// answer for: what it is, its two node verbs, the clipboard, and Delete.
pub const CANVAS_MARKUP: &str = "canvas.markup";

/// **The right-click landed on a segment of a shape that can take a new
/// point** — the `visible_when` of `markup.add_node`.
///
/// Set per right-click by [`crate::canvas::menus`], never by
/// `PdfcerApp::conditions`, for [`PANEL_DOCKED`]'s reason one step further
/// along: it is a fact about *one click on one edge*, and the frame's condition
/// set describes the frame. [`crate::canvas::annotnodes::menu::rows`] is what
/// answers it, and it answers it by **asking the engine**, so this name means
/// *the engine did not refuse this on grounds of the shape's kind*.
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
///
/// ★ This is the one condition in the pair that is genuinely *temporary*: a
/// closed shape keeps three points and an open one keeps two, and drawing
/// another corner makes the row live again. That is precisely why the row is
/// greyed rather than hidden, and why the command's tooltip states the floor.
pub const NODE_REMOVABLE: &str = "markup.node_removable";

/// Right-click on a panel tab in the dock.
///
/// Defined but not attachable from this crate — see the module header.
pub const DOCK_TAB: &str = "dock.tab";

/// **The panel under this tab is docked**, so it can be floated.
///
/// Set per drawn tab by `crate::app::surfaces`, never by
/// `PdfcerApp::conditions` — because it is a fact about *one tab*, and the
/// frame's condition set describes the frame. `MenuHost::with_conditions`
/// is the sanctioned way to correct a condition to a value the caller has
/// just computed, and its docs carry the argument for why that is not a
/// second source of truth.
pub const PANEL_DOCKED: &str = "panel.docked";

/// **The panel under this tab is in a window of its own**, so it can be
/// docked back. The complement of [`PANEL_DOCKED`]; exactly one of the two
/// holds for any panel that is being drawn at all.
pub const PANEL_FLOATING: &str = "panel.floating";

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
    CANVAS_MARKUP,
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
        // canvas.markup — a placed markup shape's menu.
        //
        // ★★★ **The sixth canvas context, 2026-09-06.** The operator's report of
        // 2026-09-05 is the whole commission:
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
        // ## ★★ The order, and the two rules it obeys
        //
        // 1. **Describe, then act, then destroy** — the same progression
        //    `canvas.object` uses and for the same reason.
        // 2. **The destructive row is last in every menu in this file.**
        //
        // So: what is it (`format.properties`) · the two node verbs, which are
        // why this menu exists · the clipboard · Delete.
        //
        // Separators between the three groups because they are three KINDS of
        // verb, which is what a rule is punctuation for. The menu engine
        // collapses a leading, trailing or doubled rule (`plan::collapse`), so
        // on a shape with no node rows — an `/Ink` stroke, a `/Square` — the two
        // rules around them become one and the menu reads as though the group
        // was never written.
        //
        // ## ★★★ The two node rows: `shown_when` AND greying, on one row
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
        // Three, and they are the three that exist. `canvas::annotclip` shipped
        // the lossless annotation route on 2026-09-05, and `edit.cut`,
        // `edit.copy` and `edit.paste` are all registered, all wired through
        // `dispatch::clipboard`, and all reach an annotation operand.
        //
        // ★ `edit.paste_duplicate` is deliberately absent even though it is
        // registered: over a markup clipboard `dispatch::clipboard` falls it
        // through to plain paste, so the row would be a second Paste under a
        // different name. Its subject is a form field, and `canvas.field` is
        // where it would belong the day that menu grows a clipboard group.
        //
        // ★★ `edit.cut` carries no `shown_when` here, unlike `format.delete`
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
            // ★★★ The same condition and the same constant `canvas.object`'s
            // and `canvas.field`'s Deletes carry. `app::conditions` publishes it
            // from a ladder whose annotation rung is guarded by `author_markup`,
            // which is what keeps this row alive in Review — deleting a comment
            // is exactly what Review is for.
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
        // ★★★ **The three per-panel layout verbs joined it on 2026-09-04.**
        //
        // Each is `shown_when` a condition the tab handler sets PER TAB — see
        // `crate::app::surfaces`, which corrects the frame's condition set
        // once per drawn tab through `MenuHost::with_conditions`. So the menu
        // on a docked panel's tab offers Float and Close; the menu on a
        // floating panel's header strip offers Dock and Close; and neither
        // ever shows a row that would do nothing.
        //
        // ★★ `shown_when` and not `enabled_when`, which is R9 exactly: an
        // unavailable capability renders NOTHING. "Dock" on a panel that is
        // already docked is not temporarily unavailable — it is meaningless —
        // and a greyed row would make the operator wonder what they had to do
        // to earn it.
        //
        // ★ Order: the two verbs that MOVE the panel first, the one that
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
        // ★★★ The two optional capabilities every pdfcer context menu is
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
mod tests;
