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
