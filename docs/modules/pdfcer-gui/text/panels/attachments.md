# `text::panels::attachments` — every string the Attachments panel shows

One area of the catalog described in [`crate::text`]'s header, covering
[`crate::panels::attachments`] and the three apply arms in
[`crate::app::actions::attachments`] that report what those verbs did.

## What this surface is about, in one paragraph

A PDF may carry **whole files inside itself** — ISO 32000-1 §7.11.4.1
*embedded file streams*, reached either from the catalogue's
`/Names /EmbeddedFiles` name tree (document-level, belongs to the file) or
from a `/FileAttachment` annotation on one page (§12.5.6.15, pinned to a
rectangle and destroyed when that page is deleted). Neither kind is visible
anywhere on the page. That single fact decides most of the wording below:
**every sentence here is a disclosure**, because there is nothing on the
canvas an operator could have looked at instead.

## ★★★ The three sentences that are NOT optional

Each one exists because `pdfcer-core` states an obligation in its own doc
comment and a shell that skipped it would be shipping a lie the operator
cannot detect:

| sentence | why it must be said |
|---|---|
| [`removed`] | `detach_file`: *"This is NOT a redaction verb and must not be described as one … Shells are expected to say so rather than let 'delete' imply erasure."* Under the default incremental save (§7.5.6) the bytes are still in the file, recoverable from the previous revision. |
| [`may_be_encrypted`] | `AttachmentNotes::may_be_encrypted`: since PDF 1.5 an embedded file can be encrypted **in an otherwise unencrypted document** (`/EFF` naming a `DefEmbeddedFile` crypt filter, §7.6.5), so the intuitive guard is wrong *silently* — the filter chain runs, produces bytes, and those bytes are garbage that looks like a successful extraction. |
| [`name_was_changed`] | `sanitize_attachment_name`: the name in a document is attacker-controlled and unconstrained — `..\..\Windows\System32\evil.exe`, `invoice.pdf\0.exe`, `CON.txt` are all authorable — so pdfcer writes a different file name than the row shows, and *"pdfcer renamed this file"* with no reason is the sneaky behaviour rule 4 forbids. |

## ★ Why the size is worded as a measurement and never as a verdict

`/Params /Size` is **optional** and §7.11.4 attaches no `shall` to it, so a
document whose declaration disagrees with its bytes is not thereby
non-conforming — `pdfcer-core` records this as ambiguity **EF-A2** and its
own `DeclaredSizeCheck::Disagrees` doc says to word it *"the document says
999999 and pdfcer counted 10"*, not *"this document is broken"*. [`size`]
follows that to the letter, and [`DeclaredSizeCheck::is_contradicted`]'s
name — *contradicted*, not *invalid* — is the same decision one layer down.

## ★ Why the dates are printed exactly as the file wrote them

`Attachment::created` and `Attachment::modified` are **raw and unparsed** by
design: `pdfcer-core` has no shared §7.9.4 date type yet and says outright
that inventing a private one *"would guarantee two parsers that disagree the
day a second caller wants dates."* So `D:20240117093000Z` is what a row
shows, and [`date_tooltip`] is where an operator finds out why — the same
decision, in the same words, that [`super::comments::comment_row_byline`]
made for `/M` on an annotation.

[`DeclaredSizeCheck::is_contradicted`]: pdfcer_core::attachments::DeclaredSizeCheck::is_contradicted
