# `panels::properties::face` — the face chooser, once, for both surfaces


★ That divergence is the reason this module exists at all, and it is worth
stating as a rule rather than as a tidy-up: **a control drawn twice is a
control that will be built twice**, and the second build is always the one
that misses the disclosure. The two callers now differ in exactly two
things — the region prefix they publish under, and what they do with the
selector that comes back — and in nothing else.

## ★★★ The list has two kinds of row now, and they are two different acts

`pdfcer-core` v0.15.0 (`Pass 162.0`):

> **FONTS** — text can be restyled to a face the document **DOES NOT
> CONTAIN**, for the fourteen faces every PDF reader is required to have.
> pdfcer authors the font resource on demand, with widths, embedding
> nothing. A face outside those fourteen still refuses by name — that needs
> a real font program.

So a row in this list is now one of:

| kind | what a click does to the file | what it costs |
|---|---|---|
| [`FaceOrigin::OnThisPage`] | rewrites one `Tf` operand | nothing — the resource is already there |
| [`FaceOrigin::PdfcerWouldAdd`] | rewrites the `Tf` operand **and writes a new `/Font` object** | a dictionary with widths and no glyph outlines, plus a face drawn from the reader's own copy |

Those are different enough that presenting them as one undifferentiated list
would be hiding a write behind a menu. [`choices`] tags every row and
[`popup_body`] draws them under two headings with the disclosure between.

## ★★ Who writes the resource — asked, and answered by reading the engine

`FormatPlan::created_font` is documented as *"a `/Font` resource the caller
must CREATE for `new_content` to be valid"*, and its own note says the caller
owns the write *"because only the caller can allocate an object number:
planning runs against an immutable `&Document`."* That is a real obligation
and it would not be this shell's to invent.

**It does not fall to this shell**, because `EditSession::format_text` is
itself that caller and it already performs the write — on both of its paths:

* the page-`/Contents` path (`pdfcer-core/src/edit.rs` ~7887) takes
  `plan.created_font`, calls `self.font_resource_writes(page.id, true, …)`
  and extends **the same command** with the writes, *"so one undo removes
  both the restyle and the resource it needed"*;
* the form-XObject path (~8015) binds the resource into the form's own
  `/Resources` before the stream is rebuilt, for the stated reason that two
  writes for one object id in one command would let the later silently win.

Both also push a disclosure when the target `/Resources` turns out to be
shared with other pages. `crate::app::actions::textstyle` already surfaces
the engine's disclosures verbatim, so that one arrives on the status bar with
no code here.

⇒ **The shell change is the chooser and nothing else.** Nothing in this
module allocates an object, writes a dictionary, or knows the shape of one.


Every [`FaceOrigin::OnThisPage`] row has been through `set_font`'s own
acceptance test for **this run's characters** — that is what
`preview_font_resources` is, and it is why a refused page font is absent from
the list rather than greyed. **The other half now matches it.**

[`choices`] reads `FontPreflight::standard_14`: one `Std14Entry` per face,
each carrying the engine's own spelling, a `presence` that says `OnPage` or
`WouldBeAdded` as a **fact**, and an `acceptance` run through `set_font`'s
gate with the embedded-subset floor included. Three local re-derivations
were deleted with it — the `Std14::ALL` walk, the `carried` filter built by
shortening `preflight.entries`, and the untested offer.

⚠ **Consequence, stated because it is a behaviour change:** a standard-14
face that cannot hold this run's characters is now **absent** rather than
offered-and-then-refused. That is the first half's behaviour, applied to the
second.

### What this paragraph said until the engine answered it, kept because the
### reasoning is why the engine answered it

> *"It cannot be done honestly from here. The engine offers no query that
> coverage-tests a face the page does **not** carry, so the shell's only
> route would be to re-derive the encoding rule — which face uses
> `WinAnsiEncoding`, which two use a built-in `FontSpecific` one, and which
> characters that leaves unmapped. `FontPreflight`'s own invariant forbids
> exactly that (`R221`), and a second copy of the rule in `pdfcer-gui` would
> drift from the commit path the first time the rule changed.*
>
> *⇒ So these rows are **offered, and a refusal is a sentence** … This is
> recorded as an engine ask rather than worked around: a
> `preview_font_resources` that also surveyed the fourteen would let this
> list be as exact as its first half already is."*

★★ **That last sentence is the request, and `Pass 142.2` is the answer.**
The refusal to copy `R221`'s rule into this crate is why the fix arrived as
an engine capability rather than as drift — worth keeping, because the
tempting shortcut was one afternoon's work and would have been wrong on the
first day the encoding rule changed.


★ The old `carried` filter is subsumed, and more exactly. It compared
*shortened* names against the page's entries; `Std14Presence::OnPage` is the
engine answering the same question from the resource dictionary it actually
resolved. A face already on the page reaches the list through the
`accepted()` half if it works, and through neither if it does not — so
`Helvetica` is never offered as *"pdfcer can add"* on a page whose own
`Helvetica` was refused.

## Rule 4

Nothing here marks the canvas. The one inference an operator cannot see —
that an added face is drawn with the *reader's* copy rather than one carried
in the file — is discharged as a sentence at the point of choice
([`crate::text::panels::face::face_addable_disclosure`]), which is the
off-canvas report rule 4 requires and the one thing this feature could not
ship without.
