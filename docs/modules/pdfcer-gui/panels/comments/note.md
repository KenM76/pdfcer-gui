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
