---
name: smoke-launch-offscreen-when-the-desktop-is-blocked
description: PDFCER_DIAG_VIEWPORT gives a real laid-out off-screen window, so a launch-and-read-the-trace check costs the operator nothing even when ui-verify is blocked
metadata:
  type: feedback
---

**When the go-ahead for `ui-verify` has not been given, a smoke launch is still
available and should still be done.**

```bash
PDFCER_DIAG=1 PDFCER_DIAG_VIEWPORT=-4000,-4000,1400,900 \
  timeout 12 ./target/release/pdfcer-gui.exe <fixture.pdf> > smoke.txt 2>&1
```

`with_position([-4000,-4000]) + with_active(false)` gives a window that is
**genuinely laid out and genuinely invisible** — panels allocate their real
sizes, regions publish their real rects — and it does not steal focus. It
injects no input, so it does not touch the pointer or the keyboard.

**Why:** [[ui-verify-competes-for-the-machine]] blocks *executing the harness*,
which is input injection plus window raising. A bare offscreen launch is
neither. Skipping it means shipping with no evidence at all that the binary
starts, which is strictly worse than shipping with partial evidence.

**How to apply:**

- Run it after every release build, and grep the trace for the regions the
  change was supposed to produce. On 2026-08-20 it proved the new document tab
  strip was composed on a real frame with 108 × 22 pt of clickable extent — the
  whole *"registered but never drawn / drawn at zero height / clipped out of its
  pane"* family, which this project keeps finding, ruled out in one command.
- **Report exactly what it does and does not establish.** It says a surface is
  drawn and where. It says nothing about any gesture. Write both sentences; the
  second is the one that keeps [[r1-drive-the-binary]] intact.
- It never substitutes for a driven check. Queue those as usual.

## ★★ Two traps, both of which take the screen while he is working

**1. There is no `--version`, and an unrecognised argument LAUNCHES THE GUI.**
On 2026-09-16 I ran `./target/release/pdfcer-gui.exe --version` to read a build
stamp. It is not a flag; the binary treated it as a path, failed to open it, and
opened a normal window **on Ken's screen while he was at the keyboard**. Two
processes had to be killed by verified PID.

⇒ **Read the stamp from the packaged `BUILD-INFO.txt`**, which carries the shell
commit, the engine pin, the digest and the build time. Never probe a GUI binary
with a guessed flag: the failure mode of a wrong flag is not an error message,
it is a window.

**2. `PDFCER_DIAG_INVOKE` is the half that reaches a dialog.** An off-screen
window is laid out but OS input cannot reach it, so anything behind a button is
invisible to a bare smoke launch:

```bash
PDFCER_DIAG=1 PDFCER_DIAG_VIEWPORT="-4200,-4200,1500,1000" \
  PDFCER_DIAG_INVOKE=file.print ./pdfcer-gui.exe <fixture.pdf> > trace.txt 2>&1
```

No pointer, no focus, and the print dialog's own regions and body geometry land
in the trace. That is how the 2026-09-16 layout regression was found **and** its
fix confirmed while he worked. Mechanism:
`D:/dev/rag/egui/an_env_var_command_seam_is_how_you_verify_a_gui_on_a_machine_whose_desktop_is_occupied.md`.
