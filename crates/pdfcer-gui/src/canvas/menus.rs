//! # `canvas::menus` — the page's right-click
//!
//! Which menu opens is decided by **what the pointer was over** and by **what
//! is selected**, in the precedence [`CanvasMenu`]'s variants are written in:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/menus.md`.

use egui_shell::HandlerToken;

use crate::canvas::selection::{ClickHit, SelectionState};
use crate::canvas::target::TargetId;
use crate::shell::manifest::{DELETE_PERMITTED, SELECTION_ACTIONABLE, SELECTION_ANY};
use crate::shell::menus::{self, MenuHost};

/// `egui::Memory` key for which canvas menu is open.
const MENU_MEMORY_KEY: &str = "pdfcer-canvas-menu"; // ui-text-exempt: internal memory id, never displayed

/// Trace slot for what a right-click on the canvas resolved to.
const MENU_SLOT: &str = "canvas-menu"; // ui-text-exempt: trace slot name, never displayed

/// Which of the canvas's two menus a right-click asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanvasMenu {
    /// The pointer was over an object: act on it.
    Object,
    /// **A caret is placed in text already on the page**: act on the
    /// paragraph.
    ///
    /// Chosen ahead of both others when it applies, and the precedence is the
    /// design. A caret in a run means the operator is *in* that text — a
    /// right-click there is about the words, never about the object underneath
    /// them and never about the zoom level. Deciding by hit test first would
    /// give them the view menu, because a text run is not a hit-testable
    /// object.
    ///
    /// It is `Anchor::Run` only. A caret placing NEW text (`Origin`/`Box`)
    /// has no paragraph behind it, so that operator gets the ordinary menus.
    Text,
    /// **A form field is selected**: act on the field.
    ///
    /// Chosen ahead of [`Self::Object`] and [`Self::Empty`], below
    /// [`Self::Text`]. The precedence is a statement about what can be true at
    /// once rather than a tie-break — a caret and a field selection are
    /// mutually exclusive by construction (`canvas::forms` owns `/Widget`
    /// presses and only Edit mode offers selection at all), so the order here
    /// is documentation of that, and the field beats the object because a
    /// widget sits on top of whatever page content is underneath it.
    Field,
    /// **Reading, and the pointer is over a picture**: offer to copy it.
    ///
    /// `OPERATOR_REQUESTS.md` **O71**. A picture became selectable in Read so
    /// it could be pasted into Word, and `Ctrl+C` was the only way to reach
    /// that — which is a route nobody discovers. Acrobat Reader offers *Copy
    /// Image* on the right-click and that is where somebody looks.
    ///
    /// ## Why a context of its own rather than [`Self::Object`]
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
    /// **A markup shape is selected**: act on the shape, and on the corner
    /// under the pointer.
    ///
    ///
    /// ## Keyed on the SELECTION, not on a hit test, and here is why that
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
    /// ## When it wins over [`Self::Object`]
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
    // Both lists, asked separately, because `object_indices_on` answers only
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
                // A right-click names a whole object. With no part there is
                // nothing to narrow to, so this is stating the rule rather
                // than declining it.
                chunk: false,
            },
            false,
            false,
        );
    }
    CanvasMenu::Object
}

/// **What a right-click landed on**, hit-tested at the object rung.
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
    /// **The decomposed page**, for the one question `object` cannot answer:
    /// *which LINE of that text block is the pointer on.*
    ///
    ///
    /// `None` when nothing has been decomposed, which is
    /// [`crate::canvas::runmenu::RunPick::Elsewhere`] and therefore no row.
    pub targets: Option<&'a crate::panels::objects::provider::ObjectModelProvider>,
    /// Whether this right-click is about a form field.
    pub field_selected: bool,
    /// Whether the DOCUMENT permits deleting a widget — the frame-top
    /// condition could not have known, so the caller answers it.
    pub field_delete_permitted: bool,
    /// Whether this mode reads rather than edits (O71).
    pub reading: bool,
    /// **Whether this mode may author markup.**
    ///
    /// `author_markup`, deliberately not `!reading`. Review edits no content
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
#[must_use]
pub fn attach(frame: Attach<'_>) -> Vec<HandlerToken> {
    let Attach {
        response,
        selection,
        page,
        object,
        targets,
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
        // The caret wins, and it is asked BEFORE the hit test so the
        // selection is not disturbed on the way past: `select_under_right_click`
        // would replace the object selection with whatever happens to sit under
        // a paragraph, which the operator did not ask for and cannot see.
        let chosen = if caret_in_existing_text(&ctx) {
            CanvasMenu::Text
        } else if field_selected {
            // Asked BEFORE the hit test, and the ordering is the same
            // protection the caret rung gets: `select_under_right_click` would
            // replace the object selection with whatever page content sits
            // under the widget, which the operator did not ask for and cannot
            // see behind the field's own outline.
            //
            // The SELECTION is what is read here, not a hit test, and that
            // is deliberate: `canvas::forms::selecting::select_click` has already made the
            // field under the pointer the selected one on this very frame — a
            // secondary click selects exactly as a primary does, minus the
            // clear-on-paper. So *"is a field selected"* and *"did they
            // right-click a field"* are one question by the time this runs, and
            // asking it twice with two hit tests is how the two answers drift.
            CanvasMenu::Field
        } else if markup_menu(selection, author_markup, object, map, screen_pos) {
            //
            // The PICK is taken here and parked, on this one frame, because
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
            // **Reading**: the object menu's rows all edit, so this mode
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
        // **The run pick, taken on this one frame and parked** — O188(A).
        //
        // Same mechanism, same reason and the same memory discipline as the
        // markup node pick four screens up: this is the only frame on which
        // the pointer is still over the text. `egui` draws the popup on every
        // frame until it is dismissed, and by the second of them the pointer
        // is on the menu. `crate::canvas::runmenu`'s header carries the whole
        // argument.
        //
        // Taken from `object` — the hit test `right_clicked_object` already
        // ran — and NOT from the selection, which step 2 has just changed. The
        // row is about the thing the pointer is on, and `select_under_right_click`
        // may legitimately have left a different thing selected (a multi-object
        // selection the click landed inside is preserved, by its second rule).
        //
        // **Parked on every right-click, not only on the object menu.** The
        // `else` arm is what makes a stale pick impossible: without it, a
        // right-click on paper would leave the previous click's line parked,
        // and the next frame that read it would be reading an operand from a
        // gesture two clicks ago. The markup pick above is narrower because its
        // condition override is guarded by `matches!(chosen, Markup)`; this one
        // does not rely on that guard to be correct.
        let run_pick = if matches!(chosen, CanvasMenu::Object) {
            crate::canvas::runmenu::pick_at(targets, page, object, map, screen_pos)
        } else {
            crate::canvas::runmenu::RunPick::Elsewhere
        };
        crate::canvas::runmenu::park(&ctx, run_pick);
        crate::canvas::runmenu::trace(run_pick);
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
    // THREE conditions, corrected on the same frame and for one reason.
    // `PdfcerApp::conditions()` ran at the top of the frame, before step 2 could
    // move the selection, so a first right-click on an object would otherwise
    // resolve `format.delete` disabled and the engine — correctly — would
    // decline to open a menu with nothing in it.
    //
    // `selection.actionable` is the wider of the two: it is also set for a
    // selected form field, which is not in `SelectionState`. Without it here,
    // `canvas.field`'s items would resolve disabled and the menu would never
    // open at all — the state `offers_anything` is built to prevent, met from
    // the one direction it cannot see.
    //
    // It used to say *"both items"*, and that stopped being true on
    // 2026-08-29: `canvas.field`'s `format.delete` now carries
    // `selection.delete_permitted` as its `visible_when`, so on a document
    // whose form structure is frozen the menu offers `format.properties`
    // alone — one item, still enough for `offers_anything`, and the Delete is
    // ABSENT rather than greyed (R9).
    //
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
    // **Applied ONLY to `canvas.field`, and the narrowness is the whole
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
    // **The two node rows' four conditions**, corrected here and nowhere
    // else, for the same reason and with the same narrowness as the Delete
    // above: they are facts about ONE right-click on ONE edge, and
    // `PdfcerApp::conditions()` ran before that click existed.
    //
    // Asked from the **parked** pick rather than from the live pointer.
    // `attach` runs on every frame the popup is drawn and the pointer is on the
    // menu by the second of them; recomputing would grey the row the operator's
    // hand was travelling toward. `annotnodes::menu`'s header carries the
    // argument; this is the call site it is about.
    //
    // The engine preflight behind [`rows`] costs one annotation walk per row,
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
    // **`format.select_text_line`'s one condition** — O188(A), and the
    // narrowness is the same argument the Delete above makes: it is a fact
    // about ONE right-click on ONE line, and `PdfcerApp::conditions()` ran
    // before that click existed. Nothing publishes this name anywhere else, so
    // outside this `matches!` it is simply absent, which `ConditionSet` reads
    // as false and the item's `shown_when` reads as *no row*.
    //
    // Read from the **parked** pick rather than recomputed, for the reason
    // stated where it is parked: on every frame after the click the pointer is
    // on the menu, and a recomputed answer would delete the row out from under
    // the hand travelling toward it.
    //
    // ONE name, where the node pair needs four. There is no greyed state
    // here — see `shell::menus::RUN_SELECT_OFFERED` — so *shown* and *enabled*
    // are one question, and the command carries the same name in its
    // `enabled_when` so a route that never consults an item cannot press a row
    // whose operand has evaporated.
    if matches!(chosen, CanvasMenu::Object) {
        overrides.push((
            menus::RUN_SELECT_OFFERED,
            crate::canvas::runmenu::parked(&ctx).offered(),
        ));
        // The frame's conditions predate a right-click that moved the
        // selection, so the merge row is re-asked here.
        let merge = crate::canvas::runmerge::operand(targets, selection, page);
        overrides.extend([
            (menus::TEXT_MERGE_OFFERED, merge.is_some()),
            (
                menus::TEXT_MERGE_ALLOWED,
                merge.as_ref().is_some_and(|m| m.allowed()),
            ),
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

    /// **A right-click over an unselected object selects it first.**
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

    /// **…and a right-click over an object that is already selected
    /// changes nothing.**
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

    /// **A right-click on blank page never clears the selection.**
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

    /// **Right-clicking the object you are inside keeps you inside it.**
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
                chunk: false,
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
                chunk: false,
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
    #[test]
    fn each_canvas_menu_names_a_context_the_shell_defines() {
        assert_eq!(CanvasMenu::Object.context_id(), CANVAS_OBJECT);
        // The third, added with paragraph reflow. Its menu is the only route
        // to that command that does not go through the ribbon, so a context id
        // that drifted from `shell::menus` would silently take the canvas route
        // away and leave the ribbon working — a half-loss no other test sees.
        assert_eq!(
            CanvasMenu::Text.context_id(),
            crate::shell::menus::CANVAS_TEXT
        );
        // The fourth, added with the form-field menu. Same argument as the
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

        // The fifth, added with the markup menu. Same argument as the third
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
            oriented: None,
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
