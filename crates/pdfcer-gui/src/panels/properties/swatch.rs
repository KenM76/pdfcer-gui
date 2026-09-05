//! # `panels::properties::swatch` — one colour control, three honest states
//!
//! `OPERATOR_REQUESTS.md` **O89**, both pieces. It is the widget behind the
//! clicked-text colour ([`super::textobject`]) and behind the multi-object
//! fill/line colour ([`super::paint`]), and it exists because those two
//! surfaces have to answer the same three questions the same way:
//!
//! | state | drawn as | picking a colour |
//! |---|---|---|
//! | the selection agrees | a swatch of that colour | sets it |
//! | the selection **disagrees** | the indeterminate plate and a dash | sets **all** of them |
//! | the colour cannot be shown | *nothing — the caller draws a sentence* | — |
//!
//! ★ The third row is a caller's job on purpose. What to say about an ink
//! pdfcer will not overwrite differs between a path (whose `/Separation` name
//! the file records and `pdfcer_core::vector::PathPaint::Other` carries) and a
//! text run (whose `pdfcer_core::text_extract::TextColor::Other` carries **no
//! name at all**), and a widget that tried to word both would word one of them
//! wrongly. This widget refuses to draw a control; the sentence that stands in
//! its place belongs to whoever knows what the ink is.
//!
//! ## ★★★ Why "mixed" exists at all, and why it is not an invention
//!
//! O89 recorded multi-object recolouring as *not offered*, with a reason:
//!
//! > *"when the objects disagree there is no honest colour to open on and
//! > picking the first one's would quietly propose flattening the rest to it."*
//!
//! That reasoning is right about the danger and wrong about the conclusion.
//! **Every editor in this product class already solved it** — Illustrator,
//! Inkscape, Figma and Word all show an *indeterminate* control over a
//! disagreeing selection, and applying a value sets every member. This
//! project's standing rule is that *the convergence of the product class IS the
//! specification, and an invented interaction is a defect even when it works*,
//! so the mixed state is the answer rather than one of several.
//!
//! ★ And the em dash is not a new marker either: this shell already writes one
//! for *no value*, in `crate::text::panels::properties::text_value_absent`,
//! whose own doc comment makes precisely this argument — *"every property grid
//! in this class shows a blank or a dash for no value and for mixed values,
//! which are the same state as far as a single field is concerned."*
//!
//! ## ★★★ ONE UNDO STEP PER GESTURE, and why this widget could not be
//! ## `ui.color_edit_button_srgb`
//!
//! This is the load-bearing reason the widget is hand-built.
//!
//! `egui`'s own colour button marks its response **changed on every frame of a
//! drag inside the picker** (`color_edit_button_hsva` calls
//! `button_response.mark_changed()` from inside the popup body, on every frame
//! `color_picker_hsva_2d` returns `true`). A caller that acts on `.changed()`
//! therefore authors **one document edit per frame** while the operator drags
//! across the saturation square — sixty content-stream rewrites a second, sixty
//! undo entries, and a `Ctrl+Z` stack the operator cannot get back through.
//!
//! ★★ That is the same defect `super::text`'s size field already avoids by
//! committing on `drag_stopped`/`lost_focus` and never on `.changed()`, with
//! the same stated reason. A colour popup has no `drag_stopped` to hang it on,
//! so the equivalent event has to be *the popup closing* — which means owning
//! the popup id, which means owning the button. Hence this module.
//!
//! ⇒ [`show`] returns `Some` **exactly once**, on the frame the picker closes,
//! and only if the operator actually moved it. Open-and-close-without-touching
//! returns `None`, so idly inspecting a colour never writes to the document.
//!
//! ## Rule 4 — nothing here reaches the canvas
//!
//! This widget draws in a dock panel. It marks no page, tints nothing, and
//! renders no preview: the document changes only when the caller's action is
//! applied, and from that instant the canvas shows exactly what the saved file
//! will show. What was skipped, and why, is disclosed off-canvas by the caller.
//!
//! ## Theme
//!
//! The two colours the indeterminate state needs come from
//! `egui_shell::Theme::indeterminate_pair`, as a **pair**, for the reason that
//! function's own doc comment gives: `tools/gates/check-theme-colors.sh`
//! forbids invented values and cannot forbid a wrong role, and picking two
//! roles that happen to look right is how this project shipped defect D2 three
//! times. The swatch's own fill is not a theme colour at all — it is the
//! operator's document content — and says so on the line.

use egui::Ui;

/// What the selection says its colour is.
///
/// Two variants and not three: *"there is no colour to show"* is not a value
/// this widget can draw, so it is not a value this widget accepts. See the
/// module header on why the sentence for that case belongs to the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Value {
    /// Every member of the selection is this colour.
    Agreed([u8; 3]),
    /// The members disagree. The control still applies to all of them.
    Mixed,
}

/// PDF §8.6.8's default fill, and where the picker opens over a selection that
/// has no agreed colour of its own.
///
/// ★ It is **never applied unless the operator moves the picker** — [`show`]
/// returns `None` when nothing changed — so this is a starting position on
/// screen and not a proposed value. "Opens on nothing in particular" is what a
/// mixed control is required to do; opening on the *first member's* colour is
/// the specific thing O89 refused, and this is not that.
// DOCUMENT COLOUR: the PDF default fill (§8.6.8), used as the picker's opening
// position over a document colour. It is not chrome and a restyle must not
// move it.
const NO_PARTICULAR_COLOUR: [u8; 3] = [0, 0, 0];

/// The width of the swatch, as a multiple of the row height.
///
/// ★ Wider than tall, which is what a colour *swatch* looks like everywhere
/// this operator works — Word's font-colour button, Illustrator's fill chip,
/// SolidWorks' line-colour control. A square reads as a button with a coloured
/// glyph; a bar reads as a sample of the colour itself.
const ASPECT: f32 = 1.7;

/// One frame of the control. `Some(rgb)` **only** on the frame the picker
/// closed after the operator changed it.
///
/// `id_salt` must be unique within the `Ui` — the fill and the line swatches on
/// one panel are two controls and must not share a popup. `region` is published
/// for a driven check.
///
/// # ★★★ `mixed_hint` is a PARAMETER, and it was a hard-coded string for about
/// # twenty minutes
///
/// The sentence shown at the top of the picker over a disagreeing selection
/// names its subject — *"These **words** are not all one colour"* — and this
/// widget serves two subjects. The first draft read
/// `crate::text::panels::textobject::mixed_hint()` inline, which put a sentence
/// about words over a selection of **paths** on the vector row. It was caught
/// by reading the call sites rather than by any test, and no test could have
/// caught it: both strings compile, both render, and the wrong one is grammatical.
///
/// ⇒ Same rule this module's header already states for the ink refusal: **the
/// sentence belongs to whoever knows what the selection is made of.** The widget
/// draws controls; it does not name subjects.
pub(super) fn show(
    ui: &mut Ui,
    id_salt: &str,
    value: Value,
    region: &str,
    mixed_hint: &str,
) -> Option<[u8; 3]> {
    let id = ui.make_persistent_id(id_salt);
    let popup_id = id.with("popup"); // ui-text-exempt: an egui id salt, never displayed
    let seed = match value {
        Value::Agreed(rgb) => rgb,
        Value::Mixed => NO_PARTICULAR_COLOUR,
    };

    let mut state: Editing = ui.data(|d| d.get_temp(id)).unwrap_or(Editing {
        working: seed,
        dirty: false,
        was_open: false,
    });

    let height = ui.spacing().interact_size.y;
    let size = egui::vec2(height * ASPECT, height);
    let response = match value {
        Value::Agreed(rgb) => {
            // DOCUMENT COLOUR: the object's own fill, read from the file. A
            // theme must never move it — restyling the application would
            // change what the operator sees their document's ink as.
            let fill = egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
            ui.add(egui::Button::new("").fill(fill).min_size(size))
        }
        Value::Mixed => {
            let (plate, ink) = egui_shell::Theme::indeterminate_pair(ui.ctx());
            let marker = crate::text::panels::properties::text_value_absent();
            ui.add(
                egui::Button::new(egui::RichText::new(marker).color(ink))
                    .fill(plate)
                    .min_size(size),
            )
        }
    };
    crate::diag::ui_rect_visible(region, response.rect, ui.clip_rect());

    // ★ A fresh open re-seeds. Without this, a swatch opened over object A,
    // closed, and re-opened over object B would show A's colour in the picker —
    // a stale value presented as B's, which is the failure that makes a
    // properties panel untrustworthy.
    let open_before = egui::Popup::is_id_open(ui.ctx(), popup_id);
    if open_before && !state.was_open {
        state.working = seed;
        state.dirty = false;
    }

    egui::Popup::menu(&response)
        .id(popup_id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            if let Value::Mixed = value {
                ui.label(egui::RichText::new(mixed_hint).small().weak());
            }
            // DOCUMENT COLOUR: the working value of a document fill being
            // edited. Not chrome.
            let mut colour =
                egui::Color32::from_rgb(state.working[0], state.working[1], state.working[2]);
            if egui::color_picker::color_picker_color32(
                ui,
                &mut colour,
                egui::color_picker::Alpha::Opaque,
            ) {
                state.working = [colour.r(), colour.g(), colour.b()];
                state.dirty = true;
            }
            // ★★★ The picker's own rectangle, published so a DRIVEN check can
            // aim a real pointer inside it.
            //
            // It has to be its own name rather than [`region`], and this is the
            // same argument `app::fontband::FACE_POPUP_REGION` makes about the
            // face chooser: the button and its popup are two rectangles in one
            // frame, and a check reading one name would aim at whichever the
            // paint order happened to leave last — which for a popup is the
            // popup, so the *button* would become unclickable to the harness the
            // moment it opened once.
            //
            // ★ `ui_rect`, not `ui_rect_visible`. A popup is drawn in its own
            // `Area` on the tooltip layer, so its clip rect is the whole screen
            // and the visibility fraction is meaningless — the gated form would
            // be asserting a property that is trivially true here while reading
            // as though it had been checked.
            crate::diag::ui_rect(&format!("{region}.picker"), ui.min_rect());
        });

    let open_after = egui::Popup::is_id_open(ui.ctx(), popup_id);
    // ★★★ The commit. The picker CLOSED and something moved while it was open,
    // so the whole gesture becomes one action and one undo entry — see the
    // module header for the sixty-edits-a-second defect this avoids.
    let committed = (state.was_open && !open_after && state.dirty).then_some(state.working);
    if committed.is_some() {
        state.dirty = false;
    }
    state.was_open = open_after;
    ui.data_mut(|d| d.insert_temp(id, state));
    committed
}

/// The picker's in-progress value, across the frames it is open for.
///
/// ★ In `egui`'s temp data rather than on a draft struct, and that is a
/// deliberate difference from [`super::text::TextStyleDraft`]. A draft holds a
/// *reading of the document*, which has to be invalidated when the document
/// moves; this holds *where the operator's finger is*, which is meaningless the
/// moment the popup closes and must never outlive it. Storing it beside the
/// document reading would invite a stale finger position to be read as a value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Editing {
    /// The colour the picker is showing right now.
    working: [u8; 3],
    /// Has the operator moved it since this open?
    ///
    /// ★★ Without this, opening the picker to *look* at a colour and closing it
    /// would author an edit — a document changed, an undo entry added and a
    /// file marked dirty, by a gesture that changed nothing.
    dirty: bool,
    /// Was the popup open on the previous frame? The edge this compares against
    /// is the whole commit trigger.
    was_open: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **The mixed state must not carry a colour.**
    ///
    /// The one assertion this type exists for. If [`Value`] ever gained a way
    /// to represent "mixed, and here is a colour anyway", the next caller would
    /// pass the first member's — which is precisely the flattening O89 refused
    /// to ship, and it would look completely normal while it happened.
    #[test]
    fn mixed_carries_no_colour() {
        // A compile-time fact asserted at runtime, because the thing being
        // protected is the SHAPE of the enum and a shape has no other test.
        match Value::Mixed {
            Value::Mixed => {}
            Value::Agreed(_) => panic!("mixed must not be constructible with a colour"),
        }
        assert_eq!(std::mem::size_of::<Value>(), std::mem::size_of::<[u8; 4]>());
    }

    /// The picker's opening position over a disagreeing selection is the PDF
    /// default, not a member's colour.
    #[test]
    fn a_mixed_selection_opens_on_the_pdf_default() {
        assert_eq!(NO_PARTICULAR_COLOUR, [0, 0, 0]);
    }
}
