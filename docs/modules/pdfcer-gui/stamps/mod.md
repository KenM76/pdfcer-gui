# Acrobat-compatible **stamp collections** — the shell's model

Engine `Pass 288.0` (`pdfcer_core::stamp_file`) reads and writes the file
format. This module is the part that cannot live in the engine: turning a
document into a *plan* an operator can look at and correct, and turning
what the engine read into sentences a panel can show.

## What a custom stamp actually is, because the answer changes the UI

**There is no stamp format and no import/export.** A stamp collection is an
ordinary PDF:

* **one file per category, one page per stamp**;
* the **category name** is the file's `/Info` `/Title`;
* the stamp names live in the catalog's `/Names` → `/Pages` name tree
  (§7.7.4 Table 31), one string per stamp in the form `internal=display`;
* an internal name beginning `#` marks a **dynamic** stamp, whose text
  Acrobat recomputes from AcroForm scripts when it is placed.

The operator asked for *"the same import/export"* Acrobat has (`O169`), and
the honest answer is that Acrobat has none — handing someone the PDF **is**
the export. That is why this module has no serialiser: there is nothing to
serialise to. It has a *plan* and a *save path*, and the save path is an
ordinary PDF write through the ordinary save route.

## Every claim above is measured, in Adobe's own files, on this machine

Not sourced from the internet. The engine measured them from
`…/Acrobat DC/Acrobat/plug_ins/Annotations/Stamps/ENU/*.pdf`, and this
module's author re-ran that measurement through the release CLI before
writing a line of it:

```text
Standard.pdf          category=Standard           14 stamps
StandardBusiness.pdf  category=Standard Business  12 stamps
Dynamic.pdf           category=Dynamic             5 stamps, all `#`-prefixed
SignHere.pdf          category=Sign Here           5 stamps
```

**`StandardBusiness.pdf` is the one that disproves the obvious design.**
Its `SBApproved` names page 1 and `SBCompleted` names page 5. **Page order
is not name-tree order** — §7.9.6 requires the tree be sorted
lexicographically by name, and a conforming reader may binary-search it. A
surface that lists pages in file order and labels them from the tree in
tree order will mislabel every stamp and look completely correct while
doing it. Everything here is keyed by page index and the sorting is left to
[`pdfcer_core::stamp_file::name_stamp_pages`], which does it in the engine
where the requirement is documented.

## What pdfcer authors, and the one thing it deliberately does not

**Dynamic stamps are read and reported, never authored.** Their text comes
from AcroForm calculation JavaScript; a dynamic stamp pdfcer wrote would
carry its design-time text, which the engine's own header calls *"correct
as a picture, wrong as a promise"* — a stamp reading `Received 10 Sep 2026`
forever, on every drawing, is worse than no stamp.

That has a consequence this module enforces rather than hopes for: a
display name the operator types must never *become* a dynamic marker.
See [`derive_internal`] and its `#`-stripping clause.

## R8b rule 4 — where the disclosures in here come from

Three things happen to an operator's typing on the way to a name tree, and
each is an inference he did not ask for:

| inference | why it happens | disclosed as |
|---|---|---|
| characters removed | a name-tree key is a PDF string; a tab or a newline in one is a file nobody can read back reliably | [`Adjustment::CharactersRemoved`] |
| a leading `#` removed | `#` means *dynamic*, and pdfcer does not author dynamic stamps — writing one would promise recomputation that never happens | [`Adjustment::DynamicMarkerRemoved`] |
| a number appended | §7.9.6 keys are unique; two pages both called *Approved* cannot both be `Approved` | [`Adjustment::MadeUnique`] |

All three are reported **off-canvas**, in the dialog that is about to write
the file, before it is written. None of them marks anything on a page.

## Placing a custom stamp — was NOT here, and now is

This section used to read *"Placing a custom stamp on a drawing … the
engine has no verb that draws one page's artwork onto another"*, and it was
true when written. **`pdfcer-core` `Pass 293.0` shipped
`EditSession::place_page_artwork`** in answer to this project's own filed
request, and [`library`] is the half above it: which stamps exist, what
they are called, and which page of which file each one is.

## Item notes

### `const MAX_INTERNAL_LEN`

Not a format limit — §7.3.4.2 puts no ceiling on a PDF string and the name
tree inherits none. It is a *legibility* limit: Adobe's longest shipped
internal name is `SBConfidential` at fourteen characters, and a key long
enough to wrap in a debugger is a key nobody will ever read. Truncation is
disclosed like every other adjustment, so an operator who wants his
forty-character name knows it did not survive.

### `enum Adjustment`

Each variant is an **inference** — something pdfcer decided that the
operator did not type — and each is disclosed before the file is written.
A collection whose plan has no adjustments needs no disclosure at all,
which is the common case and should stay silent.

### `struct ExistingName`

**Why this exists rather than passing `&StampCollection` around.** Two
reasons, and the second is the one that matters:

1. `StampEntry` and `StampCollection` are both `#[non_exhaustive]`, so no
   code outside `pdfcer-core` can build one. A model that took the engine
   type could only ever be tested by authoring a real PDF and reading it
   back — which is a fine integration test and a terrible unit test, and
   the pressure would be to skip the unit test.
2. **The plan does not want the engine's type.** It needs a page and a
   name; it has no business knowing what a `dynamic` flag is, because it
   never authors one. Narrowing at the boundary is what keeps
   [`derive_internal`]'s `#`-stripping clause honest — there is no way for
   a dynamic marker to arrive here disguised as something else.

[`existing_names`] does the reduction, in one place.

### `fn new`

`existing` is what [`pdfcer_core::stamp_file::read`] found, when the
document already **is** a collection. Re-opening one must show what it
already says rather than a fresh set of defaults — an operator fixing
one typo in a twelve-stamp set should not have to retype eleven names.

`existing` is matched **by page index, not by position in the tree**.
The tree is sorted lexicographically (§7.9.6) and the pages are not, so
zipping the two lists would attach `SBCompleted`'s name to page 1.

### `fn rederive`

Called after any edit. It re-derives the **whole** list rather than the
row that changed, because uniqueness is a property of the set: renaming
row 3 from `Approved` to `Issued` frees `Approved` for row 7, and a
per-row update would leave row 7 as `Approved2` forever with a stale
[`Adjustment::MadeUnique`] disclosure attached to it.

That stale-disclosure case is the reason this is not an optimisation
target. A disclosure has a subject; when the subject goes, the sentence
must go with it.

### `fn for_engine`

**This is the contract that is easy to get wrong.**
[`pdfcer_core::stamp_file::name_stamp_pages`] names `stamps[i]` to page
`i` — **it counts, it does not look up.** So the returned list must be
dense from page 0 of *the document being written*, which is not the
document the operator has open whenever he has excluded a page.

[`Self::pages_to_extract`] is the other half, and the two are
documented together because using one without the other silently names
the wrong artwork: exclude page 0, hand the full name list to a
document whose pages start at the old page 1, and **every stamp is
attached to the page below the one it describes** — a file that opens,
has the right stamp count, appears in Acrobat's menu, and stamps the
wrong picture every time.

The two are filtered by the same predicate in the same order, which is
the cheapest way to make that invariant true by construction rather
than by review. `plan_and_extraction_agree` in the tests is the guard.

### `fn pages_to_extract`

Fed straight to `pdfcer_core::pageops::extract`, which is how exclusion
is implemented: rather than deleting pages out of a session, the writer
**extracts** the wanted ones into a new document and names those. That
keeps the operator's open document untouched — the same argument
`app::actions::extract` makes for the page verb it owns — and it makes
the density [`Self::for_engine`] requires automatic.

### `fn adjustments`

Empty when nothing was inferred, which is what lets the dialog stay
silent in the common case instead of showing an always-present box
that says nothing.

### `fn blocker`

Returns `None` when it can. The two refusals are real states rather
than defensive checks: a collection with no stamps is not a collection,
and an untitled one shows up in Acrobat's menu with no heading.

### `enum Blocker`

⚠ R9: the Save control is **greyed** for these, not hidden, because both
are *temporarily* unavailable — the operator fixes them by typing — and
both are explained on hover.

### `fn default_display`

One-based, because it is shown to a person beside a page number that is
also one-based, and a set of stamps called `Stamp 0 … Stamp 11` is a set
somebody has to renumber by hand.

### `fn derive_internal`

Returns the name and everything that had to be done to it. See
[`Adjustment`] for why each clause exists; the order below is deliberate
and the reasons are not interchangeable.

# The clauses, in the order they must run

1. **Keep only characters that survive a round trip.** Alphanumerics,
   `_` and `-`. Spaces go rather than becoming underscores: Adobe's own
   keys (`SBApproved`, `SHInitialHere`) are unspaced camel case, and
   matching the neighbours is worth more than preserving word boundaries in
   a string no operator reads.
2. **Strip a leading `#` — after step 1, not before.** `# Approved` and
   `#Approved` must both lose the marker, and only step 1 makes those the
   same string. Doing this first would let `# Approved` through as
   `#Approved`. This is the clause with a correctness consequence rather
   than a cosmetic one; see the module header.
3. **Fall back when nothing survives.** A name of `` sanitises to
   nothing, and a name tree key of `""` is a file Acrobat shows an empty
   menu row for.
4. **Truncate**, then
5. **De-duplicate**, in that order — de-duplicating first and truncating
   after would cut the number back off and reintroduce the collision.

### `fn is_collection`

The test is the **name tree**, not the title: a PDF with a `/Title` and no
named pages is just a PDF. Mirrors
[`pdfcer_core::stamp_file::StampCollection::is_stamp_file`] and exists so
callers need not import the engine type to ask.

### `fn dynamic_count`

Worth its own function because the answer drives a sentence rather than a
number: a collection that is *entirely* dynamic — which the operator's own
signature file is — deserves to be described differently from one with a
single dynamic entry among twelve.

### `fn page_tree_unreadable`

# The field that makes every count below mean one thing again

Until engine `Pass 290.1` (2026-09-10, `bce4703`) `stamp_file::read` built
its page list with `page_tree::pages(doc).map(…).unwrap_or_default()`, so a
page tree that refused produced an **empty** list, so every
`StampEntry::page_index` came back `None` — and `None` already meant
something else and something specific: *"this name points at a page the
document does not have."* Two opposite facts arrived in one value, and on
the operator's own Acrobat-written signature file pdfcer reported both of
his real signatures as pointing at nothing when neither did.

The engine now carries the failure separately, which is what lets this
module's wording become a claim again instead of an observation carefully
phrased to be true either way:

| this returns | what `page_index: None` means |
|---|---|
| `Some(why)` | **nothing about the stamp.** The page tree is unreadable; no `page_index` in the collection carries information |
| `None` | the name genuinely points outside this document |

The names, display titles and dynamic flags are read from the `/Names` →
`/Pages` **name** tree and are unaffected by whatever is wrong with the
**page** tree, which is why `read` still returns a collection worth showing
and this is an `Option` rather than the whole call becoming a `Result`.

### `fn unresolved_count`

**`None` is not zero and must never be rendered as zero.** It means the
page tree could not be read, so no `page_index` in the collection carries
information; see [`page_tree_unreadable`] for why the two used to be
indistinguishable and what it cost. A caller that unwrapped this to `0`
would print *"every stamp resolves"* about a document where pdfcer resolved
none of them, which is the same class of defect as the one the engine just
removed, moved one crate along.
