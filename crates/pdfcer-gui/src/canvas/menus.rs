//! # `canvas::menus` — the page's right-click
//!
//! Which menu opens is decided by **what the pointer was over** and by **what
//! is selected**, in the precedence [`CanvasMenu`]'s variants are written in:
//!
//! | Situation | Menu | Because |
//! |---|---|---|
//! | a caret in existing page text | [`crate::shell::menus::CANVAS_TEXT`] | the operator is *in* the words |
//! | a form field | [`crate::shell::menus::CANVAS_FIELD`] | a widget sits on top of whatever is beneath it |
//! | a selected **markup shape** | [`crate::shell::menus::CANVAS_MARKUP`] | it has verbs — including its corners — that no other menu carries |
//! | an object, reading | [`crate::shell::menus::CANVAS_READ_OBJECT`] | every other object row edits, and this mode cannot |
//! | an object | [`crate::shell::menus::CANVAS_OBJECT`] | there is a thing to act *on*, so the menu is about it |
//! | blank page | [`crate::shell::menus::CANVAS_EMPTY`] | there is no thing, so the menu is about the *view* |
//!
//! ⚠ That table said **"Two menus"** and listed two until 2026-09-06, while
//! this file resolved five. It is the same decay the parent document's own
//! heading carried, found the same way — by adding the sixth and reading what
//! was already written. The count is now [`CanvasMenu`]'s variant list, which
//! `each_canvas_menu_names_a_context_the_shell_defines` walks.
//!
//! `GUI_ROADMAP.md` Phase 1 is why this file exists at all:
//!
//! > Right now, selecting an object produces a highlighted tree row and no
//! > way to act on it. Everything a user would try next — **right-click**,
//! > drag a handle, press Delete, change the colour — either does nothing
//! > or does not exist.
//!
//! # ★ A right-click over an unselected object selects it first
//!
//! This is the behaviour every editor has and the reason is not
//! convenience. A context menu's implicit promise is *"these verbs apply to
//! the thing you pointed at"*. Without the select-first step the promise is
//! broken in the most damaging possible direction: point at object B while
//! object A is selected, choose **Delete**, and A is destroyed. The
//! operator's evidence for what was about to happen — the pointer — and the
//! application's operand list disagree, and the verb is irreversible in the
//! sense that matters (it costs an undo and a moment of not knowing what
//! just went).
//!
//! So [`select_under_right_click`] runs **before** the menu is attached, on
//! the same frame, and the ordering is the enforcement.
//!
//! Three rules, and each of the last two is a case the naive version gets
//! wrong:
//!
//! 1. **Over an unselected object** — replace the selection with it, at the
//!    Object rung. Exactly what a left click would do, through
//!    [`SelectionState::click`] itself rather than by assembling entries
//!    here, so the two gestures cannot disagree about what "select this
//!    object" means.
//! 2. **Over an object that is already selected** — *change nothing*. A
//!    marquee over eight objects followed by a right-click on one of them
//!    must offer to delete all eight, which is the whole point of building
//!    the set. Selecting the one under the pointer would silently discard
//!    seven, and the menu would then be about something the operator did
//!    not ask for. This is also what preserves an *entered* rung: right
//!    -clicking inside the object you have descended into leaves you inside
//!    it.
//! 3. **Over blank page** — *change nothing*, and in particular do **not
//!    clear**. A left click on paper deselects, and that is right: it is an
//!    unambiguous statement. A right-click is not — it is the opening of a
//!    question — and an operator who right-clicks slightly wide of their
//!    selection, sees a menu that is not the one they wanted, and presses
//!    Escape should still have their selection. Clearing here would make a
//!    mis-aimed right-click destroy work.
//!
//! # ★ Rule 4: a right-click marks nothing
//!
//! `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`, first
//! non-negotiable:
//!
//! > **A pre-commit affordance is not content marking.** A snap indicator,
//! > a hover highlight, a rubber-band, a selection handle — these are the
//! > *cursor*; they describe what is about to happen and they are welcome.
//! > What is forbidden is styling content that has **already been applied**
//! > as though it were pending.
//!
//! Nothing in this file paints. The only visible consequence of a
//! right-click is the **selection overlay** — which [`super::overlay`]
//! already draws for a left click, which is a pre-commit affordance by that
//! clause's own list, and which is drawn identically however the selection
//! arrived. The one-line test — *would a screenshot of the editing canvas
//! differ from a screenshot of the same document saved and reopened?* — is
//! answered by the selection overlay in exactly the way it already was, and
//! this file adds no second answer.
//!
//! # ★ Why the chosen menu is remembered rather than recomputed
//!
//! `egui` opens a context-menu popup on the secondary click and then draws
//! it on **every subsequent frame** until it is dismissed. The pointer
//! moves during those frames — onto the menu itself, which is not over the
//! object any more — so recomputing the context id per frame would swap the
//! menu's contents out from under the operator's hand while they were
//! reading it. Worse, both contexts resolve to the *same* popup id
//! (`egui::Popup::default_response_id` is derived from the response, not
//! from the context), so the swap would happen in place, with no
//! reopen and nothing on screen to explain it.
//!
//! The decision is therefore taken once, at the click, and stored in
//! `egui::Memory` for the life of the popup. `Memory` is the right home for
//! the same reason [`super::GESTURE_MEMORY_KEY`] is there and the selection
//! is not: this is frame-local interaction state — *which menu is open right
//! now* — with no meaning across a document, and `Memory` is per-`Context`,
//! so a document change starts the next frame with no popup in flight.
//!
//! # What this file does not do
//!
//! **It runs nothing.** [`attach`] returns `egui_shell::HandlerToken`s —
//! intent — which travel out through `canvas::show` to
//! `PdfcerApp::dispatch_token`, the same choke point the ribbon's Delete
//! goes through. That is what makes *"the context menu carries the same
//! commands again"* literally true: `format.delete` invoked from here and
//! `format.delete` invoked from the Format tab reach the identical arm, so
//! the rung guard in `SelectionState::deletable_objects_on` covers both and
//! cannot be stated twice.

use egui_shell::HandlerToken;

use crate::canvas::selection::{ClickHit, SelectionState};
use crate::canvas::target::TargetId;
use crate::shell::manifest::{DELETE_PERMITTED, SELECTION_ACTIONABLE, SELECTION_ANY};
use crate::shell::menus::{self, MenuHost};

/// `egui::Memory` key for which canvas menu is open.
///
/// See the module header: the choice is made at the click and read on every
/// frame the popup is drawn, so it has to outlive the click and must not
/// outlive the session. One `Id`, per `egui::Context`.
const MENU_MEMORY_KEY: &str = "pdfcer-canvas-menu"; // ui-text-exempt: internal memory id, never displayed

/// Trace slot for what a right-click on the canvas resolved to.
///
/// Separate from `super::trace::SELECTION_SLOT`, which reports what the *selection*
/// did. The two answer different questions and de-duplicate on different
/// timescales — a right-click that lands on an already-selected object
/// changes no selection at all and would otherwise be invisible.
const MENU_SLOT: &str = "canvas-menu"; // ui-text-exempt: trace slot name, never displayed

/// Which of the canvas's two menus a right-click asked for.
///
/// An enum rather than a `&'static str` in `Memory`, so the only two
/// answers are the two that exist and a typo cannot store a context id no
/// menu is defined for — which would silently degrade into "right-clicking
/// the canvas does nothing", the exact symptom this whole change removes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanvasMenu {
    /// The pointer was over an object: act on it.
    Object,
    /// ★★★ **A caret is placed in text already on the page**: act on the
    /// paragraph.
    ///
    /// Chosen ahead of both others when it applies, and the precedence is the
    /// design. A caret in a run means the operator is *in* that text — a
    /// right-click there is about the words, never about the object underneath
    /// them and never about the zoom level. Deciding by hit test first would
    /// give them the view menu, because a text run is not a hit-testable
    /// object.
    ///
    /// ★ It is `Anchor::Run` only. A caret placing NEW text (`Origin`/`Box`)
    /// has no paragraph behind it, so that operator gets the ordinary menus.
    Text,
    /// ★★★ **A form field is selected**: act on the field.
    ///
    /// Chosen ahead of [`Self::Object`] and [`Self::Empty`], below
    /// [`Self::Text`]. The precedence is a statement about what can be true at
    /// once rather than a tie-break — a caret and a field selection are
    /// mutually exclusive by construction (`canvas::forms` owns `/Widget`
    /// presses and only Edit mode offers selection at all), so the order here
    /// is documentation of that, and the field beats the object because a
    /// widget sits on top of whatever page content is underneath it.
    Field,
    /// ★★★ **Reading, and the pointer is over a picture**: offer to copy it.
    ///
    /// `OPERATOR_REQUESTS.md` **O71**. A picture became selectable in Read so
    /// it could be pasted into Word, and `Ctrl+C` was the only way to reach
    /// that — which is a route nobody discovers. Acrobat Reader offers *Copy
    /// Image* on the right-click and that is where somebody looks.
    ///
    /// ## ★★ Why a context of its own rather than [`Self::Object`]
    ///
    /// Because every other row of the object menu **edits**: Delete, unshare,
    /// re-aim to the container, the Properties panel's editable fields. Reusing
    /// that context in a mode which forbids all of them would draw a menu of
    /// controls the mode refuses, and R9's answer to *"this mode cannot"* is
    /// nothing rather than greying — greying is for the temporarily
    /// unavailable, and a mode is not temporary, it is a choice the operator
    /// has made and can unmake two inches away.
    ///
    /// ⇒ So this is a two-row menu with its own id, and the rows are the two
    /// things a reader can genuinely do with a picture: take a copy of it, and
    /// look at it more closely.
    ReadObject,
    /// ★★★ **A markup shape is selected**: act on the shape, and on the corner
    /// under the pointer.
    ///
    /// Chosen below [`Self::Text`] and [`Self::Field`] and **above**
    /// [`Self::Object`], [`Self::ReadObject`] and [`Self::Empty`]. The sixth
    /// canvas context, added 2026-09-06 with the right-click route to a shape's
    /// nodes.
    ///
    /// ## ★★ Keyed on the SELECTION, not on a hit test, and here is why that
    /// is not the field menu's mistake in reverse
    ///
    /// A right-click does not select an annotation. `canvas::annot`'s hit test
    /// runs on the **primary** press (`gesture::press_kind` reads
    /// `PointerButton::Primary` throughout), and `right_clicked_object` asks
    /// the *content* model, which an annotation is not in. So there is no
    /// hit-test answer to prefer here — `SelectionState::annot` is the only
    /// statement of *which shape this is about* that exists at the moment the
    /// popup opens, and it is a frame old at worst rather than a frame behind,
    /// because the click that made it was a different click.
    ///
    /// ⇒ The operator therefore **selects the shape, then right-clicks it**,
    /// which is what they already do to move or restyle one. What it costs is
    /// the one gesture `canvas.object` gives away free: a right-click on an
    /// *unselected* markup opens the view menu, not this one. That is a real
    /// gap and it is recorded rather than smoothed over — closing it needs the
    /// annotation hit test on the secondary button, which is a change in
    /// `canvas::interact`'s press pipeline and not in a menu.
    ///
    /// ## ★ When it wins over [`Self::Object`]
    ///
    /// When the pointer is **over the shape's own outline box**, or when it hit
    /// no content object at all. Not merely "a markup is selected": a markup
    /// selection and a right-click on a path forty points away are about
    /// different things, and taking the menu would leave the operator with the
    /// shape's verbs over an object they had just pointed at — the exact
    /// pointer-versus-operand disagreement `select_under_right_click`'s rule 1
    /// exists to remove, arriving from the other side.
    Markup,
    /// The pointer was over blank page: act on the view.
    ///
    /// The default, so a frame before any right-click has happened attaches
    /// a context that is harmless — nothing opens without a secondary
    /// click, and if one arrives the view menu is the correct answer for a
    /// pointer that has hit nothing.
    #[default]
    Empty,
}

impl CanvasMenu {
    /// The context id this menu is keyed by in [`crate::shell::menus`].
    #[must_use]
    pub fn context_id(self) -> &'static str {
        match self {
            Self::Object => menus::CANVAS_OBJECT,
            Self::Text => menus::CANVAS_TEXT,
            Self::Field => menus::CANVAS_FIELD,
            Self::ReadObject => menus::CANVAS_READ_OBJECT,
            Self::Markup => menus::CANVAS_MARKUP,
            Self::Empty => menus::CANVAS_EMPTY,
        }
    }
}

/// **Make the selection agree with what the pointer is over, and say which
/// menu that is.**
///
/// Pure but for the selection it is handed: no `egui`, no geometry, no
/// provider. The hit test has already happened; what is decided here is the
/// *policy*, and the policy is the part that can be wrong in a way an
/// operator would notice. See the module header for the three rules and
/// what each one prevents.
///
/// `object` is the front-most object under the pointer, or `None` for blank
/// page — the same answer `CanvasTargetProvider::hit_test` gives a left
/// click.
pub fn select_under_right_click(
    selection: &mut SelectionState,
    page: usize,
    object: Option<TargetId>,
) -> CanvasMenu {
    let Some(target) = object else {
        // Rule 3. Deliberately not `selection.clear()`: a mis-aimed
        // right-click must not destroy a set the operator spent five clicks
        // building.
        return CanvasMenu::Empty;
    };

    // Rule 2, before rule 1: an object already in the set is left alone,
    // which is what preserves both a multi-selection and an entered rung.
    //
    // `object_indices_on` rather than a walk over `entries()`, because it is
    // the same accessor `deletable_objects_on` builds the Delete operand
    // list from — so "is this one of the things Delete would act on" and
    // "is this selected" are answered from one place.
    // ★ Both lists, asked separately, because `object_indices_on` answers only
    // about the page's own paint order — a right-click on an already-selected
    // form-interior object would otherwise read as *not* selected and clear
    // the set the operator had built.
    let already_selected = match target {
        TargetId::Object(_) => target
            .page_object_index()
            .is_some_and(|index| selection.object_indices_on(page).contains(&index)),
        TargetId::Leaf(_) => target
            .leaf_index()
            .is_some_and(|index| selection.leaf_indices_on(page).contains(&index)),
    };
    if !already_selected {
        // Rule 1. Through `click` itself, at the Object rung, with no part
        // and no node: a right-click names a whole object. Assembling a
        // `Selection` here instead would be a second statement of what
        // "select this object" means, and the ladder's own rules — leaving
        // an entered object, normalising the entry list — would have to be
        // restated with it.
        selection.click(
            page,
            ClickHit {
                object: Some(target),
                part: None,
                node: None,
            },
            false,
            false,
        );
    }
    CanvasMenu::Object
}

/// **What a right-click landed on**, hit-tested at the object rung.
///
/// Moved here from `canvas::interact` on 2026-08-20 under R2, and it belongs
/// here on its merits rather than only on line count: the *only* consumer of
/// this answer is the context menu, and the two rules below are rules about
/// what a menu means. A caller that had to know them in order to feed
/// [`attach`] would be a caller that could get a menu wrong.
///
/// ★ **The OBJECT rung only** — `hit_test`, not `probe`. `probe` also asks for
/// the nearest part and node so that a double-click can descend, and a
/// right-click never descends: it names a whole object, because the verbs a
/// context menu offers act on whole objects. Asking for the deeper rungs would
/// pay for two extra provider queries on every right-click and discard both.
///
/// ★ It takes the frame's **screen** position and the frame's **one** mapping,
/// rather than a page point. The `PointerFrame` has been consumed by the
/// gesture machine by the time a menu is attached, so re-deriving the page
/// point through the same `map` is the frame's one conversion applied twice —
/// not a second conversion, which is the distinction `canvas::mapping`'s header
/// insists on.
///
/// `None` when there was no secondary click this frame, when nothing was
/// decomposed, or when the pointer is off the page.
#[must_use]
pub fn right_clicked_object(
    secondary_clicked: bool,
    targets: Option<&crate::panels::objects::provider::ObjectModelProvider>,
    screen_pos: Option<egui::Pos2>,
    map: &crate::canvas::mapping::PageMapping,
    page: usize,
) -> Option<TargetId> {
    if !secondary_clicked {
        return None;
    }
    let targets = targets?;
    let at = screen_pos?;
    targets.hit_test(page, map.to_page(at), map.tolerance())
}

/// **Everything one frame's canvas menu needs.**
///
/// A struct rather than twelve arguments, and it crossed the threshold the way
/// every other gesture type in this crate did — see [`attach`]'s note. What it
/// buys beyond satisfying a lint is that each fact can carry the note that says
/// *why it is here*, which twelve positional parameters cannot.
pub struct Attach<'a> {
    /// The canvas response the popup attaches to.
    pub response: &'a egui::Response,
    /// Mutated: a right-click over an unselected **content object** selects it.
    /// Read: a selected markup annotation is what `canvas.markup` is about.
    pub selection: &'a mut SelectionState,
    /// The page the canvas is showing.
    pub page: usize,
    /// The front-most content object under the pointer, or `None` for paper.
    pub object: Option<TargetId>,
    /// Whether this right-click is about a form field.
    pub field_selected: bool,
    /// Whether the DOCUMENT permits deleting a widget — the frame-top
    /// condition could not have known, so the caller answers it.
    pub field_delete_permitted: bool,
    /// Whether this mode reads rather than edits (O71).
    pub reading: bool,
    /// **Whether this mode may author markup.**
    ///
    /// ★ `author_markup`, deliberately not `!reading`. Review edits no content
    /// and authors every comment there is, so a markup menu gated on
    /// `edit_content` would be absent in the one mode whose entire subject is
    /// markup. The two capabilities are separate on
    /// `crate::app::modes::Capabilities` precisely so this distinction can be
    /// made, and `app::conditions`' delete ladder already makes it.
    pub author_markup: bool,
    /// The open document — for the annotation's geometry, and for the engine
    /// preflight behind the two node rows.
    pub doc: &'a crate::app::state::OpenDoc,
    /// Screen ↔ canvas for this page, for the node and segment hit tests.
    pub map: &'a crate::canvas::mapping::PageMapping,
    /// Where the pointer is, in screen points. `None` when it is off-window.
    pub screen_pos: Option<egui::Pos2>,
    /// `None` when the built-in manifest failed to validate, in which case
    /// nothing happens at all — including no selection change.
    pub host: Option<&'a MenuHost<'a>>,
}

/// Read, resolve and attach the canvas context menu for this frame.
///
/// Called on **every** frame, not only on the frame of the click: `egui`
/// draws an open popup until it is dismissed, and the popup only exists
/// while something is attached to the response.
///
/// # The order inside, and why it is this order
///
/// 1. **No host, nothing happens** — including no selection change. The
///    select-first step exists to make the menu about the thing you pointed
///    at; performed without a menu to follow it would be a right-click that
///    silently moved the selection, which is a surprise and not a feature.
///    `menus` is `None` only when the built-in manifest failed to validate,
///    which `PdfcerApp::new` treats as "the ribbon does not render and the
///    application stays usable for reading".
/// 2. **The secondary click**, which may move the selection and does decide
///    the context. Before the attach, so the conditions below describe the
///    selection the operator is about to act on.
/// 3. **The conditions, corrected.** `PdfcerApp::conditions()` was evaluated
///    at the top of the frame, before any widget was drawn, so its
///    `selection.any` predates step 2 by construction. Left stale,
///    `format.delete` resolves disabled, the menu has nothing enabled, and
///    the engine correctly refuses to open it — so the **first** right-click
///    on an object would do nothing at all, and no later frame could
///    recover because the popup is opened by the click.
///    `MenuHost::with_condition` carries the full account.
/// 4. **Attach**, which is also where *"a menu with nothing to offer never
///    opens"* is enforced: the engine resolves the menu and asks
///    `offers_anything` before it asks `egui` for a popup, and closes an
///    already-open popup whose offer has evaporated — so a menu left open
///    over a selection that is then deleted vanishes instead of lingering
///    with a dead Delete in it.
///
/// Returns the handler tokens the operator chose, for the caller to hand to
/// the application's one dispatch point. **Nothing here executes anything.**
///
/// ★ It took eight positional arguments until 2026-09-06 under a
/// `too_many_arguments` allow whose reason ended *"the resulting type would
/// have no name that was true"*. The markup menu brought four more — the
/// document, the mapping, the pointer and one capability, every one of them
/// needed to answer *which corner* — and twelve is past the point where the
/// argument holds: [`Attach`] does have a true name, it is *one right-click*,
/// and every field can now carry the note that explains it. The same
/// conversion `Press`, `Keys`, `Frame`, `Drag`, `Swept` and
/// [`super::rightclick::Click`] all made.
#[must_use]
pub fn attach(frame: Attach<'_>) -> Vec<HandlerToken> {
    let Attach {
        response,
        selection,
        page,
        object,
        field_selected,
        field_delete_permitted,
        reading,
        author_markup,
        doc,
        map,
        screen_pos,
        host,
    } = frame;
    // 1.
    let Some(host) = host else {
        return Vec::new();
    };
    let ctx = response.ctx.clone();

    // 2.
    if response.secondary_clicked() {
        // ★★ The caret wins, and it is asked BEFORE the hit test so the
        // selection is not disturbed on the way past: `select_under_right_click`
        // would replace the object selection with whatever happens to sit under
        // a paragraph, which the operator did not ask for and cannot see.
        let chosen = if caret_in_existing_text(&ctx) {
            CanvasMenu::Text
        } else if field_selected {
            // ★★ Asked BEFORE the hit test, and the ordering is the same
            // protection the caret rung gets: `select_under_right_click` would
            // replace the object selection with whatever page content sits
            // under the widget, which the operator did not ask for and cannot
            // see behind the field's own outline.
            //
            // ★★★ The SELECTION is what is read here, not a hit test, and that
            // is deliberate: `canvas::forms::select_click` has already made the
            // field under the pointer the selected one on this very frame — a
            // secondary click selects exactly as a primary does, minus the
            // clear-on-paper. So *"is a field selected"* and *"did they
            // right-click a field"* are one question by the time this runs, and
            // asking it twice with two hit tests is how the two answers drift.
            CanvasMenu::Field
        } else if markup_menu(selection, author_markup, object, map, screen_pos) {
            // ★★★ **A selected markup shape**, 2026-09-06. See
            // [`CanvasMenu::Markup`] for the precedence argument and for what
            // this deliberately does not do — it does not select the shape,
            // because a right-click has never selected an annotation and making
            // it do so is a change in the press pipeline rather than in a menu.
            //
            // ★★ The PICK is taken here and parked, on this one frame, because
            // this is the only frame on which the pointer is still over the
            // shape. Every later frame of the popup's life has the pointer on
            // the menu itself. `annotnodes::menu`'s header carries the whole
            // argument, and it is `MENU_MEMORY_KEY`'s own argument one operand
            // deeper.
            let pick = screen_pos
                .map_or(crate::canvas::annotnodes::menu::NodePick::Elsewhere, |at| {
                    crate::canvas::annotnodes::menu::pick_at(doc, map, selection, at)
                });
            crate::canvas::annotnodes::menu::park(&ctx, pick);
            if let Some(annot) = selection.annot() {
                crate::canvas::annotnodes::menu::trace(
                    annot.target.id,
                    pick,
                    crate::canvas::annotnodes::menu::rows(doc, selection, pick),
                );
            }
            CanvasMenu::Markup
        } else if reading {
            // ★★★ **Reading**: the object menu's rows all edit, so this mode
            // gets its own two-row menu — O71. See [`CanvasMenu::ReadObject`].
            //
            // # Why the gate is HERE and was not before
            //
            // `canvas::interact` computed `secondary_clicked &&
            // caps.edit_content`, so a right-click anywhere in Read or Review
            // was discarded before this function ever ran — no menu at all,
            // not even the view menu that `CANVAS_EMPTY`'s own registration
            // calls *"the correct menu for a reader"*. That sentence was true
            // and unreachable for the life of the shell.
            //
            // ⇒ The question a mode answers is **which menu**, not *whether a
            // right-click is heard*. Moving it here is what let Read gain the
            // two rows it should always have had, and what stops a future mode
            // needing an edit in two files to get a menu at all.
            //
            // The selection still moves, through the same function and the
            // same three rules, because a menu about *this picture* has to be
            // about the one under the pointer. What differs is only which menu
            // is named at the end, and a miss still resolves to the view menu —
            // which is the right answer for a reader who right-clicked paper,
            // and is the answer this mode has been giving since the day it
            // could open a menu at all.
            match select_under_right_click(selection, page, object) {
                CanvasMenu::Object => CanvasMenu::ReadObject,
                other => other,
            }
        } else {
            select_under_right_click(selection, page, object)
        };
        store(&ctx, chosen);
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI.
                // Placed directly above the literal — see `super::trace_layout`.
                "canvas-menu context={} sel={} level={:?}",
                chosen.context_id(),
                selection.len(),
                selection.level(),
            )
        });
    }

    // 3.
    // ★★ THREE conditions, corrected on the same frame and for one reason.
    // `PdfcerApp::conditions()` ran at the top of the frame, before step 2 could
    // move the selection, so a first right-click on an object would otherwise
    // resolve `format.delete` disabled and the engine — correctly — would
    // decline to open a menu with nothing in it.
    //
    // ★ `selection.actionable` is the wider of the two: it is also set for a
    // selected form field, which is not in `SelectionState`. Without it here,
    // `canvas.field`'s items would resolve disabled and the menu would never
    // open at all — the state `offers_anything` is built to prevent, met from
    // the one direction it cannot see.
    //
    // ★★ It used to say *"both items"*, and that stopped being true on
    // 2026-08-29: `canvas.field`'s `format.delete` now carries
    // `selection.delete_permitted` as its `visible_when`, so on a document
    // whose form structure is frozen the menu offers `format.properties`
    // alone — one item, still enough for `offers_anything`, and the Delete is
    // ABSENT rather than greyed (R9).
    //
    // ★★★ And `selection.delete_permitted`, corrected for the SAME reason and
    // on the same frame — added 2026-08-29, with the form half of R83.
    //
    // The frame-top condition set answers this from
    // `panels::properties::formfield::refuses_delete`, which requires
    // `doc.selected_field` to be set. But `rightclick::Click::field_menu` opens
    // this menu for a widget merely **under the pointer**, so on a FIRST
    // right-click over an unselected widget the frame-top answer was computed
    // with no field selected — it read the annotation arm of the ladder, found
    // nothing selected, and published *permitted*. The row would then be drawn
    // over a certified form for one frame, which is precisely the *drawn and
    // silently inert* state R9 and R83 exist to remove; "for one frame" is not
    // "not at all", and it is the frame the pointer is already in.
    //
    // ⇒ The caller answers the DOCUMENT's half of the question
    // (`formfield::document_refuses_delete` — no selection in it) and passes
    // it in, exactly as it already does for `selection.actionable`. Three
    // conditions, one reason: `PdfcerApp::conditions()` ran before the click
    // that decided what this menu is about.
    //
    // ★★★ **Applied ONLY to `canvas.field`, and the narrowness is the whole
    // correctness argument.** `with_conditions` overrides a name for whatever
    // menu is about to be drawn, and `canvas.object` carries the identical
    // `visible_when` for a different subject: page content and annotations,
    // gated by `annotation_deletion_refusal` and by
    // `SelectionState::deletable_objects_on`. Overriding it there with the
    // FORMS answer would hide Delete from every selected content object on any
    // certified document — a control withheld where it would have worked,
    // which this project holds to be the worse defect of the two, because the
    // operator is left with no gesture that reports it.
    //
    // The menu is therefore read FIRST (step 4's `load`, hoisted), and the
    // override is conditional on it. `load` is a memory read of what step 2
    // just stored; reading it one statement earlier costs nothing.
    let chosen = load(&ctx);
    let mut overrides = vec![
        (SELECTION_ANY, !selection.is_empty()),
        (
            SELECTION_ACTIONABLE,
            !selection.is_empty() || field_selected,
        ),
    ];
    if matches!(chosen, CanvasMenu::Field) {
        overrides.push((DELETE_PERMITTED, field_delete_permitted));
    }
    // ★★★ **The two node rows' four conditions**, corrected here and nowhere
    // else, for the same reason and with the same narrowness as the Delete
    // above: they are facts about ONE right-click on ONE edge, and
    // `PdfcerApp::conditions()` ran before that click existed.
    //
    // ★★ Asked from the **parked** pick rather than from the live pointer.
    // `attach` runs on every frame the popup is drawn and the pointer is on the
    // menu by the second of them; recomputing would grey the row the operator's
    // hand was travelling toward. `annotnodes::menu`'s header carries the
    // argument; this is the call site it is about.
    //
    // ★ The engine preflight behind [`rows`] costs one annotation walk per row,
    // and it is paid only inside this `matches!` — a right-click anywhere else
    // on the canvas asks the engine nothing.
    if matches!(chosen, CanvasMenu::Markup) {
        let rows = crate::canvas::annotnodes::menu::rows(
            doc,
            selection,
            crate::canvas::annotnodes::menu::parked(&ctx),
        );
        overrides.extend([
            (menus::NODE_INSERT_OFFERED, rows.insert.shown()),
            (menus::NODE_INSERTABLE, rows.insert.enabled()),
            (menus::NODE_REMOVE_OFFERED, rows.remove.shown()),
            (menus::NODE_REMOVABLE, rows.remove.enabled()),
        ]);
    }
    let conditions = host.with_conditions(&overrides);

    // 4.
    let tokens = host.attach_with(response, chosen.context_id(), &conditions);
    if !tokens.is_empty() {
        crate::diag::trace_changed(MENU_SLOT, || {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI.
                "canvas-menu-invoked context={} tokens={}",
                chosen.context_id(),
                tokens.len(),
            )
        });
    }
    tokens
}

/// **Is this right-click about a placed markup shape?**
///
/// Three conditions, and each of the last two is a case the one-line version
/// gets wrong.
///
/// 1. **A markup annotation is selected.** [`AnnotKind::Markup`] and not a ce
///    dimension — Rule 15. A ce dimension is also a `/Line`, its corners are
///    [`crate::canvas::dimdrag`]'s, and its verb `move_dimension_vertex`
///    **re-measures**. Routing one to this menu would offer *Add a point here*
///    on a dimension, where the engine refuses by name
///    (`EditError::AnnotationIsCeDimension`) and where the operator's next
///    question would be why the measurement did not follow.
/// 2. **The mode may author markup.** `author_markup`, not `edit_content`, so
///    Review — whose whole subject is comments — gets the menu it exists for.
/// 3. **The pointer is over the shape, or over nothing.** A markup selection
///    plus a right-click on a path forty points away are about different
///    things; taking the menu there would leave the operator with the shape's
///    verbs over the object they had just pointed at. Over blank paper the
///    shape wins, on `select_under_right_click`'s rule 3 reasoning — a
///    right-click is the opening of a question, and an operator who aims a
///    little wide of the shape they have selected meant the shape.
///
/// ★ The containment test is on the annotation's own `/Rect` outline, in
/// **canvas** space, which is the space both that outline and
/// [`PageMapping::to_page`] speak. It is expanded by the mapping's own click
/// tolerance rather than by a number invented here, so *"near the shape"* means
/// the same distance a click means everywhere else on this canvas.
fn markup_menu(
    selection: &SelectionState,
    author_markup: bool,
    object: Option<TargetId>,
    map: &crate::canvas::mapping::PageMapping,
    screen_pos: Option<egui::Pos2>,
) -> bool {
    if !author_markup {
        return false;
    }
    let Some(annot) = selection.annot() else {
        return false;
    };
    if annot.target.kind != crate::canvas::selection::AnnotKind::Markup {
        return false;
    }
    if object.is_none() {
        return true;
    }
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the mapping's tolerance is a click radius in page units — single digits — and the outline it expands is an f32 rect" // ui-text-exempt: a lint justification, never displayed
    )]
    let slack = map.tolerance() as f32;
    screen_pos.is_some_and(|at| annot.outline.expand(slack).contains(map.to_page(at)))
}

/// Whether a caret is placed in text that is **already on the page**.
///
/// The one question that separates [`CanvasMenu::Text`] from the other two, and
/// it is asked of the draft rather than of the tool: an *armed but unclicked*
/// text tool has no paragraph, and an operator who has armed it and then
/// right-clicked a rectangle wants the rectangle's menu.
fn caret_in_existing_text(ctx: &egui::Context) -> bool {
    matches!(
        crate::canvas::textedit::read(ctx).map(|draft| draft.anchor),
        Some(crate::canvas::textedit::Anchor::Run { .. })
    )
}

/// Read which canvas menu the last right-click asked for.
fn load(ctx: &egui::Context) -> CanvasMenu {
    let id = egui::Id::new(MENU_MEMORY_KEY);
    ctx.data_mut(|d| d.get_temp::<CanvasMenu>(id).unwrap_or_default())
}

/// Write which canvas menu this right-click asked for.
fn store(ctx: &egui::Context, menu: CanvasMenu) {
    let id = egui::Id::new(MENU_MEMORY_KEY);
    ctx.data_mut(|d| d.insert_temp(id, menu));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::selection::AnnotKind;
    use crate::shell::menus::{CANVAS_EMPTY, CANVAS_OBJECT};

    /// A selection holding whole objects `indices` on page 0.
    fn selected(indices: &[u64]) -> SelectionState {
        let mut selection = SelectionState::default();
        for (n, index) in indices.iter().enumerate() {
            selection.click(
                0,
                ClickHit {
                    object: Some(TargetId::Object(*index)),
                    ..ClickHit::default()
                },
                // The first click replaces; the rest add, exactly as a
                // Shift+click sequence would build the set.
                n > 0,
                false,
            );
        }
        selection
    }

    /// **★ A right-click over an unselected object selects it first.**
    ///
    /// The rule that makes the menu about the thing the operator pointed at.
    /// Without it, right-clicking B while A is selected and choosing Delete
    /// destroys A — the pointer and the operand list disagreeing, with an
    /// irreversible verb between them.
    #[test]
    fn a_right_click_over_an_unselected_object_selects_it() {
        let mut selection = selected(&[7]);
        let menu = select_under_right_click(&mut selection, 0, Some(TargetId::Object(3)));

        assert_eq!(menu, CanvasMenu::Object);
        assert_eq!(
            selection.object_indices_on(0),
            vec![3],
            "the object under the pointer must BE the selection, or the menu's verbs \
             apply to something the operator did not point at"
        );
    }

    /// **★ …and a right-click over an object that is already selected
    /// changes nothing.**
    ///
    /// The case the naive version gets wrong. A marquee over eight objects
    /// followed by a right-click on one of them must still offer to delete
    /// all eight; collapsing to the one under the pointer silently discards
    /// seven and makes the menu about something nobody asked for.
    #[test]
    fn a_right_click_inside_a_multi_selection_keeps_it() {
        let mut selection = selected(&[1, 4, 9]);
        let menu = select_under_right_click(&mut selection, 0, Some(TargetId::Object(4)));

        assert_eq!(menu, CanvasMenu::Object);
        assert_eq!(
            selection.object_indices_on(0),
            vec![1, 4, 9],
            "right-clicking one member of a set must not collapse the set"
        );
    }

    /// **★ A right-click on blank page never clears the selection.**
    ///
    /// A left click on paper deselects, and that is right — it is an
    /// unambiguous statement. A right-click is the opening of a question,
    /// and an operator who aims slightly wide, gets the view menu and
    /// presses Escape must still have their selection.
    #[test]
    fn a_right_click_on_blank_page_opens_the_view_menu_and_keeps_the_selection() {
        let mut selection = selected(&[2, 5]);
        let menu = select_under_right_click(&mut selection, 0, None);

        assert_eq!(menu, CanvasMenu::Empty);
        assert_eq!(
            selection.object_indices_on(0),
            vec![2, 5],
            "a mis-aimed right-click must not destroy a set the operator built"
        );
    }

    /// With nothing selected, a right-click on paper is still the view menu
    /// and still selects nothing.
    #[test]
    fn a_right_click_on_empty_paper_with_no_selection_selects_nothing() {
        let mut selection = SelectionState::default();
        assert_eq!(
            select_under_right_click(&mut selection, 0, None),
            CanvasMenu::Empty
        );
        assert!(selection.is_empty());
    }

    /// **★ Right-clicking the object you are inside keeps you inside it.**
    ///
    /// A descended rung is expensive to reach — one measured CAD export
    /// holds a whole drawing view as a single path object with 1,194
    /// subpaths, so finding the subpath you meant took aim. Re-selecting the
    /// whole object because the operator right-clicked it would throw that
    /// away, and the ascent is the one thing Escape is *for*.
    ///
    /// This falls out of rule 2 rather than being a special case, which is
    /// why it is asserted: the object is already in
    /// `object_indices_on`, so nothing runs.
    #[test]
    fn a_right_click_inside_an_entered_object_does_not_ascend() {
        use crate::canvas::selection::SelectionLevel;

        let mut selection = selected(&[3]);
        // Double-click into part 1, the way the canvas descends.
        selection.click(
            0,
            ClickHit {
                object: Some(TargetId::Object(3)),
                part: Some(1),
                node: None,
            },
            false,
            true,
        );
        assert_eq!(selection.level(), SelectionLevel::Part);

        let menu = select_under_right_click(&mut selection, 0, Some(TargetId::Object(3)));
        assert_eq!(menu, CanvasMenu::Object);
        assert_eq!(
            selection.level(),
            SelectionLevel::Part,
            "right-clicking the object you have descended into must not ascend out of it"
        );
    }

    /// …but right-clicking a *different* object while inside one leaves,
    /// exactly as a left click would.
    ///
    /// The rule that stops an operator being stranded inside an object they
    /// have forgotten they entered. It is `SelectionState::click`'s own
    /// behaviour, reached rather than reimplemented, which is the point of
    /// routing through it.
    #[test]
    fn a_right_click_on_a_different_object_leaves_the_entered_one() {
        use crate::canvas::selection::SelectionLevel;

        let mut selection = selected(&[3]);
        selection.click(
            0,
            ClickHit {
                object: Some(TargetId::Object(3)),
                part: Some(1),
                node: None,
            },
            false,
            true,
        );
        assert_eq!(selection.level(), SelectionLevel::Part);

        select_under_right_click(&mut selection, 0, Some(TargetId::Object(8)));
        assert_eq!(selection.level(), SelectionLevel::Object);
        assert_eq!(selection.object_indices_on(0), vec![8]);
    }

    /// The two menus map to the two context ids the shell defines, and to no
    /// others.
    ///
    /// A `&'static str` in `Memory` could store a context id that no menu is
    /// keyed by, which degrades into "right-clicking the canvas does
    /// nothing" — the symptom this whole change removes. The enum is what
    /// makes that unrepresentable; this is what proves the two arms point at
    /// menus that exist.
    #[test]
    fn each_canvas_menu_names_a_context_the_shell_defines() {
        assert_eq!(CanvasMenu::Object.context_id(), CANVAS_OBJECT);
        // ★ The third, added with paragraph reflow. Its menu is the only route
        // to that command that does not go through the ribbon, so a context id
        // that drifted from `shell::menus` would silently take the canvas route
        // away and leave the ribbon working — a half-loss no other test sees.
        assert_eq!(
            CanvasMenu::Text.context_id(),
            crate::shell::menus::CANVAS_TEXT
        );
        // ★ The fourth, added with the form-field menu. Same argument as the
        // third: this is the only route to acting on a field by pointing at it,
        // so a drifted id takes the canvas route away and leaves the Forms
        // panel working — a half-loss no other test sees.
        assert_eq!(
            CanvasMenu::Field.context_id(),
            crate::shell::menus::CANVAS_FIELD
        );
        assert_eq!(CanvasMenu::Empty.context_id(), CANVAS_EMPTY);
        assert_eq!(
            CanvasMenu::default(),
            CanvasMenu::Empty,
            "a frame before any right-click must attach the view menu; the object menu \
             would claim a selection the pointer has not been shown to be over"
        );

        // ★ The fifth, added with the markup menu. Same argument as the third
        // and fourth, and one more that is specific to it: `markup.add_node`
        // and `markup.remove_node` are in `manifest::TAB_SCOPED`, which means
        // this menu is their ONLY surface. A drifted context id would take the
        // two node verbs away entirely, with the ribbon showing nothing missing
        // because the ribbon never had them.
        assert_eq!(
            CanvasMenu::Markup.context_id(),
            crate::shell::menus::CANVAS_MARKUP
        );
        assert_eq!(CanvasMenu::Empty.context_id(), CANVAS_EMPTY);
        assert_eq!(
            CanvasMenu::default(),
            CanvasMenu::Empty,
            "a frame before any right-click must attach the view menu; the object menu \
             would claim a selection the pointer has not been shown to be over"
        );

        let menus = crate::shell::menus::built_in();
        for menu in [CanvasMenu::Object, CanvasMenu::Markup, CanvasMenu::Empty] {
            assert!(
                menus.get(menu.context_id()).is_some(),
                "`{}` is attached by the canvas and defined by no menu",
                menu.context_id()
            );
        }
    }

    // -----------------------------------------------------------------------
    // `markup_menu` — the three conditions, one test each, every one falsified
    // by removing the clause it is about.
    // -----------------------------------------------------------------------

    /// A markup selection, outlined over the canvas rect `outline`.
    fn markup_selection(kind: AnnotKind, outline: egui::Rect) -> SelectionState {
        let mut selection = SelectionState::default();
        selection.select_annot(crate::canvas::selection::AnnotSelection {
            target: crate::canvas::selection::AnnotTarget {
                page: 0,
                id: pdfcer_core::object::ObjId::new(7, 0),
                kind,
                subtype: "Polygon".to_owned(),
                locked: false,
            },
            outline,
        });
        selection
    }

    /// A 1:1 mapping whose canvas origin is the screen origin, so a test can
    /// name screen points and canvas points with the same numbers.
    fn identity_map() -> crate::canvas::mapping::PageMapping {
        crate::canvas::mapping::PageMapping::new(
            egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(600.0, 800.0)),
            (600.0, 800.0),
            1.0,
        )
    }

    /// **A selected markup shape, right-clicked, opens the markup menu.**
    ///
    /// The whole point of the sixth context: the operator has a shape selected,
    /// points at it, and gets the menu that carries its two node verbs.
    ///
    /// ★ Falsified by returning `false` from `markup_menu` unconditionally —
    /// which is the state before this change, where the same right-click
    /// resolved to `canvas.empty` and offered four zoom levels.
    #[test]
    fn a_right_click_on_a_selected_markup_opens_its_own_menu() {
        let map = identity_map();
        let selection = markup_selection(
            AnnotKind::Markup,
            egui::Rect::from_min_max(egui::pos2(100.0, 100.0), egui::pos2(200.0, 200.0)),
        );
        assert!(markup_menu(
            &selection,
            true,
            Some(TargetId::Object(3)),
            &map,
            Some(egui::pos2(150.0, 150.0)),
        ));
    }

    /// **Rule 15: a ce dimension is NOT routed here.**
    ///
    /// Its corners are `canvas::dimdrag`'s and its verb re-measures. Offering
    /// *Add a point here* on one would name an engine verb that refuses by name
    /// (`AnnotationIsCeDimension`) and would leave the operator asking why the
    /// measurement did not follow.
    ///
    /// ★ Falsified by dropping the `kind != Markup` clause: this test fails and
    /// no other one does, which is exactly why it is written separately from
    /// the one above rather than as a second assertion inside it.
    #[test]
    fn a_selected_ce_dimension_does_not_open_the_markup_menu() {
        let map = identity_map();
        let selection = markup_selection(
            AnnotKind::CeDimension,
            egui::Rect::from_min_max(egui::pos2(100.0, 100.0), egui::pos2(200.0, 200.0)),
        );
        assert!(!markup_menu(
            &selection,
            true,
            None,
            &map,
            Some(egui::pos2(150.0, 150.0)),
        ));
    }

    /// **A mode that cannot author markup gets no markup menu**, and the
    /// capability asked is `author_markup`.
    ///
    /// ★ Falsified by passing `!reading` (i.e. `edit_content`) instead: Review
    /// has `edit_content == false` and `author_markup == true`, so the mode
    /// whose entire subject is comments would lose the comment's own menu.
    #[test]
    fn a_mode_that_cannot_author_markup_gets_no_markup_menu() {
        let map = identity_map();
        let selection = markup_selection(
            AnnotKind::Markup,
            egui::Rect::from_min_max(egui::pos2(100.0, 100.0), egui::pos2(200.0, 200.0)),
        );
        assert!(!markup_menu(
            &selection,
            false,
            None,
            &map,
            Some(egui::pos2(150.0, 150.0)),
        ));
    }

    /// **A right-click on a content object far from the selected shape is
    /// about the OBJECT.**
    ///
    /// The pointer and the operand must agree — `select_under_right_click`'s
    /// rule 1, arriving from the other side. Taking the markup menu here would
    /// leave the operator holding a shape's verbs over the path they had just
    /// pointed at.
    ///
    /// ★ Falsified by dropping the containment test and returning `true`
    /// whenever a markup is selected.
    #[test]
    fn a_right_click_on_a_distant_object_is_about_the_object() {
        let map = identity_map();
        let selection = markup_selection(
            AnnotKind::Markup,
            egui::Rect::from_min_max(egui::pos2(100.0, 100.0), egui::pos2(200.0, 200.0)),
        );
        assert!(!markup_menu(
            &selection,
            true,
            Some(TargetId::Object(3)),
            &map,
            Some(egui::pos2(500.0, 500.0)),
        ));
    }

    /// …but a right-click on **paper** while a markup is selected still opens
    /// the shape's menu.
    ///
    /// `select_under_right_click`'s rule 3 reasoning: a right-click is the
    /// opening of a question, and an operator who aims a little wide of the
    /// shape they have selected meant the shape. It is also what stops the
    /// commonest miss — a shape drawn thin, aimed at from just outside its box
    /// — from silently becoming the zoom menu.
    ///
    /// ★ Falsified by removing the `object.is_none()` early return.
    #[test]
    fn a_right_click_on_paper_beside_a_selected_markup_keeps_its_menu() {
        let map = identity_map();
        let selection = markup_selection(
            AnnotKind::Markup,
            egui::Rect::from_min_max(egui::pos2(100.0, 100.0), egui::pos2(200.0, 200.0)),
        );
        assert!(markup_menu(
            &selection,
            true,
            None,
            &map,
            Some(egui::pos2(500.0, 500.0)),
        ));
    }
}
