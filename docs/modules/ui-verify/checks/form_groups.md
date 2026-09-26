# `ui-verify/checks/form_groups`

`form_groups` — **the two surfaces an audit found the engine had shipped and
this shell had never grown**, driven against the real binary.

# What this module is about


| engine | shell, before |
|---|---|
| `delete_field_group` + `field_group_deletion_preview` | **no route at all** — a grouping node has no widget to click, no row in the fill list and no entry in the tab order, so nothing could name one |
| `rename_refusal` (and, it turned out, `deletion_refusal`) | **consulted by nothing** — Rename and both Delete controls were drawn live on every document, including one that refuses them |

Two checks, because the two are different claims about different surfaces and
a reader who sees one fail should not have to work out which half is broken.

# Why these need DRIVING and cannot be settled by a unit test

Both defects are of the shape this harness exists for, and neither is
observable from inside the crate:

- **The group route** is four links — a section that only draws when
  `AcroForm::groups` is non-empty, a button that raises an action, a funnel
  arm that runs a `&mut self` preview, and a *second* button that only
  exists once the first has been pressed. Every link has unit tests. The
  thing that ships broken is the wiring, and a two-press protocol is the
  most wiring any form verb in this shell has.
- **The refusal** is an *absence*: on a certified document the correct
  behaviour is that no control is drawn. A unit test asserting "the function
  returns Some" passes on a build where nothing calls the function — which
  is precisely the state the audit found, under 2,538 passing tests.

⇒ So [`FieldGroupDeleteRemovesTheSubtree`] presses the buttons, and
[`StructuralRefusalsAreSentencesNotControls`] reads the *absence* of two
regions beside the trace line that proves the gates were asked. That second
pairing is `crate::checks` rule 4 discharged: never treat an absence as
evidence unless you have shown the thing that would have produced it was
working.

# The fixtures, and why each is the only one that would do

| fixture | what it carries | why nothing else works |
|---|---|---|
| `forms/nested-form.pdf` | `Personal.Name`, `Personal.Address.City`, `Personal.Address.Zip` | **The only fixture with grouping nodes at all.** Core's own note on `AcroForm::groups` says it is empty for a flat form, *"which is every file in the Pass 7.0 census"* — and the two-level shape is what makes the **cascade** observable: deleting `Personal` also empties `Personal.Address`, a node nobody named |
| `../../fixtures/certified-nested-form.pdf` | `/DocMDP` at **`/P 2`**, over the SAME two-level tree | Certified **and** nested, which is the intersection nothing else in either corpus occupies. Filling permitted, restructuring refused. The engine's `PROVENANCE.md` says why `/P 1` would not do: it refuses *everything*, so a check written against it *"passes whether or not those gates differ at all"* — the fill controls would be gone too, and the check could not tell a correct build from one that disables the whole panel |

**The second fixture was `forms/certified-p2-form.pdf` and could not
serve.** That file is certified at the right `/P` and its fields are
**flat** (`FullName`, `Subscribe` — no dots), so `AcroForm::groups` is empty
and `panels::forms::groups::section` takes its early return before laying
out a single control. Phase F's arm-withholding assertion — *the section
draws no `forms.groups.arm.*` control* — was therefore true of a section
that never drew: an absence with nothing behind it, which `crate::checks`
rule 4 forbids. Every certified file in either corpus was flat
(`certified-comments.pdf`, `threaded-comments.pdf`, `certified-p2-form.pdf`)
and the only nested one (`nested-form.pdf`) was uncertified, so the fix was
a **fixture**, not a rewrite: `tools/gen-certified-nested-fixture.py` copies
`nested-form.pdf`'s field tree under `certified-p2-form.pdf`'s
certification. Its header carries the whole argument, and
`crates/pdfcer-gui/src/app/actions/forms/delete.rs`'s
`the_certified_nested_fixture_is_both_certified_and_nested` pins the four
properties it has to keep — it loads, `deletion_refusal` is `Some`,
`AcroForm::groups` is non-empty, and `fill_refusal` is `None`.

# Phases

## `field_group_delete_removes_the_subtree` — `nested-form.pdf`

| Phase | Does | Expected |
|---|---|---|
| A | View ▸ Forms, then open the Field-groups header | `form-groups nodes=2 refused=0` |
| B | read the per-row census | one `form-group-row` per node, none armed |
| C | press the root node's **Delete group…** | `form-group-preview terminals=3 nodes=2`, and the row reports `armed=1` |
| D | press **Delete 3 fields** | `delete-field-group-applied terminals=3 widgets=3 nodes=2` |
| E | re-read the census | `nodes=0` — the subtree and both grouping nodes are gone |

## `structural_refusals_are_sentences_not_controls` — `certified-nested-form.pdf`

| Phase | Does | Expected |
|---|---|---|
| F | Edit mode, View ▸ Forms, then **open** the Field-groups header | `form-groups nodes=2 refused=1`, and — with the body laid out — **no** `forms.groups.arm.*` region |
| G | click a widget the canvas census names | `form-field-selected field=…` |
| H | File ▸ Properties, then read the Properties pane's gate census | `form-field-gates rename_refused=1 delete_refused=1`, and **neither** `properties.form_field.rename` nor `properties.form_field.delete` declared |

**H opens the Properties panel, and that is a correction rather than a
flourish.** Edit's default dock puts Properties and Forms in ONE tabbed
stack, and a tabbed stack draws only its active tab — so phase F's own
`View ▸ Forms` click pushed Properties to the back and
`panels::properties::formfield::section` stopped running. The 2026-08-29
sweep reported *"a field is selected and the Properties pane traced no
`form-field-gates` line"* about a pane the check had itself hidden; the
selection in that same trace is real. See `PROPERTIES_ITEM`.

Phase F is what makes phase H's absences readable: a build that simply
failed to draw the Properties section would produce the same missing
regions, and the `form-field-gates` line — written unconditionally, refused
or not — is what tells the two apart.

**The arm half of F is LIVE, and its history is worth keeping.** It has
been through three states, and each was a worse-looking fix than it sounds:

1. **Asserted unconditionally on a flat fixture** — and passed, having
   checked that a section which never drew drew no controls.
2. **SKIPped the whole check** — the wrong half to cut. `refused` is traced
   ABOVE `groups::section`'s early return, so it is real evidence even on a
   flat form, and phases G and H (the Rename box and both Delete buttons,
   which are what this check's `defect()` actually names) need only a
   certified document with a widget. That left the check reporting SKIP on
   every run: zero coverage, counted as a check.
3. **Conditional, with a note when it did not run** — honest, and still zero
   coverage of the one assertion, because the condition was never true.

⇒ The fixture closed it. `certified-nested-form.pdf` traces `nodes=2`, so
the arm assertion runs on every run — and a run that reports `nodes=0` is
now a **FAILURE** rather than a note, because on this fixture that means
either the panel stopped listing an interior the engine can see or the
fixture was regenerated wrong. Both are red, and a silent pass is the one
outcome this phase must never produce again.
