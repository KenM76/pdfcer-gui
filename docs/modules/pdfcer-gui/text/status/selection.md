# `text::status::selection` — what is selected, said in words

Every string [`crate::app::status::selected`] draws, and nothing else. One
subject, one consumer — the organising principle
[`crate::text`]'s header states for the whole catalog, applied inside an
area that had grown large enough to need it.

## Why this became its own file

Because R2 said so, and R2 was right. `text::status` crossed 1,500 lines
when the form-containment clause landed, and the rule this project was
founded on is that the limit is the signal to find the seam rather than to
raise the limit — the GUI being replaced reached 25,005 lines in one
`main.rs`, and *"nothing could be reasoned about locally"* is the direct
cause of most of what is wrong with it.

The seam was already there. Every function here is read by exactly one
widget, they are the only strings in the area that describe a *thing the
operator picked* rather than a control they can press, and they now include
the two sentences the form-XObject work turned on. The paths do not move:
`status`'s `mod.rs` re-exports this, so `t::selection_one` still resolves
and no call site changed. A catalog area is keyed by its consumer, and the
consumer did not change — only the file did.

## The four sentences, and the state each is for

| state | line |
|---|---|
| one page object | `Selected: Path · 120.0 × 40.0 pt` |
| one object inside a form XObject | `… · inside a form` |
| several things | `3 objects selected` |
| anything, with more underneath | `… · 1 of 5 here` |

★ **The containment clause is [rule 4](R8b) disclosure and it is
off-canvas.** A form-interior object is drawn on the page exactly as it
will be drawn when saved — no badge, no tint, no dashed outline. What
pdfcer had to do to find it is reported here, in words, on a bar; never by
marking the drawing.
