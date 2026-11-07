---
name: feedback-read-layout-before-bug
description: A control past a window's edge in the trace may simply be below a scrolling body's fold; read the layout before telling him it is a bug
metadata:
  type: feedback
---

Before calling a region outside its window a defect, read the code that lays out that window.

**Why:** I told Ken "found a real bug" because Export image's Keep text box measured below the window. Its body is a ScrollArea with Export pinned below it. The region was published with `diag::ui_rect`, which ignores the clip, so the trace showed a position nothing could click; the check simply never scrolled. I had to retract in the next message.

**How to apply:** a region inside a ScrollArea reports through `ui_rect_visible(name, rect, ui.clip_rect())` so absence means "scroll". Only claim a defect after the fork is read: scrolls, or not.
