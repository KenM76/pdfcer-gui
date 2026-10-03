# `pdfcer-gui-base/prefs/shortcuts`

**The operator's own keyboard shortcuts, one `preferences.txt` line per
command he changed.**

```
shortcut.edit.find = Ctrl+K
shortcut.edit.redo = Ctrl+Y Ctrl+Shift+Z
shortcut.format.bold = none
```

# Contract

- A line gives the command **exactly** the chords listed — not "these as
  well". A command with no line keeps the program's keys, so a Reset is the
  removal of the line and nothing else.
- `none` (or an empty value) switches the command's keys off. It is stored as
  an empty list, distinct from "no line".
- Chords are stored in the spelling `keychord::canonical` gives, so one chord
  has one spelling in the file whatever the operator typed or the manifest
  used.
- An unreadable chord makes the whole line a `BadValue` note and sets nothing:
  a half-applied line would give the command keys he did not write.

# `ShortcutPrefs::apply`

Laid over a chord → command map in two passes:

1. Every chord of every command named here is removed.
2. Each listed chord is inserted, first removing any key that **parses to the
   same chord** under another spelling (`Ctrl+[` and `Ctrl+OpenBracket` are one
   chord). Without the parse comparison, a taken key would stay with its old
   holder under its old spelling and `BTreeMap` order would decide who wins.

Applying is a pure function of the defaults and the lines, which is why the
live keymap is rebuilt from the loaded snapshot rather than edited in place
(see `shell/manifest/keys`).
