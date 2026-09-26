# `pdfcer-gui/text/textannot`

## Item notes

### `fn the_printing_distinction_is_stated_in_both_directions`

The one property this module exists to hold. A text box prints and a
sticky does not, and an operator who has them backwards has either
published a private remark or hidden a public one — neither recoverable
by noticing afterwards.

Asserted on the words rather than trusted to review, because the two
controls are otherwise identical and a copy-paste between them would be
invisible in a diff.

### `fn every_offered_stamp_size_is_named_distinctly_and_carries_its_unit`

[`every_offered_stamp_is_named_distinctly`]'s argument, with one clause
that gallery does not need: a combo shows exactly one entry when it is
closed, so a duplicate label there is not merely confusing — the
operator cannot tell which of two entries is currently selected, and the
control silently stops reporting its own state.

The unit clause is the one that would actually fire. `"12"` and
`"12 pt"` are both plausible things for a future edit to produce, they
are distinct from each other, and only one of them is readable in a list
whose first entry is a sentence.

### `fn the_size_disclosure_states_both_directions`

The operator drew a rectangle. Telling them it may widen, without
telling them it will never be shrunk, leaves them believing a stamp
might come out smaller than the space they cleared for it — which on a
title block is the difference between using the control and not.

⚠ Asserted on the words rather than on the behaviour because this is
the `text` crate: the behaviour is `StampFit::GrowToText`'s, tested
where it is called. What is tested here is that the sentence still
describes it. A future edit that softened this to *"the stamp resizes
to fit"* would be true of `ShrinkToBox` too, and this shell does not
offer `ShrinkToBox`.

### `fn only_a_foreign_text_box_appearance_owes_the_operator_a_sentence`

The guard on the section banner's table, and it is deliberately built
**positive first**. The engine's own methodology note, sent with
`Pass 258.1` and aimed at this side of the wire:

> Our first version of the foreign-appearance test asserted
> `!appearance_rebaked` — and **passed with the entire re-bake
> disabled**. A "not X" assertion is vacuous when the thing that would
> produce X is absent.

So the first four assertions here establish that these functions *can*
speak, and every `is_none()` below them is a claim about a condition
rather than about a function that never returns anything. Sabotaging
either half of the guard — deleting the `paints_its_note` term or the
`!appearance_rebaked` term — turns this test red, which is the property
a vacuous version would not have.

The two silences are owed in *opposite* directions and both are
expensive:

- a sticky or a stamp that started warning "the words printed on the
  page were NOT redrawn" describes a page that never showed those
  words, and a warning an operator learns to dismiss costs the one case
  where it is true;
- a re-baked text box that warned anyway would be the deleted lie
  restored, on a build where the page demonstrably moved.

### `fn both_disclosures_state_the_half_that_did_not_move_and_why`

The negative clause was the whole point of these sentences when they
were written and it still is: a disclosure reading only *"the comment
was changed"* is true, is what the operator already knows, and leaves
the whole of the surviving half unsaid.

The *why* clause is new, and it is what stops the sentence reading as
a defect report. On the old build the picture did not move because
pdfcer could not move it; on this one it did not move because moving it
would have thrown away a shadow, a gradient or an image that another
program drew — preservation, not failure. Asserted on *"another
program"* rather than on a verb, because the verb is rewordable
(`keeps`, `leaves`, `preserves` are all honest) while the **cause** is
not: drop that clause and the sentence reports a capability pdfcer is
missing instead of a decision it made, which is the whole difference
between this sentence and the one it replaced.

Checked on `NOT` in capitals for the same reason: it is the only token
that cannot survive a trim down to the cheerful half.
