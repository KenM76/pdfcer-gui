# `text::compact` — what the Save-a-compacted-copy window says before it
throws anything away

The copy for [`crate::dialogs::compact`].

## ★★★ Why this command exists, and it is the operator's own request

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

## ★★★ Why it is a SEPARATE command and not a better Save

Because incremental saving is not a limitation pdfcer is working around — it
is a promise it makes, on an operator-visible surface, and has since the day
Save-a-copy was registered:

> *"…the edits are appended as an update so the previous version stays intact
> inside the file."*


⇒ So the whole design is: a second command, named so it cannot be pressed by
accident, **always to a new file**, with what it discards said plainly
before the picker opens.

## ★★ Every sentence here is about a LOSS, and that is deliberate

The gain — a smaller file — is why the operator pressed it and needs no
advocacy. What they cannot see is what is being dropped: a revision history
they may never have known was there, and signatures whose invalidation shows
up in somebody else's reader rather than in this one. Rule 4's surviving
half, applied to a save.
