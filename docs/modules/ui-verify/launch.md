# `ui-verify/launch`

Start the built binary, capture its diagnostic trace, find its window, and
never leave it running.

## The binary, not a test harness

This module launches `pdfcer-gui.exe` — the actual artefact, built in
release, opening an actual file. That is the entire premise of the crate:
D1 and D2 both live in the space between our code and the framework, and
neither is reachable from inside a test binary that constructs an
`egui::Context` by hand.

## Two guarantees this module owes the operator

**1. It never kills a process it did not start.** The operator may well
have the application open for their own work — this harness drives the real
desktop, which is precisely the situation where that is most likely — and a
harness that tidied up by killing "all pdfcer-gui processes" would close
their document. So [`Session`] holds a child handle and kills exactly that.

**2. It never leaks the process it did start.** pdfcer's predecessor script
killed its child on its last line, so any error before that line left a
window running: parked off-screen, invisible, and still consuming pointer
input on the operator's desktop. The operator reported it as *"do you have
some gui processes leftover that are interfering with my mouse?"* — twice
in one session, which is what made it a defect in the tool rather than an
operating mistake. Here the kill is in [`Drop`], so it happens on every
path including a panic.

## The staleness gate

[`Session::launch`] refuses a binary older than the newest source file
under `crates/`, unless explicitly told not to. The failure this prevents
is the worst kind: the traces a developer expects are simply **absent**,
which reads as "the feature does not work" rather than "the feature was
never compiled". pdfcer recorded an agent nearly concluding a panel did not
render, when the binary predated every change it had made.

An absence is only evidence when the thing that would have produced it was
actually built.

## Item notes

### `const SAFE_ORIGIN_X`

This is a **mitigation, not the guard.** `Driver::confirm_uncovered`
refuses a click on a point another window owns, wherever the window is;
this only makes that refusal rare. A machine whose furniture docks
somewhere else will still be caught, and will be told so.

### `const MIN_CLIENT_PX`

Not a guess at the application's size — a floor below which the window
cannot be a laid-out application window. See the polling loop in
[`Session::launch`] for what happens without it.

### `fn died_installing_accessibility`

Decided from the stderr the launch wrote, not from the exit code alone:
every panic exits 101, and a panic in the application's own startup must
stay reported.

### `fn place`

Two reasons, and the second is the one that cost an afternoon.

**Determinism.** Letting Windows choose gets a *cascade*: every launch
steps down and right from the last, so a long session marches its
windows toward the edge of the desktop. A failure that depends on where
the window happened to open is a failure nobody can reproduce.

### `fn frame_count`

Reads the tail of the trace rather than the whole file: the counter is
emitted every tenth frame, so the answer is always within the last few
hundred bytes, and a sweep against a long-running session would otherwise
re-read megabytes on every settle.

### `fn launch`

# Errors

Every error here is a **precondition** failure — the harness could not
begin — so callers report SKIPPED, not FAIL. Each message names the
specific thing that was missing.
Launch, retrying a launch the MACHINE killed before a window existed.


`accesskit_windows` installs its window subclass with `SetPropW`, and on
this operator's 208-hour session that call fails **intermittently** with
`HRESULT(0x80070008) "Not enough memory resources are available"` —
4 of 5 launches of tonight's build, and **2 of 4 of a twelve-hour-old
release build**, at identical free RAM (3.8 GB), 157 k kernel handles,
under 2 k USER/GDI objects, a 448-entry kernel atom table, a USER atom
table that still registers fresh names, and 40 GB of free commit. Two
earlier attributions — the OneDrive mirror's handle leak, then
Outlook's — were each measured and each wrong; what is measured is that
a binary that shipped and worked fails the same way, so the subject is
the session, and the only remedy known is a logoff.

# Why retry here rather than report SKIP

A sweep on such a session reported **11 of 14 checks SKIPPED** with
*"exited with exit code 101 before showing a window"* — a run that
verified nothing while looking like it ran. The panic happens before
the application has drawn a frame, so nothing the check is about has
been measured or spoiled; relaunching is the same experiment. It is
retried only when the stderr carries that exact HRESULT — any other
pre-window exit is the application's and is still reported as it was.

Every retry is printed, so a run that needed three launches per check
is visibly a run on a sick machine rather than a clean pass.

### `fn frame`

Re-measured on demand rather than cached: the window can be moved or
resized between one assertion and the next, and a cached frame would
convert against a geometry that no longer exists — which produces
clicks that land near the target, the hardest failure to diagnose.

### `fn maximize`

# Call this before looking for a control the tab lists LAST

A ribbon overflows when it is wider than its window, and a control in
the overflow **stops publishing a rect** — which a check cannot tell
apart from a control that does not exist. `settings_theme` found this
the hard way: it asked the File tab for `ribbon.item.file.settings`, was
handed ten controls ending at `file.print`, and would have reported a
shipped feature as missing.

It is opt-in per check rather than done on every launch, because a
maximised window is a **different layout**, and several checks measure
things — the canvas rect, the find bar's placement, the page strip — for
which the size is part of the subject. Making it universal would change
what those are testing without changing a line of them.

A no-op on platforms with no window control, exactly as [`Self::raise`]
is: a check that cannot maximise still runs, against whatever size the
window opened at.

### `fn expect_exit`

Safe to call while it is still running: the trace goes to stderr
unbuffered, one line per event, and reading a file another process has
open for writing is permitted on Windows with the share mode Rust's
`File::open` requests.

### `fn trace`

An outside reviewer opened `pdfcer ▸ Keyboard shortcuts` on a fresh
launch and the **process aborted**, taking the operator's unsaved markup
with it. `dialogs_open_in_their_own_window` drives that exact dialog and
had been reporting

> Keyboard shortcuts is a real OS window: [[186.0 209.0] - [606.0 689.0]]

**PASS, on the crashing build.** Not by luck: the `viewport-inner` line
the check reads is written *before* the panic, so by the time the
process died the evidence the check wanted already existed. The check
was not wrong about what it asserted. It simply had no opinion about
whether the program was still alive, and neither did any of the others.

⇒ **Every trace-reading check in this harness could pass on a build that
crashes**, provided the crash comes after the line it greps for. That is
a whole-harness defect, so the fix is in the one function they all call
rather than in a rule each of them has to remember — a hand-written list
inside a completeness sweep being exactly the shape this project has now
been caught by three times.

A check that legitimately expects an exit — `ctrl_s_after_an_edit_saves_and_the_program_is_still_running`
asks the question directly, and a "does it quit cleanly" check would —
calls [`Session::expect_exit`] first. That is greppable, and it is a
statement rather than an omission.



> the raster exists and the shell is not putting it on screen. THIS IS
> THE DEFECT.

It was not. The capture contained, **573 times**:

> thread '<unnamed>' panicked at tiny-skia-0.11.4/src/pipeline/mod.rs:188:9:
> range start index 356280245632 out of range for slice of length 1088737

The render worker was dying on every spawn. The process was perfectly
alive — window up, event loop running, ribbon responsive, every trace
line the check greps for written on schedule — so the liveness test
above had nothing to say, and the check went looking for its explanation
in the only place it knew about.

**A misattributed failure is worse than a missed one.** A missed failure
gets found later; a misattributed one sends somebody to rewrite a
correct module. The fix belongs here rather than in that check for the
same reason the process-exit guard does: it is a property of *every*
trace-reading check in the harness, and a rule each check has to
remember is a rule most of them will not.

Measured before it was adopted: of the 53 captures the previous full
sweep left behind, **none** contained a panic line. This detection turns
red the checks that are looking at a broken renderer, and no others.

# Errors

The exit is reported as a hard error rather than a SKIP: a process that
died is a failure of the thing under test, not a missing precondition.
Callers turn `Err` into SKIP by convention, so the message says plainly
that this one is different, and the panic line is lifted out of the
trace into the message because that is the sentence somebody needs.

### `fn settle`

Named in frames because that is the unit the thing being waited for is
measured in: a raster rebuild, a layout pass, a provider swap.


The whole body was `sleep(frames * 25ms)`. On an idle machine 25 ms is
about a frame and the name is nearly true. **Under load it is not** — the
application renders fewer frames in the same wall time, so every check
that settled and then clicked was acting before the interface had caught
up.


# How it waits now

The application emits `frame n=<count>` on the diagnostic channel every
tenth frame. This reads the newest such line, then polls until the count
has advanced by `frames`. Fast when idle, patient when loaded — which is
what the name always claimed.

**The old sleep is the floor, not the ceiling.** A short wall-clock
wait still happens first, because some of what a check waits for is not a
frame at all — a file written, a child viewport created, an OS window
map. Removing it would trade one class of flake for another.

**And there is a cap**, after which it returns rather than blocking.
An application that has stopped drawing is a finding for the check's own
assertions to report, in their own words, against the state they can see.
A settle that waited forever would turn every such defect into a hung
suite with no message at all — which is strictly less informative than
the false failure this change removes.

### `fn has_exited`

`try_wait` rather than `wait`: it must never block. A check calling this
is asking a question, not waiting for an answer.

`&mut self` is why `Session` is held mutably by the one check that
uses it. Reaping here is harmless — `Drop` kills and waits again, and
both tolerate an already-exited child.

### `fn staleness_complaint`

Deliberately a *complaint string* rather than a bool: the message has to
carry both timestamps and the rebuild command, because whoever sees it is
about to spend an hour diagnosing a feature that was never compiled.
