# `text::rotating` — every sentence the ninth handle shows

Five refusals and two disclosures, for [`crate::canvas::rotating`] and
[`crate::app::actions::annots`]. The sibling of [`crate::text::resizing`],
written the day `pdfcer-core` `Pass 155.0` and `Pass 159.0` gave this shell a
rotation for the annotation family and for ce dimensions.

## Why a rotation needs a *smaller* catalog than a resize, and one
disclosure the resize does not have

[`crate::text::resizing`] holds six refusals because a resize can go wrong
in six ways, and three of them are about **the artwork being redrawn**: a
foreign appearance stream is scaled by §12.5.5's placement matrix *after*
stroking, and no scalar `/BS /W` describes an anisotropic stroke, so pdfcer
has to refuse or distort.

**None of that applies here**, and the reason is worth stating because it
decides how much copy this file is entitled to:

> Step (a) transforms the appearance `BBox` **through its own `/Matrix`**,
> and step (c) concatenates that with the placement matrix. So pdfcer
> composes a rotation into the `/Matrix` a producer already wrote —
> **nothing is redrawn, nobody's artwork is replaced** — and it works on a
> stamp Acrobat made as well as on one we drew.

A rotation is also an **isometry**: every length is preserved, including the
drawn stroke width. So there is no `scale_stroke_width` question, no
`allow_appearance_distortion`, no options type at all — and therefore no
sentence in this file naming a switch, because there is no switch. The
engine put the operator-facing consequence in one line: *"if your grip UI
offers rotate and resize together, **rotate needs no confirmation step and
no distortion warning.** Resize does."*

## The two disclosures, and why they are the ONLY two

Rule 4's surviving half: *an inference or a consequence the operator cannot
see still owes an off-canvas report.* Applied honestly, that admits exactly
two things here and excludes several that look like candidates.

| consequence | disclosed? | why |
|---|---|---|
| the shape turned | **no** | they can see it. Narrating a visible result is noise |
| **the mark really did grow** | **yes, and ONLY for one rule** — [`rect_still_grows`] | Rewritten 2026-09-07 twice in one day. It used to read *"`/Rect` grew"* and fired on every non-quarter turn, because the outline was drawn from `/Rect` and visibly swelled. Then the outline started following the artwork (`canvas::annotquad`), and then `pdfcer-core` `Pass 155.1` stopped the growth itself. What is left is `RectDerivation::PreviousRect` — an annotation with neither an appearance nor rotatable geometry, which has **nowhere to record an orientation** and genuinely still compounds |
| **a `Linear` dimension's axis lock relaxed** | **yes** — [`axis_lock_relaxed`] | the engine's own instruction: *"an operator whose dimension silently stopped being axis-locked will find out later and blame something else"* |
| the measured value | **no** | it **cannot** change. A rotation preserves every distance, so the number is identical by construction. A sentence saying "the measurement is unchanged" would invite a reader to look for a change that cannot exist |
| `/RD` left alone | **no** | at an angle that is not a quarter turn **no** axis-aligned inset expresses the rotated result, so leaving it is the only correct behaviour. A sentence about it would teach an operator to worry about something that is right — the same ruling [`crate::text::markup`]'s move disclosure already makes about `rect_differences_untouched` |
| the appearance `/Matrix` was composed | **no** | that is *how* a rotation is expressed, not a consequence of it. It is in the trace, where implementation facts belong |

## The rule every sentence follows

**Name the thing the operator can see, never the thing pdfcer models.**
[`crate::text::resizing`] states it and this file inherits it: they can see
a stamp, a dimension and a dashed box around it; they cannot see a `/Rect`,
an `/IT /LineDimension` or an appearance stream. A refusal phrased in the
file format's vocabulary is a refusal that reads as an internal error.
