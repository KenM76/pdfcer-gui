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
