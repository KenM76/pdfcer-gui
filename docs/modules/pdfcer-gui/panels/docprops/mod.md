# `panels::docprops` — the **Document properties** panel: this file's own
title, author, subject and keywords, and the facts pdfcer read about it


> *"the document properties are still always visible in the properties tab.
> it needs to get out of there and be in its own document properties tab."*

Until this commit every line below was the **last section of the selection
inspector** (`crate::panels::properties`), drawn under the armed tool's
settings, the markup restyle controls, the ce-dimension section and the
focused object's read-only facts. It was the only one of those with no
condition attached to it at all, so the *"This document"* heading was on
screen every frame of every session, at the bottom of a panel whose subject
is what you have selected.

**The previous arrangement was DESIGNED, not accidental, and this is a
supersession rather than a bug fix.** `file.properties`' shipped tooltip
commissioned both halves in one sentence — *"The document's own title,
author, subject and keywords, and the properties of whatever is selected on
the page."* — and `RIBBON_IA.md` §5.1 put that one command in File ▸
Document. One command, two subjects, one panel. That reading was coherent
and it is the operator's to overrule; he has.

**And it was the last thing in the inspector that was not the detail of
anything.** `OPERATOR_REQUESTS.md` O123 / A7 is his: *"I never understood
why there is a tool dock when everything can be in object and properties."*
Objects and Properties became one master–detail column so that picking a row
shows that row's detail — `app::modes::defaults`' Edit arm builds exactly
that shape, two adjacent stacks in one column with a draggable split. A
permanent document-metadata block at the foot of the detail pane is the one
block in that column that answers no selection, so it made the master–detail
reading false for every operator who scrolled to the bottom. That is why the
move is worth the churn, and it is the whole of the justification.

O75 is the same complaint arriving a fortnight earlier and being answered
with a **collapse** — *"the Properties section is always showing the This
document properties instead of just the properties of the objects I am
editing."* The section learned to fold itself shut whenever a
selection-scoped section above it had spoken. That machinery is deleted with
this move rather than kept: there is nothing above this section any more, so
`anything_above` would be permanently `false` and the edge-triggered
`set_open`/`store` dance would be a mechanism whose input never changes. The
reasoning it recorded — why neither `default_open` nor `open(Some(_))` can
express a collapse the operator may override — is preserved in
`crate::panels::properties`' header, because it is a finding about egui 0.35
and not about this panel.

## What it is, and what it is not

| | |
|---|---|
| **subject** | the file: `/Info`, its size, its version, its sheet count and size, its encryption, and whether pdfcer had to rebuild its index to open it |
| **command** | `file.document_properties`, on **File ▸ Document**, beside Properties and Fonts — *"inspection, not action"*, per `shell::manifest::ladder` |
| **modes** | all three. Reading a document's title is **reading**, and Read is shown the `file` tab |
| **not here** | anything scoped to a selection. Every one of those sections stayed in `crate::panels::properties`, which is now purely the detail of what is picked |

## R9 — what it shows with no document open

Nothing of its own. [`crate::panels::Panel::show`] answers the empty case
**once**, for every panel, before any body runs: it forgets the panel state
and draws `crate::text::panels::panel_no_document`. So this module is never
called without a document and has no empty state to get wrong — which is
also why there is no "no document" sentence written here to drift from the
other eleven.

An empty *field* is a different matter and is answered below: a PDF with no
`/Info` dictionary at all renders four empty boxes, because absent is a
value and an empty box is how absent is spelled.


It opened, for months, by quoting `Panel::command_id`:

> Only the second half is built here; the first needs a `/Info` accessor
> that `pdfcer-core` does not expose on `Document` at all.

## That last clause was TRUE when written and false when read

`EditSession::info_text` and `info_bytes` both exist, both are `&self`, and
both are documented as *"reflects unsaved edits"*. `InfoField::all()`
exists too, and its own doc comment was written **for this panel**:

> Every editable field, in the order a properties panel should show them.
> Provided so a front end enumerates the real list instead of hard-coding
> one that drifts when a field is added.

So the blocker had already cleared and the prose had not moved. That is the
**sixth** stale claim of this class found in this project, and the previous
five are recorded in `NO_SURFACE.md` §4 and `RIBBON_IA.md` §5.6. The
generalisable part is the one those notes already state: *a measurement in
a document is a measurement with a timestamp*, and a blocker quoted in
prose is a measurement. **Re-run it before believing it**, especially when
it names a crate somebody else is working on in parallel.

## The disclosure this surface owes, and it is not the obvious one

Not "these are the metadata fields". It is `InfoText::exact`:

> `true` when every byte was decoded with certainty. When `false`,
> re-encoding [`InfoText::text`] would **not** reproduce the original
> bytes, so a front end must not write the field back unless the operator
> actually changed it.

A `/Title` written in an encoding pdfcer cannot fully resolve comes back
with U+FFFD where the unmappable bytes were. The operator sees a plausible
string. If the panel then wrote it back — on a focus change, on a save, on
any "keep everything in sync" impulse — it would **replace the document's
own bytes with pdfcer's guess at them**, silently, in a field nobody looks
at twice.

Two things follow, and the second is the one that is easy to skip:

1. **Never write a field the operator did not change.** Discharged by
   construction: this module commits through
   [`crate::panels::forms::rows::commit`], whose second condition is
   exactly *the draft differs from what the document already holds*. It is
   the same function the Forms panel and the canvas form editor use, so
   there is one rule and three callers rather than three rules.
2. **Say so.** Rule 4's half that survives is the inference the operator
   *cannot see* — and a substituted character in a metadata field is
   exactly that. The row carries a sentence when `exact` is false. It is a
   fact about the **document**, not a pdfcer failure, and is worded that way.

## Why an empty field CLEARS rather than sets an empty string

`set_info_field(field, None)` removes the key; `Some("")` would write an
empty string object. They are different documents, and the one an operator
means by deleting the contents of a box is the first: a document with no
title, not a document whose title is nothing.

The row says so, because it is not guessable and because it is the one
action here that *removes* something.

## Item notes

### `mod stamps`

Its own file because it is its own subject: everything else in this panel
describes *a document*, and that module describes *a document that is also
something else*. It draws nothing at all on a file that is not a collection,
which is most of them.

### `const FIELDS`

Derived from the engine's list rather than written as `4`, because
[`InfoDrafts`] holds a fixed-size array of drafts and the two must be the
same length. A fifth field added upstream lengthens the slice, which
changes this, which changes `[String; FIELDS]` — so the drafts follow
automatically instead of the fifth field being dropped off the end.


Until engine `4851316`, `all()` returned `[Self; 4]`, and this comment
argued that the cardinality lived in a **type**, so a fifth field upstream
broke the build at every call site. That was true. The engine removed it on
purpose, and its reasoning is worth having here rather than paraphrased: an
accessor whose entire purpose is that a front end **not** hard-code a list
which drifts had put that list's length into its own signature — so every
caller hard-coded it anyway, in the one place that is hardest to see. It
returns `&'static [Self]` now.

⇒ **What survives is the derivation. What is gone is the break.**
`<[T]>::len` is const-stable, so `FIELDS` is still computed from the
engine's own list at compile time and `[String; FIELDS]` still follows it.
But a fifth field now arrives **silently**: the drafts array grows to
match, the loop over `all()` covers the new position, and
`crate::text::panels::docprops::info_label`'s `_` arm hands it its own PDF
key as a label until someone writes a better one. That is a graceful
degradation and not a defect — the panel keeps working and the field is
editable the day it appears.

**It is silent, though, and that is the part to know before trusting
this constant to raise an alarm: it will not raise one.** Two instruments
do. `tools/gates/check-engine-api-drift.sh` enumerates every public item in
the pinned engine and fails on one this repository names nowhere, which is
exactly what a new variant is. And
`tools/ui-verify/src/checks/properties_metadata.rs` counts the boxes the
panel actually draws against `InfoField::all()` in a running window, which
is the only oracle that can tell a field that was added from a field that
was added *and drawn*.

### `fn fmt`

A document's `/Title` and `/Author` are the operator's own data and
this type's `Debug` reaches the trace, which is written to a file a
harness keeps. `PagesUi` makes the same choice for its selection.

### `fn sync`

Returns the current values as the document holds them, so the caller
compares against **the document** rather than against the draft it just
wrote — two different questions, and only the first is what
[`crate::panels::forms::rows::commit`] wants.

### `fn info_body`

Split out so the scroll area is impossible to forget: content added to this
function is inside it by construction, where content appended to [`body`]
after the `.show(..)` call would silently be outside it again.

### `fn facts`

# Why these five, and not the twenty Acrobat's Description tab shows

Each one below is answerable **from data already loaded**, and each answers
a question an operator actually opens this panel with: *which file is this,
how big is it, what will read it, how many sheets, what size are they, and
is it locked.* Nothing here costs a parse, a walk or an allocation beyond a
string.

What is deliberately absent is anything pdfcer would have to **infer** —
producer and creator strings are not in `InfoField`, so they are neither
read nor written here; permissions are discussed at
[`t::encryption_note`]; and page-level facts beyond the sheet
size belong to the object half of this panel.

# Every value is read through `session.document()`, and one of them lies
if you do not qualify it

`EditSession::document()` is documented as *"the base revision, not the
edited state"*. For the version, the encryption and the page geometry that
is exactly right — none of them is something this build can edit. For
`bytes().len()` it is **the file as it was opened**, which is a different
number from what a save would write the moment anything is edited. So the
size row carries a sentence while `is_modified()` is true, and does not
otherwise.

### `fn file_name`

`Origin` rather than a path check. A document `file.new` created has a
`path` that is a **name** — `text::files::untitled` — and nothing is at it,
which `Origin::Created`'s own doc comment states. Showing it as a file would
tell the operator their work is somewhere it is not, and the one panel
headed *"This document"* is the worst place available to be wrong about
that.

### `fn sheet_size`

**Mixed is the common case for this operator, not an edge case.** A
drawing set is an A1 general arrangement with A3 details behind it, and
reporting page one's size alone would be a true number that reads as a
claim about the whole document.

Compared at **millimetre** resolution rather than exactly, because a CAD
exporter's A3 and a scanner's A3 differ in the sixth decimal of a point and
an operator does not have two sheet sizes because of that.

### `fn recovery_note`

# The last silence of the family, and the quietest one

A PDF carries a cross-reference table: an index saying where every object
lives. When it is wrong — truncated download, a writer that crashed, a disc
error, a tool that appended badly — `pdfcer-core` does not refuse the file.
It **scans the whole thing and rebuilds the index from what it finds**, and
the document then opens and looks completely normal.


It is the same shape as the two silences `pdfcer-core` broke this week — a
search that found nothing, and a redaction that marked nothing, over text
that was never readable. In every case the screen looks right and the
operator has no way to know. Rule 4's less-remembered half: **an inference
the operator cannot see still owes them a report.**

Why it matters more for a CAD drawing than for a letter. A rebuilt index
is a *best reading of damaged bytes*. `last_wins_collisions` counts objects
that were defined more than once, where pdfcer had to pick one — and on a
drawing, a wrong pick is a line in the wrong place, on a page that renders
perfectly. Nobody proofreads a titleblock against a file they believe is
intact.

Off-canvas, in Properties, and **not** a banner over the page. The
document is not in doubt as *drawn*; what is in doubt is how it was
*assembled*. A badge on the page would be a second rendering path for
content that is fine — the bug class decision 059 narrows rule 4 to prevent
— and it would nag on every document that had ever been touched by a bad
writer, which is a great many of them.


The three numbers above describe what recovery *kept*. Until the engine's
`fb6e004` there was no way to ask what it **dropped**, and a recovery that
silently loses a page's content stream is the loader silence decision 145
was written to end, appearing again one layer down. `RecoveryReport`
carries `objects_dropped` now — a list, with an object number and a reason
on every entry — and the engine's own filing for it recorded that the
disclosure *"is not yet visible to anyone"*, which was a statement about
this panel. It is visible here now.

## ⚠ The two reasons are drawn as two sentences and are never summed

This is the whole of why the engine made `DropReason` an enum instead of a
prose string, and collapsing it on the way to the screen would undo that
choice at the last possible moment.

- `Unparseable` is **usually not a loss**. The scan is obliged to try every
  byte sequence that spells `N G obj`, and compressed drawing data contains
  such sequences routinely. Failing to parse them is the correct outcome and
  means nothing went missing.
- `IdMismatch` means something really was defined at that offset and its own
  number disagreed with where it was found, so the definition could not be
  trusted to be what the file claimed.

One combined total reads the same for both and would make the ordinary case
as alarming as the serious one — which is the exact failure mode rule 4's
*"fuzzy, never sneaky"* is trying to avoid in the other direction. A
disclosure that cries wolf on every recovered file is one nobody reads.

## Why the object NUMBERS are printed, and why the list is elided out loud

A count answers neither of the two questions a person holding a damaged file
actually has — *which one went* and *why* — and those are the questions the
engine cited when it chose a list. The numbers are what someone can look up
in another tool. The list stops at [`DROPPED_NUMBERS_SHOWN`] because a badly
mangled file can produce hundreds of false-positive headers and a properties
row is not a report; it stops **naming how many it did not print**, because a
silent truncation reads as *"that was all of them"*.

## R9, as ever

A recovery that dropped nothing draws **nothing** — not a reassuring
"0 objects dropped" line. The great majority of recovered files are in that
state, and a row that is almost always zero teaches the eye to skip the place
where the non-zero will eventually appear.

### `const DROPPED_NUMBERS_SHOWN`

Twelve rather than a round ten, for a reason that is about reading and not
about arithmetic: object numbers in a recovered file cluster, and a run of
consecutive numbers is the single most useful thing this line can show —
it says *one region of the file went*, rather than *scattered noise*. Twelve
is wide enough for a short run to be visibly a run.

⚠ It is a named constant and not a literal because the elision is a policy
the doc above describes; a `take(12)` buried in a call is a policy nobody can
find from the prose that promises it.

### `fn dropped_objects_note`

See [`recovery_note`]'s doc for the whole argument. The mechanics here are
only: partition by [`DropReason`], build the two sentences, and draw nothing
at all when there is nothing to say.

⚠ The `_` arm on the match is not laziness — `DropReason` is
`#[non_exhaustive]`, so a future engine reason MUST have somewhere to land or
this stops compiling on a pin bump. It lands in the `Unparseable` sentence,
which is the conservative of the two: a reason we have never seen is
described as *could not be read*, which is true of every possible arm, rather
than as *disagreed with its own numbering*, which is a specific claim we
would be inventing.

### `fn load_anomalies_note`

# Why the detail is HERE and only a count is in the bar

The same division of labour [`recovery_note`] already has with its own status
line, and for the same measured reason: the bar answers *"is there something
I should know?"* in one elided row (**R128**), and this answers *"what,
exactly?"* with as many rows as the file earned. A file with a doubled
`/PageMode` produces a sentence naming the object, the key, the value pdfcer
used and the value it left — which is four facts, and no width of status bar
holds four facts for an unbounded number of objects.

And *this* panel rather than a dialog. `crate::dialogs::diagnostics` is
the other long-form report in the shell and it is scoped by its own header to
*"this page of this file"* — it describes a **render**, reads
`doc.page_texture`, and draws nothing before the first raster. A load anomaly
is a fact about the **file**, is true before anything has been drawn, and is
true of every page at once. Document properties is where this shell already
keeps facts of that kind, and it is where the recovered-index note that both
status lines point at already lives — so the operator following either
sentence arrives at one place.


This section used to read *"No control, no button, no way to act"*, and
explained at length that choosing the other value of a duplicate key is a
**re-load with different `LoadOptions`** rather than an edit, that the shell
had no such route, and that drawing a disabled control for it would be the
placeholder **R9** forbids. Every clause of that was correct. The last one
stopped being true when `ENGINE_BACKLOG.md` rows 280 and 281 were wired, and
a limitation sentence outliving its limitation is a defect in whoever
believes it.

What is drawn now: the rows, unchanged, followed by **one button** —
[`crate::text::anomalies::reread_first_button`] or its opposite, whichever
names the reading the operator does not currently have. It raises
[`Action::RereadWithDuplicateKeys`], which asks about unsaved edits and then
hands the same bytes back to the engine under the other policy.

# Three properties of that control that are not obvious

1. **It is drawn only when a duplicate key was actually found.** A file whose
   only anomaly was a recovered stream length gets the rows and no button,
   because there is no second reading to offer — the engine measured what the
   file failed to state and there was never a choice. R9: an unavailable
   capability renders **nothing**, not a greyed stub.
2. **It never offers `DuplicateKeyPolicy::Refuse`**, which is that enum's own
   `Default` and is the behaviour that refused the operator's 46 KB drawing
   whole over one repeated `/PageMode`. Two of the three, and this is the
   surface where that rule is enforced.
3. **It stays off-canvas**, which is R8b rule 4 and the reason this control
   lives here rather than as a badge on the page. The document renders
   exactly as it will render when saved; what pdfcer had to decide is
   reported in a panel, never drawn into the view.

The rows still carry their old load: without both values on screen the
button has nothing to mean, so the disclosure is what makes the control
legible rather than the other way round.

Drawn inside [`body`]'s existing scroll area by construction — see
[`info_body`]'s doc — which is what makes an unbounded row count safe here
and unsafe in the bar.

⚠ No epoch, no cache, no snapshot: read live from the open document, exactly
as [`recovery_note`] above reads `Document::recovery()`. See
[`crate::app::status::anomalies`]' header for why a load anomaly must
**not** retire on the next edit.

### `fn offered_reading`

# Why this is a function rather than four lines inside the button

Because a unit test can drive it and cannot drive a `Ui`. Two of the three
properties that matter here are decidable without a window — *the offer is
always the reading you do not have*, and *`Refuse` is never offered* — and
this project's standing lesson is that a verb's unit test cannot see the
chain in front of it. So: the decision is tested here, and the chain from
this button to the loader is asserted by driving the binary. Both, because
neither alone is evidence.

Returns the policy to request and the label that names it, as one value, so
a label can never drift from the policy it describes — the failure mode
being a button that reads *"use the first value"* and asks for the last.

⚠ The `_` arm sends anything this build does not recognise back to pdfcer's
documented reading. [`DuplicateKeyPolicy`] is `#[non_exhaustive]`, so a
newer engine can put a policy here that this file has never heard of; the
conservative direction is the one whose label is certainly true, and
[`DuplicateKeyPolicy::Refuse`] must never be the answer — it is the
behaviour that refused the operator's 46 KB drawing whole.

### `fn reread_control`

Drawn as the last thing inside [`load_anomalies_note`]'s block, under the
rows that say what pdfcer chose. That position is the argument: the rows
name both values, and this offers the other one. Read in the other order it
would be a button with nothing to mean.

# `duplicates == 0` renders NOTHING — R9, and it is not a formality

A file whose only anomaly was a recovered stream length or a missing
`endobj` gets the rows and no button, because there was no second reading to
choose between: the engine measured what the file failed to state. A greyed
button there would be a placeholder describing a capability that does not
apply to this file, which is the exact thing **R9** forbids — greying is for
*temporarily* unavailable, and "this file has no duplicate keys" is not a
condition the operator can clear.

# One button, whose LABEL names the other reading

Not a pair of radio buttons and not a checkbox. After a re-read the opposite
choice is exactly as available as this one was, so the honest control is a
single button that always offers the reading currently *not* in effect —
read off [`crate::app::state::OpenDoc::load_options`], which is carried for
this purpose and for the password retry.

# ⚠ [`pdfcer_core::parser::DuplicateKeyPolicy::Refuse`] is never offered

It is that enum's own `Default` and it is the behaviour that refused the
operator's 46 KB drawing whole over one repeated `/PageMode` — the exact
failure `Pass 283.0` exists to end. Two of the three policies reach a
loader from this shell, and **this function is the only place that decides
which**; `Action::RereadWithDuplicateKeys` is transport and cannot enforce
it.

⚠ The `_` arm below is not a catch-all for convenience. `DuplicateKeyPolicy`
is `#[non_exhaustive]`, so a policy added to `pdfcer-core` compiles here
without a word from the compiler; the arm sends such a document back to
pdfcer's own documented reading, which is the conservative direction and the
one whose label is certainly true.

### `fn the_control_fixture_for_the_dropped_disclosure_really_recovers_and_drops_nothing`

⚠ Two assertions and not one, because the control half of the driven
check needs both halves to be true and they fail independently. If the
file stopped being recovered -- someone adds an xref, or a future engine
revision learns to open it without rebuilding -- the recovery note would
correctly draw nothing, the driven check would demand
`properties.recovery` and get nothing, and it would report an
application defect that does not exist. If the file started dropping
something -- a careless edit to the sentence drawn on the page -- the
dropped block would correctly draw and the check would report the
opposite defect, equally wrongly.

This lives in the cheap suite on purpose. A fixture that has stopped
being a control is a fact about the corpus, and finding it out costs a
second here and a ninety-minute driven sweep there.

### `fn the_recovery_fixture_drops_the_two_objects_it_was_built_to_drop`

⚠ This is the test that keeps the driven check honest. The driven check
asserts a sentence is on screen; if a future engine revision stopped
finding object 9 as a candidate, or started keeping object 8, the
fixture would quietly produce an empty `objects_dropped`, the panel
would correctly draw nothing, and the driven check would then be
asserting the absence of something that was never going to be there —
green forever, measuring nothing. Asserting the INPUT here is what makes
the output assertion downstream mean anything.

Both numbers are checked, not the count. A count of two is satisfied
by finding object 8 twice, and *which object went* is the question the
engine cited when it chose to carry a list rather than a tally.

### `fn both_dropped_objects_reach_the_lines_the_operator_reads`

The panel's drawing cannot be called without an egui context, so what is
asserted is the pair of strings it hands to `ui.label` — built here from
the same report, by the same two functions, in the same order.

The summary is asserted to be non-empty and the numbers line to carry
both numbers. It deliberately does not pin the wording: prose gets
reworded, and a test that fails on a comma teaches people to edit tests.
What it pins is that neither object vanished on the way, which is the
whole subject of the disclosure.

### `fn the_offer_is_always_the_reading_not_in_effect`

Both directions, because a control that only ever offers one of them is
a one-way door: an operator who re-reads a 200-page drawing under the
first values and finds it worse has to close the tab and reopen the file
to get back, and will reasonably conclude pdfcer changed something.

Asserted on the **policy**, not on the label — the label is prose and
prose gets reworded. The pairing between them is what the shared return
type makes unbreakable; see [`offered_reading`]'s doc.

### `fn the_panel_never_offers_the_policy_that_refuses_the_file`

⚠ The variant this test exists for is [`DuplicateKeyPolicy`]'s own
`Default`, so it is the value anything careless produces — and sent to a
loader it is *the behaviour that refused the operator's 46 KB drawing
whole over one repeated `/PageMode`*, which is the failure the whole
re-read feature exists to end. Offering it from the panel that reports
that failure would be a control that recreates it.

It is checked across every policy this build can name **including
`Refuse` itself**, which is the interesting input: a document somehow
carrying it must be offered a way out, not a way further in.

### `fn the_two_readings_are_labelled_differently`

A control whose label does not change when the reading does is a button
that appears to do nothing the second time it is pressed — the operator
presses it, the document comes back, the button still says *use the
first value*, and the only honest conclusion available to them is that
it failed.

### `fn the_draft_array_tracks_the_engines_field_list`

It asserted `FIELDS == InfoField::all().len()` and
`drafts.len() == InfoField::all().len()`, with a doc comment explaining
*"the direction it fails in"*. It has no such direction. `FIELDS` **is**
`InfoField::all().len()`, and `drafts` is `[String; FIELDS]` — all
three values are one value wearing three names, so both assertions were
`x == x` and the test could not go red for any engine change whatsoever.
It had been that way since it was written; the slice bump is only what
made someone read it.

⇒ **A check that cannot fail is not evidence, and it is worse than no
check, because its name occupies the space where a real one would go.**

# What replaced it

The property this panel actually depends on, which is a claim about the
**engine** and can therefore be wrong: that `/Title`, `/Author`,
`/Subject` and `/Keywords` are all still offered. Every region name in
[`REGION_FIELD_PREFIX`], every label in
`crate::text::panels::docprops::info_label`, and the driven check that
reads index 0 as the title were written against those four. A field
ADDED upstream degrades gracefully and is caught by
`check-engine-api-drift`; a field REMOVED upstream silently renumbers
every position this panel addresses by index, and nothing else in this
repository would notice.

### `fn only_a_blank_draft_removes_the_key`

The rule is stated here as data rather than exercised through a frame,
because the branch it protects is one line and the consequence is a
removed dictionary key. A draft of `"  "` clears; a draft of
`" Site Plan "` does not, and keeps its spaces.

### `fn tabbing_through_a_field_writes_nothing`

Asserted by calling it: tabbing through an untouched field writes
nothing, and a changed field with focus still held writes nothing
either. If this module ever grows its own predicate, one of these two
is what will fail.
