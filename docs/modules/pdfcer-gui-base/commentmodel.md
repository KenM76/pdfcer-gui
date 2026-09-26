# `commentmodel` — turning a document into a comment list

The whole of the Comments panel that is not drawing. [`collect`] walks the
session's pages, applies the exclusion rule, classifies what survives, and
hands back a [`Listing`] the body renders row for row. Nothing here touches
`egui`.

## Why the panel is split this way

Because every interesting decision this panel makes is a *classification*,
and a classification is only testable if it is separable from the widget
that shows it. The list of things that have to be right —

- which annotations are excluded, and how many of each,
- whether a `/Line` is a **ce dimension** or a genuine `/Line` markup,
- whether `/Contents` is a note or an accessibility description,
- whether the row is a reply, a group subordinate, neither, or something
  `/RT` named that pdfcer has never heard of,
- whether the annotation is suppressed on screen,
- whether its appearance state could be resolved,
- and the ordering of the whole thing

— is exactly the list `tests` below sweeps against real engine fixtures.
`crate::panels::objects` is split on the same seam and for the same reason
(`provider.rs` and `summary.rs` beside its `mod.rs`), and this module's
`Vec<CommentRow>` is that pattern at a much smaller scale.

## The ordering is `pdfcer list-annotations`', reused by name

**Page order, then `/Annots`-array order.** Reused rather than reinvented:
a second, GUI-only ordering rule could disagree with the CLI's, and an
operator comparing a panel against a command's output on the same file
would have no way to tell which of the two had drifted. `/Annots` order is
whatever [`pdfcer_core::annot::page_annotations`] returns, which is the
array order the file itself carries — not a sort pdfcer imposes.

There is deliberately **no sort by date**. `/M` is stored raw because
§12.5.2 makes it *"date or text string"* and requires a reader to accept
any format, so ordering by it would mean parsing a value the standard says
may not parse — and any such feature owns that decision itself rather than
inheriting it from a list nobody asked to be sorted.

## Read the SESSION, not the file on disk

[`collect`] takes an [`ObjectGraph`], and the body hands it
`doc.session.view()` — the base revision with **every unsaved edit
applied**, which is the same thing the canvas rasterizes. An operator who
has just drawn three shapes must see three rows without saving first.
`crate::panels::forms`' body carries the same rule and the same sentence.

## Item notes

### `fn contents_is_description`

The list is §12.5.6.2's: `Link`, `Movie`, `Widget`, `PrinterMark` and
`TrapNet` use `/Contents` purely as an accessibility alternate.

`Widget` and `TrapNet` are in the list even though [`collect`] excludes
both, and that is deliberate: this predicate answers *"what does the
standard say about this subtype"*, and a copy of the spec's list that had
been trimmed to match today's filter would silently become wrong the day
the filter changed. Two rules, kept separate.

Everything not named is treated as a subtype that **displays** its
`/Contents`, which is the conservative direction: mislabelling a markup
note as an accessibility description would tell an operator that somebody's
comment was written for a screen reader.

### `const MAX_THREAD_DEPTH`

Eight, matching `crate::canvas::notepopup::model::MAX_THREAD_DEPTH` — the
two walk the same graph in opposite directions and a shell whose upward
bound differed from its downward one could resolve a Go to onto a root
whose own window would not list the row that was clicked.

### `fn a_popup_is_excluded_and_counted`

`popup-not-painted.pdf` carries exactly one annotation and it is a
`/Popup`. The listing is therefore empty — which is the *correct*
answer and the one most likely to be mistaken for a broken panel — and
the exclusion count is what lets the panel say so.

§12.5.6.14: a pop-up *"shall not appear alone but is associated with a
markup annotation, its parent annotation"*. It is a reader-UI window,
never independent content, so one row per real annotation is the whole
rule.

### `fn a_trapnet_is_excluded_and_counted`

Prepress output state — it records the trapping a RIP applied to the
page. Neither a comment nor anything a person wrote, so it is not
listed; `crate::panels::comments`' header carries the full argument.

### `fn a_ce_dimension_is_listed_and_recognised`

Both sides of the exclusion argument at once. They are `/Line`
annotations, so they appear here; excluding them by subtype would also
hide a genuine `/Line` markup an operator drew. And because the sidecar
can tell the two apart, the row says "ce dimension" instead of "Line"
without the filter ever being involved.

### `fn a_document_with_no_sidecar_has_no_ce_dimensions`

The other direction, which is the one that would go wrong silently. If
[`ce_dimension_annots`] ever returned something for an undimensioned
document, every `/Line` markup in the corpus would be relabelled and
the mislabelling would look like a document fact.

### `fn rows_are_in_page_order`

Asserted as a monotonic page index rather than against a hard-coded
list, because the second half — `/Annots` order — is the file's own
array order and pinning it would be pinning
[`pdfcer_core::annot::page_annotations`]' contract rather than this
module's. What this module owns is the outer loop, and the failure it
would produce is a list that jumps between sheets.

### `fn a_threaded_annotation_is_recognised_as_a_relation`

`thread.pdf` carries `/IRT` links. Table 170 makes `/RT` default to
`R`, so an annotation with `/IRT` and no `/RT` **is** a reply — and a
call site that read `reply_type` directly would report `None` and get
the ordinary threaded comment wrong. Core names that as the trap; this
pins that the panel does not walk into it.

### `fn a_hidden_annotation_is_listed_and_flagged`

A panel that silently omitted a suppressed annotation would be hiding
document content. Hidden annotations are a recognised document-forensics
vector, which is why core counts them rather than dropping them and why
this panel is the off-canvas surface that reports them.

### `fn the_five_non_text_subtypes_are_descriptions_and_nothing_else_is`

The §12.5.2 dual purpose, decided here because core deliberately
declines to: *"a UI labelling this 'comment' is right for markup and
wrong for a Link"*, and the interpretation *"belongs to whoever
displays it."*

Asserted against the predicate rather than a fixture because the corpus
has no `/Link` carrying `/Contents` — and a test that silently proved
nothing would be worse than one that pins the rule it implements. The
list is checked in both directions, which is what stops a markup note
being relabelled as a screen-reader string.

### `fn the_panels_subtype_vocabulary_is_the_one_the_disclosure_asks`

[`CommentRow::subtype`] is filled from `pdfcer-core`'s own
`Annotation::subtype_label` — the raw `/Subtype` name — and
`crate::text::textannot::paints_its_note` is a **string match on that
same vocabulary**. The status-line disclosure asks it with
`MarkupNoteChange::subtype`, which is the same string from the same
producer, so the two are coupled by the spelling of a name and by
nothing else. Title-casing this panel's subtype for display, or swapping
it for an enum, is the first step toward them coming apart, and the
symptom is a *missing* sentence — which no screenshot shows.

Reusing the row above's list is the point: it is this panel's own
enumeration of what displays its `/Contents`, i.e. exactly the rows that
get an editor, so the two cannot be brought into disagreement by adding
a subtype to one list and not the other.

The positive assertion is first and is load-bearing. A version that only
looped the `!paints_its_note` claims would pass on a `paints_its_note`
reduced to `false`.

### `fn the_all_without_notes_condition_is_not_vacuously_true`

The condition the panel keys its "shapes pdfcer drew carry no note"
sentence on. It must be true when every row lacks note text, false when
any row has some, and false on an empty listing — the third being the
one a naive `.all()` gets wrong, because `.all()` on an empty iterator
is `true` and would print a paragraph about note text under a heading
that just said there is nothing at all.

### `fn every_row_can_be_navigated_to`

The index is fed straight to
[`crate::app::actions::Action::GoToPage`], so an out-of-range value
would be a navigation to nowhere. It cannot happen — the index is the
enumeration of `pages` — and it is pinned anyway, because this is the
one number in the row that leaves the panel.

### `fn with_note_text`

Counts [`Note::Text`] only. A [`Note::Description`] is the document's
accessibility alternate for a control that displays no text of its own
(§12.5.2, §14.9.3) — counting it here would make
[`Self::every_row_lacks_note_text`] false on a document whose only
"note" is a screen-reader label for a link, and the panel would then
withhold the disclosure that stops the list reading as broken.

### `fn every_row_lacks_note_text`

`false` on an empty listing, deliberately: with no rows there is
nothing for the sentence to explain, and the panel says
`comments_none()` instead. A vacuous truth here would print a paragraph
about note text under a heading that just said there is nothing at all.

### `struct Excluded`

Counted rather than discarded, because the panel discloses it. A reviewer
looking at six rows on a drawing they know carries forty annotations needs
the arithmetic; see `crate::text::panels::comments::comments_excluded`.

### `struct CommentRow`

Owned strings rather than borrows of the annotation. The annotations are
modelled fresh from the graph inside [`collect`] and dropped when it
returns, so borrowing would tie the listing's lifetime to a temporary; and
the whole listing is a few hundred short strings at most, bounded by
`pdfcer_core::annot::MAX_ANNOTS_PER_PAGE` per page.

### `enum Note`

# §12.5.2 gives the key two jobs, and they are not interchangeable

*"Text displayed for the annotation, **or** (if the type does not display
text) an alternate human-readable description"* for accessibility
(§14.9.3). Which one it is depends on the subtype, and `pdfcer-core`
deliberately models the raw value **without** that interpretation: a label
reading "comment" is right for markup and wrong for a `/Link`, so the
interpretation belongs to whoever displays it. This enum is this panel
accepting that job.

### `fn ce_dimension_annots`

# Why this cannot be answered from the annotation alone

A **ce dimension** is a `/Line` annotation carrying `/IT /LineDimension`, a
baked `/AP` and a record in the document's `/PieceInfo` sidecar — and
`pdfcer_core::annot::Annotation` models **none** of those three: `/IT` is
among the per-subtype keys it deliberately does not carry, and the sidecar
is a different structure entirely.
The authoritative answer is the sidecar's own model, whose
`DimensionRecord::annot` is the annotation each record was written for.

# Why the panel bothers

Project rule 15. A **ce dimension** and a **pdf dimension** have opposite
properties — one pdfcer authors and can restyle, regroup and delete as a
unit; the other is CAD-exported page content pdfcer reads and must not
silently alter — and a row that showed the first as plain "Line" would be
true about the file and useless to the operator.

ce dimensions are **not** filtered out, because filtering by subtype would
also hide a genuine `/Line` markup somebody drew. The sidecar is what lets
the panel tell the two apart *without* filtering either.

# Cost

One catalog → `/PieceInfo` → `/pdfcer` → `/Private` walk and a
deserialization of the sidecar, bounded by the number of ce dimensions in
the document rather than by its size. Called **once** per frame by
[`crate::panels::comments::body`], never per row. A document that has never
been dimensioned has no sidecar and gets an empty set, which is the
ordinary case and the cheapest one.

### `fn collect`

`graph` must be the **session view**, not the loaded file — see this
module's header. `pages` is `OpenDoc::pages`, the flattened page vector
resolved once at open, and its index is the page index every row carries.
`ce_dimensions` comes from [`ce_dimension_annots`].

# The exclusion rule

[`crate::panels::comments`]' header states all four clauses and the reason
for each.

### `fn thread_root`

Returns `id` itself for an ordinary comment, which is the overwhelmingly
common case and costs one lookup.

# What it is for

`crate::canvas::notepopup` draws a window for a **comment**, and shows that
comment's replies inside it. It draws none for a reply — see
`notepopup::model::notes_on`'s exclusion table, and the reason is that a
reply sits on its parent's own `/Rect`, so a bubble for it would cover the
thing it answers. So *Go to* on a reply row has to ask for the **root's**
window, or it asks for a window that will never be drawn and the operator
presses a button that does nothing.

⇒ The alternative — leaving replies drawable so Go to had something to open
— is the worse trade by a distance, because it costs the *parent's* window
on every comment anybody ever answers.

# Bounded, because a `/IRT` cycle is legal syntax

§7.3.10 makes a dangling reference not an error and says nothing at all
about a circular one, and `pdfcer-core` surfaces `/IRT` unresolved: a
dangling `/IRT` is modelled, not repaired.
A file that says `a` replies to `b` and `b` replies to `a` is therefore a
file this panel must survive, and an unbounded walk over one would hang the
frame that is trying to draw. [`MAX_THREAD_DEPTH`] is the same bound
`notepopup::model::replies_to` uses and for the same reason; when it runs
out, the deepest annotation reached is returned, which is a real row in the
document and therefore a Go to that lands somewhere rather than nowhere.

A row whose parent is not in `rows` — a `/IRT` pointing at a `/Widget`,
at a `/Popup`, or at nothing — also stops the walk and returns what it has.
Same reason: this resolves a **destination**, and the honest failure of a
destination resolver is the nearest real place, never a panic and never an
`Option` the caller would have to invent a fallback for.
