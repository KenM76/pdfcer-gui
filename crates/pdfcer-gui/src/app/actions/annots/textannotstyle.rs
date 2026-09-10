//! # `app::actions::annots::textannotstyle` — restyling a text-BEARING annotation
//!
//! One verb, `set_text_annot_style`, and the disclosure it now owes. Split out
//! of [`super`] on 2026-09-10 under **R2**, when `pdfcer-core` `Pass 292.0`'s
//! label-size disclosure took that file past 1,500 lines.
//!
//! ## ★★★ Why it is its own file rather than one more arm in `annots.rs`
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
//! ## ★ Where its siblings are
//!
//! [`crate::app::actions::textannot`] authors these same three kinds and makes
//! the same class of disclosure at placing time, off the same engine `Pass`.
//! The two files are the *before* and *after* of one capability and read best
//! together; the strings both of them speak are in
//! [`crate::text::panels::textannotstyle`], one sentence per act, so the two
//! surfaces cannot drift into wording the operator has to reconcile.

use pdfcer_core::annot_author::StampLabelFit;
use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;
use crate::text::panels::textannotstyle as ts;

/// ★★★ **Restyle a text-BEARING annotation** — a sticky note's icon and
/// colour, a stamp's colour. `EditSession::set_text_annot_style`
/// (`pdfcer-core` `edit.rs:27124`).
///
/// # ★★★ Why this is a second function beside the markup restyle, not a branch
/// # inside it
///
/// Because it is a second **verb over a second spec family**, and the engine's
/// own doc on `edit::TextAnnotStyle` (`edit.rs:15969`) is the argument:
///
/// > `MarkupStyle` reaches its annotation through
/// > `annot_author::spec_from_dict`, whose arms are the geometric family and
/// > the four text markups. **There is no `/Text` arm** … So the two verbs are
/// > not a split anyone chose for tidiness — they read through different
/// > functions because the two families are modelled by different spec types.
///
/// ⇒ A single body with an `if subtype == "Text"` inside it would put the
/// routing decision where nothing checks it. It is a `match` instead, on
/// `crate::panels::properties::markup::textannot::Reach`, decided in the panel
/// by asking each of the two engine readers in turn — and this function is only
/// ever reached down one arm of it.
///
/// # ★★ The page is `0`, and that is not a defect
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
/// # ★ What the trace carries, and why it is not the values
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
/// # ★ The undo entry it pushes
///
/// `pdfcer_core::edit::CommandKind::SetTextAnnotStyle` — the engine's own
/// label for this command, pushed by the verb itself rather than by the funnel,
/// so an undo of a restyle is one entry and names the act rather than the
/// appearance rewrite underneath it.
///
/// # ⚠ It DOES have a disclosure list, and the sentence here said otherwise
///
/// Until 2026-09-10 this header read *"no disclosure list … there is nothing
/// it can silently lose that this shell could name"*, and the clause about
/// `dropped` is still true: unlike `set_markup_style` this verb re-bakes from
/// a spec the same reader hands the authoring path, so no property is quietly
/// discarded. The conclusion drawn from it was wrong for a different reason.
///
/// `pdfcer-core` `Pass 292.0` made this verb able to change a stamp's **label
/// size**, and a re-bake at a new size can then do one of three things nobody
/// asked for: widen the box, shrink the words, or cut characters off the ends.
/// Those are inferences, and the two that a screenshot cannot show owe a
/// sentence off-canvas under R8b rule 4. So the funnel's `Vec<String>` is no
/// longer empty, and what goes in it is decided below on
/// `TextAnnotStyleChange::stamp_label_fit`.
///
/// ★★ The general lesson, and it is why this paragraph replaces the old one
/// rather than sitting beside it: **a "this cannot happen" sentence is a
/// citation with a shelf life measured in engine passes.** It was true when it
/// was written and false eight days later, and nothing in the build would have
/// said so.
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
                // ★★★ `was_foreign=` is new with `pdfcer-core` `Pass 253.5`,
                // and it is traced rather than DISCLOSED, deliberately.
                //
                // The engine's field reports that the annotation's previous
                // appearance was one pdfcer would not have drawn — a
                // designer's stream with a shadow or a gradient — and that
                // re-baking has just replaced it with pdfcer's plainer
                // rendering. That is exactly the class of thing Rule 4's
                // surviving half says must reach the operator off-canvas.
                //
                // ⚠ **It cannot happen through this shell today**, and the
                // reason is worth stating rather than discovering: the field is
                // `false` for every subtype but `/FreeText`, and the only
                // surface that raises this action —
                // `panels::properties::markup::textannot` — **declines a
                // `/FreeText` by name**. So a disclosure wired here would be a
                // sentence no operator could ever see, which is worse than none:
                // it reads as covered.
                //
                // ⇒ It goes on the trace, which is where a fact with no reader
                // belongs, and it becomes the **tripwire** for the day a
                // `/FreeText` does reach this verb: a driven run showing
                // `was_foreign=1` means the panel's decline has been lifted and
                // an operator-facing sentence is now owed. That is the same
                // posture `appearance_matrix_updated` gets one module along.
                format!(
                    "set-text-annot-style-applied id={} subtype={} icon={} colour={} \
                     size={} fit={} was_foreign={} ap={:?}",
                    id.num,
                    change.subtype,
                    change.icon_written,
                    change.color_written,
                    // ★★★ The engine's own `token()`, never `{:?}`. A driven
                    // check reading `fit=label_clipped` is reading a contract
                    // the engine publishes and tests; a check reading a `Debug`
                    // rendering is reading a shape that can change under it
                    // without a compile error anywhere — which is exactly how
                    // one check on this project came to report the opposite of
                    // the truth while quoting the truth in its own message.
                    //
                    // ★ `none` for every subtype but `/Stamp`, and for a stamp
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
            // ★★★ **The fit disclosure — R8b rule 4's surviving half, and the
            // reason it is a returned STRING rather than anything drawn.**
            //
            // Resizing a stamp's label can make it stop fitting the rectangle
            // it is in, and the engine's re-bake then does one of three things
            // the operator did not ask for: widen the box, shrink the words, or
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
            // ⇒ It goes into the funnel's disclosure `Vec`, which is the status
            // line: not blocking, not positioned relative to the document, and
            // gone on the next act.
            //
            // ★★ **Gated on `is_inference()`, which is the engine's own
            // question and not a re-derivation of it.** `AsRequested` decided
            // nothing, and a sentence there would report the operator's own
            // instruction back at them — the failure mode that teaches somebody
            // to stop reading the status line. The engine's field doc says the
            // same thing in the same words: *"a label that fitted at the size
            // the caller asked for decided nothing"*.
            //
            // ⚠ **`StampLabelFit` is `#[non_exhaustive]`.** A fourth outcome
            // this build does not know still answers `is_inference() == true`
            // and lands on the `_` arm, which says something true and vague
            // rather than nothing. Silence there would be the worst of the
            // three options: an inference that happened, reported as though
            // nothing had.
            //
            // ★★★ **`BoxGrown` is spoken HERE and is silent on the PLACING
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
            // ⇒ The question is not *"can the operator see this outcome?"* —
            // they can see both — but *"did they ask for it, in the act they
            // just performed?"* Visibility decides whether the canvas may be
            // marked (it may not, either way); **authorship** decides whether a
            // sentence is owed.
            //
            // ★ Nothing is done with `change.rect_after`, and that is measured
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
