# `dialogs::buttonaction` — the *What pressing it does* chooser

One control, drawn into the form-field placement dialog when the kind being
placed is a push button. Lifted out of `dialogs::formfield` rather than
written inline for two reasons: R2 (that file has five kinds' worth of rows
already), and because this is the only row group in the dialog that carries
a **disclosure obligation**, which is easier to review when it is not
interleaved with a comb-cell checkbox.

## ★★★ The interaction is Acrobat's, deliberately

*Field Properties ▸ Actions* is a chooser of behaviours with the chosen
one's parameters beneath it. Every form editor that came after copied it,
which makes it the convention, and this project's standing rule is that the
convergence of the product class **is** the spec. So: one drop-down, the
parameters for the chosen behaviour under it, and nothing else.


## ★★ Why the parameters are held even when they are not shown

[`ButtonDoes`] carries every parameter for every kind at once, so switching
the chooser to *Nothing* and back does not lose the page number that was
typed. That is a property of the model rather than of this file, and it is
restated here because the obvious way to draw this control — build the
engine's `ButtonAction` in the closure — silently has the opposite
behaviour.

## ★ Nothing is greyed, and nothing is hidden

All seven choices are always offered. R9's rule is that an unavailable
capability renders **nothing** and greying is for the *temporarily*
unavailable — and none of the seven is either: the engine can write all of
them into any document a push button can be placed in. A draft that is
incomplete blocks the dialog's **Add** button and says why, which is a
different mechanism and the right one, because the remedy is typing rather
than waiting.
