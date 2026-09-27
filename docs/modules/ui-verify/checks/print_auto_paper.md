# `ui-verify/checks/print_auto_paper`

`match_the_pages_picks_the_sheet_from_the_document` — **operator request
O167, driven.**

# The report


> *"we also need the option to auto select paper size based on the page
> sizes in the pdf."*

The Paper control's second entry — **Match the pages in this document** —
is that option. Choosing it measures every page of the job at its rotated
extent, picks the smallest enumerated sheet that holds them all with each
page free to lie either way round, and asks the driver for that sheet.
When nothing holds them it takes the largest available and says so.

# The claim this check exists for, and why the obvious one is not it

Four things could be asserted after clicking that entry, and three of them
are worth almost nothing on their own:

| Evidence | What it actually proves |
|---|---|
| `pick=auto` | the click landed on the entry |
| `auto=matched` | the decision ran |
| `paper=Form(8)` | its answer was turned into a request the driver will see |
| `largest=` fits `sheet=` | **the sheet has something to do with this document** |

A build that resolved auto to the first form in the driver's list would
emit the first three, correctly, and be completely wrong. The operator's
words were *"based on the page sizes in the pdf"*, and only the fourth row
is about the pages. That is why `largest=` was added to `print-plan` on
2026-09-10 — while writing this check, which is the fifth time in this
project that sitting down to write a driven check found a trace that could
not tell apart the two states the check existed for.

So the assertion is the **invariant**, not the value:

> `auto=matched` means the largest page fits the planned sheet either way
> round. `auto=toobig` means it does not.

That holds on any machine, against any driver's paper list, with any
fixture. Nothing here hard-codes A3, and nothing here needs to know what
printer this PC has — which is the difference between a check that runs on
the operator's machine and one that runs on mine.

## The band is the application's DECLARED tolerance — and the first
run is what taught this check that

This section is written out of a correction, because the corrected version
is the one worth reading and the reasoning that produced the wrong one is
the reason somebody would write it again.

**What was assumed.** `largest=` and `sheet=` do not come from the same
place: the fit decision is made against what the driver said when it
*enumerated* its forms, and `sheet=` is what `printer_caps_for` returns when
the job is planned *for* the chosen form — a second call into a second
device context. So the first version of this check treated any small
disagreement as driver noise and reported **SKIP**: "I could not tell".

**What the first driven run measured.** On the benchmark CAD drawing:

```text
before: pick=device auto=off    paper=DeviceDefault sheet=792.00x612.00   largest=none
after:  pick=auto   auto=matched paper=Form(8)      sheet=1190.40x841.68 largest=1191.00x842.00
```

The page is **0.60 pt bigger than the sheet** and the application said
`matched`. Under the assumed model that was inconclusive noise, and the run
reported SKIP. It is nothing of the sort. `autopaper::fits` is
`pw <= sw + FIT_TOLERANCE_PT` with `FIT_TOLERANCE_PT = 2.0`, and the
application overhangs by design: CAD producers write A3 as a clean
1191×842 pt and the driver calls the same sheet 1190.4×841.68. **A page
0.6 pt over an A3 form is a correct match by the rule the application
states about itself**, and a harness that rendered that as "I could not
tell" was hiding the application's declared behaviour behind its own
uncertainty.

**What it does now.** The comparison is asymmetric, because the two
outcomes make opposite claims and only one of them tolerates an overhang:

| Outcome | The claim | This check fails it when |
|---|---|---|
| `matched` | *"this sheet holds the pages"* | the page is over by **more than [`OVERHANG_PT`]** |
| `toobig` | *"nothing here holds them"* | the page has **more than [`OVERHANG_PT`] to spare** |

[`OVERHANG_PT`] is the application's 2 pt tolerance plus 4 pt for the
two-different-calls effect that was assumed to be the whole story. It is
**6 pt, about 2 mm** — two orders of magnitude below the distance between
adjacent sheets (A4 to A3 is 246 pt on the short edge), so the thing this
check exists to catch cannot hide inside it. The build that resolves auto
to the front of the driver's list would have reported `sheet=792.00x612.00`
against `largest=1191.00x842.00`: **399 pt out**, not 0.6.

⚠ **There is no SKIP band any more, and that is deliberate.** A SKIP that
fires on the ordinary case is worse than no check: it is a green-adjacent
verdict nobody investigates, produced every single run. The one thing this
check is genuinely unable to decide has its own SKIP with its own sentence
(`auto=nobasis`), and it is a state the *application* declared, not one the
harness inferred.


*"A check that cannot fail is not evidence"* is a standing lesson in this
project, and this check was made to fail on purpose before it was believed.

**What was planted.** The exact defect the module header argues about:
`autopaper::choose`'s matched arm was switched from
*"the smallest enumerated form that holds every page"* to
`forms.first()` — the front of the driver's list, chosen without looking at
the document. Two lines. Everything else left alone: the pages were still
measured, `largest_page_pt` still filled from them, `mixed` still computed,
the resolution to a `PaperChoice::Form` untouched.

**What came back**, and this is the whole argument for `largest=` existing:

```text
after choosing auto: pick=auto auto=matched paper=Form(9)
                     sheet=841.68x595.20 largest=1191.00x842.00
```

**All three of the obvious assertions held.** The operator chose the
policy (`pick=auto`). The decision ran (`auto=matched`). Its answer reached
the driver (`paper=Form(9)`). A check built on those three would have
reported PASS on a build that put an A3 site plan on an A4 sheet. The one
that caught it was the fourth: `1191.00x842.00` does not lie on
`841.68x595.20`, **349.32 pt out**, and the check said so in those words and
named `autopaper::choose` as one of the two places to look. It was the right
answer.

The file was then restored from a kept copy — never `git checkout`, which
discards uncommitted work in the same file — verified by checksum, and the
check re-run green.

⚠ If a future edit to `autopaper::choose` or to the paper combo makes this
check awkward, **re-run that falsification rather than trusting the green.**
It costs two release builds and about four minutes, and it is the only thing
that has ever demonstrated that this check discriminates the state it names
from the three states that look like it.

# What this check deliberately CANNOT establish

**It never presses Print.** Four print checks state that rule in their own
words rather than by reference, because the day somebody adds a sixth by
copying one of these files, the copied file is what they will read: this
suite runs unattended on the machine whose default printer is the
operator's plotter, and a harness that can start a print job will
eventually start one by accident.

**It cannot read the disclosure sentence.** Under R8b rule 4 an inference
is reported off-canvas and never drawn onto the page, and auto selection is
an inference — so a sentence under the combo names the sheet chosen *and*
the largest page measured, and a second sentence names the tray flag when
the job is mixed. Those are `ui.label`s with no published rect; the harness
reads rectangles and trace lines, not text. What stands in for them:
`paper_auto_matched`, `paper_auto_too_big`, `paper_auto_no_basis` and
`paper_auto_mixed` are `ui_text` entries under the string gate, and
`AutoPaper::line` is unit-tested against each outcome. That is a weaker
claim than driving it and is written down as one.

**Its `toobig` arm is weaker than its `matched` arm, and knowingly so.**
`matched` names a specific sheet and this check can say whether that sheet
holds the document. `toobig` claims something about *every* sheet the
driver has — that none of them does — and the driver's form list is not on
the trace, so the strongest thing assertable from outside is that the sheet
reported really is too small. A build that took the FIRST form rather than
the largest would satisfy that whenever the first form is also too small.
Closing it would mean putting the whole enumerated list on the trace, which
is a diagnostic line per form on every frame; the arithmetic is asserted in
`autopaper::choose`'s unit tests instead. On this machine and fixture the
outcome is `matched`, so the strong arm is the one that runs — but that is
a property of the fixture, not of the check.

**It does not assert that the sheet chosen is the SMALLEST that fits.**
`largest=` and `sheet=` are two of the driver's sheets; the list of all the
others is not on the trace, so "no smaller sheet would also have held it"
is not answerable from here. It is the property `autopaper::choose`'s unit
tests assert directly, against a fixture list, and this project's standing
division of labour is that arithmetic is asserted where it lives and the
chain in front of it is asserted by driving the binary.

# Every way this reports SKIP

No binary; `--no-input`; no `--pdf` (Print is gated on a document being
open); no start line; the ribbon control not declared; the dialog not
opening; the spooler refusing; the device enumerating no forms (the combo
is not drawn at all then, deliberately); the popup not opening or closing
within a settle; the auto entry not declared; no `print-plan` line; the
click not moving `pick=`; `auto=nobasis`; and the slack-band case above.

Each of those is a state in which nothing was learned about the
application, and each says so in its own sentence rather than through a
shared one — because a reader of a SKIP wants to know what to do next, and
"something went wrong" is not that.

## Item notes

### `const PAPER_AUTO`

That is not a naming preference, it is the reason `print_paper_changes_the_plan`
still measures what it says it measures. Auto sits second on screen, so the
obvious thing would have been to give it index 1 and push the driver's forms
up by one — which would have left that check clicking a policy while its own
report said it had clicked a sheet, still green. `REGION_PAPER_AUTO`'s doc in
the application carries the full argument.

### `const OVERHANG_PT`

**2 pt of it is the application's own `FIT_TOLERANCE_PT`**, mirrored here
deliberately: `autopaper::fits` is `pw <= sw + FIT_TOLERANCE_PT`, so a page
up to 2 pt bigger than a form is a *correct* `matched` and a check that
called it wrong would be failing the application for doing what it says it
does. CAD producers write A3 as a clean 1191×842 pt; the driver calls the
same sheet 1190.4×841.68. That 0.6 pt is the whole reason the tolerance
exists, and the first driven run of this check measured exactly it.

**4 pt of it is the two-different-calls allowance** — the fit decision is
made against the driver's *enumerated* form size and `sheet=` comes back
from `printer_caps_for`'s device context, and a driver may round the two
differently.


⚠ **If the application's `FIT_TOLERANCE_PT` grows, this must grow with
it**, or a correct `matched` starts failing here. The two cannot be shared:
this is a separate crate and that constant is private. The dependency is
therefore stated, and [`the_band_is_at_least_the_applications_own_tolerance`]
pins the direction so a shrink here is caught by the compiler's own test
run rather than by a confusing red on the operator's machine.

### `const APP_FIT_TOLERANCE_PT`

⚠ Mirrored by hand from `crates/pdfcer-gui-base/src/printautopaper.rs`.
It is not imported because `ui-verify` does not depend on the application's
crate — it drives the built binary, which is the entire point of it. A
mirrored constant is a claim about someone else's source, and this project's
standing lesson about those is that they get a test which fails in the
direction that matters.

### `const _`

These were a `#[test]` until clippy pointed out that a comparison between
two constants is decided by the compiler and a test of it can never fail at
run time. It is right, and the right answer is not to silence it: a
relationship that is knowable at compile time should **break the build**,
not produce a red test somebody has to run first. So they are `const _`
items, and a violation is a compile error naming this file.

Both directions matter, which is why both are here:

- **Too small** and a *correct* `matched` fails this check — the
  application permits a page to overhang its form by `FIT_TOLERANCE_PT`,
  so any value below that makes the check red on a build doing exactly what
  it documents. That is the failure the first driven run walked into from
  the other side, and it cost a rewrite of the oracle.
- **Too large** and a sheet that is a whole size wrong is waved through,
  which is the only defect this check exists for. The bound is stated
  against the closest adjacent pair in ordinary use, A4 to A3 on the short
  edge — 246 pt.

⚠ The lower bound is a claim about **another crate's private constant**,
mirrored by hand into [`APP_FIT_TOLERANCE_PT`]. If that constant moves and
this one is not moved with it, this assertion still holds — it pins the
relationship, not the value. The tripwire for the value is the driven run
itself: a `matched` failing by a point or two is what a widened application
tolerance looks like from here.

### `fn plan`

The **last** line rather than the first: the dialog emits one per frame, so
the first describes the state it opened in and only the last describes the
state after a click. Reading the first would make every assertion above
trivially true, which is a mistake this project has made once and is worth
naming at every site that could make it again.

### `fn plan_field`

Separate from [`plan`] because it is used only for prose: `mixed=` is worth
printing beside a pass so a reader knows whether the job was single-size,
but no assertion turns on it and adding it to [`Plan`] would suggest one
does.

### `fn fit_clearance`

Positive means it fits with that much to spare on the tighter axis;
negative means it is over by that much. Signed rather than boolean so that
a failure report can say *how far* wrong the answer is, which is the
difference between "the sheet is one size down" and "the sheet is unrelated
to this document".

Both orientations are tried, and that is not a convenience: `dmPaperSize`
names a physical piece of paper and `dmOrientation` is a separate field with
its own control in this dialog. A3 and "A3 landscape" are not two sheets. A
clearance that respected page orientation would call an A3 sheet wrong for a
landscape A3 drawing, which is the commonest CAD export there is — and it
would do so as a FAILURE, against an application that was right.

### `fn a_size_token_parses_the_way_the_application_writes_it`

The application's own test asserts the same round trip from its side.
Both exist because they are two different claims: that one writes what
it means, and that this one reads what was written. A single test on
either side would leave the boundary itself untested, which is the
shape of defect this whole harness is for.

### `fn a_turned_page_lies_on_the_same_sheet`

The second half is the one worth having: a landscape A4 page on a
portrait A4 sheet must report room to spare, not a shortfall, because
which way the image lies is `dmOrientation` and not `dmPaperSize`.

### `fn a_page_too_big_reports_how_far_over_it_is`

The sign carries the verdict and the magnitude carries the diagnosis,
so both are asserted. An A3 page on an A4 sheet is over by the
difference on the tighter axis — not by the difference in area, and not
by zero.

### `fn the_measured_producer_rounding_case_passes_and_the_wrong_sheet_does_not`

The measured case, pinned: A3 written by a CAD producer as a clean
1191x842 against the driver's 1190.4x841.68 is `matched` and correct.
The defect case beside it: the same page against a Letter sheet, which
is what "auto resolved to the front of the driver's list" looks like on
this machine.
