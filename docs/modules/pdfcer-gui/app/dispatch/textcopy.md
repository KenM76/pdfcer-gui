# `app::dispatch::textcopy` — reading text out of the document and putting
it on the clipboard

Two ids, `file.copy_page_text` and `file.copy_document_text`, and one
subject: **the operator wants the words, somewhere else**.

## Why this is a module and not two match arms

[`super::images`]' reason: these bodies are longer than most whole tabs and
their subject is genuinely its own. It is the same seam [`super::pages`]
(ids sharing an operand rule) and [`super::images`] (one id whose body is a
sequence) are cut on — a subject, not a size.

## ★ Both read the SAME extraction, and that is the load-bearing fact

`OpenDoc::page_text()` and the document-level `extract_*_view`, which is
also what a canvas text selection copies from. Two paths to *"the text of
this page"* is how a ribbon Copy and a swept selection come to disagree
about what is on it — and they would disagree **silently**, because both
answers look like text.

## ★ Neither raises an `Action`, and that is not an oversight

A clipboard write touches no document and needs no frame boundary. It is
the same call `file.print` makes, for the same stated reason: the action
funnel exists for work that changes a document or that must not happen
mid-frame, and a copy is neither.

`crate::canvas::textsel::copy` is the one place the clipboard is written and
the one place a copy is traced, so a copy from the ribbon and a copy from a
sweep leave the same evidence.
