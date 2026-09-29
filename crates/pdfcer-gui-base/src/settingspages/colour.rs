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
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/settingspages/colour.md`.

use egui::Ui;
use pdfcer_core::settings::{CmykIntent, CmykJpegPolarity, MeshPatchPadding, PageBlendSpaceSource};

use super::{Draft, widgets};
use crate::text::settings as t;

/// How CMYK ink becomes screen colour.
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
    // TWO options now, and the third is gone with its copy and — once the
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
    // AND THE DIVERGENCE NOTE IS DELETED RATHER THAN REWORDED.
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
pub fn zero_tint(ui: &mut Ui, draft: &mut Draft) {
    use pdfcer_core::settings::OverprintZeroTintScope as Scope;
    widgets::header(
        ui,
        t::zero_tint_title(),
        t::zero_tint_silence(),
        t::zero_tint_radius(),
    );
    // THE DEFAULT FIRST, and it is now ASKED rather than assumed.
    //
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
pub fn spot_model(ui: &mut Ui, draft: &mut Draft) {
    use pdfcer_core::settings::SpotColorantDeviceModel as Model;
    widgets::header(
        ui,
        t::spot_model_title(),
        t::spot_model_silence(),
        t::spot_model_radius(),
    );
    // Default first, as every radio in this window is — an operator reads
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

/// **How much memory ink blending may use** — the ceiling that decides
/// whether a page's colours change with the zoom.
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
