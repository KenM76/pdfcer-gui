# `settings_text_size`

**Owns:** the Settings window's own text size (`settings_text_scale` in
`preferences.txt`). The check confirms it opens larger than the program, that
Ctrl+wheel over the window and the slider at the far right of the search row
change it, that the main window does not move, and that the size is kept the
moment it changes.

**Drive.** One launch with `checker-image.pdf` and a sandbox with no
`settings_text_scale`. Settings opens through `file.settings`.

1. It reads `settings-text-scale scale=` and `settings-small-text pts=` and
   records every `ribbon.*` region in the main window.
2. It sends one Ctrl+wheel notch (`wheel X Y 1 mods=ctrl`) over the search box.
3. It presses the slider one point inside its right end.
4. It presses Cancel.

**Oracle.**

- On opening, the scale is the shipped 1.25.
- After the notch, the scale is 1.30 in the trace and in the file, with no
  Save pressed, and the small text is larger.
- After the slider press, the scale is 2.50 in both places and the small text
  is more than 1.8 times its opening size.
- The row reads search box, *Text size* label, slider, with the slider within 40 pt of the window's
  right edge.
- The main window's ribbon regions are identical before and after.
- After Cancel, the file still says 2.50.

A sandbox that already holds the key is refused, because the default could
not be measured.

**Not covered here.** Dragging the slider partway is stock slider behaviour.
Restore defaults resetting the size is not driven.
