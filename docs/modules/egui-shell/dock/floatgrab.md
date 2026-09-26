# `egui-shell/dock/floatgrab`

## Item notes

### `struct Settling`

One flag, kept in [`egui::Memory`]'s temporary data per panel, holding the
single fact this gesture needs from the previous pass: did it command a
move?

**Why a pass has to be declined at all.** The residual that drives
[`carry_to`] is the pointer's position minus the point it grabbed, both in
the window's own coordinates. Move the window and both ends of that
subtraction change — the grab because the window took it along, the pointer
because the platform re-reports it against the new origin — and the pass
that runs before those two readings agree reports a residual equal and
opposite to the one just acted on. Acting on it sends the window straight
back. Driving the real binary shows the full cycle in three passes: send
+14, send −14, settle at 0, with the window visibly snapping back and forth
once per pointer step for the length of the gesture.

**Why one pass and not a settled-residual test.** "Move only when the
last residual was zero" reads better and deadlocks: a platform that
declines the move — a window clamped to a monitor edge, a compositor that
places windows itself — leaves the residual non-zero forever and the window
never moves again. Skipping exactly one pass retries on the pass after,
so a command that did not take is simply re-sent.

The cost is that the window moves on at most every other pass. At frame
rates where a carry is usable that is not a rate the eye resolves, and the
alternative it buys out of is a window that shakes.

### `fn the_conversion_reproduces_the_application_windows_own_report`

The numbers are one drag sample from driving two real windows on
Windows 11 at `ppp = 1.0`: a child window 272 points narrower than the
pointer had travelled, and the *application's own* pointer report for
the same physical cursor — a witness the arithmetic never reads.

### `fn coinciding_origins_are_the_identity_and_prove_nothing_else`

Where the two windows share an origin the conversion is the identity,
which is correct — and is also exactly what omitting the conversion
looks like. Asserting it here records that the test above, with two
different origins, is the one doing the work.

### `fn a_carry_settles_where_a_delta_would_oscillate`

Off-window points are the ordinary case during a carry: the operator
is over the desktop, or over another application, for most of the
gesture. Clamping here would put every one of them on an edge, and the
dock would offer a drop the pointer is nowhere near. Whether a point
is over a compartment is [`super::super::geometry`]'s question, and it
needs the truth to answer it.
**Two frames of a carry settle, where a per-frame delta would
oscillate.**

The second frame is the whole point and is the one a delta-driven
implementation fails: the pointer has not moved physically, but the
window has, so the platform re-reports the cursor 20 points further
back in the window's own coordinates. A delta reads that as "the
pointer moved −20" and sends the window back where it came from,
giving a window that shakes for as long as the operator holds it.
Here the residual is zero and nothing is sent.

### `fn the_pass_after_a_move_is_declined_so_the_window_never_reverses`

Each row is `(outer, local)` as one pass reported them while the
pointer was walked down and right in steps of 14 — lifted from a driven
run, and containing the reversal that makes the ungated form judder:
pass 2 reports the residual as `−14` on a pointer that only ever moved
`+14`. With [`Settling`] declining that pass the window only ever
advances.

The assertion is monotonicity, not a list of positions. A position
list would be satisfied by an implementation that moved backwards and
forwards through the same values, which is the defect.

### `fn a_refused_move_is_retried_on_the_pass_after_the_declined_one`

The flag is cleared by the pass it declines, so a window that did not
move — clamped to a monitor edge, placed by a compositor that does not
take instruction — gets asked again on the pass after. The failure mode
this rules out is a carry that stops moving the window entirely and
gives the operator no way to tell it apart from a hang.
