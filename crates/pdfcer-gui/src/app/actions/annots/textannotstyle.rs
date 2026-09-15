//! # `app::actions::annots::textannotstyle` — restyling a text-BEARING annotation
//!
//! One verb, `set_text_annot_style`, and the disclosure it owes.
//!
//! ## Why it is its own file rather than one more arm in `annots.rs`
//!
//! `annots.rs` is the file for *"what happens to an annotation that already
//! exists"* — delete it, move it, resize it, reshape it, restyle it — and by
//! that description this verb belongs there. Two things say otherwise.
//!
//! It is the only verb in that file whose subject is a **second spec family**:
//! everything else routes through `annot_author`'s markup spec, and this one
//! routes through the text spec, which is why it exists as a separate engine
//! verb at all rather than as a branch. And it is the only one that
//! **decides what to tell the operator** — the rest report nothing, or report
//! a `dropped` catalogue the funnel already knows how to render. A subject
//! plus a judgement is a module; a routing arm is not.
//!
//! ## Where its siblings are
//!
//! [`crate::app::actions::textannot`] authors these same kinds and makes the
//! same class of disclosure at placing time, over the same engine outcome.
//! The two files are the *before* and *after* of one capability and read best
//! together; the strings both of them speak are in
//! [`crate::text::panels::textannotstyle`], one sentence per act, so the two
//! surfaces cannot drift into wording the operator has to reconcile.

use pdfcer_core::annot_author::StampLabelFit;
use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;
use crate::text::panels::textannotstyle as ts;

/// **Restyle a text-BEARING annotation** — a sticky note's icon and colour, a
/// stamp's colour. `EditSession::set_text_annot_style`.
///
/// # Why this is a second function beside the markup restyle, not a branch
/// # inside it
///
/// Because it is a second **verb over a second spec family**, and the engine's
/// own doc on `edit::TextAnnotStyle` is the argument:
///
/// > `MarkupStyle` reaches its annotation through
/// > `annot_author::spec_from_dict`, whose arms are the geometric family and
/// > the four text markups. **There is no `/Text` arm** … So the two verbs are
/// > not a split anyone chose for tidiness — they read through different
/// > functions because the two families are modelled by different spec types.
///
/// A single body with an `if subtype == "Text"` inside it would put the
/// routing decision where nothing checks it. It is a `match` instead, on
/// `crate::panels::properties::markup::textannot::Reach`, decided in the panel
/// by asking each of the two engine readers in turn — and this function is only
/// ever reached down one arm of it.
///
/// # The page is `0`, and that is not a defect
///
/// `set_text_annot_style` takes an `ObjId` and nothing else — the property that
/// puts this variant in `AnnotAction` at all — so there is no page to pass.
/// [`clear_note`] and the node verbs pass `0` for the identical reason, and the
/// funnel uses the argument for its undo label and its raster invalidation
/// rather than to find anything. `EditScope::Document` (the plain
/// [`crate::app::actions::apply::vector_edit`], not the `_on_page` twin) is right for the
/// same reason it is right for a note: the annotation may not be on the page
/// the view is showing, and a `/Popup` companion may not be on its own.
///
/// # What the trace carries, and why it is not the values
///
/// `icon_written` and `color_written` — the engine's own two booleans off
/// [`pdfcer_core::edit::TextAnnotStyleChange`] — plus how the appearance was
/// written. **Not the icon name**, and that is deliberate: the icon is
/// invisible in pdfcer's own picture by construction
/// (`annot_author::sticky_note` draws one marker for all seven), so what a
/// driven check needs is *did `/Name` change at all*, which no screenshot can
/// answer. A name in the trace would only restate what the panel already
/// displays.
///
/// # The undo entry it pushes
///
/// `pdfcer_core::edit::CommandKind::SetTextAnnotStyle` — the engine's own
/// label for this command, pushed by the verb itself rather than by the funnel,
/// so an undo of a restyle is one entry and names the act rather than the
/// appearance rewrite underneath it.
///
/// # It has a disclosure list, and what is in it
///
/// Nothing is quietly *lost* here: unlike `set_markup_style` this verb
/// re-bakes from a spec the same reader hands the authoring path, so there is
/// no `dropped` catalogue to render.
///
/// What it can do instead is **decide**. Changing a stamp's label size means a
/// re-bake at a new size, and that can widen the box, shrink the words, or cut
/// characters off the ends. Every one of those is an inference, and the ones a
/// screenshot cannot show owe a sentence off-canvas under R8b rule 4. So the
/// funnel's `Vec<String>` is not empty, and what goes in it is decided below
/// on `TextAnnotStyleChange::stamp_label_fit`.
///
/// **A "the engine cannot do this" sentence in a shell comment is a claim with
/// a shelf life and no gate.** Nothing in this build fails when the engine
/// grows the capability, so every such sentence here is re-checked against the
/// engine rather than trusted.
pub(in crate::app::actions) fn set_text_annot_style(
    doc: &mut OpenDoc,
    id: ObjId,
    style: &pdfcer_core::edit::TextAnnotStyle,
) {
    crate::app::actions::apply::vector_edit(doc, "set-text-annot-style", 0, 1, |session| {
        session.set_text_annot_style(id, style).map(|change| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // `was_foreign=` is traced and NOT yet disclosed, and that
                // is a gap rather than a ruling.
                //
                // The engine's field reports that the annotation's previous
                // appearance was one pdfcer would not have drawn — a
                // designer's stream with a shadow or a gradient, or a `/Stamp`
                // whose label pdfcer could not recover from its own artwork —
                // and that re-baking has just replaced it with pdfcer's
                // plainer rendering. That is exactly the class of thing R8b
                // rule 4 says must reach the operator off-canvas.
                //
                // It is `false` for a `/Text` sticky note, whose appearance is
                // icon-driven. It is `true` for a `/FreeText`, which
                // `panels::properties::markup::textannot` declines by name —
                // and `true` for a `/Stamp` the engine could not read back,
                // which that panel does NOT decline. An imported custom stamp
                // restyled here loses its artwork, and today only the trace
                // says so.
                //
                // So a driven run showing `was_foreign=1` is the tripwire: it
                // means an operator-facing sentence is owed and the
                // disclosure list below does not carry one.
                format!(
                    "set-text-annot-style-applied id={} subtype={} icon={} colour={} \
                     size={} fit={} was_foreign={} ap={:?}",
                    id.num,
                    change.subtype,
                    change.icon_written,
                    change.color_written,
                    // The engine's own `token()`, never `{:?}`. A driven
                    // check reading `fit=label_clipped` is reading a contract
                    // the engine publishes and tests; a check reading a `Debug`
                    // rendering is reading a shape that can change under it
                    // without a compile error anywhere, so the check can
                    // report the opposite of the truth while quoting it.
                    //
                    // `none` for every subtype but `/Stamp`, and for a stamp
                    // whose size was not touched, because `stamp_label_fit` is
                    // `None` there. That distinguishes "this verb ran on
                    // something with no label" from "it ran and decided
                    // nothing" (`as_requested`), which no single boolean can.
                    change.font_size_written,
                    change
                        .stamp_label_fit
                        .as_ref()
                        .map_or("none", StampLabelFit::token),
                    u8::from(change.appearance_was_foreign),
                    change.appearance
                )
            });
            // **The fit disclosure — R8b rule 4's surviving half, and the
            // reason it is a returned STRING rather than anything drawn.**
            //
            // Resizing a stamp's label can make it stop fitting the rectangle
            // it is in, and the engine's re-bake then does something the
            // operator did not ask for: widen the box, shrink the words, or
            // cut characters off. Every one of those is an **inference** — a
            // decision pdfcer made on their behalf — and rule 4 is explicit
            // about what is owed and what is forbidden:
            //
            //   * FORBIDDEN — marking the canvas. No badge, no tint, no dashed
            //     outline on the stamp. A screenshot of the editing canvas must
            //     not differ from a screenshot of the same document saved and
            //     reopened. Provisional styling is a second rendering path for
            //     the same content, and two paths drift.
            //   * OWED — a sentence off-canvas. The half that survives is the
            //     one about inferences the operator *cannot see*: a label drawn
            //     at 14 pt when they typed 24 looks exactly like a label they
            //     asked for at 14.
            //
            // It goes into the funnel's disclosure `Vec`, which is the status
            // line: not blocking, not positioned relative to the document, and
            // gone on the next act.
            //
            // **Gated on `is_inference()`, which is the engine's own
            // question and not a re-derivation of it.** `AsRequested` decided
            // nothing, and a sentence there would report the operator's own
            // instruction back at them — the failure mode that teaches somebody
            // to stop reading the status line. The engine's field doc says the
            // same thing in the same words: *"a label that fitted at the size
            // the caller asked for decided nothing"*.
            //
            // **`StampLabelFit` is `#[non_exhaustive]`.** An outcome this
            // build does not know still answers `is_inference() == true`
            // and lands on the `_` arm, which says something true and vague
            // rather than nothing. Silence there would be the worst of the
            // three options: an inference that happened, reported as though
            // nothing had.
            //
            // **`BoxGrown` is spoken HERE and is silent on the PLACING
            // route, and that asymmetry is the ruling rather than an oversight.**
            //
            // `actions::textannot::disclosures` — the same decision, over the
            // same enum, one file along — drops `BoxGrown` deliberately. What
            // differs between the two routes is not the outcome, which is
            // identical, but **whose rectangle it is**:
            //
            //   * Placing. The rectangle is a request still in progress. The
            //     operator dragged it a moment ago, the dialog forewarned them
            //     with `stamp_size_bound()` before they committed, and the box
            //     that lands is the box they are watching land. Nothing was
            //     taken from them, so a sentence would be pdfcer narrating its
            //     own arithmetic back at the person who just asked for it.
            //   * Restyling. The rectangle is **existing content pdfcer
            //     changed** — a stamp that has been on the page, possibly for
            //     months, at a size the operator chose and may have positioned
            //     against something. Typing a new size into a properties field
            //     is not a request to move the stamp's right edge, and after
            //     the re-bake it may overlap a title block it used to clear.
            //     That is an inference made on content, and the surviving half
            //     of rule 4 owes it a sentence.
            //
            // The question is not *"can the operator see this outcome?"* —
            // they can see both — but *"did they ask for it, in the act they
            // just performed?"* Visibility decides whether the canvas may be
            // marked (it may not, either way); **authorship** decides whether a
            // sentence is owed.
            //
            // Nothing is done with `change.rect_after`, and that is measured
            // rather than overlooked. This shell holds no cached copy of an
            // annotation's geometry — `canvas::selection::annot::AnnotTarget`
            // carries the page, the id, the kind and the subtype, and the
            // handles and the properties fields both re-read the `/Rect` from
            // the session every frame. A `GrowToText` widening therefore shows
            // up on the next frame with no help. If a cache is ever introduced,
            // `rect_after` is what it must be fed, and this comment is the note
            // that says so.
            let mut disclosures = Vec::new();
            if let Some(fit) = change.stamp_label_fit.as_ref()
                && fit.is_inference()
            {
                disclosures.push(match fit {
                    StampLabelFit::LabelShrunk { size, requested } => {
                        ts::stamp_label_shrunk(*size, *requested)
                    }
                    StampLabelFit::LabelClipped { hidden_chars, .. } => {
                        ts::stamp_label_clipped(*hidden_chars)
                    }
                    StampLabelFit::BoxGrown { width, .. } => ts::stamp_label_box_grown(*width),
                    // `AsRequested` is excluded by the guard above; this arm is
                    // the `#[non_exhaustive]` one.
                    _ => ts::stamp_label_fit_unknown().to_owned(),
                });
            }
            disclosures
        })
    });
}
