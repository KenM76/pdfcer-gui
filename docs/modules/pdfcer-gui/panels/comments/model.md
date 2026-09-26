# `panels::comments::model` — turning a document into a comment list

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
