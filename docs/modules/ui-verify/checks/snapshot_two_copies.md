# `ui-verify/checks/snapshot_two_copies`

`the_snapshot_boxs_two_copies_choose_its_text` — the Clipboard tab's two
snapshot copies each force their text form whatever the Snapshot setting says
(O379): *Copy snapshot, text as drawing* places vectors with no text in them,
*Copy snapshot, text as text* places vectors that keep it. It shares the
launch, the rig and the box reader of [`snapshot_box`](snapshot_box.md) and
the clipboard clear of [`snapshot_copy`](snapshot_copy.md), and runs under
`--no-input`.

The clipboard is snapshotted before the run and restored after it with
`ClipGuard`, unless another program wrote it in between.

## Steps

1. Arm View ▸ Snapshot and lay a box from (0.02, 0.02) to (0.98, 0.98) of the
   page on `blank-overhang.pdf`, so its labels and title-block text are
   inside.
2. Open the Clipboard tab, clear the clipboard, click
   `ribbon.item.edit.copy_snapshot_outlines`, and judge it as *no text*.
3. Clear the clipboard, click `ribbon.item.edit.copy_snapshot_text`, and judge
   it as *text*.

Judging reads that click's `clipboard-snapshot-copy` line (the first, then the
second) and requires:

- no `clipboard-snapshot-copy-refused` line, and `vectors=cut`;
- an SVG on the clipboard;
- for *no text*: `text_runs=0` **and** no `<text` element in the SVG;
- for *text*: `text_runs` above 0 **and** at least one `<text` element.

Both counts are required together because each alone has a way to lie: the
trace's count is what the copy reported, the SVG's is what landed. A form left
at the Snapshot setting's default fails one of the two clicks, since the
setting picks one form and the two buttons need opposite results.
