# `ui-verify/checks/dropped_file`

`a_dropped_image_reaches_the_placement_window` — **drag-and-drop**, driven
through the one seam that can carry it.

# What this is for

The operator, 2026-08-19: *"also can't drag and drop a jpg file onto a new
pdf, and the insert image button doesn't insert it either."*

Half of that was false and half was worse than reported.

- **The button works.** `insert_image_places_a_picture` drives it end to end
  and passes, including on a real JPEG once this harness was taught to feed
  it one instead of a PNG it encoded itself.
- **The drop did nothing at all.** Nothing in the shell or in `egui-shell`
  read `dropped_files`. A file dragged onto the window was ignored, silently,
  with no cursor feedback on the way in.

★★ And the second made the first *look* broken. Both were tried in the same
minute; only one of them told the operator anything, so the reasonable
conclusion from the chair was that pictures do not work.

# ★★ Why this check needs an environment seam where others need none

Because a drop **cannot be synthesised by moving a mouse**. It originates in
Explorer and is delivered by the window manager as an OLE drag-drop
transaction; this harness drives a cursor and a keyboard and has no way to
begin one. Without `PDFCER_DIAG_DROP_PATH`, drag-and-drop would be the single
feature in this shell that R1 cannot reach — implemented, unit-tested, and
never once exercised in a running window, which is precisely the state R1
exists to forbid.

The seam is honest about what it does and does not prove:

| proved here | **not** proved here |
|---|---|
| the classification, the routing, the import, the placement window opening, the disclosure | that Windows delivers the drop to this window at all |

That second column is real and is stated rather than glossed. What closes it
is the operator dragging a file onto the build, which is how the gap was
found in the first place.

# The oracle

`dropped source=env path=…` — the shell saw a drop — **plus** the placement
window's own region. Two lines, because a build that read the drop and then
routed it nowhere writes the first and not the second, and that is exactly
the shape of the defect being fixed: a file that arrives and is ignored.
