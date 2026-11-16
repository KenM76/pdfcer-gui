# `ui-verify/checks/extract_ce_dimensions`

`extract_keeps_the_ce_dimension_groups` — a page extracted through Pages ▸
Extract… keeps its ce dimensions in their group, with the group's scale and
number format, and they open as ce dimensions in the new file. One launch, on
a window off the desktop, driven only through `ScriptedPointer`, so it runs
under `--no-input`.

## Steps

1. `dimension-scaled.pdf` is copied to the run folder (one page, one linear
   ce dimension, its group calibrated so 200 pt reads 1000 mm) and opened with
   `PDFCER_DIAG_INVOKE=mode.review,measure.manage_groups`. The save target is
   deleted first, so an earlier run cannot pass for this one.
2. Dimension groups tab: `dimension-groups.row.0`, the unit combo, ft-in, the
   denominator, 16. The `dimension-member-shown` text after 16 is the source's
   reading, `3'-3 6/16"` or `3'-3 3/8"` by the reduce default. No line there
   fails: the source never had a non-default format to carry.
3. `ribbon.tab.pages`, the collapsed organise group when narrow,
   `ribbon.item.pages.extract`; the window must publish
   `extract-pages.extract`. The one page is the default range. The picker is
   answered by `PDFCER_DIAG_SAVE_PATH`.
4. The written file is read by the engine's reader, `pdfcer dimension-list`
   (`PDFCER_EXE`, else `D:/Dev/pdfcer/target/release/pdfcer.exe`; the harness
   does not link `pdfcer-core`). Absent, the check SKIPs.

## Oracle, in order

1. The listing's header says `dimensions=1`: the record came across.
2. A group line with `unit=ft-in` and `members=1`: the group's format came
   across, not the default group's.
3. That group's `scale=` is 5 / 304.8 ft/pt to 1e-6: the calibration came
   across.
4. The `dim` line names that group id and its value carries the fraction the
   source showed.
5. Only then the receipt: the `extract` line carries `ce_dims_carried=1
   ce_dims_left=0 ce_sidecar_newer=0` (`AssembleReport::ce_dimensions_carried`,
   `ce_dimensions_left`, `ce_dimension_sidecar_newer`).

The file is judged before the trace so a build whose extraction drops the
sidecar fails on what the operator loses.

## Falsification

- An engine pinned before the sidecar carry (`pageops::extract_with_labels`
  writing a fresh catalog with no `/PieceInfo`) fails oracle 1: the copy lists
  `dimensions=0`.
- `AssembleOptions::with_dimension_sidecar(false)` on the extract path fails
  oracle 1 the same way.
- The unit tests in the check file feed `judge_listing` a bare listing, an mm
  group, an unscaled group and the wrong fraction; each must fail.

## What it does not cover

The newer-schema sentence (`ce_dimension_sidecar_newer`) needs a fixture
written by a newer pdfcer and is not driven; its wording is unit-tested in
`text::cedimcopy`. Split's carry is the unit test
`each_part_keeps_its_ce_dimensions_in_their_group`.
