# `ui-verify/checks/fit_left_by_a_pan`

`a_pan_keeps_the_fit_and_the_resize_keeps_the_position` — **fit, pan away,
resize: the view stays where the operator put it AND the fit is still
live.**

# ★★★ The request changed by one clause, and this file is the record


> *"if the canvas window is resized the pdf should resize to match unless
> the person has changed the zoom **or panned around**."*


> *"unless I have **manually changed the zoom** after clicking one of the
> preset options, the pdf should maintain whichever option was selected."*

The pan clause is gone, and the same message says why it could go:
*"whatever area was centered in the current canvas should stay centered."*

## ★★★ Why the clause was only ever load-bearing by accident

A fit is a rule about **zoom**; where the operator is looking is a rule
about **position**. Until O78 nothing owned the second, so a resize under a
live fit *re-placed* the view — and the only defence available for an
operator who had panned somewhere deliberately was to stop them being in a
fit at all. Leaving the mode was a **proxy** for defending the position.

`canvas::fit::placement` now preserves the centred page point across any
viewport change, in or out of a fit. The proxy is unnecessary, and keeping
it would cost him the thing he has now asked for twice: a page that stops
re-fitting the moment he drags it an inch.

## ★★ What this check had to gain to stay honest

Steps A–D are **unchanged**, and they still have teeth: preserving an
off-centre point keeps it off centre, so "the margins are not equal" is
still the right assertion and still fails a build that re-centres.

But it passes on BOTH builds — the old one did not re-centre because it had
dropped the fit, the new one does not re-centre because it preserves the
centre — so on its own it would have been a check that survived the change
instead of testing it. **Step E is the falsifier**: the page's drawn width
must have CHANGED across the resize, because a live fit re-scales from the
viewport every frame. On the pre-O78 build the pan froze the zoom and the
width is identical, and step E fails by name.

⇒ Nothing was deleted. A check written to an earlier reading of a request
is evidence about that reading, and the honest amendment is to add the
assertion that separates the two rather than to remove the one that no
longer distinguishes them.

## ★★ Why the HAND tool, and why the primary button

`canvas::input::pan_delta` treats two gestures as one pan: the middle
button always, and the primary button while the hand tool is active. This
harness has no middle-button driver — a third gesture-class hole, found the
same day as the secondary click and the window resize — so the check arms
the hand tool and uses the primary drag it already has.

★ That is not a workaround around the subject: `pan_delta` is one function
and both buttons reach it, so a build that dropped the fit for one and not
the other is not reachable. The check drives the door that exists.

## ★★★ The wheel is deliberately NOT this gesture, and the sibling proves it

Scrolling a fit-width document is how every reader in the class is read,
and a wheel notch that dropped the fit would stop the page re-fitting the
moment anybody looked at the second half of it.

⇒ `a_fit_command_puts_the_page_on_screen` **wheel-scrolls into the
pasteboard and then asserts the fit still places the page**. So the pair
pins both directions: the wheel keeps the fit, a pan leaves it. Neither
check can be satisfied by a build that treats all view movement alike.


With `doc.view.set_fit(FitMode::None)` removed from `canvas::offset`'s pan
arm and nothing else changed, this check fails and reports:

> `margins l=8.0 r=8.0 t=108.4 b=108.3`

— dead centre on both axes, against `l=-39.0 r=-105.0 t=120.4 b=-27.3` for
the correct build. So the assertion is live and the tolerance is nowhere
near either result.

★★ **It would NOT have caught the state this shipped in before today**, and
that is worth saying rather than leaving somebody to assume otherwise. Then,
a resize re-placed nothing at all, so a panned page was not re-centred and
this check would have passed for the wrong reason. It has teeth only against
a build that re-places on a resize — which is the other half of O55, and the
state that existed for the ten minutes between the two edits.

⇒ **A guard written with a fix guards the fix, not the original defect.**
The original is covered by its sibling; this covers the regression the fix
made possible.

# The sequence

| # | step | oracle |
|---|---|---|
| A | press **Fit page**, note where the page is | `canvas … rect=` |
| B | **pan** with the hand tool, assert the page actually moved | the rect moved |
| C | **resize** the window | `canvas-viewport` changed |
| D | the page is **not** re-centred | its margins are still lopsided |

★ Step B asserts its own precondition, for the reason the sibling states: a
pan that did nothing would leave the page centred, and step D would then
pass against a build that re-centres on every resize — measuring nothing.
