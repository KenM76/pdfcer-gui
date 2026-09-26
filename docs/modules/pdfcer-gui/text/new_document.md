# `text::new_document` — the copy of the sized-New dialog

Every operator-facing string `crate::dialogs::new_document` draws. One
function per string, per `crate::text`'s contract: the gate
`tools/gates/check-ui-strings.sh` fails the build for a literal that
reaches a widget from anywhere else.

## What this surface is a size chooser FOR, which decides the wording

Not for drafting. Nobody drafts a sheet in pdfcer — documents arrive from
SolidWorks — and `crate::app::blank`'s header says so plainly while
arguing the A4 default. What this is for is the case the header also
names: *"A4 is very plausibly not the right size for this operator's next
new sheet"*, whose own drawings are **A3 and A1**.

So the copy is short and assumes competence. An operator opening this
knows what A1 is; what they need is to find it quickly, see it confirmed,
and get out. There is no explanation of what a page size is, no advice
about which to pick, and no reassurance.

## Millimetres, and why there is no unit toggle

The size list is ISO-first because the operator's corpus is, and every
entry states its size in **millimetres**. There is no inches toggle, and
that is a decision rather than an omission:

- The A series is *defined* in millimetres. A1 in inches is 23.39 × 33.11,
  which is a number nobody recognises.
- The US and ANSI entries are in the same list and would want inches, so a
  toggle would be right for four entries of sixteen and wrong for twelve.
- This shell already has a units answer elsewhere and it is **not global**:
  the measure tools express a length in whatever the *dimension group's*
  own `NumberFormat` says, which is a per-document drafting convention, not
  an application preference. A unit switch here would be a second,
  unrelated units concept for a dialog that is open for four seconds.

The custom fields are therefore millimetres, stated in the label rather
than in a suffix an operator can miss, and the resulting sheet is echoed in
**both** units by [`sheet_summary`] so a Letter-minded reader is not left
converting.

## Item notes

### `fn imperial`

Operator request, 2026-08-20: *"please add imperial sizes too — we use
imperial units — then select B size."*

The sizes were already there. `PaperSize::ALL` has carried Letter, Legal,
Tabloid, Executive and ANSI A–E since before this dialog was written, and
the combo lists every one of them. What was missing was the **unit**: every
entry read in millimetres, so ANSI B — an 11 × 17 inch sheet, defined in
inches, called "B" by everybody who uses one — appeared as
*"ANSI B — 279 × 432 mm"*.

Nobody looks for B size under 279 × 432. A list that contains the thing an
operator wants and describes it in units they do not work in is a list that
does not contain it, and the report *"please add imperial sizes"* is exactly
what that looks like from the outside. **It was a labelling defect wearing a
missing-feature costume.**

So each size is shown in the unit it is **defined** in — ISO A-series in
millimetres, US and ANSI in inches — rather than in one unit chosen for the
whole list. That is not a compromise between two preferences; it is the only
labelling that is *true*: ANSI B is exactly 11 × 17 in and approximately
279 × 432 mm, and a list that rounds the exact one into the approximate one
has thrown away the number the sheet actually has.

Keyed on the **name** rather than on the enum, so a size the engine adds
appears without a shell change — the same reason `size_name`'s wildcard
exists. A new ISO or JIS size falls through to millimetres, which is the
right default for anything not in the US series.

### `fn inches`

Not a decimal. `8.5 in` is what a programmer writes and `8 1/2"` is what
is on every title block in the operator's own corpus; the sizes that matter
here are 8½ × 11 and 11 × 17, and a list reading *"8.5 × 11 in"* is a list
that has been translated rather than written.

### `fn a_us_sheet_reads_in_inches_and_an_iso_sheet_in_millimetres`

The operator's report of 2026-08-20 was *"please add imperial sizes
too"*, and every size he wanted was already in the list — labelled in
millimetres. `ANSI B — 279 × 432 mm` is the sheet he calls B, described
in units his office does not use, which is indistinguishable from its
not being there.

So this asserts the labelling directly, on the two sizes that matter
most to him and on one from each family, because the failure it guards
is a *silent* one: a size added to the US series and not added to
`imperial` would simply appear in millimetres and nobody would file a
bug against a list that has the entry.

### `fn a_half_inch_is_written_as_a_half`

`8 1/2`, not `8 8/16` and not `8.5`. The sizes an operator reads most
are the two half-inch ones, and a list that says `8.50 × 11.00 in` has
been translated rather than written.

### `fn a_named_size_reads_back_as_its_own_millimetres`

Not a tautology: [`size_entry`] and [`sheet_summary`] each convert
points to millimetres, and `pdfcer_core::paper` builds its points *from*
millimetres by the inverse constant. A rounding or a transposed
constant here would show A1 as "593 × 840" beside a file that really is
A1 — a discrepancy an operator would read as pdfcer getting the standard
wrong.

### `fn the_custom_refusal_states_the_limits`

An operator told only that their number is wrong has to guess. The
number is what turns a refusal into an instruction, and it is the
single thing most likely to be dropped by a later rewording.

### `fn no_size_in_the_list_reads_like_an_identifier`

[`size_name`]'s wildcard exists so a size added to `PaperSize` after
this build still appears in the list, under `id().to_uppercase()`. This
pins that the wildcard is the **exception**.

# Why it cannot be written as "the name differs from the fallback"

Because for seven of the sixteen it does not, and correctly: the
uppercased id of `A0` is `"A0"`, which is also its right name. That
version of this test was written first and failed on its first run,
reporting `A0` as a defect. Which was useful — it is the same shape as
a test asserting a refusal that outlives its premise, caught early.

What distinguishes a fallback that is *wrong* is that an identifier is
hyphenated where a name is spaced: `ansi-d` becomes `"ANSI-D"` and
should read `"ANSI D"`. So the property asserted is that **no name
contains a hyphen** — which holds for every size today, fails for any
multi-word size the engine adds, and says something true rather than
something merely checkable.
