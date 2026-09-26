# `pdfcer-gui/canvas/moving/tests`

## Item notes

### `fn a_move_never_alters_the_selection`

The counterpart of
[`navigating_the_view_never_alters_the_selection`](crate::canvas::selection),
and it is asserted the same way: drive the thing that *could* reach the
selection, then compare.

What a committed move does to the shell is exactly two things — it
bumps `OpenDoc::edit_epoch`, and it makes the next decomposition report
the same objects at the same indices in new places. Both are modelled
here: the epoch moves from 0 to 1, and the provider handed to the
re-resolve is `stub_moved`. `object_identity_across_edits.rs` is what
licenses the second half — `move_*` rewrites operands in place, adds and
removes no operator, and therefore renumbers nothing.

**The test is not vacuous**, and the second assertion is what makes it
so: the outlines must have *moved*. A `resolve` that quietly did nothing
would satisfy the identity assertion perfectly.

### `fn a_drag_between_two_page_points_moves_the_same_distance_at_every_zoom`

The hit-tolerance trap, in the move gesture's clothing — and stating it
correctly is half the value of the test, because the *tempting* wording
is the wrong one. "The same screen distance yields the same page delta"
is **false and must be false**: a fixed screen distance is
`distance / zoom` page units, which is
`viewer::screen_to_page_distance_scales_as_one_over_zoom`. Asserting it
would be asserting the defect.

What must be invariant is the operator's experience: grab a point on the
page, drop it on another point on the page, and the object moves by the
distance *between those two page points* — the same answer at 25 % and
at 1200 %. So the fixture drags between two fixed **page** positions,
projects them to screen through the frame's mapping (which is what the
pointer really reports), converts back exactly as `canvas/mod.rs` does,
and asserts one answer at four magnifications.

A second division by zoom anywhere in that chain — or a missing one —
makes this fan out by a factor of the zoom, which is precisely the
failure [`crate::canvas::mapping`] was built to make unavailable and the
reason [`PageMapping`](crate::canvas::mapping::PageMapping) has no
`zoom()` accessor to divide by.

### `fn a_non_path_in_the_selection_routes_through_a_transform`

It read *"a non-path member refuses the WHOLE move, and names the
offender"*, and the reasoning was sound while it lasted:

> *"The engine does this too, and would do it correctly. Refusing here
> as well is what keeps the ghost honest: an outline that slides across
> the page and then snaps back has already told the operator something
> untrue."*

The ghost obligation stands and is now satisfied the other way round —
the outline slides **and the release commits**, because `Pass 113.0` gave
this shell a verb that moves anything. The operator asked for it three
times: *"can I please please please have the capability to move the text
after?"*

What is asserted is the **rung**, not the absence of a refusal: a
build that routed every move through the transform would also stop
refusing here, and it would be wrong for the reason `eligible`'s own
comment gives about the file rather than the API.

### `fn an_all_path_selection_still_reaches_move_objects`

The other half of the fork, and the half a tidy-up would delete. A
transform wraps each object in `q <cm> … Q` per gesture; `move_objects`
rewrites the coordinates in place and adds nothing. On a drawing that is
nudged dozens of times, the wrapping accumulates in a file somebody then
sends on.

### `fn a_line_of_text_at_the_part_rung_moves_that_line`

**The half of the old test that was load-bearing is kept**, and it is
the assertion in its failure message rather than in its `assert_eq!`:
*moving the enclosing object because a run was selected is the wrong
action, not a lenient one*. A shell that answered
[`MoveSubject::Objects`] here would drag the entire title block when the
operator had one label selected, which is a worse outcome than the silence
O188 complained about — so this asserts the exact subject, not merely that
something was returned.

### `fn several_selected_chunks_move_as_one_command`

The Node rung's defect, one rung up and found by looking rather than by a
report: the model has held several `subpath` entries on one object since
normalisation was written, the overlay outlines every one of them, and
`eligible` asked `entered_object()` — the FIRST entry. Four chunks
highlighted, one moved.

The plural arm is reached on COUNT, never on a flag, so a set that
shrinks back to one chunk takes the singular verb again with no second
decision anywhere. The next test asserts that half.

### `fn one_selected_chunk_still_takes_the_singular_verb`

The count is the only condition, so this needs no separate mechanism — but
it needs a test, because the plural arm shadowing the singular one is a
silent change of verb and `move_text_run` is the planner the engine's own
singular path is tested against.

### `fn a_run_the_engine_would_refuse_declines_before_the_ghost_is_drawn`

Both blocks are asserted, and separately, because they are two different
facts about the document and the operator is told which one he has. See
[`crate::text::arrange::run_has_no_position_of_its_own`] for why that
distinction survives all the way out to the sentence.

**What this is really guarding is the ORDER of two questions.** The
engine would refuse these moves too — `plan_move_text_run` runs the same
guard — but it would refuse them *after* the gesture, so the operator would
watch an outline slide across the sheet and snap back. Obligation 3 in this
module's header says a ghost is only ever drawn for a move that will
commit; this is the test of it for the newest subject.

### `fn a_refused_drag_on_one_line_of_text_asks_for_a_sentence`

The test above asserts that the drag is *refused*, which was never the
complaint. This asserts the half that was missing: that the refusal reaches
the operator. Ken drew a box round one label in a title block, dragged it
across the sheet, and got nothing happening with no sentence anywhere —
which from where he sits is dragging being broken.

**Why the loop, when one arm would compile.** Because the two blocks
share a remedy and differ only in their first clause, and the cheap version
of this test — assert that *a* sentence was asked for — is satisfied by a
build that raises the same sentence for both. An assertion both outcomes
satisfy is not a measurement of which one shipped. So each block is asked
for by name, and `decline::tests::no_two_declines_share_a_sentence` is what
then proves the two names are not two spellings of one string.

Asserts the ACTION, not the status bar. The store is written by the
apply phase (`app::actions::apply`) and the wordings live in
`text::arrange`; what this module is responsible for is asking. The driven
`ui-verify` check is what asserts the sentence actually lands on the bar,
per R1 — a unit test cannot see the chain in front of the verb.

### `fn a_run_index_that_is_not_there_refuses_without_a_sentence`

That is not a refusal the operator caused and there is nothing he could do
about it; a sentence would report an internal inconsistency in the
vocabulary of his drawing. R9's shape: the honest answer to *this should not
have happened* is nothing on screen, and the trace line
`canvas-move-declined — reason=run-not-there` for whoever is reading the
log. It is reachable only through a stale selection, which
[`super::Refusal::worded`] documents.

### `fn the_refusals_the_operator_can_see_raise_nothing`

One rule, asserted once, rather than one assert per case. A status bar that
narrates the obvious — *nothing is selected*, *the drag did not travel* —
stops being read, and that would cost the two sentences that matter.

`NoVerbForPart(Subpath)` is in this list. It is unreachable today
(`eligible` routes a subpath at the Part rung to `move_subpath`), and it is
asserted anyway, because an unreachable case that is written down is a claim
the next reader can check.

### `fn all_refusals`

# A hand-written list inside a completeness test is the classic hole,
# so this one is behind a compile-time guard

A test that types out its own input set is blind to the twelfth thing, and
the count still adds up. The `match` at the end of this function is
exhaustive and has **no wildcard**, so adding a variant to [`Refusal`] is a
compile error *here*, in the function whose whole job is to list them.

⇒ **And the limit of that, stated rather than implied:** somebody could
satisfy the compiler by adding an arm and not the vector entry two lines
above it. The guard puts the omission in the right file, in the right
function, next to its fix. It does not make the omission impossible.

### `fn silent_refusals`

Derived from [`all_refusals`] through `worded` itself, which is why the
silence test cannot drift out of step with the decision it is asserting: if
a tenth refusal is given a sentence, it leaves this set automatically, and
if a twelfth is added silently it joins this set and is asserted.

### `fn several_selected_anchors_move_as_one_command`

This is the regression test for a defect that lived in the gap between
two correct halves. `SelectionState::pick_within` has added a
Shift-clicked anchor as its own entry since the Node rung landed, and
`subject` read `entered_object()` — the FIRST entry. So the model held
four, the overlay drew four, and the drag moved one.

Nothing failed. Both halves' unit tests passed. The only thing that
would have caught it is driving it, or a test like this one that asks
the two halves the same question.

### `fn one_missing_anchor_refuses_the_whole_move`

The same call `move_objects` makes over a non-path member, and for the
same reason its docs give: a partial application reads as a rendering
fault rather than as a refusal, and the operator has no way to learn
which of their anchors was dropped.

### `fn one_selected_anchor_still_takes_the_singular_verb`

`EditSession` has both, and `docs/core-api/02`'s rule cuts both ways:
the plural verb is correct for a set and the singular one for a member.
Routing one anchor through a slice would lose the singular planner for
no gain.
