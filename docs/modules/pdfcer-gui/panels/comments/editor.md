# `panels::comments::editor` — everything on a row that WRITES


## Why this is the seam, and not "split the rows from the strip"

R2's rule is *find the seam*, and the file genuinely comes apart here.
[`super::body`], [`super::row`], [`super::delete_control`] and
[`super::filter_strip`] are all **the list**: they decide what is shown and
in what order, and every one of them is a pure function of the document plus
a filter. What is in this file is the only part of the panel that holds the
operator's own half-finished work — a [`super::note::NoteDraft`] — and the
only part whose output is a **verb**.

⇒ Which is why the split survives the next feature rather than needing to be
redone: a control that writes will land here, a caption that describes will
land next door, and the question *"which one is this?"* has an answer that
does not depend on how many lines are left in either file.

The same seam `super::tests` took the day before, one step further along:
that file is *what is asserted about the panel*, this one is *what the panel
can change*, and [`super`] is left as the list itself.

## TWO destinations, ONE box — the thing this file exists to keep true

A note edit and a reply are the same text box pointed at different engine
verbs: `set_markup_note` edits a dictionary that already exists, `add_reply`
**creates an annotation**. Everything they share — the box, its trace
region, the Escape route, the stale-draft rule — is written once in
[`editor`], and [`super::note::DraftTarget`] is asked exactly once, at the
point where the two genuinely differ.

⚠ The failure this shape is defending against is specific and silent: a box
captioned *Post reply* whose commit writes `/Contents`, which puts a
reviewer's answer **over the comment they were answering** and looks, on
screen, exactly like the reply having worked. Nothing but reading the file
afterwards distinguishes the two, which is why the destination lives on the
draft's stamp and is compared rather than inferred.

## What is deliberately NOT here

**Delete.** `super::delete_control` writes to the document too, and it stays
next door because it is drawn on the row's *navigation* line beside *Go to*
rather than in the editor block, and because it holds no draft. The line
this file draws is not "does it change the document" — it is "does it hold
the operator's unfinished words".

## Item notes

### `fn reply_control`

# The gap this closes, in this shell's own words

2026-09-05, filed and carried open until the verb landed: *"we can read a
comment thread and cannot add to it."* The **read half already worked** —
this panel's trace has printed `replies=` since the day it was written, and
`crate::canvas::notepopup::thread` draws the conversation inside a comment's
window. What was missing was a destination, and this is it.

# Beside *Add note*, not under it, and the pairing is the explanation

The two controls do the two things a reviewer does to a comment: **change
what it says** and **say something back**. Putting them on one row makes the
choice legible at the moment it is made, which matters here more than it
usually would because the two outcomes look almost identical afterwards —
new words appear near the same annotation either way, and only the file
records which happened. `crate::text::panels::comments::comment_row_reply_tooltip`
carries the sentence that says so.

# Why it is offered on EVERY row that takes a note editor

Including rows that are themselves replies, and including rows with no
`/Contents` at all. §12.5.6.2 puts no constraint on what may be replied to,
`add_reply` refuses only a reply to *itself*, and a shape somebody drew
without a comment is exactly the thing a second reviewer most often wants
to ask about. The one exclusion is inherited rather than added: the caller
has already returned for a ce dimension and for a row with no object id, so
this is only ever reached for an annotation the engine can name.

**The parent is the ROW's own annotation**, never the thread root. The
file therefore records the real depth of the conversation even though both
of this shell's surfaces draw it flat — see [`body`]'s threading paragraph.
A shell that rewrote the parent to the root on the way in would be
destroying a fact about who answered whom, permanently, to make its own
display arithmetic simpler.

### `fn note_text`

[`Note::Description`] counts as the words. The operator is editing that
string whichever of §12.5.2's two meanings it carries, and an editor that
opened empty over a description would invite them to destroy it by typing.

One function for both jobs on purpose: the seed and the comparison have to
agree or Escape on an untouched editor writes an edit.

### `fn editor`

# The signature line is a rule-4 disclosure, not a caption

What `/T` will say is **invisible on the page** — a sticky's byline lives in
a pop-up window this shell does not draw, and a shape's lives nowhere at all
— so an operator has no way to discover what name their comments carry, or
that they carry none, or that editing somebody else's comment will leave
their name on it. Two sentences, one per case, and the case is decided by
the row rather than by a preference this panel cannot see.

# Escape writes what was typed and then closes — see [`escape_commits`]

It is detected through egui rather than by reading the keyboard: `TextEdit`
surrenders focus on Escape, so `lost_focus()` plus the key is the idiomatic
test and — importantly for this codebase — it asks nothing about whether
"the operator is typing". A panel that read the raw key would be a second
claimant on a key the canvas caret and the tool arming both want, and
`tools/gates/check-typing-guard.sh` exists because that class of second
claimant has already cost this project the Delete key and the space bar.

### `fn reply_editor`

Reached from [`editor`] when the draft's destination is
[`DraftTarget::Reply`], *after* that function has already drawn the text
box, published [`REGION_BOX`] and handled Escape. That ordering is the
whole reuse: the box, the region name, the exit key and the
stale-draft rule are written once and cannot come to differ between
writing a note and answering one.

# What differs, and every difference is a fact rather than a style

| | note editor | this |
|---|---|---|
| the verb | `set_markup_note` — edits a dictionary that exists | `add_reply` — **creates an annotation** |
| the commit's label | *Save note* | *Post reply*, because they are different acts |
| the byline | `keep_author` decides; somebody else's `/T` may be preserved | always the operator's; there is no prior byline |
| *Remove note* | offered when there is one | never — there is nothing yet to remove |
| an empty box | permitted; an empty comment is a comment | **refused**, see [`reply_is_postable`] |

# The threading disclosure, and why it is here rather than in a document

A row that is itself a reply gets [`t::comment_row_reply_to_a_reply`]. The
`/IRT` this writes is the **row's own** annotation, so the file keeps the
true depth of the conversation, while both of this shell's surfaces draw a
thread flat. That is a real difference between what is recorded and what is
shown, and rule 4 makes stating it mandatory — see [`body`]'s threading
paragraph for the argument, which is short enough to live at the code.

⇒ Drawn only on a reply row, deliberately: on an ordinary comment the
sentence would be answering a question the operator has not raised, which
is the noise every other conditional caption in this panel exists to avoid.
