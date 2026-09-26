# `dialogs::new_document` — the other half of New

`RIBBON_IA.md` §5.1's File band specifies one row as
**`New (blank / from template)`**, and only the blank half shipped:
`file.new` makes an A4 page with no question asked, which is what Acrobat
and Inkscape both do. This is the other half — `file.new_from_template`,
the command that asks what kind — and Inkscape's split is the shape being
followed: `Ctrl+N` makes a document, `Ctrl+Alt+N` chooses what kind.


`crate::app::blank`'s §3a is the record and is worth reading before
touching this file. In short: nothing in `pdfcer-core` wrote a `/MediaBox`,
so the only implementation available to a shell was **one checked-in
template asset per size** — ten with landscape, more with ANSI, *and a
custom size impossible at any count*. That half-capability was refused
rather than built, because a surface that answers twelve of thirteen cases
forecloses the fix for the thirteenth.

`EditSession::set_media_box` and `pdfcer_core::paper` shipped, so the whole
thing is one asset, one dialog, every size, both orientations, custom
included.

## What it does NOT do, and why each is a decision

**It does not resize an existing page.** `set_media_boxes` supports that
and it is a genuinely different capability with genuinely different
questions — does content move, does `/CropBox` follow, is shrinking below
the content a refusal. None of those arise here because `file.new`'s page
is empty by construction, which is exactly why the request that unblocked
this deliberately asked only for the narrow answer. A page-resize surface
belongs in Document ▸ Properties and is not this module's.

**It does not remember the last size chosen.** The obvious convenience, and
it is left out on the evidence rather than on principle: nobody drafts a
sheet in pdfcer, so the *second* use of this dialog is rare enough that a
remembered value would more often be stale than helpful — and a New command
that silently produced A1 because of something the operator did last
Tuesday is worse than one that always starts where its sibling does. If the
operator reports otherwise, `crate::app::prefs` is where it would go, beside
the opening-view preferences, and this paragraph is the argument to
overturn.

**It offers no templates**, despite the command's name. `RIBBON_IA.md`'s
parenthetical for this row is `(page size)`, which is the IA's own
annotation of scope — the same shape as `Export image… (PNG/JPEG/TIFF, DPI
picker)`. So the label follows the IA, and
[`crate::text::new_document::intro`] states in the window's first line what
it actually offers, because that is the cheapest correction available to a
session that may propose IA amendments and may not make them.

## Application-scoped, like About and Settings

It draws with nothing open and must: an operator with an empty shell is the
one most likely to want it, which is the same argument `file.new` and
`file.open` are registered with no `enabled_when` for. So it is drawn
**before** `crate::dialogs::DialogsState::show`'s document guard, beside
About, and closing a document does not close it.

## Rule 4: nothing here is drawn on a page

There is no page yet. This window states what it will make and makes it;
the sheet it reports is the sheet that lands, and there is no inference to
disclose because there is no inference — the operator typed or picked every
number in it. The one thing pdfcer decides on their behalf is the **refusal**
of an out-of-range custom size, and that is stated in words with its limits
named (`crate::text::new_document::custom_refused`).

## Item notes

### `const REGION_SIZE_ITEM_PREFIX`

# Why the entries are published

The same argument the print dialog's paper list makes: an egui combo popup
is an `Area` laid out at paint time, so nothing outside the process can
compute where an entry is — and a check that can open a list but not choose
from it can assert only that a control exists. "The control exists" is
exactly what was true of the print dialog's tray checkbox for four months
while it did nothing.

Here the property worth asserting is that **picking a size produces a page
of that size**, end to end through `set_media_box`, a full rewrite and a
re-parse. That needs a click on a specific entry.

### `const REGION_PORTRAIT`

Published because the transposition is the most likely defect in this
window and the one a unit test cannot see end to end: `sheet_pt` is pinned
in tests, and what is *not* pinned there is that the radio the operator
clicks is the one that reaches it.

### `const MIN_CUSTOM_MM`

ISO 32000-1 Annex C.2 advises a minimum of **3 units** (≈ 1.06 mm), so 1 mm
would be marginally under it and 2 mm is comfortably over. Rounded up
rather than to the letter of the advice because a sub-millimetre page is
not a thing anybody wants and a bound an operator can remember is worth
more than a bound derived to two decimal places.

### `const MAX_CUSTOM_MM`

**5,080 mm = 200 inches = 14,400 default user space units**, which is
ISO 32000-1 Annex C.2's advised maximum. See
[`crate::text::new_document::custom_refused`] for the full sourcing,
including the fact that ISO 32000-2 drops the number entirely and this is
therefore 1.7-era portability advice pdfcer is choosing to honour.

### `enum Choice`

A separate type from `pdfcer_core::paper::PaperSize` rather than
`Option<PaperSize>`, because "custom" is a *state of the dialog* — it opens
two fields and changes what the summary line reads — and not a missing
size. `Option` would have made the two fields' relevance depend on a `None`
that also means "nothing chosen yet", which is a state this dialog never
has.

### `fn sheet_pt`

# One function, read by three callers, and that is the point

The summary line, the validity check and the action all ask this. Three
separate computations of "what did they pick" is how a window comes to
promise 841 × 1189 and produce 1189 × 841 — and the transposition is
exactly the kind of arithmetic that is easy to write twice and hard to
notice once.

A standard size comes from `PaperSize::rect_with`, which is the
engine's own table applying its own orientation rule. Nothing here
re-derives a sheet size: `594.0 * 72/25.4` is a number the engine
computes and a hand-rounded `1683.78` is a number that is *not* A1 and
will not compare equal to a CAD exporter's.

### `fn is_valid`

Only a custom size can fail: every entry in `PaperSize::ALL` is a real
sheet in range by construction. The check is on the **millimetre
fields** rather than on the computed points, so the message can name the
numbers the operator typed.

### `fn landscape_transposes_a_standard_size_and_a_custom_one`

The single most likely defect in this window: a standard size that
turns and a custom size that does not, or the reverse. Both go through
[`NewDocumentDialog::sheet_pt`] precisely so they cannot diverge, and
this is what holds that.

### `fn a_standard_sheet_is_the_engines_own_number`

Not a tautology test. The failure it exists for is a shell that
hand-rounds A1 to `1683.78 × 2383.94` — numbers that look right, are
wrong in the fourth significant figure, and will not compare equal to
the `/MediaBox` a CAD exporter writes. The engine converts from the
defining millimetres for exactly that reason and this pins that the
dialog does not re-derive it.

### `fn an_out_of_range_custom_size_is_refused_in_both_directions`

Both directions, because a check written as `> 0` would let a 12-metre
sheet through and one written as `< MAX` would let a zero through, and
each is a single missing clause.

### `fn it_opens_on_the_size_the_plain_new_command_makes`

The two commands sit beside each other in one ribbon group, and the
difference between them must be "one asks" and nothing else. A default
that drifted to A3 here would make the sibling controls quietly
disagree about what a new document is.

### `fn open`

**A4 portrait and not something cleverer.** It is what `file.new`
makes, and the two commands sit next to each other in the same ribbon
group: an operator who opens this window to check what it offers should
see the state the plain command would have produced, so the difference
between the two controls is *"one asks"* and nothing else.

`crate::app::blank`'s §3 is where A4 is argued — two of the three
reference applications ship it, and the operator's own corpus is
A-series.
