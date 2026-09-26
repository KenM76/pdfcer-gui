# `dialogs::print::autopaper` — pick the sheet from the pages

Operator request **O167**, 2026-09-10: *"we also need the option to auto
select paper size based on the page sizes in the pdf."*

## What this module is, and what it deliberately is not

It is **arithmetic over two lists**: every sheet the driver enumerated
([`super::spooler::PaperForm`]) and every page of the job measured at its
rotated extent. It answers one question — *which enumerated sheet should
this job be asked for?* — and returns that answer together with everything
the disclosure line needs in order to say what happened and why.

It is **not** a policy about mixed page sizes, not a call into the spooler,
and not a `Ui`. It opens no device context, reads no `DEVMODE` and cannot
print. That is on purpose: this is the half of the feature a unit test can
drive, and this project's standing lesson is that a verb's unit test cannot
see the chain in front of it — so the arithmetic is asserted here and the
chain from the combo entry to the spooled `dmPaperSize` is asserted by
driving the binary (`tools/ui-verify/src/checks/print_paper.rs`).

## The rule, stated once

> **The chosen sheet is the smallest enumerated sheet that contains every
> page in the job, with each page free to lie either way round on it. If no
> enumerated sheet contains them all, the largest enumerated sheet is
> chosen and the shortfall is reported.**

Three parts of that sentence are load-bearing and each is argued below.

### "every page", not "the first page"

A `DEVMODE` names **one** sheet and a job has many pages. Choosing from
page 1 would put a 40-page set of A3 details onto A4 because the cover
sheet happened to be A4, and nothing on screen would say why every drawing
came out at 71 %. Choosing the sheet that holds the *largest* page means
the biggest drawing is right and the smaller ones are scaled down onto a
bigger sheet — visibly generous rather than invisibly cropped.

Windows' own answer for a genuinely mixed set is the **choose tray by sheet
size** flag, which this dialog already exposes beside the paper control
(`spooler::DeviceSettings::pick_tray_by_page_size`). A device with more
than one roll or tray will then feed each page its own sheet, and the
`dmPaperSize` this module picks is what the rest lands on. The disclosure
says so when the job is mixed, because an operator with a mixed set needs
to know that one flag is the difference.

### "either way round"

`dmPaperSize` names a **physical piece of paper**; which way the image is
laid on it is `dmOrientation`, a separate field with its own control in
this dialog. A3 and "A3 landscape" are not two sheets. So the fit test
tries the page both ways round, and it is not a compromise — a fit test
that respected page orientation would refuse an A3 sheet for a landscape
A3 drawing, which is the commonest CAD export there is.

### "the largest, and reported"

When nothing fits — an A0 site plan on an office printer whose largest
sheet is A3 — there is no honest choice that makes the drawing come out
right. The alternatives were to fall back to saying nothing about paper
(which prints on whatever the device is standing on, chosen by nobody), or
to pick the biggest sheet the device has and **say that the page is bigger
than it**. The second is chosen: the operator asked pdfcer to match the
pages, and the closest available match plus a sentence naming the shortfall
is a better answer than a silent no-op. See
[`crate::text::print::paper_auto_too_big`].

## Tolerance, and why it is 2 pt

Producers do not emit exact ISO sizes. A4 is 595.276 pt and is written
`595.32`, `595.3`, `595` and `595.2756` by four different exporters; a
SolidWorks sheet set carries the drawing frame's size rather than the
standard's. A fit test with no tolerance would refuse an A4 sheet for a
595.4 pt page and step up to A3, which is a whole size wrong for two
hundredths of a millimetre.

2 pt is 0.7 mm. It is comfortably larger than every rounding divergence
measured in the producer-quirk notes, and comfortably smaller than the gap
between any two ISO or ANSI sizes (the closest pair in ordinary use is
Letter at 612 pt wide and A4 at 595.3 — 16.7 pt apart, eight times the
tolerance). It cannot promote a page to the next size up and it cannot let
a genuinely oversized page pass.

## What "smallest" means when two sheets are the same size

Drivers routinely enumerate the same physical sheet twice under different
names — `"A4"` and `"A4 210 x 297 mm"`, or a borderless twin. The tie is
broken by **the driver's own enumeration order**, first wins, which makes
the choice deterministic for a given device and matches what the operator
sees at the top of the list. Nothing better is available: pdfcer has no way
to know which of two identically-sized forms the device would rather have.

## Item notes

### `fn area`

Area rather than either edge, because the ordering has to be total: two
sheets can each be wider than the other on one axis (a 200 x 400 roll cut
and a 300 x 300 square), and "smallest" has to mean something for that
pair. Area is the measure of how much paper is consumed, which is what the
operator is choosing between.

### `fn the_smallest_sheet_that_holds_the_page_wins`

The list is deliberately ordered A3, Letter, A4 so that a `first()`
would answer A3 and a `max` would answer A3 as well — only the stated
rule answers A4.

### `fn a_turned_page_lies_on_the_same_sheet`

The commonest CAD export there is. A fit test that respected page
orientation would step this up to A3 — a whole size wrong, on every
drawing, silently.

### `fn turning_a_page_does_not_make_a_job_mixed`

One sheet serves it and the tray flag has nothing to add, so the extra
sentence would be noise. This is the case the `turned` clause in
[`choose`] exists for, and without it every rotated page in an
otherwise uniform set would trip the mixed sentence.

### `fn a_page_too_big_for_every_sheet_takes_the_biggest_and_says_so`

An A0 site plan on an office printer. The verdict is `TooBig`, not
`Matched`, which is what makes the disclosure name the page size rather
than claim a fit that will not happen.

### `fn a_tenth_of_a_millimetre_of_rounding_is_not_a_size_change`

⚠ The input is chosen to be awkward rather than convenient: 595.4 pt is
*wider than A4* by a tenth of a millimetre, which is what a real
exporter writes and what a zero-tolerance fit test refuses. Both the
slightly-over and slightly-under cases are tried, because a tolerance
applied on one side only is a tolerance that was never tested.

### `fn the_tolerance_does_not_swallow_a_real_oversize`

The other half of the tolerance claim, and the one that matters: a
tolerance wide enough to accept a real oversize would silently crop
every drawing. 10 pt over A4 on the short edge is 3.5 mm — small,
visible, and refused.

The answer is A3, not Letter, and the reason is worth keeping: Letter
is **wider** than this page (612 vs 605) and **shorter** than it
(792 vs 842), so it does not hold it either way round. A fit test that
compared one axis, or compared areas, would have answered Letter — and
the drawing would have come off the machine with 50 mm missing from the
bottom. This test was written expecting Letter and the code was right;
the expectation is recorded here because the same mistake is the
obvious one to make when this function is next changed.

### `fn no_outcome_resolves_to_the_auto_variant_itself`

The property asserted is the one that matters downstream:
`AutoFromPages` must never survive the resolution. A build where it did
would hand the spooler a variant it maps to `DeviceDefault` anyway — so
the job would print on the device's own sheet while the sentence under
the combo claimed a match, which is precisely the silent divergence
this whole feature is a disclosure about.

### `fn a_size_token_reads_back_as_the_number_it_was_written_from`

`size_token` is unlike [`pick_token`] and [`outcome_token`]: it is not
a fixed vocabulary, it is a measurement written for another process to
read. So this parses it the way `tools/ui-verify` parses it — split on
`x`, two `f64`s — rather than asserting a literal, because a test that
asserted `"595.28x841.89"` would pass on a spelling no consumer could
read back, and that is precisely the failure this token replaced.

The tolerance is one hundredth of a point, which is the rounding the
two-decimal format applies on purpose. 0.01 pt is 3.5 micron; the fit
tolerance this number is compared against is [`FIT_TOLERANCE_PT`], two
hundred times larger.

### `fn an_unchosen_auto_reports_absence_rather_than_an_answer`

The distinction is the same one [`outcome_token`] makes: *"this job is
not mixed"* and *"nobody asked"* are different facts, and a driven check
that read the first for the second would be asserting a property of a
decision that never ran. A `false` in that slot would be indistinguishable
from a real measurement.

### `fn the_trace_tokens_are_distinct_and_parseable`

A trace line is `event key=value ...` split on whitespace, so a token
carrying a space would silently truncate the field and every field
after it. Two tokens that collided would be worse: the check would read
a value, believe it, and report a verdict about the wrong state.

### `fn equal_sheets_break_the_tie_on_the_drivers_own_order`

Drivers really do enumerate `"A4"` and a borderless twin. The tie-break
is documented as enumeration order, and it is asserted here so that a
later change from `min_by` to `min_by_key` — which does not promise
which of the equal elements it keeps — cannot silently reverse it.

### `enum AutoPaper`

One value carries the outcome *and* the evidence for it, so the sentence
under the combo can never describe a different decision from the one the
job was planned with — the same pairing argument
`crate::panels::docprops::offered_reading` makes for a label and a policy.

### `fn resolved`

[`AutoPaper::NotChosen`] cannot legitimately be asked — the caller only
resolves when the operator picked auto — but it answers
[`PaperChoice::DeviceDefault`] rather than panicking, for the reason
[`choose`]'s unreachable arm gives: a print dialog that unwraps is a
print dialog that can take the application down mid-job.

### `fn pick_token`

# ⚠ Why this exists rather than `{:?}` on the enum

A machine reads it. This project has already shipped a driven check that
reported the opposite of the truth because it was parsing a `Debug` tuple,
and `PaperChoice::Form(9)` contains a space in no rendering but does carry
punctuation a naive `key=value` split will mangle. These three tokens
contain no whitespace, no punctuation and no numbers, and their spelling is
pinned by a test.

### `fn outcome_token`

The companion to [`pick_token`], and the field that makes a driven check
able to tell "auto matched A4" from "auto was chosen and found nothing".
Both leave `pick=auto`; only this field separates them.

### `fn size_token`

# ⚠ Why this exists rather than `{:?}` on the tuple


Two decisions inside it:

- **Two decimal places, always.** Fixed rather than `{}` so the spelling is
  deterministic — `1190.4` and `1190.40` are the same number and two
  different tokens, and a check that string-compares a before and an after
  would see a change that did not happen. 0.01 pt is 3.5 micron, two orders
  below the 2 pt fit tolerance, so nothing is lost by rounding here.
- **`none` for absent**, not `None`: lower-case, no punctuation, and it
  reads the same as the other absent-value tokens on the same line.

### `fn largest_token`

This is the field that makes O167 checkable from outside the process,
and it is worth saying why the other three are not enough. `pick=auto` says
the operator chose the policy; `auto=matched` says the decision ran;
`paper=Form(8)` says it was turned into a request. **None of them says the
sheet has anything to do with this document.** A build that resolved auto
to the first form in the driver's list would emit all three, correctly, and
be completely wrong — and the operator's words were *"based on the page
sizes in the pdf"*.

With this beside `sheet=`, a driven check can assert the actual invariant:
`matched` means the largest page fits the chosen sheet either way round,
and `toobig` means it does not. That is the rule the module header states,
measured against a real driver's geometry rather than against the fixture
list a unit test supplies.

### `fn mixed_token`

`off` rather than `no` when auto was never chosen, because "this job is not
mixed" and "nobody asked" are different answers and a check that read the
first for the second would be asserting a property of a decision that never
happened. The same three-state care as `outcome_token`, one field along.

### `fn choose`

`page_sizes` are the pages of the job at their **rotated** extents — the
same measurement the preview and the canvas use, so all three agree by
construction. Passing raw `/MediaBox` sizes here would choose portrait
sheets for `/Rotate 90` landscape drawings.

Returns [`AutoPaper::NoBasis`] rather than an `Option`, so that every caller
has to name what it does about the no-basis case rather than reaching for
`unwrap_or_default`.

### `fn auto_paper_line`

Keeping the four outcomes in one `match` is what stops a state from
silently having no sentence. A control with no line under it, where
every other state has one, reads as a control that failed.

[`AutoPaper::NotChosen`] is unreachable from the caller — it only asks
when the operator picked auto — but it answers the no-basis sentence
rather than an empty string, for the same reason.

### `fn auto_paper_is_mixed`

`false` unless auto selection actually ran and found one, so the extra
sentence cannot appear beside a hand-picked sheet — where it would be
true but pointless, the operator having already chosen the sheet
themselves.
