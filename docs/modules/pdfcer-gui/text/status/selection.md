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

**The containment clause is [rule 4](R8b) disclosure and it is
off-canvas.** A form-interior object is drawn on the page exactly as it
will be drawn when saved — no badge, no tint, no dashed outline. What
pdfcer had to do to find it is reported here, in words, on a bar; never by
marking the drawing.

## Item notes

### `fn coverage_line`

# Remedy first, which reverses the sentence when there is one

This module's rule is *remedy first in every arm that has one*, because the
operator is looking at text that did not change and the useful half is what
to do now. Without a list there is no remedy to lead with and the sentence
opens on the diagnosis; with one it opens on the faces. That is two
sentences rather than one with a clause bolted on, and it is deliberate: a
sentence that opens *"That face has no shape…"* and ends *"… Times-Roman
can"* buries the actionable half behind the explanation.

`"The face you picked"` rather than naming it. The name is in the face
chooser the operator is looking at, and repeating it costs width on a bar
that is already carrying up to fourteen face names in the first clause.

# Why `const WITHOUT` and not a second catalog function

Because it is the *same* refusal. Two catalog entries would be two
sentences that must be kept consistent with each other by hand, and this
project has a gate (`check-ui-strings`) that would be content with both.

### `fn join_or`

`or`, not `and`: the faces are **alternatives**, and `join_and` in
`crate::text::page_size` — whose subject is edges a drawing runs past, all
of which are true at once — would read as though the operator needed all
three. Copied rather than shared for exactly that reason: the two differ in
the one word that carries the meaning, so a shared helper would need a
parameter that is really a choice about a sentence.

### `fn the_rung_clause_counts_what_is_held`

The wording carried a literal `1` for as long as one chunk was all the
rung could hold. A Shift-click now adds a second, and the drag that
follows moves the set — so a sentence that cannot say *2 lines of 27* is
a confident, wrong statement about the operand of the next keystroke.
