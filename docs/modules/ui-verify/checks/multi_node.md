# `ui-verify/checks/multi_node`

`multi_node_move_moves_every_picked_anchor` — **Shift-picked anchors move
together**, driven against the operator's own drawing.

# What this is for

`pdfcer`'s `gui` column ticked *"multi-node select-and-move"* `[x]` for
months. Their own sweep of 2026-08-19 corrected it: *"objects move together;
nodes one at a time"*, one of six rows that were true of the **old** in-repo
shell and became false, without anyone touching them, when the column's
referent moved to this build.

It was false in an unusually quiet way. The selection model had held a
multi-node set since the Node rung landed — `SelectionState::pick_within`
adds a Shift-clicked anchor as its own entry — and `canvas::moving::subject`
read `entered_object()`, which is the **first** entry. Four anchors picked,
one moved. Nothing failed; both halves' unit tests passed; the capability
was present in the data model and absent from every consumer.

# ★★ And the operator could not see the anchors AT ALL

`FEATURES.md` recorded, against `view.show_points`, that *"this build draws
no anchor mark at any rung"*. So the Node rung could be entered, a set could
be picked, and there was **no surface anywhere** that said which points were
in it. The marks and the multi-node move landed together on 2026-08-19
because they are one feature: a set the operator cannot see is not a set
they can choose.

That is also what makes this check possible. An anchor's screen position is
a fact about the page's *decomposition* — a harness cannot compute it
without re-implementing the content walk — so before the marks existed there
was nothing to aim at, and this check could not have been written at all.

# The oracle

`move-nodes`, **plus** `canvas-move … action=MoveNodes` carrying more than
one entry. The second is the whole point: a build with the old bug raises
`MoveNode` (singular), reaches the engine, redraws, and looks like a working
drag — it just moves one of the four points the operator picked. A check
asserting only *"the move committed"* would pass on the defect.
