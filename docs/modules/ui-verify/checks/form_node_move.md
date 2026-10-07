# `ui-verify/checks/form_node_move`

`an_end_point_inside_a_wrapped_drawing_can_be_dragged` — O288 item 2: a line
made part of the page (which `flatten_annotations` wraps in a form XObject)
has its end point dragged, and the drag applies.

## What it drives

`fixtures/form-xobject.pdf`, whose every mark is inside one form. Edit mode;
a click on the horizontal bar selects the form, a double-click enters it to
the bar (`canvas-selection first=leaf:N`), a second double-click enters the
bar's Part rung and publishes `canvas.anchor.N`; a double-click on
`canvas.anchor.0` picks that end point (`canvas.selected-anchor`), and a
drag moves it 30 px on each axis.

## Verdict

PASS when a `move-node-in-form` line with `n=1` follows: the funnel writes it
after the engine returned `Ok`. FAIL with the preview count and the
`canvas-move` line when it does not. Earlier rungs that cannot be reached are
SKIPPED with the trace line that shows why.

## Falsified

With the `NodeInForm` and `NodesInForm` arms removed from
`canvas::moving::drag`, the check FAILs: the end point is selected, one shape
previews, and no `move-node-in-form` line follows, because `action` refuses
with `Refusal::NodeNotFound`, which is silent. With the arms back, it PASSes
with `move-node-in-form page=0 n=1`.

## What it does not prove

The plural rung (`move-nodes-in-form`) beyond sharing the same lookup; the
paint and width of an in-form leaf, which have no engine verb (see
`ENGINE_BACKLOG.md`).
