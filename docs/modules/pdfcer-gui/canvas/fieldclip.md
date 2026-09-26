# `canvas::fieldclip` — **cut, copy and paste a FORM FIELD**

## What this closes

**Ken, 2026-08-29:** *"wire the request. ctrl v for paste as new. ctrl shift
v for paste as duplicate."* — `OPERATOR_REQUESTS.md` **O58**.

Before this module there was **no path at all** from a selected form field
to `Ctrl+C`. Not a lossy one, not a refused one — none. The reason is two
deliberate decisions that were each correct and that together left a hole:

1. `canvas::selection::annot`'s exclusion table drops `/Widget` — *"the form
   field surface owns it — a click there focuses an editor, and two owners
   of one press is how a field becomes unfillable"*. A selected field
   therefore lives on `OpenDoc::selected_field`, not in `SelectionState`.
2. [`crate::canvas::clipboard::copy`] reads `doc.selection` and nothing
   else.

So `Ctrl+C` over a form field fell through to the *content* copy, which
looked at an empty content selection and refused with *"nothing is
selected"* — over an object with visible grips around it. That is
`DEFECTS.md` D4a's shape exactly: a refusal whose sentence describes a
different world than the one the operator is looking at.

## The two pastes

Copying a field has two legitimate meanings and `pdfcer-core` refuses to
guess between them. The operator made the decision, and made **both**
answers reachable on two chords:

| chord | [`PasteAs`] | engine policy | the value |
|---|---|---|---|
| `Ctrl+V` | [`PasteAs::NewField`] | `FieldPastePolicy::NewField` | its own |
| `Ctrl+Shift+V` | [`PasteAs::Duplicate`] | `FieldPastePolicy::AdditionalWidget` | shared — type in one, both fill |

Each engine policy **refuses the other's situation by name**: a `NewField`
onto a taken name is `FieldNameTaken` and never a silent merge, and an
`AdditionalWidget` naming a field the document does not have is
`FieldNotFound` and never a silent creation. That matters because *the
difference between an independent field and a linked one is invisible on the
page* — it shows up only when somebody types in one and the other does not
follow.

## This module was rewritten the day it was written, and the rewrite is
the interesting part

The first version **re-authored**: it read the source field into a
`canvas::formfield::Draft` and pushed that back through `add_text_field` and
its four siblings. That worked, and it was lossy in eight measurable ways —
`/DA` (font, size, colour), `/Q`, `/DV`, `/AA`, `/MK` colours, `/BS` styles,
the flags no `New*Field` spec can express, and the baked `/AP`. Each was
*readable* on `forms::Field` and *writable* nowhere, so the shell carried a
hand-written table of what survived and disclosed it on the status row.

`Pass 167.0` shipped `pdfcer_core::formclip` in answer to this project's own
request, **within the hour**, and every row of that table now travels. So:

- The `Lost` enum is **deleted**, not deprecated. The engine's own words:
  *"delete the fidelity table — you should not be maintaining a hand-written
  map of which properties survive, because it rots silently every time we
  add an authoring key. The clip does not express properties, it carries
  them."*
- Two things travel that this shell **did not know to ask for**: the actual
  font its `/DA` names (installed into the destination's `/AcroForm /DR`,
  renamed if that name is taken there, with the `/DA` rewritten to match),
  and `/Ff` as an integer, which brings `DoNotSpellCheck`, `DoNotScroll`,
  `FileSelect`, `RichText` and `CommitOnSelChange` for free.
- **Signature fields are no longer refused.** An *unsigned* one copies and
  pastes normally, which — as the engine points out — hands this shell
  signature-field *authoring* it never had, because there is still no
  `add_signature_field`. A **signed** one is refused at the **copy**, by the
  engine, so the operator learns before spending a placement gesture.

⇒ The general lesson, and it has cost this project three days across three
separate capabilities: **a reply arriving is not a capability landing.** The
engine session works in parallel and answers within the hour; the failure
mode is a shell that files a request, ships a workaround, and never comes
back. This module came back the same afternoon and recovered eight
properties' worth of fidelity by doing so.

## Rule 4 — disclosure is the ENGINE's now, and that is a simplification

A pasted field renders exactly as a saved-and-reopened one would. No badge,
no tint, no "this copy is incomplete" marker anywhere on the page, because
provisional styling is a second rendering path for the same content and two
paths drift.

The half of rule 4 that binds is the off-canvas report, and
`FieldPasteOutcome::disclosures` is where it now lands — a `Vec<String>` the
engine builds, covering a dropped value, dropped actions, a carried
calculation and its `/CO` entry, a **renamed font resource**, an ignored
rectangle size, the tab-order position, a dropped structure-tree link and a
reused accessibility name. This shell surfaces it verbatim rather than
re-deriving any of it, which is the same *one fact, one wording* rule that
removed this module's own merge sentence a few hours earlier.

## Radio groups travel whole, and that changes what the rectangle means

`copy_field` on a radio field carries **every** widget in `/Kids` order with
its own rectangle and export value. On a `NewField` paste the group is
**translated** so widget 0's lower-left lands on the target point; every
widget keeps its size and its offset from widget 0, and **the target
rectangle's size is ignored**. The engine discloses that, and this module
does not try to be cleverer: a best-fit rescale of a radio group into a
rectangle is a guess that looks deliberate, and which button sits above
which is part of the group's meaning.

On `AdditionalWidget` exactly **one** widget is placed even from a
multi-widget clip, because adding all N would give one field several views
with duplicate export values — radio buttons that select together.
