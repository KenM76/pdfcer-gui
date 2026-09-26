//! # `app::fontband` — the three Format ▸ Font controls the ribbon cannot draw
//! itself
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/fontband.md`.

use egui::Ui;
use egui_shell::commands::{CommandRegistry, ConditionSet, HandlerToken};

use crate::app::actions::textstyle::StyleChange;
use crate::app::state::OpenDoc;
use crate::panels::properties::text::TextStyleDraft;
use crate::text::panels::properties as t;

/// The width of the face chooser, in points.
const FACE_WIDTH: f32 = 78.0;

/// The width of the size field, in points. Four digits and a `pt` suffix.
const SIZE_WIDTH: f32 = 46.0;

/// Draw one Format ▸ Font custom item, or nothing.
pub(super) fn draw(
    ui: &mut Ui,
    kind: &str,
    registry: &CommandRegistry,
    conditions: &ConditionSet,
    doc: Option<&OpenDoc>,
    draft: &mut TextStyleDraft,
    parked: &mut Option<StyleChange>,
) -> Option<HandlerToken> {
    let id = command_for(kind)?;
    // R8: a build that does not register the command draws no control for it.
    let command = registry.get(id)?;
    let enabled = command.is_enabled(conditions);

    // The read-back is attempted only when the control is live, and that is
    // a **performance** decision with a measured number behind it rather than
    // tidiness: `TextStyleDraft::sync` runs a text extraction with provenance
    // capture on, which is 392 ms on the operator's benchmark sheet. The draft
    // is stamped so it re-reads only when the selection or the document moves,
    // but a greyed control has nothing to read and must not be the thing that
    // asks.
    let ready = enabled.then(|| resolved(doc, draft)).flatten();
    let live = ready.is_some();
    report_enablement(id, enabled, live);

    let mut invoked = false;
    let response = ui
        .add_enabled_ui(live, |ui| {
            match kind {
                k if k == crate::shell::manifest::FONT_FACE => {
                    invoked = face(ui, doc, draft, ready.as_ref(), parked);
                }
                k if k == crate::shell::manifest::FONT_SIZE => {
                    invoked = size(ui, draft, live, parked);
                }
                k if k == crate::shell::manifest::FONT_COLOUR => {
                    invoked = colour(ui, draft, live, parked);
                }
                // Unreachable while `command_for` names exactly these three,
                // and named rather than left as the colour swatch's catch-all
                // for that reason: a fourth Font kind added to `command_for`
                // and forgotten here would otherwise draw a **colour swatch**
                // under its own tooltip, which is a control that works, reports
                // a rect, and edits the wrong property.
                _ => {}
            };
        })
        .response;

    crate::diag::ui_rect(&egui_shell::ribbon::report::band_item(id), response.rect);

    // The same tooltip in both states, from the same field, because
    // `render_command` does exactly that for every other control on the band —
    // `on_hover_text` when live and `on_disabled_hover_text` when not. It is
    // why `crate::text::commands`' Font block writes every one of these five
    // tooltips to read correctly with nothing selected.
    if let Some(tip) = command.tooltip.as_ref() {
        if live {
            response.on_hover_text(tip);
        } else {
            response.on_disabled_hover_text(tip);
        }
    }

    invoked.then_some(command.handler)
}

/// Report whether this control was drawn pressable, on CHANGE only.
fn report_enablement(id: &str, enabled: bool, live: bool) {
    if !crate::diag::enabled() {
        return;
    }
    // ui-text-exempt: diagnostic trace key, never displayed.
    let key = format!("{} id={id}", egui_shell::ribbon::report::ENABLEMENT_EVENT);
    crate::diag::trace_on_change(&key, || {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("enabled={} live={}", u8::from(enabled), u8::from(live))
    });
}

/// The command each custom kind draws the control for.
fn command_for(kind: &str) -> Option<&'static str> {
    match kind {
        // ui-text-exempt: command ids, never displayed.
        k if k == crate::shell::manifest::FONT_FACE => Some("format.font"),
        k if k == crate::shell::manifest::FONT_SIZE => Some("format.font_size"),
        k if k == crate::shell::manifest::FONT_COLOUR => Some("format.font_colour"),
        _ => None,
    }
}

/// The page and the runs the controls would act on, and the draft synced to
/// them — or `None` when there is nothing to act on.
fn resolved(doc: Option<&OpenDoc>, draft: &mut TextStyleDraft) -> Option<(usize, Vec<usize>)> {
    let doc = doc?;
    let operand = draft.operand(doc)?;
    let &first = operand.runs.first()?;
    draft.sync(doc, operand.page, first).then_some(())?;
    Some((operand.page, operand.runs))
}

/// The trace region the ribbon's face chooser publishes its POPUP under.
///
/// Deliberately **not** `egui_shell::ribbon::report::band_item("format.font")`
/// — that name is the control's rect on the band, published below by [`draw`]
/// for every custom item alike, and a popup body that reused it would put two
/// different rectangles under one name in one frame. A driven check reading the
/// later one would aim at whichever the paint order happened to leave last.
///
/// It is a **prefix**: [`crate::panels::properties::face::popup_body`] hangs
/// `.addable`, `.disclosure` and `.new` off it, and the Properties panel's copy
/// hangs the same three off `properties.text.face`. Two namespaces for one body
/// so a check can say which surface it is looking at — which matters precisely
/// because the two must otherwise be identical.
// ui-text-exempt: trace region name, never displayed
const FACE_POPUP_REGION: &str = "ribbon.font.face";

/// The face chooser.
fn face(
    ui: &mut Ui,
    doc: Option<&OpenDoc>,
    draft: &TextStyleDraft,
    ready: Option<&(usize, Vec<usize>)>,
    parked: &mut Option<StyleChange>,
) -> bool {
    // The face the draft holds, or the no-value placeholder — see
    // [`crate::text::panels::properties::text_value_absent`] for why a greyed
    // control must not show a value it does not have. The draft is only synced
    // when the control is live, so `None` here is the ordinary greyed state and
    // not an error.
    let current = draft.face().unwrap_or_default().to_owned();
    let shown = if current.is_empty() {
        t::text_value_absent()
    } else {
        crate::panels::properties::text::shorten(&current)
    };
    let mut invoked = false;
    egui::ComboBox::from_id_salt("ribbon-format-font-face")
        .width(FACE_WIDTH)
        .selected_text(shown)
        .show_ui(ui, |ui| {
            // The popup is only reachable while the control is live, so this
            // closure runs only with a document and a page. Written as a
            // `let else` rather than an `expect` anyway: a paint-loop panic on
            // a state that is merely unexpected is a worse failure than a
            // popup that opens empty.
            let (Some(_doc), Some(_ready)) = (doc, ready) else {
                return;
            };
            if let Some(selector) = crate::panels::properties::face::popup_body(
                ui,
                FACE_POPUP_REGION,
                draft.faces(),
                crate::panels::properties::text::shorten(&current),
            ) {
                // Parked, not dispatched. `egui-shell`'s contract is *"the
                // shell reports, the application dispatches"*, and it is also
                // what keeps the five Font commands honest as one family: the
                // operand derivation (which page, which runs) is written once,
                // in `app::dispatch::format`, rather than once there and once
                // here.
                *parked = Some(StyleChange::Face(selector));
                invoked = true;
            }
        });
    invoked
}

/// The size field.
fn size(
    ui: &mut Ui,
    draft: &mut TextStyleDraft,
    live: bool,
    parked: &mut Option<StyleChange>,
) -> bool {
    let was = draft.size();
    // A greyed size field shows the PLACEHOLDER, not a number.
    //
    //
    // A `Button` rather than a `DragValue` with clever formatting, because a
    // `DragValue` in this state is a control that can be dragged: `add_enabled_ui`
    // makes it inert, but the shape of the thing an operator is looking at
    // should say *there is no value here*, not *here is a number you may
    // scrub*. It is disabled, so it takes no clicks and reports nothing.
    if !live {
        let response = ui.add_enabled(false, egui::Button::new(t::text_value_absent()));
        let _ = ui.allocate_space(egui::Vec2::new(
            (SIZE_WIDTH - response.rect.width()).max(0.0),
            0.0,
        ));
        return false;
    }
    let response = ui.add(
        egui::DragValue::new(draft.typed_size_mut())
            .speed(0.25)
            .range(1.0..=1440.0)
            .suffix(t::text_size_suffix())
            .max_decimals(1),
    );
    let _ = ui.allocate_space(egui::Vec2::new(
        (SIZE_WIDTH - response.rect.width()).max(0.0),
        0.0,
    ));
    if live
        && (response.drag_stopped() || response.lost_focus())
        && (draft.typed_size() - was).abs() > f64::EPSILON
    {
        *parked = Some(StyleChange::Size(draft.typed_size()));
        return true;
    }
    false
}

/// The colour swatch, or a greyed stand-in for a run this control must not
/// touch.
fn colour(
    ui: &mut Ui,
    draft: &TextStyleDraft,
    live: bool,
    parked: &mut Option<StyleChange>,
) -> bool {
    let Some(current) = draft.colour() else {
        let response = ui.add_enabled(false, egui::Button::new(t::text_colour_label()));
        // **THE SENTENCE DEPENDS ON WHY THERE IS NO COLOUR, and for
        // eight days it did not** — `OPERATOR_REQUESTS.md` O89, the third
        // candidate, which O89 recorded as *"R9's own rule, and it is not
        // doing it today."*
        //
        // `draft.colour()` answers `None` for **two completely different
        // reasons**, and this arm answered both with one sentence:
        //
        // | why | the truth |
        // |---|---|
        // | the run is painted in CMYK or a spot colour | the sentence below |
        // | **nothing is swept, so no run has been read at all** | *"sweep the text first"* |
        //
        // The second is the state an operator is in **every time they go
        // looking for this control** — they have clicked a piece of text with
        // the Select tool, the Format tab has appeared, and the Font group is
        // greyed. `resolved` never ran, so the draft holds its `Default` and
        // `colour()` is `None` — and hovering the greyed swatch answered with
        // *"Set in CMYK or a spot colour…"*: a confident, specific claim about
        // text this control had not read one byte of.
        //
        // It is the same defect class as the size field two functions up
        // (a greyed `DragValue` clamping its `Default` to `1.0 pt` and
        // reading as a fact about the document), and it is worse in one way:
        // a wrong number invites a second look, and a plausible sentence
        // ends the operator's search. He reported not being able to find the
        // control; the one surface that could have answered him told him the
        // document was the problem.
        //
        // ⇒ `live` is the discriminator and it was already in scope, unused
        // by this arm. When the control is greyed for want of an operand, the
        // hover carries the registry's own tooltip — the one
        // `crate::text::commands`' Font block writes to read correctly with
        // nothing selected, ending *"Sweeping text with the Text tool (T)
        // chooses what it applies to."* — which [`draw`] attaches to the
        // enclosing region. Saying nothing here lets that one through instead
        // of covering it with a falsehood.
        if live {
            response.on_disabled_hover_text(t::text_colour_not_plain());
        }
        return false;
    };
    let mut rgb = current;
    if ui.color_edit_button_srgb(&mut rgb).changed() && rgb != current && live {
        let components = vec![
            f64::from(rgb[0]) / 255.0,
            f64::from(rgb[1]) / 255.0,
            f64::from(rgb[2]) / 255.0,
        ];
        if let Ok(fill) =
            pdfcer_core::text_edit::NewFill::new(pdfcer_core::text_edit::FillModel::Rgb, components)
        {
            *parked = Some(StyleChange::Fill(fill));
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every custom kind the manifest declares for this group is drawn
    /// here, and every kind drawn here backs a registered command.**
    #[test]
    fn every_font_kind_in_the_register_is_drawn_by_this_module() {
        let mut registry = egui_shell::commands::CommandRegistry::new();
        crate::shell::commands::register(&mut registry);
        for (id, kind, _) in crate::shell::manifest::CUSTOM_BACKED {
            let Some(mapped) = command_for(kind) else {
                // Not this module's kind — `recent_files` is the other entry.
                continue;
            };
            assert_eq!(
                mapped, *id,
                "`{kind}` is registered as backing `{id}` and this module draws it for `{mapped}`"
            );
            assert!(
                registry.get(id).is_some(),
                "`{id}` is drawn by this module and is not in the registry, so the control would \
                 silently vanish"
            );
        }
    }

    /// The three kinds this module claims are exactly the three the manifest
    /// declares — asserted as an **exact set**, not as three `contains`.
    #[test]
    fn this_module_draws_exactly_the_three_font_kinds() {
        use crate::shell::manifest::{FONT_COLOUR, FONT_FACE, FONT_SIZE};
        let mine: Vec<&str> = [FONT_FACE, FONT_SIZE, FONT_COLOUR]
            .into_iter()
            .filter(|k| command_for(k).is_some())
            .collect();
        assert_eq!(mine, [FONT_FACE, FONT_SIZE, FONT_COLOUR]);
        assert!(
            command_for(crate::shell::manifest::COLOUR_SWATCH).is_none(),
            "the Markup pen's swatch is not a Font control and must not be claimed here"
        );
        assert!(command_for("nonsense").is_none());
    }
}
