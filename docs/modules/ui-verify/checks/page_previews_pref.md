# `ui-verify/checks/page_previews_pref`

`the_page_preview_limit_is_remembered_and_zero_means_never` — **the two
halves of O187, driven.**

# The report


> *"the draw page previews timeout needs to be remembered, and setting it to
> 0 should set it to infinity (never time out)"*

Two sentences, two entirely separate mechanisms, and a build can ship either
one without the other:

1. **Remembered** — the Pages panel's previews tick and the time limit
   beside it now write through to `preferences.txt` the moment they change,
   and are read back once at construction. Before O187 both were
   session-only, so an operator who cleared the tick because previews were
   slow on their sheet set met them again on the next launch.
2. **Zero means never** — the limit's type became `Option<Duration>`, `None`
   is *no limit*, and `0` is the operator's own notation for it in both the
   control and the file.

The second is the one most at risk from the first, and that is the whole
reason this check exists in the shape it does. Every other small number in
that control is raised to a 100 ms floor, because a one-millisecond budget
is an off switch wearing a number. `0` has to pass through the same clamp
**and come out the other side untouched**, twice — once on the way into the
cache and once on the way back out of the file. A clamp that treated it like
its neighbours would quietly turn *no limit at all* into the default, and
the operator would experience that as *the box refusing to keep the zero he
typed*.

# Why this is a THREE-PROCESS check

Because the subject is a value **surviving a process**, and one launch
cannot express that. Neither can two, for this particular pair:

| Launch | Gesture | What its successor proves |
|---|---|---|
| 1 | clear the previews tick | the **tick** survived a close |
| 2 | type `0` into the limit | the **limit** survived a close, as a zero |
| 3 | nothing — it only reads | both, together, from a file written by two different gestures |

A two-launch version would have to change both controls in the same process,
and then a build that wrote only the last field touched would still pass.
Splitting the two gestures across two processes is what makes the third
launch's `previews=0 budget_ms=0` a statement about **both** halves of the
whole-file write rather than about whichever one happened to go last.

# Why each process is KILLED and not closed gracefully

This is the deliberate opposite of [`super::page_display_pref`], and the
contrast is the point. That check presses `Alt+F4` because its subject is a
**debounced** write that only an exit hook can rescue — kill it and you are
measuring whether the debounce happened to expire, which is true on a slow
machine and false on a fast one.

O187's write is not debounced. Property 4 of the preference family
(`app::actions::prefs`' header) is *one discrete operator decision is one
write, **now***. So killing the process is not a shortcut here — it is the
assertion. A build that wrote the preference from an exit hook, or from a
750 ms debounce, would pass a graceful-close check and fail the operator the
first time the program was closed from the Task Manager or fell over. This
check closes in the way that gives the feature no help at all.

# The oracles, and why there are three of them

| Oracle | Line | Answers |
|---|---|---|
| the gesture | `page-previews-persisted on=… budget_ms=…` | the verb ran and carried **both** values |
| the next launch | `pages-panel … previews=… budget_ms=…` | the file was read back into the live cache |
| the file | `page_preview_budget_ms = 0` in `preferences.txt` | the number on disk, independent of either trace |

The third is not redundant. The first two are both written by the program
under test, in the same run, from values that could in principle both come
from the same wrong place. Reading the file is the one observation this
harness makes that the application cannot have coloured — an oracle built
from the system under test needs an independent calibration, and the file is
it.

`budget_ms` is deliberately **not** compared against whatever
`thumbnails::PAGE_BUDGET_DEFAULT` currently holds. The starting state is
asserted against the number this harness planted and the end state as
*"zero"*, which is a real change in a known direction that no program
constant can make true by accident.

# The starting limit is PLANTED, because the shipped default is the
value this check types

`previews=1` and `budget_ms != 0` before the first gesture — and the second
of those is **not** a shipped default. The shipped default is *no limit*,
which is `0`, which is the exact value launch 2 types in; a run that began
there could not tell a zero that was kept from a zero that was never
changed, and would report a pass for a build that persists nothing at all.

So the seed carries `page_preview_budget_ms = 2000`. That `2000` is the
harness's own number, not a copy of a program constant — it has no
obligation to match anything in the shell, and nothing in the shell going
stale can make it wrong. What it has to be is *a legal, non-zero limit*,
and it is checked after the launch rather than assumed, because a seed the
program declined to read is indistinguishable from one it read and ignored.

⇒ The same discipline applies to the tick, which **is** a shipped default:
it is planted by the same reset and confirmed out loud anyway. *A fixture
that defeats a default does not defeat a starting state.*

# ⚠ What is normalised, and the one file that is RESET rather than deleted

Only `preferences.txt`, and through [`crate::sandbox::write_prefs`], never
`fs::write` and never a delete. The sandbox header carries
`ask_default_app = false`; three checks that wrote the file directly
re-enabled the O173 startup offer in front of their own launches, and one of
them then measured the *dialog's* client area and reported a working
preference as broken. A [`RestorePrefs`] guard puts the seed back however
this check ends, because a suite that shares state measures the order it ran
in.

Safe because the suite is **never** pointed at a published build — that
is the standing rule, and a check that rewrites `userdata/preferences.txt`
is one of the reasons for it.

# Every way this reports SKIP

No binary; `--no-input` (this check is entirely pointer and keyboard); the
fixture missing; the profile declaring no ui-rect event; the preferences
file not writable; the Pages panel's controls not declared; the mode segment
not declared; the control chord producing no `chord-command` line, which
means no keystroke reached the window and nothing typed below would mean
anything; or the starting state not being the planted one, which means
either the seed was not read or something outside this check reached the
sandbox.
