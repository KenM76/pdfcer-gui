# `pdfcer-gui-base/text/unsaved`

## Item notes

### `fn nothing_promises_a_save_this_build_cannot_do`

Both halves of this module's header, mechanised. The first is the more
important: a *Save* label here would be a claim that the open file is
written, and this build cannot write it.

### `fn every_question_says_the_open_document_is_going_away`

Asserted on the shared property rather than on the wording, because the
wording will change and the property must not: an operator who pressed
Open must be told that opening closes what they have, which is the fact
they are missing.

### `fn title`

A statement, not a question. The question is in the body and varies by
intent; a title that also asked one would put two questions on screen, and
an operator answering the wrong one is answering about their document.

### `fn question_open`

Names **the document they are leaving**, not the one they are opening.
The operator's attention is already on the file they picked; the whole
purpose of this interruption is to move it back for one sentence.

### `fn question_reread`

# It says the edits cannot survive it, because they cannot

Every other sentence here is about *leaving* a document. This one is about
**the same document coming back**, which is a distinction an operator will
draw on their own and get wrong: re-reading sounds like refreshing, and
refreshing sounds like something edits survive.

They do not. The engine's own words for why the intervention is a re-load
rather than an edit — *"a decision made during parsing is not a value that
can be edited afterwards"* — mean the document that comes back is a new
parse of the bytes on disk, with an empty undo stack. So the sentence spends
its second clause saying exactly that rather than leaving the operator to
infer it from the word *close*.

### `fn edits_at_stake`

# Why this counts EDITS and says so, rather than saying "changes"

`OpenDoc::edit_epoch` counts applied edits — one per action that reached the
document — so the number is real and is the only quantity this shell has.
Rendering it turns a contentless warning into a decision an operator can
actually make: *"1 edit"* is a misplaced click they will happily discard,
and *"48 edits"* is an afternoon.

It deliberately does **not** claim to be a count of *things on the page*.
An edit that was undone still bumped the epoch, so the number is an upper
bound on work rather than an inventory — which is why the sentence says
*"edits made"* rather than *"changes to this document"*. Overstating what a
number means is the same defect as inventing one.

### `fn save_button`

Drawn only when the document HAS a file to be written over
(`app::save::has_a_file`). A never-saved document renders no Save button at
all and keeps [`save_copy_button`] alone — R9: an unavailable capability
renders nothing, and "this document has never been written anywhere" is not
a temporary condition a hover could explain away.

# Why this did not exist until today

Because this module was written when it was TRUE that pdfcer had no Save,
and it said so in as many words: *"pdfcer cannot yet save over the file it
opened."* `file.save` landed on 2026-08-20 and this window was never
revisited, so the one prompt an operator meets when they are about to lose
work went on offering a file picker as its only way of not losing it. He
pressed the save button here, got asked for a filename, and the document
closed — which is what he reported as *"it closes the document after
saving."*

# No ellipsis, and that is the point of the pair

[`save_copy_button`]'s ellipsis promises a picker. This one promises the
opposite — a write to a destination already decided — and the two labels
have to be told apart at a glance by an operator who is one click from
discarding their work.

### `fn save_all_button`

The count is **in the label**, not implied. *"Save all"* over a modal
asking about one document is ambiguous — all of what? — and the operator is
being asked this while trying to leave. *"Save all 4"* answers the question
the button raises, in the button.

Drawn only when the count is above one, so the singular case never occurs
and is not worded for. `UnsavedDialog::body` carries that decision.

### `fn save_copy_button`

The ellipsis is doing real work: it promises a file picker, which is exactly
what happens next, and it distinguishes this from a Save that would write
somewhere already decided.

### `fn save_copy_note`

The most important sentence on the surface, and the one an operator would
otherwise have to discover by looking at their file system afterwards.
Written in two halves on purpose: what pdfcer will do, then what it will
**not** do. The second half is the part that is surprising.

### `fn save_choice_note`

Says which of the two touches the file they opened, because that is the
whole difference between them and it is not deducible from four words of
button text.

### `fn discard_reread`

*Read* rather than *Reread*, and *lose the edits* in the same words the
two buttons above use. The verb changes; the consequence clause does not,
because an operator reading only the buttons — which is most operators,
most of the time — is scanning for the consequence.

### `fn cancel_button`

Last, and named *Cancel* rather than *Go back* or *Keep editing*, because it
is the one label in this window an operator does not have to read to
understand. Every convention they have is on its side; spending novelty here
would buy nothing and cost the one word they can recognise at a glance.
