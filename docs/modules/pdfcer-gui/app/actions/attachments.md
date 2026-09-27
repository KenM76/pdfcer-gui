# `app::actions::attachments` — the three verbs whose subject is a whole
FILE living inside the document

A sub-enum rather than three variants of [`super::Action`].
`super`'s declaration of `action` states the rule that puts them here —
*"the next family of variants to **grow** is the one that will have to
become a sub-enum"* — and a family of three verbs has grown before it is
written.

## What makes these a family, and it is not "they are all about
attachments"

A subject label would be the weak answer, and this enum has a structural one
that no other family in the vocabulary shares:

> **Every verb here operates on bytes that are in no page's content stream,
> and two of the three are file-system operations that happen to touch a
> PDF.**

Three consequences follow from that one property, and the whole module is
shaped by them:

1. **Every one of them opens a native file dialog, and therefore must be an
   `Action`.** `super::write`'s header states the rule in the sharpest form
   this project has written it: *"A native file dialog must not open inside
   a layout pass. It is a modal OS window that blocks the thread, so opening
   one from a widget's `clicked()` branch leaves egui part-way through a
   frame that will not finish until the operator has answered."*
   [`AttachmentAction::Attach`] and [`AttachmentAction::SaveCopy`] are
   `Action`s for **both** reasons — the funnel's invariant *and* the
   dialog's timing — where `WriteAction`'s three are `Action`s for the
   second alone.
2. **Nothing they do is visible on the canvas.** A document-level
   attachment (§7.11.4.1 route 2) is reached from the catalogue's
   `/Names /EmbeddedFiles` and appears in no rendering, so *every* one of
   these verbs owes a sentence to `crate::app::status`. Contrast
   `super::bookmarks`, where a rename is deliberately silent because the row
   the operator is looking at now reads the new name. There is no such row
   here: the panel is the only witness, and the panel is where the operator
   already is, so the disclosure carries the part the panel cannot show —
   what happened to the *file*.
3. **The operand cannot be an index and cannot be an `ObjId` either.** See
   [`AttachmentRef`], which is the interesting type in this module.

## The one refusal this module surfaces, and the three it does not

`EditSession::attach_file` refuses four ways. Three of them —
`DocumentEncrypted`, the certification gate, and
`ObjectCreationWouldExposeHiddenObjects` — are properties of the **document**
that every authoring verb in this shell shares, and this shell's settled
answer for them is [`super::apply::vector_edit`]'s trace. That is not
neglect: they are conditions no control on this panel created and none can
clear, and four sentences competing for the one status slot would evict the
disclosures the successful verbs owe.

`AttachmentTreeUnsupported` is surfaced, and the argument is at
[`crate::text::panels::attachments::attach_refused_multi_node_tree`]. In one
line: it is unreachable from any other surface, and the press otherwise
produces **nothing at all** — no row, no message — which an operator cannot
distinguish from a broken button.

## What is deliberately NOT here

**Editing a description.** `attach_file` takes it at attach time and
`pdfcer-core` has no verb that changes one afterwards, so there is no
variant, no field and no control. R9: an absent capability renders nothing.
The panel *says* so — see `attach_description_note` — because a control that
cannot exist is different from a limit that must not be discovered by
trying.

**Removing a page-level file attachment.** `detach_file` addresses the
`/EmbeddedFiles` name tree and answers `AttachmentNotFound` for a
`/FileAttachment` annotation **by name**, precisely so a shell can tell the
two apart; those are removed with `delete_annotation`, which is
`super::annot::AnnotAction::Delete`'s business and reached from the Comments
panel. Wiring a second route to it from here would give one act two
implementations, and the second would be the one that forgot the page
invalidation.

## Item notes

### `fn attach`

# The order of operations, and why the picker is first

Opposite to `super::export::dxf`, which writes first and asks second. That
verb can do it because its write *cannot fail*; this one has nothing to
produce until the operator has named a file, so the picker is the first
step by necessity rather than by choice.

The read is second and the mutation third, which does matter: a file the
operator picked and pdfcer cannot read must decline **before** the session is
touched, so a failed attach leaves no undo entry to step past.

# The name written into the PDF is the file's own base name

Not the full path. §7.11.2.1 says a file-specification string's bytes
*"shall be passed to the operating system without interpretation"*, so
writing `D:\quotes\2026\supplier.xlsx` into `/F` would produce a document
that names a location on the machine that made it — a small privacy leak in
every copy of the file, and a name that means nothing to anyone else.
Acrobat writes the base name; so does this.

`FALLBACK_SAFE_NAME` covers the case the OS admits and nobody expects: a
path with no final component. It is the engine's own constant rather than a
literal here, so the fallback pdfcer *writes* and the fallback pdfcer
*substitutes when saving one out* cannot drift apart.

### `fn detach`

# The disclosure is the point of this function


`crate::text::panels::attachments::removed` carries that sentence and names
the remedy — `file.save_compacted`, the full rewrite — because a disclosure
that states a hazard and leaves the operator to find the way out has done
half the job.

# What it cannot be asked to do

A page-level file attachment. `detach_file` answers `AttachmentNotFound` for
one **by name**, and the panel does not offer a Remove control on those rows
at all, so the refusal is unreachable from this surface rather than routed
around. Recorded here so nobody adds a guard for a case that cannot occur.

### `fn save_copy`

# The listing and the extraction happen in one breath, and that is a
contract rather than a style

`extract_attachment`'s doc comment is explicit:

> *"An `Attachment` carries object ids, and an id only means something
> relative to a document. Passing a view of a different document will either
> fail … or, if the other document happens to have a stream at the same id,
> return **that** document's bytes. pdfcer cannot detect the confusion …
> Listing from `doc` and extracting through `doc.view()` in the same breath
> … makes it a non-issue."*

So the panel does not carry an `Attachment`, and this function does not
cache one. It re-lists, resolves the operand it was given, and extracts,
all against one borrow of one session.

# The name is sanitised before it touches the filesystem

[`Attachment::name`] is **attacker-controlled text** and nothing in
ISO 32000-1 constrains it: `..\..\..\Windows\System32\evil.exe`,
`/etc/cron.d/pwn`, `report.pdf\0.exe` and `CON.txt` are all authorable, and
§7.9.6 says even less about a name-tree key. `Attachment::safe_name` exists
*"so the **safe** call is the short one"*, and this is the extraction path
its docs say should reach for it.

And the sanitiser's answer is **reported**, not merely used. The listing
shows the raw name — because a reader that quietly repairs its evidence is
not a reader — so the row and the file on disk can legitimately disagree,
and `SafeName::hazards` is carried precisely so the sentence can say what
changed and why.

# What is still the caller's problem, per the engine's own warning

A `SafeName` is *"a name, not a location"*. This joins it to a directory the
operator chose in a native save dialog — which is also where overwrite
confirmation comes from, because the OS dialog owns that question and asks
it better than pdfcer could.

### `fn resolve`

A free function, and pure, so [`tests`] can hold it to the two properties
that matter without a `Ui` and without a running application.

# Why the comparison is byte-for-byte and case-sensitive

§7.9.6 requires name-tree keys to be *"compared for equality on a simple
byte-by-byte basis"* — not by any collation, not case-folded, not
normalised. Two keys differing only in case are two different attachments,
and a lenient comparison here would let a Remove press find the wrong one.

`None` is a reachable, ordinary answer rather than an error: the queue
drains after the frame, so an undo or a removal raised earlier in the same
frame can take the row away before this action is applied.

### `fn suggested_path`

Beside the **document**, named after the attachment — which is the
combination the two halves of the rule give. `super::export::suggested_path`
states the directory half and its reason: *"a picker that opens in the
last-used directory of some other application is a picker that makes the
operator navigate back to their own project every time."* The name half is
different from every other suggestion in this application, because the file
being written is not derived from the document at all — it is a file that
was put inside it, and it has its own name.

`safe_name` is the **sanitised** value and must be: this string reaches a
native save dialog, and a raw attachment name can be a path.

### `fn a_reference_finds_its_own_kind_and_not_the_other`

The assertion is not "resolve returns something". It is that the two
kinds do not cross — which is exactly what a match arm written in a
hurry gets wrong, and which the fixture can actually distinguish because
it holds one of each.

### `fn an_unknown_operand_resolves_to_nothing`

The failure this forbids is the one that would make the whole
address-by-key argument hollow: a `find` written as *"the first
document-level entry"* passes the test above and removes the wrong file
the moment a document has two.

Both directions are checked, because a resolver can be wrong in two
ways — finding something when it should find nothing, and matching a
key against the wrong kind's operand.

### `fn a_key_differing_only_in_case_is_a_different_attachment`

§7.9.6 requires exactly this — *"compared for equality on a simple
byte-by-byte basis"* — and a lenient comparison is the kind of
helpfulness that removes the wrong attachment from a document holding
both `Report.pdf` and `report.pdf`, which is legal.

### `fn the_verbs_and_their_operands_are_distinguishable`

The second half is the one worth having: the queue may hold more than
one action from a frame, and a variant that compared equal on only part
of its operand would let a de-duplicating caller drop the wrong one.

### `fn a_hostile_attachment_name_cannot_escape_the_chosen_folder`

The fixture is the engine's own, and it exists because these names are
authorable in a real document. What is asserted is the property the
suggested path must have: **one component**, inside the directory the
document is in, whatever the document called the file.

The check is deliberately on the assembled path rather than on
`safe_name` alone — sanitising and then joining wrongly would pass a
test of the sanitiser and still write outside the folder.

### `fn the_suggested_path_is_the_document_s_folder_and_the_attachment_s_name`

Both halves, because getting either wrong is invisible until an operator
is hunting for a folder: a suggestion in the wrong directory makes them
navigate back to their own project, and one named after the *document*
would offer to save a spreadsheet as `drawing.pdf`.

### `fn apply`

The dispatch half of this module, reached from `PdfcerApp::apply`'s single
[`super::Action::Attachment`] arm. A free function taking
`&mut OpenDoc` rather than a method, exactly like [`super::bookmarks::apply`]
and [`super::pages::apply`], because the caller is the one place that owns
the borrow and the arm should be one line.

**Two of the three do not go through [`super::apply::vector_edit`]**, and
the exception is principled rather than convenient: that function is the
cancel–mutate–bump–invalidate protocol for an edit, and
[`AttachmentAction::SaveCopy`] performs no edit. Running it through anyway
would cancel the render worker and bump the epoch for an operation that
changed nothing, which is how a status bar comes to retire a disclosure that
is still true. [`AttachmentAction::Attach`] and
[`AttachmentAction::Detach`] **do** mutate and **do** go through it.

The `page` argument passed to `vector_edit` is `0` for both mutating
verbs, and that is honest rather than lazy: a document-level attachment
belongs to the catalogue and to no page. [`super::bookmarks::apply`] passes
`0` for the identical reason, and its comment records that the parameter
exists so the diagnostic trace can say which sheet a *geometry* edit
touched.
