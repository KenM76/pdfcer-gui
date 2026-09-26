# `pdfcer-gui/text/panels/layers`

`text::panels::layers` — **every sentence pdfcer says about which layer a
selection is on**, in one module.

# Why one module, when the sentences appear on two surfaces

The answer is shown in two places and they are deliberately not the same
width:

| surface | form | why it exists |
|---|---|---|
| the **Layers panel**, under the count | the long form, a whole sentence | the panel is where the operator went to ask |
| the **status bar**, appended to the selection line | a short clause | ★★★ **the canvas is the primary surface, never a panel** — clicking the object must reach the answer with no panel open |

Two surfaces saying one thing is `DEFECTS.md` D5's shape exactly: *the same
concept implemented in two places, and the copies drift.* The drift here
would be invisible and expensive — a bar reading *"not on a layer"* beside
a panel reading *"pdfcer could not tell"* is a contradiction the operator
can only resolve by deciding the program is broken.

⇒ So both forms are generated **here**, from the same
[`Membership`](crate::panels::layers::highlight::Membership), by two
functions whose matches are exhaustive. A new state cannot be added to that
enum without both of these failing to compile, which is the mechanism this
project keeps re-learning it needs: **a hand-written list inside a
completeness sweep is not a sweep.** `check-ui-strings.sh` would have said
nothing about a missing arm; the compiler says everything.

# ★★ The register: a measurement, never a hedge

*"This may be part of a larger object"* is a disclaimer, and a disclaimer
printed on every selection teaches the operator to skip the line. *"This
object holds 1,194 parts"* is a number he can act on, and it is silent on
the object that holds one. Every sentence below is written to that rule —
see [`layer_selection_granularity`], which is the one that carries his own
finding back to him.

# ★★ Rule 4 lives in what these sentences are FOR

None of this is drawn on the drawing. An inference the operator cannot see
— an unresolvable `/OC` section, a group the document never listed, an
object nested deeper than the leaf list can see through — still owes a
report, and this is where that report is worded. **Render normally; report
separately. Both.**
