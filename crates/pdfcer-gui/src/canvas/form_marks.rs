//! # `canvas::forms::marks` — the two marks the canvas draws over fillable fields
//!
//! Split out of [`super`] on 2026-09-02 under R2, when O98's trace carried that
//! file past the 1,500-line ceiling. The seam is a real one rather than a cut at
//! a convenient line number: everything else in `canvas::forms` is about
//! **interaction** — which box the pointer is over, which one has focus, what a
//! keystroke does to it, what a click commits. These two functions draw and
//! decide nothing.
//!
//! ## The two marks, and they answer different questions
//!
//! | | question | when |
//! |---|---|---|
//! | [`shade`] | *"which of these boxes can I type in?"* | O96 — always, while the option is on |
//! | [`spotlight`] | *"which one am I typing in right now?"* | O98 — while a panel row has focus |
//!
//! They are deliberately different weights. Every fillable field already wears
//! the wash, so a spotlight drawn as a second wash would be invisible on top of
//! the first — hence an outline, argued at
//! [`crate::canvas::overlay::draw_field_spotlight`].
//!
//! ## ★★ Rule 4, for both, and the answer is the same
//!
//! Neither is a mark on **content**. They are the cursor: transient, following
//! the operator's attention, gone when the option is turned off or the row
//! loses focus. That is the class rule 4's fourth clause permits by name —
//! *"a snap indicator, a hover highlight, a rubber-band, a selection handle —
//! these are the cursor"*.
//!
//! ⇒ The one-line test — *would a screenshot of the canvas differ from one of
//! the same document saved and reopened?* — answers **yes**, and correctly, for
//! the same reason a text caret does. What rule 4 forbids is styling *applied
//! content* as provisional, and a field's own appearance stream is drawn
//! identically either way.

use crate::app::state::OpenDoc;
use crate::canvas::forms::boxes::WidgetBox;
use crate::canvas::strip::PageView;

/// The trace slot for the panel→canvas spotlight — `OPERATOR_REQUESTS.md` O98.
///
/// Its own slot rather than folded into [`BOXES_SLOT`], because the two change
/// at completely different rates: the box census changes when the page or the
/// document does, and the spotlight changes every time the operator moves
/// between rows. Sharing one `trace_changed` slot would make each suppress the
/// other's line.
// ui-text-exempt: trace slot name, never displayed
const SPOTLIGHT_SLOT: &str = "canvas-form-spotlight";

/// The trace slot for the fillable-field wash — `OPERATOR_REQUESTS.md` O96.
///
/// Its own slot, for [`SPOTLIGHT_SLOT`]'s reason: the wash changes when the
/// document or the page set does, and the spotlight changes every time the
/// operator moves between rows. One shared `trace_changed` slot would make each
/// suppress the other's line.
// ui-text-exempt: trace slot name, never displayed
const SHADE_SLOT: &str = "canvas-form-shade";

/// **Outline the field the Forms panel is pointing at** — O98.
///
/// ★★ **Every widget of that field**, not one. A field may be painted in
/// several places — a header repeated on each page, a radio group — and
/// spotlighting one of them would answer *"where is this field"* with a half
/// truth. `WidgetBox::field` is the fully-qualified name, so the filter is the
/// same identity the panel wrote.
///
/// ★ Draws nothing when the panel is not pointing at anything, which includes
/// every frame the panel is not on screen: it clears the channel before its rows
/// draw, so an unhidden panel with nothing focused leaves it empty.
pub(super) fn spotlight(ui: &egui::Ui, pages: &[PageView], list: &[WidgetBox]) {
    let Some(spot) = crate::panels::forms::spotlight::get(ui.ctx()) else {
        // ★★ Traced, and NOT as a silence. "The panel is pointing at nothing"
        // and "the panel is pointing at a field this canvas cannot find" are
        // different states with the same appearance — no outline — and a check
        // that could not tell them apart would report a working build broken
        // whenever the fixture's field happened to be on another page.
        crate::diag::trace_changed(SPOTLIGHT_SLOT, || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-form-spotlight field=none drawn=0".to_owned()
        });
        return;
    };
    let painter = ui.painter().clone();
    let visuals = ui.visuals();
    let mut drawn = 0usize;
    for view in pages {
        for widget_box in list
            .iter()
            .filter(|b| b.page == view.page && b.field == spot.field)
        {
            crate::canvas::overlay::draw_field_spotlight(
                &painter,
                visuals,
                &view.map,
                widget_box.rect,
            );
            drawn += 1;
        }
    }
    crate::diag::trace_changed(SPOTLIGHT_SLOT, || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        //
        // ★★★ `drawn=` beside the name, because the two failures this feature
        // can have are opposite and produce the same picture:
        //
        //   * `drawn=0` with a name — the channel carried a field the canvas
        //     could not place. Either the panel and the canvas disagree about
        //     the fully-qualified name (the identity bug this channel was
        //     deliberately built on names rather than indices to avoid), or the
        //     field is simply on a page that is not on screen, which is not a
        //     defect at all.
        //   * `drawn>1` — a multi-widget field, which is CORRECT and is the
        //     case `spotlight`'s own doc comment argues about: outlining one
        //     placement of a field that appears three times would answer
        //     "where is this field" with a half truth.
        //
        // A bare "the spotlight ran" line could distinguish neither.
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-form-spotlight field={} drawn={drawn} candidates={}",
            spot.field,
            list.iter().filter(|b| b.field == spot.field).count()
        )
    });
}

/// **Wash every fillable field on screen** — `OPERATOR_REQUESTS.md` O96.
///
/// The whole of the feature's drawing. `crate::canvas::overlay::draw_field_shade`
/// owns the colour and the alpha and argues both; this owns *which* boxes and
/// *whether at all*.
///
/// ★ One painter per page view rather than one for the lot, because a box's rect
/// is in its own page's space and `PageMapping` is per page — the same reason
/// [`cursor`] walks the views rather than the boxes.
pub(super) fn shade(ui: &egui::Ui, doc: &OpenDoc, pages: &[PageView], list: &[WidgetBox]) {
    // ★ `doc.prefs`, which is the snapshot taken when the document opened —
    // the same field `canvas::paging` reads for the wheel gesture. A live read
    // would be wrong for the reason `OpenDoc::prefs` states: this is drawn per
    // frame and a preference that changed mid-frame would flicker.
    if !doc.prefs.shade_form_fields {
        // ★★ Traced as a STATE, not as a silence. "The operator turned the wash
        // off" and "the wash is on and found nothing to paint" both draw
        // nothing, and a check that could not tell them apart would report a
        // working build broken on a document with no form — or, far worse,
        // report a build with the feature dead as working because the fixture
        // had no fields either.
        crate::diag::trace_changed(SHADE_SLOT, || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-form-shade on=0 drawn=0 boxes=0".to_owned()
        });
        return;
    }
    let painter = ui.painter().clone();
    let visuals = ui.visuals();
    let mut drawn = 0usize;
    for view in pages {
        for widget_box in list.iter().filter(|b| b.page == view.page) {
            crate::canvas::overlay::draw_field_shade(&painter, visuals, &view.map, widget_box.rect);
            drawn += 1;
        }
    }
    crate::diag::trace_changed(SHADE_SLOT, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            //
            // ★ `boxes=` is the whole census and `drawn=` is what this frame
            // painted, and they differ legitimately: a box on a page that is
            // scrolled out of view is in the census and is not drawn. Carrying
            // both is what lets a check say "the wash is on, the document has
            // fields, and none of them were painted" — which is the one shape
            // that is actually a defect.
            "canvas-form-shade on=1 drawn={drawn} boxes={}",
            list.len()
        )
    });
}
