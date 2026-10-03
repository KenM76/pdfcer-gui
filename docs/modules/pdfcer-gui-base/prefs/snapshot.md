# `pdfcer-gui-base/prefs/snapshot`

**The resolution View ▸ Snapshot copies its box at, as a picture.** One
`preferences.txt` line:

```
snapshot_dpi = 300
```

# Contract

- `SnapshotPrefs::dpi` is always within `MIN_DPI..=MAX_DPI` (36 to 2400);
  `set_dpi` clamps, so no caller can store a value the renderer would refuse.
- A number out of range is **clamped on read**, not refused: a hand-edited
  `snapshot_dpi = 5000` opens as 2400 rather than as a `BadValue` note that
  throws the operator's intent away.
- A value that is not a whole number is a `BadValue` note and the default
  (300) stands.
- The line is always written, with its comment, so the key is discoverable in
  the file.

# Where it is set

Settings ▸ Images, under the two resampling settings
(`settingspages::snapshot::dpi`). The box accepts `150` or `150 dpi`; a value
outside the range is clamped and the page echoes the stored value beside what
was typed. The page on show traces `snapshot-dpi-setting dpi=N` when the value
it draws changes; the invisible search pass traces nothing.

# Why 36 to 2400

36 is half a screen's resolution, below which a copied snapshot is not legible.
2400 is twice a fine photo print's. A region large enough that 2400 dpi
exceeds the renderer's pixmap edge is copied at a lower resolution and the
status line says so; that is the snapshot's job, not this setting's.
