# `ui-verify/checks/legibility`

Shared machinery for the two pixel checks: *where to look*, and *is what is
drawn there readable*.

Both [`super::ribbon_captions`] and [`super::settings_headings`] ask the
same question of a different surface: *given this image and these named
regions, is every one of them legible?* The question is worth answering in
one place because the two ways of getting it wrong are subtle and would
otherwise be got wrong twice.

## The two failure modes, and why they are reported separately

**Illegible** — something is drawn and its contrast against its background
is below the threshold. This is D2.


There is now a third, which only became reachable once the application
started declaring its own regions: **off-surface** — the application says a
region is at a rect that is not inside the window it was captured from.
That is not a colour problem and not a missing caption; it is the control
being laid out outside its pane, which `PROJECT_PLAN.md` §4.2 prerequisite
2 names as one of the two recorded cases that justified having a pixel
oracle at all.

## Where a region comes from — two sources, and the order is the point

[`resolve_set`] consults them in this order:

1. **The application's own `ui-rect` declarations, from this run's trace.**
2. **A calibrated [`RegionSet`] of fractions** in [`crate::profile`].

### Why the trace wins

A rect the application measured on the frame it reported it for **cannot
go stale**, because there is no interval between the measurement and the
claim. A fraction written into the harness is a claim about a layout that
held when somebody looked at it, and it becomes wrong the first time a
panel is resized, the ribbon collapses to an icon rail, or a workspace is
switched — all three of which are on `MODES_AND_PANELS.md`'s roadmap.

And it goes wrong in the worst available way: **silently, by measuring the
wrong pixels**. The assertion still runs, still samples thousands of
pixels, and still prints a contrast ratio. Nothing about the output says it
is now describing the wrong widget. That is the hazard `PROJECT_PLAN.md`
§4.2 prerequisite 1 names, and it is why the ordering is not a matter of
taste: if both sources are available they are both *plausible*, and only
one of them is *dated to this frame*.

### Why the fraction source is kept anyway

Not every surface reports. `evidence/crop_settings.png` is a screenshot
taken in 2026-08 of a binary that has no `ui-rect` vocabulary at all, and
the D2 falsification run asserts against it — that run is the harness's own
acceptance evidence and it cannot ask a PNG to declare its regions. So the
fraction source stays, as the fallback, guarded by [`Calibration`].

## The calibration guard

[`resolve_set`] refuses to apply a *fraction* region set to a surface it
was not calibrated for. Fractions measured against a 1860×1035 evidence
crop describe *that crop*; applied to a live 2560×1440 window they would
sample whatever happens to lie at those fractions and produce a real
measurement of the wrong thing. A number like that is worse than no number,
because it looks like evidence.

Note that the guard has nothing to say about the trace source, and does not
need to: a traced rect carries its own calibration, in the sense that the
surface it describes is the window it was measured in, which is the window
that was then captured. That is the same argument as above, stated as an
absence of machinery.

## Item notes

### `struct Measurement`

`contrast` is `None` for a region that could not be sampled at all — an
off-surface region. Deliberately an `Option` rather than a contrast of
1.0 over zero pixels: the two are the same number and completely different
findings, and this crate's whole thesis is that a measurement of nothing
must not be presentable as a measurement.

### `fn clip`

A traced rect is already clamped to the client area by
[`crate::coords::WindowFrame::logical_to_capture_pixels`]; this repeats the
clamp against the *image* because the two can differ by a pixel when a
window is resized between the measurement and the capture, and because
`--image` mode can hand a plan a surface of an entirely different size.

### `enum RegionArea`

Two variants because the two sources are resolved against different things
and at different times: a fraction needs the image's dimensions, which are
only known once the PNG is loaded, whereas a traced rect was converted to
capture pixels at the moment the window was measured. Collapsing them into
one would mean either resolving fractions too early or carrying a
[`crate::coords::WindowFrame`] into the offline path that has no window.

### `struct TraceRegions`

Built by the check, because only the check knows which of the declared
names are the ones it is about. It carries the unmatched names too, and
that is not padding: it is what lets a SKIP reason distinguish

* "the application declared nothing at all" — the trace channel is not
  working, or the diagnostic switch did not reach the process;
* "the application declared five regions and none of them is a ribbon
  caption" — the trace channel is fine and the *ribbon* is what is missing.

Those two send a reader to different files, which is the entire reason this
crate is fussy about reason strings.

### `fn resolve_set`

`image_source` is the file being asserted against in offline mode, or
`None` in live mode. `trace` is what the application declared this run, or
`None` when no trace was consulted at all (offline mode, or a check that
does not launch the binary).

Precedence is documented at length in the module docs: **trace first**,
fractions second. The short version is that a rect measured on the frame it
was reported for cannot go stale, and a fraction is wrong the first time a
panel resizes — silently, by measuring the wrong pixels while still
printing a plausible number.

# Errors

Returns `Err(reason)` when the check should SKIP. The reason is assembled
from *what was actually consulted*, never from a template: a reason that
says the trace declared no regions when no trace was read is not merely
imprecise, it sends the reader to the wrong file — which is worse than
giving no reason at all, because they will believe it.

### `fn assess`

Returns `None` if every region is legible, or `Some(reason)` naming every
region that is not — all of them, not the first, because a reader fixing a
theme wants the whole list.
