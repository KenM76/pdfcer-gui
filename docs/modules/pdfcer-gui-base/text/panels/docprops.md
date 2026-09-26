# `text::panels::docprops` — the Document properties panel's copy

Every string the **Document properties** panel says about the file itself:
the four `/Info` fields' labels, the seven read-only facts, and the two
disclosures a document can owe an operator — a value pdfcer could not decode
exactly, and an index pdfcer had to rebuild in order to open the file at all.

## Why this is its own module, and it is two reasons rather than one

**1. The panel is its own panel** — the operator, 2026-09-05: *"the document
properties are still always visible in the properties tab. it needs to get
out of there and be in its own document properties tab."* These strings moved
out of [`super::properties`] with the section they belong to, in the same
commit, because copy that lives in the catalog of a surface it is no longer
drawn on is copy nobody finds when they come to change it. Every one of them
was reached from exactly one file before the move and from exactly one file
after it.

**2. R2, measured rather than anticipated.** `text/panels/properties.rs`
stood at **1,469 lines against the 1,500-line ceiling** on the day of the
move — its own `textobject` module records having been split off at 1,446 for
the same reason. So the move is also the split that gate was going to force
within a couple of sentences, and it is a split along a subject boundary
rather than an arithmetic one: what remains in `properties` describes **what
is selected**; what is here describes **the file**.

## The names lost their `properties_` prefix, deliberately


The three `recovered_*` functions kept their names: they were never
prefixed, they name the *event* rather than the surface, and renaming them
would have been churn with no reader served.

## What is NOT here

The empty state. A panel with no document open never draws its own body —
[`crate::panels::Panel::show`] answers that case once for all twelve panels
with [`super::panel_no_document`] — so there is no "open a document first"
sentence in this file to drift from the other eleven.

## Item notes

### `fn the_two_drop_reasons_are_never_collapsed`

Driven from all four corners rather than from the one case a fixture
happens to produce. The interesting corners are the two SINGLE-kind
ones: a recovery whose drops are all false positives must not produce
the sentence about untrustworthy numbering, and a recovery whose drops
are all mismatches must not produce the reassuring one. Either would be
the collapse the engine's enum exists to prevent, arriving at the last
possible moment.

### `fn an_elided_list_of_dropped_objects_counts_what_it_left_out`

⚠ Both sides of the boundary, because an elision tested only above its
threshold cannot tell a correct rule from one that always elides, and
one tested only below it cannot tell a correct rule from one that never
does.

The exactly-at-the-limit case is here on purpose: an off-by-one there
produces "and 0 more", which is the silent-truncation failure wearing
the opposite coat — a remainder announced that does not exist.

### `fn the_heading_is_not_the_tabs_name_again`

The panel's tab is called *Document properties* — it takes its name from
`file.document_properties`' label, which is how every dock tab in this
build is named. A heading reading the same words immediately under it is
a line of an inspector that tells the operator nothing they did not
learn by clicking, and it is the obvious thing for a later edit to
"tidy" the heading into.

### `fn the_disclosures_describe_the_file`

Both are drawn at ordinary weight beside facts, and the wording is what
carries the distinction — a sentence that opened "pdfcer could not…"
would read as a defect report about the program in a panel whose whole
subject is the operator's file.

### `fn heading`

**"This document" rather than "Document properties"**, and the reason
survived the move to a panel that is *called* Document properties. The tab
says what the surface is; the heading says what the words under it are
about. Repeating the tab's own label immediately below it is a line of
screen that tells the operator nothing they did not learn by clicking.

It also reads as the answer to a question — *which document?* — which is the
live one with several documents open.

### `fn note`

The second sentence is the one that could not be guessed. Clearing a box
**removes the key from the file** rather than storing an empty string —
`set_info_field(field, None)` against `Some("")` — and they are different
documents. It is also the only action in this panel that removes anything,
which is why it is stated at the top rather than left to be discovered.

### `fn info_label`

Takes the engine's `InfoField` rather than a string, so the panel
enumerates `InfoField::all()` and this maps the result — which is the
discipline that enum's own doc comment asks for: *"so a front end
enumerates the real list instead of hard-coding one that drifts when a
field is added."*

The words are the PDF spec's own field names in ordinary English. "Subject"
and "Keywords" are not obvious, and neither is improved by inventing
something friendlier: an operator who has met these fields in any other PDF
tool has met them under these names, and a novel word would be a novel
thing to learn for no gain.

# `InfoField` is `#[non_exhaustive]`, so the compiler CANNOT catch a new
field here

This function was first written as a `const fn` with four arms and no
wildcard, on the assumption that a fifth variant would break the build. It
would not: `#[non_exhaustive]` forces a downstream crate to write `_`, and
with a `_` arm the match compiles for ever no matter what the engine adds.
The protection people expect from an exhaustive match is **not available
across a crate boundary** when the enum is marked that way, and assuming it
is available is how a new field ends up silently unlabelled.

So the safety is built another way instead:

**The fallback is the field's own PDF key**, taken from `InfoField::key()` —
the engine's answer, not a guess. A field added upstream appears in the
panel labelled `Producer` or `Creator` rather than disappearing or reading
"Unknown". Imperfect English, correct, and reachable, which beats all three
alternatives.



> **A test asserts none of the four known fields reaches the fallback.**
> That is the alarm the compiler cannot raise: if the mapping is ever broken
> the four named fields start rendering as their raw keys, and the test says
> which.

No such test was ever written, and one was attempted while moving this
function into its own module. It fails on the first field it looks at, for a
reason that is not a defect: **`InfoField::Title`'s PDF key IS `Title`.** So
is `Author`'s, `Subject`'s and `Keywords`'s. The mapping and the fallback
return byte-identical strings for all four, which means a deleted arm
changes nothing an operator or a test could see.

⇒ The arms are not a safety net for these four; they are a **place to put a
better word** when the engine adds a field whose key is not already English
(`Producer`, `CreationDate`, `Trapped`). That is a real job and the
wildcard is a real floor under it. What is gone is the false comfort of a
test nobody could have written — and the shape is the one this project keeps
finding: *a safeguard described in prose is not a safeguard*, and the only
way to tell is to try to write it.

What IS asserted, in [`crate::panels::docprops`]'s own tests: every field
`InfoField::all()` returns has a non-empty label and no two share one. That
catches the failure that can actually reach the operator — two boxes they
cannot tell apart, or an unlabelled one.

### `fn info_not_exact`

Drawn under a field whose `InfoText::exact` is `false`. That flag means
re-encoding the displayed text would **not** reproduce the file's own
bytes, so what is on screen is a rendering with substitutions in it, not a
copy.

This is rule 4's surviving half in its purest form: an inference the
operator **cannot see**. A replacement character in a metadata field looks
like a character rather than like a gap, and without this sentence the
operator's only clue that pdfcer is guessing would be a glyph they might
read as the document's own.

Worded as a fact about the **document**, not as a pdfcer failure — the file
really does carry bytes in an encoding it does not declare well enough to
resolve — and it says what to do about it, which is the part that makes it
actionable rather than alarming: leaving it alone is safe.

### `fn file_unsaved`

`OpenDoc::has_file` is `false` for a document `file.new` created, and its
`path` in that state is a *name*, not a location. Showing the name as
though it were a file would tell the operator their work is somewhere it is
not.

### `fn size_is_base`

`Document::bytes()` is documented as *"the base revision, not the edited
state"*, so with unsaved edits this number is the file on disk and not what
a save would produce. Shown only while edits are pending, because on an
unedited document the two are the same and the sentence would be noise that
trains the operator to skip it.

"Size on disk" rather than "File size" for the same reason: the label
itself carries most of the distinction, and the sentence carries the rest
when it matters.

### `fn page_size`

Millimetres in the sentence, because a drafter knows an A3 by `420 × 297`
and nobody's intuition is in 72nds of an inch. The page tile's tooltip made
the same choice for the same reason — and for a while the two of them
disagreed, because each converted with its own constant and rounded with its
own rule. A sheet of exactly 210.5 mm read `210` on one and `211` on the
other.

Both now take points and round through `units::whole_mm_from_points`,
half away from zero. Printed with `{}`; `{:.0}` rounds half to EVEN and was
the source of the disagreement.

### `fn page_size_mixed`

**The common case for this operator**, not an edge case: a drawing set is
an A1 general arrangement with A3 details behind it. Reporting page one's
size alone would be a true number that reads as a claim about the document,
so the mixed case says so and gives the first sheet's size as an example
rather than as the answer.

### `fn not_encrypted`

Stated rather than left blank. This panel's posture is that its silences
must be as legible as its numbers, and an absent encryption row is
indistinguishable from a panel that does not check — which on this
particular question is exactly the wrong impression to leave.

### `fn encryption_note`

The Signatures panel's discipline applied here: *say what you cannot tell
you, first*. A row reading "Encrypted" invites the operator to conclude
something about what the document permits, and pdfcer reports nothing about
permissions in this panel.

The reason is in `pdfcer-core`'s own `DocumentEncryption::perms` doc: `/Perms`
is *"the only integrity check in PDF encryption"*, it is a `should` rather
than a `shall`, and it is `NotApplicable` for every `/R` ≤ 4 document —
which is *"the ordinary answer, not a failed check, and a front end must
not render it as one."* Reporting permissions properly means reporting that
distinction properly, and this build does not.

### `fn recovered_heading`

Plain, and about the FILE rather than about pdfcer. "pdfcer had to repair
this file" would read as pdfcer struggling; the file is the thing that is
damaged, and the operator's next question is about the file.

### `fn recovered_detail`

Three numbers, and only the middle one is a warning. Objects recovered
says how big the job was; **objects defined more than once** is the one that
can put a line in the wrong place, because pdfcer had to choose; and repaired
says how much else needed inference. Naming them separately lets an operator
tell "large but clean" from "small and ambiguous", which a single "recovered
N objects" cannot.

### `fn recovered_tooltip`

States the consequence in the operator's terms and stops. It does not
tell them to do anything, because there is nothing reliable to tell them:
the file may be perfectly fine, and the only real remedy is a good copy from
whoever produced it. Inventing an action would be worse than naming the
uncertainty.

### `fn dropped_summary`

# WHY THE TWO KINDS ARE NEVER ADDED TOGETHER

The engine went out of its way to make this an enum rather than a sentence,
and its reasoning is the reason this function takes two arguments instead of
one total. `DropReason::Unparseable` is **usually not a loss at all**:
compressed picture and drawing data routinely contains bytes that happen to
spell `N G obj`, the scan is obliged to try them, and failing to parse them
is the correct and uninteresting outcome. `DropReason::IdMismatch` is a
different animal -- something really was defined there, and its own number
disagreed with the offset it was found at, so pdfcer could not trust the
definition to be what the file claimed.

One combined number reads identically for both and would make the routine
case as alarming as the serious one. That is the specific failure the engine
named when it chose the enum, and collapsing it here would undo the choice
on the way to the screen.

## Why the wording leads with the ordinary explanation

R8b rule 4 requires the disclosure; nothing requires it to be frightening.
The first clause of the unreadable sentence says what the common cause is,
so an operator who reads only the first line gets the true impression rather
than an alarming one. The second clause is the part that matters when
something IS missing, and it is stated plainly rather than hedged.

### `fn dropped_numbers`

# Why the numbers are printed at all

The engine's own argument for carrying a list rather than a count: the
complaint this came from was never that a tally was wrong, it was that a
human holding a file with a missing page could not find out WHICH object
went. A count answers neither question. The numbers are what a person with
the file in another tool can actually look up.

# ⚠ Why it elides, and why the remainder is COUNTED rather than dropped

A badly damaged file can produce hundreds of false-positive headers, and a
panel row is not a report. So the list stops -- but it stops **out loud**,
naming how many it did not print. A silent truncation reads as "that was all
of them", which is the one impression a disclosure must never leave.

`limit` is passed rather than hard-coded so the caller owns the elision
policy and the test can drive both sides of it without a fixture the size of
the threshold.

### `fn dropped_tooltip`

Ends without an instruction, for the same reason [`recovered_tooltip`]
does: the file may be entirely fine, and the only real remedy is a good copy
from whoever produced it. What it adds over that tooltip is the one thing an
operator can actually check -- whether anything is visibly absent -- because
that is the symptom a dropped content stream produces and it is checkable
without any tool but their own eyes.
