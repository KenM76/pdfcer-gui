# `canvas::keys` tests — the Delete ladder enumerated, and the two keys
# that only pass through

## The seam, and why the half left behind is the interesting one

[`super`] is **two precedence ladders** — which claimant a Delete reaches,
and which a press of Escape does — and each is a short function whose whole
content is an ordered list of `if let` arms. The tests are bulky because a
ladder is exercised by enumerating it, not by three cases: every rung needs
a case that reaches it AND a case that proves the rung above did not swallow
it.

⇒ So this is a split between **the ladder** and **the enumeration of the
ladder**, a subject boundary rather than a size-driven cut — the same one
`gesture::meaning` took three commits earlier. The Escape ladder's
enumeration is [`escape_ladder`], one level down, for the same reason
applied once more: two ladders is two subjects, and the fixtures above are
all either one needs.

## The two keys here that are not a ladder at all

Delete and Escape choose between claimants. An arrow key and Tab do not —
they are routed straight through to the module that decides them, and the
only thing this file asserts about either is **that the route exists**:

* `an_arrow_key_reaches_the_nudge_through_canvas_keys` — the nudge's own
  enumeration lives in [`crate::canvas::moving::nudge`]'s tests, beside the
  module that decides it;
* `a_claimed_tab_reaches_the_object_ring_through_canvas_keys` — the ring's
  lives beside `canvas::objring`, where a narrowed [`PickFilter`] is the
  point rather than an incidental.

Both are single route assertions, and both are why the three paragraphs
that used to stand here — *no case presses Tab*, *no assertion presses an
arrow*, *every case passes `page: None`* — are gone rather than edited. Each
was true of the Delete and Escape ladders and was written as though it were
true of the file.

## Every **ladder** case passes `targets: None` **and**
## `model_attempted: true`, and the pairing is deliberate

Those two fields answer different questions and a unit test is the one place
it is easy to set them inconsistently:

* `targets: None` — *there is no decomposition*, which is what lets these
  run without opening a file;
* `model_attempted: true` — *the frame asked for one and did not get it*,
  which is the **page-would-not-decompose** case.

The same cases pass [`PickFilter::all()`](PickFilter::all) and
`page: None`, and neither is a stub: `all()` is what a shell that has never
touched the filter hands over, and a Delete never crosses into PDF space, so
a frame with no page on screen genuinely has none to hand. The two route
assertions above are the exceptions, and each names its own reason.

## Item notes

### `fn an_arrow_key_reaches_the_nudge_through_canvas_keys`

The nudge's own rules are enumerated in
[`crate::canvas::moving::nudge`]'s tests, which call that module directly.
This asserts the one thing those cannot: that the wiring exists — that a
press of Up on a canvas with a markup selected comes out of `canvas_keys`
as an `AnnotAction::Move`.

It is a separate test on purpose, and the reason is this project's most
expensive recurring defect: **a working verb reachable by nothing.** It has
shipped four times (`Resize`, `Handle`, `DimensionVertex`, the Delete key),
every time with the module's own tests green, because a module tested in
isolation cannot tell whether anybody calls it. Deleting the
`moving::nudge::keys` call above leaves fifteen nudge tests passing and this
one red.

It also pins the two things this function contributes and the nudge module
does not: that the Tab branch above does not swallow the frame, and that the
`page` field really reaches the coordinate crossing — a nudge wired with
`page: None` would decline, so the positive `dy` here is proof the page
arrived.

### `fn delete_removes_a_selected_form_field_and_nothing_else`

`OPERATOR_REQUESTS.md` **O53**. Delete did not reach a form field at all:
this ladder never had `doc.selected_field` in front of it, because a
widget is deliberately not an annotation selection and the form surface
owns those presses.

The assertion is a **comparison**, not a presence check. A build that
raised the field deletion *and* fell through to the content one would
satisfy "did it raise the field action?" and would delete the operator's
page content as well — silently, because the field deletion they asked
for did happen. Exactly one action, and it is the right one.

### `fn delete_does_not_act_on_a_form_field_whose_deletion_would_be_refused`

# What this pins, and why the shape of the failure is the point


The consequence was not a harmless no-op. `actions::forms::delete_widget`
cleared `doc.selected_field` **before** calling the engine, so on an
ordinary certified fillable form the press produced:

1. an action raised for a verb that would refuse,
2. a refusal into `actions::apply::vector_edit`'s `Err` arm — a trace line
   and, by that arm's own recorded decision, nothing to the operator,
3. **and the selection cleared anyway**, which blanked the Properties
   panel's `formfield` section — the one surface that was correctly saying
   *"This document does not allow form fields to be removed."*

⇒ A silence that also destroys the sentence explaining it. `actions` being
empty pins step 1, which is the only one of the three this ladder can
prevent — and preventing it prevents all three.

A **content** selection is live at the same time, deliberately, and that
is what makes the second assertion evidence rather than decoration: a build
that declined the field rung by *falling through* instead of returning would
delete the page objects underneath the widget. Refusing one verb is never a
licence to run a different one.

### `fn delete_acts_on_a_form_field_when_the_gate_is_open`

A rung that declined unconditionally would satisfy every assertion above
perfectly, and would be a strictly worse defect than the one being fixed: a
control withheld where it would have worked leaves the operator no gesture
that reports it. This is the other half every rung in this file is required
to carry.

It is `delete_removes_a_selected_form_field_and_nothing_else` above with
one field flipped, and it is written separately rather than folded into it
because that test is about **precedence** and this one is about the
**gate**. Two questions, two failures worth telling apart.

### `fn delete_with_an_object_selected_raises_the_delete_action`

D1's own words: *"I can't even click on an object and delete it by
hitting the delete key."* `app::keyboard` proves the key survives a
canvas click; this proves the key now reaches a verb.

### `fn delete_inside_an_object_never_borrows_the_object_rungs_verb`

The destructive wrong action this ladder must not ship, and the assertion
has outlived the reason originally given for it — which is why the reason is
rewritten here rather than left standing.

**It used to be:** *"the Part rung has no delete verb wired"*. True until
2026-09-05, when `delete_subpath`, `delete_text_run` and `delete_node` were
wired through [`crate::canvas::deleting`]. The rung has a verb now.

**What is asserted is unchanged and is the part that mattered all along:**
the Part rung must not raise `DeleteSelection`. One measured CAD export
holds an entire drawing view as a single path object with 1,194 subpaths and
one text object with all 237 pdf-dimension labels in it, so borrowing the
Object rung's verb removes a drawing in answer to *"remove this line"*.
*"They can undo it"* is not an answer to that.

Here it raises nothing at all, and the reason is stated so the assertion
is not read as stronger than it is: these tests pass `targets: None`, so
`deleting::subject` declines `NoObjectModel` — a deeper rung with no
decomposition cannot know whether it is looking at a subpath or a label, and
guessing is the one thing it must not do. Which verb it reaches **with** a
model is asserted exhaustively in `canvas::deleting::tests`, over real
documents, because a text run needs a resolved font to exist at all.

### `fn a_focused_text_field_keeps_delete_for_itself`

`app::keyboard`'s regression test proves the *other* direction: a
focused NON-text widget must not suppress the key. Both are needed.
This one builds a real `TextEdit` and focuses it, because
`text_edit_focused()` resolves the focused id and looks for a
`TextEditState` under it — a hand-requested focus on a bare id would
pass vacuously.

### `fn delete_does_not_act_on_an_annotation_whose_deletion_would_be_refused`

# What this pins, and why the shape of the failure matters more than the rung


The consequence was not a harmless no-op. `actions::annots::delete` clears
the annotation selection **after** the funnel rather than on success, so on a
certified drawing the press produced:

1. an action raised for a verb that would refuse,
2. a refusal into `actions::apply::vector_edit`'s `Err` arm — a trace line
   and, by that arm's own recorded decision, nothing to the operator,
3. **and the selection cleared anyway**, taking the Properties panel's
   explanation off the screen with it.

⇒ A silence that also destroys the sentence explaining it. Asserting
`actions.is_empty()` pins step 1, which is the only one of the three this
ladder can prevent — and preventing it prevents all three.

Asserted with the annotation **unlocked**, deliberately. A locked
annotation would be refused by the older half of the gate, so the test would
pass on the code this fixes and prove nothing.

### `fn delete_acts_on_an_annotation_when_the_gate_is_open`

A rung that declined unconditionally would satisfy the assertion above
perfectly. This is the other half every rung in this file is required to
carry: a case that reaches it, beside the case that proves it was not
swallowed.

### `fn a_delete_declined_for_want_of_asking_is_not_allowed_to_be_quiet`

# Why this test exists rather than only the assert


⚠ It is the ONE case in this file that passes `model_attempted: false`, and
it is why every other case passes `true`: `false` means *nobody asked*, and
nobody asking at a deeper rung is not a state the program is allowed to be
in. See this module's header.

`should_panic` matches on a fragment of the assert's own message rather
than on the bare fact of a panic, because a panic from somewhere else in
`canvas_keys` would satisfy an unqualified `should_panic` and report this
tripwire as working when it had not run at all.
 And why it is compiled out of a RELEASE test run.

The tripwire is a `debug_assert`, so `cargo test --release` compiles the
panic away and this test then fails for a reason that says nothing about
the program: *"test did not panic as expected"*. Found 2026-09-10 -- the
release suite reported `3313 passed; 1 failed` while the debug suite
reported `3314 passed; 0 failed`, and the difference was entirely this.

A permanently-red test in one profile is worse than no test, because the
only way to keep using that profile is to learn to ignore the red -- and a
suite you ignore cannot tell you about the fifth recurrence this tripwire
exists to catch. Gating it on `debug_assertions` is the honest shape: the
assertion under test does not exist in release, so neither does the test.

### `fn a_claimed_tab_reaches_the_object_ring_through_canvas_keys`

The ring's own rules are enumerated in [`crate::canvas::objring`]'s tests,
which call `stops` directly. This asserts the one thing those cannot: that
the wiring exists.

It is written as *the press was spent*, not *the selection moved*,
because a unit test has no decomposition to move a selection within —
`targets: None` is what every case in this file passes and what lets them
run without opening a file. Spending the press is still the whole of what
the wiring does: `canvas::tabnav::claim` has already removed the event from
`RawInput`, so nothing else in the program can see it, and a press left in
the store would be a Tab that did nothing this frame and something
arbitrary several frames later.

This project has shipped a working verb reachable by nothing **four**
times, every time with the verb's own tests green. Deleting the
`objring::advance` call from `canvas_keys` leaves all five ring tests
passing and this one red.
