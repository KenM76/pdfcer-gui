# `buttonactionpicker` — the *What pressing it does* chooser

One control, drawn into the form-field placement dialog when the kind being
placed is a push button. Lifted out of `dialogs::formfield` rather than
written inline for two reasons: R2 (that file has five kinds' worth of rows
already), and because this is the only row group in the dialog that carries
a **disclosure obligation**, which is easier to review when it is not
interleaved with a comb-cell checkbox.

## The interaction is Acrobat's, deliberately

*Field Properties ▸ Actions* is a chooser of behaviours with the chosen
one's parameters beneath it. Every form editor that came after copied it,
which makes it the convention, and this project's standing rule is that the
convergence of the product class **is** the spec. So: one drop-down, the
parameters for the chosen behaviour under it, and nothing else.


## Why the parameters are held even when they are not shown

[`ButtonDoes`] carries every parameter for every kind at once, so switching
the chooser to *Nothing* and back does not lose the page number that was
typed. That is a property of the model rather than of this file, and it is
restated here because the obvious way to draw this control — build the
engine's `ButtonAction` in the closure — silently has the opposite
behaviour.

## Nothing is greyed, and nothing is hidden

All seven choices are always offered. R9's rule is that an unavailable
capability renders **nothing** and greying is for the *temporarily*
unavailable — and none of the seven is either: the engine can write all of
them into any document a push button can be placed in. A draft that is
incomplete blocks the dialog's **Add** button and says why, which is a
different mechanism and the right one, because the remedy is typing rather
than waiting.

## Item notes

### `const CHOSE`

Written on **change**, not every frame. A driven check needs to know the
chooser was reached and what it was set to, and a per-frame line would bury
that in thousands of identical ones.

### `const ROW_REGION`

A driven check reads `form.button.action.row.ResetForm`, not `…row.1`.
See the publisher for why: an index survives a reordering of
`ButtonDoesKind::ALL` and goes on passing while aiming at the wrong row.

### `fn page_rows`

The number box is a plain text field rather than a `DragValue`, and that is
deliberate: a `DragValue` cannot be empty, so it would have to open at some
page, and opening at page 1 is pdfcer choosing a destination. An empty box is
the honest starting state and [`ButtonDoes::blocker`] refuses it.

### `fn show_hide_rows`

Two radio buttons rather than one *Hidden* checkbox, and the engine's own
CLI made the same choice for the same reason: *show* is the value that has
to be written out to exist (Table 210's `/H` defaults to **true**, so an
absent entry means HIDE), and a single `Hidden` checkbox left unticked is
one misreading away from an operator believing they configured "show" when
they configured nothing.

### `fn submit_disclosure`

Two blocks, and the second is conditional:

- **Always**: what the declaration would cover — the six §12.7.5.2 facts
  nobody can guess, in `text::buttonaction::submit_disclosure`.
- **When the address is not `https:`**: that it is unencrypted. A
  **statement**, never a refusal — no scheme is blocked anywhere, because
  `https` appears zero times in ISO 32000-1 and refusing one would be pdfcer
  inventing a conformance requirement.

Both are off-canvas by construction: they are in a dialog, and the button
they describe is drawn on the page exactly as the saved file will draw it.

### `fn rows`

Returns nothing: the draft is the output, and the dialog reads
[`ButtonDoes::blocker`] itself when it decides whether Add may be pressed.
A `bool` return would be a second opinion about the same question.
