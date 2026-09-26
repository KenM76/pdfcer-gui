# `panels::attachments` — the files this document carries inside itself

A PDF can hold **whole other files**: ISO 32000-1 §7.11.4.1 *embedded file
streams*, reached either from the catalogue's `/Names /EmbeddedFiles` name
tree (document-level) or from a `/FileAttachment` annotation on one page
(§12.5.6.15). This panel is where an operator sees them and acts on them.

## Why this panel exists at all, and what its absence cost

`pdfcer-core` has carried `attach_file`, `detach_file`, `list_attachments`,
`list_attachments_with_notes`, `extract_attachment`, `attachment_bytes` and
`sanitize_attachment_name` — a fully worked feature with fixtures, hazard
analysis and a spec-ambiguity register — and **this shell had no way to
reach any of it.** Not a command, not a menu item, not a panel. An engine
capability with no operator surface is, from the operator's chair,
indistinguishable from a capability that does not exist.

`crate::panels`' own header records the same defect from the other
direction, for three panels that shipped with a body and no control an
operator could click, and names what kept it invisible: *"their only callers
were the harness step handlers, so every verification passed while the
panels were unreachable in a real build."*

## The shape is Acrobat's, deliberately

`crate::text::tool`'s rule — *use the conventional interaction, never invent
one* — settles the layout before any argument about it starts. Acrobat's
Attachments panel is a **list with a toolbar**: each row names a file, gives
its description, size and date, and the toolbar adds one and removes one;
double-clicking or *Save attachment* writes one out. Every reader that
competes with it does the same thing. So:

| Acrobat | here |
|---|---|
| the list, with name · description · size · modified | [`body`]'s rows |
| *Add* (paperclip) | [`attach`], drawn **above** the list |
| *Delete* | the per-row Remove |
| *Save attachment* | the per-row Save a copy |
| *Edit description* | **absent** — see below |

## What is absent, and why each absence is R9 rather than an omission

- **Edit description.** `attach_file` takes the description at attach time
  and `pdfcer-core` has no verb that changes one afterwards. R9: an absent
  capability renders nothing, and a greyed control would promise a state of
  the program that cannot exist. The **limit** is disclosed in the attach
  row, because an operator needs to know it before they leave the box empty.
- **Remove, on a page-level row.** `detach_file` addresses the
  `/EmbeddedFiles` name tree and answers `AttachmentNotFound` for a
  `/FileAttachment` annotation **by name** — the engine separated the two
  cases precisely so a shell could. Those are annotations and are removed as
  annotations. The row says so ([`t::remove_lives_with_the_note`]) rather
  than leaving a hole where the other rows have a button.
- **Save a copy, on a row with no bytes.** A filespec with no `/EF` is an
  *external* file reference (§7.11.3) — legal, and there is nothing to
  write. The row says that too.
- **Open.** Acrobat opens an attachment in its host application; that is a
  process launch on bytes that came from inside a file that arrived from
  somewhere, and `pdfcer_core::attachments`' module docs are explicit that
  pdfcer *"does not execute, open, or interpret them, and neither should a
  caller without its own gate."* Saving a copy is the whole of what this
  shell offers, and the operator's own file manager is the gate.

## Three disclosures this panel is REQUIRED to make

Each is an obligation `pdfcer-core` writes into its own doc comments, and
each is invisible to an operator who is not told:

1. **Removing does not erase.** Under the default incremental save (§7.5.6)
   every prior revision stays in the file — that is what makes existing
   signatures survive — so a removed attachment's bytes are recoverable
   until a full rewrite. `detach_file`: *"Shells are expected to say so
   rather than let 'delete' imply erasure."*
2. **The bytes may be ciphertext.** Since PDF 1.5 an embedded file can be
   encrypted **in an otherwise unencrypted document** (`/EFF` naming a
   `DefEmbeddedFile` crypt filter, §7.6.5), and pdfcer does not decrypt on
   this path — so an extraction can succeed and produce garbage, silently.
3. **The name pdfcer writes to disk may not be the name shown.** The listing
   shows the raw name because *"a forensic reader that quietly repairs its
   input is not a reader"*; the filesystem gets a sanitised one because the
   raw one may be `..\..\Windows\System32\evil.exe`.

All three live off-canvas, in the status line, which is `README.md`'s first
non-negotiable: *"Disclosure lives off-canvas … never blocking, never
requiring acknowledgement, never positioned relative to the document."*

## Actions, not mutations

This body is handed `&OpenDoc` — **shared**, so it is a compile-time fact —
and pushes `crate::app::actions::Action::Attachment`. All three verbs open a
native file dialog, which must not happen inside a layout pass, so the
picker lives in `PdfcerApp::apply`. See
[`crate::app::actions::attachments`] for the whole argument.

## Why the listing is read fresh each frame

`list_attachments_with_notes` takes an object graph rather than `&mut self`,
so it can run inside the draw closure, and the answer changes under every
attach, every removal, every undo and every page delete (a page-level
attachment dies with its page). [`crate::panels::bookmarks`] makes the same
trade for the same reason: a cache would need invalidating on every edit,
which is a correctness problem traded for a walk of a structure that is a
handful of entries on any real document — and `MAX_ATTACHMENTS` bounds even
a hostile one.
