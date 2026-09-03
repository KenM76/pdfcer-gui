//! # `dialogs::settings::colour` — the two questions about ink
//!
//! The group that **starts expanded**, because it holds the setting most likely
//! to have brought someone to this window: *"my black lines look grey."*
//!
//! It is also the only group containing a default that **knowingly departs from
//! what Acrobat and pdfium do**, on an explicit operator ruling — and that
//! departure is disclosed at the setting rather than in a footnote, because the
//! person reading this radio group is precisely the person who has noticed the
//! difference and is deciding whether it is a bug.

use egui::Ui;
use pdfcer_core::settings::{CmykIntent, CmykJpegPolarity, MeshPatchPadding, PageBlendSpaceSource};

use super::{Draft, widgets};
use crate::text::settings as t;

/// How CMYK ink becomes screen colour.
///
/// # The default is a deliberate divergence, and the order says so
///
/// By the standing rule the default here would be `Calibrated`: that is tier
/// (a)/(c) evidence — Acrobat's shipped profile *and* pdfium both produce it —
/// which is the strongest evidence behind any default in this window. It is
/// `NeutralBlack` anyway, because the operator looked at what calibrated
/// rendering does to pure-K line art and overruled it.
///
/// **The default is listed first**, ahead of the better-sourced option, and
/// that ordering is the argument: an operator scanning this group should see
/// what pdfcer is currently doing before they see the alternatives. Every other
/// setting in the window lists its default first for the same reason, so the
/// one place it would be tempting to make an exception is the one place the
/// consistency matters most.
///
/// The divergence is narrow by construction — only the pure-K axis moves, every
/// mixed colour still uses the measured table — and the option note says so,
/// because an operator worrying that pdfcer has invented its own colour science
/// deserves to be answered from the window rather than from the source.
pub fn intent(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::cmyk_intent_title(),
        t::cmyk_intent_silence(),
        t::cmyk_intent_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.cmyk_intent,
        CmykIntent::NeutralBlack,
        t::cmyk_intent_neutral_label(),
        Some(t::cmyk_intent_neutral_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.cmyk_intent,
        CmykIntent::Calibrated,
        t::cmyk_intent_calibrated_label(),
        Some(t::cmyk_intent_calibrated_note()),
    );
    // ★★★ TWO options now, and the third is gone with its copy and — once the
    // engine lands it — its arithmetic. `OPERATOR_REQUESTS.md` **O52**:
    // *"you can also remove the The old pdfcer formula from that section, even
    // the code for it."*
    //
    // It was offered for one purpose, stated in its own note: *"only useful for
    // comparing against something pdfcer produced earlier."* That was true while
    // pdfcer had recently produced such files. It is a control whose entire
    // justification expires with time, which is a shape worth naming — nothing
    // fails when it stops being useful, so nothing prompts anybody to remove it.
    //
    // ★★ AND THE DIVERGENCE NOTE IS DELETED RATHER THAN REWORDED.
    //
    // It said *"pdfcer's default deliberately differs from Acrobat here"*, and
    // it existed so a future session would not investigate a render-parity
    // difference as a defect. With the default now MATCHING, that sentence is
    // not redundant — it is **backwards**, and a reworded version would be a
    // second copy of a fact the radio group already states by which button is
    // selected.
    //
    // => A note explaining a divergence must die with the divergence. Keeping
    // it "for context" is how a window comes to describe a program that no
    // longer exists.
}

/// **Which colours get overprint's zero-tint rule** — `Pass 143.0`.
///
/// ## ★★ Why it sits with the blend-space setting and not on its own
///
/// Because it is meaningless without it. `page_blend_space` decides *whether a
/// page is composited in ink at all*; this decides *which source colours the
/// ink rules then apply to*. An operator who has the first switched off will
/// never see this one do anything, and the window's ordering rule — read
/// downward and each setting has the context of the one above — is what makes
/// that legible without a cross-reference.
///
/// ## ★ It is a spec ambiguity, not a preference, and the copy says so
///
/// §8.6.7 scopes the zero-tint rule to `DeviceCMYK`. `DeviceGray` is not one,
/// so pdfcer's original literal reading was defensible; Acrobat converts grey to
/// K-only CMYK first and *then* applies the rule, which is equally defensible
/// and is what the print-conformance suite is scored against. The default
/// follows the instrument.
///
/// ⇒ The third option is **unmeasured** and its note says so in those terms.
/// A radio group that presents a guess and a measurement in the same voice is
/// asking the operator to trust three things equally when only two earned it.
pub fn zero_tint(ui: &mut Ui, draft: &mut Draft) {
    use pdfcer_core::settings::OverprintZeroTintScope as Scope;
    widgets::header(
        ui,
        t::zero_tint_title(),
        t::zero_tint_silence(),
        t::zero_tint_radius(),
    );
    // ★★★ THE DEFAULT FIRST, and it is now ASKED rather than assumed.
    //
    // `intent`'s argument is unchanged — an operator reads what pdfcer is doing
    // now before they read the alternatives — but the list used to hard-code
    // which one that was, and on 2026-09-03 the engine moved it. Sorting by
    // `OverprintZeroTintScope::default()` means the ordering follows the engine
    // instead of having to be remembered, and the "(pdfcer's default)" marker
    // comes from the same question rather than from a label.
    let mut scopes = [
        Scope::GreyAsKOnly,
        Scope::DeviceCmykOnly,
        Scope::AllProcessSpaces,
    ];
    scopes.sort_by_key(|s| u8::from(*s != Scope::default()));
    for scope in scopes {
        widgets::option(
            ui,
            &mut draft.working.overprint_zero_tint_scope,
            scope,
            // ui-text-exempt: the two halves are catalog strings; this line
            // only joins them, and the join is punctuation.
            &format!(
                "{}{}",
                t::zero_tint_label(scope),
                t::zero_tint_default_suffix(scope)
            ),
            Some(t::zero_tint_note(scope)),
        );
    }
}

/// **Whether a spot ink keeps its own plate, or is mixed down first.**
///
/// ★ New in `pdfcer-core 0.20` (`OPERATOR_REQUESTS.md` O100), and placed
/// immediately after [`zero_tint`] because the two are the same subject seen
/// from two sides: that one is *what overprints a spot colour*, this one is
/// *what a spot colour IS*. An operator who has arrived at either has arrived
/// because white behaved unexpectedly on a print-ready drawing, and finding
/// only one of them would leave them thinking they had exhausted the options.
///
/// [`crate::text::settings::spot_model_title`] carries why both values are
/// conformant and what the visible difference is.
pub fn spot_model(ui: &mut Ui, draft: &mut Draft) {
    use pdfcer_core::settings::SpotColorantDeviceModel as Model;
    widgets::header(
        ui,
        t::spot_model_title(),
        t::spot_model_silence(),
        t::spot_model_radius(),
    );
    // ★ Default first, as every radio in this window is — an operator reads
    // what pdfcer is doing now before they read the alternative.
    for model in [
        Model::SimulateSeparations,
        Model::AlternateSpaceSubstitution,
    ] {
        widgets::option(
            ui,
            &mut draft.working.spot_colorant_device_model,
            model,
            t::spot_model_label(model),
            Some(t::spot_model_note(model)),
        );
    }
}

/// Whether a CMYK JPEG's ink values are stored inverted.
///
/// # ★ The one well-sourced default in the whole window
///
/// Every other default here is *reasoned inference* — a guess — and says so.
/// This one is not, and says that instead: `"invert"` occurs **zero times** in
/// the Adobe technical note ISO 32000-1 makes normative, the APP14 marker
/// carries **no polarity flag at all** (so "invert on marker" keys off mere
/// presence), and all four reference engines accept the ambiguity rather than
/// inverting.
///
/// Keeping the two claims distinguishable is the point. "pdfcer matched every
/// other implementation" and "pdfcer guessed" must not read alike, or the
/// operator has no way to tell which of thirteen defaults to trust — and the
/// catalog has a test for each direction.
///
/// # It is also the only preview setting that can change saved bytes
///
/// A re-encode under the wrong polarity bakes the inversion in **permanently**,
/// which is why its radius line names the saved file and the other four preview
/// settings' do not.
pub fn polarity(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::polarity_title(),
        t::polarity_silence(),
        t::polarity_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.cmyk_jpeg_polarity,
        CmykJpegPolarity::NeverInvert,
        t::polarity_never_label(),
        Some(t::polarity_never_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.cmyk_jpeg_polarity,
        CmykJpegPolarity::InvertOnApp14,
        t::polarity_invert_label(),
        Some(t::polarity_invert_note()),
    );
}

/// Where a page's blending colour space comes from when its own group
/// declares none — the engine's `PGB-A1`, and the setting that decides
/// whether **overprint** is simulated at all.
///
/// # Why this belongs in the Colour group and not in Images
///
/// Because of the symptom that sends somebody looking for it. It is not an
/// image question; it is *"the overprinted areas in my print file look wrong"*
/// — or, from the other direction, *"this file renders differently in pdfcer
/// than it did last month."* Both are ink questions, and this is the ink
/// group.
///
/// # What the operator is actually choosing between
///
/// ISO 32000-1 §11.4.7 says a page with no declared blending space uses the
/// **device's native** space. pdfcer's pixmap is RGBA8, which is additive — and
/// in an additive space overprint is not merely unsimulated, it is
/// **unrepresentable**: §11.7.4.3 makes the blend function return the source
/// colour for every component *"specified in the current colour space"*, and
/// in sRGB every component is always specified. The engine measured the
/// consequence on the industry print-conformance suite it measures itself
/// against: **24 of its 51 patches request overprint**, and under the literal
/// reading all 24 are wrong. (The suite is licensed and is deliberately not
/// named in this repository — operator ruling, 2026-08-25, enforced by
/// `tools/check-suite-name-absent.py`. It is named in full in the private map
/// directory, which is where a reader who needs it should look.)
///
/// So the shipped default consults the file's **output intent**, but only when
/// that intent is subtractive. That conditional is what makes it safe: an RGB
/// or greyscale intent cannot drag a page into ink, so the only files it moves
/// are ones that already declare themselves destined for print.
///
/// ★ The three options are ordered **strict → shipped → most literal reading
/// of Annex P**, and the middle one is the default. That is deliberate: an
/// operator scanning this group meets the conforming-but-degenerate answer
/// first, so *why is the default not the one the standard says* is answered
/// before it is asked.
pub fn page_blend_space(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::blend_space_title(),
        t::blend_space_silence(),
        t::blend_space_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.page_blend_space_source,
        PageBlendSpaceSource::DeviceNative,
        t::blend_space_label(PageBlendSpaceSource::DeviceNative),
        Some(t::blend_space_note(PageBlendSpaceSource::DeviceNative)),
    );
    widgets::option(
        ui,
        &mut draft.working.page_blend_space_source,
        PageBlendSpaceSource::OutputIntentIfSubtractive,
        t::blend_space_label(PageBlendSpaceSource::OutputIntentIfSubtractive),
        Some(t::blend_space_note(
            PageBlendSpaceSource::OutputIntentIfSubtractive,
        )),
    );
    widgets::option(
        ui,
        &mut draft.working.page_blend_space_source,
        PageBlendSpaceSource::OutputIntentAlways,
        t::blend_space_label(PageBlendSpaceSource::OutputIntentAlways),
        Some(t::blend_space_note(
            PageBlendSpaceSource::OutputIntentAlways,
        )),
    );
}

/// ★★ **How much memory ink blending may use** — the ceiling that decides
/// whether a page's colours change with the zoom.
///
/// # Why it is in Colour, and why its title is not "buffer"
///
/// The symptom that sends somebody looking for it, in the operator's own words:
///
/// > *"seems I get different results depending on Zoom level … up to 474 % they
/// > are mismatched, but at 579 % they match."*
///
/// That is an ink question, so it is in the ink group, beside the two settings
/// that decide *whether* a page is blended in ink at all. And it is titled
/// *"Colours changing when you zoom"* rather than anything about a buffer,
/// because nobody has ever gone looking for a buffer.
///
/// # The only free-text control in this window
///
/// Every other setting here is a choice among named options. This one is a
/// quantity, and the operator asked for it to be a quantity *"up to the maximum
/// possible"* — so it is [`widgets::text_value`], uncapped, with the cost
/// stated and no guard. That is the same ruling that governs the maximum zoom,
/// and the engine built its side to match: the allocation is fallible, so a
/// ceiling this machine cannot honour refuses down the ordinary disclosed path
/// rather than aborting. **State the cost; do not prevent the choice.**
///
/// ★ It parses with `pdfcer_core::settings::parse_byte_size` and displays with
/// `format_byte_size` — the engine's own pair, which is what `settings.txt`
/// uses. So the window and the file accept and show identical strings, and
/// `256mb` and `256mib` both mean 1,048,576 × 256 here, deliberately: every
/// figure pdfcer reports about this buffer is binary, and an operator who types
/// `512mb` after reading "256 MB" should get double rather than 488 MiB.
pub fn cmyk_ceiling(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::cmyk_ceiling_title(),
        t::cmyk_ceiling_silence(),
        t::cmyk_ceiling_radius(),
    );
    widgets::text_value(
        ui,
        "settings.colour.cmyk_ceiling",
        &mut draft.working.max_cmyk_buffer_bytes,
        t::cmyk_ceiling_label(),
        Some(t::cmyk_ceiling_note()),
        |v| pdfcer_core::settings::format_byte_size(*v),
        |s| pdfcer_core::settings::parse_byte_size(s).ok(),
    );
}

/// How a mesh-shading patch record is byte-padded (spec ambiguity `MSH-A1`).
///
/// # Why this is in Colour and not in Images
///
/// Because the thing that goes wrong is a **gradient**, and a gradient is
/// colour. The mechanism is a bit-alignment question about a binary stream,
/// which would file it under nothing an operator can name; the symptom is
/// *"this smooth fill came out as noise"*, and that is what the title says.
///
/// # ★ It is observable in very few files, and the note says so honestly
///
/// The two readings agree unless `BitsPerFlag + k·BitsPerCoordinate +
/// m·BitsPerComponent` fails to be a multiple of 8. Every combination the
/// engine has measured in real files — 8/32/8 in the print-conformance suite's
/// type 7 meshes, and the common 8/16/8 — is byte-aligned for every record
/// shape, so the two render identically. A file with `BitsPerFlag` 2 or 4, or
/// 12-bit coordinates, is where they diverge, **and there the divergence is
/// total**: one record out of step desynchronises every record after it.
///
/// That is why the second option's note is phrased as a remedy to try rather
/// than as a preference to hold — an operator will only ever reach this control
/// because something on screen is already wrong.
pub fn mesh_patch_padding(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::mesh_padding_title(),
        t::mesh_padding_silence(),
        t::mesh_padding_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.mesh_patch_padding,
        MeshPatchPadding::PerRecord,
        t::mesh_padding_record_label(),
        Some(t::mesh_padding_record_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.mesh_patch_padding,
        MeshPatchPadding::None,
        t::mesh_padding_none_label(),
        Some(t::mesh_padding_none_note()),
    );
}
