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

## Item notes

### `fn launch`

`tag` names the artefact, so a failing run leaves all three traces side by
side — which is the first thing anybody reading a failure here will want,
because every assertion in this file is a comparison between two launches.

### `fn open_panel`

⚠ The `GRID` test is not an optimisation. The Pages panel is docked by
default, and its ribbon item is a **toggle** — pressing it on a panel that
is already up would CLOSE the surface under test, and every assertion below
would then report a missing control. `pages_drag::open_pages_panel` carries
the same guard at its call site and this is the same rule, stated here
because this module has three call sites for it.

### `fn click_region`

Through [`stable_rect`] rather than [`driving::declared`], because raising
a dock panel re-lays the dock out over several frames and `ui-rect` is a
change log: reading it the frame after a panel opens answers *where that
control was*, and a click aimed at a stale coordinate lands on the canvas
with no error anywhere. This project has that failure on record twice.

And through [`frame_of`] rather than `session.frame()`, which costs
nothing on a main-window region and survives the day this panel is allowed
to float into its own OS window — at which point its rectangles become
relative to *that* window's origin and every click would land hundreds of
pixels away, with plausible numbers and no error. Thirteen checks learned
that on one day.

### `fn keyboard_reaches_the_window`

Without this, a build in which the pointer works and the keyboard does
not would produce *"typing `0` into the limit wrote nothing"* — a confident,
detailed and entirely wrong report naming O187 as the culprit. `find_bar`'s
first run did exactly that against a build in which `Ctrl+F` worked.

`Ctrl+2` is bound to `mode.review` in the application's key table, and this
check is already in Review mode by the time it gets here. So the probe is
idempotent: it proves the channel without changing a thing the assertions
below depend on.

# Errors

No new `chord-command` line — reported as a SKIP at the call site, because a
check that types into nothing must never name a feature as the culprit.

### `fn type_the_zero`

**No `Ctrl+A` first, deliberately.** egui's `DragValue` selects the whole
of its displayed text the frame its edit gains focus, so the click has
already done it — and `Ctrl+A` is bound in this application to a document
verb whose guard depends on a text field being focused. Pressing it here
would make the check's own setup depend on the very guard that a sibling
defect in this project once broke, which is a dependency worth not having
when the alternative is nothing at all.

`Enter` rather than clicking elsewhere. The commit is on `ended` —
`lost_focus` — because `app::spinnerdraft` exists to stop a re-seeded value
throwing a drag away, and Enter is the only way to end the edit that does
not also press something else.

### `fn panel_state`

⚠ `last`, and the panel emits through `trace_changed` — one line per change
rather than one per frame — so the last one is the current state and not a
fossil from the frame the panel happened to be drawn on.

### `fn persisted_after`

Counted rather than compared against an absolute absence, because this
check performs two write-through gestures in two processes and a naive
`last()` would happily return the previous one. An absence assertion is only
as good as when its baseline was taken.

### `fn read_prefs`

# Errors

Unreadable — reported as a SKIP at every call site, because a file this
harness cannot read is a harness problem and must never be named as a defect
in the program.

### `fn value_of`

Deliberately a five-line parser rather than a call into the application's
own reader: the point of reading this file is to observe it with something
the program under test did not write. A harness that parsed it with the
crate's own parser would agree with the program by construction.

### `const SEED_BUDGET_MS`

Two seconds. Any legal, non-zero value would do — see the module header's
section on why it is planted at all. It is the harness's own number and is
not required to match anything the shell compiles in.

### `fn write_prefs`

Through `sandbox::write_prefs`, never `fs::write`, and never a delete.
The header it prepends carries `ask_default_app = false`; three checks that
wrote the file directly re-enabled the O173 startup offer in front of their
own launches, and deleting it does the same thing by omission — every absent
key takes its compiled-in default, and that one's is `true`.

The previews tick is left absent and takes its compiled-in default, which
is on. The limit is written, because its compiled-in default is the value
this check types — the module header carries that argument in full.

# Errors

The directory could not be created or the file could not be written. A SKIP
at the call site: a preference that could not be written means the check
never began.

### `struct RestorePrefs`

A guard rather than a line at the end, because there are a dozen returns
above and the one that gets forgotten is the one that leaves this check's
gestures — the previews tick cleared, a planted time limit — standing in
front of every check that runs afterwards. The next check to draw a Pages
panel would then find a grid that draws nothing, and report it. A suite that
shares state measures the order it ran in.

Reset rather than deleted, for the reason `ui_scale` records: a *missing*
file exercises the absent-file path, which is a different state and not the
one the other checks were written against.

Failure to restore is warned about rather than fatal — this type runs during
unwinding as well as on the ordinary path, and a harness that turned its own
housekeeping problem into a verdict would be reporting itself as a defect in
the program.

### `fn the_check_types_the_one_value_that_is_a_sentinel`

Pinned because every other number this control accepts is clamped to a
floor, and a check that typed `1` would pass against a build with O187's
second half missing entirely — `1` and `100` are both *a limit*, and the
operator would never know. Only `0` can tell the two builds apart.

### `fn the_regions_named_are_the_panels_own_controls`

A check that named `ribbon.item.view.panel_pages` as its control would
be asserting that a menu entry exists, which is true in every build that
has ever shipped and says nothing about the preference.

### `fn the_control_chord_changes_nothing_the_check_measures`

`Ctrl+2` puts the application into the mode it is already in, so the
probe cannot change any state an assertion reads. A probe bound to a
verb with a side effect would be a setup step pretending to be a
measurement.

### `fn the_file_oracle_reads_past_the_comments`

The header is comment lines and a blank; a parser that took the first
`=` it saw anywhere would read one of them. This is the assertion that
the independent oracle is actually independent *and* correct.
