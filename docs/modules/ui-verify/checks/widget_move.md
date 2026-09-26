# `ui-verify/checks/widget_move`

`dragging_a_form_field_moves_it` — **a form field's box follows the
pointer.**

# What this is for

The operator's standing instruction that week was *"work on form field
editing next and the rest of the features required for editing."* A field
could be placed, selected, renamed, deleted, and its position and size typed
into four boxes with an Apply button — and **dragging it did nothing.**

★★ Four numbers and an Apply button are a form for editing a rectangle.
**Dragging is how a person moves a box**, and every program in this class
does it; the typed fields are the precise route, not the primary one.

## ★★★ Why the defect was found rather than reported

Ten days earlier the same shape was found on the annotation surface: the
canvas forked on *"is an annotation selected?"*, the only module on that
branch answered for ce dimensions, and a drag on a stamp was **consumed and
discarded** — no move, no decline, nothing.

This was the identical state one surface along, and it was found by asking
*"where else does this shape exist?"* ⇒ **A class of defect that has been
named once is cheap to look for; the same class waiting for an operator to
trip over it is not.**

★ The routing here was worse in one way: a widget is deliberately **not** an
annotation selection — `canvas::selection::annot` excludes `/Widget` so the
form surface owns the press — so a selected field did not even reach the
annotation branch. It fell into the CONTENT branch, where the mover found no
content, and was dropped there.

## ★★ The oracle is the engine's own `move-widget-applied`, with BOTH deltas

`dy` is the one with a sign convention to lose: PDF user space increases
**upward** (§8.3.2.3) and every screen coordinate in this harness increases
downward, so a term dropped in the conversion shows up as a move that
travels in x only. The drag is diagonal by construction and both axes are
asserted.

## ★ Why it authors its own field rather than needing a form fixture

`edit.form_text_field` places one, and `PDFCER_DIAG_FORM_ACCEPT=1` makes the
placement dialog press its own Add — the same seam `form_field` uses. So this
runs on any `--pdf` with a page, which is what keeps it in the ordinary
sweep rather than behind a fixture nobody remembers to pass.
