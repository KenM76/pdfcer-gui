# `the_preflight_panel_saves_its_report_and_loads_a_profile`

**Owns:** the Preflight panel's Load profile…, Save profile… and Save
report… (O458).

**Drive.** `prepress-mix.pdf` (`fixtures/prepress-mix.PROVENANCE.py`), which
has no TrimBox, launched with `mode.edit,view.preflight` and the three file
seams pointed into the run's output folder. The check rewrites the load
seam's file between presses:

1. A profile naming an unknown check; press `preflight.load_profile`.
2. A one-rule profile (`needs-trim`, check `no-trim-box`); press it again.
3. Press `preflight.run`.
4. Press `preflight.save_report`, then `preflight.save_profile`.

**Oracle.**

- Step 1: `preflight-profile-loaded ok=0` with a `why=`: the engine's
  `ProfileError`, and nothing loaded.
- Step 2: `ok=1 rules=1`.
- Step 3: `preflight-finished profile=custom findings>0`: the loaded profile
  is the one run, and the fixture's missing trim box is found.
- Step 4: `preflight-report-saved ok=1 bytes=N`, the file is N bytes, begins
  `Preflight report: ` and holds `Rule needs-trim (error)`: the engine's
  `report_text` for the loaded profile. The saved profile names `needs-trim`
  and `no-trim-box`.

**Not covered.** The native dialogs; the note's pixels.
