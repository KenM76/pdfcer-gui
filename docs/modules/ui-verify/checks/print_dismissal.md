# `ui-verify/checks/print_dismissal`

`the_print_window_forgets_what_cancel_undid` — **operator request O185,
driven.**

# The report


> *"I set the printer up, close the window to go check something, and it's
> all gone."*

and, in the same breath, the other half — that a window which keeps what you
set up needs a way to say *no, not that*:

> *"they revert back to what they were when we opened the print dialogue"*

O166 had already made the print window remember its settings **when Print is
pressed**. O185 is about the other three ways out. There are now four, and
they mean two different things:

| route | meaning |
|---|---|
| **Print** | remember, spool, close *(O166, unchanged)* |
| **Keep and close** | remember, print nothing |
| **Cancel** | put the settings back to what this window opened with |
| the OS close button, and Escape | the same as **Cancel** |

That last row is `ui-conventions/dialogs.md` **G4** held to the letter: the
chrome, Escape and the cancel button are deliberately indistinguishable, so
all three keep one meaning, and the meaning they keep has to be the safe
one. *Keep and close* is a fourth, positively-chosen route that G4 never
contemplated.

# What this check is FOR, said before what it does

**It is the assertion that the two labelled routes out are not the same
button.** Everything else here is scaffolding for that one sentence.

A build in which *Keep and close* and *Cancel* are wired identically — the
obvious regression, and the state of the program the day before this landed
— is a build in which every screenshot is correct, every unit test passes,
and the operator's actual complaint is unfixed. Two buttons sit in the
footer, both close the window, and which one you press changes nothing.
Nothing a photograph can show distinguishes that from a working build.

So the load-bearing assertion is **cross-run**: the value the window opens
on after a Cancel and the value it opens on after a Keep must *differ*. Each
run also makes its own absolute claim — Cancel restores the original, Keep
keeps the change — and those are worth having, but either one alone is
satisfiable by a wiring that ignores the distinction. A check that asserted
only "after Cancel the setting is the original" passes perfectly against a
build that never saves anything at all, which is precisely the build O185
replaced.

## And the order the three are tested in is not cosmetic

The cross-run comparison is tested **first**, and it has to be, because in
the order this file was originally written — Cancel-restored, Keep-kept,
then the comparison — **the comparison could never fire**. The per-run guard
establishes that the window did not open on the value about to be clicked;
given that, "Cancel reopened on what it opened with" and "Keep reopened on
the clicked value" together already imply the two differ. The claim the
module header calls load-bearing was three lines of unreachable `if`.

⚠ **Both falsification runs went red anyway, which is how it survived
them** — the two absolute claims caught both planted builds and reported
them well. A falsification proves the check *as a whole* discriminates; it
says nothing about whether every assertion inside it can be reached. Reading
the assertions as a system, and asking of each one *what input reaches this
line*, is a separate act and this file is the argument for doing it.

With the comparison first, all three are live: a degenerate build stops at
it, and a build where one route produces some *third* value — neither the
opening token nor the clicked one — passes it and is caught by the absolute
claim below.

# What this check deliberately CANNOT establish

**It never presses Print, and no future edit may make it.** Committing is
how a print job reaches a real device, and this suite runs unattended on the
machine whose default printer is the operator's plotter. Four other print
checks state that rule in their own words rather than by reference, because
the day somebody adds a sixth by copying one of these files, the copied file
is what they will read. This is the sixth, and it says it too.

One consequence is specific and worth naming, because a reader will
otherwise take this green for more than it is:

⚠ **`reverted=true` is unreachable from here.** `Cancel` does two things —
it declines to write, and it *puts back* anything already written — and only
the first is driveable. The second has exactly one reachable cause in the
program: **a spool the driver refuses.** `remember()` runs before the spool
and whether or not the spool succeeds, and a refused spool leaves the window
open, so an operator can press Print, be refused, change more settings and
then press Cancel — at which point the preferences on disk hold what the
failed press wrote. That is the one path where the word *revert* earns its
place over *decline*, and reaching it from here would mean pressing Print.

What stands in for it:

| The claim | What holds it |
|---|---|
| Cancel declines to write | **this check** — the Cancel run's reopen |
| Keep writes | **this check** — the Keep run's reopen |
| the two are different buttons | **this check** — the cross-run assertion |
| Cancel puts back a value already written | ⚠ **nothing automated** — see below |
| the settings survive to disk at all | `the_print_window_opens_on_the_settings_you_last_used`, which reads a seeded file back in a second process |

⚠⚠ **That fourth row says "nothing", and the word is measured rather than
modest.** An earlier draft of this table cited *"`remembered`'s unit
tests"*; that file has no `#[cfg(test)]` module, and neither does
`export_remembered`, which has the same shape — this layer's convention in
this crate is that a driven check covers it. A citation to a holder that
does not exist is worse than an admitted gap, because a reader who checks it
finds a plausible file and stops.

What genuinely constrains the write-back, short of a test:
`PrintDialog::store` is the **sole** writer of `prefs.print` and the sole
emitter of `print-remembered`, so `restore` cannot write by some other
route; it differs from `remember` only in the payload it hands over
(`opened_with` against `habits()`) and in the `how=` token; and
`print-dismissed reverted=` discloses at runtime whether a put-back
happened. **None of that is a measurement.** Reaching the real path needs a
spool the driver refuses, which needs a Print press, which this suite may
not make — so the gap is structural and is recorded as such in
`DESIGNS.md` §O185 rather than left as a silence somebody has to rediscover.

Neither half is claimed by the other, and a reader who took this green as
covering the failed-spool revert would be taking more than is here.

# The gesture, and why it is the paper policy and not something friendlier

The check has to *change a setting* between opening the window and leaving
it. Of the twenty-odd controls in the print window, **one** publishes a
rectangle a driver can aim at: the paper combo, `print.paper`, with its
entries under `print.paper.item.N` and `print.paper.auto`. Copies, collate,
reverse, duplex and the rest are drawn and unaddressable.

Within that combo the target is **`print.paper.auto`**, not a numbered
form, and the difference matters more than it looks:

- A numbered entry is one of the **driver's** forms, so which one exists and
  what it is called is a property of the machine the suite happens to be
  running on. `print_paper` has to try up to five of them in a loop for
  exactly that reason, and says so at length.
- Worse for *this* check, a hand-picked `Form(id)` is **not remembered as
  itself**. `paper_key` reduces every form to the token `device` — deliberate
  loss, argued on `habits()` — so choosing a form and choosing nothing come
  back from the preferences file as the same string. The gesture would be
  invisible to the oracle.
- `Auto` is pdfcer's own second *policy*, it is at a fixed region name
  outside the numbered namespace, and it persists as its own token,
  `match-pages`. One click, no loop, no machine dependence.

# The oracle: three lines, and the third is the one that matters

```text
print-open   … paper=device …                          (the window as opened)
print-plan   … pick=auto  auto=matched …               (the gesture took effect)
print-dismissed reason=revert saved=true reverted=false (which way out was taken)
print-open   … paper=device …                          (the window, reopened)
```

`print-open`'s `paper=` is spelled by the preferences file's own
`paper_key`, so the token this check compares is the token that would be
written to disk — not a second spelling free to drift. `print-plan`'s
`pick=` is a stable one-word token from `autopaper::pick_token` for the same
reason, and it is read here as a **precondition, not an assertion**: if the
click on the combo entry did not change the live choice, this check has
learned nothing about dismissal and says so as a SKIP rather than accusing
the application of a defect it caused itself.

`print-dismissed` did not exist before O185. Nothing traced a close at
all — no `print-close`, no event on `frame.closed` — so a build that took
the wrong branch on the way out was indistinguishable from one that took the
right branch and wrote nothing. This is the fifth time in this project that
sitting down to write a driven check found a trace that could not tell apart
the two states the check exists for.

# `reverted=false` on a green Cancel run is correct, and is not a bug

The Cancel run's `print-dismissed` line reads `reverted=false`, and a reader
meeting that for the first time will read it as *the revert did not happen*.
It means the opposite of a defect: `prefs.print` still equalled
`opened_with`, because nothing in this session had written over it, so there
was nothing to put back. `restore` reports what it *did*, not what it
*would have done*, which is what makes the field usable for telling a Cancel
on an untouched window from a Cancel that undid a failed print.

`saved=true` beside it is the other half: the preferences **now hold** the
settings the window opened with, which they do, trivially, by never having
stopped.

# Every way this reports SKIP

No binary; `--no-input`; no `--pdf` (the Print command is gated on a
document being open, so the ribbon control is greyed and there is no dialog
to reach); no ui-rect channel; the ribbon control not declared; the dialog
not opening; the spooler refusing on this machine; the paper combo's popup
not opening within the settle; the Auto entry leaving the live choice
unchanged; or the shipped default already being `match-pages`, which would
make the gesture a no-op and every assertion below it vacuous. Each says
which, and none of them is reported as a pass.
