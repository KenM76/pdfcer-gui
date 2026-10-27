# `app::dispatch::textcopy` — reading text out of the document and putting
it on the clipboard

Two ids, `file.copy_page_text` and `file.copy_document_text`, and one
subject: **the operator wants the words, somewhere else**.

## Why this is a module and not two match arms

[`super::images`]' reason: these bodies are longer than most whole tabs and
their subject is genuinely its own. It is the same seam [`super::pages`]
(ids sharing an operand rule) and [`super::images`] (one id whose body is a
sequence) are cut on — a subject, not a size.

## Both read the SAME extraction, and that is the load-bearing fact

Both read the page's provenance extraction, the one the reflow editor's model
is built from, through `clipboard::richtext`. A canvas text selection copies
from that extraction too. Two paths to *"the text of this page"* is how a
ribbon Copy and a swept selection come to disagree about what is on it, and
they would disagree **silently**, because both answers look like text.

## Neither raises an `Action`, and that is not an oversight

A clipboard write touches no document and needs no frame boundary. It is
the same call `file.print` makes, for the same stated reason: the action
funnel exists for work that changes a document or that must not happen
mid-frame, and a copy is neither.

`crate::canvas::textsel::copy_rich` is the one place formatted text is placed
and traced, so a copy from the ribbon and a copy from a sweep leave the same
evidence: `text-copied source=… chars=… paragraphs=… formats=…`.

## Item notes

### `fn handles`

The same shape [`super::pages::handles`] uses, and for the same reason: the
`match` in `super` stays a list a reader can scan, and the routing predicate
lives beside the bodies it routes to — so a new verb here is one edit rather
than two.

### `fn dispatch`

Takes the whole application rather than a `&Status`, because both bodies
read caches that live on the open document.
