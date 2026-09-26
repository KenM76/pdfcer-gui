# `panels::comments::note` — the note being typed, and the stamp that keeps
it honest

One annotation's `/Contents` while the operator is editing it, and nothing
else. Split out of [`super`] rather than added to it because this is the
first piece of **inter-frame operator state** the Comments panel has ever
had, and that module's own header says in as many words that it had none:
*"it is a pure function of the document. Nothing in it is expanded, picked,
drafted or remembered."* That sentence is now false and is corrected there;
this file is what made it false, and it is worth its own file so the
argument for the correction is in one place.

## Why a draft at all, rather than writing on every keystroke

Because a keystroke is not an operator act. `EditSession::set_markup_note`
is **one undoable command**, so a live binding would raise one per letter
and `Ctrl+Z` would walk backwards through a sentence a character at a time
— the same argument `crate::panels::properties::geometry` makes for its
Apply button, and the same one `pdfcer-core` itself makes about looping a
singular verb.

⇒ So: type freely, press **Save note** once, get one undo entry.

## The stamp is `(annotation, edit epoch)`, and the epoch is the member
that is easy to leave out

The annotation half is obvious — a draft belongs to the row it was opened
on, and clicking a different row must not carry the words across. The epoch
half is the one that has already gone wrong once in this project, in
`GeometryDraft`, and the failure has the same shape here:

> Open the editor on a highlight, type three sentences, press `Ctrl+Z`
> (undoing something unrelated), then press Save.

Without the epoch that Save writes the operator's words onto whatever
annotation now holds that object id — and after an undo of an *add*, that
id may name nothing at all, or, worse, a different annotation entirely once
the writer reuses the number. With the epoch the draft is simply dropped
when the document moves under it, and the operator retypes, which is the
honest outcome: **the alternative is words landing somewhere nobody asked
for, silently.**

Dropped rather than *refused at Save time*, because a stale draft on
screen is a lie for however long it stays there — it shows text next to a
shape that no longer has that text — and the moment to stop lying is the
moment it goes stale, not the moment somebody presses a button.


`EditSession::add_reply` (`Pass 253.0`) closed *"we can read a comment
thread and cannot add to it"*, and the affordance it needed was **this
editor pointed somewhere else**: the same box, the same Escape route, the
same stale-epoch rule, committing to `add_reply` instead of
`set_markup_note`. So [`DraftTarget`] joined the stamp rather than a second
draft type joining the panel.

⇒ The rule that made it worth doing this way: **the epoch discipline above
is the expensive part, and it must not be written twice.** A separate
`ReplyDraft` would have needed its own `sync`, its own close-on-Escape and
its own reasoning about what a stale stamp means — three chances for the
reply path to keep words alive across an edit that the note path drops, and
the consequence of that divergence is an operator's answer landing on an
object id the writer has since reused.

What is NOT shared is the **seed**: [`NoteDraft::begin`] fills the box
with the existing words and [`NoteDraft::begin_reply`] leaves it empty, for
the reason on that method — a reply carries the operator's own byline, so
anything pre-filled goes out signed by them.

## What this deliberately does NOT do

- **It does not hold a second selection.** The draft names one annotation
  by `ObjId`; it does not decide what the canvas outlines, what the Format
  tab describes or what Delete acts on. `crate::panels::ObjectTreeUi::focus`
  carries the whole argument about why a panel growing its own selection is
  a defect waiting for two surfaces to disagree.
- **It does not know the author or the date.** Those are supplied at apply
  time from `crate::app::prefs` and `crate::app::clock`, because they are
  properties of the operator and the moment rather than of the draft. A
  draft that captured the clock when the editor opened would date a comment
  by when somebody started typing it.

## Item notes

### `fn a_reply_draft_and_a_note_draft_remember_which_they_are`

The single assertion this whole `DraftTarget` change exists for. A
build that stamped `Note` for both would compile, would draw a box that
said *Post reply*, and would commit `set_markup_note` — writing the
operator's answer **over the comment they were answering**. Nothing on
screen distinguishes that from a reply having worked until somebody
reads the file, which is exactly the class of defect a unit test can
still catch.

Both directions, because asserting only the reply case would pass on
an implementation that returned `Reply` unconditionally — and that
build turns every note correction into a new annotation, leaving the
typo on the page with an answer stuck to it.

### `fn a_reply_starts_blank_and_a_note_edit_does_not`

The rule on `begin_reply`, asserted because it is a one-character
difference in the implementation with a consequence in the file: a
reply pre-filled with its parent's words is authored as a new
annotation carrying the **operator's** `/T`, so it publishes somebody
else's sentence under their name.

The seeded case is asserted beside it as the control — without it this
would pass on a draft that never seeds anything, which would make
correcting a typo into retyping the sentence.

### `fn a_reply_editor_counts_as_the_rows_one_open_editor`

[`NoteDraft::editing`] deliberately ignores the destination, so a row
whose reply box is open reports itself as editing and does not also
offer *Add note* beside it. Two boxes on one row would be two drafts,
and this type holds one — the second would silently take the first's
words.

### `fn an_edit_under_the_operator_drops_a_reply_draft_as_well`

The epoch rule is the expensive half of this type and the whole reason
the reply reuses it rather than getting a draft of its own. A build
that added a parallel reply draft without a `sync` would keep the words
alive across an edit and post them at an object id the writer may have
reused — the failure `an_edit_under_the_operator_drops_the_draft`
describes, on the path where the words become a *new annotation* rather
than a correction.

### `enum DraftTarget`

# Why the destination lives on the draft rather than beside it

Because a reply *is* the note editor with a different destination, and
building it as a second draft, a second `TextEdit`, a second Escape
handler and a second stale-epoch rule would have been four places for the
two to drift apart. `crate::panels::comments::editor` therefore draws one
box and asks this enum which verb the commit raises, which means the
**stale-draft rule, the Escape route and the seeding rule are written
once** and cannot come to differ between the two.

And it is on the **stamp**, beside the annotation and the epoch, rather
than a fourth loose field. Those three answer one question together —
*what is open, on what, as of when* — and a destination that could be read
while nothing was open is a destination that eventually is.

# Why an enum and not `is_reply: bool`

The two reach different engine verbs with different outcomes:
`set_markup_note` edits a dictionary that exists, `add_reply` creates an
annotation. `crate::app::actions::annot::AnnotAction::Reply` makes the same
argument for keeping them apart on the action bus, and the argument is the
same one rung down — a bool is a value a future `match` can forget to
consider, and the outcome of forgetting here is a reviewer's answer written
over the comment it was answering.

### `fn begin`

Seeded rather than blank, because *edit* is the commoner act than
*replace*: correcting a typo in a comment is the fourth of the four
rows `pdfcer-core`'s own reply lists as what a review IS, and it would be
retyping from scratch against an empty box.

### `fn begin_reply`

Empty, and that is the opposite of [`Self::begin`]'s rule rather
than an oversight in it. Seeding a note editor is right because *edit*
is commoner than *replace* and the operator is correcting words that
already exist. Seeding a **reply** with the parent's words would put
somebody else's sentence in the operator's mouth: `add_reply` writes a
new annotation with the operator's own `/T`, so anything left in the
box goes out signed by them.

⇒ Two entry points rather than one with a `seed` the caller passes
`""` to, because that spelling puts the rule at every call site instead
of in the type — and there would be exactly one call site until the day
there were two.

### `fn close`

Called from Cancel, from Escape, from a successful Save, and from
[`Self::sync`] when the document moves. **One way to close**, so a
future arm cannot leave the text behind while clearing the stamp.

### `fn editing`

The epoch is compared rather than ignored so that a caller cannot
accidentally draw a stale editor between the edit landing and the next
[`Self::sync`]; in practice `sync` runs first, and this is the belt to
its braces.

The **destination is deliberately not compared**, and that is what
makes the row draw one editor rather than two: a row whose reply box is
open must not also offer *Add note* beside it, because two boxes on one
row is two drafts and this type holds one. Ask [`Self::target`] when the
answer matters.

### `fn target`

Asked by the row that is drawing the editor, so the labels, the hint,
the signature disclosure and the verb the commit raises all come from
one answer — the failure mode being a box captioned *Post reply* whose
Save writes `/Contents`, which is invisible until somebody reads the
file.

### `fn sync`

Called once per frame by the panel, before anything is drawn, with the
document's current edit epoch. See the module header: a draft stamped at
an older epoch describes a document that no longer exists, and the
moment to stop showing it is the moment it goes stale.

A no-op when nothing is open, which is almost every frame.

### `struct CommentsUi`

Two members, and the pairing is the point: both are *the operator's place in
this panel* rather than anything about the document.

# Why a struct rather than two fields on `PanelsState`

Because `PanelsState` hands each panel **one** accessor, deliberately — a
panel reaches its own state and cannot reach another's. Two loose fields
would need two accessors and would let a future panel take one of them by
accident. `crate::panels::pages::PagesUi`, `redact::RedactUi` and
`bookmarks::BookmarksUi` are the same shape for the same reason.
