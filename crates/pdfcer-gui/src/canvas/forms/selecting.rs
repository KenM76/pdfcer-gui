//! **Selecting a form field, rather than filling it** — the Edit-mode half of
//! [`super`].
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/forms/selecting.md`.

use super::*;

/// The environment variable that selects a form field when no pointer can.
///
/// Takes one fully-qualified field name. Consumed on the first frame the field
/// appears in the placement census, and never again.
///
/// # Why it exists
///
/// `doc.selected_field` is the Properties pane's only input, and until this
/// seam the only writer was [`select_click`] — a pointer gesture. So every
/// properties surface for a form field was reachable by R1 only on a machine
/// whose desktop was free, and `tools/ui-verify` moves the operator's real
/// mouse. The result is that the largest editing surface in the shell could not
/// be driven on any day he was working, which is most days.
///
/// It is the same wall `DIAG_OPEN_PATH` and
/// [`DIAG_TYPE`](crate::canvas::textedit::DIAG_TYPE) are behind, arriving from
/// the other side: not *"synthetic input cannot reach this dialog"* but
/// *"synthetic input cannot be produced at all right now"*.
///
/// # What it deliberately cannot do
///
/// It resolves the name against **the same census [`select_click`] hit-tests**,
/// `boxes::Placed::targets`, so its reach is the click's reach exactly. That
/// census admits a widget on the strength of its rectangle alone — every widget
/// with a place on the canvas is in it, including the ones `boxes::classify`
/// refuses as unfillable — and excludes one only when it has no place at all:
/// no entry in the placement, or a page transform that will not invert. Such a
/// widget is unreachable through this seam for the same reason it is
/// unreachable by pointing at it. A seam that resolved names against the
/// AcroForm dictionary instead would let a check pass on a field no operator
/// can select, which is the defect class R1 exists to catch rather than a
/// convenience worth having.
///
/// And it raises the same [`FieldAction::Select`] the click raises, through the
/// same queue, so what it substitutes is the gesture and not the selection.
// ui-text-exempt: an environment variable name, never displayed
pub const DIAG_SELECT_FIELD: &str = "PDFCER_DIAG_SELECT_FIELD";

/// Select the field [`DIAG_SELECT_FIELD`] names, once.
pub(super) fn seeded_select(
    doc: &OpenDoc,
    targets: &[boxes::FieldTarget],
    actions: &mut Vec<Action>,
) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static DONE: AtomicBool = AtomicBool::new(false);
    if !crate::diag::enabled() || DONE.load(Ordering::Relaxed) {
        return;
    }
    let Ok(name) = std::env::var(DIAG_SELECT_FIELD) else {
        return;
    };
    let name = name.trim().to_owned();
    if name.is_empty() {
        return;
    }
    // An empty census is a document that has not finished arriving, not a
    // document without the field. Returning without reporting anything is
    // right here and only here: the next frame asks again.
    if targets.is_empty() {
        return;
    }
    let found = targets.iter().find(|t| t.field == name);
    // A miss is disclosed rather than left as silence, for `report_clipped`'s
    // reason: a check that sees no selection cannot otherwise tell a misspelt
    // name from a selection mechanism that is broken, and those have opposite
    // fixes.
    //
    // ui-text-exempt: diagnostic trace, never displayed in the UI
    crate::diag::trace_on_change("form-field-seam", || match found {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        Some(t) => format!("name={name} found=true page={} widget={}", t.page, t.widget),
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        None => format!("name={name} found=false candidates={}", targets.len()),
    });
    let Some(target) = found else {
        return;
    };
    DONE.store(true, Ordering::Relaxed);
    let picked = crate::app::state::SelectedField {
        field: target.field.clone(),
        widget: target.widget,
        page: target.page,
    };
    if doc.selected_field.as_ref() == Some(&picked) {
        return;
    }
    actions.push(FieldAction::Select(Some(picked)).into());
}

/// A click in **Edit mode**: select the field under the pointer, or clear the
/// selection.
pub(super) fn select_click(
    ctx: &egui::Context,
    doc: &OpenDoc,
    pages: &[PageView],
    drawn: &[DrawnPage],
    targets: &[boxes::FieldTarget],
    actions: &mut Vec<Action>,
) {
    let Some(pos) = ctx.pointer_interact_pos() else {
        return;
    };
    // **A right-click selects too, and the two buttons are NOT the same
    // rule.** `OPERATOR_REQUESTS.md` O53's ruling — anything the engine can do
    // to an object must be reachable by clicking that object — reaches the
    // context menu, and a menu about a field the operator did not point at is
    // the `canvas.object` select-first defect in another costume: point at
    // field B while field A is selected, choose Delete, and A is gone.
    //
    // ⇒ The difference is **what happens over PAPER**:
    //
    // | | primary | secondary |
    // |---|---|---|
    // | over a field | select it | select it |
    // | over the selected field | no change | no change |
    // | over blank paper | **clear** | **change nothing** |
    //
    // The last row is `canvas::menus`' rule 3 and its reason carries here
    // unchanged: a left click on paper is an unambiguous *"deselect"*, a
    // right-click is the opening of a question. An operator who right-clicks
    // slightly wide of the field they meant, sees the wrong menu and presses
    // Escape should still have their field.
    let primary = drawn
        .iter()
        .find(|d| d.response.clicked_by(egui::PointerButton::Primary))
        .map(|d| d.page);
    let secondary = drawn
        .iter()
        .find(|d| d.response.clicked_by(egui::PointerButton::Secondary))
        .map(|d| d.page);
    let Some(page) = primary.or(secondary) else {
        return;
    };
    let clearing = primary.is_some();
    let Some(map) = pages.iter().find(|v| v.page == page).map(|v| v.map) else {
        return;
    };

    let point = map.to_page(pos);
    let picked =
        boxes::hit_target(targets, page, point).map(|t| crate::app::state::SelectedField {
            field: t.field.clone(),
            widget: t.widget,
            page: t.page,
        });

    // Raised only on a CHANGE. A click that re-selects what is already
    // selected, or that clears an empty selection, is not an event — and this
    // surface is asked on every frame the pointer is down, so raising
    // unconditionally would put an action on the queue sixty times a second
    // and bump the epoch with it.
    if picked == doc.selected_field {
        return;
    }
    // The one asymmetry between the buttons, and it is the table above's
    // last row. A secondary click that hit nothing leaves the selection alone;
    // a primary one clears it. Placed after the no-change guard so an
    // unchanged selection still costs nothing either way.
    if picked.is_none() && !clearing {
        return;
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        match &picked {
            Some(f) => format!(
                "form-field-selected page={} field={} widget={}",
                f.page, f.field, f.widget
            ),
            None => "form-field-selected none".to_owned(),
        }
    });
    actions.push(FieldAction::Select(picked).into());
}

/// **Is a right-click at `point` about a form field?**
#[must_use]
pub fn right_click_hits_a_field(
    ctx: &egui::Context,
    doc: &OpenDoc,
    caps: &crate::app::modes::Capabilities,
    page: usize,
    point: egui::Pos2,
) -> bool {
    if !caps.edit_content || !doc.annotations_visible() {
        return false;
    }
    // `placed` is memoised on `(path, edit_epoch)`, so this is a map lookup on
    // every frame after the first of an epoch — the same call `widgetdrag`
    // makes for the same reason.
    let placed = placed(ctx, doc);
    boxes::hit_target(&placed.targets, page, point).is_some()
}

/// The pointer over a selectable widget in Edit mode.
pub(super) fn select_cursor(
    ctx: &egui::Context,
    pages: &[PageView],
    targets: &[boxes::FieldTarget],
) {
    let Some(pos) = ctx.pointer_latest_pos() else {
        return;
    };
    for view in pages {
        if boxes::hit_target(targets, view.page, view.map.to_page(pos)).is_some() {
            ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
            return;
        }
    }
}

/// **Paint the selected form field: its outline and its eight grips.**
pub(super) fn selection_overlay(
    ctx: &egui::Context,
    visuals: &egui::Visuals,
    doc: &OpenDoc,
    pages: &[PageView],
    targets: &[boxes::FieldTarget],
) {
    let Some(selected) = doc.selected_field.as_ref() else {
        return;
    };
    let Some(target) = targets.iter().find(|t| {
        t.page == selected.page && t.field == selected.field && t.widget == selected.widget
    }) else {
        return;
    };
    let Some(view) = pages.iter().find(|v| v.page == target.page) else {
        return;
    };
    let screen = view.map.rect_to_screen(target.rect);
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("form-field-selection"), // ui-text-exempt: a layer id.
    ));
    // The LIVE theme, read from the caller's own `Context`, never a colour
    // resolved somewhere else and carried here: `Theme::of` returns the
    // operator's current preset, so a painter that guessed would draw a
    // selection outline in the wrong colour on exactly the build where
    // somebody had changed it. (`visuals` is still the caller's, and
    // `draw_grips` below still needs it for `window_fill`.)
    //
    // Never `visuals.selection.stroke.color`, however identical the value
    // looks. That is `egui`'s SELECTED-WIDGET channel, not a canvas role:
    // pointing the canvas at it makes every selected chrome control in the
    // application share this outline's colour, so a theme that wanted one
    // changed cannot change it without the other.
    // `tools/gates/check-selection-channel.sh` keeps that address out.
    let stroke = egui::Stroke::new(1.5, egui_shell::theme::Theme::canvas_selection_ink(ctx));
    painter.rect_stroke(
        screen,
        egui::CornerRadius::ZERO,
        stroke,
        egui::StrokeKind::Middle,
    );
    // Published under the SAME region name every other selection outline
    // uses, so a driven check aiming at a grip reads one name whatever is
    // selected. `handles::grip_rects` derives all eight from this box.
    crate::diag::ui_rect(crate::canvas::overlay::SELECTION_OUTLINE_REGION, screen);
    // `scale_only()`, spelled here as the same value `pressing::grabbable`
    // hands the hit test for this selection — H7, and the field is the one
    // selection where the two flags differ in the direction that would be
    // easiest to get wrong by inheritance.
    //
    // **A widget scales and does not turn**, and the asymmetry is
    // §12.5.6.19 Table 189's rather than a gap in pdfcer: a widget's rotation is
    // `/MK /R`, a quantised 0/90/180/270 *declaration* the field's appearance
    // generator reads, not a free-angle transform. `rotate_annotation` refuses
    // a widget by name and points at a verb that is not built.
    //
    // ⇒ So no ninth handle is painted here and none is hit-tested. **R9**:
    // rendering nothing is the honest answer for a capability that does not
    // exist — a circle on a stem that declined on release would be the
    // "visible control, silently inert" defect wearing the costume of a fix.
    crate::canvas::overlay::draw_grips(
        &painter,
        visuals,
        screen,
        crate::canvas::handles::GripSet::scale_only(),
    );
}
