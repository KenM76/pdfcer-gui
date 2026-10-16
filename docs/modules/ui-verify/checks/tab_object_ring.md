# `ui-verify/checks/tab_object_ring`

`tab_walks_the_objects_on_a_clicked_page` — **after a click on the page, Tab
selects the page's objects in turn and Shift+Tab steps back** —
`OPERATOR_REQUESTS.md` O204, its canvas half.

# The drive

`three-squares.pdf` (three separate path objects on an upright 300 × 200 pt
sheet), opened in Edit, single page. Click blank paper above the squares, so
the page takes the keyboard and nothing is selected. Then Tab, Tab, Shift+Tab.

# The assertions

1. A `tab-object` line after each press. Without one, the check reports which
   of the two causes it was: no `tab-claim scope=object` (the press went to
   egui's focus walk, the ribbon) or a claim the ring spent on nothing
   (`tab-object-unmodelled` or `tab-object-empty`).
2. `stops` is at least 3, else SKIP: a ring of two cannot tell a step from a
   bounce.
3. The two forward presses land on different `to=` stops, both `backwards=false`.
4. Shift+Tab is `backwards=true` and returns to the first stop.
5. The last `selection-set` carries `via=tab-object`: the ring set the
   selection, not some other path.

# Why only a driven run sees it

Each unit test of `objring::advance` hands it a model. In the running app the
model is built only when something in the frame asks for it, and a claimed Tab
with nothing selected was not one of the reasons — so the ring was handed
`None` on every real press. This check failed on that before the fix in
`canvas::modelneed` and passes after it.

It depends on the scripted pointer reporting window focus: the page publishes
its claim on Tab only while `Response::has_focus` holds, and that ANDs in the
window's OS focus, which an off-screen window never has. See
`docs/modules/pdfcer-gui-base/pointerscripted.md`, *Window focus*.

# Falsified

- Without the scripted pointer's focus: FAIL, *never claimed for the object
  ring*.
- Without the `tab_claimed` model need: FAIL, *claimed and the ring spent
  nothing*, with `tab-object-unmodelled page=0` in the trace.
