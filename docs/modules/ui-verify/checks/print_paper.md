# `ui-verify/checks/print_paper`

`print_paper_changes_the_plan` — choosing a sheet must re-plan the job,
not merely re-label it.

# What this is about


The interesting failure is not "the combo does not appear". It is this:

> The combo appears, the operator picks A3, the label reads A3, the job is
> **planned against the device's default sheet**, and the pages come out
> scaled for Letter on A3 paper with no clip reported and nothing to
> explain it.

That is the same defect `DeviceGeometry::from_caps` exists to prevent — a
plan computed against one sheet and printed onto another — arriving through
a second dimension. `printer_caps` reads the device's *default* `DEVMODE`;
`printer_caps_for` reads the geometry of the sheet the job actually asked
for. A shell that adopted paper selection without switching to the second
function would look entirely correct from inside: the label is right, the
request is sent, the paper is right, and only the *scale* is wrong.

**No unit test in this workspace can see it.** The conversion test pins
that `PaperChoice::Form(8)` becomes `PaperSelection::Form(8)`, and it would
keep passing. The geometry comes from a Windows device context, so the only
evidence that the plan followed the request is what a running process says
about a real driver.

# What it measures

One trace line, emitted every frame the dialog is open:

```text
print-plan printer="…" … orientation=Auto duplex=Simplex paper=DeviceDefault
           pick=device auto=off sheet=612.00x792.00 largest=none mixed=off
           config=false scale=Some(0.97) tab=PagesLayout
```

`paper=` is what was **asked for**; `sheet=` is the physical sheet the
geometry came **back** with. The check reads the line before and after
choosing an entry from the list, and requires both to move:


| `paper=` | `sheet=` | verdict |
|---|---|---|
| unchanged | unchanged | the click did not reach the entry — harness failure, reported as a skip with the rect it aimed at |
| changed | unchanged | **the defect** — the request was recorded and the plan ignored it |
| changed | changed | pass |

## Why the second row needs more than one click before it is believed

A driver is entitled to enumerate a form whose size **equals the sheet the
device already defaults to** — `dmPaperSize` naming Letter on a
Letter-default printer is not a bug, it is the list being complete. Choosing
that one moves `paper=` and correctly leaves `sheet=` where it was, which is
indistinguishable, from one click, from the defect above.

The harness cannot read an entry's label — it publishes rects, not text — so
it cannot pick a form it knows to be different. It therefore tries several,
and reaches the verdict only when **every** form tried leaves the sheet
unmoved. One that moves it proves the plumbing and stops the loop.

The number tried is capped and the cap is **reported**, not silent: a
driver's list can run to forty entries, five clicks is enough to settle the
question, and a report that said "tried the list" while trying an eighth of
it would read as coverage it did not have.

# What it deliberately does NOT do

**It never presses Properties….** That button opens the *driver's own*
modal dialog — a nested Win32 message loop owned by a vendor's print
driver, whose layout, controls and dismissal keys pdfcer does not know and
cannot publish rects for. A harness that opened one would be driving
somebody else's UI blind, and a driver dialog left up blocks the
application's event loop, so a failed dismissal does not fail the check —
it hangs the run and every check after it.

The button is still asserted, from outside: its rect is published as
`print.properties`, and this check requires it to be declared and
substantial. That is the whole of what a harness can honestly claim about
it — *it is drawn, where it is drawn, at a usable size* — and the rest is
a human pressing it once. The same reasoning, and the same limit, as
`print_dialog`'s refusal to press commit.

**It never presses commit.** Same rule, same reason, and it is stated in
both files rather than by reference: a harness that can start a print job
will eventually start one by accident.

## Item notes

### `fn last_plan`

The **last** line rather than the first: the dialog emits one per frame, so
the first describes the state it opened in and only the last describes the
state after a click. Reading the first is a mistake that would make every
assertion below trivially true and is worth naming.
