# `panels::docprops::stamps` — **the read half of O169**

The operator asked for Acrobat's custom stamps *"with the same
import/export"*. The finding that shaped the whole feature is that there is
no import and no export to match: **a stamp collection is an ordinary PDF**
— one file per category, one page per stamp, the category in `/Info`
`/Title`, the names in the catalog's `/Names` → `/Pages` name tree. Handing
somebody the file *is* the export; `file.open` *is* the import.

⇒ Which means an operator can open one and pdfcer will show him a document
of unrelated pictures with nothing to say that it is anything else. That is
what this section exists to end. [`crate::dialogs::stamp_collection`]
authors a collection; this one **discloses** one.

## Why this is disclosure and not decoration

**R8b rule 4 — "fuzzy, never sneaky".** Everything pdfcer infers about a
file it did not write is reported **off-canvas**, and this is as off-canvas
as it gets: a section in a properties panel, on the far side of the window
from the page. Nothing here draws on the document, tints a page, badges a
thumbnail, or marks a collection's pages as special in the page list. A
screenshot of the canvas with this panel closed is identical to a screenshot
of the same file opened by a build with no stamp support at all.

The half of rule 4 that is easy to drop is the half that binds here: *the
inferences the operator cannot see still owe a report.* Every fact in this
section is invisible in the page view — the category is a metadata string,
the names are name-tree keys, and *dynamic* is a `#` on an identifier no
rendering will ever show. Precisely because none of it is visible, all of it
is owed.

## Why nothing here is editable

The rows are labels, not fields. Renaming a stamp in place would need
`EditSession` verbs against the name tree that do not exist — the engine
writes a whole tree (`stamp_file::name_stamp_pages`) and does not amend one
— and **R9** says an unavailable capability renders **nothing**, not a
greyed stub. The route that does exist is the real one: *Save as stamp
collection…* reopens this file with its own names already filled into the
dialog, and writes a new collection.

⇒ So there is deliberately no *Edit names* button here, and no sentence
explaining its absence either. A sentence explaining an absent control is
the nagging rule 4 exists to prevent.

## What is read, and what it costs per frame

[`pdfcer_core::stamp_file::read`] is called on **every frame this panel
draws**, with no cache, and that is a decision rather than an oversight.

- **Staleness first, because cost is the wrong question to answer with.**
  A cache here would have to be invalidated on the edit epoch, on a re-read
  under the other duplicate-key policy, and on document replacement — three
  invalidations for a value that must never be wrong, in a panel whose whole
  job is to state facts about the open file. This project has already
  shipped one cache whose comment argued *cost* while the question in front
  of it was *staleness*.
- **And then cost, measured against the engine's actual code path.** `read`
  looks up `/Info` `/Title`, then the catalog's `/Names`, then its `/Pages`.
  On an ordinary PDF the second lookup fails and it returns — two dictionary
  probes and an empty `Vec`. It walks the page tree **only** when a `/Pages`
  name tree exists, which is to say only on a document that really is a
  collection, and Adobe's largest ships twelve stamps in one flat node.

⚠ The read is against `session.document()` — the **base revision**, exactly
as [`super::facts`] reads. That is right rather than merely convenient: no
verb in this shell edits a name tree, so the base revision's tree *is* the
current one, and there is no session state that could disagree with it.

## Item notes

### `const REGION`

That conditional publication is the contract, and it is the half a driven
check can only test with a **second launch**: a region declared on
`fixtures/stamps-standard-business.pdf` *and* declared on an ordinary
drawing would be a heading that is always there — a different defect wearing
the same green tick. [`super::REGION_ANOMALIES`] carries the same shape and
the same warning.

Named under the `properties.` prefix like its neighbours, so that the
`declared_names(&trace, "properties")` dump several checks print when they
cannot find a region lists it. A region under a prefix nobody enumerates is
discoverable only by whoever wrote it.

### `const REGION_ROW_PREFIX`

⚠ Indexed by tree position, **not** by page number, and the difference is
the whole subject of [`crate::stamps`]' re-opening logic: the name tree is
sorted lexicographically (§7.9.6) and the pages are not, so tree entry 1 is
routinely not page 2. A check that wants a particular stamp must read the
row it finds, never compute an index from a page.

### `fn section`

# The test is the name tree, never the title

[`crate::stamps::is_collection`] asks whether the document has **named
pages**. A PDF with a `/Title` and no name tree is just a PDF with a title,
and every drawing the operator opens has one of those. Testing the title
would put this section on most of his files, which is the failure mode a
conditional section has: shown too often, it stops being information and
becomes furniture.
