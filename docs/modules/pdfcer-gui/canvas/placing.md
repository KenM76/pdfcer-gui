# `canvas::placing` — **point at the page instead of typing coordinates**


> *"anything we are inserting like this should have an option in its
> dialogue box to place it with the mouse instead of by positional
> co-ordinates."*

Note *"anything we are inserting"*. That is a rule about a **class** of
dialog, not a feature for one window, and it is why this is a shared arm
rather than a drag added to `dialogs::insert_image`.

## What was actually missing, which is not "a drag"

Exactly one dialog in this crate asks for a page-space position
numerically — [`crate::dialogs::insert_image`], with four millimetre
spinners. Its own module header names this gap, gives three reasons for the
numeric route, and ends: *"A drag-to-place gesture is a second ROUTE to the
same action, not a replacement for this window, and it is the natural next
slice."*

Meanwhile the **canvas → dialog** direction already exists three times over
(`Action::BeginTextAnnot`, `FieldAction::Begin`, `open_scale_calibrated`)
and the **dialog → canvas** direction exactly once, hard-wired in
`app::frame` for the Set-scale window. So the missing thing was never the
gesture; it was a *general* way for a window to step aside, let the operator
point, and come back.

## The one design decision, and it is the whole file

**A dialog is hidden for exactly as long as a placement is pending for it,
and "hidden" is DERIVED from the pending record rather than stored.**

That is `canvas::tool`'s own space-bar idiom — *"no stored override and
nothing to restore"* — and it is chosen here for a reason with teeth: the
precedent this arm generalises **is already broken in exactly the way a
stored flag breaks.**

Press Escape during a Set-scale calibration **as it stood when this file
was written** and the key landed on `disarm_measure`. Nothing reopened the
window, and `close_scale` had already destroyed the half-typed ratio. The
operator was stranded with no route back, and no line of code was wrong —
the cancel path simply was never one of the places anybody remembered to
reopen from.


⇒ With a stored `hidden: bool` this arm would inherit that, five times over:
a mode change through `tool::arm::retire_forbidden`, the Tool panel putting
the pen down, a ribbon control arming a different tool, the document
closing, Escape. Every one is a route somebody has to remember to clear a
flag on.

With `hidden` derived, **stranding is unrepresentable**. Whatever clears the
pending record — anything at all, including a route added next year by
somebody who has never read this file — the window comes back on the next
frame, because its absence was never a fact of its own. A lost cancel costs
one frame of missing window instead of an operator with nowhere to go.

## Rule 4

The preview drawn while a placement is armed is a **pre-commit affordance**
— the cursor — and is explicitly allowed. Nothing here marks applied
content, and the moment the placement commits it is an ordinary insert that
renders exactly as a saved one will.

## Item notes

### `const PLACING_MEMORY_KEY`

Beside `canvas::tool`'s own `TOOL_MEMORY_KEY`, and for the same reason: it
is per-window state with no meaning outside the frame loop, and putting it
on `OpenDoc` would make a *gesture* a property of the *document*.

### `const CANCELLED_MEMORY_KEY`

Separate from the result rather than an `Option` inside it. *"The
operator placed nothing"* and *"the operator has not finished yet"* are
different states, and a single slot would make them the same absence — the
distinction `crate::app::files::Picked` exists to preserve, applied here.

### `struct PlacedRect`

A named function rather than two inline casts, and not only for tidiness:
the narrowing happens at exactly one boundary — where `markup::band`'s
`f64` endpoints meet `egui`'s `f32` geometry — so a reader asking *"where
does the precision go?"* gets one answer instead of two identical ones with
an attribute apiece.

The loss is not material here. These are page coordinates on a sheet
measured in points, where `f32` carries about seven significant figures
against a largest sensible magnitude in the low hundreds of thousands.

### `struct PlacedRect`

A newtype rather than four `f64`s in the slot: the four numbers have an
order and a meaning, and a tuple of them is a thing three call sites could
each get subtly wrong.

### `fn a_cancellation_is_not_an_absent_result`

The property `Picked` exists to preserve, asserted here because the
tempting simplification — one `Option<Rect>` slot where `None` means
cancelled — makes "they pointed nowhere" and "they have not pointed
yet" the same observation, and the second is true on almost every frame.

### `fn cancelling_nothing_reports_nothing`

Load-bearing: Escape consults this on every press, and a `cancel` that
reported success unconditionally would swallow the key from the three
claimants below it in `canvas::keys`.

### `fn a_click_places_a_corner_and_a_drag_places_a_box`

The regression test for the defect the driven check found on its first
run: `click` took the canvas point it was handed and wrote it into a
`page_tree::Rect` unconverted, so a placement near the TOP of the sheet
was recorded near the BOTTOM. Nothing refused it — a mirrored
coordinate is a perfectly ordinary number on a perfectly ordinary page.

Note what the previous version of this test asserted: that the rect
carried the numbers passed in. That is true of the broken build and of
the fixed one, because it was a test of the *plumbing* on a function
whose defect was the *space*. This one asserts the flip by magnitude —
canvas y 200 on an 800 pt page is PDF y 600 — which no unconverted
build can satisfy.
