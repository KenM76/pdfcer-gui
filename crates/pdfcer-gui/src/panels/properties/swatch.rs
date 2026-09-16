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

/// What one `/MK` colour key says, as far as a swatch is concerned.
///
/// ★★★ Deliberately **not** [`Value`], and the difference is the subject.
/// [`Value`] models a *selection of document objects*, which has exactly two
/// states — they agree or they do not. One widget's `/MK` `/BG` is one key on
/// one dictionary, and Table 189 gives it four: absent, the empty array that
/// states *no colour*, a DeviceGray or DeviceRGB value, and a DeviceCMYK
/// separation. Two of those four are colours no swatch can draw, and one of
/// them is not a colour at all.
///
/// So this enum carries the one state a swatch CAN draw and defers the rest to
/// the caller, exactly as the module header requires: *"the sentence that
/// stands in its place belongs to whoever knows what the ink is."* Which of the
/// three unshowable states a widget is in is a fact about `/MK`, and `/MK` is
/// not a thing this file knows about.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum MkValue<'a> {
    /// A colour the swatch draws exactly.
    Shown([u8; 3]),
    /// No colour this control can draw. `mark` is what the button reads
    /// instead — an em dash for *the file says nothing*, a word for *no
    /// colour*, four numbers for a separation — and `note` is the sentence the
    /// popup shows above the picker, or empty for none.
    Unshowable {
        /// The button's face.
        mark: &'a str,
        /// The popup's opening sentence. Empty draws no label at all.
        note: &'a str,
    },
}

/// What the operator chose. Returned **once**, on the frame the gesture ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MkPick {
    /// A colour, from the picker, on the frame it closed.
    Colour([u8; 3]),
    /// Table 189's empty array — *this widget has no such colour*, stated
    /// positively, which is not the same as the key being absent.
    NoColour,
}

/// One `/MK` colour control's inputs, bundled because there are four and the
/// two strings would otherwise be positional.
pub(super) struct MkControl<'a> {
    /// What the widget's key currently says.
    pub value: MkValue<'a>,
    /// Table 189's *no colour* entry, or `None` when this key has no such
    /// state worth offering.
    ///
    /// ★★ `None` is the border colour's case and it is R9 rather than an
    /// omission: `WidgetChrome::stroke` resolves an empty `/BC` and an absent
    /// `/BC` to the same black, so an entry writing the empty array would
    /// change a byte, rebuild an appearance stream, cost an undo entry and
    /// alter no pixel. A capability that does nothing renders nothing.
    pub no_colour: Option<MkNoColour<'a>>,
    /// Draw the swatch as a disc rather than a bar.
    ///
    /// ★ A radio button's background is filled as a **circle** by the engine's
    /// own builder, which says so in as many words. A rectangular preview over
    /// a control that will come out round is this panel mis-stating the result
    /// of the operator's own press.
    pub disc: bool,
}

/// The *no colour* entry, when a key has one.
pub(super) struct MkNoColour<'a> {
    /// The entry's label.
    pub label: &'a str,
    /// Whether pressing it would change anything. `false` when the widget
    /// already states *no colour*: the press would author an edit that changes
    /// no byte and still rebuild the appearance, which is an undo entry for
    /// nothing.
    pub available: bool,
    /// R9 requires a greyed control to explain itself on hover.
    pub unavailable_hover: &'a str,
}

/// One frame of a `/MK` colour control.
///
/// Returns `Some` **only** on the frame a choice was made: the picker closed
/// after the operator moved it, or the *no colour* entry was pressed. The
/// commit rule is [`show`]'s, for [`show`]'s reason — the module header's
/// sixty-content-stream-rewrites-a-second defect — and the two functions share
/// [`Editing`] so there is one implementation of it rather than two that drift.
///
/// ★ The *no colour* entry commits **immediately** rather than on close, and
/// that is not an inconsistency: it is a discrete press, not a drag, so there
/// is no run of intermediate values for a close-edge to collapse. It closes the
/// popup itself, because `PopupCloseBehavior::CloseOnClickOutside` would
/// otherwise leave a picker open over a widget that no longer has a colour.
pub(super) fn show_mk(
    ui: &mut Ui,
    id_salt: &str,
    control: &MkControl<'_>,
    region: &str,
) -> Option<MkPick> {
    let id = ui.make_persistent_id(id_salt);
    let popup_id = id.with("popup"); // ui-text-exempt: an egui id salt, never displayed
    let seed = match control.value {
        MkValue::Shown(rgb) => rgb,
        // ★ Black, never a conversion of the CMYK the widget may be carrying.
        // Converting would put a number in the picker that the file does not
        // contain, and the operator's first nudge would commit pdfcer's guess
        // at their separation as though it were their own.
        MkValue::Unshowable { .. } => NO_PARTICULAR_COLOUR,
    };

    let mut state: Editing = ui.data(|d| d.get_temp(id)).unwrap_or(Editing {
        working: seed,
        dirty: false,
        was_open: false,
    });

    let height = ui.spacing().interact_size.y;
    let size = egui::vec2(height * ASPECT, height);
    let response = match control.value {
        MkValue::Shown(rgb) => {
            // DOCUMENT COLOUR: the widget's own `/MK` value, read from the
            // file. A theme must never move it — restyling the application
            // would change what the operator sees their form's ink as.
            let fill = egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
            if control.disc {
                disc(ui, size, fill)
            } else {
                ui.add(egui::Button::new("").fill(fill).min_size(size))
            }
        }
        // The button's ordinary chrome fill, deliberately: there is no document
        // colour to show, and tinting it would be this panel proposing one.
        MkValue::Unshowable { mark, .. } => {
            ui.add(egui::Button::new(egui::RichText::new(mark).small()).min_size(size))
        }
    };
    crate::diag::ui_rect_visible(region, response.rect, ui.clip_rect());

    // A fresh open re-seeds, for [`show`]'s reason: a picker opened over one
    // widget, closed, and re-opened over another would otherwise show the first
    // one's colour as though it were the second's.
    let open_before = egui::Popup::is_id_open(ui.ctx(), popup_id);
    if open_before && !state.was_open {
        state.working = seed;
        state.dirty = false;
    }

    let mut cleared = false;
    egui::Popup::menu(&response)
        .id(popup_id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            if let MkValue::Unshowable { note, .. } = control.value
                && !note.is_empty()
            {
                ui.label(egui::RichText::new(note).small().weak());
            }
            if let Some(entry) = &control.no_colour {
                let none = ui.add_enabled(entry.available, egui::Button::new(entry.label));
                // `ui_rect`, not the gated form, for the reason the picker's
                // own rect below carries: a popup lives in its own `Area` on
                // the tooltip layer, where the clip rect is the whole screen
                // and a visibility fraction asserts nothing.
                crate::diag::ui_rect(&format!("{region}.none"), none.rect);
                if entry.available {
                    if none.clicked() {
                        cleared = true;
                        // The working colour is abandoned rather than
                        // committed: the operator's last act was to ask for no
                        // colour at all.
                        state.dirty = false;
                        ui.close();
                    }
                } else {
                    none.on_disabled_hover_text(entry.unavailable_hover);
                }
                ui.separator();
            }
            // DOCUMENT COLOUR: the working value of a widget's `/MK` colour
            // being edited. Not chrome.
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
            crate::diag::ui_rect(&format!("{region}.picker"), ui.min_rect());
        });

    let open_after = egui::Popup::is_id_open(ui.ctx(), popup_id);
    let committed = if cleared {
        Some(MkPick::NoColour)
    } else if state.was_open && !open_after && state.dirty {
        Some(MkPick::Colour(state.working))
    } else {
        None
    };
    if committed.is_some() {
        state.dirty = false;
    }
    state.was_open = open_after;
    ui.data_mut(|d| d.insert_temp(id, state));
    committed
}

/// A round swatch, for a control whose background the engine fills as a circle.
///
/// ★ Hand-drawn rather than a `Button` with a rounded corner radius, because a
/// rounded rectangle at this size reads as a button with a tint and the point
/// of the shape is that the operator sees a **disc** — the thing a radio
/// button's background will actually be. It keeps the button's own interaction
/// (a click opens the popup) by allocating an interactive rect of the same size
/// and painting into it.
fn disc(ui: &mut Ui, size: egui::Vec2, fill: egui::Color32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        // The plate the disc sits on is CHROME — it is the control's own
        // surface, not the document's ink — so it comes from the style the way
        // every other button's does.
        ui.painter()
            .rect_filled(rect, visuals.corner_radius, visuals.weak_bg_fill);
        let radius = rect.height().mul_add(0.5, -2.0).max(1.0);
        ui.painter().circle_filled(rect.center(), radius, fill);
        ui.painter()
            .circle_stroke(rect.center(), radius, visuals.bg_stroke);
    }
    response
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
