# `annotquad` — where an annotation's artwork **actually** sits, as four
corners rather than as an upright rectangle

## The operator's sentence this module exists to answer

> *"the box outlined when an object is selected should be in the same angled
> orientation as the object."* — 2026-09-07, `OPERATOR_REQUESTS.md` **O147**


## ★★★ THIS MODULE WAS A WORKAROUND FOR ONE DAY, AND IS NOW A THIN ADAPTER


**`Pass 155.2` shipped that afternoon and gave us both**, and the engine's
reply answered the addendum in as many words: *"you were right that it is
the better shape … delete your matrix reader, your angle decomposition,
your `/AS` handling and your `MIN_BOX_EXTENT` copy."* All four are deleted.
What is left is the projection into canvas space, which is this shell's
business and nobody else's.

⇒ **The tripwire worked.** `the_engine_still_has_no_rotation_field` read the
pinned engine's own source and went red the moment `Annotation` grew
`appearance_matrix` — hours after it was written. It is kept, inverted, as
`tests::the_engine_owns_the_placement_and_this_module_only_projects`.

## What the engine now answers, and what is still ours

| question | who answers |
|---|---|
| where do the artwork's four corners land, in page space? | `pdfcer_render::annot::appearance_placement` — §12.5.5's full algorithm, **the one the paint path runs** |
| what angle is that, in degrees? | `Annotation::appearance_rotation_degrees` — `None` for a shear, a mirror or a non-uniform scale |
| what are the raw six numbers? | `Annotation::appearance_matrix` |
| where is that **on this canvas**, at this zoom, on a `/Rotate 90` sheet? | **here**, via `crate::canvas::mapping::oriented_canvas_quad` |

★ The engine also publishes the free function
`pdfcer_core::annot::rotation_degrees([f64; 6])`, which decomposes a matrix
this shell does not have in its hand. **It is deliberately not called
here**: the method reads `appearance_matrix` *and* applies Table 95's
default for an appearance with no `/Matrix` key, so it answers `Some(0.0)`
where the free function would need this module to decide what an absent
matrix means — and deciding that here is precisely the private opinion about
somebody else's format that this module was rewritten to stop having.

★ **The corner order is the engine's and it is deliberate**: `[LL, LR, UR,
UL] of the /BBox` — *appearance* space, before the transform. Past 90° the
first element is no longer the leftmost point on the page. That is what lets
a caller draw an outline that follows the object and read a bearing off one
edge, and it is why `handles::GripFrame::Turned` documents its corners as
the **artwork's** frame rather than the page's.

★★ The engine pins, with a test of its own, that **the bearing of the first
placed edge equals `appearance_rotation_degrees()`** on the same annotation.
This shell reads the angle from one and draws the outline from the other, so
a divergence would put a grip where the artwork is not; that agreement is
held by an assertion on their side rather than by intent on ours.

## ⚠⚠ PDFCER HAS **TWO** ROTATION CONVENTIONS, AND THIS IS THE BOUNDARY


| reader | range | why |
|---|---|---|
| `annot::rotation_degrees`, `Annotation::appearance_rotation_degrees` | **`(−180, 180]`** — SIGNED | it decomposes a matrix through `atan2` |
| `forms::WidgetRotation::was` / `::now` | **`[0, 360)`** — unsigned | `/MK /R` is a *stored declaration*, a multiple of 90, which pdfcer normalises |

★★★ **Each was documented correctly at its own definition and neither
mentioned the other**, which is the whole defect surface — and the engine
makes the point that this shell fell into it *from the other direction*,
having already learned the normalising convention from the widget path and
carried it forward. Both now cross-reference by name.

⇒ **This module is where the convention changes**, and the `rem_euclid` in
[`oriented`] is the one expression that changes it. The engine considered
normalising at source and declined, for a reason worth knowing before
anybody asks again: a 1° clockwise nudge would read **`359`**, which is the
wrong number to put in a properties field; and `set_annotation_rotation`
computes `wanted − current`, where signed is the arithmetic-correct form, so
a normalised reader feeding an unnormalised setter would be a second place
for the two to disagree. They also declined to add a second accessor, on
their `R243`: two functions answering *"what angle is this"* means the one
called less is the one that drifts, silently, because both are individually
correct.


## What is deliberately NOT done here

**Nothing is drawn.** This module answers a geometric question and returns
numbers; the painter is `crate::canvas::overlay`. That split is what lets
one answer feed the outline, the grips, the properties panel's angle
read-out and a driven check that wants to assert on an orientation without a
screenshot.

**No fallback quad is invented.** An annotation with no usable appearance
returns `None` and the caller keeps drawing `/Rect`, which for such an
annotation *is* where the mark is.
