# `ui-verify/checks/off_page_wall`

`an_object_off_the_page_is_drawn_past_its_halo_wall` — `fixtures/off-page-object.pdf`
off the desktop, driven only through `ScriptedPointer`, so it runs under
`--no-input`.

The halo box is the sheet unioned with the content, 360 pt against a 200 pt
sheet, so it outgrows the pixmap limit at a little over half the zoom the sheet
does. Between those two zooms the whole-page raster still fits and has no
pixels off the sheet; the page must take the region tier there, whose request
`render::halo::reach` widens past the sheet. `canvas::tier::decide` traces that
choice as `canvas-halo tier=walled`.

## Steps

1. Enter Edit (off-page display is off in Read), single page, 100 %. The trace
   must carry `canvas-halo tier=halo offpage=on`, or the run is SKIPPED.
2. Ctrl+wheel one notch at a time over the midpoint of the off-page square's
   bottom edge (−100, 100), re-located after every notch, and stop on the first
   notch after which a `canvas-halo` line names a tier other than `halo` with
   `offpage=on`. The zoom must rise on every notch.
3. Settle. If the last tier decision is `region`, that notch crossed the
   sheet's wall too and stepped over the window: SKIPPED, because a picture in
   the sheet's region tier passes whether or not the walled tier exists.
4. Capture the window's own frame. A 20 × 20 logical-pt patch 40 pt above the
   edge (inside the square) must be at least 90 % ink; the control, 40 pt below
   it, at most 5 %. Logical offsets keep both on screen at any zoom.

## Falsification

- `canvas::tier::decide` ignoring `walled`: the canvas leaves the halo for
  `tier=whole` at 5460 %, the patch inside the square reads 0.000 ink and the
  screenshot is blank pasteboard; FAIL.
- Picturing three notches past the wall instead of at it reached the sheet's
  own region tier and PASSED on the defect, which is why step 3 exists.
- Fixed build: `tier=walled` at 5446 %, ink 1.000, control 0.000; PASS.
