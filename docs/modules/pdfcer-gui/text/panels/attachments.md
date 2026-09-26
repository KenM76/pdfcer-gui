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

## The three sentences that are NOT optional

Each one exists because `pdfcer-core` states an obligation in its own doc
comment and a shell that skipped it would be shipping a lie the operator
cannot detect:

| sentence | why it must be said |
|---|---|
| [`removed`] | `detach_file`: *"This is NOT a redaction verb and must not be described as one … Shells are expected to say so rather than let 'delete' imply erasure."* Under the default incremental save (§7.5.6) the bytes are still in the file, recoverable from the previous revision. |
| [`may_be_encrypted`] | `AttachmentNotes::may_be_encrypted`: since PDF 1.5 an embedded file can be encrypted **in an otherwise unencrypted document** (`/EFF` naming a `DefEmbeddedFile` crypt filter, §7.6.5), so the intuitive guard is wrong *silently* — the filter chain runs, produces bytes, and those bytes are garbage that looks like a successful extraction. |
| [`name_was_changed`] | `sanitize_attachment_name`: the name in a document is attacker-controlled and unconstrained — `..\..\Windows\System32\evil.exe`, `invoice.pdf\0.exe`, `CON.txt` are all authorable — so pdfcer writes a different file name than the row shows, and *"pdfcer renamed this file"* with no reason is the sneaky behaviour rule 4 forbids. |

## Why the size is worded as a measurement and never as a verdict

`/Params /Size` is **optional** and §7.11.4 attaches no `shall` to it, so a
document whose declaration disagrees with its bytes is not thereby
non-conforming — `pdfcer-core` records this as ambiguity **EF-A2** and its
own `DeclaredSizeCheck::Disagrees` doc says to word it *"the document says
999999 and pdfcer counted 10"*, not *"this document is broken"*. [`size`]
follows that to the letter, and [`DeclaredSizeCheck::is_contradicted`]'s
name — *contradicted*, not *invalid* — is the same decision one layer down.

## Why the dates are printed exactly as the file wrote them

`Attachment::created` and `Attachment::modified` are **raw and unparsed** by
design: `pdfcer-core` has no shared §7.9.4 date type yet and says outright
that inventing a private one *"would guarantee two parsers that disagree the
day a second caller wants dates."* So `D:20240117093000Z` is what a row
shows, and [`date_tooltip`] is where an operator finds out why — the same
decision, in the same words, that [`super::comments::comment_row_byline`]
made for `/M` on an annotation.

[`DeclaredSizeCheck::is_contradicted`]: pdfcer_core::attachments::DeclaredSizeCheck::is_contradicted

## Item notes

### `fn entry_count`

A helper rather than seven hand-written `if`s, because seven copies of a
plural rule is seven chances to ship *"1 entries"*, and that is the exact
tell this catalog's header calls out.

### `fn hazard`

`NameHazard` is `#[non_exhaustive]`, so the catch-all is required and is
deliberately the weakest claim available: *"it was not safe to use as a file
name"* is true of any hazard a later engine adds, where guessing at a
specific cause would not be.

### `fn human_bytes`

Delegates to [`super::byte_size`], which carries the argument for base-1024
arithmetic with the colloquial `KB`/`MB` labels: this figure is read by
operators comparing it against what Explorer tells them about the file they
just attached, and matching that is worth more here than matching IEC.

The cast is saturating rather than lossy: a `u64` byte count larger than
`usize` cannot be produced by a file this application can read, and
saturating is the answer that stays a number instead of wrapping to a small
one on a 32-bit build.

### `fn an_undamaged_listing_discloses_nothing`

`AttachmentNotes`' own doc states the contract — *"all-zero/false means
the listing is complete and everything parsed"* — and a
[`listing_notes`] that returned a reassurance instead of nothing would
put a permanent sentence above every ordinary document's list.

### `fn a_well_formed_document_needs_no_caveat`

The companion to [`an_undamaged_listing_discloses_nothing`], and the one
that would catch a flag pdfcer sets over-eagerly: a `Default` is a value
nobody produced, and a listing that quietly reported *"pdfcer stopped
reading early"* about every well-formed document would still pass that
test.

### `fn one_is_never_spelled_as_a_plural`

The tell this catalog's header names, checked on the panel's own first
line and on the helper every counted note goes through — which is the
point of that helper existing: seven hand-written plural rules would be
seven chances to ship *"1 entries"*.

### `fn the_removal_sentence_does_not_let_remove_imply_erasure`

`detach_file`'s doc comment makes this a shell obligation in as many
words, and the failure mode it guards against is an operator who removed
a sensitive attachment, saved, and believes it is gone. Both halves are
pinned: the fact, and the command that acts on it — a warning with no
route out is half a disclosure.

### `fn a_size_disagreement_is_reported_as_a_measurement`

§7.11.4 attaches no `shall` to `/Size` (ambiguity EF-A2), so a
disagreement is a measurement rather than a verdict. The words this
forbids are the ones that would turn one into the other.

### `fn an_unchecked_size_says_it_is_unchecked`

The case `DeclaredSizeCheck` exists for: the stream is filtered, so its
raw byte count is not its decoded byte count, and printing the
declaration bare would present an unchecked claim as a measurement.

### `fn a_renamed_save_says_both_names_and_the_reason`

The gap this bridges is structural: the listing shows the raw name
because a reader must not repair its evidence, and the filesystem gets
the safe one because the failure mode is a file written outside the
destination. Without this sentence the operator sees two different names
and is told nothing.

### `fn a_reasonless_rename_does_not_dangle`

Reachable: `SafeName::changed` is true whenever the value differs from
the input, and a future sanitiser step could change one without pushing
a hazard. The failure this pins is the dangling em dash — a sentence
that ends in punctuation waiting for a clause that never came.

### `fn a_page_row_warns_that_deleting_the_page_takes_the_file`

A `/FileAttachment` is destroyed with its page, this application can
delete a page from three surfaces, and *"On page 3"* alone would leave
an operator to discover that from a file that has lost something.
