---
name: smoke-launch-before-every-release-it-is-ninety-seconds
description: Launch the built binary off screen and read its trace before packaging. It costs 90s, needs no focus or pointer so it works while the desktop is blocked, and it found a defect that 2,677 tests, 29 green gates and a matching ribbon all missed.
metadata:
  type: feedback
---

**Before packaging any release, launch the binary off screen and read its
trace.** Not `ui-verify` — a bare launch. Ninety seconds.

```sh
cp target/release/pdfcer-gui.exe /d/temp/pdfcer-smoke/
cd /d/temp/pdfcer-smoke
PDFCER_DIAG=1 PDFCER_DIAG_VIEWPORT="-4200,-4200,1400,900" \
  ./pdfcer-gui.exe "D:/Dev/pdfcer-gui/fixtures/<a fixture that exercises the work>" \
  > trace.txt 2>&1 &
```

`PDFCER_DIAG_VIEWPORT` sets `with_active(false)` and positions the window off
screen, so it takes **neither focus nor pointer** — safe while Ken is working.
The fixture goes in **`argv[1]`**; `PDFCER_DIAG_OPEN_PATH` does not open it.
The trace goes to **stdout**, not to the filename in `PDFCER_DIAG`.

**Why:** on 2026-09-05, eight tracks had landed, `cargo test --workspace` was
2,677 passing / 0 failing, all 29 gates were green and
`compare-mockup-ribbon.py` exited 0. The trace read:

```text
mode-changed to=read panels=4
ui-rect name=comments.note_edit rect=[[1086.0 347.0] - [1146.9 365.0]]
ui-rect name=comments.delete    rect=[[1133.7 368.0] - [1239.0 386.0]]
```

**Three Delete buttons and a note editor in Read mode** — twelve live controls
that write to the document, in the mode whose stated posture is *the document is
not yours to alter*. Forty-six tests over that panel could not have caught it:
**none of them enters a mode**, and the capability accessor falls back to `FULL`
for an unset `Context`, so every one ran as though it were in Edit.

**How to apply:**

- Do it **before** `package-portable.py`, not after. A shipped defect costs a
  retraction; a caught one costs a commit.
- Pick the fixture that exercises **what just landed**, not a generic one.
- Read the trace for three classes: a **panic or missing line** (it did not
  start), a `ui-rect` for a control that **should not exist in this state**, and
  a count line (`comments-panel listed=…`, `dock panels=…`) that disagrees with
  what the mode is supposed to show.
- What it proves: startup, shell build, mode entry, dock mount, document open,
  first render, and **which controls were laid out**. What it does not prove:
  anything needing a click.

⚠ **The watchdog Monitor kills it** — it cannot distinguish an off-screen
unfocused launch from a driven run seizing the screen. That is correct
behaviour and should not be loosened; **the trace survives the kill**, and the
process lives long enough for everything above.

Related: [[smoke-launch-before-every-release-it-is-ninety-seconds]] — the
same technique, recorded as *possible*. This is the upgrade: it is not a
fallback for when driving is blocked, it is **routine, and it pays**.
Also [[feedback_unit_tests_that_call_the_verb_cannot_see_the_chain_in_front_of_it]]
— the defect is that lesson one rung up: a test of the *predicate* would have
passed on the build that never called it.

## When the desktop is blocked — drive it off-screen

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
  second is the one that keeps R1 intact.
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
