# `ribbon_and_dock_tabs_follow_their_own_face_setting`

**Defect it guards.** O299: the ribbon's tabs and the dock's panel tabs each
show picture, name, or both, as two separate preferences say. A setting that
does not reach its bar, or that reaches the other bar too, fails.

**Fixture.** `fixtures/a1-titleblock.pdf`, after `view.reset_layout`, launched
twice off-screen.

**Steps.**

1. Launch with no preferences: record the ribbon tabs' mean width and count
   the `tabicon.ribbon.*` and `tabicon.dock.*` glyph regions.
2. Launch with `ribbon_tabs_show = glyph` and `dock_tabs_show = text`.
3. The ribbon must still draw glyphs, with tabs averaging at most 0.8× the
   named width; the dock must draw its tabs with no glyph region.

**Falsified** by not calling `with_tab_label` on the ribbon: the mixed
launch's ribbon tabs keep their named width.

**What it does not prove.**

- The Settings rows that write these preferences; they are not pressed.
- The hover text of a picture-only tab, which the trace does not carry; the
  egui-shell tests cover it.
