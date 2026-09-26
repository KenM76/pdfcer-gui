# `dialogs::settings::display` — how pdfcer draws, as distinct from what it draws

The eighth group, and the only one whose settings are **not** about the PDF
standard. Every other group in this window exists because a clause declines
to have an opinion; these two exist because a machine has a speed.

## Two settings, out of seven commissioned

`RIBBON_IA.md` §5.2 specified a View ▸ Render group of five, plus two
behaviour settings on the same tab, and `shell::manifest::DIRECTED` carried
all seven as *"named individually, with their value sets and their defaults,
when this shell was commissioned"* — which is a stronger statement of intent
than a status mark, and is why they were emitted despite carrying no `G`.



`DIRECTED`'s own doc comment anticipated this outcome and named the remedy:
*"if it turns out to be wrong, the fix is deleting eight rows from one list
rather than re-deriving which entries were deliberate."* Six rows went; the
two that survived became these controls and left the ribbon, because a
setting belongs in the settings window and `RIBBON_IA.md` §6's own list of
what does not go on the ribbon now has a real destination to point at.

`crate::app::prefs`' header carries the full table with the evidence for
each verdict.

## Why these two are not in the engine's settings file

They are **preferences**, not answers to a silent standard, and this
window's own opening paragraph promises the latter. They live in
`userdata/preferences.txt` beside `settings.txt` — same roof, same
fail-soft parser, different file — for the reason `crate::app::prefs`
states. The group sits in this window because a *window* is where an
operator looks for a choice, and which file a choice is stored in is not
their concern.

## Item notes

### `fn the_shipped_quality_changes_no_raster`

The "a build that omits nothing behaves as it did before" rule, at the
one place it can be checked cheaply. `viewer::raster_scale` was
`zoom × pixels_per_point` exactly before this setting existed, so
`Normal` must multiply by one or every raster in the application
silently changed size the day the control landed.

### `fn the_qualities_ascend`

The control reads left to right as a scale, so a list whose middle
entry was not between its neighbours would be a scale that does not
scale.

### `fn the_shipped_settle_is_reachable_on_the_slider`

A default outside its control's bounds would be silently rewritten the
first time anybody opened this window, on every machine, without a
click. Third instance of this check in the window; third setting with a
range that must be the store's.
