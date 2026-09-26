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

## Item notes

### `fn header`

**Above every control, without exception.** Each of these describes a
condition under which what the operator sees here and what a different
viewer shows can legitimately disagree, and a caveat below a list arrives
after the operator has already drawn a conclusion.

The order is by how much it changes what the operator should do:
the refusal first (nothing below it will work), then the two rendering
divergences, then the scripts, then the malformed entries.

### `fn fill_disclosure`

Rule 4 in one clause: *"disclosure lives off-canvas — a status line, a
results panel, a report after the command, a properties field."* This is
the panel half of that, and it is the half this build can reach: a status
line would be better still for a fill made by clicking the **page**, when
this panel may not even be open, and wiring one is a change to
`crate::app::status` that this work does not own. Named rather than
silently absent — see [`edit::last_fill_disclosure`].

# Why these two, and only these two

Six of `FillOutcome`'s eight facts are derivable from the document the panel
re-reads on the next frame, which is why they are discarded. These two are
not: an auto-size pdfcer chose and a character pdfcer replaced both look, in
the saved file, precisely like an author's decision. Re-reading the field
cannot distinguish them, so there is nothing to derive and the note has to be
carried.

Shown only while it describes the revision on screen, which is what the
epoch comparison inside [`edit::last_fill_disclosure`] is for: an undo moves
the epoch past the disclosure and the sentence stops being drawn, with
nothing anywhere that has to remember to clear it.

### `fn canvas_routing`

`crate::canvas::forms` lets the operator fill a form by clicking it. Five
reasons a field is not offered there are listed in that module's header, and
every one of them ends in *"fill it in the panel"* — so the panel is where
they have to be said, or the capability has silently shrunk from the
operator's point of view and they have no way to find out why.

# The counts come from the canvas's own walk, and must never be re-derived

[`crate::canvas::forms::placed`] is **the** classification of this form —
the one whose boxes the canvas hit-tests — and it hands back
[`crate::canvas::forms::boxes::Routing`] from the same pass. This panel reads
that and does not repeat the rule.

Repeating it is wrong twice over. Once for the ordinary reason: two
statements of one rule drift, and a panel promising "3 fields can only be
filled here" over a canvas that declined four is worse than no count. And
once for a reason no review catches — a re-derivation naturally asks each
widget's `/P` to work out which page it is on, which is the silent defect
[`crate::canvas::forms::boxes::place`] documents and which **no test against
the fixture corpus can reach**.

It is also free: the walk is cached per `(document, revision)` and the
canvas has already paid for it this frame.

Conditional, so it is a signal rather than furniture: a form every one of
whose fields can be clicked says nothing at all.

### `fn offers_a_control`

**The predicate behind the count line**, and it is deliberately expressed
as "will a control be drawn?" rather than as a copy of the row dispatch,
because the two would drift. It is exactly the negation of
[`rows::block_reason`] plus the rich-text case, which is offered a
*conversion* rather than a box and so is not somewhere the operator can
type today.

[`tests::the_fillable_count_agrees_with_what_the_rows_offer`] pins it
against `block_reason` itself so the two cannot come apart.

### `fn calculated_fields`

Above the field list and below the document-wide disclosures, because a
recompute acts on the whole form and because its result changes what the
rows below it show. An operator who scrolled past this and then read a
stale total would have been misled by the layout.

**Collapsed by default and never auto-run.** Merely opening a form must not
change a computed `/V`. The plan is computed on every frame the section is
open — cheap on any real form, and always current with the fills the
operator just made, which a cached plan would not be.

### `fn reset_section`

Beside the recompute section and collapsed for the same reason, but the
disclosure is doing more work here: a recompute writes a number the
operator can check, a reset **destroys what they typed**. So the section
lists every field it would clear, with its current value, before offering
the button — the loss is what has to be on screen, not the outcome.

# The preview comes from core, and is filtered here

`EditSession::reset_preview` returns a row for **every** field in scope,
including ones that are ineligible and ones that already hold their reset
value; core's own doc calls filtering the shell's job. That is the third
bite again in miniature — `preview.len()` is not the number to display, and
the number that matters is the count of rows with `would_change` set, which
core pins as equal to `ResetOutcome::fields_reset`.

Re-deriving the preview here instead would be a second reset algebra beside
the engine's, and the two would disagree the first time `/DV` inheritance
changed.

### `fn whole_form_controls`

**Placed above the list, not below it**, because they act on everything
beneath them: a control that acts on everything below it belongs above it,
and a Flatten button under a forty-row list is a button an operator scrolls
past without meeting.

# Redraw comes first, and the order is load-bearing

Flatten works by invoking each widget's **existing** `/AP` as a page
XObject. A field with no drawn appearance has nothing to invoke, so
flattening burns nothing for it and then removes the field — the typed
value disappears from the visible page. Core's own guidance is to
regenerate first, and this panel both orders the buttons that way and says
so, conditionally, when the document actually has fields at risk.

### `fn field_list`

The scroll area wraps **only** the rows. Everything above it — the
disclosures, the two review sections, the whole-form controls — stays put,
because a disclosure that scrolls out of sight while the operator works
through a long form has stopped disclosing.

### `fn raise`

A function of its own rather than nine inline `push` calls, so the mapping
from this panel's vocabulary to the action funnel is stated once.

It does nothing but wrap. The mutation protocol, the epoch bump, the texture
invalidation and the refusal trace all live in [`edit::apply`], for the
reasons that module's header sets out — chiefly that the six form outcome
types do not unify into `vector_edit`'s `Result<Vec<String>, EditError>`.

### `fn id`

One id for the whole panel rather than one per field: the drafts are a
single coherent unit keyed on one document revision, and splitting them
would mean the key had to be checked per field.

### `fn prune`

The epoch key already catches an edit, so this exists for the case the
key cannot see: a field that was never in this form to begin with,
which is reachable when the same path is reopened after being changed
elsewhere. Cheap, and it stops the map growing without bound across a
long session.

### `fn the_forms_command_is_reachable_from_the_ribbon`

The failure this defends against is invisible to every other kind of
check: a panel can have a `PaneSubject`, a body, a rail entry and a
diagnostic step, be driven successfully by the harness through all of
them, and still offer no control an operator could click.

Two assertions, and both are needed. A command **the manifest
references** is one the ribbon draws a control for; a command **the
registry holds** is one that has a label, a tooltip and an enable
predicate. Either alone is half a control.

`crate::panels::tests::every_panel_is_reachable_from_the_ribbon` sweeps
`Panel::ALL` and covers this panel too. This is the belt to that sweep's
braces, and deliberately the *same* two assertions, so the duplication is
clean rather than divergent.

### `fn the_fillable_count_agrees_with_what_the_rows_offer`

The whole of the third bite, pinned. [`offers_a_control`] and
[`rows::row`]'s dispatch are two statements of one rule, and the
failure when they drift is silent: a count line promising twelve
fillable fields above twelve disabled boxes.

Asserted against `block_reason` — the function the row actually calls —
rather than against a re-derivation, so the test cannot pass by
agreeing with a third copy of the rule.

### `fn a_revision_change_forgets_every_draft`

The defect this prevents, in full: the operator types "Anna", tabs away
so it commits, then presses Ctrl+Z. The document reverts to empty. If
the draft survived, the panel would show "Anna" in a box over a field
holding nothing — disagreeing with the document it is describing, and
arming a re-commit of the value that was just undone.

Exercised through the real key, without an egui context: [`FormsUi`]'s
key comparison is the whole mechanism, and it is a pure comparison.

### `fn a_draft_for_a_departed_field_is_dropped`

The case the epoch key cannot see — the same path reopened after being
changed elsewhere — and the thing that stops the map growing without
bound across a long session.
