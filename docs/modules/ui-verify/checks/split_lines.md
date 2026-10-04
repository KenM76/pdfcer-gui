# `ui-verify/checks/split_lines`

`a_text_object_splits_into_lines` — the canvas object menu's *Split into
lines* cuts one text object into one per line; Ctrl+Z rejoins them; a text
object the engine refuses to cut says why.

# What it drives

Two launches, each on a copy of its fixture in the output folder.

1. `fixtures/stacked-labels.pdf`: one text object, three lines placed by `Tm`
   then two `0 -20 Td` steps — the CAD pattern the verb exists for. Edit mode,
   click the middle line, right-click it, press
   `menu.item.canvas.object.format.split_text_lines`.
   `split-text-lines-applied … object=0 cuts=2 pieces=3 disclosed=1`.
   `disclosed=1` is the engine plan's inference disclosure, which goes to the
   status line. Then Ctrl+Z must trace `undo-applied`: one undo entry.
2. `fixtures/quote-operator.pdf`: two lines, the second drawn with `'`. The
   row is offered, because the object model cannot see the operator (see
   `canvas::runsplit`). The press must trace no `split-text-lines-applied`
   and must trace `split-text-lines-declined reason=line-show-operator`, the
   record behind the status-line sentence.

The second half is what makes the partial preflight safe: it proves a refusal
only the content stream can see still reaches the operator.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.
