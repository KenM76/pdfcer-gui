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

## Item notes

### `const INVOKE`

`file.properties` is the command that mounts and activates the panel from
any arrangement — `app::PdfcerApp::show_panel` mounts it first if the
operator's saved layout no longer holds it — so the check does not have to
know what dock layout the machine it runs on happens to have persisted.

### `const CERTIFIED`

**Relative to `CARGO_MANIFEST_DIR`, which is `tools/ui-verify`** — so
two levels up, not three. Written as `../../../fixtures/…` when this check
was authored, which resolves to `D:/Dev/fixtures/…`: a directory that does
not exist and never will. The `pdf.exists()` guard below turned that into a
SKIP whose message told the reader to run the generator, and the generator
writes to `fixtures/` in *this* repository — so the check could not pass on
any build, and its reason for not passing pointed at a fix that could not
work. `reflow`, `text_edit` and `signature_save` all use `../../fixtures/`;
that is the convention, and this is the only file that departed from it.

### `const GATES_EVENT`

The verb suffix is not decoration: `tools/gates/check-trace-names.py`
forbids a module's own summary line from sharing its first token with a
`vector_edit` funnel label, and `delete-annotation` is such a label. A
harness asking `last("delete-annotation")` would get the funnel's line
instead — `page`, `n`, `epoch`, `disclosures`, and none of the keys read
below. That confusion has produced a confident false negative on this
project three times.

### `const FUNNEL_EVENT`

Asserted **absent** in phase D. Its presence would mean the gate let the
action through and the engine refused it — which is the pre-fix behaviour
exactly, and which no region assertion above would catch, because the panel
would still have drawn its sentence on the frames before the press.

### `const FUNNEL_REFUSED_EVENT`

`app::actions::apply::vector_edit` writes `<label>` on success and
`<label>-refused` on an `Err`, and it is the second that a Delete which
walked past the gate produces on a certified document: the ladder raised
`AnnotAction::Delete`, `EditSession::delete_annotation` refused it, and this
line went to the trace **and nothing went to the operator**.

Reading it is what turns phase D from an accusation into a diagnosis. A
phase that does not read it reports *"the keystroke did not reach
`canvas::keys` at all — check that the canvas had focus"* over a trace
carrying this very line four rows above the region it goes on to read: the
key arrived, was processed, and was silently refused. The failure hiding
behind that wording has nothing to do with focus —
`canvas::keys::Keys::annot_delete_refused` stuck at `false`. That is why
`canvas::interact` passes the selection in explicitly
(`annotdelete::refuses(doc, &selection)`) rather than letting the helper
re-read `doc.selection`, which by that point in the frame has already been
moved off the document.

**A check that reads only the line it hopes for can only say *"nothing
happened"*.** Naming the line that means *"the wrong thing happened"* is
what lets it say which.

### `const SQUARE_CENTRE`

Derived from `SQUARE_RECT` in `tools/gen-certified-fixture.py`
(`[120 560 320 700]`), and stated as a point rather than as a page fraction
— unlike most checks in this suite, which place their own operand and can
therefore choose a fraction. Here the operand is **in the fixture**, so the
aim has to be where the fixture put it. **An assertion that checks a
relation rather than a magnitude is satisfied by any absurdity in the right
direction**, and that applies here from the other side: phase B asserts that
the click actually selected the square *by object id*, so a click that
missed reports as a miss rather than as a broken gate.

### `struct Run`

Factored because phases A–C and phase E are the **same** sequence against
two files, and the whole value of the pair is that they were driven
identically. Two hand-written copies would eventually differ in a settle or
in an aim, and the difference would be reported as a difference between the
documents.

### `fn page_geometry`

Stated rather than read from the file: the check is bound to fixtures it
generates itself, so a page size read back from them could only ever confirm
what the generator wrote — and a `--page-size` override would let a caller
aim this check at a document it is not about.
