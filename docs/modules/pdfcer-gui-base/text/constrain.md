# `text::constrain` — the sentence a held Shift puts on the status row

Three strings, for [`crate::canvas::constrain`].

## Why a constraint gets words at all, when the ghost already shows it

`ui-conventions/drag-moves.md` D5 states the failure mode in the operator's
own position:

> *"The operator holds Shift, gets a result they did not expect, and cannot
> tell whether the modifier did anything."*

The ghost answers *"the object is behaving like this"*. It does not answer
*"…because you are holding Shift"*, and those are different questions. An
operator whose drag comes out horizontal cannot tell, from the picture
alone, whether the tool locked it or whether their hand was simply steady —
and the moment they cannot tell, they stop trusting the key and start
aligning by eye, which is the whole capability lost.

## The wording rule these three follow

**Say what is happening, in the operator's terms, and name the key.** Not
*"axis constraint active"* — that is a state, in the program's vocabulary,
and it tells someone who does not already know nothing at all. The key is
named because the sentence is also how the feature is *discovered*: an
operator who reads *"Shift: locked to left and right"* once has learned that
Shift constrains, which no amount of correct behaviour teaches on its own.

Present tense and no period, matching the other transient in-flight line on
this row (`text::doctabs`' drag captions): these are captions on something
happening now, not statements about something that happened.

## Item notes

### `fn each_lock_has_its_own_sentence`

The sharing case is the one worth guarding: a copy-paste that left both
axes saying "left and right" would be invisible in review and would tell
the operator the exact opposite of the truth half the time.

### `fn caption`

One function over the enum rather than one per variant, for the reason
[`crate::text::resizing::refusal`] gives for the same shape: a variant added
to [`Lock`] becomes a compile error here instead of a constraint that
silently announces nothing.
