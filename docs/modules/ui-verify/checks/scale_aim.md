# `checks::scale_aim` — getting to a zoom rung, and staying on the target


## The seam is a real subject boundary, not a size-driven cut

`scale_sweep` is now one thing: **what the mouse can do once you are
there.** Click-select, drag, marquee, nodes, handles, pan — a battery of
probes, each of which changes when a *gesture* changes.

This file is the other thing: **how you get there and stay on the point.**
The zoom ladder, the closed-loop re-aim, the tier reading, and the
vocabulary those three share. It changes when the *zoom or position model*
changes — a new render tier, a new position anchor, a different zoom
gesture — and it is entirely uninterested in what a press means.

⇒ The two change for different reasons, which is this project's stated test
for a module boundary (`canvas::interact`'s header makes the identical
argument about composition versus interaction).

## Why the aiming half is worth reading alone

Because it is the half that can make every measurement in the other file a
statement about the harness. The 2026-09-05 sweep filed *"clicking directly
on the content the zoom is anchored to selected nothing"* at five rungs;
what had happened is written in [`re_aim`]'s own header — the correction
moves the **pointer**, and Ctrl+wheel holds the point under the pointer
fixed, so an error the correction cannot close is magnified by every
further notch rather than reduced. It was a limit of this loop and not
anything the application did. A reader who wants to know whether a
scale-sweep finding is real starts here, and [`aim_residual`] is the number
that answers it.

## The dependency runs one way

`scale_sweep` uses this module; nothing here knows that `Rung`, the report
or any probe exists. The three trace names this layer reads live **here**
rather than upstairs, and are `pub` so the battery can share the ones it
also needs — one definition, imported upward, so a rename in the
application cannot leave one file corrected and the other quietly reading a
name that is never printed.

## Item notes

### `const RE_AIM_STEPS`

Each is one pointer move and one 70 ms settle, and the ordinary case
exits after the first. Twelve is enough to close a full-viewport error at a
third of the viewport per step with room to spare.

### `const RE_AIM_TOLERANCE_PX`

Screen pixels rather than canvas points, deliberately: what the next
click needs is to land on the same ink, and "the same ink" is a screen
distance. In canvas points the same tolerance would be meaninglessly tight
at 100 % and meaninglessly loose at 200,000 %.

### `const POSITION_EVENT`

`region=none` is the whole-page tier and anything else is the region tier,
so this line — not an arithmetic guess — is what says which tier a rung
actually reached. `tier=` names the POSITION model, which is the third
boundary.

### `fn tier_of`

`region=none` is the whole-page raster; anything else is the region tier.
`tier=` is the *position* model, the third boundary. Read rather than
computed, because a boundary this check derived itself would be a second
copy of arithmetic that lives in `render::strategy` and `viewer::ceiling`.

### `fn re_aim`

# Why the aim is a loop and not a calculation

The subject of this sweep is a **0.85 pt** pair of cells on a US Letter
sheet. A single conversion at the opening zoom places the pointer to within
one screen pixel, which is about one page point there — larger than the
thing being aimed at. Every rung after that would then be measuring blank
paper beside the cells rather than the cells.

So the aim is corrected before every wheel batch from `canvas-pointer`,
which publishes the canvas-space point the application believes the pointer
is on. Because zoom-to-cursor keeps that point fixed, each correction is
applied at a higher magnification than the last and the error halves with
every doubling: one screen pixel of residual error is one page point at
100 %, and 5 × 10⁻⁵ of one at twenty thousand percent.

It is also, incidentally, a **second** reading of the conversion under
test — if `screen_to_page` were lying, this loop would diverge rather than
converge, and the caller would see the aim wander. That is why the corrected
aim is reported at every rung.

The correction is capped at a third of the viewport per step: a larger jump
means the target has left the window entirely, and chasing it with one
enormous pointer move would land somewhere arbitrary. Capped, the loop still
converges over the following steps.

### `fn zoom_to`

Returns the zoom actually reached, and leaves `aim` corrected onto the
target — see [`re_aim`]. Rolls in small batches and re-reads, because one
notch's factor is egui's and not this harness's to know.

### `fn aim_residual`

`None` when the application has published no `canvas-pointer` line at all,
which is a different fact and has its own line in [`probe_pointer`].

Read from the application's own report of where the pointer is, never
computed from the harness's mapping. A residual computed through the same
conversion the sweep is testing would be zero by construction — the shape
of measurement this project calls a proxy.
The pointer is put back on `aim` first. [`probe_pointer`] leaves it
[`PROBE_PX`] away, and reading the residual from that position would report
the probe's own displacement as an aiming error — a harness measuring its
own last move.
