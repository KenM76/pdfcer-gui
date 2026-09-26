# `panels::properties::geometry::annot` — the **annotation** arm of the
geometry section: X, Y, W, H and, since 2026-09-07, **Angle**

## Why this is a separate file

R2, and the seam was already drawn in prose before it was drawn in the file
system. `geometry.rs` served two subjects — a page-content object and a
markup annotation — behind one heading and one draft type, and its own
header describes them as *"two engine verbs that take different shapes"*.
The shared parts (the draft, the seed, the arithmetic, [`super::plan`] and
[`super::annot_plan`]) stay in the parent, where both arms can reach them
and where their tests already live. What moved is only the half that talks
to `EditSession`'s **annotation** verbs.

⇒ The split is deliberately NOT "UI here, arithmetic there". The parent
keeps `annot_plan`, because the whole reason that function is pure is so it
can be tested without a document, and moving it beside its caller would
have put it back in a file that needs `OpenDoc` to compile a test.

## What this arm does that the content arm does not

| | content | annotation |
|---|---|---|
| move | `move_nodes` on the object's anchors | `move_annotation(id, dx, dy)` |
| resize | `resizing::action` factors | `resize_annotation(id, anchor, sx, sy, opts)` |
| **turn** | — | `rotate_annotation(id, pivot, degrees)` |
| lock | no such thing in a content stream | §12.5.3 bit 8, and every field is greyed |

The **turn** row is the one this file is newest for, and the operator asked
for it in as many words: *"the angle should be editable from the
properties."* Read [`super::GeometryDraft::angle_delta`] before changing
anything about it — the field is absolute, the verb is a delta, and the
conversion has a normalisation in it that is not decoration.

## Item notes

### `fn bounds_of`

`None` when the annotation is not among the page's — reachable after an undo
or an external reload has removed it while the selection still names it —
and when it carries no `/Rect`, which `EditSession` refuses by name
(`EditError::AnnotationRectMissing`) rather than inventing one.

# Read from the DOCUMENT, not from the selection's `outline`

[`AnnotSelection::outline`](crate::canvas::selection::AnnotSelection::outline)
is right there and is the wrong number. It is in **canvas space** — Y down
from the page's top-left, with `/Rotate` applied and the crop box's origin
subtracted — and getting back to PDF user space from it means running
`viewer::canvas_to_pdf_space` twice and through `f32`.

`canvas::mapping`'s header calls a second conversion *the classic silent
defect*, and here it would be worse than usually: the fields would show the
number that came back from a round trip through two transforms, the operator
would type `40.00`, and on a rotated page the value written into `/Rect`
would be neither what they typed nor what they saw. Reading the dictionary
is one hop and no convention.

# Normalised, because §7.9.5 does not require a `/Rect` to be

A rectangle may legitimately be written with its *upper-right* corner first,
and producers do it. `min`/`max` on both axes is what makes "Left" mean the
left edge rather than "whichever X the file happened to write first" — and
without it a width would come out negative, which
`resize_annotation` would divide by and turn into a mirror.

[`crate::canvas::annotclip::rect_centre_of`] gets the same fact right by a
different route (it averages the pair, which needs no normalisation) and
says so; the two agree because both are reading §7.9.5 rather than a habit.

# Cost

One `/Annots` walk per frame, bounded by
`pdfcer_core::annot::MAX_ANNOTS_PER_PAGE`. The same price the clipboard's
deleted `carried_options` paid for the same reason —
there is no public verb that models one annotation dictionary — and the same
order as the content arm's `doc.page_objects()`, which is also per frame.

### `fn section`

# The three refusals, and why each takes the surface it takes

| condition | surface | why |
|---|---|---|
| a **ce dimension** | draws nothing, returns `false` | it is not this section's subject at all — [`super::dimension`] owns it, and both engine verbs refuse it **by name** |
| the annotation is **gone** or has no `/Rect` | draws nothing, returns `false` | there is no number to show; four spinners over `0.0` would be an invitation to place a mark at the sheet's corner |
| `/F` bit 8 — **locked** | draws the fields and Apply, **greyed**, with [`crate::text::panels::annotgeometry::locked`] on hover | R9's reserved case exactly: the capability is present and this annotation is out of bounds, so selecting a different one restores it |

**The ce-dimension guard is an [`AnnotKind`] match, never a `/Subtype`
string comparison**, and that is rule 15 made mechanical. A ce dimension IS
a `/Line` — `/IT /LineDimension` — so `subtype == "Line"` reads `true` for a
dimension and for a plain arrow alike, and routing a measurement into
`resize_annotation` would scale its rectangle and its baked appearance and
leave the sidecar geometry the displayed number is derived from where it
was. The mark would then say `1250 mm` about a line that is 900 long.
`canvas::selection::annot::AnnotKind`'s header states why it is an enum:
*a bool is a fact a caller may forget to read; a variant is one the compiler
makes them handle.*

The engine would in fact catch it — `move_annotation` returns
`AnnotationMoveWrongVerb` naming `move_dimension` — so this guard is not the
last line of defence. It is the one that keeps the shell from **offering**
the affordance, which is R83: a control that can only produce a refusal is
not drawn.

# The foreign-appearance refusal is NOT guarded here

`resize_annotation` refuses a non-uniform scale over an `/AP` pdfcer did not
draw, unless `allow_appearance_distortion` is set. That condition cannot be
evaluated without rebuilding the appearance and comparing bytes, so there is
nothing honest to grey. It is surfaced **after** the press, by name, because
the action raised here is the same [`AnnotAction::Resize`] the eight grips
raise and `app::actions::annots::resize` already catches that error and
records `decline::record_resize_not_rebuildable`. A typed Width that the
engine declines therefore says exactly what a dragged one says.
`crate::text::panels::annotgeometry`'s header carries the whole argument.
