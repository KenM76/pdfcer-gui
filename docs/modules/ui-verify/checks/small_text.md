# `ui-verify/checks/small_text`

One check, `small_text_is_the_caption_size` (O303). The session opens with
`PDFCER_DIAG_INVOKE=file.settings`, no document, the sandbox's default
preferences (the Quiet preset).

Two trace lines are compared:

- `settings-small-text pts=` — written by the Settings window where it lays
  out its store-location line, from the galley it then draws. That line is
  `RichText::small()` like every Settings description, caption and
  disclosure, so its size is `TextStyle::Small`'s.
- `theme-caption pts=` — written by the frame after `Theme::apply`, the
  active preset's `ribbon_caption_pts` (11 Quiet, 12 Roomy).

`egui_shell::theme::Theme::write_style` sets `TextStyle::Small` from the
same metric, so the two agree unless that link breaks.

# Driven

| Preset | Passes when |
|---|---|
| Quiet (default) | `settings-small-text pts` equals `theme-caption pts`, and that size is above 9 |

No `settings-small-text` line within 140 frames is an error: the window did
not draw. The Roomy preset is not driven; its value comes through the same
metric.

# Falsified

| Break | Result |
|---|---|
| `write_style` sets `TextStyle::Small` to 9 pt | FAIL: drawn at 9 pt, not the caption size 11 pt (egui's stock small size) |
