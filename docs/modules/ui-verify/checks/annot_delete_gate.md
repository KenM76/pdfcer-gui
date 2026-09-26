# `ui-verify/checks/annot_delete_gate`

`annot_delete_gate` — **a certified document does not offer a Delete for its
comments, and says so.**

The driven assertion for `EditSession::annotation_deletion_refusal`, which
`crate::panels::properties::annotdelete` consumes.

# The failure this refuses, and why only driving can prove it gone

On a certified or encrypted drawing there are three routes to a Delete —
the Format tab's control, the canvas object menu's, and the Delete key. A
build that leaves all three live sends every press into
`delete_annotation`, which refuses; the refusal lands in
`app::actions::apply::vector_edit`'s `Err` arm, which writes one line to the
trace and *says nothing to the operator*. Worse,
`actions::annots::delete` clears the selection **anyway**, because it clears
after the funnel rather than on success — so the press also removes the
panel sentence that would have explained the refusal, had there been one.

Three visible controls, silently inert, and a gesture that destroys its own
explanation. That is the shape this project is named after.

Every unit test in the crate can assert the *rules*. None of them can
assert the **sequence**: a manifest `visible_when` resolved by `egui-shell`
against a condition set rebuilt per frame, a canvas hit test against a real
page, a real keystroke through `canvas::keys`' ladder, and a panel section
drawn into a dock slot. R1: a capability is not verified until the running
binary has been driven through it.

# The fixture pair, and why the check drives BOTH

`fixtures/certified-comments.pdf` and `fixtures/threaded-comments.pdf` are
**one document differing in one dictionary** — the catalog's `/Perms`. Same
pages, same annotations, same object numbers, same geometry.
`tools/gen-certified-fixture.py` builds both and its header carries why.

Driving only the certified one would satisfy a build whose gate refused
*unconditionally*, which is a worse defect than the one being fixed: a
control withheld where it would have worked leaves the operator no gesture
that reports it. So phase E re-launches on the ordinary twin and asserts the
control is **there**. Because the two files differ in exactly one
dictionary, any difference the harness sees between the two runs is caused
by that dictionary and by nothing else.

# The absence assertions, and what makes them admissible

Two of this check's five assertions are that something is **not** there —
`properties.annot_delete.collateral` on the certified run, and the funnel's
own `delete-annotation` line after the keystroke. `crate::checks`' rule 4
forbids treating an absence as evidence unless the thing that would have
produced it has been shown to be working.

Both are admissible here, and for the same reason:
`panels::properties::annotdelete` writes its `annot-delete-gates` census
line on **every frame the section runs**, refused or not, collateral or not.
So the check first reads that line — which proves the section drew and the
gate was asked — and only then reads the regions. Without it, "no collateral
region" and "the Properties panel never opened" would be the same trace.

# Phases

| Phase | Does | Expected |
|---|---|---|
| A | launch on the **certified** fixture in Review with the Properties panel shown | the `page` region declared |
| B | click the centre of the square's `/Rect` | `annot-select` naming it |
| C | read the census and the regions | `annot-delete-gates … refused=1`, `properties.annot_delete.refused` declared, `…collateral` **not** |
| D | press Delete **until the trace shows it was heard** | `canvas-delete-declined … reason=annot-delete-refused`, and **neither** `delete-annotation` nor `delete-annotation-refused` |
| E | relaunch on the **ordinary** twin, click the same point | `annot-delete-gates … refused=0`, and `properties.annot_delete.refused` **not** declared |

# Why Review mode rather than Edit

`canvas::keys`' annotation rung is gated on `caps.author_markup`, which is
true in Review and in Edit both — but in Review `caps.edit_content` is
**false**, so a build whose Delete fell through the annotation rung to the
*content* rung would raise nothing there and the check would pass on a
broken build. Review is the mode in which the annotation rung is the only
rung that can act, which makes phase D's assertion about that rung and not
about the ladder's shape.
