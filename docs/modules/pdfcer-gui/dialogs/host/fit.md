# `pdfcer-gui/dialogs/host/fit`

## Item notes

### `const FIT_MARGIN`

See [`Host::fit`], whose first version had no such floor and grew the About
window from 560 px to 1,624 px in a few frames. It is a floor in BOTH
directions and that is the whole of its job: content within 8 pt of the
inner rect is treated as fitting, so a one-pixel rounding difference
between what egui laid out and what the compositor gave back cannot ask for
a resize -- and a resize, once asked for, changes the very number that was
measured.

## Rule 15

A **window** size in points. Neither a ce dimension nor a pdf dimension.

### `const FIT_BUDGET`

See the growth budget in [`Host::fit`]. The legitimate case settles in one
round trip and two covers a body that re-flows in response to the first;
three is one more than has ever been needed, and it is the difference
between a bounded nuisance and a window that grows for as long as it is
open.

### `fn fit_target`

# Why this is a free function

Because the growth branch could not otherwise be tested. `Host::fit` needs
a live viewport to read `inner_rect` and to issue the resize, so the whole
decision was reachable headlessly only in its no-op half — and the adjacent
test said as much in its own doc comment: *"only the no-op half is
reachable headlessly … the growing branch is asserted by the driven check,
which is the only place it can be."*

That was true of the *resize*, and it was never true of the *arithmetic*.
Splitting them costs one function and buys the convergence test that would
have caught the print dialog's runaway before an operator did: feed this
its own output and it must reach a fixed point.

# The contract

* `None` when the content already fits within [`FIT_MARGIN`] on both axes.
  The margin is a floor on what is worth acting on — below it the
  difference is noise between `min_rect` and a client size the window
  manager reports, and acting on noise is what creep is made of.
* Otherwise a size that is **never smaller than the current window** on
  either axis, and never smaller than `min_size`. Growth only: shrinking to
  content would fight the operator every time they enlarged a window, and
  would shrink a scrollable body to its own scroll viewport, which is
  circular by construction.

Note what this function cannot do, and why the budget in [`Host::fit`]
exists as well. Its answer is **idempotent** — feed it a window that has
already been grown to its content and it returns `None` — but idempotence
only holds if `content` stays put when the window changes. When the content
is measured *from* the window, every answer is new and correct in
isolation, and the sequence still runs away. No pure function of
`(inner, content)` can detect that; only a count of how often it has been
asked can.

### `fn fitting_a_window_that_already_fits_asks_for_nothing`

Only the no-op half is reachable headlessly — issuing the resize needs a
live viewport — and the no-op half is the one with the hazard in it.
Named rather than claimed: the growing branch is asserted by the driven
check, which is the only place it can be.

### `fn a_reopened_dialog_forgets_the_last_opening_s_fit`

Both of [`Host::fit`]'s guards live in `egui::Memory` keyed on the
dialog's id, and both are statements about **one opening**:

  * `fit_key` says *"I have already asked for this size"*, which on a
    brand-new window at its opening bid is false and suppressed the
    only resize that mattered;
  * `budget_key` counts growths, and a cumulative count means the
    fourth opening of a dialog in a session can never grow at all.

The remembered POSITION deliberately survives a close (G6). The fit
state deliberately does not, and this is the line that says which is
which.

⚠ What this test does NOT cover: that [`Host::show`] actually calls
[`Host::forget_fit`] on the opening pass. That needs a live viewport,
so it belongs to the driven check. The mechanism is asserted here; the
wiring is asserted there.

### `fn growing_to_fit_settles_after_one_step`

The half the test above says it cannot reach. It can, now that the
arithmetic is a free function: grow once, then feed the result back and
require silence.

### `fn content_measured_from_its_own_window_diverges_and_never_repeats`

Operator report, 2026-08-25: the print dialog *"keeps expanding its size
in little steps to infinity"* after pressing Print. The cause was a
footer row whose right-to-left button block reached the right edge of
whatever width it was offered, with a status label appended AFTER it —
so the row overflowed by the label's width no matter how wide the
window became.

This test models exactly that: content that is always `OVERFLOW` wider
than its window. Every individual answer [`fit_target`] gives is
correct, every one is a size it has never returned before, and the
sequence still diverges — which is precisely why the fix is a **count**
in [`Host::fit`] and not a smarter comparison here.

It is written as a test rather than a comment so that anyone tempted to
replace the budget with "just check the size is different" has to delete
an assertion that says why it will not work.
