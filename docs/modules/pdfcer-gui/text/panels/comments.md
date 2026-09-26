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

### `fn comments_count`

Salvaged verbatim. `note(s) and markup item(s)` rather than "comments":
the list contains a `/Link`, a `/Stamp` and a ce dimension as readily as it
contains somebody's sticky note, and calling all of those "comments" is
wrong on the rows where the distinction matters most.

### `fn comments_none`

Salvaged verbatim. Names what is EXCLUDED, because a document full of form
fields would otherwise show an empty comment list and look broken. Form
fields and pop-up windows are deliberately not comments.

[`comments_excluded`] follows it with the actual numbers when there are
any, so this sentence answers *"is this panel broken?"* and that one
answers *"then where did my annotations go?"*.

### `fn comments_excluded`

# Why the counts are stated rather than the rule

`crate::panels::comments`' filter is settled and argued in that module's
header, but an operator reading a list of six rows on a drawing they know
carries forty annotations needs the arithmetic, not the doctrine. The old
shell stated the rule ([`comments_none`]) and only on the empty case; this
states the numbers, on every case where there are any.

It is a **disclosure**, in rule 4's sense: the panel made a decision about
what to show, and the decision lives off-canvas in the panel that made it.

# `Option` rather than an empty string

So the caller cannot accidentally draw a blank line. A panel that renders
an empty label still reserves its height, which on a narrow dock reads as
a rendering fault. `None` means *draw nothing*, which is also the
no-placeholders rule applied to prose.

# The three clauses are built by hand rather than by a list joiner

Because each one has to name *where the thing went*, and the destinations
differ: form fields have another panel, pop-ups belong to the annotation
they hang off, and a `/TrapNet` is prepress output state that no reviewer
wrote and nobody can answer. A generic "N items were excluded" would be
the count without the fact that makes it actionable.

### `fn comments_all_without_notes`

Salvaged verbatim, and it is **mandatory copy** rather than a nicety —
`docs/core-api/03-capabilities.md` §3.4 requires it in these words, and
records why: pdfcer's own markup tools cannot attach a note to a shape
(`pdfcer_core::annot_author::MarkupSpec` has no text-bearing variant, on
purpose), so a document whose annotations pdfcer authored shows a column
of identical "no note" captions. Said once at the top rather than left to
be inferred from the repetition.

**Note-text authoring for geometric markup is a filed request against
the engine** (`ENGINE_BACKLOG.md`). ⚠ The sentence is therefore worded as a
fact about THE SHAPES — which stays true for everything already drawn —
rather than as a claim about what pdfcer can never do, which would go false
the day the verb lands.

### `fn comment_row_heading`

Salvaged verbatim. `subtype` is
`pdfcer_core::annot::Annotation::subtype_label`, which is the `/Subtype`
name decoded lossily or `(no Subtype)` when the key is absent — a
malformed annotation, surfaced rather than repaired.

### `fn comment_row_ce_dimension_heading`

# Why this is a second function and not a substituted label

Project rule 15: a **ce dimension** is the thing pdfcer authors — a `/Line`
annotation carrying `/IT /LineDimension`, a baked `/AP` and a `/PieceInfo`
sidecar record — and a **pdf dimension** is CAD-exported page content. They
have opposite properties. A row that showed a ce dimension as plain "Line"
would be true about the file and useless to the operator, who has a Measure
tab full of verbs for exactly this object.

The subtype is kept **in brackets rather than replaced**, and that is the
whole reason this reads the way it does. The old shell's exclusion argument
(`comments_panel`'s own doc) turns on ce dimensions being ordinary `/Line`
annotations — that is why they cannot be filtered out by subtype without
also hiding a genuine `/Line` markup somebody drew. A heading that hid the
`/Line` would quietly contradict the argument that put the row here.

### `fn comment_row_byline`

`None` when it has neither, so the caller draws nothing rather than an
empty line. See [`comments_excluded`] on why absence is an `Option` here.

# Both halves are legitimately absent, and neither absence is a fault

`/T` is a **Table 170 markup key**, so it is legitimately absent on a
`/Link` or a `/PrinterMark`, where `None` means *"this subtype has no such
concept"* rather than *"anonymous"* (`Annotation::title`). Printing an
"(unknown author)" placeholder would turn a correct fact about a subtype
into a claim about a person.

`/M` is optional on everything.

# The date is passed through verbatim, and that is not laziness

§12.5.2 gives `/M`'s type as *"date **or** text string"* and requires a
conforming reader to *"accept and display a string in any format"*, so
`pdfcer-core` stores it raw and its own docs say *"do not assume it
parses"*. Formatting it here would mean writing a §7.9.4 parser whose
failure mode is either rejecting a value the standard requires be accepted
or silently mangling it. The word "modified" carries the meaning; the value
carries whatever the file said. [`comment_row_modified_tooltip`] is where
the operator finds that out.

### `fn comment_row_modified_tooltip`

On hover rather than on the row, because it is the answer to a question
most operators will never ask: the ordinary case is `D:20240117093000Z`,
which is legible enough to compare two rows by, and the sentence below is
only wanted by whoever wonders why pdfcer did not tidy it.

### `fn comment_row_body`

Salvaged verbatim, and it is a passthrough on purpose: the operator is
reading somebody else's words, and a catalog entry that decorated them
would be putting pdfcer's voice inside a quotation.

### `fn comment_row_no_note`

Salvaged verbatim. "No note text" rather than blank space: an empty row is
indistinguishable from a rendering failure, and this is a real, expected
state — see [`comments_all_without_notes`] for the reason it is *usually*
this state on a document pdfcer drew on.

**Worded as a fact about the document, not as an error.** There is no
"missing", no "(none)", no empty-set glyph and no warning colour. The
annotation is exactly as its author left it; a panel that dressed that up
as absent data would send an operator looking for damage in an ordinary
file — the same defect `crate::panels::bookmarks`' three-state row
distinction exists to avoid.

### `fn comment_row_ce_dimension_no_note`

A ce dimension never has note text, and for a different reason from the
shapes [`comments_all_without_notes`] covers: its measurement is baked into
its own appearance stream by `author_dimension`, so the number the operator
reads is *on the page*, not in a note. "No note text on this markup" would
be true and would read as a shortcoming; this says where the text actually
is.

### `fn comment_row_description_caption`

# The trap this closes

`docs/core-api/03-capabilities.md:1113` states it as a trap in so many
words: *"`/Contents` is dual-purpose — a UI labelling this 'comment' is
right for markup and wrong for a Link."* §12.5.2 defines the key as *"text
displayed for the annotation, **or** (if the type does not display text) an
alternate human-readable description"* for accessibility (§14.9.3), and
which of the two it is depends entirely on the subtype.

So on a `/Link`, a `/Movie` or a `/PrinterMark` the string above this
caption is not something a reviewer wrote — it is the document's
description of a control, addressed to a screen reader. Showing it in a
list headed "Comments" without saying so invites an operator to reply to
nobody.

### `fn comment_row_hidden`

# Listed and marked, never omitted

`docs/core-api/03-capabilities.md:1100` is explicit: *"an annotation the
file says not to show. A Comments panel that silently omits it is hiding
document content; list it and mark it hidden."*

The predicate is `AnnotFlags::suppressed_on_screen`, which is `/F` bit 2
(Hidden) or bit 6 (NoView) — §12.5.3 Table 165. The wording says *"you will
not find this on the page"* rather than *"this is hidden"* because the
operator's next act is to click Go to and look for it.

### `fn comment_row_appearance_unresolved`

# Why this is here at all, and why rule 4 makes it mandatory

`docs/core-api/03-capabilities.md:1093`: `Appearance::StateUnresolved`
means pdfcer *"displays nothing and does not guess a first / `On` / `Off`
key"*, and *"a blank annotation with no explanation looks like a rendering
bug."*

The governing setting is `MissingAppearanceState`, whose default
(`PaintNothing`) is documented in core as **evidence tier (d), a reasoned
guess**. That makes the blank page an *inference of pdfcer's*, and rule 4
says an inference the operator cannot see still owes an off-canvas report:
*"Render normally; report separately. Both."* A panel is the right home for
it, and nothing may be drawn on the page to mark it.

### `fn comment_row_is_reply`

Flat rather than indented under its target, and that is a scope decision
worth stating: threading the list would need the panel to resolve every
`/IRT` into a row, decide what to do about a dangling one, and pick a depth
for a cycle. This says the same fact in one line and leaves the ordering —
page order, then `/Annots` order — meaning exactly what it says.

Table 170 makes `/RT` default to `R` when absent, which is the ordinary
case for a threaded comment; the panel asks
`Annotation::effective_reply_type` rather than reading `/RT` itself, for
the reason core's docs give: *"a call site that treats an absent `/RT` as
'not a reply' is wrong in the ordinary case."*

### `fn comment_row_is_group_member`

# The one row in this panel where pdfcer and another reader will disagree

§12.5.6.2 says a group subordinate's own `Contents`, `M`, `C`, `T`,
`Popup`, `CreationDate`, `Subj` and `Open` *"shall be ignored"* in favour
of the group primary's. `pdfcer-core` **deliberately does not apply that
rule** — `Annotation::contents` is the raw dictionary value, because
*"silently substituting a primary's `/Contents` for a subordinate's would
make the model disagree with the file, which is the one thing
`pdfcer-core`'s read half must never do"* — `Annotation::contents`'s own doc.

Both halves of that are right, and together they mean the note text on this
row is text the standard instructs a conforming reader **not** to display.
Saying so is rule 4's *"pdfcer inferred something, and another reader may
compute different values"* in its purest form: nothing here is wrong, and
an operator who does not know it will be surprised by another viewer.

### `fn comment_row_add_note`

Two labels rather than one, because *add* and *edit* are different acts to
the operator and the difference is legible from the row: a row showing
somebody's words offers to change them, a row saying "No note text" offers
to write some. A single "Note…" would make the operator read the row above
the button to find out what pressing it does.

# Why two labels can exist at all

Setting `/Contents` on an annotation that **already exists** is an engine
verb, and while it was missing every shape this shell drew was permanently
wordless and this panel was a viewer. It exists now — which is why
[`comments_all_without_notes`] is worded as a fact about shapes pdfcer drew
rather than as a denial of the capability.

### `fn comment_row_note_save`

**An explicit commit, not a live binding**, for the reason
`crate::panels::properties::geometry` states about its own Apply: one
keystroke per undo entry would make `Ctrl+Z` walk backwards through a
sentence one letter at a time. One press is one `CommandKind::SetMarkupNote`.

### `fn comment_row_note_remove`

A separate control from Save-with-empty-text because `pdfcer-core` models
them as separate verbs and says why: *"an empty comment is a comment, and a
reviewer deleting their remark is not the same as leaving a blank one."*
`clear_markup_note` removes `/Contents`, `/T` and `/M` together.

### `fn comment_row_note_hint`

Names what is NOT obvious in a multi-line box, and one thing that is not
obvious anywhere: Enter inserts a line rather than saving, so the operator
needs telling how to save — and Escape **writes** rather than abandoning,
which is the opposite of what the key usually means, so the sentence has to
say it and has to name the control that does abandon. See
`crate::panels::comments::editor::escape_commits` for the ruling.

### `fn comment_row_note_signature`

Rule 4's surviving half: the author name is invisible on the page — a
sticky's byline lives in a pop-up window this shell does not draw, and a
shape's lives nowhere at all — so an operator who has never opened Settings
has no way to discover what name their comments carry, or that they carry
none.

# Why it names the setting rather than the value

A panel body is handed `&OpenDoc` and `&mut PanelsState` and **nothing
else** — no preferences — so this string cannot quote the configured name
without threading prefs through every panel signature in the crate for one
sentence. Naming the place answers the operator's real question (*where do
I change what my comments say?*), and the value itself becomes visible the
moment the note is saved, as the row's own byline.

Shown only when the row has **no** author, because on a row that has one
nothing is written to `/T` at all: `pdfcer-core` leaves an omitted key
untouched, which is what stops correcting a typo from un-signing somebody
else's comment.

### `fn comment_row_note_signature_kept`

The operator is about to change somebody's words, and the thing they cannot
see is that the byline will not move with them. Saying so is what stops the
panel looking as though it silently re-attributed a comment.

### `fn comment_row_note_not_editable_ce_dimension`

A **ce dimension** is refused by `pdfcer-core` **by name** — its `/Contents`
is generated from the measurement by `author_dimension`, so a note written
over it would be silently regenerated away. Saying where the text comes from
is more use than a greyed button, and R9 forbids the greyed button anyway:
this is not *temporarily* unavailable.

### `fn comment_row_note_no_handle`

§12.5.2 Table 164 requires an annotation dictionary to be an indirect
object, so this is a malformed file rather than a shortcoming of pdfcer, and
the row says which. Nothing that needs a handle may be offered for it.

### `fn comment_row_selected_heading`

# A word, not a colour

`DEFECTS.md` **D2** is this project's record of a theme making text
invisible against its own background — near-white on light grey, shipped,
with two theme tests sitting next to it that measured no
foreground/background pair. Every list in this shell that marks a row marks
it with a **shape or a word**: the Pages panel changes a tile's outline
*and* writes a count, and the Objects tree indents.

A reviewer scanning forty rows for the cloud they just drew needs that mark
to survive a theme nobody has measured yet.

# Why it wraps the heading rather than sitting on its own line

Because the row is already between two and seven lines tall and a separate
line would put the mark a variable distance from the thing it marks. The
arrow leads, so a column of headings scanned down the left edge shows it
without reading a word.

### `fn comments_filtered`

This panel's founding discipline is that *"nothing is silently omitted"* —
[`comments_excluded`] already states the arithmetic for widgets, pop-ups
and `/TrapNet`. A filter is a fourth kind of omission and the only one the
operator caused, which makes it **more** important to state rather than
less: an exclusion is a property of the document a reviewer can learn once,
while a filter is a switch they set an hour ago and have since forgotten.

A reviewer who reads six rows off a drawing they know carries forty and
concludes the other thirty-four are gone has been misled by this surface.

### `fn comment_filter_clear`

One press rather than three menus reset one at a time, because the state a
reviewer wants back is *"all of them"* and reaching it by undoing each
choice is three chances to leave one set.

### `fn comment_filter_author`

*Author*, not *Reviewer*: `/T` is §12.5.6.4 Table 170's *"name of the
person who created the annotation"*, and this panel lists `/Link`s and
stamps as readily as review comments. Calling the column Reviewer would
name a role the file does not record.

### `fn comment_filter_with_note`

It exists because of a property of pdfcer rather than of PDF:
`MarkupSpec` has no contents field on any variant, so **every shape this
program draws arrives with no `/Contents`**. On a drawing marked up here
the list is mostly rows with nothing to read, and this is the switch that
leaves the remarks somebody actually wrote.

### `fn comment_row_delete`

# Why it is worth a doc comment of its own

This control was once forbidden by a written, correct reason — that no
`Action` variant could carry the intent — and the reason expired silently.
`AnnotAction::Delete` reaches `EditSession::delete_annotation`, the canvas
Delete key and the Format tab both use it, and this panel — **the
reviewer's own work list** — was the last surface that could not do the
thing a reviewer most obviously does.

⇒ **A prohibition needs a tripwire in the same way a shim does.** The one
here is `crate::panels::comments::tests::the_delete_control_reaches_the_engine`.

### `fn comment_row_delete_tooltip`

What goes, that **delete is not redaction**, and — implied by the second —
that the words may still be in the file. The collateral (a pop-up removed,
replies orphaned, group members promoted) is reported *after* the call by
[`crate::text::markup::deleted_collateral`], because only the engine knows
what it actually took.

### `fn comment_row_reply`

A verb, not *"Reply…"* with an ellipsis: the ellipsis convention in this
shell means *this opens a dialog*, and this opens an editor **in the row**,
three pixels below the button. The same reasoning
[`comment_row_add_note`] follows.

### `fn comment_row_reply_tooltip`

It names **a new comment**, deliberately. The visible result of pressing
Post reply is a new row in this list and a few words inside the parent's
pop-up, which looks exactly like a note having been edited. It is not: it
is a `/Text` annotation of its own, with its own object number, its own
byline and its own date, and *"answers"* is the shortest true word for
that.

### `fn comment_row_reply_save`

*Post*, not *Save*, and the difference is not decoration: [`comment_row_note_save`]
writes words onto an annotation that already exists, and this **creates
one**. Two controls that read alike would leave the operator with no way to
tell, at the moment of pressing, which of the two things is about to happen
— and one of them cannot be told apart from the other afterwards either,
because a corrected note and a new reply both just show new words on screen.

### `fn comment_row_reply_hint`

Says what [`comment_row_note_hint`] says, naming *this* editor's commit
rather than that one's — an operator with two editors in one panel needs
the hint to say which button it is talking about.

### `fn comment_row_reply_signature`

# Why this is a different sentence from [`comment_row_note_signature`]

Because the *rule* is different, not because the wording drifted. Saving a
note over an existing comment may leave somebody else's `/T` untouched —
that is what `keep_author` decides, and that panel's disclosure has two
forms because the outcome has two shapes. A reply is a **new annotation**
and there is no prior byline anywhere in the question: it carries the
operator's name or it carries none, always, and one sentence covers it.

It names the *setting* rather than quoting the configured value, for
[`comment_row_note_signature`]'s stated reason: a panel body is handed
`&OpenDoc` and `&mut PanelsState` and no preferences at all, so quoting the
name would mean threading prefs through every panel signature in the crate
for one sentence. Naming the place answers the operator's real question —
*where do I change what my comments say?*

### `fn comment_row_reply_to_a_reply`

§12.5.6.2 permits a reply to a reply and `add_reply` allows it (it refuses
only a reply to *itself*, which is not a thread). This panel and the canvas
pop-up both draw a thread **flat**, so an answer to an answer appears in
the same list as everything else under the root rather than indented under
the thing it answers.

⇒ That is a real difference between what the file records and what the
screen shows, and rule 4 makes saying so mandatory. The `/IRT` written is
the row's own annotation — the file keeps the true depth — and this
sentence is what stops an operator concluding from a flat list that pdfcer
flattened their thread. `crate::panels::comments`' threading paragraph
carries the full argument.

### `fn reply_posted`

# The fact being disclosed

`add_reply` gives the reply a `/Popup` of its own (§12.5.6.14), and
`ReplyAdded::reply_has_popup` reports it because this shell asked to be
told: *"a reply that quietly acquired a second window at a second location
is something we would rather be told about than discover on a screenshot."*

pdfcer does **not** draw that window —
`crate::canvas::notepopup::model::notes_on` excludes replies, so an answer
is shown inside the thread of the comment it answers and never as a second
bubble on top of it. Another reader may draw it, and an operator who has
only ever seen this program has no way to know the window is in their file.

# `None` when there is no window

The engine reports the fact rather than guaranteeing it, so the sentence is
conditional on the engine's own answer rather than on this shell's
expectation. A disclosure that fired unconditionally would be a claim about
the file that pdfcer had not checked.
