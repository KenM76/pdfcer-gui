# `ui-verify/checks/field_delete_gate`

`field_delete_gate` — **a certified document does not offer a Delete for its
form fields, says so, and does not lose the sentence when Delete is
pressed.**

The driven assertion for `EditSession::deletion_refusal`, which
`crate::…::formfield::refuses_delete` derives once and four surfaces read.
[`super::annot_delete_gate`] is its twin one `/Subtype` along; read that one
first, because this file is the same defect wearing a `/Widget` and the
shapes are deliberately identical.

# What was wrong, and why "the annotation one is fixed" was not enough

On 2026-08-29 the R83 pass closed the annotation door: `format.delete`
acquired `visible_when: "selection.delete_permitted"` on the Format tab and
on the `canvas.object` menu, `canvas::keys`' annotation rung acquired a
gate, and `panels::properties::annotdelete` drew the sentence. Its commit
claimed *"where a gate refuses the controls are not drawn at all."*

That held for **one surface of three**, and for a form field the gate was a
no-op by construction:

1. `app::conditions` published `selection.delete_permitted` from
   `doc.selected_field.is_none() && annotdelete::refuses_selected(doc)`.
   With a field selected the first conjunct is **false**, so the condition
   was set unconditionally for every selected field on every document.
2. The `canvas.field` context menu's `format.delete` carried **no
   `visible_when` at all**, while `canvas.object`'s carried one.
3. `canvas::keys`' rung 0 pushed `DeleteWidget` on `caps.edit_content &&
   selected_field` with **no gate**, and returned six lines above the
   annotation branch that does ask one.
4. And `actions::forms::delete_widget` cleared `doc.selected_field`
   **before** the engine call and said nothing when the engine refused.

⇒ On an ordinary certified fillable form: right-click a widget, Delete is
drawn live and undimmed, press it, the box stays, the selection vanishes,
nothing is said — **and the Properties panel that was correctly showing
"This document does not allow form fields to be removed" goes blank.** A
refused gesture that destroys its own explanation, which is the exact defect
shape the R83 work existed to remove.

# Why only driving can prove it fixed

Every unit test in the crate can assert the *rules*, and this fix ships
seven of them. None can assert the **sequence**: a manifest `visible_when`
resolved by `egui-shell` against a condition set rebuilt per frame, a canvas
hit test that turns a click into a `SelectedField`, a real keystroke through
`canvas::keys`' four-rung ladder, a panel section drawn into a dock slot,
and — the assertion no unit test can reach — **that the panel's sentence is
still on screen after the press.** R1: a capability is not verified until
the running binary has been driven through it.

# The fixture pair, and why the check drives BOTH

`fixtures/certified-comments.pdf` and `fixtures/threaded-comments.pdf` are
**one document differing in one dictionary** — the catalog's `/Perms` —
built by one function in `tools/gen-certified-fixture.py`. Both carry the
same merged signature field, `/T (Certifier)`, whose sole widget is at
`[60 60 300 120]` on page 1.

That widget is the one [`super::annot_delete_gate`] deliberately steers
its click *away* from, and its reason is this check's operand: a click that
lands there *"would select a form field, take the form surface's branch, and
report the annotation gate as broken."* The two checks aim at the two
objects in the same file and each asserts the branch the other avoids.
**No new fixture was authored**, which matters: a second certified document
could differ from this one in ways nobody intended, and the whole evidential
value of a pair is that the difference between the runs is the `/Perms`
entry and nothing else.

Driving only the certified file would satisfy a build whose gate refused
**unconditionally** — a worse defect than the one being fixed, because a
control withheld where it would have worked leaves the operator no gesture
that reports it. So phase E re-launches on the ordinary twin and asserts the
control is **there**.

# The absence assertions, and what makes them admissible

Three of this check's assertions are that something is **not** there:
`properties.form_field.delete` on the certified run, the funnel's own
`delete-widget` line after the keystroke, and
`properties.form_field.delete_refused` on the ordinary run.
`crate::checks`' rule 4 forbids treating an absence as evidence unless the
thing that would have produced it has been shown to be working.

All three are admissible for one reason: `panels::properties::formfield`
writes its `form-field-gates` census line on **every frame the section
runs**, refused or not. The check reads that line first — which proves the
section drew and both gates were asked — and only then reads the regions.
Without it, "no delete button" and "the Properties panel never opened" would
be the same trace.

# Phases

| Phase | Does | Expected |
|---|---|---|
| A | launch on the **certified** fixture in Edit with the Properties panel shown | the `page` region declared |
| B | click the centre of the signature widget's `/Rect` | `form-field-selected … field=Certifier` |
| C | read the census and the regions | `form-field-gates … delete_refused=1`, `properties.form_field.delete_refused` declared, `properties.form_field.delete` **not** |
| D | press Delete | `canvas-delete-declined … reason=field-delete-refused`, **no** `delete-widget` funnel line, and `properties.form_field.delete_refused` **still** declared |
| E | relaunch on the **ordinary** twin, click the same point | `form-field-gates … delete_refused=0`, `properties.form_field.delete` declared, `…delete_refused` **not** |

# Why Edit mode rather than Review

[`super::annot_delete_gate`] drives Review, and for a reason that inverts
here. `canvas::forms` gives the **selection** surface to `edit_content` and
the **fill** surface to Read and Review — *"the same click cannot both type a
value and select the box to rename it"* — and `canvas::keys`' rung 0 is
gated on the same capability. In Review the click would open a fill editor
and Delete would never reach rung 0, so the run would say nothing about the
gate while passing every assertion that does not name it.

⇒ Phase D's assertion is therefore about **rung 0 specifically**, not about
the ladder's shape: in Edit the annotation rung below is reachable too, so
the check reads the decline's `reason=` key rather than settling for *"a
decline happened"*.

## Item notes

### `const INVOKE`

`file.properties` is the command that mounts and activates the panel from
any arrangement, so the check does not have to know what dock layout the
machine it runs on happens to have persisted. `mode.edit` is load-bearing
rather than cosmetic — see the module header's last section.

### `const CERTIFIED`

**Relative to `CARGO_MANIFEST_DIR`, which is `tools/ui-verify`** — two
levels up, not three. See [`super::annot_delete_gate`]'s note on the same
constant: this file inherited the wrong depth from it, and both resolved to
a `D:/Dev/fixtures/` that does not exist, so both SKIPPED on every run while
telling the reader to run a generator that writes somewhere else.

### `const GATES_EVENT`

The `-gates` suffix is not decoration: `tools/gates/check-trace-names.py`
forbids a module's own summary line from sharing its first token with a
`vector_edit` funnel label. A harness reading a bare name would get the
funnel's line — `page`, `n`, `epoch`, `disclosures`, and none of the keys
read below. That confusion has produced a confident false negative on this
project three times.

### `const DECLINED_EVENT`

Shared with the annotation rung, which is why the `reason=` key is read
rather than the event alone: in Edit mode both rungs are reachable, and a
decline from the wrong one would say nothing about the gate under test.

### `const FUNNEL_EVENT`

Asserted **absent** in phase D. Its presence means the ladder let the action
through and the engine refused it — which is the pre-fix behaviour exactly,
and which no region assertion would catch, because the panel would still
have drawn its sentence on the frames before the press.

### `const WIDGET_CENTRE`

Derived from `objs[11]` in `tools/gen-certified-fixture.py`
(`/Rect [60 60 300 120]`), and stated as a point rather than as a page
fraction for the reason [`super::annot_delete_gate`] gives about its own
operand: the target is **in the fixture**, so the aim has to be where the
fixture put it. Phase B asserts the click really selected `Certifier` by
name, so a click that missed reports as a miss rather than as a broken gate.

### `const FIELD_NAME`

The whole evidential value of the pair is that the two documents are
identical apart from one dictionary. A check that went looking for
*"a field"* could find a different one in each run and would report the
difference as a gate difference.

### `struct Run`

Factored because phases A–C and phase E are the **same** sequence against
two files, and the whole value of the pair is that they were driven
identically. Two hand-written copies would eventually differ in a settle or
in an aim, and the difference would be reported as a difference between the
documents.

### `fn page_geometry`

Stated rather than read from the file: the check is bound to fixtures the
repository generates itself, so a page size read back from them could only
confirm what the generator wrote — and a `--page-size` override would let a
caller aim this check at a document it is not about.
