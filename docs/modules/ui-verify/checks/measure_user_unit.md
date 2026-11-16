# `ui-verify/checks/measure_user_unit`

`a_measurement_on_a_user_unit_page_reads_physically` — a Linear ce dimension
drawn across a known span of a page with `/UserUnit 2` shows the span's
physical length. ISO 32000-1 §14.11.2 makes `/UserUnit` the size of one
user-space unit in 1/72 in, so a length of `n` box units on that page prints
`n × UserUnit / 72` in. The window is off the desktop and driven only through
`ScriptedPointer`, so the check runs under `--no-input`.

## The oracle

Taken from the fixture, never from the program: the span is the bottom edge of
the frame `18 18 270 360 re S` in `fixtures/user-unit.pdf`, from (18, 18) to
(288, 18) — 270 box units. `/UserUnit` is read from the file's bytes. The
expected raw readout of the default never-set group is `270 × 2 = 540` pt
(7.5 in), within 2 pt.

## Steps

1. Copy `fixtures/user-unit.pdf` to the run folder and open it.
2. Click `ribbon.mode.review`, `ribbon.tab.measure`, then
   `ribbon.item.measure.linear`, opening collapsed `ribbon.group.measure.*`
   buttons until it shows.
3. Hover and click each end of the span. A new `dimension-added` line must
   follow the second click.
4. The last `dimension-member-shown` line's `text` must parse as points and
   read 540 within tolerance. A reading of 270 fails with the box-units
   sentence; any other reading fails as neither.

## Falsification

Replacing `EditSession::dimension_display` with `dimension_model().display`
in `trace_members_shown` reads 270 and fails step 4 with the box-units
sentence. The two readings differ by the whole span, far outside the 2 pt
tolerance, so a pick that misses the snap cannot pass the wrong one.
