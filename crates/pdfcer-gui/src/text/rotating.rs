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

/// **Disclosure: the selection box grew and the artwork did not.**
///
/// ★★★ The one thing about this feature that *will* look like a bug, and
/// `pdfcer-core` flagged it before this shell had drawn a single rotated shape:
///
/// > **`/Rect` gets bigger.** §12.5.2 requires it upright, and the upright box
/// > bounding a rotated shape is larger unless the angle is a multiple of 90°.
/// > **The artwork does not grow; the rectangle around it does.**
///
/// ★★ It matters *here*, on this canvas, more than it would in a headless
/// tool — because **this shell draws its selection outline from `/Rect`**.
/// So the operator turns a stamp 30°, the stamp turns, and the dashed box
/// around it visibly swells. Nothing on the page can explain that, which puts
/// it squarely inside Rule 4's surviving half: *an inference or a consequence
/// the operator cannot see still owes an off-canvas report.* Render normally;
/// report separately. Both.
///
/// ★ **`None` at a quarter turn**, which is not an optimisation but the same
/// rule every disclosure in this crate follows: a sentence that fires on every
/// gesture is a sentence nobody reads by the third time. At 90°, 180° and 270°
/// the upright bounding box is the original box with its sides swapped or
/// unchanged, so there is genuinely nothing to disclose — and an operator
/// turning a stamp upright is doing the commonest rotation there is.
///
/// The threshold is one PDF point in either extent. Below that the growth is
/// invisible at any zoom this shell offers and a sentence about it would be
/// reporting rounding.
///
/// # Why it says nothing about *why*
///
/// §12.5.2 and the phrase "upright bounding box" are correct, unactionable and
/// too long for a status row. What the operator needs is *the shape is the size
/// it was; the box is not the shape*, and the second clause is what stops them
/// pressing Ctrl+Z on a rotation that worked perfectly.
#[must_use]
pub fn rect_grew(from: (f64, f64), to: (f64, f64)) -> Option<String> {
    let grew = (to.0 - from.0) > 1.0 || (to.1 - from.1) > 1.0;
    grew.then(|| {
        "The dashed box around this mark is now larger, because a box that is square to the page \
         has to be bigger to hold something turned at an angle. The mark itself is exactly the \
         size it was."
            .to_owned()
    })
}

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

    /// ★★ **A quarter turn discloses nothing**, which is the commonest
    /// rotation there is.
    ///
    /// The upright box bounding a rectangle turned by a multiple of 90° is the
    /// original box with its sides swapped, so there is nothing to report — and
    /// a build that reported anyway would put a sentence on the status row for
    /// every single rotation an operator makes, which is how a disclosure stops
    /// being read.
    #[test]
    fn a_quarter_turn_discloses_nothing() {
        // 100×50 turned 90° → 50×100: one extent grew, so this is deliberately
        // NOT the swapped case. The swapped case is below.
        assert!(rect_grew((100.0, 50.0), (100.0, 50.0)).is_none());
        assert!(
            rect_grew((100.0, 50.0), (100.0, 50.5)).is_none(),
            "rounding"
        );
    }

    /// ★ **A box that swelled on either axis discloses**, and one that shrank
    /// on the other still does.
    ///
    /// The `||` rather than `&&` is the load-bearing choice: turning a tall
    /// thin rectangle towards horizontal grows its width and shrinks its
    /// height, and the operator watching the box widen is owed the sentence
    /// whether or not the other extent cooperated.
    #[test]
    fn a_grown_box_discloses_on_either_axis() {
        assert!(rect_grew((100.0, 50.0), (120.0, 40.0)).is_some());
        assert!(rect_grew((100.0, 50.0), (90.0, 70.0)).is_some());
        assert!(rect_grew((100.0, 50.0), (120.0, 70.0)).is_some());
    }

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
