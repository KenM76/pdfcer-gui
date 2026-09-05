---
name: he-is-not-at-the-keyboard-unless-he-says-so
description: Ken's standing rule — assume the PC is free. He texts from his phone and will TELL you when he is at the machine. Default to driving, not to deferring.
metadata:
  type: user
---

**Ken, 2026-09-05, verbatim:**

> ***"I'm not at the keyboard unless I tell you I am there. I am texting you
> from my phone. The PC is yours to use."***

⇒ **The default is that the machine is FREE.** Drive the GUI, run `ui-verify`,
take the screen. He will say when he is there — and only then does the
no-driving posture apply.

## ★★ 2026-09-05, made concrete and open-ended

> ***"10 minutes and PC is free for you to use until Tuesday."***

**The machine is his gift until 2026-09-08.** Drive without asking, run the full
175-check sweep as often as it is worth running, take screenshots, launch
Acrobat and Word as reference apps, and treat `ui-verify` as the ordinary way
to answer a question rather than a thing to save up for.

⇒ ★ The corollary that is easy to miss: **there is no longer any excuse for a
"built and undriven" row.** Twenty of them accumulated in a single day under the
opposite assumption. If a check exists and has not run, that is now a choice.

⚠ **Only ONE driven run at a time.** The harness moves the real cursor and types
into the real window; two concurrent runs corrupt each other. When several
tracks are in flight, exactly one owns the pointer and the rest are told so
explicitly and work headlessly. Check with
`tasklist | grep -icE "^(pdfcer-gui|ui-verify)\.exe"` before driving.

## ★★★ What this cost before it was said

On 2026-09-04 he said *"I'm back using the PC"* and, later, *"my screen is still
being driven."* The correct response to that was to stop. **The incorrect
response — and the one taken — was to treat it as permanent.** For the next
twenty-four hours every track was told *"do not launch the GUI and do not run
ui-verify; the operator may be at his keyboard"*, and **twenty driven check
modules were written and left unrun.** Nine tracks shipped a release whose
headline interaction had never been executed once.

★★ R1 is this project's founding rule — *verify by driving the binary, not by a
passing test* — and it was suspended for a day on an inference. **A constraint
inferred about the environment is not a fact.** Either he stated it, or it is a
reading and gets labelled as one. That is the same mis-inference the global
rules already record for subagent dispatch, repeated in a new costume.

## How to apply

- **"I'm back at the PC" / "my screen is being driven" → stop NOW**, kill the
  run, and say so. That is a live report, and he should never have to repeat it.
- **Silence is not that report.** When he has not said he is present, drive.
- When a session begins and it is unclear, **the last thing he said wins** — and
  if the last thing was that he was present, one line asking is cheaper than a
  day of unrun checks. This is the narrow case where asking beats guessing,
  because the wrong guess is expensive in **both** directions.
- ⚠ **The watchdog Monitor** (`kill any GUI window an agent launches while Ken is
  at the keyboard`) must be **stopped** when he says the machine is free — it
  cannot distinguish an off-screen unfocused smoke launch from a driven run, and
  it will kill legitimate work silently. `TaskList` to find it, `TaskStop` to
  end it. Re-arm it the moment he says he is back.

Related: [[feedback_smoke_launch_before_every_release_it_is_ninety_seconds]] —
the off-screen launch stays useful even when the machine is free, because it is
cheap and it does not disturb anything. But it is no longer the *ceiling* of
what may be verified.
