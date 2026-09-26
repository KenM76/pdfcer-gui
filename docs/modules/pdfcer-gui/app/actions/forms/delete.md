# `app::actions::forms::delete` — the two structural delete verbs, and the
gate that is the last door to them

`EditSession` has two verbs for removing part of a form and they are
deliberately different requests:

| verb | removes | the operator pressed |
|---|---|---|
| [`field`] | the field **and every widget it draws**, on every page | *Delete field* in the Properties panel |
| [`widget`] | **one box**, leaving the field — unless it was the last, in which case the engine removes the field too and says so | *Delete this box*, the `canvas.field` menu's Delete, or the Delete key over a widget |

## Why this is a file of its own

R2 (no `.rs` over 1,500 lines) forced the split and the subject boundary
decided where: `super` is the whole form-authoring surface — five `add_*`
specs, fill, import, export, rename, adopt, move — and these two are the
only ones that **destroy** something. Both therefore carry a gate none of
the others needs, and the gate's argument is longer than either verb.

## THE FAILURE MODE THIS FILE GUARDS AGAINST

Clear `doc.selected_field` **before** the engine call and say nothing when
the engine refuses, and on an ordinary certified fillable form — `/Perms
/DocMDP` at `/P 2`, which §12.8.2.2 Table 257 permits *filling* and forbids
*restructuring* — the operator gets:

1. right-click a widget, or press Delete over it;
2. the box stays, because `deletion_refusal` was always going to refuse;
3. the selection vanishes anyway;
4. nothing is said, because `crate::app::actions::apply::vector_edit`'s
   `Err` arm words an un-categorised decline and names no field;
5. **and the Properties panel, which was correctly showing "This document
   does not allow form fields to be removed", goes blank**, because that
   section is drawn from `doc.selected_field`.

⇒ A refused gesture that destroys its own explanation. Two rules answer it,
and both are enforced here:

- **[`refused`]** — a refusal must be a **sentence**, never a silence. R83
  gates the four *controls*, but a gate is a forecast of the engine's guard
  and this verb is where the residue those four cannot cover arrives.
- **[`clear_selection_if_edited`]** — the selection is cleared on **success
  only**, so the panel keeps the field it is describing whenever the
  document did not change.

## Item notes

### `fn refused`

Returns `true` when the caller must not proceed.

THE LAST DOOR, and the one that is not a control. Every *drawn* route to
these two verbs already asks
[`crate::panels::properties::formfield::refuses_delete`] and withholds
itself where it answers `true`: the Properties panel's two buttons, the
condition behind the `canvas.field` menu's Delete, `canvas::keys`' Delete
rung 0, and `app::dispatch::format`'s arm. A gate is a **forecast** of the
engine's guard, though, and there are three ways past all four of them — a
chord bound to `format.delete` (a chord consults no `visible_when`), a
condition gone stale within a frame, and a refusal the query does not
predict.

Left to `crate::app::actions::apply::vector_edit`'s `Err` arm, that residue
gets an un-categorised decline — the floor under every verb, and no
substitute for a sentence that names the field. R83's rule is not *gate the
controls*; it is **a refusal must be a sentence, never a silence.**

Asked here rather than left to the engine's own guard so that the
decline is *worded*: the engine returns an `EditError` into a funnel that
discards it, whereas this returns before the funnel and puts
[`crate::text::status::field_delete_declined_structural`] in the status bar.
It is the same query the four controls asked — `EditSession::deletion_refusal`
through the one derivation — so a state where a control is drawn and this
refuses cannot arise from two rules disagreeing.

It does **not** clear the selection, and that is half the fix: the
Properties panel's own sentence is drawn from `doc.selected_field`, so
clearing here would delete the explanation on the same frame as the
refusal.

# Why `document_refuses_delete` rather than `refuses_delete`

Not a second derivation — the **same** engine query, asked at a different
scope. `refuses_delete` answers *would deleting **the selected field** be
refused?* and is `false` when nothing is selected, because its four readers
are all deciding whether to offer a control about a selection. This verb
takes a field **by name** and can be reached with no selection at all (a
chord that fired after the selection moved, an action queued a frame
earlier), where `refuses_delete` would answer `false` and wave through the
very press it exists to stop. One query, two scopes, and the scope is the
difference.

### `fn clear_selection_if_edited`

The defect this replaces, in one sentence: `doc.selected_field = None`
was the FIRST statement of both delete verbs, so a refused delete cleared
the selection anyway — and the Properties panel's
`panels::properties::formfield` section, which draws the sentence
*"This document does not allow form fields to be removed"*, is drawn from
exactly that field. The operator pressed Delete on a certified form, the box
stayed, and the explanation vanished. A refused gesture that destroys its
own explanation is strictly worse than one that merely fails, because after
it there is nothing on screen to read.

# Why the epoch, rather than a second copy of the gate

[`refused`] already turns back the refusal the query can
forecast. This covers **every other way the engine can decline** — a field
the document no longer has, a widget index that moved under an undo, a
borrowed session — none of which any query predicts and all of which reach
`crate::app::actions::apply::vector_edit`'s `Err` arm, where the selection would otherwise
have been cleared by a delete that did not happen.

`crate::app::actions::apply::vector_edit` bumps `OpenDoc::edit_epoch` on `Ok` and on `Ok`
only, so the epoch having moved is the one observable that means *the
document changed*. Reading it is not a second mechanism for the same
outcome; it is the only mechanism, because the funnel returns `()` and
deliberately does not report success — every caller that needed to know has
asked the epoch.

On success the clear is still wanted, and for the reason it was always
wanted: the field or the box is gone, so a selection naming it describes
nothing. `panels::properties::formfield`'s second early return handles the
dangling case (undo and redo do not clear selections), which is why this is
tidiness rather than a correctness requirement on the success path.

### `fn the_certified_nested_fixture_is_both_certified_and_nested`

# Why this test exists at all, and why here

`tools/ui-verify/src/checks/form_groups.rs`'s phase F asserts an
**absence**: that on a certified document the Forms panel's Field-groups
section lists its grouping nodes and draws **no** `forms.groups.arm.*`
control, because R9 says a permanently-refused capability renders
nothing rather than a greyed button.

An absence assertion is only evidence if the thing that would have
produced the presence was working. `panels::forms::groups::section`
returns early — before a single control is drawn — the moment
`AcroForm::groups` is empty. So a fixture that loads, refuses deletion,
and has an **empty** `groups` makes phase F **pass while testing
nothing**: the arm control is missing because the section never drew,
not because the refusal withheld it. That is strictly worse than the SKIP
it replaces, because a SKIP is legible in the report and a vacuous pass
is not — and it is exactly the failure that produced this fixture's task.

⇒ The four properties are pinned here, **inside the crate**, so that a
change to the engine's field-tree walk or to its certification census
breaks a fast unit test rather than quietly hollowing out a driven check
that no one may run for a week.

It lives in this module because this module is where
`deletion_refusal`'s shell-side consequence is asserted — the two tests
below use `certified-comments.pdf` for the flat half of the same
question, and a reader comparing them can see what the nesting adds.

# The four claims

1. **It loads.** `open_local_fixture` panics otherwise, so reaching the
   first assertion is the assertion.
2. **`deletion_refusal()` answers `Some`** — the file is certified and
   the certification is enforced. Without this the Field-groups section
   would draw its arm controls live and phase F would fail for the right
   reason but on the wrong subject.
3. **`AcroForm::groups` is non-empty**, and is the *two-level* shape:
   `Personal.Address` then `Personal`, post-order, deepest first — core
   records that order on the field itself, and it is the opposite of what
   "DFS order" suggests. Asserted as an exact list rather than as a count
   so that a walk which lost the interior node but kept the root, or
   which reversed the order, is caught.
4. **`fill_refusal()` answers `None`** — filling is still permitted. This
   is the claim `/P 2` is chosen for, and the reason `/P 1` was rejected:
   at `/P 1` both gates refuse, the fill controls disappear too, and a
   build that had disabled the whole Forms panel would be
   indistinguishable from a correct one. R162 — an assertion that cannot
   come out false is not an assertion.

Claim 4 is what makes claims 2 and 3 a *withholding* rather than an
outage. On this one document the two gates disagree, so the control
group is inside the file and no second fixture is needed to supply it.

### `fn a_refused_widget_delete_keeps_the_selection_and_words_itself`

The defect in one line: `doc.selected_field = None` was the first
statement of this verb, so on a certified form the press cleared the
selection whether or not the engine did anything — and
`panels::properties::formfield`'s section, which draws
*"This document does not allow form fields to be removed"*, is drawn
from exactly that field. The box stayed, the selection vanished, and the
sentence explaining why vanished with it.

Three assertions, and each pins a different half of the fix:

1. **the epoch did not move** — nothing was edited, which is what makes
   the other two about a *refusal* rather than about a success;
2. **the selection survived** — so the panel still has a field to
   describe and its sentence is still on screen;
3. **a decline was recorded** — R83's actual rule is not *gate the
   controls*, it is **a refusal must be a sentence, never a silence**,
   and `apply::vector_edit`'s `Err` arm says nothing by design.

### `fn a_refused_field_delete_keeps_the_selection_and_words_itself`

The reviewer named [`widget`]. An absence claim is a claim about
every route, so its sibling is asserted here rather than reasoned about
— it opened with the same `doc.selected_field = None` and reached the
same funnel.

### `fn an_uncertified_document_still_deletes_and_then_clears_the_selection`

`threaded-comments.pdf` differs from `certified-comments.pdf` in one
dictionary — the catalog's `/Perms` — so a difference here is caused by
that dictionary and by nothing else. A verb that refused unconditionally
would satisfy everything above and would be the worse defect: a delete
withheld where it would have worked leaves the operator no gesture that
reports it.

And the selection IS cleared on the success path, which is the other
half of [`clear_selection_if_edited`]: the box is gone, so a selection
naming it describes nothing.

### `fn field`

The disclosure names the **widget count**, because that is the part the
operator cannot see: a field drawn in three places disappears from three
pages, and they are looking at one of them. A confirmation that said only
"deleted" would be true and would leave two pages changed without mention.
**The selection is cleared ON SUCCESS, never ahead of the call** — see
[`clear_selection_if_edited`], which carries the whole argument.

### `fn widget`

The engine may report that the field went too, and the disclosure has to
follow it rather than assume: removing the last widget of a field leaves a
name nothing draws and nothing can fill, so `delete_widget` removes the
field as well. That is the right behaviour and it is **not** what the
operator pressed, so it is said out loud.
**The selection is cleared ON SUCCESS, never ahead of the call** — see
[`clear_selection_if_edited`], which carries the whole argument.
