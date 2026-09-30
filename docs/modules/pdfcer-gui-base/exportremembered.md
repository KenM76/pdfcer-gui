# `pdfcer-gui-base/exportremembered`

**What the three export windows remember between jobs** —
`OPERATOR_REQUESTS.md` **O196**, the operator's words of 2026-09-13:

> *"the export windows forget everything. every time I export a dxf I have
> to set it up again."*

This file is the **writing** half, shared by all three windows. The reading
half lives in each window's own `open`, which seeds every remembered field
from what it is handed.

# Why one file for three windows rather than three `remembered.rs`

`pdfcer_gui::dialogs::print::remembered` is the precedent and it is one window,
so it put its argument beside its projection. Here the projection is three
different struct literals — an image format is nothing like a DXF unit — but
**the argument is identical three times**, and an argument written out three
times is an argument that will be corrected in one of them.

So the split is by *what varies*:

| half | where | why |
|---|---|---|
| `habits()` — the projection | each window's own file | it reads that window's private fields, and the membership judgement is about *those controls* |
| `remember_*()` — the write | here | the no-op guard, the swallowed failure, the position of the call and the trace's shape are one decision made once |

# The three properties this file is responsible for

## 1. Nothing is written when nothing changed

Exporting the same page twice with the same answers is the commonest export
there is, and rewriting the whole preferences file on each one buys nothing.
The comparison is a plain `!=` on the group struct, which is why
[`crate::prefs::ExportImagePrefs`] and its two siblings derive
`PartialEq`.

⚠ The comparison is **per group**, not on the whole of
[`crate::prefs::ExportPrefs`]. An operator who exports a DXF and then
an image must not have the image write suppressed because the DXF group is
unchanged — and, in the other direction, a DXF export must not rewrite the
file merely because the image group differs from what it was at startup.

## 2. The failure is swallowed

[`crate::app::actions::prefs`] states the rule, as the fourth of the four
properties every preference verb shares: *"one discrete operator decision is
one write, and losing a preference across a restart does not justify a modal
in front of somebody who is"* — here — *about to export*. A read-only
`userdata` folder must not turn an export into an error dialog; the export
is the operator's actual errand and it proceeds unchanged.

## 3. The trace spells values as TOKENS, never `{:?}`

This project's standing lesson, learned on the print window: never
`Debug`-format a field a machine reads. A `{:?}` on a payload-carrying
variant prints the payload too, so a driven check grepping `scale=custom`
misses `scale=Custom(2.5)` **while quoting the truth in its own failure
message** — a confident false negative that reads as an application defect.

Every value below goes through the same `*_key` function the preferences
file itself uses, so the token a driven check reads and the token on disk
cannot drift.

# Where these are CALLED, which is the part that is a decision

At the **Export press**, immediately before the `Action` is pushed — never
when the window closes.

Closing without exporting is how a person says *"not this"*: they opened the
window, changed the resolution, thought better of it, and cancelled.
Persisting on close would make that abandoned configuration the state the
next export opens in, which is the opposite of what cancelling means.

That is [`crate::dialogs::print::PrintDialog::remember`]'s ruling, applied
unchanged, and it is stated here as well because the three call sites are in
three other files and a rule visible only from the print window is a rule
the next export window will not find.

## Item notes

### `fn remember_image`

Takes the group **by value** rather than by reference: the caller has just
built it out of its own fields and has no further use for it, and a move is
what makes the assignment below a store rather than a clone.

### `fn remember_dxf`

⚠ **No `scale=` here, and its absence is the design rather than an
oversight.** The DXF scale is derived per open from the page's own
dimension groups and is deliberately not a remembered preference — see
[`crate::prefs::ExportDxfPrefs`] for the argument. A trace key naming
a value this function does not store would be the first place somebody
looked when the scale failed to survive a restart, and it would tell them
the opposite of the truth.
