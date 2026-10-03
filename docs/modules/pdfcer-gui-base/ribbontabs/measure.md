# `pdfcer-gui/shell/manifest/measure`

The **Measure** tab — *what am I measuring, and in what units?*

`RIBBON_IA.md` §5.6. Two groups: Dimension, Scale.

# This tab is small on purpose, and that is not the same as underfilled

P2, from the salvage source and kept: **the ribbon picks the activity;
the sidebar holds that activity's controls.** The Measure tab arms
"Linear"; the group picker, scale entry, number format and drafting
standard live in the Tool Options pane. That is why this tab has few
controls, and:

> The fix for an underfilled tab is never to move sidebar controls up
> into it.

What the tab *is* short of is dimension **kinds**, and those are a
build, not a layout decision.

# The group model is the thing not to dilute

Named dimension groups, each carrying a shared scale, number format and
drafting standard, are — per `RIBBON_IA.md` §5.6 — genuinely better
than what the comparison product does. `measure.set_scale` sets the
*current group's* scale rather than a document-wide one, and
`measure.manage_groups` is where groups are created and inspected. Both
tooltips name the group explicitly, because an operator who thinks they
are setting a global scale will be surprised twice: once when a second
dimension reads differently, and once when they cannot find where it
was set.

# What is absent, and one entry worth flagging

`Angular`, and the whole **Quantity** (distance, count) and **Takeoff**
(schedule panel, export CSV) groups, are **N**. Angular is the
conspicuous absence for anyone doing takeoff on a drawing.

`measure.area` sits beside `measure.perimeter` in the ce dimension group,
not in Quantity: it authors a closed perimeter ce dimension labelled with
its area, and Quantity holds the readings that author nothing.

**`Two-line` is a C row and is at the top of the project's own queue.**
Core and CLI shipped and were measured; the gesture's entry point has
no caller. That makes it a shell-only task — the cheapest real command
on this list — and it is still absent here, because P3 is about what
the operator can reach, and an engine with no caller is not reachable.


**`Aligned` is marked *partial G*** — the constraint exists inside the
linear tool, but there is no separate tool to arm. A partial that
cannot be armed is, from the ribbon's point of view, an **N**, so it is
in [`super::PLANNED`] with that reasoning recorded rather than emitted
as a button that would arm nothing.

**`Calibrate from a known length` is also marked *partial G*** and is
treated the same way, and this is the least certain of the judgements
in this module: it is plausible that calibration is reachable today
through the scale entry rather than as its own command. If it is, the
fix is to move one line out of PLANNED and into the Scale group — which
is exactly what PLANNED exists to make findable.
