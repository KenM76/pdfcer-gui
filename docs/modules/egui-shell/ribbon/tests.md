# `egui-shell/ribbon/tests`

## Item notes

### `fn every_rendered_group_emits_a_caption`

This is the invariant the band exists to make structural. Two
caption-less groups shipped in the salvage source and were caught
by a screenshot, not by a test: nothing was *wrong*, two call
sites simply did not follow a convention.

The test asserts it three ways, on purpose, because each catches a
different regression:

1. **The counts match.** Catches a second drawing path being added
   that skips the caption.
2. **A caption rect is published for every group id, inside that
   group's own rect.** Catches a caption drawn somewhere other
   than under its own group — which is what an inline caption
   looks like geometrically, and an inline caption is the shape
   that makes the grouping invisible.

   It deliberately does **not** assert a positive height. This
   crate depends on `egui` with `default-features = false`, so a
   test process has no font data and every galley measures zero;
   a height assertion here would be measuring the absence of a
   font, not the presence of a caption. Legibility is what
   `ui-verify` asserts against a real window, on the frame the
   rect was measured on — which is the entire reason the rects
   are published rather than assumed.
3. **The uncaptioned group still produced one.** Catches the
   fallback in [`band::caption_text`] being "simplified" to an
   empty string, which would reintroduce the original defect for
   exactly the manifests most likely to have it.

### `fn every_band_command_publishes_the_rect_it_was_drawn_in`

# The gap this closes

The band reported its *groups* and their *captions* and nothing else,
so the forty controls an operator actually clicks were invisible to
anything outside the process. A caption rect answers "is this label
legible"; it cannot answer "did clicking this button do anything",
because nothing outside the window could find the button in order to
click it. The whole chain from a ribbon click to a control rendering
pressed was therefore covered only by unit tests, one per link — which
is exactly the state the icon painter was in when it shipped drawing
nothing: every part tested, the join untested, the join wrong.

# What is asserted, and why each part

1. **One rect per `Item::Command` in the drawn tab**, under
   [`report::band_item`]'s name. Catches the report being conditioned
   on something — enabled, selected, having an icon — which would make
   it go quiet in the states a harness most wants to inspect.
2. **Inside its own group's rect.** Catches a control reported at a
   rectangle that is not where it was drawn, which would aim a click
   at empty chrome and make a working command look broken.
3. **Nothing is published for `Item::Separator` or `Item::Custom`.**
   Neither is a command and neither has an id to report under; a rect
   appearing for one would mean the name is being built from
   something other than a command id.
4. **`ribbon.item.` and `ribbon.qat.` stay disjoint.** `file.open` is
   on the File tab *and* on the quick-access toolbar in this fixture,
   so the two surfaces genuinely draw the same command twice on one
   frame. A harness filtering for band controls must not pick up the
   QAT's copy, and `qat_icons`' aspect-ratio assertion must not pick
   up the band's — a band control legitimately shows a label and is
   legitimately wide.

Sizes are deliberately not asserted: this crate builds `egui` with
`default-features = false`, so a test process has no font data and
every galley measures zero. What a control *looks* like is
`ui-verify`'s question, against a real window, using precisely these
rects.

### `fn the_overflow_control_is_hit_testable_at_a_width_that_hides_groups`

The observed defect: *"past ~6 tabs the overflow button itself
gets hidden, leaving no route to the hidden tabs."*

"Hit-testable" is asserted by **hovering it**, not by checking
that a rectangle exists. A rectangle proves something was
allocated; only `egui`'s own hit test proves it can be reached,
because that is what accounts for clipping, for occlusion by a
later widget, and for a zero-area interact rect.

The width is chosen below one group's worth of the band, which is
the case a naive implementation gets wrong: there is still *some*
room, so it spends it on a group and has nothing left for the
affordance.

### `fn a_wide_enough_band_shows_every_group_and_no_affordance`

The other half of the previous test: an affordance that never went
away would also satisfy "always reachable", and would be a
permanent tax on a band that fits.

### `fn the_mode_selector_draws_every_position`

`MODES_AND_PANELS.md` Part 1 forbids *"a bare track with a knob,
where the available positions are invisible until you drag."* The
assertable form is: three modes, three segments, each with area.
A knob-and-track implementation publishes one rect and fails here.

### `fn the_mode_selector_sits_right_of_the_tab_strip`

Asserted as a geometric relation rather than as a coordinate, so
it survives a theme change, a font change and a fourth mode —
which is the whole reason rects are published rather than
hard-coded.

### `fn a_click_reports_a_handler_token_and_nothing_else_happens`

A synthetic click on a QAT control returns that command's handler
token — and returns *only* that. Nothing in this crate can act on
it, which is the seam the module header describes.

### `fn a_disabled_command_reports_no_intent`

`SHELL_FRAMEWORK.md` §5: *predicates are safety, not decoration.*
The enable predicate has to hold at the point the intent is
reported, not only at the point the control is greyed — otherwise
a customized ribbon could route around it.

### `fn a_custom_item_is_handed_to_the_application_with_its_context`

The extension point that keeps the item vocabulary from growing a
variant per widget an application happens to want — which is the
road by which a reusable shell acquires a `ColourSwatch` variant
and stops being reusable.

### `fn an_unregistered_command_loses_one_control_not_the_band`

`SHELL_FRAMEWORK.md` §4: an unknown id is a *disclosed skip*, not
a crash. Reaching the renderer with one means the application did
not validate its manifest — a real defect, whose correct penalty
is one missing control rather than a panic in the paint loop with
a document open.

### `fn a_manifest_with_no_modes_draws_no_selector`

A one-position segmented control is a control that cannot be
operated, and a zero-position one is a gap. An application without
modes gets neither.

### `fn the_plain_entry_point_draws_a_working_ribbon`

Every builder capability is optional. An application bringing the
shell up for the first time — no icon set, no custom items, no
harness — must get something that draws, and must not have to
discover four builder methods before it does.

### `fn an_empty_manifest_draws_an_empty_ribbon`

The first frame of an application that has not built its manifest
yet, and the last frame of one whose customization file emptied
itself. Neither is a reason to crash in the paint loop.

### `fn the_mode_selector_moves_with_the_arrow_keys`

`MODES_AND_PANELS.md` Part 1, behavioural rule 6: *"the selector
is a real focusable control with arrow-key movement — not a
mouse-only affordance."*

Driven the way a keyboard user would: focus the control, press
Right, press Right again, press Right a third time at the end.
The third press must do nothing, because
[`mode_selector::move_index`] clamps — a wrap would take the most
capable stance straight to the least in one keystroke.

This asserts the whole path — focus registration, key
consumption, index movement, state write — rather than the pure
[`mode_selector::move_index`] that `arrow_movement_clamps_rather_than_wrapping`
covers on its own. Either alone would pass with the other broken.

### `fn a_group_in_a_popup_is_captioned_too`

The menu is the place a second, simpler drawing path would be
most tempting — it is a vertical list, the band's centring does
not obviously apply, and nobody looks at it in a screenshot. That
is exactly how the two caption-less groups in the salvage source
happened, so the menu is routed through the same
`band::captioned_group` closure as the band, and this asserts it.

The count is the whole point: with the menu open, **every** group
on the tab has been drawn and every one of them emitted a caption.

### `fn the_builder_takes_plain_local_closures`

Worth a test because the four capabilities are `&'a mut dyn`
borrows and an over-constrained signature would compile here in
this crate and fail in the *application*, at the one call site
that matters, with a lifetime error nobody can read.

### `fn the_icon_request_reports_selected_state`

# Why this needs a test at all

A field that is always `false` compiles, renders, and looks correct.
The shell already draws selection with the button's frame, so an icon
set that ignored a permanently-false flag would produce a ribbon
nobody could tell was wrong by looking at it — and the whole reason
the field exists is to let an application keep the rule *selected
state is never colour alone*, which is precisely the rule whose
violations are invisible in the preset you happen to be using.

So both polarities are asserted in one run, from one registry, with
the only difference being the condition set. Asserting `true` alone
would pass against a field hard-wired to `true`.

### `fn two_ribbons_can_coexist_with_distinct_id_salts`

Without a distinct base id the symptom is that hovering a control
in one window highlights the corresponding control in the other —
a bug that is baffling until it is understood and trivial
afterwards.

### `fn scrolling_right_moves_the_band_and_offers_the_way_back`

The left arrow is the one control on the scroll surface a static
observation cannot reach. Every other part of the scroll can be measured
against the running application without touching anything: the right arrow
appears at 1000 pt and not at 1200, the ladder compacts in the right order,
the band keeps its height. The left arrow exists only once something has
scrolled — so unless a check *scrolls*, that control is **built and never
observed**.

*A check that cannot fail is not evidence*, and neither is a control that
has never been seen to draw. This is the falsification: press the right
arrow, and assert the band moved AND that the way back is on screen.

The second assertion is the one that matters. A scroll that advances with no
way back is not a scrolled band, it is a **trapped** one — every group left
of the fold unreachable for the life of the session, with the ribbon looking
entirely normal.
