# `ui-verify/checks/caption_theme`

`title_bars_follow_the_theme` — every visible window of the application has
a dark title bar under the Dark preset and a light one under Quiet, with the
Settings window open as the second window.

# The oracles

The scratch binary starts on `theme = quiet` (its `settings.txt` is restored
afterwards). Each visible top-level window's `DWMWA_USE_IMMERSIVE_DARK_MODE`,
read with `DwmGetWindowAttribute`, must be false at start. Clicking
`settings.theme.dark` must trace `caption-theme dark=true` and leave every
window — at least the main window and the Settings window — reading true;
clicking `settings.theme.quiet` must trace `dark=false` and read false.

| proved here | **not** proved here |
|---|---|
| the attribute on every window, in both directions, at runtime | the caption's pixels: the window is off the desktop and the desktop is never captured |

Falsified by removing the per-frame `caption::sync` call: the windows keep
the host's mode, so one of the three stages reads wrong whichever mode the
host is in.
