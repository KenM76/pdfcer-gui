//! # `app::actions::fonts` — the document-level font verbs
//!
//! Two: embedding the font programs a document references but does not carry,
//! and removing the ones it does.
//!
//! ## ★★ Why this is its own module rather than an arm in `apply`
//!
//! Not size — `apply.rs` had room. It is that a font edit is the one mutation
//! in this shell whose **operand was resolved outside the engine**: the donor
//! bytes came off the operator's disk, chosen by `app::fonts` against a folder
//! list the operator maintains. Every other verb here takes an operand the
//! document already contained.
//!
//! That has a consequence worth keeping next to the code: the shell is
//! responsible for the honesty of the match, and `pdfcer-core` is explicit that
//! it will not check. `SuppliedFont::matched` is *"the inference rule 4
//! governs"*, and the engine's symbolic guard turns on it — so a shell that
//! reported every donor as `Exact` would disable a correctness check in the
//! engine from the outside. `dialogs::embed` carries that mapping and this
//! module carries the reason it must not be moved.
//!
//! ## ★ The plan comes back from the commit, and it is the reported one
//!
//! `embed_fonts` returns the `EmbedPlan` it acted on rather than a count, and
//! the disclosure is built from *that* value, never from the one the dialog
//! showed. The two are computed by the same function from the same request, so
//! they will agree — and building the report from the returned one means they
//! cannot silently stop agreeing.

use crate::app::state::OpenDoc;

/// **Embed every font the request names, as one undoable command.**
///
/// ★ No pre-flight refusal check here. `embed_fonts` runs `embed_refusal`
/// itself *"before any mutation"* and returns the refusal as an `Err`, so
/// calling it first would be a second implementation of a guard the engine
/// already owns — the failure `dispatch::routes`' header names in a different
/// register.
pub(super) fn embed(doc: &mut OpenDoc, request: &pdfcer_core::font_embed_missing::EmbedRequest) {
    let supplied = request.supplied.len();
    super::apply::vector_edit(doc, "embed-fonts", 0, supplied, |session| {
        session.embed_fonts(request).map(|plan| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    // `-applied`, per the convention `forms::import_data`
                    // records at length: `vector_edit` writes its own
                    // `embed-fonts …` line for the same edit, and a driven
                    // check taking `.last()` on a shared name reads the wrong
                    // one and then reports failure about a gesture that worked.
                    "embed-fonts-applied embedded={} bytes={} missing_before={} \
                     missing_after={} substituted={}",
                    plan.targets.len(),
                    plan.bytes_added_uncompressed(),
                    plan.missing_before,
                    plan.missing_after(),
                    plan.substitutes_any()
                )
            });
            crate::text::embed::embedded_disclosure(
                plan.targets.len(),
                plan.missing_after(),
                plan.substitutes_any(),
            )
        })
    });
}

/// **Remove every embedded font program the request names, as one undoable
/// command.**
///
/// ★ No pre-flight refusal check, for [`embed`]'s reason: `unembed_fonts` runs
/// `unembed_refusal` itself before mutating.
///
/// ★★ And **no PDF/A gate here either**, deliberately. The engine leaves PDF/A
/// out of its refusal and says why: *"unembedding genuinely breaks that
/// conformance … but it is a consequence the operator may knowingly accept, not
/// a structural impossibility. The core reports it and **the shells gate on
/// it**."* This shell's gate is the sentence in `dialogs::unembed`, which the
/// operator reads before pressing the button — a disclosure, not a refusal,
/// because the decision is theirs and the engine says so.
pub(super) fn unembed(doc: &mut OpenDoc, request: &pdfcer_core::font_unembed::UnembedRequest) {
    super::apply::vector_edit(doc, "unembed-fonts", 0, 1, |session| {
        session.unembed_fonts(request).map(|plan| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    // `-applied`, per the convention `forms::import_data`
                    // records: `vector_edit` writes its own bare-named line for
                    // the same edit, and `.last()` on a shared name reads the
                    // wrong one.
                    "unembed-fonts-applied removed={} blocked={} bytes={} renamed={}",
                    plan.targets.len(),
                    plan.blocked.len(),
                    plan.bytes_reclaimable(),
                    plan.renames_any()
                )
            });
            crate::text::unembed::removed_disclosure(
                plan.targets.len(),
                plan.bytes_reclaimable(),
                plan.renames_any(),
            )
        })
    });
}
