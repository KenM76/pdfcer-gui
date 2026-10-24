# `ui-verify/checks/tab_tips`

One check, `a_glyph_only_tab_names_itself_on_hover` (O300). The sandbox's
preferences set `ribbon_tabs_show = glyph` and `dock_tabs_show = glyph`; the
session opens `fixtures/layer-hiding.pdf` in Edit mode.

The oracle is the shell's `tab-tip` line (`egui_shell::tabshape::tip`), read
with `shell_trace`. It is written inside the tooltip's own closure, which
`egui` runs only while the tip is on screen, so it records a tip that was
drawn, not one that was offered. Fields: `surface` (`ribbon` or `dock`),
`tab` (the tab or panel id), `text` (the tip, quoted).

# Driven

| Tab | Gesture | Passes when |
|---|---|---|
| ribbon `view` | hover `ribbon.tab.view`, 60 frames | a later `tab-tip surface=ribbon tab=view` whose `text` starts with `View` |
| dock `view.panel_objects` | hover `dock.tab.view.panel_objects`, 60 frames | a later `tab-tip surface=dock` whose `text` starts with `Objects` |

No `tab-tip` line after the hover fails it too: the tip never showed.

# Falsified

| Break | Result |
|---|---|
| the ribbon's glyph-only arm shows the question alone | FAIL: ribbon tip "What is on my screen, …" |
| the dock's glyph-only arm shows the tooltip alone | FAIL: dock tip "Show or hide the right-hand panel …" |
