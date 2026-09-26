# `ui-verify/checks/layers_search`

`layers_search_narrows_the_list` — **the Layers search field is drawn,
is reachable, and narrowing the list is not the same as emptying it.**

# What this is for

`OPERATOR_REQUESTS.md` O126: *"there is a search to implement on the
layers"*.

`panels::layers::search`'s unit tests own the **predicate** — case
folding, substrings, trimming, the multi-byte walk — and they are swept
with no window open. This check answers the two things they cannot:

1. **Is the field on screen at all?** A predicate with no control in
   front of it is a function nobody can call. `panels::layers` draws the
   field only when the document has at least
   `search::MIN_LAYERS_FOR_SEARCH` layers, so a fixture with too few
   layers would produce a green run about a control that is not there —
   which is why this check refuses to pass on absence (see below).
2. **Is it reachable, rather than merely laid out?** The field lives at
   the top of a panel body that is clipped into its stack. A rect proves
   layout; `crate::diag::ui_rect_visible` is what proves visibility, and
   the dock's whole rect stream goes through it.

# ★★★ Why this check cannot pass on an absence

The field is drawn conditionally, and `crate::diag::ui_rect` is a **change
log** — it emits a line when a rect appears or moves, and
`ui-rect-gone` when it retires. So "no `panel.layers.search` line" has
three causes that look identical from here:

| cause | is it a defect? |
|---|---|
| the field regressed and is not drawn | **yes** |
| the fixture has fewer than two layers | no — correct behaviour |
| the Layers panel is not on screen in this mode | no |

`D:/dev/rag/egui/a_check_that_may_decline_to_judge_is_a_check_that_cannot_fail.md`
is the rule: a check whose "nothing happened" branch is a pass is a check
that has stopped running and will not say so. So this one **establishes
its own precondition first** — it asserts the panel drew at all by looking
for `layer-row`, which `panels::layers` traces once per drawn row — and
only then requires the field. A fixture with too few layers makes the
check ERROR with a message naming the fixture, not pass.

# ★★★ What is deliberately NOT covered, and it is the honest limit

**Typing, and therefore the narrowed and empty states.**

There is no seam in this application that fills a `TextEdit`, and adding
one would be a second way to set a value the operator sets exactly one
way — which is the shape of harness affordance that ends up exercising a
path no operator has. Driving it properly needs synthetic keystrokes into
a focused field, i.e. a pointer to focus it first, which puts this check
in the class that cannot run on a machine somebody is using.

⇒ So the three states behind the field are held by unit tests instead,
and they are held tightly:

| state | where it is proved |
|---|---|
| the list narrows, case-insensitively, on substrings | `panels::layers::search::tests` — nine tests, swept |
| clearing restores the list | `an_empty_query_matches_every_layer` |
| an empty result is distinguishable from an empty document | `an_empty_result_knows_it_was_the_query_that_emptied_it` |
| the empty result quotes the query back | `text::panels::layersearch::tests::the_empty_case_repeats_what_was_typed` |

Every one of those was falsified by planting the inverted behaviour. What
**no** unit test can say is whether the field is on screen — and that is
precisely what this check is for. The division is deliberate: the pure
rule to the sweep, the existence of the control to the driven run.
