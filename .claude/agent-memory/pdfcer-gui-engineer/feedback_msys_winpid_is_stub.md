---
name: feedback-msys-winpid-is-stub
description: Killing the /proc/$PID/winpid of a Git Bash background exe left the real program running and hung `wait`; look up the PID by exe path instead
metadata:
  type: feedback
---

After a smoke launch, find the process by its full path through Win32_Process and kill that PID. Do not trust `/proc/$!/winpid`.

**Why:** the smoke run killed winpid 28044, reported SUCCESS, and the real pdfcer-gui, 32212, kept running from the smoke folder. `wait` then hung until the tool timed out. The likely mechanism is that the winpid is Git Bash's stub for a native child; not confirmed.

**How to apply:** match `ExecutablePath` under the smoke folder so his own pdfcer-gui is never touched, then `taskkill //PID`, and skip `wait`. See [[feedback_never_kill_pdfcer_gui]].
