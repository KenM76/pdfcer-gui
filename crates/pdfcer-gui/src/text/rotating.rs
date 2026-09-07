//! # `text::rotating` — every sentence the ninth handle shows
//!
//! Five refusals and two disclosures, for [`crate::canvas::rotating`] and
//! [`crate::app::actions::annots`]. The sibling of [`crate::text::resizing`],
//! written the day `pdfcer-core` `Pass 155.0` and `Pass 159.0` gave this shell a
//! rotation for the annotation family and for ce dimensions.
//!
//! ## ★★★ Why a rotation needs a *smaller* catalog than a resize, and one
//! disclosure the resize does not have
//!
//! [`crate::text::resizing`] holds six refusals because a resize can go wrong
//! in six ways, and three of them are about **the artwork being redrawn**: a
//! foreign appearance stream is scaled by §12.5.5's placement matrix *after*
//! stroking, and no scalar `/BS /W` describes an anisotropic stroke, so pdfcer
//! has to refuse or distort.
//!
//! **None of that applies here**, and the reason is worth stating because it
//! decides how much copy this file is entitled to:
//!
//! > Step (a) transforms the appearance `BBox` **through its own `/Matrix`**,
//! > and step (c) concatenates that with the placement matrix. So pdfcer
//! > composes a rotation into the `/Matrix` a producer already wrote —
//! > **nothing is redrawn, nobody's artwork is replaced** — and it works on a
//! > stamp Acrobat made as well as on one we drew.
//!
//! A rotation is also an **isometry**: every length is preserved, including the
//! drawn stroke width. So there is no `scale_stroke_width` question, no
//! `allow_appearance_distortion`, no options type at all — and therefore no
//! sentence in this file naming a switch, because there is no switch. The
//! engine put the operator-facing consequence in one line: *"if your grip UI
//! offers rotate and resize together, **rotate needs no confirmation step and
//! no distortion warning.** Resize does."*
//!
//! ## ★★★ The two disclosures, and why they are the ONLY two
//!
//! Rule 4's surviving half: *an inference or a consequence the operator cannot
//! see still owes an off-canvas report.* Applied honestly, that admits exactly
//! two things here and excludes several that look like candidates.
//!
//! | consequence | disclosed? | why |
//! |---|---|---|
//! | the shape turned | **no** | they can see it. Narrating a visible result is noise |
//! | **`/Rect` grew** | **yes** — [`rect_grew`] | §12.5.2 requires `/Rect` upright, so the box bounding a rotated rectangle is larger at any angle that is not a quarter turn. **The selection outline is drawn from `/Rect`**, so the operator watches a box grow around artwork that did not — which reads as a bug, and is not one |
//! | **a `Linear` dimension's axis lock relaxed** | **yes** — [`axis_lock_relaxed`] | the engine's own instruction: *"an operator whose dimension silently stopped being axis-locked will find out later and blame something else"* |
//! | the measured value | **no** | it **cannot** change. A rotation preserves every distance, so the number is identical by construction. A sentence saying "the measurement is unchanged" would invite a reader to look for a change that cannot exist |
//! | `/RD` left alone | **no** | at an angle that is not a quarter turn **no** axis-aligned inset expresses the rotated result, so leaving it is the only correct behaviour. A sentence about it would teach an operator to worry about something that is right — the same ruling [`crate::text::markup`]'s move disclosure already makes about `rect_differences_untouched` |
//! | the appearance `/Matrix` was composed | **no** | that is *how* a rotation is expressed, not a consequence of it. It is in the trace, where implementation facts belong |
//!
//! ## The rule every sentence follows
//!
//! **Name the thing the operator can see, never the thing pdfcer models.**
//! [`crate::text::resizing`] states it and this file inherits it: they can see
//! a stamp, a dimension and a dashed box around it; they cannot see a `/Rect`,
//! an `/IT /LineDimension` or an appearance stream. A refusal phrased in the
//! file format's vocabulary is a refusal that reads as an internal error.

/// **Why a rotation did not happen**, in the shell's own reading of the cases.
///
/// # ★★★ Five variants, and the founding rule they answer
///
/// > A REFUSAL MUST BE A SENTENCE, NEVER A SILENCE.
///
/// This project's founding defect shape is a grip that is dragged, released,
/// and does nothing with no explanation — `DEFECTS.md` D4a, and the eight
/// resize grips lived in exactly that state for the whole life of this shell.
/// A ninth handle shipped without this enum would have reproduced it on its
/// first day.
///
/// # ★★ Why a `Copy` enum rather than the engine's own `Display`
///
/// [`crate::text::status::TextStyleRefusal`]'s reason, adopted unchanged: a
/// `format!` of an `EditError` would route **diagnostic prose into the UI**,
/// which `tools/gates/check-ui-strings.sh`' exclusion 3 names in as many words.
/// An enum keeps `crate::app::status::decline::Declined` `Copy` and keeps every
/// operator-visible word in this file, under **R1**.
///
/// # ★ Two of the five are unreachable today, and they are kept
///
/// [`Self::WrongVerb`] and [`Self::NoDimensionRecord`] describe **routing
/// failures**, and this shell routes: `canvas::rotating` matches on
/// `AnnotKind` and sends a markup to `rotate_annotation` and a ce dimension to
/// `rotate_dimension`, and a widget is never an annotation selection at all.
/// If either sentence ever appears, the routing has broken.
///
/// ⇒ **That is the argument for keeping them, not against.** A routing bug
/// with a sentence is a bug report; a routing bug without one is a handle that
/// does nothing, which is the exact defect the handle was built to close and
/// the one this canvas has now produced four times. The sentences are written
/// for an operator, not for a maintainer — they say what to do next — but their
/// *existence* is a tripwire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotateRefusal {
    /// The engine refused by name and pointed at a different verb — a widget,
    /// or a ce dimension handed to the annotation rotation.
    ///
    /// `EditError::AnnotationMoveWrongVerb`. See the enum docs: unreachable
    /// while the routing holds, and the sentence is what makes a break in it
    /// visible.
    WrongVerb,
    /// The document is **certified**, and its permissions forbid a change of
    /// this kind.
    ///
    /// `EditError::CertificationForbidsChange`. ★ The one variant here that is
    /// genuinely reachable on an ordinary file, and the one an operator has no
    /// way to guess at: a signed drawing looks exactly like an unsigned one on
    /// the canvas.
    Certified,
    /// The selected ce dimension has **no record in the document's measurement
    /// sidecar** that this shell could resolve.
    ///
    /// Shell-side, raised by `canvas::rotating` before any verb is called.
    /// `rotate_dimension` addresses the sidecar record rather than the
    /// annotation, so without one there is nothing to turn.
    NoDimensionRecord,
    /// The drag reached the **page-content** rotation and the selection names
    /// no page object on this page.
    ///
    /// ★★★ Added 2026-08-29, when the first driven run of
    /// `rotating_a_markup_turns_it` found this state returning in **silence** —
    /// see [`crate::canvas::rotating::drag`]'s own account of the guard that
    /// produced it, and the module header's "sixth instance" section.
    ///
    /// ★★ It is reachable, and by an ordinary route rather than a routing bug:
    /// `SelectionState::object_indices_on` keeps entries carrying a
    /// `page_object_index` and drops the ones carrying only a `leaf_index`, so
    /// an operator who has clicked **into** a form XObject has an outline, a
    /// grip box, a painted rotate handle — and nothing this verb can address.
    /// That is the *"you selected something this verb cannot reach"* half of
    /// the distinction `SelectionState::leaf_indices_on` exists to let a caller
    /// word, and it is why the sentence below does not say "select something":
    /// they did.
    NothingSelected,
    /// Anything else the engine declined.
    ///
    /// ★ A catch-all with a **hand-written** sentence, not a rendered error.
    /// `TextStyleRefusal::Other` sets the precedent and the reasoning is the
    /// same: wording a decline is catalog work per refusal, and the honest
    /// fallback is a sentence that says *nothing changed and Ctrl+Z has nothing
    /// to take back* rather than one that guesses at a cause.
    Other,
}

impl RotateRefusal {
    /// The sentence.
    ///
    /// Remedy first wherever there is one, for [`crate::text::resizing`]'s
    /// stated reason: the operator is looking at something that did not turn,
    /// and the useful half is *what to do now*.
    #[must_use]
    pub const fn line(self) -> &'static str {
        match self {
            // ★ It names what the operator can see — a form field, a dimension
            // — rather than "the wrong verb", which is a fact about this
            // program's internals and would read as an internal error. The
            // second clause is the actionable half: nothing was changed, so
            // there is nothing to undo and nothing to repair.
            Self::WrongVerb => {
                "pdfcer cannot turn that kind of item, and it changed nothing rather than turn part of it. Form fields and dimensions each need their own tool."
            }
            // ★★ "Signed", not "certified": the operator's word for what
            // happened to the file is that somebody signed it. And it says the
            // limit is the DOCUMENT's rather than pdfcer's, because an operator
            // told only "cannot" will look for a setting to change.
            Self::Certified => {
                "This document has been signed, and the signature does not allow it to be changed this way. pdfcer turned nothing."
            }
            // ★ It says the dimension is still usable, because the alternative
            // reading — "this dimension is broken" — would send somebody to
            // delete and redraw a perfectly good measurement.
            Self::NoDimensionRecord => {
                "pdfcer could not find the measurement behind this dimension, so it turned nothing. The dimension itself is unchanged and still measures what it did."
            }
            // ★★ It does NOT say "select something first" — the resize
            // catalogue's wording for its own `NothingSelected`, and wrong
            // here. This fires when something IS selected: an operator who has
            // clicked into a form XObject is holding a piece of one, and being
            // told to select something would send them to do again the thing
            // they just did. What it names instead is the remedy — step back
            // out to the whole shape, which is the rung this verb can address.
            Self::NothingSelected => {
                "pdfcer turns whole shapes, and what is selected here is a piece of one. Press Escape to select the whole shape, then drag the round handle again."
            }
            // ★ No cause named, because none is known. What it does say is the
            // one thing the operator needs: the page is exactly as it was.
            Self::Other => {
                "pdfcer could not turn that, and it changed nothing — the page is exactly as it was, and there is nothing to undo."
            }
        }
    }
}

// **★★★ DELETED 2026-09-07 — the consequence this sentence explained no longer
// happens, and the sentence had become false.**
//
// It read:
//
// > *"The dashed box around this mark is now larger, because a box that is
// > square to the page has to be bigger to hold something turned at an angle.
// > The mark itself is exactly the size it was."*
//
// Every word of that was true while **this shell drew its selection outline
// from `/Rect`**. §12.5.2 requires that rectangle upright, so a turned mark
// was boxed rather than described, and the operator watched a dashed box swell
// around artwork that had not changed size. Rule 4's surviving half applied
// exactly — a consequence the operator can see and cannot explain owes an
// off-canvas report — and this was that report.
//
// **`canvas::annotquad` removed the subject.** The outline is now drawn at the
// mark's own angle (`OPERATOR_REQUESTS.md` O147), hugging the artwork, so
// there is no swelling box and nothing to explain. A status row still saying
// *"the dashed box is now larger"* would describe something that does not
// happen, which is worse than saying nothing: it teaches an operator to worry
// about a thing that is right, and the next time he sees a box that really is
// wrong he will have been trained to ignore it.
//
// ⚠ **Do not restore it for O145.** A mark rotated twice really does get
// bigger — *the ink, not the box* — and that is an engine defect
// (`request_rotate_annotation_grows_the_artwork_when_applied_twice.md`,
// reproduced in `tests/annotation_rotation_grows.rs`), not a consequence to be
// disclosed. Disclosing a defect as though it were correct behaviour is how
// this project's predecessor accumulated its red flags.
//
// ⇒ The general rule, and it is why this comment is kept rather than the code:
// **a disclosure has a subject, and when the subject goes the disclosure is a
// lie with a citation attached.** Deleting one is as much a part of the work
// as writing one.

/// **Disclosure: a dimension that was locked to horizontal or vertical is no
/// longer locked.**
///
/// ★★★ The engine commissioned this sentence by name, and its argument is the
/// whole reason the disclosure exists rather than the relaxation being silent:
///
/// > A `Linear` dimension locked to horizontal or vertical cannot stay locked
/// > through a rotation. We relax it to *aligned* and report
/// > `constraint_relaxed: true`. **Say so: an operator whose dimension silently
/// > stopped being axis-locked will find out later and blame something else.**
///
/// ★★ There were three options and two are wrong, which is worth carrying here
/// because the sentence has to sound like a *choice* rather than a failure:
/// **refusing** makes rotation impossible for the most common constrained
/// dimensions, which is most of a CAD drawing; **keeping** the constraint
/// leaves the drawn line and its own stated constraint disagreeing, which is
/// worse than either alone and invisible until something regenerates from the
/// constraint. Relaxing preserves exactly what is on the page.
///
/// ★ **It says the measurement did not change**, in the same breath, and that
/// clause is doing real work. An operator told *"the constraint was relaxed"*
/// and nothing else will reasonably wonder whether the number moved too. It
/// cannot: a rotation preserves every distance, so the value is identical by
/// construction. Saying so here is the one place that fact belongs — a separate
/// disclosure asserting the number is unchanged would fire on every rotation
/// and invite a reader to look for a change that cannot exist.
///
/// # Vocabulary
///
/// "Straight across or straight up" rather than *horizontal/vertical
/// constraint*, and "follows the two points you picked" rather than *aligned*.
/// The operator set that lock by clicking a control; they did not name an
/// `AxisConstraint`.
#[must_use]
pub fn axis_lock_relaxed() -> String {
    "This dimension was locked to run straight across or straight up, and turning it means it no \
     longer can — it now follows the two points you picked. What it measures has not changed."
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ★★★ TWO TESTS WERE DELETED HERE ON 2026-09-07, WITH THEIR SUBJECT.
    //
    // `a_quarter_turn_discloses_nothing` and `a_grown_box_discloses_on_either_axis`
    // both asserted on `rect_grew`, which is gone: the selection outline is now
    // drawn at the mark's own angle, so no box swells and there is nothing to
    // disclose. The account is at the deleted function's site above.
    //
    // ⇒ They are named here rather than silently removed because a test that
    // vanishes from a file looks like coverage that was never written. These
    // two were correct, they passed, and their subject stopped existing —
    // which is a different thing from a gap.

    /// Every refusal has a sentence, and none of them is empty.
    ///
    /// ★ The check that a variant added later cannot ship silent — the whole
    /// failure this enum exists to prevent, applied to the enum itself.
    #[test]
    fn every_refusal_is_a_sentence() {
        for why in [
            RotateRefusal::WrongVerb,
            RotateRefusal::Certified,
            RotateRefusal::NoDimensionRecord,
            RotateRefusal::NothingSelected,
            RotateRefusal::Other,
        ] {
            let line = why.line();
            assert!(!line.is_empty(), "{why:?} has no sentence");
            assert!(
                line.ends_with('.'),
                "{why:?} is not a sentence — the founding rule is that a refusal IS one"
            );
        }
    }
}
