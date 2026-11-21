# `ui-verify/checks/scripteddrive`

Helpers that checks driving through the scripted pointer (`--no-input`)
share, held once so the structure gate's no-duplicated-helper rule holds.

- `click(session, pointer, name)` — click the centre of a `ui-rect` region in
  the viewport that declared it (a dialog is its own OS window), then settle 15
  frames. A missing region fails with the names declared under the same
  prefix, which is usually enough to see a renamed or undrawn region.
- `userdata(ctx)` — the `userdata` folder beside the binary under test; seed
  `preferences.txt` there through `sandbox::write_prefs` before launch.

Group-three helpers, each the scripted counterpart of an OS-input route:

- `aim(ctx, session, page, at)` — a document point as a window point, after
  requiring the canvas to show `at`'s page.
- `click_command(session, pointer, ui_rect, (name, id), settle)` — a ribbon
  item wherever it is (band or overflow), requiring a new invoke of `id`.
- `click_tab(session, pointer, ui_rect, (region, id))` — a ribbon tab,
  requiring a new tab-activated line for `id`.
- `mark_through_the_menu(session, pointer, ui_rect, at, what)` — right-click
  `at`, choose the canvas menu's Mark for redaction row, and read the mark.
- `resolve_pick(session, pointer, report, label, at, settle)` — click until
  the shell's pick at `at` resolves, as `picking::resolve_pick` does.
- `launch_with(ctx, report, pdf, trace_name, env)` — launch off the desktop
  with both diagnostic channels and extra variables, requiring the start line.
- `type_into(session, pointer, name, text)` — click a field and type into the
  viewport that declared it.
- `click_scrolled(session, pointer, (name, body), notches, report)` — wheel a
  scroll area until a control is wholly inside it, then click it.
- `census_refresh` / `census_baseline` — `comments_census`'s reads through any
  `input::Click`, bringing the Comments panel forward when it has gone quiet.

Kept out of `checks::driving`, which is near the file-size limit and serves
the OS-input `Driver` rather than the scripted pointer.
