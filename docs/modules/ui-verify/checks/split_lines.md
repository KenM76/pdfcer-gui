# `ui-verify/checks/split_lines`

`a_text_object_splits_into_lines` — the canvas object menu's *Split into
lines* cuts one text object into one per line; Ctrl+Z rejoins them; a text
object the engine refuses to cut is greyed before the press.

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
   object model cannot see the operator; the engine's preflight
   (`EditSession::text_object_split_refusal`, asked by `canvas::runsplit::refresh`)
   can. Selecting the text must trace
   `split-preflight … refusal=line-show-operator`, and pressing the row must
   trace neither `split-text-lines-applied` nor `split-text-lines-declined`:
   the row is greyed, so the click reaches no press.

The second half fails on a build that greys only what the object model sees:
the row stays pressable and the press traces the decline.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.
