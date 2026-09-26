# `panels::forms` — filling this document's interactive form

## What this panel is for

The operator's goal for this build is *"replace Acrobat Reader first"*, and
**a reader fills forms; it does not create fields.** So this panel offers
text fields, check boxes, radio groups, choice lists, a reviewed reset, a
reviewed native recompute of script-driven fields, an appearance redraw and
a flatten. Nothing here adds, renames or moves a field.

**One exception, stated rather than left to be discovered:** the [`groups`]
section removes a *field group* — a name the form files fields under — and
every field beneath it. It is here and not on an authoring surface because a
grouping node is reachable from nowhere else: it has no widget to click on
the page, no row in the fill list, and no entry in the tab order, so the
Properties pane can never be pointed at one. See that module's header.

## This panel writes `/V`, and that changes nothing about the discipline

Most of [`crate::panels`] is a report, and [`crate::panels::layers`] changes
what is *drawn* without changing what would be *saved*. This one writes to
the document.

The body is handed `&OpenDoc` — a **shared** reference, so this is a
compile-time fact and not a convention — it reads, and it raises a
[`crate::app::actions::Action`]. The only thing peculiar to this panel is
that the action reaches an `EditSession` verb at the far end; see [`edit`]
for the four-step mutation protocol that makes that safe, and for why
nothing travels back.

## Rule 4: disclosure lives off-canvas

`D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`'s rule 4 in one
clause: *"Disclosure lives off-canvas: a status line, a results panel, a
report after the command, a properties field."* **A panel is the right
home**, and this panel is nothing but disclosure and controls — it draws
not one pixel on the page, and it must not start to. The one-line test is
*would a screenshot of the editing canvas differ from a screenshot of the
same document saved and reopened?*

The **spotlight** is the one thing this panel puts on the canvas, and it is
permitted rather than an exception. Rule 4's fourth clause allows *"a snap
indicator, a hover highlight, a rubber-band, a selection handle — these are
the cursor"*, and a spotlight is exactly that: transient, following the
operator's attention, gone the moment they look elsewhere. It answers *"which
of these is the one I am about to type into?"* It travels through
[`spotlight`], which is a channel and nothing more — this panel still writes
no content to the page and must not start to.

## The page is a second way in, and this panel is the accessible one

[`crate::canvas::forms`] lets an operator click a field where it is drawn
and type into it. The two surfaces share [`rows::block_reason`],
[`rows::commit`] and the whole [`edit::FormEdit`] vocabulary, so there is one
rule for what is fillable, one rule for when a draft is written, and one
place a form verb is called.

Two of this panel's obligations exist **because** the page is fillable, and
both are disclosure:

1. **[`fill_disclosure`]** — the two things a fill decides that the document
   cannot afterwards be asked: an auto-size pdfcer chose, and characters it
   replaced. Six of `FillOutcome`'s eight facts are re-derivable from the
   next frame's re-read of the document; these two are not, because in the
   saved file they are indistinguishable from an author's decision. See
   [`edit`]'s header.
2. **[`canvas_routing`]** — which fields the page cannot be clicked for, and
   why. Without it the canvas silently shrinks the capability from the
   operator's point of view.

**This panel is the accessible surface, and that is not a courtesy.** Its
rows are real widgets with tab order, AccessKit exposure and `/TU` labels; a
box projected onto a page raster has none of those, because the thing
underneath it is a picture with no text alternative.

**Two facts are disclosed BEFORE the edit rather than after it**, because
both are knowable before anything is typed and a note that arrives after the
click is a note the operator can only act on by undoing:

- *this form carries an XFA packet, so a fill may not stick* — one line above
  the list, read from `AcroForm::xfa`, a property of the FILE.
- *this check box has no appearance for the state you selected* — the control
  is **disabled**, because core would refuse the call. See [`rows`]' header.

## Two counts that are not the counts to display

The README's third bite — *"a returned count is not always the count to
display"* — lands twice in this panel, and neither instance is the worked
example it uses:

1. **"N you can fill here"** is derived from what this panel will actually
   draw a live control for, **not** from `Field::is_fillable`. The model's
   predicate knows about read-only, signature and push-button fields; it
   does not know that a certification signature disables the whole document
   or that a rich-text field is offered a conversion rather than a box. See
   [`crate::text::forms::forms_field_count`].
2. **`AcroForm::fields.len()` is not the number of fields in the file.** It
   excludes `inline_field_roots` — `/Fields` entries written as direct
   dictionaries, which Table 218 forbids and which have no object identity
   a fill could write to. Disclosed rather than silently absorbed, because
   an operator comparing pdfcer's count against another reader's needs to be
   able to find out why.

## JavaScript is never executed

A standing project rule, not an unfinished feature. Script-driven fields
are **recognised** (`Field::has_additional_actions`, surfaced as a
disclosure above the list) and a whitelisted subset of Acrobat's built-in
calculations is **recomputed natively** by
`pdfcer_core::form_script::recompute` — arithmetic pdfcer reproduces itself,
never a script it ran.

The Calculated Fields section carries two rule-4 disclosures: a **derived
evaluation order** when the form fails to list its calculated fields in
`/CO` (pdfcer inferred something, and another reader may compute different
values), and **coerced operands** where a blank or non-numeric input counted
as zero. Skips are listed **before** the changes, because a field pdfcer
declined to compute is the thing an operator most needs to notice and a list
of successful changes above it reads as completeness.

The section is collapsed by default and **never auto-runs**: merely opening
a form must not change a computed `/V`.

## What was deliberately left behind

Everything that is `Edit ▸ Forms` **authoring**: field creation, field
deletion, widget deletion, field renaming with its ancestor breadcrumb, and
the grouping-node roster. Also the FDF/XFDF/CSV import and export surface,
which needs a file dialog this stage does not have. Each answers to a
different ribbon command and, in the deletion and renaming cases, to a
**different certification gate** — see
[`crate::text::forms::forms_structural_certification_disabled_tooltip`].
They land with the commands that name them.

## A note on very large forms

`pdfcer_core::forms::MAX_FORM_FIELDS` is 500,000, and this panel lays out
every row inside one `ScrollArea` — so a pathological form would lay out
half a million rows per frame. Not addressed, and stated rather than
discovered: the fix is `ScrollArea::show_rows`, which needs a uniform row
height, which these rows do not have (a multiline text field, a radio
cluster and a one-line combo are three different heights). Every real form
measured in `pdfcer-core`'s corpus is under a thousand fields.
