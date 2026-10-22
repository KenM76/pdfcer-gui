//! # `swatch` — a form box's `/MK` colour control
//!
//! A swatch with a popup picker for one `/MK` colour key (background or
//! border), offering *No colour* and *Remove* beside the colour, and a key the
//! picker cannot show disclosed rather than guessed. It answers `Some` only on
//! the frame the picker closes after a change, so one gesture is one edit.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/swatch.md`.

use egui::Ui;

/// PDF §8.6.8's default fill, and where the picker opens over a key it cannot
/// show.
///
/// It is **never applied unless the operator moves the picker** — [`show_mk`]
/// returns `None` when nothing changed — so this is a starting position on
/// screen and not a proposed value.
// DOCUMENT COLOUR: the PDF default fill (§8.6.8), used as the picker's opening
// position over a document colour. It is not chrome and a restyle must not
// move it.
const NO_PARTICULAR_COLOUR: [u8; 3] = [0, 0, 0];

/// The width of the swatch, as a multiple of the row height.
const ASPECT: f32 = 1.7;

/// What one `/MK` colour key says, as far as a swatch is concerned.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MkValue<'a> {
    /// A colour the swatch draws, in the space the file states it in.
    Shown(crate::colourpick::Ink),
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MkPick {
    /// A colour, from the picker, on the frame it closed, in the space the
    /// picker was on.
    Colour(crate::colourpick::Ink),
    /// Table 189's empty array — *this widget has no such colour*, stated
    /// positively, which is not the same as the key being absent.
    NoColour,
    /// Take the key away, so the file states nothing about it and whatever
    /// builds the appearance falls back to its own default.
    ///
    /// Not a colour and not the absence of one. On a push button the two read
    /// differently on screen: [`Self::NoColour`] gives no plate, this gives
    /// the plate back.
    Remove,
}

/// One `/MK` colour control's inputs, bundled because there are four and the
/// two strings would otherwise be positional.
pub struct MkControl<'a> {
    /// What the widget's key currently says.
    pub value: MkValue<'a>,
    /// The entries above the picker, in the order they are drawn.
    ///
    /// A list rather than one named field per state, because the states a
    /// `/MK` key can be put into are a fact about `/MK` and this file does not
    /// know about `/MK`. An empty slice draws no entries and no separator.
    ///
    /// The border colour offers one entry where the background offers two,
    /// and that asymmetry is R9 rather than an omission:
    /// `WidgetChrome::stroke` resolves an empty `/BC` and an absent `/BC` to
    /// the same black, so an entry writing the empty array would change a
    /// byte, rebuild an appearance stream, cost an undo entry and alter no
    /// pixel. A capability that does nothing renders nothing.
    pub entries: &'a [MkEntry<'a>],
    /// Draw the swatch as a disc rather than a bar.
    ///
    /// A radio button's background is filled as a **circle** by the engine's
    /// own builder, which says so in as many words. A rectangular preview over
    /// a control that will come out round is this panel mis-stating the result
    /// of the operator's own press.
    pub disc: bool,
}

/// One entry above the picker — a discrete state the key can be put into
/// that is not a colour the picker can express.
pub struct MkEntry<'a> {
    /// What pressing it commits. Never [`MkPick::Colour`]: a colour comes from
    /// the picker, and an entry claiming to return one would give the same
    /// outcome two routes that commit on different edges.
    pub pick: MkPick,
    /// The entry's label.
    pub label: &'a str,
    /// Whether pressing it would change anything. `false` when the key is
    /// already in this state: the press would author an edit that changes no
    /// byte and still rebuild the appearance, which is an undo entry for
    /// nothing.
    pub available: bool,
    /// R9 requires a greyed control to explain itself on hover.
    pub unavailable_hover: &'a str,
    /// What this entry's rect is called in the trace, appended to the
    /// control's own region with a dot.
    pub trace: &'a str,
}

/// One frame of a `/MK` colour control.
pub fn show_mk(
    ui: &mut Ui,
    id_salt: &str,
    control: &MkControl<'_>,
    region: &str,
) -> Option<MkPick> {
    let id = ui.make_persistent_id(id_salt);
    let popup_id = id.with("popup"); // ui-text-exempt: an egui id salt, never displayed
    let seed = match control.value {
        MkValue::Shown(ink) => ink,
        MkValue::Unshowable { .. } => crate::colourpick::Ink::from_srgb(NO_PARTICULAR_COLOUR),
    };

    let mut state: MkEditing = ui.data(|d| d.get_temp(id)).unwrap_or(MkEditing {
        working: seed,
        dirty: false,
        was_open: false,
    });

    let height = ui.spacing().interact_size.y;
    let size = egui::vec2(height * ASPECT, height);
    let response = match control.value {
        MkValue::Shown(ink) => {
            // DOCUMENT COLOUR: the widget's own `/MK` value as pdfcer renders
            // it. A theme must never move it.
            let fill = ink.screen();
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

    // A fresh open re-seeds: a picker opened over one
    // widget, closed, and re-opened over another would otherwise show the first
    // one's colour as though it were the second's.
    let open_before = egui::Popup::is_id_open(ui.ctx(), popup_id);
    if open_before && !state.was_open {
        state.working = seed;
        state.dirty = false;
    }

    let mut chosen: Option<MkPick> = None;
    egui::Popup::menu(&response)
        .id(popup_id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            if let MkValue::Unshowable { note, .. } = control.value
                && !note.is_empty()
            {
                ui.label(egui::RichText::new(note).small().weak());
            }
            for entry in control.entries {
                let pressed = ui.add_enabled(entry.available, egui::Button::new(entry.label));
                // `ui_rect`, not the gated form, for the reason the picker's
                // own rect below carries: a popup lives in its own `Area` on
                // the tooltip layer, where the clip rect is the whole screen
                // and a visibility fraction asserts nothing.
                crate::diag::ui_rect(&format!("{region}.{}", entry.trace), pressed.rect);
                if entry.available {
                    if pressed.clicked() {
                        chosen = Some(entry.pick);
                        // The working colour is abandoned rather than
                        // committed: the operator's last act was to ask for
                        // something that is not a colour.
                        state.dirty = false;
                        ui.close();
                    }
                } else {
                    pressed.on_disabled_hover_text(entry.unavailable_hover);
                }
            }
            if !control.entries.is_empty() {
                ui.separator();
            }
            // `/MK` colours carry no opacity: Table 189 gives them none.
            let mut pick = crate::colourpick::Pick {
                ink: state.working,
                opacity: None,
            };
            if crate::colourpick::body(ui, &mut pick, crate::colourpick::Spaces::ALL, region) {
                state.working = pick.ink;
                state.dirty = true;
            }
            crate::diag::ui_rect(&format!("{region}.picker"), ui.min_rect());
        });

    let open_after = egui::Popup::is_id_open(ui.ctx(), popup_id);
    let committed = if chosen.is_some() {
        chosen
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

/// The picker's in-progress value across the frames it is open for, in the
/// space the `/MK` key states.
#[derive(Clone, Copy, Debug, PartialEq)]
struct MkEditing {
    /// The colour the picker is showing right now.
    working: crate::colourpick::Ink,
    /// Moved since this open: opening to look and closing authors nothing.
    dirty: bool,
    /// Open on the previous frame; the close edge is the commit trigger.
    was_open: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The picker's opening position over an unshowable key is the PDF
    /// default.
    #[test]
    fn an_unshowable_key_opens_on_the_pdf_default() {
        assert_eq!(NO_PARTICULAR_COLOUR, [0, 0, 0]);
    }
}
