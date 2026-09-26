# `dialogs::stamp_collection` — turning this document into stamps Acrobat
will show in its own menu

`OPERATOR_REQUESTS.md` **O169**, the operator's words:

> *"if acrobat has a way of adding custom stamps or text, we need the same
> feature too with the same import/export to make the stamps as Adobe has
> and is compatible with adobe's"*

## The finding that decided the whole shape of this window

**There is no interchange format, because Acrobat has none.** A stamp
collection *is an ordinary PDF* — one file per category, one page per stamp,
the category in `/Info` `/Title` and the names in the catalog's `/Names` →
`/Pages` name tree. So "the same import/export" is not a format to
implement: **handing someone the PDF is the export**, and dropping one into
Acrobat's stamps folder is the import. What the operator was missing was not
a converter — it was a way to *author* one of those files, and that is
exactly what this window does.

⇒ The window therefore has no format picker, no options page and no
"compatibility mode". It asks two questions — **what is this set called**
and **what is each page called** — and writes a file measured against the
four Adobe collections on this machine.

## The trap this window is arranged around

`pdfcer_core::stamp_file::name_stamp_pages` names `stamps[i]` to **page `i`
of the document it is given — by counting, not by lookup.** Untick page 1,
hand the engine the full name list, and every stamp names the page below the
one it describes: a file that opens, holds the right number of stamps,
appears in Acrobat's menu, and **stamps the wrong picture every time**.
There is no symptom short of looking at the artwork.

The guard is structural rather than careful. [`crate::stamps::Plan`]'s
`for_engine()` and `pages_to_extract()` are the *same filter over the same
rows*, so the name list and the extracted page list cannot drift apart, and
`crate::stamps::tests::plan_and_extraction_agree` is the unit test that
fails if somebody makes them two filters again.

## Why the operator's document is never touched

`Save as stamp collection…` is a **Read-mode-legal act** by the standing
rule — *Read may produce a new document; it may not modify this one*. The
write path never takes a mutable handle on his session at all: it extracts
the ticked pages into a new document and names *those*. See
[`crate::stamps::write`]'s header for the two reasons that matters.

## Rule 4, in this window

Every adjustment pdfcer makes to a *hidden* identifier is listed **before**
the write, off-canvas, in [`Self::adjustments_group`] — and only when there
is something to list. Nothing on the page is marked, tinted or flagged,
because the operator's own words are that *"the nagging and red flagging in
the original GUI made for a lot of extra bugs in the visibility when
editing."*

The half that is easy to skip is the one that matters most here: an operator
**cannot see** the identifier. His name is written exactly as typed; the
identifier beside it may have lost a space, a `#` or eight characters, and
the only place he could ever learn that is a sentence like these.
