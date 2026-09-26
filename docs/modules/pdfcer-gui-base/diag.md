# diag — an opt-in trace of what the shell actually received

This file is the channel itself — [`enabled`], [`trace`], and the
change-gated writers built on them. Nothing in it is a feature; it is the
instrument every other module of the application is measured with, and the
contract below is what keeps it safe to leave switched on in the source.

## Why this exists

A GUI defect in this project has exactly one honest oracle: the running
application (**R1**). Everything else — reading the dispatch chain,
unit-testing the pure decision functions, checking the CLI's answer to the
same query — can be entirely green while the operator still cannot select
an object, because the thing that failed sits between the window manager
and our first line of code.

⇒ The candidates that survive a careful reading are all of the form *"does
`Response::clicked()` fire at all"*, and **that is unobservable from the
source**. It has to be reported from inside the process, on the frame it
happens, or it is not observed at all.

## Why it does not just take a screenshot

The operator uses this machine for real work and has asked that the screen
not be commandeered. So the diagnostic comes out of the process as *text*,
from a window that need never be looked at — which also makes it usable
from a script, a CI run, or a machine with no display at all.

## Contract

- **Off unless asked.** Enabled only when the `PDFCER_DIAG` environment
  variable is set to a non-empty value, read once per process. With it
  unset, [`enabled`] is a relaxed atomic load and [`trace`]'s argument
  closure is never called — so a call site costs nothing and may be left in
  place permanently rather than added and deleted around each investigation
  (which is how the *next* defect ends up needing this file written again).
- **Writes to stderr, one line per event, `key=value` fields.** stderr
  because it needs no path, no handle to keep open, no failure mode of its
  own, and redirects with `2>`. `key=value` because the consumer is a grep
  or an LLM, not a person reading a log.
- **Never a user-facing string.** Nothing here is shown in the interface, so
  none of it belongs in `pdfcer_gui::text` (the ui-string catalog governs
  operator-visible copy).
- **Never load-bearing.** No behaviour may depend on the trace. If deleting
  this module changed what the application does, the trace would have become
  a feature with no tests.

## Usage

```text
PDFCER_DIAG=1 pdfcer-gui file.pdf 2> trace.txt
```

---

## What the harness is owed, and by what

`PROJECT_PLAN.md` §4.3 tabulates the three contracts this module honours so
that `tools/ui-verify` needs no workarounds; each of them removes a harness
workaround, and two are implemented by machinery in this file.

### The de-duplicating gate ([`trace_changed`])

The trace is written for a *machine* consumer, and the machine's question
is almost always *"what is the current value of X?"* — answered by the
**last** line carrying X. A call site in the frame loop that re-emits an
unchanged value 60 times a second answers that question no better and
buries every other event while doing it. Measured: an ungated
`canvas-pointer` line produces **50 identical lines in 9 seconds** with the
pointer stationary, because it fires once per frame rather than once per
movement.

That is not merely untidy. `ui-verify` reads the trace file repeatedly
while it drives (`Session::trace` re-parses the whole capture after every
settle), so per-frame noise is re-parsed on every read and grows the
capture without adding information. Worse, it makes a human reading the
trace scroll past thousands of lines to find the one event that matters —
and a rejection line that is traced on every run is one a reader stops
seeing, which is how a capture that contains the answer still fails to
deliver it.

So: [`trace_changed`] remembers the last line emitted under a **slot**
and emits only when the newly built line differs. "Changed" is defined as
*the formatted line differs*, which is deliberately the same definition
the consumer uses — a difference too small to change the printed text is,
by construction, a difference the consumer could not have read anyway.

### The named-region sink ([`ui_rect`])

§4.3 requirement 2. A pixel check needs to know **where** to look, and
there are only two honest sources: the application measures the rect on
the frame it reports (correct under every layout change), or the harness
hard-codes a fraction of the window (stale the first time a panel is
resized — the hazard §4.2 prerequisite 1 names). [`ui_rect`] is the first
source.

It is a **process-global sink on purpose**, and that is the seam.
`egui-shell` cannot depend on this crate —
`tools/gates/check-shell-purity.sh` enforces the one-directional dependency
— so the ribbon and the dock take a *rect sink* from their caller and this
module supplies one that forwards to [`ui_rect`]
(`pdfcer_gui::app::surfaces`). Because the sink captures nothing and needs no
`&mut` threaded through every widget signature, the shell can grow new
named regions without a line changing here.

### Zero-cost when off, in both

Both check [`enabled`] before touching their registries, so with
`PDFCER_DIAG` unset a call site costs one relaxed atomic load and no lock,
no hash, no allocation and no formatting. That is what makes it correct
to leave these calls in permanently — see the contract above.

## Item notes

### `static LAST_LINE`

A `Mutex` rather than a `thread_local!` or a `RefCell` because
[`ui_rect`] is designed to be handed to `egui-shell` as a plain
`fn(&str, Rect)` callback (see the module docs), and a callback whose
correctness depends on which thread invokes it is a trap for whoever wires
it up. The lock is uncontended in practice — everything that traces layout
runs on the UI thread — and it is only ever taken when tracing is on.

Keys are `&'static str`, which is not an accident: a slot names a *call
site*, and call sites are known at compile time. It also means the
steady-state (nothing changed) path performs **no allocation at all** —
only a hash of a string that already exists.

### `static LAST_UI_RECT`

Separate from [`LAST_LINE`] and typed as a [`egui::Rect`] rather than as a
rendered string for two reasons: region names are runtime values (a ribbon
group's caption id is data, not a literal), so they cannot key
[`LAST_LINE`]; and comparing the rect itself rather than its rendering
keeps the comparison independent of the format the line happens to be
printed in.

### `static UI_RECTS_THIS_FRAME`

## Why this exists: the trace is a CHANGE LOG, and a change log cannot
say that something stopped

[`ui_rect`] emits only when a region's rect *differs* from the last one
emitted for that name, which is what keeps the channel usable — a per-frame
dump of ~60 regions at 60 fps is a torrent nobody can read. The cost is
that a region which stops being drawn **emits nothing**, so its last known
rect stands in the trace forever and a reader has no way to tell "still
there, unmoved" from "gone forty frames ago".

That is not academic, and the failure it produces is **confident and
wrong**: a ribbon whose overflow has correctly swallowed a control leaves
that control's last rect standing in the trace, at the position it held
under an earlier layout, and a harness measuring it reports a live layout
defect against a fossil. A screenshot of the same frame shows a perfectly
laid-out ribbon.

So [`end_ui_frame`] diffs this set against the previous frame's and emits
`ui-rect-gone name=…` for anything that disappeared. The log stays a change
log and becomes an *honest* one, reporting both directions of change.

### `fn lock`

A panic while one of these locks was held would otherwise disable the
trace for the rest of the process — and the trace is the thing you reach
for *because* something went wrong. `into_inner` keeps the channel alive
on a possibly-stale map, which can at worst cost one duplicate or one
suppressed line. The contract says the trace is never load-bearing; this
is that contract applied to its own failure mode.

### `fn record_if_changed`

Split out from [`trace_changed`] — and taking the map as an argument
rather than reaching for the global — so the de-duplication rule is
testable without an environment variable, without stderr capture, and
without two parallel tests fighting over one process-global registry. The
rule is the interesting part; `eprintln!` is not.

Returns whether the caller should print, and records the line if so.

### `static VIEWPORT`

A thread-local rather than a parameter because `ui_rect` is called from
~40 sites, none of which knows or should know that dialogs exist. It is
safe as a thread-local for a reason specific to *immediate* viewports:
egui runs a child's callback **synchronously, inside the parent's
frame, on the parent's thread**, so the scope is a straight-line region
of one call stack rather than a global mode.

### `fn viewport_suffix`

Empty for the application's own window, so **every existing trace line is
byte-identical to what it was** and no consumer has to learn anything to go
on working. A harness that never opens a dialog sees no change at all; one
that does gets a field it can ask for. That is the cheaper half of the
change and it was a deliberate choice over tagging every line with `root`.

### `const FRAME_TICK_EVERY`

Ten, which is one line per ~250 ms of animation and per ~0.2 s of a busy
redraw — negligible beside the hundreds of `ui-rect` lines a frame already
emits, and fine enough that `Session::settle` can wait on a count rather than
on a clock.

### `fn record_rect_if_changed`

The comparison is exact rather than epsilon-based, deliberately. An
unmoved region is laid out from the same inputs every frame and produces a
bit-identical `Rect`; a region that moved by a quarter of a point moved,
and a check measuring it wants to know. There is no third case in which an
epsilon would help.

### `fn a_disabled_trace_never_builds_its_message`

This is the property that lets call sites be left in permanently:
the moment a disabled trace still formats its message, every one of
them becomes a per-frame allocation and the next engineer starts
deleting them again.

The test is written so it is meaningful in BOTH environments: if the
harness itself runs under `PDFCER_DIAG`, the closure is expected to
run, so the assertion follows `enabled()` rather than assuming it.
