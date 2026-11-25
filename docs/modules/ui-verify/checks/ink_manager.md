# `the_ink_manager_aliases_and_converts_spots_as_one_step`

**Owns:** File ▸ Print ▸ Ink Manager (O458): the list, the alias and the
conversion, one undo step, and Output Preview reading the result.

**Drive.** `ink-manager.pdf` (`fixtures/ink-manager.PROVENANCE.py`): one
Letter page painting three Separation spaces, `PANTONE 185 C`, `PMS185`
(the same red under another name) and `Gold`, plus a DeviceCMYK control.
Launched with `mode.edit,view.output_preview,file.ink_manager`. Switch Output
Preview on; choose `prepress.ink-choice:PMS185:as:PANTONE_185_C` and
`prepress.ink-choice:Gold:process`; press `prepress.apply.inks`.

**Oracle.**

- Before: `ink-manager-listed drawn=` names all three spots, and Output
  Preview's `output-preview on=1 inks=` has a plate for each.
- `prepress-applied verb=inks changed=1 aliased=1 converted>=1 commands=2
  folded=1`: both engine commands ran and were folded into one undo entry.
- After: a later-epoch `ink-manager-listed` has no `PMS185` at all and
  `Gold` in `inks` but not in `drawn` — `spot_to_process` converts every
  paint and leaves the space's own array in place, so the ink is still
  defined and the window says *not drawn*. Output Preview's next
  `output-preview` line keeps `PANTONE_185_C` and loses `PMS185` and `Gold`.

**Not covered.** Undo itself (the edit funnel's checks own it); refusals,
which the engine's own tests own; the plates' pixels.
