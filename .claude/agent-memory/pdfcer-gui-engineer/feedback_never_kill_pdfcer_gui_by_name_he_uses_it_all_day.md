---
name: never-kill-pdfcer-gui-by-name-he-uses-it-all-day
description: Ken runs pdfcer-gui as his everyday PDF reader instead of Acrobat — never taskkill by image name, only by the PID of a process you launched
metadata:
  type: feedback
---

**Never `taskkill /IM pdfcer-gui.exe`.** Ken runs pdfcer-gui as his **daily PDF
reader**, in place of Acrobat, throughout the working day.

**Why:** 2026-09-08. He said *"I am back using the PC"*, I checked for anything
driving the screen, found one stray process and killed it with
`taskkill //F //IM pdfcer-gui.exe`. That matches by **image name, not path**, so
it would have force-killed his own reader — with whatever he had open and
unsaved — had one been running. It happened not to be. His correction: *"don't
keep killing off pdfce-gui as I do use it throughout the day instead of acrobat
reader."*

★ `/F` makes it worse: no chance to prompt about unsaved work.

**How to apply:**
- Kill **by PID**, and only a PID you launched yourself this session.
- The processes that are safe to end are the ones under the harness's own
  isolation directory — `…/.ui-verify-profiles/<check>/pdfcer-gui.exe` — or a
  scratch copy under `target/scratch/`. Check the **path** in `ps -W` output
  before ending anything.
- A stray off-screen smoke launch (`PDFCER_DIAG_VIEWPORT` at negative
  coordinates) is harmless and does not take focus. Prefer leaving it to
  risking his.
- ⚠ This also means *"is anything of mine running?"* is a question about
  **paths**, not names. A bare count of `pdfcer-gui` processes includes his.

Related: [[ui-verify-competes-for-the-machine]],
[[never-drive-the-published-build]],
[[smoke-launch-offscreen-when-the-desktop-is-blocked]].
