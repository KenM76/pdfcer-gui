# `panels::properties::fieldedit` — a placed form field's properties, and
the controls that change them


## The sentence this module deletes


> ~~Required, read-only, the tooltip and the border can only be set when a
> field is placed. **To change one, delete this field and place a new one.**~~

`EditSession::edit_field` landed the **same day**, three commits before the
revision this shell compiles against, and the engine wrote a full pane
design brief into the request channel saying so. Nothing consumed it.

So the program spent a day telling an operator to perform a **destructive
workaround** for a capability it already had — and delete-and-replace is
genuinely destructive: it loses the field's name, its filled value and its
place in the tab order, every one of which an FDF import or a filling script
keys on.

The lesson is not *grep harder*. The claim was **true when it was
written** and false within hours, because it was an absence claim about a
crate this project does not build. Such a claim has a shelf life. What
catches it is reading the reply, and the reply was sitting unread in
`open/`.

## SCOPE — field or widget — and getting it backwards is invisible

The engine took this verbatim from Acrobat's own scripting model, and it is
the decision that shapes the whole pane: some properties *"apply to all
widgets that are children of that field"*, others *"are specific to
individual widgets"*.

| scope | verb | properties |
|---|---|---|
| **field** — one write, every placement | `edit_field` | required, read-only, tooltip, multiline, password, comb, max-len, no-toggle-to-off, radios-in-unison, combo, editable, multi-select, sort, options |
| **widget** — per placement | `edit_widget` | rect, border, visibility, caption |

> **Getting this backwards is invisible on the ordinary one-widget field
> and wrong on every radio group** — where "the border" can only sensibly
> mean one button and "required" can only sensibly mean the group.

This module holds the **field** half. The widget half is the next piece of
work and is named in `FEATURES.md` rather than left as a silence.

## One press is one undo entry, and one exception the standard forces

Every control here sends a `FieldEdit` naming **one** property, though the
struct can carry fourteen. That is `StyleChange`'s rule for its reason: a
pane that batched a flag and a max-length into one request would make
`Ctrl+Z` after two presses take back a state the operator never saw.

The exception is not a batch. Table 228 permits `Comb` only when
`/MaxLen` is present, and the engine checks its gates **against the
resulting field** — so turning comb on for a field with no max-length must
send both in one edit or be refused. That is one act the standard makes
indivisible, not two the pane chose to combine.

## The refusals arrive from a direction the request does not name

The engine's §6, and it is the part most likely to produce a confusing
message:

* `.with_max_len(None)` on a **comb** field → `CombPreconditionUnmet`,
  naming comb, which the request never mentioned;
* `.with_combo(false)` on an **editable** drop-down → `ChoiceEditWithoutCombo`.

Its instruction: *"show it against the control the operator touched, not the
one the standard named."* So every action carries a `touched` label, and the
decline names the control that was pressed.

## There is no type change, and there is nothing to grey

Acrobat has offered no field-type conversion since Acrobat 6; the only route
is delete-and-recreate. pdfcer models the same limit by making the request
**unrepresentable** rather than by returning an error, so this pane has
nothing to disable — the property does not exist. Said here because "why is
there no Type control" is the obvious question and an absence with no
explanation is indistinguishable from an oversight.

## Rule 4

Nothing here marks the canvas. Every disclosure — a value that no longer
fits its own limit, a `Sort` claim the list does not meet, three widgets
changed by one write — lands in the status bar through
`app::actions::forms`. The field renders exactly as the saved file will
render it.

## Item notes

### `fn flag_row`

# It reads its state from the DOCUMENT, not from a draft

The `checked` argument is `field.flags`, re-read every frame from the
session. There is no local copy to go stale, and the visible consequence is
the right one: a press that the engine **refuses** leaves the box where it
was, because the document did not change. A draft-backed checkbox would
show the operator's intent and the document would disagree with it silently
— which is the "the control does nothing" report with an extra step.

`PanelsState` therefore holds no boolean for any of these, and the two
controls that *do* need a draft — the tooltip and the max-length — are the
two that take typing.

### `fn max_len_row`

Zero means **absent**, and it is spelled that way rather than with a
separate "limit the length" checkbox, because a spinner at zero and an
unchecked box beside a greyed spinner say the same thing and the second
costs a control. `/MaxLen` of zero is not meaningful in a file — a field
that accepts no characters is not a field — so the value is free to carry
the absence.

### `fn comb_row`

Table 228 permits `Comb` only when `/MaxLen` is present, and the engine
checks its gates **against the resulting field** rather than against the
request. So turning comb on for a field with no max-length must send both or
be refused with `CombPreconditionUnmet` — a refusal naming a property the
operator never touched.

The pane sends both. `.with_comb(true).with_max_len(Some(n))` is explicitly
accepted by the engine, and `n` is whatever the spinner above holds, or a
default when it holds nothing: a comb field needs a cell count and there is
no honest way to have one without it.

This is **not** a violation of "one press, one undo entry". It is one act
that the standard defines as two writes, which is a different thing from a
pane choosing to batch two acts.

### `const DEFAULT_COMB_CELLS`

Ten, and the number is a **disclosed guess** rather than a right answer:
there is no way to know how many cells the operator wants, `/MaxLen` is
mandatory for comb, and refusing the press would mean a checkbox that
cannot be ticked on the majority of text fields. Ten is a postcode, a phone
number and a part number, and it is immediately editable in the spinner
directly above the control that set it — which is what makes a guess
acceptable here and not elsewhere.

### `fn tooltip_row`

A draft and a button, not a live write — `super::formfield`'s rename row
makes the argument and it is identical here: a `TextEdit` bound straight to
the field would author one `edit_field` per keystroke, each one a real,
separately undoable change.

**Empty commits `TooltipChoice::Declined`, which REMOVES `/TU`**, and that
is the engine's instruction rather than this pane's choice: *"an empty `/TU`
would be worse than none, because a screen reader announces the empty name
instead of falling back to the field's."*

### `fn default_value_row`

# Why this is worth a control at all

This shell already ships Reset — `FormEdit::Reset`, and `set_button_action`
can author a `/ResetForm` button — and until `Pass 264`-era `FieldEdit`
gained a writer, **pdfcer could not set a default value on any field it
authored**. §12.7.5.3 resets a field to its `/DV`, and a field with no `/DV`
resets to *nothing*.

⇒ So on a form pdfcer made from scratch, Reset emptied every field. That is
**spec-correct behaviour** and it is not a defect in Reset — it is a missing
authoring control, which is what this is. The engine's own note put it
exactly: *"a reset without a writer could only ever restore some OTHER
application's defaults."*

Same draft-and-commit shape as [`tooltip_row`], for that function's stated
reason: a `TextEdit` bound straight through would author one `edit_field`
per keystroke, each separately undoable.

**Empty clears `/DV` rather than writing an empty string**, and the two
are genuinely different: an empty `/DV` is a default *of nothing*, which
Reset restores by blanking the field; no `/DV` is *no default*. Both blank
the field on Reset today, so the distinction is invisible now — but it is
the difference between a form that states its defaults and one that does
not, and `clearing_default_value` exists precisely so a caller can say
which. Writing `""` where the operator meant "remove" would leave the
document asserting something it was never told.

### `fn alignment_row`

# Three named choices, not a number, and the refusal is unreachable

`FieldEdit::with_quadding` takes an `i64` and the engine **refuses** anything
outside `0..=2` by name (`EditError::QuaddingInvalid`) rather than clamping —
which is the right shape for an API and would be the wrong shape for a
control. A spinner over `i64` would offer a press that fails.

⇒ So this offers exactly the three `Quadding` has, and passes `code()`. The
refusal cannot fire from here **by construction**, which is a better answer
than wording it: R9's *"an unavailable capability renders nothing"* applied
to a failure mode rather than to a feature. If a future caller can produce
an out-of-range `/Q`, that caller owes the sentence — this one cannot.

**No draft, and that is deliberate.** [`FieldPropsDraft`]'s own doc gives
the rule: a draft exists only for controls that take *typing*, because every
other control reads the document each frame and a refused press therefore
leaves the control where it was. A three-way chooser is a press, not typing.

The list is `ALL_QUADDINGS` rather than three literals, for
`markup::ending_chooser`'s stated reason — and
[`the_alignment_list_covers_every_variant_the_engine_has`] is what stops it
drifting, by matching an exhaustive set with no wildcard so a fourth
justification fails to compile here rather than going quietly unoffered.

### `fn text_appearance_rows`

# Three controls and ONE struct, so every press re-states the other two

`FieldEdit::with_appearance` takes a whole `FieldAppearance` — font, size and
colour together — because `/DA` is one string and there is no way to write a
colour into it without writing a `Tf` beside it. So picking a colour here
re-sends the face and the size the file already had.

⇒ The failure that shape invites is a control that silently resets the other
two to whatever this pane thinks a default is. [`current_appearance`] is the
single place the existing values are recovered, and all three rows start
from it.

It is still one property per press as far as the operator is concerned,
which is what `StyleChange`'s rule is about: one gesture, one undo entry, and
the two values nobody touched come back unchanged.

## The whole group disappears over an ink pdfcer will not narrow

A `/DA` may set its colour in a space this engine models as a flag rather
than a value — `/Separation`, `/DeviceN`, `/ICCBased`. There is no
`TextColor` that round-trips one, so **any** write from this pane would
replace it with black: not a refusal the operator could see, a silent
narrowing written into their document. So the rows render nothing and say
why, which is R9 applied to three controls at once rather than to one.

### `fn font_row`

A document's own embedded face is the **selected** entry and is not in the
list, because `FieldFont::Resource` is refused by name unless the key is
already in `/AcroForm` `/DR` `/Font` — so this pane can offer a key it read
out of this field and cannot offer one it made up. Picking any of the
fourteen replaces it; there is no route back, and that is the file's
property rather than this control's limit.

### `fn text_size_row`

Committed on release or on losing focus, never on `.changed()`, for the
reason [`max_len_row`] states: a drag across the spinner would otherwise
author one `/DA` rewrite per pixel, each one separately undoable.

### `fn text_colour_row`

A four-ink separation gets the sentence and no control, for
`text::panels::formfield::text_colour_unshowable`'s stated reason — showing a
converted approximation would put a colour on screen the file does not
contain, and the first nudge of the picker would commit pdfcer's guess. The
font and size rows above still work on such a field, and re-send the CMYK
ink unchanged.

### `enum Face`

Two variants rather than reusing `FieldFont`, which is `#[non_exhaustive]`
and therefore forces a wildcard arm on every match — and a wildcard here
would mean a future engine variant silently becoming Helvetica. This enum is
exhaustive, so a third case would fail to compile instead.

### `fn current_appearance`

`Field::default_appearance` is already the **inherited** value — `forms`
falls back to `/AcroForm` `/DA` when the field states none — so this reads
what the field actually draws with, not only what it states.

⚠ A field with no `/DA` anywhere, and one whose `/DA` does not parse, both
land on Helvetica at auto size in black. That is a guess, and it is the one
guess this pane cannot avoid: `with_appearance` writes all three or none,
and a field with no `Tf` is one §12.7.3.3 already calls malformed.

### `fn face_of`

# Why this table is here and not a call into the engine

`pdfcer_core::fontdata::std14_by_base_font` takes a **`BaseFont` name** —
`Helvetica`, `Times-Roman` — and a `/DA` carries a **resource key**, which by
Acrobat's long-standing convention is a four-letter abbreviation: `Helv`,
`TiRo`, `Cour`, `Symb`, `ZaDb`. The engine's own resolver for those is
`pub(crate)`, so it cannot be called from here.

⇒ The canonical spellings still go through the engine's function, so the
fourteen names are stated once. Only the abbreviations are local, and a key
this table does not know becomes [`Face::Embedded`] — which is the right
answer for one anyway, because it is then re-authored verbatim.

### `fn appearance_edit`

`FieldAppearance` is `#[non_exhaustive]`, so a caller outside `pdfcer-core`
cannot build one with a struct literal — the two constructors are the only
route, and they are what selects between the two `FieldFont` variants.

### `fn swatch_rgb`

CMYK is refused rather than converted, O202 decision 4: pdfcer owns no
rendering intent for a form field, so a converted swatch would show a colour
the file does not contain.

### `fn sync`

# The stamp is the name AND the epoch, and both are load-bearing

**The name**, because clicking a second field must not leave the first
field's tooltip sitting in the box waiting to be applied to the wrong
one — the failure `super::formfield`'s rename draft stores a key to
prevent, and the reason that draft stores a key at all.

**The epoch**, because a property edit is an edit: after committing a
max-length the document holds a new value, and a draft that did not
re-read would show the pre-edit number for ever after the first change.
That is the failure that makes a properties panel untrustworthy, and it
is the same term `super::text`'s `TextStyleDraft` carries for the same
reason.


The refactor was forced by a test and is right on its own: `Field` has
no `Default`, so a unit test could not build one without a document —
and this function reads exactly two things off it. Taking those two
makes the dependency honest and the staleness rule testable in
isolation, which is the same move `app::actions::forms::disclosures`
made and for the same stated reason.

[`Self::read`] is the one place the two are pulled off a real field, so
there is still exactly one statement of *where a tooltip lives*.

### `fn read`

`/TU` is `alternate_name`, **not** a field called `tooltip`. The
standard's own name for it is *alternate field name*; every application
calls it the tooltip because that is where a reader shows it and what a
screen reader announces. Raw bytes (§7.9.2), decoded the way every other
operator-visible name in this crate is.

### `fn the_alignment_list_covers_every_variant_the_engine_has`

# Why a `match` and not `assert_eq!(ALL_QUADDINGS.len(), 3)`

A length check passes when a variant is *replaced*, and passes when the
list holds the same variant three times. What this needs to know is that
the list **covers the enum**, which only the compiler can answer — so the
match below is exhaustive with **no wildcard**, and each arm asserts the
list contains that variant.

⚠ This is the shape `markup::ending_chooser`'s list already uses for the
same reason, and it is the direct answer to a defect this project has on
record: **a hand-written list inside a completeness test is the gap** —
a new variant is invisible to the check built to find it, and the count
still adds up.

### `fn every_offered_alignment_is_a_code_the_engine_accepts`

The engine refuses anything outside `0..=2` by name instead of clamping.
That is right for an API and wrong for a control, so this pane offers the
enum rather than a number — and this test is the statement that the
mapping is total and in range.

### `fn a_draft_is_reseeded_when_the_selection_moves`

The failure this stamp exists for, and it is the expensive one: the
operator types a tooltip, clicks a different field without committing,
and the pane is now holding the first field's text over the second
field's name. Pressing Enter would write it to the wrong field, and
nothing on screen would have said so.

### `fn an_edit_to_the_same_field_reseeds_the_draft`

Without the epoch in the stamp, committing a max-length would leave the
spinner showing the number it had *before* the commit — and it would
stay there, because the name has not changed. The operator would see
their own edit not take.

### `fn section`

Returns whether it drew, which is always `true` when a field is selected:
**every** field type has required, read-only and a tooltip, so there is no
field for which this section is empty. The type-specific rows come and go.

### `struct FieldPropsDraft`

# Why only two properties have a draft

Because only two take typing. Every checkbox reads `field.flags` straight
from the session each frame, which is what makes a refused press leave the
box where it was — see [`flag_row`]. A draft for a boolean would show the
operator's intent while the document disagreed with it, silently.
