# `text::panels::comments` — every string the Comments panel shows

The copy for [`crate::panels::comments`], which lists **every annotation
in the document** — the comment list a reviewer works through. One module
per panel surface, as [`super`]'s header lays out; `crate::panels::comments`
is the sole consumer.

## Most of this is salvaged verbatim, and the doc comments came with it

Nine of the entries below came across from the old shell's `ui_text.rs`
**with their doc comments**, because in this project a doc comment on a
string is usually the record of the defect the wording was changed to fix.
⚠ **Fresh words re-derive a decision already paid for, without access to
the evidence that bought it.**

The two that carry the most reasoning:

- [`comments_all_without_notes`] exists because pdfcer's own markup
  authoring cannot write `/Contents` on a geometric shape —
  `pdfcer_core::annot_author::MarkupSpec` has no text-bearing variant on
  purpose — so a document whose annotations pdfcer drew shows a column of
  identical "no note" captions. `docs/core-api/03-capabilities.md` §3.4
  makes saying so **mandatory copy**: *"A bare 'No note text' column reads
  as data loss."*
- [`comments_none`] names what is **excluded**, because a document that is
  nothing but form fields would otherwise show an empty comment list and
  look broken.

## What is new here, and why each one exists

Six entries have no ancestor in the old shell. Every one of them is a
**disclosure** that `docs/core-api/03-capabilities.md` §3.4 ("what the UI
must disclose") or §3.5 ("Traps") asks for by name, and each says which:

| Entry | Commissioned by |
|---|---|
| [`comments_excluded`] | this panel's own decision to state its filter in numbers rather than in the abstract — see [`crate::panels::comments`]' header |
| [`comment_row_hidden`] | §3.4.5 — *"A Comments panel that silently omits it is hiding document content; list it and mark it hidden."* |
| [`comment_row_appearance_unresolved`] | §3.4.4 — pdfcer *"displays nothing and does not guess"*, and the governing default is explicitly a reasoned guess, i.e. an inference, i.e. rule 4 applies |
| [`comment_row_description_caption`] | §3.5 — *"`/Contents` is dual-purpose … a UI labelling this 'comment' is right for markup and wrong for a Link"* |
| [`comment_row_is_group_member`] | §3.5 — the §12.5.6.2 group-attribute rule is **deliberately not applied** by core, so what this panel shows is the raw dictionary value and a conforming reader shows something else |
| [`comment_row_ce_dimension_heading`] / [`comment_row_ce_dimension_no_note`] | project rule 15 — a **ce dimension** is a `/Line` annotation, and a row that called it "Line" would be true about the file and useless to the operator |

## Rule 15 is enforced by a test in this module

*"Never write a bare 'dimension".* **ce dimensions** are the ones pdfcer
authors (`/Line` + `/IT /LineDimension` + a `/PieceInfo` sidecar); **pdf
dimensions** are CAD-exported page content. They have opposite properties
and the ambiguity has already sent one investigation down the wrong path,
so [`tests::no_string_here_says_a_bare_dimension`] sweeps every entry
rather than trusting review — this is a *catalog*, which is exactly the
kind of file where a bare noun slips in during a late reword.

## Conventions, restated from [`crate::text`] because they bind here

- **Sentence case, no trailing period on labels; full sentences with
  punctuation for prose.**
- **Never state a capability the build does not have.**

### A SENTENCE ABOUT WHAT THE BUILD CANNOT DO HAS A SHELF LIFE

This panel once carried, as a reasoned decision, *"this build's panel has
no Delete, because `Action` carries no variant that could delete an
annotation."* ⚠ **The reason was correct, was written down, and stopped
being true without anything in this file changing.**
`AnnotAction::Delete { page, id }` exists, `crate::app::actions::annots::delete`
reaches `EditSession::delete_annotation` through it, and the canvas Delete
key and the Format tab both use it — leaving this panel, the reviewer's own
work list, as the one surface that could not do the plainest thing on it.

The operator's standard for this panel is *"the review features should
look and act the same as they do in Acrobat Reader"*, and Acrobat lets a
reviewer delete their own comment.

⇒ [`comment_row_delete`] and [`comment_row_delete_tooltip`] are below, and
the rule this cost is: **where a claim about a capability can be an
ASSERTION, it must be one.** Here it is
`crate::panels::comments::tests::the_delete_control_reaches_the_engine`,
which goes red the day the wiring stops being real — which prose cannot.

## Item notes

### `fn all_fixed`

Hand-written, like every enumeration of things Rust cannot enumerate
for us. It is only used by tests, so an entry missed here weakens a
check rather than shipping a defect — but it is listed in the order
the panel draws them so a reader can diff the two.

A function rather than a `const`, for the same reason
`crate::app::modes::defaults`' `SideSpec` is owned rather than
`&'static`: these are ordinary functions, and an ordinary call cannot
be promoted into a `const` initializer.

### `fn nothing_excluded_draws_nothing`

The alternative — an empty string — still reserves a label's height,
which on a narrow dock reads as a rendering fault, and it is the
no-placeholders rule applied to prose.

### `fn the_exclusion_line_names_where_each_kind_went`

The failure this stops is the one-clause version that says "12 items
were not listed": the count without the destination, which tells an
operator something is missing and not where to look for it.

### `fn a_byline_with_neither_half_is_not_drawn`

Both halves are legitimately absent — `/T` is a Table 170 markup key
and means "this subtype has no such concept" on a `/Link` — so all four
combinations are reachable on real documents and each has to render
correctly. The `(None, None)` case in particular must not become an
empty line.

### `fn a_modification_date_is_never_reformatted`

§12.5.2 makes `/M` *"date or text string"* and requires a reader to
accept any format, so `pdfcer-core` stores it raw. A catalog entry that
tidied it would either reject a value the standard requires be accepted
or silently mangle it — and the mangling would look like a document
fact rather than pdfcer's own edit.

### `fn an_absent_note_is_never_described_as_missing_or_broken`

The whole point of the sentence. Note text is absent on every shape
pdfcer itself drew — `MarkupSpec` carries no contents field, deliberately
— so this caption is the *ordinary* case on a pdfcer-marked document, and
a word like "missing" or "error" would send an operator hunting for
damage in a file that has none.

### `fn prose_is_punctuated_and_the_one_label_is_not`

`crate::text`'s convention: a label is a name and carries no trailing
period; a message is a statement and does. [`comment_row_goto`] is the
only label here, and it is the only entry allowed to end without
punctuation.

### `fn every_row_state_says_something_different`

Each one distinguishes a *different* state — an absent note, a hidden
annotation, an unresolved appearance, a reply, a group member — and two
that read alike would collapse two states the operator has to be able
to tell apart. Same reasoning as
`crate::panels::bookmarks`' three-row-state check, which exists because
a heading and a broken destination rendering identically would send an
operator hunting for damage in an ordinary document.

### `fn a_page_number_reaches_the_string_unchanged`

The off-by-one guard, from the other side. `Action::GoToPage` takes a
**0-based** index and these take a **1-based** human page number, so the
`+ 1` happens exactly once, at the call site — see
`crate::panels::comments`' own test for the half of this that pins the
action.

### `fn a_replys_own_popup_is_disclosed_only_when_the_engine_reports_one`

# The fact, and why nothing on screen can carry it

`add_reply` authors a `/Popup` companion for the reply it creates, and
`ReplyAdded::reply_has_popup` reports it because this shell asked to be
told. pdfcer does **not** draw that window —
`canvas::notepopup::model::notes_on` excludes replies, so an answer
appears inside the thread of the comment it answers and never as a
second bubble on top of it — which means an operator who has only ever
seen this program has no way to learn the window is in their file.
Another reader will draw it.

The `false` case is asserted beside it because the sentence is a
**claim about the file**: firing it unconditionally would tell the
operator about a window pdfcer had not established was there, which is
rule 4 broken in the direction that is hardest to notice — a disclosure
that is wrong reads exactly like one that is right.
