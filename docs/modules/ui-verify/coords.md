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

### `struct DocPoint`

**This is the only spatial literal a check may write.** It is stable under
every layout change the roadmap contemplates, because it describes the
document rather than the window.

### `struct PageFrame`

[`DocPoint`] has always been documented as *PDF user space*, and until this
type existed the conversion did not honour that: it took `p.x` as a canvas
coordinate unchanged and flipped `p.y` once against the page height. For an
upright page whose crop origin is `(0, 0)` — every fixture in this
repository — that is exactly right, which is why it survived.

It is wrong for every other page, and it was measured wrong on the
operator's `A-591.pdf`. That sheet's `/Rotate` is **270**, so the canvas is
1224 × 792 while the crop box is 792 × 1224. Consequences, all silent:

* every `--doc-point` landed at a transposed position, so a check that
  *reported* aiming at (396, 612) actually aimed somewhere else entirely;
* the bounds check refused any `x > 792`, i.e. **the right-hand third of the
  canvas was unreachable by the harness**, reported as "outside the page
  box" — which reads as a typo in the check, not as a defect in the mapping;
* a check that then found nothing there would have blamed the application.

This is the same defect, in the harness, that O174 was in the renderer:
one `height - y` subtraction standing in for a rotation. Finding it twice in
one afternoon is the argument for this type existing at all — the mapping is
now written once, in terms the PDF actually uses, instead of open-coded as
arithmetic at each site.

# The two spaces

| space | origin | y | `/Rotate` |
|---|---|---|---|
| **PDF user space** | crop box lower-left | **up** | not applied |
| **canvas space** | page top-left as drawn | **down** | already applied |

`canvas space` is what the application lays out in and what `rect=` and
`zoom=` are expressed against, so it is the space the window conversion
needs. [`Self::user_to_canvas`] is the bridge.

### `fn new`

A `/Rotate` that is not a multiple of 90 is not representable and is
treated as 0 — the same reading the application takes, so the harness
and the application agree about a malformed page rather than disagreeing
about it.

### `fn canvas_extent`

A quarter turn swaps them. This is the number a bounds check must use,
and using the crop box's own width instead is what made a third of the
canvas unreachable.

### `fn user_to_canvas`

The coefficients mirror `pdfcer-render`'s own region geometry table,
which is the only authority on how this application draws a turned page.
They are the *inverse* of the shell's `render::region::PageFrame::
canvas_to_user`, and the pair is falsified together by that module's
round-trip test — this copy exists because `ui-verify` deliberately has
no `pdfcer-core` dependency (see its `Cargo.toml` header) and because a
harness that shares the code under test cannot measure it.

### `struct WindowPoint`

Fields are readable — a failure report should be able to say *where* it
clicked — but the type is only ever produced by
[`CanvasMapping::doc_to_window`]. There is deliberately no public
constructor: a check that could build one directly could write a window
coordinate literal, which is the same defect as a screen coordinate literal
wearing a different hat.

### `struct ScreenPoint`

Private fields, no constructor, produced only by [`WindowFrame::to_screen`].
That is the enforcement mechanism for this module's rule; everything above
is the explanation of why it is worth enforcing.

### `struct CanvasMapping`

Constructed from a trace, never by hand. If the application did not emit
what this needs, construction fails and the caller SKIPs — see
[`CanvasMapping::from_trace`].

### `fn from_trace`

# Errors

Every failure here is a **precondition**, not an assertion: it means
the harness cannot aim at all, so the caller must SKIP rather than FAIL.
Each message names the specific missing field, because "no mapping" is
useless to whoever has to add it.

### `fn user_rect_to_canvas`

All four corners go through [`Self::user_to_canvas`] and the extremes
are taken. Mapping only the two named corners and calling the result a
rectangle would be the upright assumption in a new place: a quarter
turn sends the lower-left corner somewhere that is neither lower nor
left.

The result is `(min_x, min_y, max_x, max_y)` in canvas space, which is
the space the application's own rectangles are published in, so a
caller comparing a traced `/Rect` against a traced canvas rect can do
it without arithmetic of its own.

### `fn doc_to_window_off_page`

# Why this exists, and why it is a second entry point rather than a flag

`OPERATOR_REQUESTS.md` O92: *"we should be able to select things offside
of the page, especially since I sometimes drop objects there, and when I
do I can't get them back."* A check for that has to drive a rubber band
**into the grey margin**, and [`Self::doc_to_window`] refuses every point
outside the media box — correctly, for every other caller.

That refusal is not softened and its reasoning is untouched: a point
outside the page is almost always a check aiming at the wrong sheet or
converting against the wrong geometry, and silently allowing it would let
those land somewhere plausible and wrong. **This is the narrow, named
exception**, and a caller has to say the words to get it.

# What is NOT relaxed

A bound stays, and it is the **viewport's**, passed in — see the
comment in the body for why bounding against `image_rect` instead
rejects the entire class this function exists for. A point off the page
can still be clicked, because the grey margin is part of the canvas
widget; a point off the *viewport* cannot be clicked by anybody, and
clamping it would land on an edge and hit-test nothing.

Coordinates may be negative. The flip is the same one line, and a
negative `x` produces a window position left of the page's own origin,
which is exactly where a dropped object sits.

# Errors

Wrong page, or a point that is off the **viewport** rather than merely
off the page.

### `struct WindowFrame`

Measured per run, never assumed. A window that the window manager placed
somewhere other than where it was asked, a DPI change between runs, a
second monitor at a different scale — all of these change these numbers,
and all of them are invisible to a harness that hard-codes them.

### `fn logical_to_capture_pixels`

This is the counterpart of [`Self::to_screen`] for areas rather than
points, and it is what makes the application's own `ui-rect`
declarations usable as a pixel check's region source.

## The two spaces, and the one thing that separates them

A traced rect is in **window logical points**, relative to the client
area's top-left corner — that is where egui's coordinate system starts,
so `[[8.0 8.0] - [1092.0 792.0]]` means "eight points in from the left
and top of the client area". A capture taken by [`crate::capture`] is
of exactly the client area, so its pixel origin is the *same* corner.

Therefore the whole conversion is the DPI scale. No origin term appears
here, and its absence is deliberate rather than forgotten: adding
`client_origin` would be correct for a desktop-space rectangle and is
wrong for this one, and the resulting regions would be offset by the
window's position on the desktop — which is zero on a maximised window
at the top-left of the primary monitor, i.e. exactly the configuration
a developer would test on.

## Clipping, and what an empty result means

The result is clamped to the capture. A region that lies **entirely**
outside it comes back with zero area, and the caller must treat that as
a finding rather than as a measurement: the application declared a
region that is not on screen, which is the clipped-out-of-its-pane
defect `PROJECT_PLAN.md` §4.3 prerequisite 2 exists for. It is
emphatically not "contrast 1.0".

### `fn offset_from`

# Why this is on `Frame` and not a method on `ScreenPoint`

`coords`' standing rule is that **a coordinate is produced by a
conversion and never assembled**, and a bare `ScreenPoint::offset` would
be exactly the assembly the rule forbids — it would let any caller
invent a screen position out of arithmetic and a hope.

This is deliberately narrower: a *displacement in screen pixels from a
point the application itself published*. That is what a drag is, and what
a sweep looking for a neighbouring control is. Living on `Frame` keeps it
beside `declared_at`, which is the other member of the same family —
both take an application-supplied anchor and move within it.

It does not clamp. A sweep that walks off the window is caught by the
application not responding, which is the honest answer; clamping would
silently retry the same point and report a false negative.

### `fn declared_at`

`(0.5, 0.5)` is [`Self::declared_center`]; `(0.75, 0.5)` is
three-quarters across, vertically centred.

# Why this exists rather than callers building a `WindowPoint`

Because `WindowPoint`'s fields are private and deliberately so — this
module's rule is that a coordinate is produced by a conversion, never
assembled — and a check that needs to aim at *part* of a control was
otherwise stuck reaching for the centre.

The case that forced it is real rather than hypothetical:
`checks::pages_drag` has to release the pointer over the **right half**
of a page tile, because the Pages panel resolves the nearer vertical
edge and the two halves mean two different landing boundaries. Aiming
at the centre would be aiming at the one place the answer is undefined.

The fractions are **not** clamped. A caller asking for `1.5` means a
point outside the control and is entitled to it — that is how a check
aims *beside* a widget rather than at it — and silently correcting it
would produce a click at an edge the caller did not choose, which is
the class of failure that reads as the application misbehaving.

### `fn client_logical`

# Why this is not [`Self::client_pixels`] divided by the scale

It is, arithmetically — and the point of having it as its own accessor
is that a caller comparing a published rect against the window must not
have to remember which of the two spaces it is in. `ui-rect` carries
logical points; `client_pixels` carries desktop pixels; a check that
compared one against the other would pass or fail by the display's scale
factor, which is a property of the machine the suite happens to run on.

Used by the "is it actually visible" assertions. **Drawn is not seen** —
a widget below the fold publishes a perfectly good rect.

### `fn layout_probe_point`

# This is the ONE exception to this module's rule, and it is narrow

Everything else in this crate refuses to turn a window-relative
quantity into a screen point without a document-space mapping. This
function does exactly that, and it exists for one reason: **some
applications only report their layout when something happens.**

The old GUI is one of them. Its `canvas` trace fires on
`pressed || released || down || zoom` and on nothing else, so a freshly
opened document produces no canvas rect at all — and without a canvas
rect there is no document-to-window mapping, and without that mapping
there is no click. The harness cannot aim until the application has
spoken, and the application will not speak until the harness clicks.

So: one **layout probe**. A click at the client-area centre, whose only
purpose is to make the application report where its canvas is. It is
explicitly not an assertion, it does not care what it hits, and the
real document-space click follows it and replaces whatever it selected.

## The assumption it rests on, stated

That the centre of the client area is the canvas. True of every layout
in `MODES_AND_PANELS.md` — the document is the centre of a document
editor — and it fails safe: if the probe lands on chrome, no canvas
event appears, and the check SKIPs with a reason naming exactly that.
It cannot produce a *wrong* mapping, only no mapping.

## The right fix — which has landed, for one of the two binaries

The application should trace its canvas layout **unconditionally**, at
least once per document open, rather than only on pointer events. Then
the harness reads the rect from a quiet trace and the probe disappears.
That was `PROJECT_PLAN.md` §4.3 requirement 1, discovered by building
this harness rather than by reading the code, and the new application
implements it: its `canvas` line is built every frame and emitted
through a de-duplicating gate that is cleared on document open, so
there is a line before any input is delivered. A run against it never
reaches this function — confirmed by the absence of the "no `canvas`
event yet" note in its reports.

The probe stays for the **old** binary, which still traces only on
pointer events and is still the thing the D1 reproduction is driven
against. Deleting it would delete the acceptance evidence.

### `fn for_image`

Used by `--image` mode, where the "window" is a PNG somebody captured
earlier. Its origin is (0, 0) and its scale is 1.0, so fractional
regions resolve against the image exactly as they would against a
client area — which is what lets one region set be checked against a
dated screenshot and against a live run without being written twice.
