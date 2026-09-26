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
