---
name: screen-hours-weekday-evenings-and-weekends
description: Ken's standing schedule for on-screen (real OS input) ui-verify runs — weekdays 17:00–08:00 and all of Saturday and Sunday; outside it, off-screen only
metadata:
  type: user
---

Ken, 2026-11-25: on-screen tests are allowed **weekdays from 17:00 to 08:00, and all
day Saturday and Sunday** (local time). Inside those hours, run the real-input
checks too: the sweep without `--no-input`, screenshots, and taking the desktop.
Outside them, drive off-screen only (`--no-input`, the scripted pointer, an
off-desktop viewport). See [[drive-off-screen-while-ken-uses-the-pc]].

**Why:** he works on this PC during weekday office hours, and real-input runs steal
his mouse and focus. The sweeps' "needs real input" skips, 37 in a release, only
clear when the screen is free.

**How to apply:** check the clock and the weekday before any driven run. "I'm at
the PC" still overrides the schedule; stop at once
([[he-is-not-at-the-keyboard-unless-he-says-so]]). Still one driven run at a time.
Never print, and never drive the published OneDrive build: two copies of it are
open with his documents.
