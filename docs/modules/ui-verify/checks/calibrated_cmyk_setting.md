# `calibrated_cmyk_setting`

**Owns:** Settings ▸ Colour ▸ *Keep the CMYK numbers as written*, which is
the engine's `Settings::preserve_calibrated_cmyk`. The check confirms the
switch is offered and that Save writes it to `settings.txt`.

**Drive.** One launch with `checker-image.pdf`. Settings opens through
`file.settings`, and the check presses the Colour heading. It rolls the page
until `settings.colour.calibrated_cmyk` is declared, since that is the page's
last section. It presses the switch, then presses Save and waits for a
`settings-saved` line.

**Oracle.** After Save, the sandbox's `settings.txt` reads
`preserve_calibrated_cmyk = true`. Before the drive the key must be absent or
`false`, because pressing the switch toggles it. A file that already said
`true` would be turned off, so the check refuses to run on that input rather
than pass on it.

**Not covered here.** Whether the setting reaches the renderer is covered by
the unit test `the_calibrated_cmyk_switch_reaches_the_renderer`. What it
changes on a page is the engine's to measure. It applies only where a page is
composited in ink, and no fixture in this repository has an `ICCBased` CMYK
colour on such a page.
