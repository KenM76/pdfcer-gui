# `ui-verify/checks/picking`

`checks::picking` — **one click is not always one pick**, in one place.


`measure_calibrates_by_picking_two_points` and
`set_scale_reads_the_group_it_is_about_to_overwrite` both drive the two-point
scale calibration. Both had **never once produced a verdict** — the first
SKIPPED in all five recorded sweep baselines, the second on its only one,
for a reason that had nothing to do with picking (see
[`crate::coords::CanvasMapping::span_from`]). The moment that was fixed and
they ran for the first time, both FAILED with the same sentence:

```text
two points were clicked on the page and no `scale-calibrate measured_pt=`
line was traced, so the pick never completed.
```

The accusation is false, and the application had already said so — forty
lines above, in its own trace:

```text
measure-snap-marker kind=DerivedCenterline marker=677.0,554.4 …
measure-pick outcome=Promoted reason=derived-candidate-needs-confirm
measure-pick kind=Scale in_progress=true committed=false
```

Two clicks were sent. The first found a **derived** snap candidate — a
centreline pdfcer *inferred* rather than one the file states — and pdfcer
refused to act on an inference without confirmation, which is
`pdfce_FeatureRequests/README.md` **rule 4** working exactly as specified.
The second click confirmed it, resolving pick **A**. Pick B was therefore
never clicked at all, and the check blamed the routing.

# The lesson, and it is not "snapping is awkward"

**A sibling check had already found this, documented it at length, and
solved it — and the two checks that needed the solution could not see it.**
[`crate::checks::measure_linear`]'s header carries a full section on rule 4
and a table of the three alternatives it rejected; the constant beside its
event name is annotated, verbatim, `← click again`. That was written before
either calibration check ran, and neither inherited a line of it.

This is the same shape as the correction recorded in
[`crate::checks::text_edit_real`]: *a rule written down beside one instance
of itself does not generalise on its own — the next instance arrives wearing
a different event name and reads as a new problem.* Here it did not even
change its event name. It changed its `kind=` field, from `Linear` to
`Scale`, and that was enough.

⚠ So the rule is not in a comment this time. It is in [`resolve_pick`], and
a check that clicks the canvas with a measure tool armed and does **not**
call it is a check that will one day report a promotion as a routing
failure. That is what this module is for.

# Why not in [`crate::checks::driving`]


# What this module deliberately does NOT do

It does not assert the *shape* of a pick sequence. `measure_linear` checks
that three picks read `committed=false, false, true`, that the resolving
tool was `Linear` and not something else, and that the promotion count is
reported; those are properties of *the linear-dimension feature*, not of
clicking, and they stay at that call site. This module owns exactly one
thing: **turn a point into a resolved `measure-pick` line, taking the
confirming click when the application asks for one, and refuse to keep
clicking forever.**

`measure_linear` keeps its own copy of the loop for now, and that is stated
rather than implied. Folding it on is a safe change but it is not *this*
change: rewriting the one check already known to detect its defect, in the
same commit that first makes two others run, would destroy the independent
evidence at the moment it is most needed. [`crate::checks::driving`]'s
header records the identical judgement about `markup_rectangle`. ⚠ The
tripwire against that copy drifting is [`tests::the_two_implementations_agree_on_the_click_bound`].

## Item notes

### `fn the_two_implementations_agree_on_the_click_bound`

[`crate::checks::measure_linear`] keeps its own copy of the loop, for
the argued reason stated there and here. This asserts the one number
the two copies must not disagree about — if `measure_linear` ever
decides a pick may take three clicks, or this module does, the two
stop being the same behaviour and a reader comparing their traces is
misled.

It is keyed on the OTHER module's constant, not on a literal
restatement of it here, so it measures the copy rather than measuring
this test's own opinion.
