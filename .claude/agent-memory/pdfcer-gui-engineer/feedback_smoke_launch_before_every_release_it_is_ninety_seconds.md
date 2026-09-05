---
name: smoke-launch-before-every-release-it-is-ninety-seconds
description: Launch the built binary off screen and read its trace before packaging. It costs 90s, takes no focus or pointer, and it found a defect that 2,677 tests, 29 green gates and a matching ribbon all missed.
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

Related: [[feedback_smoke_launch_offscreen_when_the_desktop_is_blocked]] — the
same technique, recorded as *possible*. This is the upgrade: it is not a
fallback for when driving is blocked, it is **routine, and it pays**.
Also [[feedback_unit_tests_that_call_the_verb_cannot_see_the_chain_in_front_of_it]]
— the defect is that lesson one rung up: a test of the *predicate* would have
passed on the build that never called it.
