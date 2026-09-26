# `ui-verify/checks/read_mode_exit`

`read_mode_says_how_to_get_back_out` — while read mode is on, **the way out
is written on two surfaces, and it names the key the keymap actually
holds.**


Stated first, in its own section, because a check nobody has run is not
evidence and this project has shipped twenty modules in that state. The
session that wrote it did not launch it: three other tracks were live in the
tree, one of them owning the pointer, and `RESUME.md`'s standing note is that
**a green test count is not a substitute for R1.** Whoever runs it first
should expect to correct it; see *What could still be wrong* at the foot.


> *"I didn't see a way to get back out of read mode. if there is a shortcut
> for this it should have a note what the key combo is in the top bar that
> holds the window controls."*

`view.read_mode` hides the ribbon and the docks. The only control that turns
it off — View ▸ Window ▸ Read mode — is **on the ribbon**. So from the moment
the mode is on, the control that undoes it is hidden by the thing it
toggles, and the only remaining route is a chord nothing on screen names.

`app::window`'s header had answered this with *"the tooltip on the control
states the chord before the operator presses it"*. That is true about
Acrobat and it does not follow: `Ctrl+H` is a **bound chord**, pressable from
memory or by accident having pointed at nothing, and a click is not a hover.
**A tooltip is not a disclosure; it is a disclosure available to somebody who
already knows where to point.**

# ★★★ The vacuous shapes this is written to avoid

Three, and each of them is a check that passes on a broken build:

| vacuous assertion | passes on |
|---|---|
| *the hint exists* | a hint naming a chord nothing is bound to |
| *the hint exists* | a hint shown permanently, including when read mode is **off** — furniture, and a false statement for every minute the mode is not on |
| *the hint says `Ctrl+H`* | ★ nothing — but it FAILS on a legitimate rebind, which makes it a second copy of the binding rather than a test of it |

So what is asserted is an **identity between two derivations that a wrong
build breaks and a rebind does not**:

1. The application publishes `read-mode-exit chord="…"`, resolved by
   `app::window::publish_exit_chord` **from the keymap that dispatches**.
   That field is the keymap's own answer, whatever the manifest says today.
2. Both operator-visible strings — the status line (`line=`) and the window
   title — must **quote that value verbatim**.

A catalog that re-introduced a hard-coded `"Ctrl+H"` passes today and goes
red the first time anybody rebinds the command, which is exactly the moment
it becomes a lie. A rebind with the mechanism intact moves `chord=`, `line=`
and the title together, and this check stays green.

# ★★ And the absence half, taken from a run that REACHED the state

An absence assertion is vacuous when the run never reaches the state it is
asserting absence in. This one reads a **single launch that does both**:
`PDFCER_DIAG_INVOKE=view.read_mode` turns the mode on some frames after
start-up, and `window-title` is traced whenever it changes, so one trace
carries the ordinary title *and* the read-mode title.

The absence is then derived rather than spelled:

```text
first  window-title  "pdfcer — 2026-09-05 06:24 UTC"
last   window-title  "Read mode — Ctrl+H to exit — pdfcer — 2026-09-05 06:24 UTC"
```

`last.ends_with(first)` and `last != first`, so the statement is a **prefix**
that was not there before. Nothing in this file spells the sentence, so a
rewording of the operator copy cannot make it fail — which is the property a
`contains("Read mode")` assertion would not have.

★ The prefix being a prefix is itself load-bearing, not incidental. A taskbar
button truncates from the right; a hint appended after the build stamp would
be the first thing the ellipsis eats, on the window of the one operator who
most needs it. It is also what keeps
[`super::title_build_stamp`]'s right-hand parse aimed at the build stamp.

# ★ Full screen is checked by its absence

`fullscreen=` must be **empty** here. Full screen hides no chrome of pdfcer's
own — the ribbon stays, its control stays, `app::conditions` renders it
pressed — so naming `F11` when the window is not full screen would be the
furniture this feature is written not to add. The one state where it *is* a
trap is read mode **and** full screen together, where the ribbon is gone and
the control with it; that combined state is covered by
`app::status::readmode`'s unit tests and is deliberately not driven here,
because filling the operator's display is a cost `read_mode_chrome` already
pays once and should not pay twice.

# It needs no input at all

No pointer, no keyboard. `PDFCER_DIAG_VIEWPORT` lays out a real window
without taking focus and `PDFCER_DIAG_INVOKE` rings the command through the
same `dispatch_command` a chord reaches. So this can run beside somebody
working — which for a check about a state an operator gets *stuck in* is
worth having, because it can then run often.

★ And no `--pdf`. Read mode is per **window**, not per document
(`app::window` §3), so the statement must appear with nothing open — and
`title_build_stamp`'s note applies: a check whose subject does not need a
document should not acquire a dependency on one, or a moved fixture turns it
into a SKIP and a SKIP is not red.

# What a passing run does NOT prove

That pressing the advertised key works. That is `chords`' subject — it
presses `Ctrl+H` through the OS and asserts `chord-command … id=view.read_mode`
— and it needs `--allow-input`. The two together are the whole claim: this
one says *the surface advertises what the keymap holds*, that one says *what
the keymap holds arrives*. Neither alone is enough and neither is redundant.

# What could still be wrong, for whoever runs it first

* **The invoke may not have landed by the settle.** `scripted_invoke` rings
  one id per frame and the first frames are start-up; if `read-mode on=true`
  is missing the check SKIPs with that said, rather than failing, because a
  command that never ran is not a defect in what it would have done.
* **The title trace is de-duplicated on change.** If a future build sent the
  title unconditionally, `first` and `last` would still be right, but a build
  that stopped sending it at all leaves one line and the check SKIPs.
* **`line=` and the title quote the chord with different spellings** would be
  a real failure and is the one this check is most likely to catch first.
