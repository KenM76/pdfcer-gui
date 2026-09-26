# `ui-verify/checks/bezier_handle`

`bezier_handle_drag_changes_a_curve` — **a control point follows the
pointer and the engine rewrites the segment**, driven end to end.

# What this is for

`pdfcer`'s `gui` column ticked *"edit a Bézier handle"* `[x]`. Their sweep of
2026-08-19 corrected it to `⬜ nothing` — one of six rows that were true of
the **old** in-repo shell and became false, untouched, when the column's
referent moved to this build.

Nothing was blocking it. `EditSession::move_handle` has existed since Pass
30.1 with a `Handle` enum, a planner, a `v`/`y` re-spelling path and a
disclosure contract. What was missing was a way to **see** a handle and a
way to **grab** one, and both are this shell's.

# Why this cannot be a unit test

Five links, and three exist only in a running window:

| # | link | its own test |
|---|---|---|
| 1 | a selected anchor on a curve draws two marks and a tether | `overlay` — nothing; it is pixels |
| 2 | a press within 8 px of a mark becomes `DragKind::Handle` and **not** a move | `handledrag::at` — the distance, not the routing |
| 3 | the handle outranks `Grip::Move`, which `grip_at` returns for the same press | **nothing** |
| 4 | the pointer's canvas position converts to PDF user space against **this** frame's mapping | **nothing** |
| 5 | `move_handle` rewrites one operator and the page redraws | `pdfcer-core` |

Link 3 is the one that would fail silently and plausibly, and it is not
hypothetical: a handle sits **inside** the selection's bounding box, so
`handles::grip_at` answers `Grip::Move` for every press on one. Without the
priority rule in `gesture::meaning`, every attempt to drag a handle moves
the whole object instead — which looks like a clumsy gesture, not a defect,
and is exactly the shape of the bug that made the corner *anchors*
undraggable until the eight scale grips were confined to the Object rung.

# The oracle

`handle-commit node=… side=… to=[x y]`, **plus** `move-handle` from
`vector_edit`. Two lines because they answer different questions: the first
says the shell decided a handle drag happened and where it put the control
point, the second says the engine accepted it. A build that computed the
right position and never reached `EditSession` writes the first and not the
second, and from a chair that is indistinguishable from the handle not
moving at all.

It is **not** enough to assert that `move-handle` appeared. A drag that
was routed to `DragKind::Move` would write `canvas-move` and `move-objects`
— different lines — so the check would fail, correctly. But a drag routed to
the handle and given the *anchor's* position instead of the pointer's would
write both expected lines and change nothing visible. So `to=` is asserted
to differ from where the handle started, which is the number a wrong build
gets wrong.
