# `app::status::decline::textedit` — the two declines the TEXT CARET raises

`OPERATOR_REQUESTS.md` **O127**, defects 2 and 3. Two recording functions,
and the argument they share.

## Why it is its own file

**R2.** [`super`] crossed 1,500 lines the day these two arrived — the second
split from it, after `decline/floor.rs` — and the seam is a real one rather
than a size-driven cut. Everything else in that file answers *"what is a
decline, and how long does it owe its sentence?"*; this answers *"what does
the text caret decline, and who says so?"*, which is a subject with two
call-site families and an argument about **channels** that nothing else on
that surface shares.

## The argument, once, for both: a sentence in the wrong slot is silence

Every cause below was **already being reported** before O127. Four of the
reflow refusals went through `crate::app::actions::record_note`, which the
bar draws under **`⚑ About your last edit:`**; four more collapsed into
[`super::Declined::EditRefused`]'s nine cause-free words; and Enter in an
existing run said nothing at all, because it quietly committed instead.

The operator's verdict on the first family was *"I haven't seen the reflow
option actually work with anything when I press it."* **He was answered
every single time.** In a slot whose whole contract is *an edit happened;
here is the part you cannot see* — for a press where nothing happened — in
the past tense, about an earlier gesture, truncated to 45 % of the bar
(`NOTES_WIDTH_FRACTION`).

[`super`]'s own header had already ruled on this exact swap, for two other
sentences, in these words:

> *"an operator who reads 'About your last edit' after a gesture that did
> nothing has been told a small lie confidently."*

⇒ So the fix for two of O127's three defects is **not new copy**. It is
these two functions, which put existing sentences in the slot that wears
`⊗` and means *nothing happened*. That is worth a file of prose because it
is the second time this project has proved the same thing: **a control that
answers in the wrong place is indistinguishable, from the operator's chair,
from a control that does not answer at all.**

## Written unconditionally, overwriting whatever was live

[`super::record_text_style`]'s rule, and it matters more here: reflow is a
control an operator presses **twice** when nothing appears to happen. The
second press must produce the second press's sentence, and `LAST` is a slot
rather than a queue precisely so the most recent answer is the visible one.
