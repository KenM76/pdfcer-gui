# `text::acrobat` — every word O122 puts on screen

`OPERATOR_REQUESTS.md` **O122**: the *Open in Acrobat* control beside the
Read / Review / Edit selector, the three things it can say before it acts,
and the Settings field that says where Acrobat is.

## Why one module for four surfaces

The ribbon command, three dialogs and a Settings group would normally be
four places in this catalog. They are one here because they are **one
conversation**: the button promises something, the dialog says what that
costs, and the Settings field is where the promise comes from. A wording
change to any one of them is nearly always a wording change to another, and
catalog entries that must move together are entries that should be read
together.

## The sentence this whole module is written around

> **The file will be closed.**

That is the operator's own instruction — O122 point 6, *"it will note the
file will be closed when opened in acrobat"* — and it is the fact every
string here has to carry without burying. Acrobat takes its own lock on the
file it opens; two editors on one PDF is how an afternoon's work
disappears; so pdfcer gives the file up rather than keeping it open
alongside. The operator is told **before** it happens, because closing
somebody's document is not a thing to do quietly.

## The button that is not offered, and the reason it is not

There is no *"open without saving"*. [`crate::dialogs::unsaved`] offers
exactly that shape — *Save · Don't save · Cancel* — and is right to, because
there the document is merely being closed. Here it is being closed **and
handed to a program that will happily save over it**, so the discarded
edits would not simply be lost: they would be lost, and then overwritten by
a version of the file that never had them, in a program the operator
believes is showing them their work.

So the wording never implies a third answer exists. [`save_first_body`]
says what will be saved; it does not ask whether to save.

## Vocabulary

- **Acrobat**, never *"Adobe Acrobat"* in a button and never *"the external
  viewer"*. It is a proper name the operator uses, and a generic phrase in
  its place reads as a program that is not sure what it found.
- **The document**, not *"the PDF"* — the same choice the rest of this
  catalog makes.
- The edition is named **only where it is useful**: on hover, where there
  is room to say *Acrobat Pro* or *Acrobat Reader* and it tells the
  operator which of their two installations is about to open. The button
  itself says *Acrobat*, because a label that changed between machines is a
  label nobody can be told to press.

## Item notes

### `fn both_confirmations_say_the_document_will_be_closed`

O122 point 6 in checkable form. The two dialogs that lead to a handover
must both carry the fact; the third must not, because nothing is being
closed there and saying so would be false.

### `fn nothing_offers_to_open_without_saving`

The module header's argument, mechanised. A future edit that adds a
*Don't save* button to this dialog — reasonably, by analogy with
[`crate::dialogs::unsaved`] — has to delete this test first, which is
where they will read why it is there.

### `fn a_missing_file_does_not_read_as_a_missing_acrobat`

The one that matters is that *"no file on disk"* does not read as
*"Acrobat is missing"*: they have different remedies, and an operator
sent to look for an installer they already have will conclude the
feature does not work.

### `fn file_open_in_acrobat`

**No ellipsis**, deliberately, and it is a close call. This crate's
convention is that an ellipsis means *"activating this opens a dialog
rather than acting"*, and this control always raises a dialog. But the
dialog is a **confirmation of the thing the label names**, not a request
for information the label left out — the difference between *Settings…*,
which cannot act until you tell it what to change, and a Close button that
asks whether you meant it. An ellipsis here would suggest there is more to
decide than *yes* or *no*.

The tooltip states the closing up front rather than at the end. It is
the surprising half, and a hover sentence is read from the left until it
stops being interesting.

### `fn no_document_tooltip`

R9's second half: *greying is reserved for temporarily unavailable and is
**always explained on hover***. "No document" is exactly that, and this is
the explanation.

### `fn title`

One title rather than three. The window is the same window answering the
same request; changing its name according to which sentence is inside it
would make a taskbar entry that renames itself while the operator is
looking away.

### `fn confirm_close_body`

It says **why**, in one clause, and the why is the part that makes the
behaviour reasonable rather than officious. An operator told only *"it will
be closed"* reads a program being awkward; told *"because two programs
editing one file is how work gets lost"*, they read a program looking after
their drawing.

### `fn save_first_body`

It names the number of edits and it names the **file**, because those
are the two things that make the choice concrete. And it closes on the
consequence of *not* saving, phrased as a fact rather than as an option:
there is no button for that outcome and the sentence must not read as if
there were.

### `fn no_file_heading`

A statement, not a question. There is nothing to decide: this is the one
of the three that offers no way forward, and a heading shaped like a
question would promise one.

### `fn no_file_body`

It says what to do — *save it somewhere first* — because a refusal that
only refuses leaves the operator to guess, and the guess most people make
is that the button is broken. And it is careful **not** to say Acrobat is
missing: that is a different refusal with a different remedy, and confusing
the two sends somebody looking for an installer they already have.

### `fn save_and_open_button`

Named for **what it does**, never *Yes* or *OK*, which is this crate's
standing rule for a button whose press has consequences: an operator who
reads only the buttons — which is most operators, most of the time — must
still get it right.

### `fn close_and_open_button`

Also named for what it does, and *not* "OK" — even though the operator's
own words were *"with and ok button to continue"*. What he asked for is a
confirm-and-proceed control, which this is; what "OK" would cost is the
sentence that says the document is closing being the only place that fact
appears, and a dismissed dialog is one nobody read.

### `fn launch_failed`

A real state and the one worth wording most carefully: the operator's work
is safe on disk, and the only thing that did not happen is the handover.
Saying that plainly is the difference between a nuisance and a panic.

### `fn save_failed`

Nothing else follows a failed save. The document is not closed and
Acrobat is not started, because the whole point of the question was that
the edits must survive the handover.

### `fn path_note`

This is the string O122's decision hangs on: *"the path control lives in
Settings and is visible there whether or not discovery succeeded, so a
non-standard install is fixable without the button ever having appeared."*
A person whose Acrobat is somewhere unusual arrives at this field having
seen **no button at all**, so the note has to explain a control they have
never met.

### `fn resolved_note`

The half of the escape hatch that makes it usable rather than merely
present. Without this line a person who typed a path with a letter missing
sees exactly what a person who typed it correctly sees — a filled-in field
and no button — and has no way to tell the two apart. With it, the mistake
is visible at the place it was made.

### `fn edition_name`

Not `Debug`, and not a bare "Acrobat": which of the two is installed is a
thing the operator knows about their own machine, and naming it is how they
confirm that pdfcer found the one they meant. Somebody with both installed
who sees *Acrobat Reader* here has been told about a misconfiguration they
could not otherwise have detected.
