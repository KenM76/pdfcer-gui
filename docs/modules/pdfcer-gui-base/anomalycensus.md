# `anomalycensus` — one reading of `load_anomalies()`, for two
surfaces

The derivation behind the status bar's *"this file contradicted itself"* line
and the Document-properties list underneath it. It counts, orders and words
nothing itself — [`crate::text::anomalies`] holds the words — but it decides
**which anomalies are reported, in what order, and how they are grouped**,
and it decides that **once**.

## Why the derivation is split out rather than written at each site

This is the same seam, for the same reason, as [`super::notes::findings`],
whose header states it plainly:

> Split out … so that the status bar's one line and the Render-diagnostics
> dialog's list are the *same ten decisions* … Two tables would agree on the
> day they were written and disagree the first time an eleventh counter was
> added to one of them, and the symptom would be two surfaces describing one
> raster differently, which is the worst available outcome for a *diagnostic*.


Every word of that transfers. `LoadAnomaly` is `#[non_exhaustive]`; a fifth
variant is not a hypothetical, it is the thing
`tools/gates/check-engine-api-drift` exists to catch. One match statement is
one place to teach.

It lives under `app::status` rather than beside the document state because
[`super::notes`] set that precedent and `crate::dialogs::diagnostics` already
reaches across for it. A second convention for the same shape would cost a
reader more than the slight misfiling does.

## The lifetime question, answered honestly

Two of this module's neighbours in [`super::disclosure`] — the fill and edit
disclosures — are keyed on [`crate::app::state::OpenDoc::edit_epoch`] and
**retire on the next edit**. That is right for them: they describe *something
the operator just did*, and a later edit moves the document past it.

A load anomaly is not that. It is true of the **file as it was opened**, and
it stays true for as long as that document is open — through every edit,
every undo, and every save. Keying it on `edit_epoch` would delete a
permanent fact the first time the operator nudged a line, which is precisely
the failure decision 145 was written against: the operator would be told once
and then quietly un-told.

So there is **no epoch key here, and no cached state anywhere**. The census
is recomputed from `doc.session.document().load_anomalies()` on the frame it
is drawn, exactly as [`super::disclosure::recovered_disclosure`] recomputes
from `Document::recovery()`. The slice lives on the `Document`, is populated
at load, and is replaced wholesale when a different document is opened into
the tab — so there is nothing to clear, nothing to invalidate, and no way for
one file's anomalies to be shown against another's. State that must be
cleared is state that will one day be shown against the wrong document; this
has none.

⚠ The list is short by construction — one entry per contradiction in the file
— so recomputing per frame is a walk over a handful of items, not a parse.
The one file that motivated the feature has two.

## `recovery()` and `load_anomalies()` are different questions

The shell already discloses `Document::recovery()`, and it would be easy to
assume this is the same fact twice. It is not, and the two are disjoint in
both directions:

| | `recovery()` | `load_anomalies()` |
|---|---|---|
| fires when | the stored cross-reference table could not be parsed and pdfcer rebuilt the index by scanning | any object contradicted itself, **including on a file whose index was perfect** |
| the operator's file | index was fine | doubled `/PageMode`, and a `/Metadata` stream with no `/Length` |

The file behind engine decision 145 has a **sound xref** and would light
neither the status line nor the Properties note that existed before today.
That is the gap this module closes.

## Item notes

### `type Clause`

A named type for the reason `super::notes`' own `NoteEntry` is one: it makes
[`clauses`]' table read as a table, and the `fn(usize) -> String` half is a
plain function pointer rather than a closure so the pairing is a *lookup* a
reviewer can read straight down the page.

### `fn the_census_is_ordered_by_consequence`

Every class at once, deliberately. A weaker fixture — two classes, one
of them absent — is satisfied by orderings that differ from the intended
one, so it would pass against a table somebody had shuffled. The property
being asserted is *"most consequential first, because the bar truncates
from the right"*, and only the full sequence states it.

### `fn the_panel_row_shows_what_pdfcer_chose_between`

This is the assertion that distinguishes a built feature from a counted
one. The engine carries `kept` and `discarded` specifically so a shell can
show what pdfcer chose between; if only the count arrives, the operator's
question — *"what if it is the wrong one?"* — has no answer on screen.

### `fn the_contradicting_fixture_produces_exactly_one_anomaly_through_the_engine`

Every test above builds its `LoadAnomaly`s by hand and asserts on the
prose. That is the right way to pin wording, and it proves **nothing at
all** about whether the engine ever hands this shell an anomaly, or
whether the variants it hands over are the ones these tests construct.
A build where `Document::load_anomalies()` always returned empty would
pass every one of them, and the operator would see a status bar that
never mentions his file.

So this one opens `fixtures/contradicts-itself.pdf` — a catalog naming
`/PageMode` twice with two different values, the shape of engine
decision 145's own file — and asserts three things in order of what
they would cost if untrue:

1. **It loads.** Before `Pass 283.0` a file like this was refused
   whole. `open_local_fixture` panics if it does not, so this claim is
   made by the test existing.
2. **Exactly one anomaly comes out**, and it is the duplicate key. Not
   "at least one": a loader that reported the same contradiction twice
   would put a wrong count in the status bar, and the count is the
   entire content of the census clause.
3. **Both values survive the trip.** The kept value and the discarded
   one reach the panel row, which is what makes the operator's eventual
   override offerable rather than theoretical.

⚠ The kept value is `/UseOutlines` — the LAST occurrence, under the
engine's default `DuplicateKeyPolicy::KeepLast`. If a future engine
revision changed the winner this assertion would go red, which is
wanted: it is a change an operator can see.

⚠ It also asserts the recovery line stays SILENT. `recovery()` and
`load_anomalies()` are disjoint questions — the xref of this fixture is
sound and every offset correct — and a fixture that lit both would let
a check pass while reading the wrong disclosure.

### `fn the_control_fixtures_a_driven_run_uses_are_genuinely_clean`

Without this, `ui-verify`'s `load_anomalies_are_disclosed` is a check
that cannot fail in one direction. That check launches twice - once on
`contradicts-itself.pdf`, asserting the status line and the
Document-properties rows are THERE, and once on a clean file, asserting
they are NOT - and the second launch is the half that distinguishes
"the disclosure works" from "the disclosure is always on screen".

If the control fixture quietly grew an anomaly of its own, that absence
assertion would start failing and the report would name the wrong
defect: it would say the disclosure leaks onto clean files, when what
actually happened is that the control stopped being clean. That is the
harness-input failure mode this project has paid for more than once -
a check whose verdict is about its own fixture, worded as a verdict
about the program.

⚠ So this is a tripwire for the harness's INPUT, not for this module's
logic, and both fixture names live here rather than only in the check:
a Rust test runs on every `cargo test`, and a driven check runs when
somebody has the machine's pointer to spare.

### `struct Census`

A named struct rather than a tuple so the two call sites read as prose and so
a fifth field cannot be added without every construction site being visited
by the compiler.

`unknown` is not a bug counter. It is the count of anomalies reported by a
`LoadAnomaly` variant this build has no arm for, which becomes possible the
moment the engine adds one — see [`crate::text::anomalies::clause_unknown`]
for why silently dropping them would be the worst available behaviour.

### `fn is_clean`

The control the engine's notice calls out by name: *"An empty slice means
the file was clean — that is the control, and it is tested."* Both
surfaces draw nothing in this case; see
[`crate::text::anomalies`]' header for why the all-clear is not stated
out loud.

### `fn census`

⚠ The `_` arm is **not** dead code and must never become an `unreachable!()`.
[`LoadAnomaly`] is `#[non_exhaustive]`, which means a newer `pdfcer-core`
compiles against this shell without the compiler saying a word about a
variant that has no arm here. The gate catches the API growing; this arm
catches what the operator sees in the window between the engine growing and
this file being taught.

### `fn clauses`

Empty when the file was clean, which is how the caller knows to draw nothing.

# The order is the argument

Most consequential first, because the bar **truncates** (R128) and truncation
drops from the right. Whatever an operator's window width leaves room for, he
should be reading the gravest clause:

1. **Objects that could not be read** — the only class where content is
   *absent*. Every other class is a choice between two readings of content
   that is present.
2. **Duplicate keys** — pdfcer picked one of two values the file offered, and
   on a drawing a wrong pick is a line in the wrong place on a page that
   renders perfectly.
3. **Stream lengths** and 4. **missing terminators** — the file is damaged
   and pdfcer measured what the file failed to state. There was no second
   reading to choose between, so there is nothing here the operator could
   have decided differently.
5. **Unknown** — last, because it is the only clause that is about *this
   build* rather than about the file.

### `fn status_line`

`Option` rather than an empty `String` so the caller cannot accidentally draw
an empty row: an allocated, empty, hoverable rect in the status bar is a
disclosure that reads as a rendering bug, and R9 says an inapplicable
capability renders **nothing**.

### `fn rows`

# Why these are NOT re-ordered the way [`clauses`] is

The bar's census is a summary and is sorted by consequence. This is the list
the operator reads when he has gone looking for *which*, and the engine's
order is the file's own order — roughly the order the objects appear in the
bytes. Sorting it by class would scatter the two defects of one broken object
(the operator's file has a doubled key *and*, one object later, a stream with
no `/Length`) into two distant groups, and locating a fault in a file is a
positional job.

⚠ Unbounded in length: a badly damaged file can report hundreds. The caller
draws this inside the Properties panel's existing scroll area — never in the
status bar, which gets the census and nothing else.
