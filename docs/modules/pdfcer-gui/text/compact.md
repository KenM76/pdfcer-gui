# `text::compact` — what the Save-a-compacted-copy window says before it
throws anything away

The copy for [`crate::dialogs::compact`].

## Why this command exists, and it is the operator's own request

`OPERATOR_REQUESTS.md` **O48**, answered *"yes to all three"* on 2026-08-28.
It was raised by this project rather than by him, from a limit found while
wiring Remove-embedded-fonts:


§7.5.6's update section is *appended*. Deleted objects get free
cross-reference entries in a **new** section and their bytes remain in the
prior revision, which is still in the file. So every space-reclaiming
operation pdfcer has — unembedding a font, deleting a page, deleting an image
— produces a file that is very slightly **larger**.

Only a full rewrite drops the bytes, and until now this shell had no way to
ask for one.

## Why it is a SEPARATE command and not a better Save

Because incremental saving is not a limitation pdfcer is working around — it
is a promise it makes, on an operator-visible surface, and has since the day
Save-a-copy was registered:

> *"…the edits are appended as an update so the previous version stays intact
> inside the file."*


⇒ So the whole design is: a second command, named so it cannot be pressed by
accident, **always to a new file**, with what it discards said plainly
before the picker opens.

## Every sentence here is about a LOSS, and that is deliberate

The gain — a smaller file — is why the operator pressed it and needs no
advocacy. What they cannot see is what is being dropped: a revision history
they may never have known was there, and signatures whose invalidation shows
up in somebody else's reader rather than in this one. Rule 4's surviving
half, applied to a save.

## Item notes

### `fn a_tidy_file_is_told_it_is_tidy`

The failure this guards is *"this will save 0 KB"*, which is accurate and
reads as the feature failing. It is also the case a rewrite can come out
LARGER — §7.5.4's table pays for gaps in object numbering — so a sentence
built from a subtraction would underflow or print a negative saving.

### `fn the_signature_warning_says_it_is_irreversible`

Every other disclosure in this shell describes something an operator can
undo or redo. This one cannot, and the word that says so is the whole
difference between a warning and a note.

### `fn intro`

It leads with the **mechanism**, not the benefit, because the mechanism is
the part that explains every consequence below it. An operator who
understands *"pdfcer normally adds to the end of the file"* can predict what a
rewrite costs; one told only *"this makes your file smaller"* cannot.

### `fn size_change`

A **measurement of this document**, taken by writing it — not an
estimate. The window computes the compacted bytes before it opens, because a
prediction that turned out wrong on the operator's own file would be worse
than saying nothing: they would have accepted the losses below for a saving
that did not arrive.

The **no-saving** case is a real outcome and gets its own sentence. A file
that has never been edited, or one whose deletions were all in the current
revision anyway, has nothing to reclaim — and *"this will save 0 KB"* reads
as a failure when it is an accurate answer about a tidy file.

### `fn revisions_line`

Said to everybody, because everybody loses it and almost nobody knows it
was there. An incremental save leaves the file's earlier state recoverable
from inside the file itself; a rewrite is the moment that stops being true,
and it is the only moment at which saying so is any use.

### `fn signature_line`

The loudest sentence in this window, and the only one that is
**conditional**, for `text::unembed`'s reason: a warning about signatures on
every unsigned drawing is noise that teaches an operator to skip the window.

It says *"cannot be repaired"*, which is the part that distinguishes this
from every other loss pdfcer discloses. A signature covers a byte range
(§12.8.1); rewriting the file moves everything, and no later save puts it
back. The operator is being asked to accept something irreversible and is
entitled to be told that in the word that means it.

### `fn save_button`

It names the **act**, not "OK". The operator has just read three sentences
about losses, and a button reading `OK` after that asks them to agree to a
question rather than to perform a thing they chose.

### `fn refused`

A real refusal with a named cause, not a fallback. `pdfcer-core` refuses
a full rewrite by name — since `Pass 281.0` for a **hybrid file whose
`/XRefStm` does not parse**, not for the whole hybrid class this line
claimed until 2026-09-11 — and points at incremental as the supported
path. `app::save`'s header states the rule this obeys:
*"if a future change finds incremental genuinely impossible for some input,
the honest response is to refuse and say so, not to fall back."* This is that
rule read in the other direction, and quietly writing an incremental copy
here would give the operator a file that is not what the command promised.

### `fn write_failed`

The reason is passed through verbatim, on `export_dxf::export_failed`'s
stated rule: *"access is denied"* and *"the device is not ready"* are
different problems with different remedies, and a shell that collapsed them
to *"could not save"* would leave the operator with the one fact they cannot
derive.
