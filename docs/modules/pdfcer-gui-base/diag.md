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

### `fn enabled`

Resolved once and cached: the check sits in a per-frame path, and re-reading
the environment there would put a lock and an allocation in the frame loop
to answer a question that cannot change after start-up.

### `fn trace`

Takes a closure rather than a `String` so a disabled build path performs no
formatting — the call sites interpolate rects, pointer positions and hit
counts, and doing that work every frame to throw it away would be a real
cost in the one loop that must not get slower.

### `fn trace_changed`

# What this is for

Frame-loop call sites. A value that is re-reported unchanged 60 times a
second tells a consumer nothing it did not already know from the previous
line, and buries the events that *are* news. See the module docs for the
measured case (50 identical `canvas-pointer` lines in 9 seconds) and for
why noise costs the harness real work rather than merely looking untidy.

# The definition of "changed", and why it is the formatted line

Not the underlying value: the **rendered text**. Two consequences, both
wanted:

* A difference too small to change the printed text is a difference the
  consumer could not have read anyway, so suppressing it loses nothing.
  The pointer trace prints `{:.2}`; sub-hundredth jitter is invisible to
  the parser by construction.
* A call site does not have to invent an epsilon, or keep a parallel copy
  of its own state to compare against. There is one rule, in one place.

# ⚠ A repeated value is silent, and a consumer must expect that

Suppression is invisible from outside. A call site reporting the same
result twice in a row writes ONE line, not two, so a consumer that marks
a point in the trace and then reads the next line for this slot finds
nothing at all when the second occasion produced an identical result.

A driven check written that way reddens through its absence branch rather
than through the branch comparing the value, and the two causes mean
different things: the call site was never reached, or it produced exactly
what it produced last time. An absence message must name both. A check
that has to tell two occasions apart needs something in the formatted
line that differs between them.

# Slots

A slot is the event name, plus a discriminator when one event has several
independent subjects. Two call sites sharing a slot will each suppress the
other's lines, which is a real bug and the reason the parameter is
`&'static str` — it is meant to be a literal you can grep for.

Costs nothing when tracing is off: the closure is not called and neither
registry is touched.

### `fn visible_enough`

`true` when at least [`VISIBLE_FRACTION`] of `rect` survives `clip`.


Because *a change to a diagnostic channel is exactly the kind that can be
green and wrong*, and the only way to write a test that fails on the wrong
behaviour is for the decision to be something a test can call. Everything
else in this module writes to a global map and to `stderr` behind an
environment variable, which is observable only by a driven run — and a
driven run is precisely what cannot tell you that a check silently became a
SKIP.

So the rule the whole visibility channel turns on is one pure function, it
is public, and `crates/pdfcer-gui/src/app/surfaces.rs`'s dock-sink test
calls it against rectangles a **real** `egui_shell::dock::Dock` produced.

Note what it does with a zero-area region: `false`. A rectangle with no
area cannot be 60 % anything. Said outright rather than left to fall out of
a division, because "a collapsed control is not visible" is a claim worth
being able to read and to test.

### `struct ViewportScope`

Entered by `pdfcer_gui::dialogs::host::Host::show` around a dialog's body. A
guard rather than a closure because the body needs `&mut` on the dialog it
belongs to, and threading that through a closure parameter would push the
borrow problem into every caller.

# What this is for, and the defect it is a fix for rather than a nicety

A region's rectangle is **relative to the viewport that drew it**, and a
harness that adds the application window's client origin to every rect is
right until a dialog opens in its own OS window — whose rectangles look
exactly the same and name a different place on the desktop.

A coordinate-space defect with plausible numbers is the one class this
project keeps meeting: a marker off by the scroll origin, a drag tracking at
`1/zoom`, a caret measured against the wrong font. Each presents only as
*"it lands somewhere else"*, and none of them is visible to a test that does
not drive the real window. The tag plus [`viewport_inner`] puts the fix in
the instrument rather than in anybody's care.

### `fn end_ui_frame`

Called once at the end of every frame, from `pdfcer_gui::app::frame`. See
[`UI_RECTS_THIS_FRAME`] for the defect this exists to remove — in one
sentence: a change log that only reports appearances lets a consumer read a
stale rect as a live one, and report a layout defect against a region that
is no longer drawn.

# What it emits

One `ui-rect-gone name=…` line per region that was drawn last frame and was
not drawn this frame. Nothing at all on a steady frame, which is the common
case and keeps the channel as quiet as it was before.

# It also forgets the region's last rect

Deliberately, and it is the half that is easy to omit. Without it, a region
that disappears and later comes back **at the same rect** would emit
nothing on its return — `record_rect_if_changed` would compare against the
remembered value and suppress it — leaving the trace saying the region went
away and never saying it returned. Forgetting on retirement makes a
reappearance always visible.

### `fn reset_change_gates`

Called when a document is opened. Without it, opening a second document
whose layout happens to be identical to the first would emit **no** canvas
line for the new document, and §4.3 requirement 1 is specifically *"at
least once per document open"* — a guarantee the consumer is entitled to
read as "there is a line for this document", not "there is a line for some
document whose numbers still happen to apply".

It is cheap and it is not per-frame, so it clears both registries rather
than trying to decide which slots a document open could have invalidated.

### `fn trace_on_change`

# Why this exists beside [`trace`]

Some facts are worth reporting and are only true per frame — whether a text
draft exists, whether the keyboard is owned, how long the draft is. Tracing
those with [`trace`] produces a line every frame at sixty hertz, which is
not a diagnostic; it is a denial of service on the reader, and the reader is
somebody already having a bad day.

A change log is the honest shape for a *state* rather than an *event*, and
this module already has one: [`ui_rect`] emits only when a rect moves. This
is the same idea for a string, keyed so several callers can use it without
interfering.

**It has [`ui_rect`]'s known weakness, stated rather than left to be
discovered.** A change log cannot report that something *stopped* — see
[`end_ui_frame`], which exists to close that gap for regions. Here the
equivalent is a state that ceases:
the last line stands, and a reader must not take it for "still true". Where
that matters, include the *ceasing* in the value — `draft=false` is a value,
not an absence, which is why the text-edit line reports it that way.

The closure is not called at all when tracing is off, exactly as [`trace`]'s
is: the whole cost of a disabled diagnostic is one atomic read.
