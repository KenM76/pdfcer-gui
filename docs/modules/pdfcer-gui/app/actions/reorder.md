# `app::actions::reorder` — putting a page's annotations in a new order

One verb, its own file under R2 rather than a section of
[`super::forms`]. `OPERATOR_REQUESTS.md` O99.

## Why it is worth its own file rather than a shorter comment

Because the interesting part is not the call — it is one line — but the
**three things the operator did not ask for** and which the engine reports so
they can be said. A tab order is a list of *fields*; `/Annots` order is more
than that, and every one of the three below is a consequence an operator
would not predict from the gesture they made.

## Two callers, one verb, and OPPOSITE surprises

The three disclosures below are written for the **form-field tab-order
panel**. [`arrange`] is the other caller, and it arrives from the other end
of the same array:

| the operator did | what they meant | what the engine reports | the surprise |
|---|---|---|---|
| dragged a row in the **tab-order** list | *this field comes second* | `non_widgets_moved` | the **drawing order** changed |
| pressed **Bring to front** on a mark | *draw this on top* | `moved - non_widgets_moved` | the **tab order** changed |

⇒ `/Annots` order is **two lists at once** — paint order for every
annotation and the tab sequence for the widgets among them — so whichever
one the operator was thinking about, the other moved. That is why the two
callers do not share a note set even though they share every line of the
call: the disclosure is *"the thing you were not looking at"*, and they were
looking at different things.

The second number is a **subtraction**, not a field. `AnnotsReorder`
reports `moved` and `non_widgets_moved`; how many widgets moved is
`moved - non_widgets_moved`, computed at the one call site that cares rather
than asked of the engine, because it is a fact about this caller's intent
rather than about the reorder.
