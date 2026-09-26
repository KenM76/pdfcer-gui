# `panels::attachments::attach` — putting a file into the document

## The gap this closes

`EditSession::attach_file` and `EditSession::detach_file` have existed in
`pdfcer-core` with **no GUI surface of any kind** — not a command, not a
panel, not a menu item. This row is the writing half of the surface that
closes it, and `super` is the reading half.

## Why this row is drawn ABOVE the list, and it is not a style choice

`panels::bookmarks` paid for this lesson in a driven run and wrote it down
as a rule; this module obeys the rule rather than rediscovering it:

> A driven run on a 122-bookmark drawing found the panel body occupying
> y=133..770 and this row laid out at y=899..923 — **below the bottom of the
> panel**, with no way to reach it. The row drew. It published its region.
> Every unit test passed. […] **A control that must always be reachable
> cannot be placed after an unbounded `ScrollArea`.** Reserve-and-hope is
> not a second option; it is the same defect with a tuning parameter.

It also reads correctly there. Acrobat's Attachments panel puts its add
control in a toolbar **above** the list for the same reason every list in
every application does: a control's position is a claim about what it acts
on, and this one acts on the document that owns the list.

## The description can only be set now, and the row says so

`attach_file` takes `description: Option<&str>` and writes `/Desc` on the
file specification (Table 44, whose row for it says `/Desc` *"shall be used
for files in the `EmbeddedFiles` name tree"* — precisely this route).
`pdfcer-core` exposes **no verb that edits one afterwards**.

So there is no *Edit description* control anywhere in this panel — R9: an
absent capability renders nothing, and a greyed one would be a promise no
state of the program could keep. But the *limit* is disclosed in the row,
because "you cannot change this later" is a fact an operator needs
**before** they leave the box empty, and R9 has never said a missing verb
must also be a secret.

## Why the picker is not opened here

A native file dialog is a modal OS window that blocks the thread. Opening
one from this `clicked()` branch would leave egui part-way through a frame
that will not finish until the operator has answered — `app::actions::write`
calls that *"the sharpest seam this enum has"*. The button raises
[`AttachmentAction::Attach`]; `PdfcerApp::apply` opens the picker, in step 3,
after every panel and dialog has closed.

That is also what gives the feature a **driver**:
`crate::app::files::DIAG_ATTACH_PATH` answers the dialog without a human,
and no synthetic input reaches a native dialog otherwise.

## Item notes

### `fn a_blank_description_becomes_no_description_at_all`

The expression under test is the one the button arm uses, spelled the
same way, so the two cannot come apart. What it defends: writing
`Some("   ")` would put a `/Desc` key holding blanks into the file — a
key a later reader has to interpret and that no operator asked for —
while the row that shows it would appear to have a description and show
nothing.

### `fn the_two_regions_are_named_apart`

A driven check clicks a region by name; two controls sharing one would
make the harness click whichever was published last, and the failure
would present as *"the button does nothing"* on whichever run lost.
