# `enlarge_scans_doubles_every_pages_picture_as_one_undo_step`

**Defect it guards.** O296: File ▸ Enlarge scans is missing, misses a page
(the one carrying text included), enlarges by another factor than the one
chosen, or leaves the run as one undo entry per page.

**Fixture.** A copy of `fixtures/skewed-scans.pdf`: four 1275×1650 grey
scans; page 4 also draws typed text. Opened off the desktop, through the
scripted pointer.

**Steps.**

1. File tab; expand the Recognise group when it is collapsed. The item
   `ribbon.item.file.enlarge_scans` must be drawn — its absence is a FAIL,
   not a harness SKIP.
2. Enlarge scans, then Enlarge with the defaults (all pages, 2×, bicubic).
3. Wait for `deskew-finished`. The four `deskew-page` lines must read
   `enlarged`, each `to=` twice `from=` on both sides, within one sample.
4. The finish must read `corrected=4 folded=one`.

**Falsified** against the v0.82.0 build (no item) and by folding the run as
`CommandKind::DeskewImage` in `app::actions::deskew::finish`
(`folded=unfolded`).

**What it does not prove.** The rendered pixels: the engine's own tests own
the resampling. Lanczos, factors other than 2×, the selected-pictures scope
and Stop are not driven; a unit test in `dialogs::deskew` walks every
offered factor to the engine.
