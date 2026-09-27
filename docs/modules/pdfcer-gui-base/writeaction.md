# `writeaction` — the three verbs that exist only to move a file
picker out of the layout pass


## The seam, and it is the sharpest one this enum has

Every other `Action` exists because something has to happen **after** the
frame that raised it -- an edit through the funnel, a dialog opened, a
selection changed. These three exist for a reason about **egui**:

> **A native file dialog must not open inside a layout pass.** It is a modal
> OS window that blocks the thread, so opening one from a widget's
> `clicked()` branch leaves egui part-way through a frame that will not
> finish until the operator has answered.

⇒ That is the whole of what they share, and it is a property no other family
in the enum has. `super`'s declaration of `action` named **markup** as the
measured candidate for the next sub-enum -- 370 lines, and still the largest
-- and the rule it stated was *"the next family of variants to **grow**"*.
Today the family that grew is this one, so this one moved. The markup
measurement stands and is still the answer the day markup grows.

## Why a SAVE is in here with two exports

`Compacted` writes the document itself rather than a derivative of it, so it
reads at first like the odd one out. It is not: it is here because it is an
`Action` for exactly the reason above and for no other. The document is not
changed, no undo entry is made, no epoch moves -- `app::save`'s header states
that a save is a **read** of the session, and all three of these are.

The alternative grouping -- *"things that produce a file"* -- would put
`SaveCopy` in here too, and `SaveCopy` is NOT an action: it is called
directly, because its picker opens from the command dispatcher rather than
from a widget's `clicked()`. Grouping by what a verb produces would have
collected a set whose members do not share the property the set exists for.
