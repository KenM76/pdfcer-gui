# `text::panels::textobject` — the words the CLICKED-TEXT colour control uses

`OPERATOR_REQUESTS.md` **O89**, piece 1, and the candidate O89 called
*"closest to what you tried"*:

> *"I don't see where I am able to edit the color of text, vectors, etc."*

He clicked a piece of text. That selects the **object**, and every text
colour control in the program was gated on a **swept range**, so the swatch
he was looking for was greyed and the way to un-grey it — arm the Text tool,
sweep the words — is unguessable. `crate::panels::properties::textobject`
is the control that acts on the object he actually clicked; these are its
words.

## Why a separate strings module rather than `super::properties`

**R2, measured rather than assumed.** `text::panels::properties` was at
**1,446 lines of its 1,500** on the day this was written — 54 lines of
headroom for a subject that needs six strings and the argument behind each.
Adding them there would have spent the whole remaining budget of the file
that holds every other Properties string, and the next person to reword a
form-field caption would have met the gate instead.

The seam is real and not merely a size cut: every string in
`super::properties` describes **what is selected**; every string here
describes **a route that was not obvious** and what the control on that
route will and will not touch. They change for different reasons —
`super::properties` changes when a property is added, this changes when the
route changes.

## The refusal is the load-bearing string, exactly as it is for vectors

`crate::text::paint`'s header states it for a path and it is if anything
sharper here, because **a text run's unmodelled colour cannot even be
named**: `pdfcer_core::text_extract::TextColor::Other` is a fieldless
variant — the extraction records *"set in a space this pass does not
decode"* and carries no `/Separation` name with it, where
`pdfcer_core::vector::PathPaint::Other` carries `space`.

⇒ So [`ink_present`] must **not** promise a name. Writing *"PANTONE 300 — a
named ink"* here would be a sentence this shell cannot source, which is the
claim-bearing-copy failure in miniature. It says what is true: the colour is
set in a space pdfcer will not overwrite with a screen colour, and it says
which surface *can* name it (the Objects panel's own reading), rather than
inventing one.

## Rule 4 — none of these words reaches the page

Every string here is drawn in the Properties panel. Nothing in this module
marks the canvas, tints a run, or renders a recoloured object differently
from the way the saved file will render it. What was skipped and why is
disclosed **off-canvas**, here and in the status bar, which is where Rule 4
puts it.
