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
