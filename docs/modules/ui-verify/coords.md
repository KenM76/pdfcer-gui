# `ui-verify/coords`

**The coordinate seam.** Document space in, screen pixels out — and the
rule that a check may never write down the screen pixels itself.

# The rule

> **Scripts are written in document coordinates. Never in absolute screen
> coordinates.**

`PROJECT_PLAN.md` §4.3 lists this as one of three prerequisites that
"belong in S1, not later", ahead of the panel-flexibility work that would
otherwise invalidate it.

# Why the rule is not merely tidiness

Two reasons, and the second is the one that has already cost this project
real time.

**1. Every screen coordinate in this application is about to become
variable.** `MODES_AND_PANELS.md` puts multi-column docks, a tab-overflow
menu, named workspaces, collapse-to-icon-rail and eventually tear-out on
the roadmap. Each one changes where the canvas begins. A harness whose
scripts say `click at 819,513` is a harness that has to be re-baselined
after every layout change — and the re-baselining is manual, so in practice
it does not happen and the checks quietly stop testing anything.

**2. A stale screen coordinate is symptom-identical to a broken coordinate
conversion.** This is the part that matters. When a click lands on empty
canvas instead of on the object, the trace shows a hit test returning
nothing — which is *exactly* what a genuinely broken document-to-screen
conversion looks like. The recorded outcome in this codebase was a
coordinate-space defect filed and then retracted: the conversion was
correct all along and the harness was pointing at the wrong pixel.

A false defect is worse than no defect. It consumes an investigation, and
it teaches everyone involved to distrust the harness — after which the
harness's true reports get discounted too.

So the fix is structural rather than advisory: a check *cannot* write a
screen coordinate, because [`ScreenPoint`] has private fields and no
constructor. The only way to obtain one is to start from a [`DocPoint`] and
pass it through a [`CanvasMapping`] the application itself supplied this
run, and then through the [`WindowFrame`] measured from the live window.
If the application did not supply a mapping, there is no [`ScreenPoint`],
and the check SKIPs saying so — which is the honest answer, and is not the
same answer as "the click missed".

# The four spaces

```text
  DocPoint          PDF user space. Page index + (x, y) in points,
                    origin BOTTOM-LEFT, y growing UP.
                    ── written by the check author. Stable across every
                       layout change, every window size, every DPI.
       │  CanvasMapping::doc_to_window   (needs: the page's height, and the
       ▼                                  canvas rect + zoom from the trace)
  WindowPoint       egui logical points, relative to the window's CLIENT
                    origin, y growing DOWN.
                    ── the space the application's own trace speaks in.
       │  WindowFrame::to_screen         (needs: the live window's client
       ▼                                  origin and its DPI scale)
  ScreenPoint       Physical desktop pixels. The only thing the OS input
                    API accepts, and the only space a check may not name.
```

(The fourth is [`crate::geom::PixRect`], the screenshot's own pixel space,
which shares an origin with the captured region rather than with the
desktop. It is handled by [`WindowFrame::client_pixels`].)

# The y-flip happens exactly once

PDF user space has its origin at the bottom-left with y growing up. egui
has its origin at the top-left with y growing down. That flip is performed
in [`CanvasMapping::doc_to_window`] and nowhere else in this crate. Every
codebase that flips y in two places eventually flips it twice on one path,
and the resulting bug is a mirror image that looks like a rounding problem.

# What is verified, and what is assumed

Stated separately, because this project has recorded the cost of a comment
that asserts a cause nobody tested.

**Verified** (against `D:\Dev\pdfcer`'s trace and its `tools/gui-drive.ps1`
notes): the canvas trace line carries `rect=` (the image rect in window
logical points) and `zoom=`, and the conversion
`window = rect.min + canvas_point * zoom` with `canvas_y = page_height -
pdf_y` is the one that script's own header documents for picking points.

**Assumed, and NOT verified here**: that `rect=` already accounts for the
scroll offset — i.e. that when the view is scrolled, the image rect moves
rather than the content moving inside a fixed rect. The canvas line also
carries an `off=` scroll offset, and if the assumption is wrong, every
conversion is wrong by exactly that offset whenever the view is scrolled.

[`CanvasMapping::scroll`] exists to hold that correction, defaults to zero,
and is applied if a profile supplies it. **The falsification test**, for
whoever gets there first: drive the same document point twice, once
unscrolled and once after a `Scroll` step, and compare the resulting
`vector-click canvas=` values. If they differ by the scroll amount, the
assumption is wrong and the profile should name the scroll field. Until
someone runs it, the checks stay at scroll zero — which is why every check
in [`crate::checks`] operates on an unscrolled view and says so.

## Item notes

### `fn user_to_canvas`

With a traced [`PageFrame`] the mapping is the real one, `/Rotate` and
crop origin included. Without it, the historical behaviour is preserved
exactly: `x` unchanged, `y` flipped against the page height. That
fallback is not a compromise so much as a statement of what the legacy
binary's trace can support — it cannot emit a crop box, so the harness
cannot honour one.

### `fn mapping`

Deliberately left frameless. These tests are the record of what the
fallback does, and a build whose profile names no crop field still takes
this path — so it has to keep being measured.

### `fn a_turned_page_does_not_get_the_flip`

The falsification: the naive `(x, height - y)` answer for this input is
`(100, 542, 300, 592)`, which this asserts it is NOT. Without the
inequality the test passes on the arithmetic the method was written to
replace.

### `fn each_crop_corner_lands_on_the_matching_canvas_corner`

This is the falsifiable form of the whole fix. A mapping that
ignored `/Rotate` — the one that shipped — sends the crop box's four
corners to the canvas's four corners on `/Rotate 0` and to a transposed
set on every other, so the upright case alone proves nothing. Looping
over all four is what makes the test able to fail.

The expected canvas corner per rotation is derived from where the page
is *drawn*: at 270° the crop box's lower-left `(llx, lly)` appears at the
canvas's top-right, and so on around.

### `fn a_point_near_the_far_edge_of_a_turned_page_is_reachable`

The literal symptom: on `A-591.pdf` the harness refused every
`--doc-point` whose user-space `x` exceeded 792, because it measured
against the crop box's width where the canvas is 1224 wide. Here the
point is well inside the crop box and near the canvas's right edge, and
the conversion must produce a window position inside the image rect
rather than an error.

### `fn a_point_off_a_turned_page_is_refused_in_the_words_of_user_space`

A point outside the crop box is still refused — that guard is not
relaxed. What changed is the message: it has to say which space it
measured in and what the rotation was, because the failure a rotated
page produces looks exactly like a typo in the check.

### `fn aiming_at_a_declared_rect_takes_its_centre_in_desktop_pixels`

The two conversions are next to each other and differ by exactly the
origin term, which is the mistake worth pinning: a capture is of the
client area and shares its corner, whereas the input driver works in
desktop pixels and does not. Getting them the wrong way round is
invisible on a maximised window at the top-left of the primary monitor —
i.e. on the machine anybody would test it on.
