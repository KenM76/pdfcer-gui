---
name: window-capture-grabs-desktop
description: ui-verify's capture::window_to_png raises the window and reads the composited desktop — in an off-screen drive it photographed Ken's SolidWorks screen
metadata:
  type: feedback
---

Never call `capture::window_to_png` / `capture::window` / `frame_capture` in a
check that runs while Ken may be using the PC. It calls `raise_window` (steals
focus) and then `sys::capture_screen` on the frame's region — a DESKTOP grab.
On 2026-10-03 an off-screen (`-4200,-4200`) drive of `button_icon` returned
a 1200×1350 picture of Ken's SolidWorks session, which passed the
uniformity guard because real content is not uniform. File deleted, code
removed the same minute.

**Why:** the standing rule is "never capture his desktop"; a capture call
added "for a person reading the run" broke it with no failure signal.

**How to apply:** in an off-screen drive the oracle is the trace. A pixel
oracle needs the app's own rendering (an in-process screenshot seam), not a
screen grab. Checks that already call window_to_png are on-screen checks —
ask before running them. See [[drive-off-screen-while-ken-uses-the-pc]].
