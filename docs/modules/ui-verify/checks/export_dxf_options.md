# `export_dxf_writes_the_version_and_scale_chosen`

**Request:** `OPERATOR_REQUESTS.md` O252 — the DXF export window's version
drop-down and drawing scale.

**Defect it catches:** the window offers a version and a scale and the file it
writes ignores them.

## What it drives

Off-screen (`-4200,-4200,1400,900`, scripted pointer, `--no-input` safe), on
the pinned `fixtures/cropped-sheets.pdf`, whose page 0 has a stroked rectangle
and so gives the file non-zero extents.

Two rounds, each: File → Export to DXF, pick a version from the drop-down
(`export-dxf.version.item.N`), click the scale ratio's real-world field
(`export-dxf.scale`), Ctrl+A, type a number, Enter, press Export.

| Round | Entry | Must write `$ACADVER` | Typed real-world number |
|---|---|---|---|
| 0 | 0 (R12) | `AC1009` | 37 |
| 1 | 2 (R2004) | `AC1018` | 53 |

## Oracles

- **Version:** the file's `$ACADVER` is the picked entry's, and the
  `export-dxf-requested` trace says the same.
- **Remembered:** the second opening's `export-dxf-open version=` is the first
  export's version.
- **Scale typed:** the traced scales' ratio is 53/37. Comparing two typed
  values, not one against a default, makes the check independent of the
  remembered ratio and units.
- **Scale applied:** the file's `$EXTMAX` x grows by the same ratio. Same page,
  same units, so only the scale can move it.

## Notes

- The window closes on Export, so each round reopens it.
- Each check runs its own copy of the binary under `.ui-verify-profiles`, so
  the remembered choices never reach the operator's settings.
- Falsified on 2026-10-01: a build forcing R2000 fails round 0 on
  `$ACADVER`; a build forcing scale 1 fails on the extents ratio.
- Not covered: picking a scale from a scale group. The fixture has no
  calibrated ce dimension groups.
