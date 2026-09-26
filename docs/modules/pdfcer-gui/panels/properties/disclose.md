# `panels::properties::disclose` — what pdfcer last worked out, at a width
it fits in

This block renders
[`crate::app::actions::disclosure::last_edit_disclosure`] — the slot that
carries, among other things, **the text tools' refusal sentences**.

## The problem this block exists to solve

`crate::text::textedit::refusal` writes three good sentences and
`every_refusal_says_something` holds them to it, but they are long:
`Refusal::SpansRuns` is **47 words**. The status row cannot hold that — R128
forbids that row growing, so anything put there is elided — and a decline
nobody can read teaches an operator that the feature does not exist.

That is the likely cause of the operator's *"no text editing or adding text
on the canvas"*: on a dense CAD sheet the first click of `edit.text` lands
where the operator *wants* text rather than where text *is*, so
`Refusal::NoRun` is the likely first outcome.

## Why Properties is a permitted home, and the status bar is not

* **The status bar already carries this exact slot.**
  `crate::app::status::disclosure::edit_disclosure` reads the same
  `last_edit_disclosure` and draws it through `disclosure_line`. So the
  question is not *may the bar mention this* — it already does.
* **What the bar may not do is make it readable.** `disclosure_line`
  truncates rather than wraps, because wrapping is how a one-row bar becomes
  a two-row bar, and it draws into a **bounded** sub-region so a long
  sentence cannot push the navigation controls off the right of the bar. A
  47-word sentence rendered under those rules is a hover, not a disclosure.
* ⇒ **So the decline slot in the status bar is not the answer**, and it is
  not asked to become one. It keeps its elided line. The *readable* copy
  lives here.

**A dock panel's width is the dock's**, decided before the body draws, so
text wrapped inside it cannot drive a width and R128 does not apply. That is
the property every wrapping surface in this shell relies on.

## And why THIS panel rather than any other

[`super::annotdelete`] makes the same argument for the same reason: R9 sends
a permanently-refused capability's explanation to the surface that describes
what is selected, because the control it is about lives somewhere that
cannot hold a sentence.

A refusal is a fact about the last thing the operator tried to do to the
document in front of them. Properties is where this application puts facts
of that shape.

## FIRST in the panel

Every disclosure sits above the thing it qualifies, without exception: a
caveat below a list arrives after the operator has already drawn a
conclusion. Everything below this block describes what is selected, and a
refusal read after that description arrives too late to explain it.

## It renders NOTHING when there is nothing, heading included

Not *"No notes."*, not an empty heading. R9, and also honesty: a heading
present on every frame trains an operator to stop reading the region under
it, which would waste the one surface a disclosure has.
