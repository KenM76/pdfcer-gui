# `canvas::measure::perimeter` — **click around a shape; one number for the
whole way round**

## The operator's ask, verbatim

> *"give me perimeter measuring tool as well where I click around to make a
> shape and it adds the distance of all the segments together for the
> dimension display. let me right click and add segments to the dimension.
> also I want to be able to edit the endpoints of the lines to adjust the
> shape. this should come with all the scaling options of the other
> dimensioning tools."*

— Ken, 2026-08-20. He measures CAD site plans; a perimeter is a fence run, a
kerb line, a wall length. The last sentence is the one that decided the
design: *"all the scaling options"* means it must be a real
[`DimensionKind`] carried by a [`Group`], not a markup annotation with a
number typed into it — so that scale, unit, number format, drafting
standard, layer and the style cascade all come free rather than being
reimplemented badly.

The engine agreed and shipped its whole half the same day it was filed.

## This tool is a HYBRID of the two that already exist, and that is why it
## was cheap to write

| | picks | how it ends |
|---|---|---|
| [`MeasureKind::Linear`] | **points**, with snapping | a fixed arity — three clicks and it is done |
| [`MeasureKind::Circular`] | **objects** | **open-ended** — double-click, or the `measure.finish` command |
| **this** | **points**, with snapping | **open-ended**, plus a third ending of its own |

So the point resolution — the snap query, the derived-candidate two-click
confirm, the operator's snap master toggle — is [`super::click`]'s existing
machinery, untouched, and this module is only asked *"what does one resolved
point mean?"*. And the ending is [`super::circular`]'s answer, already
settled with the operator on 2026-08-14: a **double-click**, because that is
what every polyline tool in every drawing package uses, plus a ribbon
command for a pick that is awkward to double-click on.

## The third ending: click the first vertex to CLOSE

A perimeter has a shape the other two do not — it can be a *ring*. Ken's
words were *"click around to make a shape"*, which is a closed one; a path
length (a pipe run, a cable route) is the same gesture left open.

Rather than a modifier or a toggle nobody would find, the convention every
drawing package uses: **clicking the first vertex closes the shape**. It is
discoverable by accident, it is what a hand does anyway when tracing a
footprint, and it makes the two shapes one tool instead of two.

The hit test for "the first vertex" is in **canvas space** at the same
tolerance a selecting click uses, so it is the same physical target size at
every zoom. Doing it in page space would make the ring impossible to close
when zoomed out and trivially easy to close by accident when zoomed in.

## What is NOT here

**Vertex editing** — dragging a corner, right-clicking a segment to add one,
right-clicking a vertex to remove one. That is editing a *committed*
dimension, so it belongs beside [`crate::canvas::dimdrag`] with the rest of
the after-the-fact editing, not in the tool that authors it. The engine's
verbs for it (`move_dimension_vertex`, `insert_dimension_vertex`,
`remove_dimension_vertex`, and `vertex_edit_preview` so a menu can be greyed
correctly) all exist. Recorded here so the gap is named rather than implied.

## Item notes

### `fn closes_the_ring`

Compared in **canvas space** at the same tolerance a selecting click uses,
so the target is the same physical size at every zoom. In page space the
ring would be impossible to close zoomed out — the first vertex would be a
sub-pixel target — and trivially easy to close by accident zoomed in.

Requires [`MIN_CLOSED`] vertices before it will answer `true`. Below that
there is no ring to close, and reading a click on the first of two vertices
as a close would consume a pick and then refuse, which from the operator's
chair is a click that did nothing.

### `struct PerimeterPick`

# Why the vertices live here and not in [`crate::canvas::selection`]

The same rule [`super::pick::CircularPick`] states and for the same reason:
a half-traced outline is **not a selection**. No verb on the Format tab
means anything applied to it, Delete least of all, and borrowing the
selection to hold it would arm a destructive control over a set the operator
assembled for a completely different purpose.

### `const MIN_OPEN`

pdfcer policy, and the engine labels it as such: ISO 32000-1 §12.5.6.9 states
no minimum, no maximum and no degenerate-case behaviour at all. Two points
is one segment, which is a length.

### `const MIN_CLOSED`

Also policy. A closed shape with two vertices traces a line there and back:
one stroke on screen, printing twice the distance between two points — a
number that disagrees with the picture, which is the one thing this
subsystem exists to prevent.

### `fn push`

No de-duplication: a repeated point contributes a zero-length segment,
which is invisible rather than wrong and which the operator removes by
dragging the vertex after the fact. Refusing it would mean this tool
silently discarding a click, which reads as the tool being broken.

### `fn close`

Refuses below [`MIN_CLOSED`], and refuses a second close — an already
closed pick has been committed and emptied, so reaching here twice would
mean the state machine has slipped.

### `fn author`

The single place a `DimensionKind` is built for this tool, so the
preview and the commit cannot describe different shapes — the standing
rule in [`super`], and the reason [`super::circular::commit`] exists as
one function reached by two endings.

`offset` and `text_along` are zero: the label starts at the vertex
centroid, and moving it from there is a [`crate::canvas::dimdrag`] drag
afterwards rather than a fourth thing to get right during authoring.

### `fn preview`

Used by the live preview and by nothing else. The provisional vertex is
appended rather than replacing anything, so the rubber band runs from
the last committed pick to the pointer — which is the picture every
polyline tool draws and the one that says *"this click would add this
segment"*.

`None` before the first pick: there is no shape yet and drawing a
zero-length segment at the pointer would be a mark that means nothing.

### `fn length_points`

Page points, deliberately — this is the raw measurement, and turning it
into the operator's units is [`crate::text::measure`]'s job through the
group's scale and number format. Two places that both applied the scale
would double it, and one that applied it here would put a unit-aware
number in a geometry function.

### `fn commit`

The one commit path, reached by all three endings — closing the ring,
double-clicking, and the `measure.finish` command. That is the same argument
[`super::circular::commit`] makes and it matters more here, because there
are three doors rather than two: three places each building a
`DimensionKind` is three chances for one of them to forget the `closed`
flag.

Pure over the state and the action list — no `egui`, no context, no memory —
which is what makes every ending assertable without a window.

Returns `false` and raises nothing when there is not enough shape to author.

### `struct Click`

Called from [`super::click`]'s match, *after* the point has been through the
snap query and the derived-candidate confirm — unlike
[`super::circular::click`], which runs before all that because it picks
objects rather than points. A perimeter's vertices want snapping as much as
a linear dimension's do: an operator tracing a building footprint is aiming
at the corners of paths that are already on the page.

# The order of the three questions, and why it is this order

1. **A double-click ends it open.** Asked first because the pair's *first*
   click has already been through here as an ordinary pick and has already
   added its vertex — see [`super::click`]'s note on why that is the right
   reading of how `egui` reports a double-click rather than an accident of
   it. So by the time this fires, the shape is complete and the second click
   must not add a duplicate vertex on top of the last one.
2. **A click on the first vertex closes the ring.** Asked before the
   ordinary pick, because that point is also a perfectly good place to put a
   vertex and the operator who clicks it means the ring — this is the whole
   of the convention.
3. **Otherwise it is a vertex.**
