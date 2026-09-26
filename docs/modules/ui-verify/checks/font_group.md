# `ui-verify/checks/font_group`

`the_format_tab_offers_font_controls_for_swept_text` — **the ribbon route
to a restyle, and the sentence that tells an operator how to reach it.**

# What this is for, and how it differs from `restyle_text`


O37 shipped with an admission written into its own row:

> You are in Edit mode, so a drag with the Select tool draws a marquee round
> objects. Press **T** first — that arms the text tool — then sweep across
> the words. **That is a discoverability gap and it is ours, not a
> limitation. Nothing on screen tells you to press T.**

Three surfaces now do. This check asserts two of them in the state an
operator is actually in when they need them, which is the state **before**
anything is swept — and that ordering is the whole design of the check.

## The two phases, and why the first one has to come first

| phase | the operator's state | what must be true |
|---|---|---|
| 1 | clicked a piece of text with the Select tool; **nothing swept** | the Format tab appears, it carries a **Font** group, and the Properties panel says how to get an operand |
| 2 | pressed `T`, swept the words | the ribbon's **Bold** commits a restyle to the document |

Phase 1 cannot be reached after phase 2, because a sweep is not undone by
clicking again — so a check that swept first would have destroyed the state
it is meant to observe. That is not a harness convenience; it is the
operator's own sequence. They click the thing they want to change *before*
they know a sweep is needed, which is precisely why the gap existed.

## What phase 1 can and cannot see, said plainly

The ribbon publishes `ribbon.item.<id>` for **every** command control,
enabled or greyed — deliberately, and `egui_shell::ribbon::control`'s own
note says why: *"the question a consumer asks is where is this control, and
a control that is greyed is still a control that was drawn somewhere."*

So a region tells this check the control is **on screen**. It does not tell
it the control is greyed. That is a real limit and it is not papered over:
the greying is asserted by
`app::conditions::tests::the_font_groups_visibility_follows_the_mode_and_its_enablement_the_sweep`,
which reads the registered command's own predicate against the published
conditions — the join, not either half. What *this* check adds, and what no
unit test can, is that the controls are **drawn at all**, on a real ribbon,
in a window, at a real width, on the tab that really appeared.

And the appearing is itself under test. The Format tab is contextual: its
`visible_when` moved from `selection.any` to `selection.formattable` when
the Font group landed, and a build that missed that change shows **no tab**
after a sweep and therefore no Font group — a whole feature with no surface,
which is exactly the shape of defect this project exists to catch.




The answer was in the trace the check was already holding:
`pdfcer-diag properties-panel object=832 kind=Path notes=0`. So
[`aimed_at_one_text_object`] now reads that line, plus
`canvas-selection … sel=N`, and **skips** — never fails — when the click did
not leave exactly one text object selected. Phase 2 had this guard from the
start (`chars == 0` is a skip, not a failure); phase 1 did not, and the
asymmetry is what let a correct program be blamed.

The correct aim for this fixture is `--doc-point 0,1140,62`, which
`RESUME.md`'s aim table gives and the sweep did not use: a 5 pt title-block
run at PDF (1135.7, 58.4)–(1190.5, 63.4).

# The oracle

Phase 1: the precondition above, then the regions `ribbon.tab.format`,
`ribbon.group.format.font`, the five `ribbon.item.format.*`, and
`properties.text` with `properties.text.face` inside it.


Phase 2: `text-style-applied … applied=N` **and** the `format-text` label
`vector_edit` writes when the edit reached the engine — the same two-line
oracle `restyle_text` uses, for its reason: the first without the second is
a module that decided to act and whose action never landed.

## Item notes

### `const VK_T`

Pressed only in **phase 2**, and the fact that phase 1 works without it is
the point of the check: the whole complaint is that an operator does not
know to press it, so the surfaces that tell them must be observed in the
state where they have not.

### `enum Aim`

Separated from the wording above so the READ is testable without a running
program: every variant here is reachable from a three-line trace, and the
tests at the foot of this module reach all four. That is the whole point of
the split — a guard against a harness misreading its own oracle is worth
nothing if the guard itself can only be exercised by driving the mouse.

### `fn aim_verdict`

**Order matters, and it is "what" before "how many".** A click that lands
on a path inside a marquee of eleven is an aim problem twice over, and the
kind is the more useful half to be told about: it names the fixture
coordinate that has to change. Reporting "11 objects selected" first would
send a reader looking for a stray Shift.

### `fn list_of`

`driving::list` takes owned `String`s and `driving::list_str` takes a slice
of `&str` — this is the latter, spelled locally only because the filter
above produces a `Vec<&str>` and handing it straight over reads better than
a collect-into-owned at the call site.

### `fn the_two_font_lists_describe_the_same_five_controls`

Two hand-written lists over one set is the shape a completeness check
goes blind in: a sixth control added to the band and to one list reads as
a measured group from either end. The pairing is mechanical — a region
is `ribbon.item.` followed by the command id — so it can be asserted
even though neither list is derived from the other.

### `fn a_click_that_landed_on_a_path_is_the_harnesss_aim_and_not_a_defect`

These are the actual values from a run at `--doc-point 0,300,500` — a
drawing view on `SW41177.pdf` — where the check reported the program's
correct silence as O37's complaint returning.
The verdict must be a SKIP that names the kind, so the reader is sent to
the fixture coordinate rather than to `panels::properties::text`.

### `fn a_multi_selection_whose_first_object_is_text_still_skips`

This is the case `properties-panel` alone cannot see: it describes the
first selected object and says nothing about how many there are, which
is why the count is read from `canvas-selection` instead of inferred.

### `fn a_missing_selection_count_is_zero_and_not_one`

`canvas-selection` is written through `diag::trace_changed`, so a run
that never changed its selection carries no line — and defaulting that
to one would let the guard pass on silence, which is the failure mode
the guard exists to end.

### `fn the_oracle_names_are_the_ones_the_program_writes`

Pinned here rather than trusted: the two lines are quoted verbatim from
`canvas::trace` and `panels::properties::mod::object_section`, and the
kind spelling is `summary::ObjectKind::Text` under `{:?}`.
