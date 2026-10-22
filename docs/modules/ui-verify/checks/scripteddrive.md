# `ui-verify/checks/scripteddrive`

Two helpers that checks driving through the scripted pointer (`--no-input`)
share, held once so the structure gate's no-duplicated-helper rule holds.

- `click(session, pointer, name)` — click the centre of a `ui-rect` region in
  the viewport that declared it (a dialog is its own OS window), then settle 15
  frames. A missing region fails with the names declared under the same
  prefix, which is usually enough to see a renamed or undrawn region.
- `userdata(ctx)` — the `userdata` folder beside the binary under test; seed
  `preferences.txt` there through `sandbox::write_prefs` before launch.

Kept out of `checks::driving`, which is near the file-size limit and serves
the OS-input `Driver` rather than the scripted pointer.
