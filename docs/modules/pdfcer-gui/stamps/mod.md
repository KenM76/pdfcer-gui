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

## ★★ Every claim above is measured, in Adobe's own files, on this machine

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

★ **`StandardBusiness.pdf` is the one that disproves the obvious design.**
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

## ★★★ Placing a custom stamp — was NOT here, and now is

This section used to read *"Placing a custom stamp on a drawing … the
engine has no verb that draws one page's artwork onto another"*, and it was
true when written. **`pdfcer-core` `Pass 293.0` shipped
`EditSession::place_page_artwork`** in answer to this project's own filed
request, and [`library`] is the half above it: which stamps exist, what
they are called, and which page of which file each one is.
