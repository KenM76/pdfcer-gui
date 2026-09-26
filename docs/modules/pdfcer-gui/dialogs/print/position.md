# `dialogs::print::position` — where the page sits on the paper

Operator request O208: *"can we add a control to our print preview screen
so that when we are printing at a scale that will lose content we have the
option to drag the drawing to a new position on the print page? That way we
can choose what gets cropped."*

## What this module owns

One quantity: a **displacement from the placement pdfcer chose**, per
document page. Everything else here is either arithmetic over that quantity
or the controls that set it. The copy is next door in
[`crate::text::print`]; the preview that draws the result is
[`super::preview`].

## ★ Contract: a delta, keyed on the DOCUMENT page, sparse

- **A delta, not a position.** Zero means *where pdfcer put it*, which is
  what makes Reset a meaningful command distinct from Centre — see
  [`Positions::centre`] for why those two are not synonyms.
- **Keyed on `PagePlan::index`**, never on a position in the plan list. The
  job may be reversed, subset-filtered, or print a page more than once, so
  the two coincide only for a whole-document forward job.
- **Sparse.** An empty map means every placement is byte-identical to the
  one [`super::spooler::plan`] returned, which is what stops this feature
  from being able to change a job nobody has touched (`R6`).

## Where it is applied, and why not in the spooler

[`Positions::displace`] is called from `PrintDialog::show` on the value
`plan` returns, **before any reader**. Not inside [`super::spooler`],
because that module's header states that nothing in it computes a
placement, a sequence or a scale; the operator's displacement is a shell
decision and it belongs on the shell side of that line.

Applying it there rather than at each reader is what makes the preview, the
per-edge readout, the clip count, the commit and the trace agree by
construction — including [`super::verdicts`], whose cache key is the
[`Placement`] itself, so a displaced page correctly invalidates the ink
verdict measured at its old position.
