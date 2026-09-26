# `panels::forms::groups` — the Field-groups section, and the shell's only
route to deleting one

A third list in the Forms panel, beside the fill list and the Tab-order
section, answering a question neither of them can: **what names does this
form file its fields under, and what would happen if one of them went?**

## What a field group is, and why the panel had no way to reach one

`AcroForm::groups` is the field-name tree's *interior* — `Personal` and
`Personal.Address` in `Personal.Address.Zip`. §12.7.3 gives such a node
existence only as a link in a `/Parent` chain: **no type, no value, no
widget, no rectangle.** It is drawn nowhere, in any viewer, ever.

Which is exactly why the fill list cannot list it (it has nothing to fill),
the Tab-order section cannot list it (it has no `/Annots` entry), the canvas
cannot select it (it has no box to click), and the Properties pane cannot
describe it (`doc.selected_field` is populated by a click on a widget, and
there is no widget). Until this section, `EditSession::delete_field_group`
was an engine capability with **no surface in the shell at all** — the
finding that produced this work.

## ★★★ Everything this section does is disclosure, because everything its
verb does is invisible

Rule 4 says *"disclosure lives off-canvas: a status line, a results panel, a
report after the command, a properties field"*, and a panel is the right
home. On this verb it is more than the right home — it is the **only**
evidence. Deleting `Personal` produces:

- the same page, pixel for pixel, at every zoom;
- the same raster, the same print, the same export;
- a fill list some rows shorter, if the operator happens to be looking at
  it and happens to remember how long it was.

⇒ So this section says what would go **before** the press, in numbers and
names, and the funnel says what did go **after** it, from the engine's own
report. Neither is optional and neither substitutes for the other: the first
is a decision, the second is a receipt.

## ★★ The two-press protocol, from this side of it

| press | raises | changes | draws |
|---|---|---|---|
| **Delete group…** | `FieldAction::ArmGroupDeletion(Some(name))` | nothing | the disclosure block, next frame |
| **Delete N fields** | `FieldAction::DeleteGroup { group }` | the document, one undo entry | the status bar's receipt |
| **Cancel** | `FieldAction::ArmGroupDeletion(None)` | nothing | the row, plain again |

The preview runs in the funnel and not here, and that is a compile-time fact
rather than a convention: `field_group_deletion_preview` takes `&mut self`,
this body is handed `&OpenDoc`, and the session lives behind an `Arc`. See
[`crate::app::actions::forms::groups`] for the whole argument, including why
this is not a modal dialog.

## ★★★ R83 — the refusal is asked BEFORE any control is drawn

`EditSession::deletion_refusal` is a pure query and this section asks it
once, at the top, exactly as [`super::body`] asks `fill_refusal` and
`flatten_refusal` for their own controls. On a certified or encrypted
document it renders **the sentence and no controls at all**.

That is R9 rather than greying, and the distinction is the one R9 draws:
greying is for a capability that is *temporarily* unavailable and is always
explained on hover. A certification signature is not temporary and cannot be
argued out of — so a greyed Delete-group button would imply a state the
operator could reach, and would hide the explanation behind a hover they
have no reason to make.

**And it is a sentence, not a silence.** Rendering nothing at all here would
be indistinguishable from a feature nobody built, on a panel that lists the
groups either way.

★ It asks `deletion_refusal`, not `flatten_refusal` and not `fill_refusal`.
The three are different questions with different answers on documents that
are not exotic — `super`'s body carries the measured account of that — and
core's own doc comment names the hazard precisely: *"a call site that asks
the wrong question is correct only until the answers diverge, at which point
it is wrong silently, in a control that stays enabled while its verb
refuses."*

## Where the section sits, and why

Immediately after the Tab-order section and **above the fill list**. Two
constraints decide it and neither is taste:

1. **Anything below the fill list is unreachable.** That list's own
   `ScrollArea` takes the rest of the pane, and the panel's top level does
   not scroll, so content after it is laid out past the bottom of a
   container with no way to get there. This panel has already shipped that
   defect once, measured in a driven run at y=773 in a body ending at y=770.
2. **It belongs beside the other structural surface.** Tab order lists
   controls the form does not claim and offers to register them; this lists
   names the form files fields under and offers to remove them. Both are
   about the form's *shape*; the fill list is about its *contents*.

## Rule 4: this section draws nothing on the page

Not one pixel. No highlight over the widgets of a group under the pointer,
no badge, no outline. The one-line test — *would a screenshot of the editing
canvas differ from a screenshot of the same document saved and reopened?* —
answers no, and must keep answering no.

Worth naming what rule 4 would *permit*, so nobody reads the absence as a
prohibition: highlighting the widgets beneath the group under the pointer is
the fourth clause's *"a hover highlight … these are the cursor"*, and it
would be a genuinely good affordance for a verb this invisible. It is not
built for the reason [`super::tab_order`] gives for the same wish: the
panel→canvas channel for *which row is hovered* does not exist in this
build, and `crate::canvas` is not this module's to extend.

## `PDFCER_DIAG` proves what this computed

One `form-groups` census line per frame the section is reached — carrying
the node count and whether the document refused — and one `form-group-row`
line per row, capped. Written whether or not the collapsing header is open,
so the listing is provable from a trace without anyone having to click.

That matters more here than on a visual surface: a screenshot of this
section cannot tell you that a node the file carries is missing from the
list, or that the refusal query was never asked. Both are in the trace.
