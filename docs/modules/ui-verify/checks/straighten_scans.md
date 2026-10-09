# `straighten_scans_turns_tilted_pages_and_leaves_the_rest`

**Defect it guards.** O289 item 13: File ▸ Straighten scans misses a tilted
page, turns a level page or one carrying text, measures the wrong angle, or
leaves the run as one undo entry per page.

**Fixture.** A copy of `fixtures/skewed-scans.pdf`: four grey scans drawn
tilted 2.0°, -1.5°, 0° and 2.0° counter-clockwise; page 4 also draws typed
text. Opened in Edit mode, off the desktop, through the scripted pointer.

**Steps.**

1. File ▸ Straighten scans, then Straighten with the defaults (all pages,
   skip pages with text).
2. Wait for `deskew-finished`. The four `deskew-page` lines must read
   `straightened` (about 2.0°), `straightened` (about -1.5°), `level` and
   `has-text`, each measured angle within 0.1° of the drawn one.
3. The finish must read `corrected=2 folded=one`.

**Falsified** by ignoring `skip_text` in `app::actions::deskew::page_turn`
(page 4 is straightened) and by skipping the fold in `finish`
(`folded=unfolded`).

**What it does not prove.** The rendered pixels: the engine's own tests own
the resampling. The selected-images scope and Stop are not driven.
