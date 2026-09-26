# `text::attachclip` — the words the attachment clipboard uses

Copy, Cut and Paste for an embedded file, and the one question that has to
be asked **before** the paste rather than reported after it.

## The disclosure this module exists for

`EditSession::attach_file` rebuilds the `/EmbeddedFiles` name tree by
dropping any existing entry whose key equals the new file's name and then
pushing the new one.

⇒ **A same-named attachment is REPLACED.** Not refused, not renamed, not
given a numeric suffix — the existing entry is dropped from the tree and the
new one takes its key. The bytes of the old one survive in the earlier
revision until a full rewrite, so it is recoverable, and **nothing on screen
would say it had happened**.

That is the same class as the bookmark paste's dropped destination, and it
gets the same treatment: the question is asked beside the button, while the
operator can still choose, rather than reported as an outcome. See
[`replaces_note`].

It is a **statement**, not a confirmation dialog. A destructive act must be
confirmed or clearly undoable, and a paste is one `EditSession` command and
therefore one `Ctrl+Z`. What it must not be is **silent**, which is a
different requirement and the one being met here.

## What is deliberately NOT disclosed before the press

**`AttachmentTreeUnsupported`** — a document whose `/EmbeddedFiles` root
holds `/Kids` rather than `/Names`. `attach_file` refuses it by name, and
rightly: inserting into a multi-node tree means repairing every `/Limits`
range up the chain, and getting that subtly wrong stops the document's
*existing* attachments resolving.

This shell **cannot ask in advance**. `attachments::AttachmentNotes`
reports malformed and unresolvable entries but says nothing about the
*shape* of the name tree, and nothing else in the read API exposes it. So
the refusal arrives after the press, in words, through the ordinary decline
path — honest, but one press worse than R9 wants. Filed rather than worked
around.

## Item notes

### `fn the_replacement_note_names_the_file_and_the_consequence`

Both halves. A note saying only *"a file of that name exists"* leaves the
operator to guess what pressing the button does — and the answer is the
surprising one.
