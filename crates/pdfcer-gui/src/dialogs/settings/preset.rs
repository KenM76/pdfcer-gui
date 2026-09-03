//! # `dialogs::settings::preset` — a named vector of answers
//!
//! **Operator request O38, 2026-08-25:**
//!
//! > *"I'd like a preset setting for rendering things to what the [print
//! > conformance suite] page needs to render correctly … since it is for
//! > conformance to PDF/X-4 (ISO 15930-7) … maybe we should have a dropdown to
//! > select view options between the different standards."*
//!
//! ## What a preset IS, and what it deliberately is not
//!
//! A preset is a **named bundle of settings that already exist**, applied in one
//! click, and **individually editable afterwards**. It is not a mode, not a
//! second rendering path, and not a lock. Choosing one writes values into the
//! draft exactly as though the operator had set each control by hand, and the
//! window then behaves as it always did.
//!
//! That is the whole design, and it is what makes the feature cheap and safe:
//! there is no new state to persist, no new thing that can disagree with
//! `settings.txt`, and no way for a preset to express something the individual
//! controls cannot. A preset that could would be a second source of truth.
//!
//! ## ★★★ Where the values came from, and why every one of them is graded
//!
//! Not from here. A control labelled *ISO 15930-7* carries that standard's
//! authority whether or not it was meant to, so the vector was **asked for**
//! and the engine answered with an API rather than a table
//! (`settings::presets`, Pass 128.1). Its reply quoted this project's own
//! reason back at it, and then made the sharper point:
//!
//! > *"The interesting column is not the value. It is how much weight the
//! > value can bear, and for most of these axes the answer is less than the
//! > button implies."*
//!
//! Only **one** of PDF/X-4's six answers is a claim about the standard at all,
//! and it is graded `implied` rather than `sourced`. So the row shows the
//! grading beside the choice; a row that showed the name and hid the grade
//! would be the over-claim this whole request was careful to avoid.
//!
//! ★★ Three consequences the engine had to spell out, all of which shape this
//! file:
//!
//! 1. **Not every standard binds a renderer.** PDF/A and PDF/UA both put
//!    *"operational details of rendering"* outside their own scope. `PdfUa1`
//!    exists and correctly **sets nothing** — surfaced as an answer, not
//!    hidden as an omission, because *"nothing, and here is the measurement"*
//!    cannot be mistaken for unfinished work.
//! 2. **A third of the grid is axes a standard does not reach.** No PDF/X part
//!    contains a shading clause, so none of them says anything about mesh
//!    padding. `PresetAction::LeaveAlone` is a real state and is rendered as
//!    one — blank would read as missing data, a value would assert a
//!    requirement that does not exist.
//! 3. **`cmyk_intent` has no conformant value.** Every PDF/X level guarantees a
//!    *colorimetric* definition of device colour, and `CmykIntent` selects
//!    among fixed built-in tables, which is not one. The preset takes the
//!    least-wrong value and **discloses that the file's own output intent was
//!    not applied** — mandatory under rule 4, because a colour transform that
//!    did not happen leaves nothing on screen to notice.
//!
//! ## What DOES ship today, and why it is not a consolation prize
//!
//! **pdfcer's own recommended answers.** The operator's report that opened this
//! request included *"touching some of our presets caused some test to show up
//! as failed"* — which is a person who has changed several settings while
//! investigating and now wants a way back. That is a real need, it is
//! answerable today with complete authority (these are *our* defaults; no
//! external standard is being spoken for), and it is the half of the request
//! that was never blocked.

use pdfcer_core::settings::Settings;
use pdfcer_core::settings::presets::{Evidence, PresetKey, RenderPreset, RenderStandard};

use super::Draft;

/// One thing the operator can choose from the presets row.
///
/// Two kinds, and the distinction is the whole model:
///
/// * [`Choice::Recommended`] — pdfcer's own shipped answers. **We** are the
///   authority, so it can say what it does without qualification.
/// * [`Choice::Standard`] — a published standard's answers, from
///   `pdfcer_core::settings::presets`. **We are not the authority**, so every
///   value carries the engine's own evidence grade and the row shows it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    /// Restore what pdfcer ships with.
    Recommended,
    /// Render as a published standard specifies — as far as it specifies
    /// anything at all.
    Standard(RenderStandard),
}

impl Choice {
    /// A stable id for the radio's identity and for tests.
    fn id(self) -> &'static str {
        match self {
            Self::Recommended => "pdfcer",
            Self::Standard(s) => s.as_str(),
        }
    }

    /// What the operator sees.
    fn label(self) -> String {
        match self {
            Self::Recommended => crate::text::settings::preset_pdfcer_label().to_owned(),
            Self::Standard(s) => s.title().to_owned(),
        }
    }

    /// Write this choice's answers into a settings struct.
    fn apply(self, settings: &mut Settings) {
        match self {
            Self::Recommended => apply_pdfcer(settings),
            // ★ A standard's preset is applied ON TOP of pdfcer's defaults, not
            // onto whatever the operator last had. Otherwise "PDF/X-4" would
            // mean something different depending on what was set before it,
            // which is the one thing a named preset must not do.
            Self::Standard(s) => {
                apply_pdfcer(settings);
                let _ = RenderPreset::for_standard(s).apply(settings);
            }
        }
    }
}

/// **Everything the presets row offers**, in display order.
///
/// ★★★ pdfcer's own answers first, then every standard the engine knows about.
/// The list is *derived* from `RenderStandard::all()` rather than restated, so
/// a standard the engine adds appears here with no change at all — R8's
/// registration rule, reached through the crate boundary instead of through a
/// command registry.
#[must_use]
pub fn choices() -> Vec<Choice> {
    std::iter::once(Choice::Recommended)
        .chain(RenderStandard::all().iter().copied().map(Choice::Standard))
        .collect()
}

/// pdfcer's own recommended answers.
///
/// # ★★★ It is `Settings::default()`, and getting here took two corrections
///
/// The first draft restated two values explicitly, believing pdfcer-gui diverged
/// from the engine on both. **Neither was true**, and the test written to catch
/// exactly this caught both, an hour apart:
///
/// * **`image_minify`** genuinely diverged — for one day. The engine adopted
///   `Smooth` on 2026-08-25 (Pass 128.0) on the strength of the operator's
///   Acrobat comparison, and the restatement became a no-op that same evening.
/// * **`cmyk_intent`** never diverged at all. `NeutralBlack` has been the
///   engine's *shipped default* since the operator's 2026-08-08 ruling was
///   adopted there. What the engine's doc records is a divergence **in
///   reasoning** — it says openly that its default is knowingly not the
///   best-evidenced answer — and that was misread here as a divergence in
///   **value**.
///
/// ★ The second is the more instructive mistake. A doc comment asserting a
/// difference that does not exist is worse than no comment: it invites the next
/// reader to preserve a line that does nothing, and it makes a *deliberate*
/// override indistinguishable from a copied one. **Read the value, not the
/// prose about the value.**
///
/// So this assigns nothing. Every answer is the engine's, taken by *not
/// assigning it*, so a default the engine changes tomorrow arrives here for
/// free and cannot rot into a stale literal.
fn apply_pdfcer(s: &mut Settings) {
    *s = Settings::default();
}

/// ★ **The `&'static str` a stored id names**, or `None` if this build has no
/// such choice.
///
/// The seam between `Prefs`'s owned `String` and the `&'static str` every other
/// reader here compares against. Resolving through [`choices`] rather than
/// leaking the string is what keeps an unknown id — hand-edited, or written by
/// a newer pdfcer that knows a standard this one does not — from travelling any
/// further than this function. It is not an error: the window simply falls back
/// to the derived reading, which is the honest answer for a choice this build
/// cannot offer.
#[must_use]
pub fn resolve_id(stored: Option<&str>) -> Option<&'static str> {
    let stored = stored?;
    choices()
        .into_iter()
        .map(Choice::id)
        .find(|id| *id == stored)
}

/// The de-duplication slot for the presets row's state line.
const PRESET_SLOT: &str = "settings-preset"; // ui-text-exempt: a trace slot key, never displayed

/// Which preset the control should show as selected.
///
/// The operator's own choice while it still describes the working settings,
/// and otherwise the derived reading. See [`row`]'s ★★★ comment for why the
/// order is that way round and not the other.
fn live_choice(draft: &Draft) -> Option<&'static str> {
    draft
        .chosen_preset
        .filter(|id| still_holds(id, &draft.working))
        .or_else(|| matching(&draft.working))
}

/// ★ **Whether the draft's chosen standard still describes its working
/// settings** — the question `commit` asks before writing the choice to disk.
///
/// [`live_choice`]'s own filter, exposed so the display rule and the stored
/// value cannot drift apart. Without one function answering for both, the
/// window would stop showing a standard while the preferences file went on
/// naming it, and the next session would open claiming a choice the values
/// contradict.
///
/// `true` when nothing is chosen: there is no claim to retire.
#[must_use]
pub fn still_chosen(draft: &Draft) -> bool {
    draft
        .chosen_preset
        .is_none_or(|id| still_holds(id, &draft.working))
}

/// Whether `id`'s preset still describes `settings`.
///
/// ★ Asked against the preset rather than remembered as a flag, because the
/// operator can change any control in the window after choosing a preset and
/// nothing tells this module when they do. A flag would need every other
/// control to remember to clear it — the shape of a guard that is correct until
/// somebody adds a widget.
fn still_holds(id: &str, settings: &Settings) -> bool {
    choices().into_iter().any(|c| {
        c.id() == id && {
            let mut probe = Settings::default();
            c.apply(&mut probe);
            same(&probe, settings)
        }
    })
}

/// **How many other conformance presets set exactly the same render answers.**
///
/// ★★★ Measured on demand, never stated as a fact in prose. As of 2026-08-26
/// this returns 7 for every one of the eight PDF/X and PDF/A presets: they
/// agree on all six values pdfcer renders differently for. That is not a defect
/// — the standards genuinely make the same demands of a *renderer*, and differ
/// in what they demand of a **file**, which is a preflight question and not a
/// rendering one.
///
/// It is disclosed because the operator's reason for wanting the control was
/// *"especially PDF/X-4 … to see how far we are along with matching the
/// [conformance suite's] tests"*, and switching to it will change nothing on screen. Discovering that
/// by staring at an unchanged page costs an hour and reads as the setting being
/// broken. Saying it costs a line.
///
/// ★ Computed rather than written down, so the day a standard's answers diverge
/// the sentence corrects itself instead of becoming a stale claim — which is
/// this file's own recorded lesson: *read the value, not the prose about the
/// value.*
fn identical_siblings(id: &str) -> usize {
    let Some(mine) = choices().into_iter().find(|c| c.id() == id) else {
        return 0;
    };
    let mut probe = Settings::default();
    mine.apply(&mut probe);
    choices()
        .into_iter()
        .filter(|c| c.id() != id && matches!(c, Choice::Standard(_)))
        .filter(|c| {
            let mut other = Settings::default();
            c.apply(&mut other);
            same(&other, &probe)
        })
        .count()
}

/// Which preset the given settings currently match, if any.
///
/// ★ Returns `None` for "none of them", which is the **normal** state once an
/// operator has adjusted anything, and is not a fault. The control shows no
/// selection rather than pretending the nearest one is chosen — a radio that
/// claimed "pdfcer recommended" over settings that are not pdfcer's recommended
/// answers would be lying about the thing it exists to report.
#[must_use]
pub fn matching(settings: &Settings) -> Option<&'static str> {
    choices().into_iter().find_map(|c| {
        let mut probe = Settings::default();
        c.apply(&mut probe);
        same(&probe, settings).then_some(c.id())
    })
}

/// Whether two settings agree on **everything a preset sets**.
///
/// ★★ Compares the render-radius fields explicitly rather than deriving
/// `PartialEq` on `Settings`, and the reason is not tidiness. `Settings` also
/// carries `theme` — the *program's* appearance — and the two write-radius
/// entries that change bytes on disk. A preset is about how a **document
/// renders**; an operator who picks a dark theme has not stopped using pdfcer's
/// recommended rendering, and a comparison that said otherwise would clear the
/// radio for a reason that has nothing to do with rendering.
fn same(a: &Settings, b: &Settings) -> bool {
    a.cmyk_intent == b.cmyk_intent
        && a.image_minify == b.image_minify
        && a.mask_resample == b.mask_resample
        && a.page_blend_space_source == b.page_blend_space_source
        && a.mesh_patch_padding == b.mesh_patch_padding
        && a.separations == b.separations
}

/// **Draw the presets row.**
///
/// Above the groups rather than inside one, because it acts on all of them —
/// `widgets::group`'s convention is that a group holds settings sharing a
/// *subject*, and a preset shares a *purpose*.
///
/// ## ★★★ What is shown BESIDE the choice, and why it is not decoration
///
/// The engine's reply that supplied these values spent most of its length on
/// one point: *"the interesting column is not the value, it is how much weight
/// the value can bear."* Only one of PDF/X-4's six answers is a claim about the
/// standard at all, and that one is `implied` rather than `sourced`. A row that
/// showed the name and hid the grading would be exactly the over-claim the
/// whole request was careful to avoid.
///
/// So a selected standard shows:
///
/// * **its disclosures**, verbatim from the engine. These are not advisory —
///   `cmyk_intent` has no conformant value at all, so choosing a PDF/X preset
///   means a colour transform did *not* happen, and rule 4 requires saying so
///   because nothing on screen would reveal it.
/// * **what it leaves alone**, named. Roughly a third of the grid is axes a
///   standard does not reach — no PDF/X part contains a shading clause, so none
///   of them says anything about mesh padding. Showing those as blank would
///   read as missing data; showing them as values would assert a requirement
///   that does not exist.
pub fn row(ui: &mut egui::Ui, draft: &mut Draft) {
    let rect = ui
        .scope(|ui| {
            super::widgets::header(
                ui,
                crate::text::settings::preset_title(),
                crate::text::settings::preset_silence(),
                // Deliberately no radius line: a preset's radius is the union
                // of the radii of what it sets, and restating "affects what you
                // see" here while each setting states its own would be a
                // second, vaguer copy of something already precise one screen
                // down.
                "",
            );
            // ★★★ **The operator's CHOICE outranks the reading of the
            // values**, and this two-line rule is the whole of the fix for the
            // defect reported as *"I can only select (ISO15930-1, -4)"*.
            //
            // `matching` finds the FIRST choice whose settings equal the
            // current ones. Measured 2026-08-26: all eight PDF/X and PDF/A
            // presets apply byte-identical render settings, so it always found
            // PDF/X-1a and the dot jumped back there from wherever it was
            // clicked. Deriving a selection from values can only ever show as
            // many states as there are distinct values.
            //
            // ★★ The choice is authoritative **only while it remains true**.
            // `still_holds` re-asks whether the working settings are still that
            // preset's, so adjusting any control by hand drops the selection
            // back to the derived reading rather than leaving a dot claiming an
            // intent the settings no longer express. That is what keeps this
            // from becoming a label that lies.
            let current = live_choice(draft);
            // ★ The row's own state, on change only. Two facts a driven check
            // has no other way to read: WHICH standard the window is showing as
            // selected, and whether the draft has anything to save.
            //
            // The second is the operator's report of 2026-08-26 — *"select some
            // of the standards [and] the save button is greyed out"* — and it
            // is not readable from a `ui_rect`, because an enabled button and a
            // disabled one are the same size and the same place. Without this
            // line the only oracle would be a screenshot of a greyed control,
            // which is a contrast measurement standing in for a state.
            crate::diag::trace_changed(PRESET_SLOT, || {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "settings-preset chosen={} stored={} dirty={}",
                    current.unwrap_or("none"),
                    draft
                        .working_prefs
                        .chosen_standard
                        .as_deref()
                        .unwrap_or("none"),
                    draft.is_dirty()
                )
            });
            for c in choices() {
                let selected = current == Some(c.id());
                if ui.radio(selected, c.label()).clicked() && !selected {
                    c.apply(&mut draft.working);
                    draft.chosen_preset = Some(c.id());
                    // ★★★ **And into the PREFERENCES, which is what makes Save
                    // live.** 2026-08-26, on the operator's report that
                    // *"select some of the standards [and] the save button is
                    // greyed out"*.
                    //
                    // `Draft::is_dirty` compares working values against
                    // original ones, and all eight PDF/X and PDF/A presets
                    // apply byte-identical render settings — see
                    // `identical_siblings`, which measures it rather than
                    // asserting it. So choosing a second standard moved no
                    // value and the draft was, correctly, not dirty.
                    //
                    // The choice itself is now a persisted preference, so it is
                    // part of what `is_dirty` compares. Nothing about the
                    // greying logic changed: there is simply now something to
                    // save.
                    draft.working_prefs.chosen_standard = Some(c.id().to_owned());
                }
                if selected {
                    detail(ui, c);
                }
            }
        })
        .response
        .rect;

    // Published so a driven check can assert the row is ON SCREEN rather than
    // merely laid out. The settings window is a `ScrollArea`, and a control
    // scrolled out of one still reports a rect — hence `ui_rect_visible`,
    // intersected with the clip, exactly as the group headings do. A row that
    // exists and cannot be reached is a defect this project has already met
    // twice, in the Tool panel and in the overflow menu.
    crate::diag::ui_rect_visible(REGION, rect, ui.clip_rect());
}

/// What a selected choice says about itself.
///
/// Indented under its radio, muted, and **only for the selected one** — the
/// full grid for nine standards at once would be a wall of text nobody reads,
/// and the operator only needs the caveats for the answer they have chosen.
fn detail(ui: &mut egui::Ui, choice: Choice) {
    let Choice::Standard(standard) = choice else {
        ui.label(
            egui::RichText::new(crate::text::settings::preset_pdfcer_note())
                .small()
                .weak(),
        );
        return;
    };
    let preset = RenderPreset::for_standard(standard);
    ui.indent(standard.as_str(), |ui| {
        // ★★★ THE WEIGHT LINE, and it is the reason this feature is not just a
        // dropdown. The engine's own framing: *"the interesting column is not
        // the value, it is how much weight the value can bear, and for most of
        // these axes the answer is less than the button implies."* For PDF/X-4
        // exactly ONE of six answers is a claim about the standard at all.
        //
        // Summarised rather than tabulated: a six-row grid per standard, times
        // ten standards, is a wall nobody reads, and burying the choice under
        // it would trade one over-claim for an unusable control. One sentence
        // carries the same fact.
        let mut sourced = 0_usize;
        let mut inferred = 0_usize;
        let mut chosen = 0_usize;
        for e in preset.entries() {
            match e.evidence {
                Evidence::Sourced => sourced += 1,
                Evidence::Implied => inferred += 1,
                Evidence::BestEffort => chosen += 1,
                // Counted through `left_alone` below instead, where it is
                // named rather than tallied — "does not apply" is a different
                // kind of fact from "we chose", and adding them together would
                // be the arithmetic that makes a preset look better sourced
                // than it is.
                _ => {}
            }
        }
        ui.label(
            egui::RichText::new(crate::text::settings::preset_weight(
                sourced, inferred, chosen,
            ))
            .small()
            .weak(),
        );

        // ★ Verbatim from the engine, never paraphrased. These sentences carry
        // clause citations and one is quoted word-for-word out of ISO 15930;
        // rewording them here would put our phrasing behind a standard's
        // authority, which is the failure this whole feature was shaped to
        // avoid.
        for line in preset.disclosures() {
            ui.label(egui::RichText::new(line).small().weak());
        }

        // ★★★ THE `why` OF EVERY SET CLAIM ARRIVES IN `disclosures()` ABOVE —
        // it did not, for one day, and this is the record of that.
        //
        // On 2026-09-02 `RenderPreset::disclosures()` emitted an entry's `why`
        // only for entries a preset LEAVES ALONE, so the reasoning behind every
        // value a preset actually SETS never left the crate. That mattered
        // because the engine's own shipping note for the spot axis asked us to
        // show exactly that sentence — the two values render visibly
        // differently, and pdfcer's choice looks like a bug beside Acrobat's
        // composite view unless the reason is on screen.
        //
        // This window worked around it in nine lines, reading `entries()`
        // directly and filtering on
        // `evidence.is_a_claim_about_the_standard()`. It was reported as a
        // boundary finding rather than a Pass request, and it carried a test
        // asserting the engine did NOT already emit these sentences — a
        // tripwire pointing at its own deletion.
        //
        // The engine shipped the same rule, moved to where the data lives, the
        // same day. The tripwire went red on `75a17ac` and the nine lines are
        // gone. **A workaround outlives its cause only by neglect**, and the
        // way to make that not happen is to write the condition for its removal
        // as a failing test on the day you write the workaround.

        // ★★★ **How many other standards give the same answers**, measured
        // rather than asserted. See `identical_siblings`: today every one of
        // the eight conformance presets returns 7, so choosing between them
        // changes nothing pdfcer renders — which is exactly what the operator
        // is about to test for and would otherwise spend an hour discovering.
        //
        // Placed after the evidence weight and before the left-alone list,
        // because it is a statement about THIS preset's answers and belongs
        // beside the other two.
        let siblings = identical_siblings(choice.id());
        if siblings > 0 {
            ui.label(
                egui::RichText::new(crate::text::settings::preset_same_as_others(siblings))
                    .small()
                    .weak(),
            );
        }

        let untouched = preset.left_alone();
        if !untouched.is_empty() {
            // ★★ Named with the titles the operator already reads one screen
            // down, NOT with the engine's key names. `mesh_patch_padding` is a
            // field identifier; "A gradient fill that comes out scrambled" is
            // the control they would go looking for. Printing the identifier
            // would be leaking our vocabulary into their window, and the
            // ui-strings gate would be right to object.
            let names: Vec<&str> = untouched.iter().map(|k| operator_title(*k)).collect();
            ui.label(
                egui::RichText::new(crate::text::settings::preset_leaves_alone(
                    // ui-text-exempt: a list separator, not a sentence. The
                    // sentence around it lives in `text::settings`.
                    &names.join("; "),
                ))
                .small()
                .weak(),
            );
        }
    });
}

/// The operator-facing title for a settings key.
///
/// ★ The engine names these `mesh_patch_padding`, `image_minify` and so on —
/// field identifiers, correct for an API and wrong for a window. Every one of
/// them already has a title in [`crate::text::settings`], written by the
/// symptom that would send somebody looking for it, and this is the one place
/// the two vocabularies meet.
///
/// A `PresetKey` the engine adds later falls through to its own name rather
/// than to a placeholder: an unfamiliar identifier is ugly and honest, whereas
/// a guessed title would be a sentence pdfcer never wrote.
fn operator_title(key: PresetKey) -> &'static str {
    use crate::text::settings as t;
    match key {
        PresetKey::PageBlendSpaceSource => t::blend_space_title(),

        PresetKey::MeshPatchPadding => t::mesh_padding_title(),
        PresetKey::MaskResample => t::mask_title(),
        PresetKey::ImageMinify => t::minify_title(),
        PresetKey::CmykIntent => t::cmyk_intent_title(),
        PresetKey::Separations => t::separations_title(),
        // ★ Added 2026-09-02, the same day the engine gained the axis. This
        // test is the fourth time it has caught a key the engine added and this
        // shell had not been taught — the shipping reply for that Pass calls it
        // out by name: their preset grid predates the setting and nobody
        // revisited it, and our coverage contract is what noticed.
        PresetKey::SpotColorantDeviceModel => t::spot_model_title(),
        // ui-text-exempt: the engine's own key name, shown only when this shell
        // has not yet been taught a title for a key the engine added.
        other => other.as_str(),
    }
}

/// The published region name for the presets row.
///
/// A cross-repo stability contract with `tools/ui-verify`: renaming it is
/// changing an API, not tidying a string.
// ui-text-exempt: a diagnostic region name, never displayed.
pub const REGION: &str = "settings.presets";

#[cfg(test)]
mod tests {

    /// **★★★ The reasoning behind every SET claim reaches the operator — and
    /// since 2026-09-02 it arrives in the engine's own `disclosures()`.**
    ///
    /// This test began life asserting the output of a nine-line workaround in
    /// this window, because `disclosures()` emitted a `why` only for keys a
    /// preset LEAVES ALONE. The engine shipped the same rule the same day; the
    /// workaround is gone and this now points at the real thing.
    ///
    /// ★★ What it checks is deliberately the *content* that makes this a rule-4
    /// disclosure — that another viewer will show these areas differently — and
    /// not merely that some sentence mentioning spots appeared. The two values
    /// render visibly differently, and without that sentence an operator
    /// comparing pdfcer with Acrobat sees pdfcer being wrong on a page where it is
    /// being deliberately more correct.
    ///
    /// PDF/A is the counter-case in the same test, because the engine's sourced
    /// reply was explicit that PDF/A does not reach this axis: its Scope clause
    /// excludes the operational details of rendering, in every part from 2005 to
    /// 2020. Showing the sentence there would assert a requirement that does not
    /// exist.
    #[test]
    fn a_pdf_x_preset_says_a_composite_viewer_will_differ_and_pdf_a_does_not() {
        let x4 = RenderPreset::for_standard(RenderStandard::PdfX4)
            .disclosures()
            .join(" ");
        assert!(
            x4.contains("knocked out"),
            "PDF/X-4 does not tell the operator that another viewer will show \
             these areas differently, which is the whole reason the engine was \
             asked for this sentence: {x4}"
        );
        assert!(
            x4.contains("No ISO 15930 clause requires this"),
            "the sentence lost the clause that keeps it from over-claiming: {x4}"
        );
        let a2 = RenderPreset::for_standard(RenderStandard::PdfA2)
            .disclosures()
            .join(" ");
        // ★★★ PDF/A must not CLAIM the axis — but it SHOULD name it and say
        // it does not reach it. The first version of this assertion demanded
        // the word "spot" be absent, and went red on a correct build: the
        // engine emits `spot_colorant_device_model` **left alone (sourced)**
        // for PDF/A, explaining that ISO 19005's Scope clause excludes the
        // operational details of rendering and that how a spot ink is shown
        // on screen is not a PDF/A question.
        //
        // That is exactly the disclosure an operator wants, and asserting a
        // WORD absent instead of a MEANING would have deleted it. The test
        // for over-claiming is the device-simulation sentence, which is the
        // thing PDF/X says and PDF/A must not.
        assert!(
            !a2.contains("knocked out"),
            "PDF/A is claiming the spot device model, and no part of ISO 19005 \
             reaches that axis: {a2}"
        );
        assert!(
            a2.contains("left alone"),
            "PDF/A says nothing about the axes it does NOT reach, so a blank in the grid \
             reads as missing data rather than as not-applicable: {a2}"
        );
    }

    /// **★★ A best-effort value is still summarised as a COUNT, not spelled
    /// out** — which is the half of the rule that keeps the panel readable.
    ///
    /// The distinction the engine adopted, in one sentence: *a count is an
    /// honest summary of a judgement and not of a claim*. So a value pdfcer
    /// CHOSE where the standard is silent is reported as a number, and a value
    /// pdfcer says the standard IMPLIES must show its citation, because that is
    /// something an operator may want to check.
    ///
    /// ★ Pinned because the obvious "improvement" is to print every `why`, and
    /// that turns a disclosure into a wall of prose nobody reads — six entries
    /// times ten standards. `image_minify` is best-effort on every PDF/X and
    /// PDF/A preset and is the stable example to test against.
    #[test]
    fn a_best_effort_value_is_counted_and_not_spelled_out() {
        for standard in RenderStandard::all() {
            let preset = RenderPreset::for_standard(*standard);
            let disclosed = preset.disclosures().join(" ");
            for entry in preset.entries() {
                if entry.evidence == Evidence::BestEffort
                    && !matches!(entry.action, PresetAction::LeaveAlone)
                {
                    let probe: String = entry.why.chars().take(50).collect();
                    assert!(
                        !disclosed.contains(&probe),
                        "{} spells out a best-effort value's reasoning \
                         instead of counting it, which is how this disclosure \
                         becomes a wall nobody reads: {probe}",
                        standard.as_str()
                    );
                }
            }
        }
    }
    use super::*;
    // `PresetAction` is a TEST-only need since the engine took over emitting
    // set entries' reasoning: the only remaining reader is the assertion that
    // a best-effort value stays counted rather than spelled out.
    use pdfcer_core::settings::presets::PresetAction;

    /// **Applying a choice leaves the settings matching a choice with the same
    /// answers** — and that is deliberately weaker than "matching itself".
    ///
    /// ★★★ Because **two standards can mean the same thing to pdfcer**, and two
    /// of them do: applying `pdf-x3` leaves settings that `matching` reports as
    /// `pdf-x1a`. That is not a defect in either — it is a fact about the
    /// domain. PDF/X-1a and PDF/X-3 differ in what colour spaces a *file* may
    /// contain, and pdfcer's render-radius settings cannot see that difference,
    /// so the two produce an identical vector.
    ///
    /// The first version of this test asserted the stronger property and failed
    /// on the second standard it tried. Weakening it was the right response
    /// rather than adding state to remember which button was pressed: the radio
    /// reflects **what the settings are**, not what was last clicked, and a
    /// remembered choice could disagree with `settings.txt` — which is the one
    /// thing this feature's design set out to make impossible.
    ///
    /// What must hold is the round trip that the radio actually depends on: the
    /// settings after applying a choice are the settings of *whatever* choice
    /// is reported, so the selection shown is never a lie about the values.
    #[test]
    fn applying_a_choice_leaves_settings_that_match_an_equivalent_one() {
        for c in choices() {
            let mut applied = Settings::default();
            c.apply(&mut applied);

            let id = matching(&applied)
                .unwrap_or_else(|| panic!("applying `{}` must match SOME choice", c.id()));

            let equivalent = choices()
                .into_iter()
                .find(|o| o.id() == id)
                .expect("`matching` returns an id from `choices`");
            let mut reapplied = Settings::default();
            equivalent.apply(&mut reapplied);
            assert!(
                same(&applied, &reapplied),
                "applying `{}` reported `{id}`, but `{id}`'s own answers differ from what was applied — the radio would be showing a selection that does not describe the settings",
                c.id()
            );
        }
    }

    /// ★★ **At least two standards share a vector**, recorded so that the
    /// weakened assertion above is understood as describing the domain rather
    /// than as a concession.
    ///
    /// If this ever fails — because pdfcer gains a setting that distinguishes
    /// them — the test above can be strengthened back, and this is the note
    /// that says so.
    #[test]
    fn some_standards_are_indistinguishable_from_the_settings_alone() {
        let mut a = Settings::default();
        let mut b = Settings::default();
        Choice::Standard(RenderStandard::PdfX1a).apply(&mut a);
        Choice::Standard(RenderStandard::PdfX3).apply(&mut b);
        assert!(
            same(&a, &b),
            "PDF/X-1a and PDF/X-3 now differ in pdfcer's render settings, so `applying_a_choice_leaves_settings_that_match_an_equivalent_one` can go back to asserting each choice matches ITSELF"
        );
    }

    /// ★★ **pdfcer's recommended answers ARE the engine's defaults**, and this
    /// test is the guard on the day that stops being true.
    ///
    /// Its predecessor asserted the opposite and failed twice in one evening,
    /// which is the whole reason this one is worded as it is. See
    /// [`super::apply_pdfcer`] for what those two failures taught.
    ///
    /// If a real divergence is ever added, this fails — and that failure is the
    /// prompt to state the reason in `apply_pdfcer`'s doc comment *before*
    /// amending the assertion, so an override always arrives with its
    /// justification attached rather than as a bare line somebody later deletes
    /// as redundant.
    #[test]
    fn the_recommended_preset_is_the_engines_defaults() {
        let mut ours = Settings::default();
        apply_pdfcer(&mut ours);
        assert!(
            same(&ours, &Settings::default()),
            "pdfcer-gui now overrides one of the engine's rendering defaults. \
             That may be right — but say WHY in `apply_pdfcer`'s doc comment \
             before changing this assertion, or the next reader cannot tell a \
 deliberate override from a stale restatement"
        );
    }

    /// **A changed setting matches no preset**, rather than the nearest one.
    #[test]
    fn adjusted_settings_match_nothing() {
        let mut s = Settings::default();
        apply_pdfcer(&mut s);
        assert!(matching(&s).is_some());
        s.mask_resample = pdfcer_core::settings::MaskResample::Bilinear;
        assert_eq!(
            matching(&s),
            None,
            "an operator who has adjusted a render setting is on no preset, and \
 the control must say so rather than claiming the closest one"
        );
    }

    /// ★★★ **The theme is not part of a preset**, because a preset is about how
    /// a DOCUMENT renders and the theme is about the program.
    #[test]
    fn choosing_a_theme_does_not_leave_the_preset() {
        let mut s = Settings::default();
        apply_pdfcer(&mut s);
        let before = matching(&s);
        s.theme = "dark".to_owned();
        assert_eq!(
            matching(&s),
            before,
            "picking a dark theme is not a rendering decision, and must not \
 clear a rendering preset"
        );
    }

    /// **Every choice has a distinct copy and a distinct id.**
    #[test]
    fn presets_are_distinct() {
        let all = choices();
        for (i, a) in all.iter().enumerate() {
            for b in all.iter().skip(i + 1) {
                assert_ne!(a.id(), b.id());
                assert_ne!(a.label(), b.label());
            }
        }
    }

    /// ★★ **Every key a standard leaves alone is named in the operator's own
    /// words**, not in the engine's field identifiers.
    ///
    /// `mesh_patch_padding` is correct for an API and wrong for a window. The
    /// fallback arm exists so a key the engine adds later degrades to an
    /// unfamiliar identifier — ugly and honest — rather than to a guessed
    /// title, which would be a sentence pdfcer never wrote appearing under a
    /// standard's name.
    ///
    /// This fails when the engine adds a `PresetKey`, which is the point: the
    /// failure is the prompt to write a title for it.
    #[test]
    fn every_key_a_standard_leaves_alone_has_an_operator_facing_title() {
        let mut seen = 0_usize;
        for standard in RenderStandard::all() {
            for key in RenderPreset::for_standard(*standard).left_alone() {
                seen += 1;
                assert_ne!(
                    operator_title(key),
                    key.as_str(),
                    "`{}` is shown to the operator with its ENGINE key name. Give it a title in `text::settings` and add the arm to `operator_title`",
                    key.as_str()
                );
            }
        }
        assert!(
            seen > 0,
            "no standard left any key alone, so this test asserted nothing — either the engine changed shape or the presets are not loading"
        );
    }

    /// **The weight line adds up to the answers that carry a value.**
    ///
    /// ★ `NotApplicable` is deliberately excluded from the tally and named in
    /// the left-alone list instead. *"Does not apply"* is a different kind of
    /// fact from *"we chose"*, and adding them together is the arithmetic that
    /// would make a preset look better sourced than it is.
    #[test]
    fn the_weight_tally_excludes_what_the_standard_does_not_reach() {
        for standard in RenderStandard::all() {
            let preset = RenderPreset::for_standard(*standard);
            let graded = preset
                .entries()
                .iter()
                .filter(|e| !matches!(e.evidence, Evidence::NotApplicable))
                .count();
            let left = preset.left_alone().len();
            let not_applicable = preset
                .entries()
                .iter()
                .filter(|e| matches!(e.evidence, Evidence::NotApplicable))
                .count();
            assert!(
                left >= not_applicable,
                "{}: {not_applicable} entries are marked not-applicable but only {left} keys are reported as left alone, so the summary would count an axis twice",
                standard.as_str()
            );
            let _ = graded;
        }
    }

    /// ★★★ **Every preset in the list can be selected**, which is the defect
    /// the operator reported as *"I can only select (ISO15930-1, -4)"*.
    ///
    /// Drives the real rule — click a radio, then ask what the control would
    /// show — for all ten choices. Before the fix, eight of them answered
    /// `pdf-x1a` and one answered `pdfcer`, because `matching` returns the
    /// FIRST choice whose settings equal the current ones and all eight
    /// conformance presets apply byte-identical settings.
    ///
    /// ★★ It asserts the property the operator cares about — *can I choose
    /// this?* — rather than the mechanism. A test that asserted
    /// `chosen_preset == Some(id)` would pass against a version that stored the
    /// choice and still drew the dot somewhere else.
    #[test]
    fn every_preset_in_the_list_can_actually_be_selected() {
        for c in choices() {
            let mut draft = Draft::new(&Settings::default(), &crate::app::prefs::Prefs::default());
            // Exactly what `row` does on a click, in the same order.
            c.apply(&mut draft.working);
            draft.chosen_preset = Some(c.id());
            assert_eq!(
                live_choice(&draft),
                Some(c.id()),
                "choosing {:?} left the control showing something else — the operator \
                 cannot express this choice",
                c.label()
            );
        }
    }

    /// ★★★ **The operator's report, as an assertion: choosing a standard must
    /// make Save live.**
    ///
    /// > *"When I go to settings and select some of the standards the save
    /// > button is greyed out and I can't save the change."* — 2026-08-26
    ///
    /// Both halves were true and the second explains the first. `is_dirty`
    /// compares values, and [`identical_siblings`] measures that **all eight
    /// PDF/X and PDF/A presets apply byte-identical render settings** — so
    /// choosing a second one moved nothing and Save was correctly greyed about
    /// a draft that really did equal what was saved.
    ///
    /// ★ The test is written over **every pair** of choices rather than the two
    /// that were reported, because the reported pair is not special: any two of
    /// the eight reproduce it. Picking two would have passed the day a ninth
    /// standard arrived and collided with a tenth.
    #[test]
    fn choosing_any_standard_after_another_leaves_something_to_save() {
        let all = choices();
        for first in &all {
            for second in &all {
                if first.id() == second.id() {
                    continue;
                }
                let mut draft =
                    Draft::new(&Settings::default(), &crate::app::prefs::Prefs::default());
                // Choose one, save it — i.e. start the second sitting from the
                // state the first one left.
                first.apply(&mut draft.working);
                draft.chosen_preset = Some(first.id());
                draft.working_prefs.chosen_standard = Some(first.id().to_owned());
                let saved = draft.working.clone();
                let saved_prefs = draft.working_prefs.clone();

                let mut draft = Draft::new(&saved, &saved_prefs);
                second.apply(&mut draft.working);
                draft.chosen_preset = Some(second.id());
                draft.working_prefs.chosen_standard = Some(second.id().to_owned());

                assert!(
                    draft.is_dirty(),
                    "choosing `{}` after `{}` left nothing to save. These two apply the same \
                     render settings, so the values cannot carry the difference — the CHOICE \
                     has to be part of what `is_dirty` compares.",
                    second.id(),
                    first.id()
                );
            }
        }
    }

    /// ★★ **A choice survives the window being closed and reopened.**
    ///
    /// The half the operator had not seen yet. Before this was persisted the
    /// window showed whichever standard `matching` found first, so choosing
    /// PDF/X-4 and coming back read PDF/X-1a — the window contradicting the
    /// operator about what they had asked for.
    ///
    /// Asserted for **every** standard, since the failure is invisible for
    /// whichever one happens to sort first.
    #[test]
    fn a_chosen_standard_is_what_the_window_shows_when_it_reopens() {
        for choice in choices() {
            let mut draft = Draft::new(&Settings::default(), &crate::app::prefs::Prefs::default());
            choice.apply(&mut draft.working);
            draft.chosen_preset = Some(choice.id());
            draft.working_prefs.chosen_standard = Some(choice.id().to_owned());

            // A new sitting, seeded the way `Draft::new` seeds it from disk.
            let reopened = Draft::new(&draft.working, &draft.working_prefs);
            assert_eq!(
                live_choice(&reopened),
                Some(choice.id()),
                "the window reopened showing a different standard from the one that was chosen"
            );
        }
    }

    /// ★★ **A stored choice the settings no longer express is retired, not
    /// shown.**
    ///
    /// The guard that stops persistence turning into a lie. `live_choice`
    /// already declines to show it; this asserts the same of
    /// [`still_chosen`], which is what `commit` asks before writing — because
    /// a window that stopped claiming a standard while the file went on naming
    /// it would reopen claiming it again.
    #[test]
    fn a_hand_edited_setting_retires_the_stored_choice_as_well_as_the_shown_one() {
        let x4 = choices()
            .into_iter()
            .find(|c| matches!(c, Choice::Standard(_)))
            .expect("the engine offers at least one standard");
        let mut draft = Draft::new(&Settings::default(), &crate::app::prefs::Prefs::default());
        x4.apply(&mut draft.working);
        draft.chosen_preset = Some(x4.id());
        draft.working_prefs.chosen_standard = Some(x4.id().to_owned());
        assert!(
            still_chosen(&draft),
            "the choice holds before anything moves"
        );

        // Move a value the preset governs, by hand.
        draft.working.cmyk_intent = match draft.working.cmyk_intent {
            pdfcer_core::settings::CmykIntent::Calibrated => {
                pdfcer_core::settings::CmykIntent::NeutralBlack
            }
            _ => pdfcer_core::settings::CmykIntent::Calibrated,
        };

        assert!(
            !still_chosen(&draft),
            "a hand-edited setting must retire the stored choice, or the file keeps a claim the \
             settings contradict"
        );
        assert_ne!(
            live_choice(&draft),
            Some(x4.id()),
            "and the window must stop showing it, which is the half that already worked"
        );
    }

    /// An id this build does not know resolves to nothing and is not an error.
    ///
    /// Reachable two ways: a hand-edited preferences file, and a file written
    /// by a newer pdfcer that knows a standard this build does not. Neither is a
    /// fault in the file, so neither may become a parse note — the window falls
    /// back to the derived reading, which is the honest answer for a choice it
    /// cannot offer.
    #[test]
    fn an_unknown_stored_standard_falls_back_rather_than_failing() {
        assert_eq!(resolve_id(Some("pdf-x99-not-a-standard")), None);
        assert_eq!(resolve_id(Some("")), None);
        assert_eq!(resolve_id(None), None);
        let known = choices()[0].id();
        assert_eq!(resolve_id(Some(known)), Some(known));
    }

    /// ★★ **Adjusting a control by hand drops the chosen preset**, so the dot
    /// cannot go on claiming a standard the settings no longer describe.
    ///
    /// The other half of the fix, and the reason the choice is filtered through
    /// `still_holds` rather than simply believed. Without this a window could
    /// read *"PDF/X-4"* over settings that are nobody's.
    #[test]
    fn changing_a_setting_by_hand_retires_the_chosen_preset() {
        let mut draft = Draft::new(&Settings::default(), &crate::app::prefs::Prefs::default());
        let x4 = choices()
            .into_iter()
            .find(|c| c.id() == "pdf-x4")
            .expect("pdf-x4 must be offered"); // ui-text-exempt: test panic
        x4.apply(&mut draft.working);
        draft.chosen_preset = Some(x4.id());
        assert_eq!(live_choice(&draft), Some("pdf-x4"));

        // The operator turns image smoothing back on, one screen down.
        draft.working.image_minify = pdfcer_core::settings::MinifyFilter::Smooth;
        assert_ne!(
            live_choice(&draft),
            Some("pdf-x4"),
            "the settings are no longer PDF/X-4's, so the control must not say they are"
        );
    }

    /// ★★★ **The eight conformance presets really are identical today** — the
    /// measurement the disclosure is built on, pinned so it cannot rot.
    ///
    /// This is not asserting that they SHOULD be identical. It records what is
    /// true of the engine this build links, so that if a standard's answers
    /// ever diverge, this test fails and whoever reads it learns that the
    /// sentence under the radio has become interesting rather than routine.
    ///
    /// ★ It is also the falsification for the test above: with the presets all
    /// distinct, `every_preset_in_the_list_can_actually_be_selected` would pass
    /// against the OLD code, and would have proved nothing.
    #[test]
    fn the_conformance_presets_give_the_same_render_answers_today() {
        let standards: Vec<Choice> = choices()
            .into_iter()
            .filter(|c| matches!(c, Choice::Standard(_)))
            .collect();
        assert!(
            standards.len() >= 8,
            "the engine offers fewer standards than it did"
        );

        let identical = standards
            .iter()
            .filter(|c| identical_siblings(c.id()) > 0)
            .count();
        assert!(
            identical >= 8,
            "only {identical} preset(s) share their answers with another. If the standards \
             have started to differ, that is good news — update the disclosure's reasoning \
             and this expectation, and check the operator is told what changed"
        );
    }
}
