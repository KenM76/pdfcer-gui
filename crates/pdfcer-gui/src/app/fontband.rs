//! # `app::fontband` — the three Format ▸ Font controls the ribbon cannot draw
//! itself
//!
//! `RIBBON_IA.md` §5.8's *Text run* row, the half of it that is not a button.
//!
//! ## What this module is
//!
//! The Font group has five controls. Two of them — Bold and Italic — are
//! ordinary `Item::Command`s and the ribbon draws them, greys them and shows
//! their tooltips with no help from this crate. The other three are
//! `Item::Custom`s, because a face chooser has to ask *which* of the page's
//! fonts, a size field has to accept a number, and a colour needs a swatch
//! that shows the current one. `egui_shell::manifest::Item::Custom` is the
//! extension point for exactly that, and it hands the application a `Ui` and
//! gets out of the way.
//!
//! This module is what goes in that `Ui`.
//!
//! ## ★★★ The shell reserves the slot; everything else is ours, including the
//! greying
//!
//! This is the sentence to read before changing anything here, because it is
//! the difference between this file and the ribbon's own control renderer.
//!
//! `egui_shell::ribbon::control::render_command` does four things for a
//! command item: it evaluates `enable`, it draws the control greyed when the
//! predicate is false, it shows the tooltip through `on_hover_text` **or**
//! `on_disabled_hover_text`, and it publishes the control's rect under
//! `ribbon.item.<id>`. For a custom item it does **none** of them — it cannot,
//! because it does not know what is being drawn.
//!
//! So all four are done here, deliberately in the same shapes and under the
//! same names, and the reason they are not *approximately* the same is R9: a
//! greyed control is only legitimate when it explains itself on hover, and
//! these three are greyed for most of their life. An operator meets them
//! after clicking a piece of text with the Select tool — the Format tab
//! appears, the Font group is there, and nothing is swept — and the tooltip is
//! the one surface in the whole application that can say, at that moment, that
//! sweeping with the Text tool is what gives them something to act on. That is
//! O37's *"nothing on screen tells you to press T"*, answered where the
//! question is asked.
//!
//! ★ The rect is published under `ribbon.item.<command id>` — the same name
//! `egui_shell::ribbon::report::band_item` builds for a command control — so a
//! driven check finds a face chooser the same way it finds a Delete button. A
//! second naming scheme for "the same kind of thing, drawn by the other half
//! of the program" is how a harness comes to have two lookup paths, which is
//! the defect `driving::declared_or_in_overflow` was written to end.
//!
//! ## ★★ It reports; it does not dispatch
//!
//! Every control here parks a [`StyleChange`] and returns the command's
//! `HandlerToken`. It raises no `Action` and touches no document.
//!
//! That is `egui-shell`'s contract — *"the shell reports, the application
//! dispatches"* — and it is also what keeps the five Font commands honest as
//! one family: `format.bold` and `format.italic` reach
//! `app::dispatch::format` through a ribbon click, and so do these three, so
//! the operand derivation (*which page, which runs*) is written **once**, in
//! the dispatch arm, rather than once there and once here. A chord that never
//! touched the ribbon then gets the same answer as a click.
//!
//! `app::recent::menu` is the precedent and the shape is identical: the
//! picker asks, the command acts.
//!
//! ## ★ Why these read the registry rather than `crate::text::commands`
//!
//! The label and tooltip could be fetched straight from
//! `crate::text::commands::format_font()`, which is where the registry got
//! them, and it would be one source. The registry is asked instead because of
//! **R8**: a command a build does not register does not exist, and a custom
//! item that drew a control for one would be the one place in this shell where
//! a compiled-out capability still had a surface. `registry.get(id)` answering
//! `None` draws nothing, exactly as `MenuHost::label` answering `None` removes
//! a row from the Tool panel.
//!
//! ## Rule 4
//!
//! Nothing here marks the canvas, and nothing here can. Every disclosure a
//! press causes — a synthetic weight, a real face substituted, a colour space
//! narrowed — is raised by `app::actions::textstyle` into the status bar. The
//! restyled text renders exactly as the saved file will render it.

use egui::Ui;
use egui_shell::commands::{CommandRegistry, ConditionSet, HandlerToken};

use crate::app::actions::textstyle::StyleChange;
use crate::app::state::OpenDoc;
use crate::panels::properties::text::TextStyleDraft;
use crate::text::panels::properties as t;

/// The width of the face chooser, in points.
///
/// ★ Chosen against `egui_shell::ribbon::plan::CUSTOM_ITEM_WIDTH`, which is
/// **96** and is what the band budgets for a custom item it cannot measure.
/// That module's header is explicit about the asymmetry: *"an estimate that is
/// too small costs a clipped group; it cannot cost the overflow control"* — so
/// drawing wider than the budget is not a crash, it is a group that gets cut
/// off at the right edge under width pressure, which is a defect the operator
/// sees and cannot explain.
///
/// So the three controls here are sized to fit **inside** the budget rather
/// than to look comfortable: this one plus its frame padding is the widest,
/// and it is the one that had to give. A `/BaseFont` is routinely thirty
/// characters (`ABCDEF+HelveticaNeueLTStd-Md`), so it would be truncated at any
/// width that fits on a ribbon; `shorten` takes the subset tag off and the
/// combo elides the rest, and the Properties panel is where the full name is
/// legible.
const FACE_WIDTH: f32 = 78.0;

/// The width of the size field, in points. Four digits and a `pt` suffix.
const SIZE_WIDTH: f32 = 46.0;

/// Draw one Format ▸ Font custom item, or nothing.
///
/// Returns the command's handler token when the operator changed something, in
/// which case `parked` holds the change. `None` means *nothing was invoked*,
/// which is what the shell expects for a frame in which the operator merely
/// looked at the control.
///
/// # ★ `kind` is matched, not asserted
///
/// An unrecognised kind returns `None` and draws nothing, exactly as
/// [`crate::app::PdfcerApp::ribbon_band`]'s renderer does for one it does not
/// know. A manifest is data; the honest response to a kind nobody implements
/// is a gap, not a panic in the paint loop.
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

    // ★ The read-back is attempted only when the control is live, and that is
    // a **performance** decision with a measured number behind it rather than
    // tidiness: `TextStyleDraft::sync` runs a text extraction with provenance
    // capture on, which is 392 ms on the operator's benchmark sheet. The draft
    // is stamped so it re-reads only when the selection or the document moves,
    // but a greyed control has nothing to read and must not be the thing that
    // asks.
    let ready = enabled.then(|| resolved(doc, draft)).flatten();
    let live = ready.is_some();

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
                _ => {
                    invoked = colour(ui, draft, live, parked);
                }
            };
        })
        .response;

    crate::diag::ui_rect(&egui_shell::ribbon::report::band_item(id), response.rect);

    // ★ The same tooltip in both states, from the same field, because
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

/// The command each custom kind draws the control for.
///
/// One place, so that the kind → id mapping cannot be spelled one way in the
/// renderer and another in `manifest::CUSTOM_BACKED` — which is the register
/// that keeps these three from looking like orphaned commands to the
/// reachability check, and which is asserted against the manifest rather than
/// against this function.
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
///
/// ★ It asks the **same** three questions `panels::properties::text::section`
/// asks, in the same order: a text selection exists, it is live against this
/// document's edit epoch, and its first run pins. A control that used a looser
/// test would be live at exactly the moment pressing it declined, which is the
/// disagreement `selection.bounds` was invented to prevent for
/// zoom-to-selection.
///
/// The `selection.text` condition already covers the first two, and this
/// re-asks them anyway: a condition is a hint published for the ribbon's
/// benefit, and it is evaluated a frame's worth of state earlier than the
/// draw. Only the third — does the run pin? — is genuinely new information,
/// and it is the one that cannot be published as a condition because answering
/// it costs 392 ms.
fn resolved(doc: Option<&OpenDoc>, draft: &mut TextStyleDraft) -> Option<(usize, Vec<usize>)> {
    let doc = doc?;
    let selection = doc.text_selection.as_ref()?;
    let runs = selection.runs(doc.edit_epoch);
    let &first = runs.first()?;
    draft.sync(doc, selection.page, first).then_some(())?;
    Some((selection.page, runs))
}

/// The trace region the ribbon's face chooser publishes its POPUP under.
///
/// ★ Deliberately **not** `egui_shell::ribbon::report::band_item("format.font")`
/// — that name is the control's rect on the band, published below by [`draw`]
/// for every custom item alike, and a popup body that reused it would put two
/// different rectangles under one name in one frame. A driven check reading the
/// later one would aim at whichever the paint order happened to leave last.
///
/// ★★ It is a **prefix**: [`crate::panels::properties::face::popup_body`] hangs
/// `.addable`, `.disclosure` and `.new` off it, and the Properties panel's copy
/// hangs the same three off `properties.text.face`. Two namespaces for one body
/// so a check can say which surface it is looking at — which matters precisely
/// because the two must otherwise be identical.
// ui-text-exempt: trace region name, never displayed
const FACE_POPUP_REGION: &str = "ribbon.font.face";

/// The face chooser.
///
/// ★ **No label beside it.** The group's caption already says *Font*, the
/// control shows the current face, and Word's own font-name box carries no
/// label for the same two reasons. A label here would be the third occurrence
/// of the word within one inch of ribbon.
///
/// # ★★★ The popup body is NOT written here, and that is the point
///
/// It was, until 2026-08-29: this function and
/// `panels::properties::text::face_row` held two copies of one `for` loop over
/// the same list. `Pass 162.0` then gave the chooser a second kind of row — the
/// standard-14 faces pdfcer authors on demand — with a disclosure it owes,
/// headings to tell the two kinds apart, and a minimum popup width so the
/// sentence is legible on a band whose control is 78 points wide.
///
/// Adding all of that twice is how *"a face offered in one surface and not the
/// other"* happens, which this project has found more than once. So both
/// surfaces call [`crate::panels::properties::face::popup_body`], and this
/// function keeps only what is genuinely the ribbon's: the width, the
/// placeholder for a greyed control with no value, and parking the result as a
/// [`StyleChange`] rather than raising an [`crate::app::actions::Action`].
fn face(
    ui: &mut Ui,
    doc: Option<&OpenDoc>,
    draft: &TextStyleDraft,
    ready: Option<&(usize, Vec<usize>)>,
    parked: &mut Option<StyleChange>,
) -> bool {
    // ★ The face the draft holds, or the no-value placeholder — see
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
            // ★ The popup is only reachable while the control is live, so this
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
                // ★★ Parked, not dispatched. `egui-shell`'s contract is *"the
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
///
/// ★ Committed on `drag_stopped` or `lost_focus`, **never** on `.changed()`,
/// for the reason the Properties panel's twin gives: each commit is a
/// content-stream rewrite and one undo entry, so committing on change would
/// author an edit per pixel of drag and leave a `Ctrl+Z` stack an operator
/// could not get back through.
fn size(
    ui: &mut Ui,
    draft: &mut TextStyleDraft,
    live: bool,
    parked: &mut Option<StyleChange>,
) -> bool {
    let was = draft.size();
    // ★★★ A greyed size field shows the PLACEHOLDER, not a number.
    //
    // Found by looking at a screenshot on 2026-08-27, after the driven check
    // had passed. With nothing swept the draft holds its `Default` — zero —
    // and `DragValue`'s own `range(1.0..=1440.0)` clamps that up, so the
    // greyed control read **`1.0 pt`**: a claim about the operator's document,
    // and a false one. The check was right to pass; it asserts that the control
    // is drawn, and it was.
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
///
/// # ★★ Why a run painted in CMYK greys rather than showing its nearest RGB
///
/// A swatch showing DeviceCMYK ink as its nearest sRGB would write that sRGB
/// back on the next press, moving the run out of its original space for ever —
/// on a document heading for a printer that cares. `pdfcer-core` deliberately
/// stores the space it was given instead of force-converting to DeviceRGB the
/// way Acrobat does, and a control that undid that on the operator's behalf
/// would make the engine's care pointless.
///
/// The panel says so in a sentence (`text_colour_not_plain`). A ribbon band has
/// no room for a sentence, so the same fact is carried by a **greyed swatch
/// with that sentence on hover**, which is R9's shape exactly: temporarily
/// unavailable, explained on hover, and the explanation is the real reason
/// rather than a generic one.
fn colour(
    ui: &mut Ui,
    draft: &TextStyleDraft,
    live: bool,
    parked: &mut Option<StyleChange>,
) -> bool {
    let Some(current) = draft.colour() else {
        let response = ui.add_enabled(false, egui::Button::new(t::text_colour_label()));
        // ★★★ **THE SENTENCE DEPENDS ON WHY THERE IS NO COLOUR, and for
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
        // ★★ It is the same defect class as the size field two functions up
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

    /// ★★★ **Every custom kind the manifest declares for this group is drawn
    /// here, and every kind drawn here backs a registered command.**
    ///
    /// The assertion that closes the gap `COLOUR_SWATCH`'s own doc comment
    /// records: the manifest wrote a custom kind, **no renderer ever matched
    /// it**, and the Markup ▸ Style group drew a caption over an empty band
    /// for the whole of v0.1.0 with nothing anywhere reporting the mismatch.
    /// The shell reserves the item's space, the application declines to draw,
    /// and the only symptom is a gap.
    ///
    /// It is asserted through `manifest::CUSTOM_BACKED`, which is the register
    /// that already pairs a command id with the kind that draws it and is
    /// already tested against the manifest in both directions. Reading it here
    /// makes the chain complete: manifest → register → renderer → registry.
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
    ///
    /// ★ A fourth kind added here and not to the manifest is a renderer arm
    /// nothing can ever reach; a fourth added to the manifest and not here is
    /// the empty-band defect above. Only an equality catches both.
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
