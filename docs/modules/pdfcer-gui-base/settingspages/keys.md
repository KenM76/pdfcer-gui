# `pdfcer-gui-base/settingspages/keys` and `keyscatalog`

**Settings ▸ Keyboard shortcuts: every registered command, its keys, and
Change / Off / Reset.** Operator request O274.

# What the page reads

A `keyscatalog::Catalog`, built by the app from the shell and the command
registry when the Settings window opens and carried on `Draft::shortcuts`. The
page needs no shell and no registry, which keeps it in the base crate:

- `groups` — commands under the first ribbon tab that shows them, in tab
  order, then the rest under "Other commands" sorted by label. A command
  absent from the registry is absent from the page (R8).
- `typing` — the commands a text edit routes itself
  (`canvas::textedit::draftkeys::TYPING`), shown again under "While typing
  text" with the fixed editing keys explained.
- `defaults` — the program's keymap after the paste-order preference, before
  any `shortcut.` line. Reset compares against this.
- `kept` — keys bound outside the keymap (page navigation, zoom, and the
  canvas's Escape / Delete / arrows / Tab). No command may take one.

# Rows show effective keys

Each row shows `prefs::shortcuts::effective(defaults, lines)` for its command,
never the line alone: a command whose key another line took shows that it has
lost it. A display that showed only what was written would claim keys the
command no longer answers.

# Capturing a key

Change puts the page into capture. `capture()` runs before anything else draws
and **consumes** the frame's key, text and clipboard events, so the pressed
chord neither types into the filter box nor fires a command. Ctrl+C / X / V
arrive as clipboard events, not keys, and are read as those chords.

- Esc cancels.
- A kept key is refused with its reason; capture stays on.
- A key another command holds raises a clash notice. Only **Reassign** moves
  it: the holder keeps its other chords and the command gets this one.
- Otherwise the command gets exactly this chord.

Changes land in `Draft::working_prefs.shortcuts`; Save writes them and
rebuilds the live keymap, Cancel drops them.

# Trace

`shortcut-set command=… chords=…`, `shortcut-reset command=…`,
`shortcut-clash chord=… holder=… wanted=…`, `shortcut-refused chord=… why=…`.
Rows publish `settings.keys.change.<id>`, `settings.keys.off.<id>`,
`settings.keys.reset.<id>` (typing rows insert `typing.`), plus
`settings.keys.filter`, `settings.keys.reset_all`, `settings.keys.reassign`,
`settings.keys.cancel`.
