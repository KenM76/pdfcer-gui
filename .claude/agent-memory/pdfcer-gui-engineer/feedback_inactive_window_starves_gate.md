---
name: inactive-window-starves-focus-gate
description: Opening scripted dialogs inactive (so drives never steal Ken's keyboard) starved every field that waits for OS window focus; grep the focus gates when changing activation
metadata:
  type: feedback
---

Making scripted dialog windows open inactive (`with_active(false)`, no
`ViewportCommand::Focus`) was right for Ken: a drive must never take his typing.
But two dialogs (Insert text, form field) only requested field focus while
`viewport().focused != Some(false)`. Under the script that never became true,
so typed words went nowhere. `a_caret_goes_on_the_current_layer` failed on the
new build and passed on the released control, and the trace showed
`window_focused=false` against the control's `true`.

**Why:** a change that withdraws a platform fact (OS focus) silently disables
every gate that reads that fact. The gates live in other files, far from the change.

**How to apply:** when changing window activation or focus behaviour, grep
`viewport().focused` and route each gate through
`pointerscripted::window_takes_keys`. The control-build A/B (same check on the
released exe copy) is what separated this from a flake in minutes. See
[[feedback_proxy_condition]], [[feedback_drive_only_affected]].
