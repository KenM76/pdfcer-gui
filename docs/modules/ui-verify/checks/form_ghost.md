# `ui-verify/checks/form_ghost`

`the_outline_follows_the_pointer_and_is_what_gets_placed` — **the live
preview of a form field's size and placement** — `OPERATOR_REQUESTS.md`
O203.

# The operator's report

> *"when placing form items, there should be a live preview of their size
> and placement of what we will get if we just click once to place them."*

Read the last clause carefully. He did not ask for *an outline*; he asked
for the outline **of what he will get**. A preview that tracks the pointer
and promises a rectangle the click does not write is worse than no preview,
because it is a promise the program then breaks.

# The three assertions, in the order they build on each other

1. **There is an outline at all**, with a form tool armed and the pointer
   over the canvas: a `form-ghost` line.
2. **It follows.** Move the pointer by a known distance in PDF points and
   the rectangle moves by the same distance. A ghost drawn once and left
   where it was satisfies assertion 1 completely.
3. **It was the truth.** Click, let the placement dialog accept its own
   defaults, and the rectangle the application publishes for the field it
   just authored is the rectangle the ghost was drawing. This is the one
   that makes the feature the feature.

# Why a screenshot cannot do any of it

A ghost that tracks the pointer while promising the wrong `/Rect` and a
ghost that promises the right one while drawn in the wrong place are
**the same picture** at the moment of the screenshot, and the second is
only visible one frame later when the field appears somewhere else. The
trace line carries both rectangles — the PDF `/Rect` the click would write
and the screen box being drawn — precisely so a harness can separate them.

# And why the click point is checked against every corner

`formfield::ghost::click_rect` anchors the click at the **lower-left**
corner in PDF space, to match what a drag does. On a `/Rotate 90` sheet the
same PDF corner is a different corner of what the operator sees, so the
check asserts the click point is *a* corner rather than naming one. An
assertion that named the corner would be a claim about the fixture's
rotation dressed up as a claim about the feature.
