# `pdfcer-gui/text/textedit`

## Item notes

### `fn the_multi_run_note_says_what_happens_rather_than_refusing`

Its ancestor asserted `s.contains("Click directly on the word")` — advice
that could not work, because the refusal was about the *line* and not
about where on it the operator clicked. The property that replaces it is
the one that matters now: the sentence must name **the consequence the
operator cannot otherwise see**, which is that the neighbouring pieces
will not move.

### `fn sharing_the_line_pins_the_neighbours`

If this reason ever reflowed, editing one cell of a SolidWorks parts
table would slide every cell after it sideways — content the operator did
not touch, moved by an edit that did not mention it. Asserted here rather
than only in `disposition`'s own tests because this module is where the
sentence promising it lives, and a sentence and a behaviour that disagree
is worse than either alone.

### `fn each_pinning_reason_explains_itself_differently`

A single generic sentence would be the cheaper implementation and would
be wrong for both cases: "right-aligned" is something the operator's
document is, and "rotated" is something they can see, and the remedy
differs.

### `fn every_reflow_cause_says_something_of_its_own`

The property the enum exists for. Before O127 the four shell-side causes
went to one channel and the four engine-side ones collapsed into nine
generic words — so the operator could press Reflow for four genuinely
different reasons and be told the same nothing. A duplicate here would
be that failure re-arriving with a type in front of it.

### `fn the_two_stale_plan_causes_name_the_remedy`

A refusal naming a cause with no route is half a sentence — the rule
`text::embed`'s blocker rows already follow — and *"save this file and
open it again"* is not guessable from *"cannot reflow"*.

⚠⚠ **Both of this test's subjects are UNREACHABLE at engine `025d703d`**
and it is kept anyway. Said plainly so it is not mistaken for coverage
of something an operator can meet: `PageSetChanged` has been unreachable
since the engine's `Pass 257.0`, and `PageAlreadyEdited` since `G015`.
The test guards the two SENTENCES, which are deliberately retained
against a future engine reinstating either guard by name — and a
retained sentence with no test is how a retained sentence rots. What it
is NOT is evidence that either refusal can be produced.

**The two have DIFFERENT instruments for that question, and saying
"the gate covers it" would be wrong about one of them.**

* `PageAlreadyEdited` is dead because of something in the ENGINE —
  nothing constructs `ReflowApplyError::PageEditedThisSession`. This
  side of the boundary cannot see that change happen, so it is watched
  by `tools/gates/check-unreachable-refusals`, which re-measures the
  engine source at the pinned revision on every commit.
* `PageSetChanged` is dead because of something in THIS crate — no arm
  of `app::actions::textstyle::reflow_refusal` produces it. That is a
  fact about a twelve-line function, and the instrument is the unit
  test beside it (`…::tests`, which asserts the `Unsupported` arm has
  not drifted back onto it). A gate is the wrong tool for a question
  `cargo test` already answers, and claiming one covers it would be an
  unevidenced excuse — which reads as an answered question, so nobody
  investigates.

### `fn the_enter_refusal_offers_a_keyboard_route_and_a_gesture_route`

The sentence has to carry the keyboard route as well as the gesture
route, because O127's brief is explicit that commit must not be reachable
only by mouse: an operator told *"use Add text"* and nothing else has
been given a way to place text and no way to finish the edit they are
already in.

### `fn the_point_text_disclosure_names_the_edge_it_wraps_at`

Rule 4's obligation, and the reason the sentence is longer than *"placed
as a block"*: the operator can see two lines of text and cannot see the
rectangle they were laid into, so the one fact they need is what decides
where a long line will break.

### `fn every_refusal_says_something`

The whole point of the module: the old shell's answer to the cross-run
case was no sentence at all.


It is tolerable here for the same one reason [`EditRefusal`]'s
`EVERY` gives: [`refusal`]'s own `match` is exhaustive, so a fifth
variant is a **compile error** in the catalog before it can be a silent
gap in this list. The list is a convenience over a closed set, not the
closure itself — which is why the compile error is the tripwire and this
paragraph is only the reminder to extend the array when it fires.
