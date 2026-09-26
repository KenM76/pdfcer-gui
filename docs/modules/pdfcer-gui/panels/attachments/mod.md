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

## Item notes

### `fn fmt`

A description is the operator's own words about their own file, and this
reaches a trace file a harness keeps. `panels::bookmarks::BookmarksUi`
and `panels::docprops` make the same choice for the same reason.

### `struct Published`

# Why a struct rather than four `&mut bool`s

It began as two, grew to four when the clipboard arrived, and tripped
clippy's seven-argument limit — which was the right complaint about the
wrong symptom. The four flags are **one fact**: *"the first visible row has
been drawn"*, asked separately per control because a control that is absent
on the first row (Remove, on a page-level attachment) must not consume the
flag for the row that does have one.

A region name is a key in a **flat** namespace. Publishing
`attachments.save` from every row would emit one rectangle per row under one
key, and a driven check would click whichever was written last — not the row
it meant, and not stable between runs. These rows also live in a
`ScrollArea`, where a control scrolled out of view still reports a rect, so
the publish goes through [`crate::diag::ui_rect_visible`] as well.

### `fn rows`

# Why the two `published` flags exist

A region name is a key in a flat namespace. Publishing `attachments.save`
from every row would emit one rectangle per row under one key, and a driven
check would click whichever was written last — which is not the row it meant
and is not stable between runs. The Comments panel solved this the same way
and its comment carries the other half of the reason: these rows live in a
`ScrollArea`, and *"a control scrolled out of view still reports a rect. A
harness clicking a coordinate that is behind the scroll edge clicks whatever
IS there, which fails as something else entirely."* Hence
[`crate::diag::ui_rect_visible`] rather than `ui_rect`.

### `fn controls`

# Every branch here is R9 applied to a different fact

| state | what is drawn | why |
|---|---|---|
| no `/EF` at all | a sentence, no button | an **external** file reference (§7.11.3) is legal and has nothing to save |
| an `/EF` that does not resolve | a different sentence, no button | `AttachmentNotes::unresolvable_streams` is documented as *"always a defect"*, and calling it the same thing as the legal case would either accuse a good document or excuse a damaged one |
| a page annotation | Save, and a sentence instead of Remove | `detach_file` refuses one by name; it is removed as an annotation |
| a document-level entry with bytes | Save and Remove | the full case |

A control is **absent** in each case rather than greyed, because P3 reserves
greying for something *temporarily* unavailable that can say when it will
not be — and none of these will ever become available by waiting.

### `fn display_name`

An empty name is legal — `NameSource::None` is reachable when a filespec
carries no `/F`, `/UF`, `/DOS`, `/Mac` or `/Unix` and there is no tree key
to fall back on — and the row must still exist, because the bytes can still
be saved out. A blank line where a name belongs reads as a rendering fault.

Pure, so [`tests`] can hold it to that without a `Ui`.

### `fn where_it_lives`

Pure, so [`tests`] can hold the page numbering to being 1-based without a
`Ui` — `page_index` is 0-based *"into `pages`"* and the off-by-one is the
kind that looks like a document defect rather than a bug.

### `fn addressable`

`AttachmentKind::PageAnnotation::annot_id` is an `Option` — `None` when the
`/Annots` entry was a direct dictionary rather than a reference — and a row
pdfcer cannot name gets no button. That is R9 rather than caution: a control
whose operand cannot be constructed is an affordance for something that
cannot work.

### `fn readable`

# Two substitutions, and both are rendering rather than reporting

- **`CR` becomes `LF`.** §12.5.6.2 makes carriage return the paragraph
  separator in annotation `/Contents`, which is where a page-level
  attachment's description comes from — and egui lays a bare `CR` out as
  nothing at all, so a two-paragraph description would render as one long
  run with a gap in it.
- **Other C0 controls become a space.** A name or description from a
  document is unconstrained text (see `Attachment::name`), and a `NUL` or a
  `BEL` in a label is a glyph nobody can read.

Neither is a disclosure case, and the distinction is worth stating because
this crate's rule 4 posture is otherwise to disclose everything: pdfcer is
not reporting a different *value* here, it is drawing the same value
legibly. The value that reaches the **filesystem** goes through
`sanitize_attachment_name` instead, and that one *is* disclosed — see the
module header's third required disclosure.

### `fn the_two_kinds_are_described_differently_and_the_page_is_one_based`

Both halves fail invisibly. Describing them alike would tell an operator
that a file pinned to page 2 belongs to the document and survives that
page's deletion — which is exactly backwards, and is the one fact
`AttachmentKind`'s own docs say *"bites hardest at save time and at
page-delete time"*. And `page_index` is 0-based, so a row that printed
it raw would name the wrong sheet on every document.

### `fn only_a_nameable_attachment_gets_a_verb`

The first half is what makes Remove possible at all. The second is the
property that keeps a button from being drawn for an operand this code
cannot construct — asserted by construction, because no fixture in the
engine's tree carries a direct-dictionary `/Annots` entry and inventing
one here would be testing a hand-built value rather than a document.

### `fn an_unnamed_attachment_is_labelled_rather_than_blank`

`NameSource::None` is reachable, the bytes are still saveable, and a
blank line where a name belongs is indistinguishable from a rendering
failure. Checked against the whitespace cases too — a name of three
spaces is an invisible row, which is the same defect as no row.

### `fn a_control_character_is_made_legible_without_changing_the_words`

A name in a PDF is unconstrained text, `NUL` and `BEL` are authorable,
and §12.5.6.2 makes `CR` the paragraph separator in the `/Contents` a
page-level description comes from — which egui lays out as nothing.

What is asserted is that the *substitution* happened, not that the
string was censored: the visible characters are untouched, because this
panel reports what the document says.

### `fn a_hostile_name_is_shown_and_not_quietly_repaired`

The bargain this panel makes, and both halves have to hold or neither is
worth anything: the *listing* reports the raw name, because
`sanitize_attachment_name`'s own docs say a reader that quietly repairs
its evidence is not a reader and *"the operator investigating a
suspicious file would be looking at pdfcer's cleaned-up version"*; the
*save path* uses the sanitised one, which
`crate::app::actions::attachments` asserts from the other side.

### `fn the_row_regions_are_named_apart`

One name from two controls would leave a driven check clicking whichever
was published last, and the failure presents as *"the button does
nothing"* on whichever run lost the race.
