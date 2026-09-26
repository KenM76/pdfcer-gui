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

### `fn count`

Singular is spelled out rather than reached by a plural rule, matching
[`super::bookmarks_count`]'s shape: *"1 attached files"* is the tell that a
program is filling in a template.

### `fn empty`

**Worded as a fact about the document, not as an absence of a feature.**
The overwhelming majority of PDFs have no attachments and are perfectly
ordinary; an operator reading this must not be left wondering whether pdfcer
failed to look.

### `fn unnamed`

`NameSource::None` is reachable — a filespec may carry no `/F`, `/UF`,
`/DOS`, `/Mac` or `/Unix`, and a page annotation has no name-tree key to
fall back on — and the row must still exist, because the operator can still
save the bytes out. A blank line where a name belongs reads as a rendering
fault.

### `fn where_document`

The distinction this states is the one `pdfcer_core::attachments`' module
docs say *"bites hardest at save time and at page-delete time"*: this kind
belongs to the document and survives the deletion of every page.

### `fn where_page`

The clause about page deletion is the whole reason this string is not
simply *"On page 3"*. A `/FileAttachment` annotation (§12.5.6.15) is
**destroyed when its page is deleted**, and this application can delete a
page from three different surfaces. An operator who has been told is one
who can decide; one who has not finds out from a file that used to have
their supplier's spreadsheet in it.

### `fn kind_claimed`

*"claims"* is load-bearing and is not softened. `/Subtype` on an embedded
file stream is a **claim by the document about its own payload, never a
measurement** — `pdfcer-core` does not sniff the bytes, and `/text#2Fplain`
on a Windows executable is trivially authorable. A caller that presented
this as a safety signal would be turning an unverified assertion into an
assurance, which is exactly the shape of the mistake that gets somebody to
double-click.

### `fn size`

# Four different sentences, because there are four different facts

Collapsing them would put a number on screen with no way to tell an
agreed measurement from an unchecked declaration — and it is the *third*
case that makes the collapse dishonest rather than merely lossy:

| state | what pdfcer actually knows |
|---|---|
| `NotDeclared` | the document said nothing. §7.11.4 makes `/Size` optional, so this is ordinary. |
| `NoStream` | there are no bytes at all — an external file reference (§7.11.3), legal and not extractable. |
| `Unverified` | a size was declared and the stream is **filtered**, so its raw byte count is not its decoded byte count. Comparing them would manufacture a false verdict in both directions. |
| `Agrees` / `Disagrees` | pdfcer counted. Only here is a comparison honest. |

The `Disagrees` wording states both numbers and passes no judgment; see
this module's header for why that is a requirement rather than a courtesy.

### `fn dates`

Printed verbatim. See the module header: `pdfcer-core` stores these raw
because it has no shared §7.9.4 date type, and a parser written here would
be a second one that disagrees with whichever is written next.

### `fn date_tooltip`

On hover rather than on the row, exactly as
[`super::comments::comment_row_modified_tooltip`] is and for its reason: it
answers a question most operators will never ask, and the ordinary value is
legible enough to compare two rows by.

### `fn name_is_approximate`

`Attachment::name_exact` is `false` when decoding needed at least one
U+FFFD substitution — an undefined PDFDocEncoding code, an odd trailing byte
after a UTF-16BE BOM, an unpaired surrogate. That is **pdfcer's own
lossiness**, and rule 4 requires disclosing it exactly as much as it
requires disclosing an inference about the document.

### `fn name_is_the_index_key`

A name-tree key is **not** a filename and has no declared encoding.
Table 31 describes `/EmbeddedFiles` as mapping name strings to file
specifications and stops there — the sibling `/Renditions` row in the same
table *does* require Unicode, so the omission is deliberate — and §7.9.6
says outright that *"any encoding of the keys may be used as long as it is
self-consistent"*. Producers routinely mangle these with numeric suffixes
and portfolio folder prefixes, so a key shown as a name is a guess twice
over, and this sentence is how both are disclosed.

### `fn no_bytes`

**Not necessarily a defect.** §7.11.3 file specifications also describe
*external* files, which legitimately have nothing embedded — so this is
worded as a fact about what the row can do, not as damage.

### `fn broken_stream`

Distinct from [`no_bytes`], and the distinction is the whole reason both
exist: `AttachmentNotes::unresolvable_streams` is documented as *"always a
defect"* — an `/EF` entry that exists and does not resolve to a stream —
while `filespecs_without_stream` is ordinary. One sentence for both would
either call a legal document damaged or let real damage pass unremarked.

### `fn may_be_encrypted`

See this module's header for why over-warning is the correct error here:
the flag is set from the presence of `/Encrypt` alone, which is cheap and
deliberately over-broad, and the failure it guards against is *silent* —
a successful-looking extraction of garbage.

### `fn listing_notes`

# Why this is a function over the whole struct rather than a string per flag

Because the panel must show **all** of them, and the failure mode of a
string-per-flag catalog is a caller that renders four of the seven. The
disclosure obligation here is not per-flag; it is *"is this list
complete?"*, and that question has one answer assembled from the whole
struct. Written as a pure function so [`tests`] can hold it to that without
a `Ui`.

An all-default `AttachmentNotes` returns an **empty vector**, which is the
property the panel relies on to draw nothing: *"all-zero/false means the
listing is complete and everything parsed."*

`page_tree_unwalkable` is reported even though the document-level list is
still complete, because the operator cannot tell the difference between
*"there are no page attachments"* and *"pdfcer could not go and look"* — and
those are the two answers that matter when a file has gone missing.

### `fn attach_description_hint`

*"optional"* is in the hint rather than in a sentence beside it, because
it is the answer to the only question the field raises and an operator who
reads it in the box has been answered before they wonder.

### `fn attach_description_note`

The second half is a **capability disclosure**, not a nicety.
`EditSession::attach_file` takes the description at attach time and
`pdfcer-core` has no verb that edits one afterwards, so an operator who
leaves the box empty has made a decision they cannot revisit without
removing the file and attaching it again. R9 forbids drawing a control for
the edit that does not exist; it does not forbid saying so.

### `fn attached`

Three clauses, and each one is a thing the operator has no other way to
learn:

1. **the file is embedded, and a copy** — the original is untouched, which
   is the first thing anybody wonders and the thing that decides whether
   they go and delete it;
2. **it is not on any page** — a document-level attachment (§7.11.4.1
   route 2) appears nowhere in the rendering, so an operator looking for a
   visual confirmation will not find one and must not conclude the attach
   failed;
3. **the document has grown** — the bytes are now inside the PDF, and on a
   large attachment that is the difference between a file that emails and
   one that does not.

### `fn attach_refused_multi_node_tree`

# Why this refusal is surfaced and the other three are not

`attach_file` refuses four ways. Three of them —
`DocumentEncrypted`, the certification gate and
`ObjectCreationWouldExposeHiddenObjects` — are properties of the *document*
that every other authoring verb in this shell shares, and this shell's
settled answer for those is `super::super::apply::vector_edit`'s trace: they
are conditions an operator cannot fix from this panel, and wording them here
would put four sentences in the one status slot for states the Attachments
panel did not create.

`AttachmentTreeUnsupported` is different in kind, and that is the whole
argument for this string. It is **specific to this feature**, it is
**unreachable from any other surface**, and — the part that matters — the
press produces *nothing at all*: no row appears, no error appears, and an
operator has no way to distinguish it from a button that is broken.

It is worded as a limit of pdfcer rather than as a fault in the file, because
that is what it is: a `/Kids` name tree is entirely legal (§7.9.6), and the
engine's refusal is a refusal to risk *"a document whose EXISTING
attachments stop resolving"* by guessing at a `/Limits` repair.

### `fn attach_source_unreadable`

The detail is the operating system's own message, passed through: it names
the file and says whether it was a permission, a lock or a missing path,
and none of those is a distinction this catalog could redraw better.

### `fn remove_tooltip`

It names the **three objects** that go, because *"remove the row"* is what
a careless implementation would do and it is the worst possible outcome:
`detach_file`'s own docs say that removing only the tree entry leaves *"the
bytes in the file with nothing pointing at them: invisible to every reader,
still fully present on disk."* Saying what pdfcer does is how an operator can
tell this implementation from that one.

### `fn remove_lives_with_the_note`

R9 says an absent capability renders nothing, and this is the sentence that
makes the absence legible rather than mysterious. It is not that pdfcer
cannot remove one; it is that a `/FileAttachment` is an **annotation**, is
listed in the Comments panel as one, and is removed as one —
`EditSession::detach_file` answers `AttachmentNotFound` for it by name,
precisely so a shell can say which of the two kinds the operator is looking
at.

### `fn removed`

# This sentence is required by `pdfcer-core`, in its own words

> *"This is NOT a redaction verb and must not be described as one. If the
> attachment was sensitive, the operator needs a full rewrite … Shells are
> expected to say so rather than let 'delete' imply erasure."*


And the second sentence **names the command that does it**. A disclosure
that states a hazard and leaves the operator to find the remedy has done
half the job; `file.save_compacted` is the full rewrite, it is on File ▸
Save, and it is one control away.

### `fn save_button`

*"Save a copy"* rather than *"Extract"*: extraction is the engine's word for
decoding a stream, and an operator's word for what this does is saving a
copy. Nothing is taken out of the document.

### `fn save_tooltip`

The second clause is the disclosure the first invites: the bytes came from
inside a file that arrived from somewhere, and `pdfcer-core`'s own module
docs say it *"does not execute, open, or interpret them, and neither should
a caller without its own gate."* pdfcer writes the file and stops; opening it
is the operator's decision, and they should make it knowing that the
document's declared type is a claim rather than a check.

### `fn name_was_changed`

# Why this is a required disclosure and not a nicety

`sanitize_attachment_name`'s own docs record the design choice this string
completes. pdfcer reports the **raw** name in the listing, because *"a
forensic reader that quietly repairs its input is not a reader"* — the
operator investigating a suspicious file must see the traversal that made it
suspicious. And pdfcer refuses to *use* that raw name on a filesystem,
because the failure mode is silent, remote and severe.

Between those two correct decisions sits a gap: the row says one thing and
the file on disk is called another. This sentence is the bridge, and
`SafeName::hazards` exists — sorted and deduplicated, *"so a message can
list them deterministically"* — for exactly this call.

The hazard names are translated to plain English rather than printed. An
operator seeing *"ParentTraversal"* has been shown a Rust identifier; one
seeing *"it tried to climb out of the folder you chose"* has been told what
happened to them.

### `fn extract_failed`

The detail is `AttachmentError`'s own `Display`, which distinguishes the
four causes the engine went to the trouble of separating — an external
reference, a missing stream, an unservable span, and a filter chain that
failed or blew the decompression-bomb ceiling. Re-wording them here would
be a second vocabulary for facts the engine already states precisely.

### `fn gone`

Reachable, and by the ordinary route rather than an exotic one: the queue
drains **after** the frame, so an undo or a second removal raised earlier in
the same frame can take the row away before this action is applied.
Declining with a sentence beats declining in silence, and both beat acting
on whatever moved into its place.
