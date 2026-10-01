# `system_theme_follows_the_host`

**Request:** `OPERATOR_REQUESTS.md` O268. A look that follows the OS.

**Defect it catches:** the Match the system theme drawing the app's own blue,
or light when the host is dark (or the reverse).

## What it drives

Nothing is clicked. The sandbox's `userdata/settings.txt` is written with
`theme = system` (an engine setting, not a `preferences.txt` key), then the
app is launched off-screen (`-4200,-4200,1400,900`, scripted pointer,
`--no-input` safe) on `fixtures/cropped-sheets.pdf` and its own frame shot.

The host's accent (`HKCU\...\Explorer\Accent\AccentColorMenu`, `0xAABBGGRR`)
and mode (`...\Themes\Personalize\AppsUseLightTheme`) are read with `reg.exe`.

## Oracle

- The central panel's mean channel is at least 128 exactly when the host is in
  light mode.
- The selected `ribbon.mode.read` segment's fill is nearer the host's accent
  than the base preset's own accent (Quiet `#175CC4`, Dark `#4C9AFF`).

SKIP when the host's accent is within `2 × MIN_PRESSED_DELTA` of the base
accent, since the fill could not tell them apart.

Falsified by the theme never reaching the app (written to `preferences.txt`):
the segment drew `#175CC4`, FAIL.

## Not covered

An accent adjusted for contrast is only checked to be nearer the host's than
the preset's; the theme unit tests cover that every accent ends readable. The
accent is read once at start, so a change in Windows Settings shows on the
next launch.
