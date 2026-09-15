# RIBBON_SCALING.md — how the ribbon narrows

How wide a control is, how a group gives up space, and what happens when there
is still not enough: read it before changing `egui-shell`'s `ribbon::plan`,
`ribbon::sizing`, `ribbon::rhythm` or `ribbon::overflow`, and before declaring a
size, a row hint or a collapse priority in a manifest. It does **not** decide
where a command lives — `RIBBON_IA.md` owns that.

All lengths are **points**, in the ribbon's own coordinate space, in `f32`.

**Walk the series, never the endpoints.** Sampling either side of a transition
and concluding there is no transition is a specific, repeatable error, and it is
what the monotonicity tests here guard against by sweeping hundreds of widths
rather than checking three.

## 1. The ladder

Four responses to a narrowing band, applied in this order. Each rung is
exhausted before the next begins, and the band stops at the first rung that
fits.

| rung | what the band does | what it costs the operator | may a group decline? |
|---|---|---|---|
| natural | every group at its narrowest packing within `GROUP_ROWS` rows | nothing | — |
| re-wrap | a group re-flows onto up to `MAX_GROUP_ROWS` rows | nothing: every control stays on the band, labelled | no |
| collapse | a group becomes one captioned button with a chevron, contents in a popup | one click | yes |
| scroll | groups past the budget are not drawn; `‹` and `›` shift the band | the group is off screen until the operator acts | — |

**The order is by what is lost, and by nothing else.** Re-wrapping hides
nothing, so no group may decline it and it is always better to re-wrap five
groups than to collapse one; collapsing keeps the caption on the band and the
controls one click away; scrolling takes commands off screen. A band that
scrolled first would be hiding commands while the space to show them,
compacted, was still there.

`ribbon::plan::collapse` decides the rungs, and runs **before** `plan_band`
pushes anything past the band's edge. Its three states are `Natural`,
`Rewrapped`, `Collapsed`.

`Candidate::gains_from` **measures** whether advancing a rung is worth
anything, rather than assuming it: a group whose re-wrapped layout is no
narrower than its natural one would otherwise consume a rung, change nothing,
and make the band appear to stall at one width and jump at the next.

**The re-wrap rung currently fires for nothing, and that is arithmetic rather
than policy.** `GROUP_ROWS` and `MAX_GROUP_ROWS` are equal, and
`band::measure_group_rows` is called with the same row ceiling and the same row
hint for both measurements, so `Candidate::rewrapped == Candidate::natural` for
every group and `gains_from` skips the rung. Nothing is lost by it: the effect
the rung exists for is already inside the natural measurement, because every
group is planned against the ceiling (§4). The rung becomes live again the
moment the natural ceiling drops below `MAX_GROUP_ROWS`, and the machinery is
kept for that reason.

## 2. Band geometry, and the height that must not move

**R128.** The ribbon sits directly above the canvas, so a band that changed
height would move the canvas under a fit-to-page zoom — the feedback loop this
module is arranged against. The height is derived from the theme, is the same at
every width and on every tab, and is reserved before a single group is measured.
The **row count is a property of what is packed into the fixed budget**, never
the other way round.

The budget, from `ribbon::rhythm`:

```text
band_height = ribbon_pad_top
            + ribbon_rows          (the row area: a budget, not a row count)
            + CAPTION_GAP
            + one line of the caption font
            + BAND_PADDING_BOTTOM
```

Rows are laid into the top of the row area and the caption hangs off the
bottom, so every caption in a band shares one baseline whether its group used
one row or three.

| quantity | how it is derived |
|---|---|
| height of a control on the band | `ribbon_rows / GROUP_ROWS − BAND_ROW_SPACING`, floored at `icon_pts` |
| height of a control in a re-wrapped group | `ribbon_rows / MAX_GROUP_ROWS − BAND_ROW_SPACING` |
| whether re-wrapping is permitted at all | `rhythm::rewrap_is_legible` — the line above is at least `icon_pts` |

The control height **does not read the group's row count**: a height that varied
with it would draw one group's controls at a different size from the group
beside it. What varies is the slack underneath.

The per-preset numbers move. Read them rather than quoting them:

```sh
grep -n "ribbon_rows\|ribbon_pad_top\|icon_pts\|control_height\|gutter" \
    crates/egui-shell/src/theme/mod.rs
```

Every term of the budget is spent by a **named line** in `band::group_body`.
That is deliberate: `egui` advances the cursor past every laid-out rect by
`item_spacing` — **after the last row as much as between rows** — so a group's
column must not also carry the theme's `item_spacing.y`, or the rows overshoot
their budget by exactly one gap and the caption is drawn into the band's
clearance. A gap the framework inserts on your behalf is still a gap you spent.
Inside a group's rows `item_spacing.y` is therefore set to `BAND_ROW_SPACING`.

The band's appearance, from `mockups/pdfcer-shell.html`, which is this band's
specification:

| | value | mockup source |
|---|---|---|
| resting item frame | none — painted on hover, focus, press and selection only | `.rb { border: 1px solid transparent }` |
| a `Large` control | `ribbon_large_pts` tall, `ribbon_icon_large_pts` glyph, label wraps at `LARGE_LABEL_WRAP` | `.rb.big` |
| group caption | `ribbon_caption_pts` | `.grp .cap { font-size: 11px }` |

### The self-disabling guard

The floor at `icon_pts` is a **safety, not a target**: a control that cannot show
its icon is not a smaller control, it is a clipped one. When the compressed row
height falls below `icon_pts`, `rhythm::rewrap_is_legible` returns false, the
re-wrapped measurement is reported as the natural one, `gains_from` sees no gain
and the rung is skipped. **The feature turns itself off rather than clipping.**
If instead the *natural* floor binds, the rows no longer fit their budget and
the band grows on every tab — R128 arriving through the theme table.

That guard is also the hazard: a change to the row area could silently remove a
rung of the ladder, in one preset, with every other test green — a rung that has
switched itself off looks exactly like a rung that was never needed.
`height_tests::every_preset_can_still_re_wrap_a_group` asserts the margin for
**every** preset rather than for the shipped one, and
`every_preset_reserves_room_for_its_own_rows` asserts the natural floor does not
bind.

## 3. The three mechanisms

### 3.1 Item sizes

| size | form |
|---|---|
| `Large` | icon above label, label may wrap to two lines, spans the band's row area |
| `Medium` | small icon, label to the right; several stack vertically. The default |
| `Small` | icon only, optional dropdown chevron |

**Mixing sizes within a group is where a dense band's density comes from.** A
band whose every control is `Medium` has no way to give up space except to
vanish.

`ItemSize` (`egui-shell/src/manifest/item.rs`) is declared **per item** and
defaults to `Medium`, so a manifest that says nothing renders as it always did.

- **`Small` is earned, not asserted.** A control goes icon-only only when it
  names an icon, carries a tooltip, **and** an icon painter is installed — the
  same rule `ribbon::qat::shows_label` applies to the quick-access toolbar,
  resolved in `sizing::resolved`. The tooltip is the icon's accessible name;
  without one an icon-only button is an unlabelled rectangle to a screen reader
  and a guess to everyone else. A `Small` that has not earned it **falls back to
  `Medium`** rather than rendering a mystery, so a manifest may ask for
  icon-only freely and a command that gains or loses an icon needs no audit.
- **`Large` leads its group.** A `Large` control spans the rows and cannot live
  inside the row wrapping, so `sizing` draws the `Large` run first, at the
  group's left, and wraps everything else beside it. Writing `large(...)`
  anywhere but at the head of a group therefore **silently hoists that item to
  the front** — an IA change made by a glyph size, and `RIBBON_IA.md`'s to
  decide. A `Large` is legal exactly where hoisting is a no-op, that is where
  the `Large` items are already a prefix of the group, and
  `manifest::tests::large_items_already_lead_their_group` fails the build
  otherwise.
- A `Large` control's width is the wider of icon and label plus
  `LARGE_SIDE_PADDING` a side, floored at `LARGE_MIN_WIDTH` so a run of them
  reads as a row of equal buttons rather than a ragged fence. `Large` is **not**
  conditional on an icon: a large button with no icon is a large label, legible
  and unambiguous.

**Size is a property of the command, not of the band.** `B`, `I`, `U`, the page
displays and the page rotations are findable by shape and position;
`Export form data…` is not. Tabs full of *named verbs* keep their labels; tabs
full of *iconic* commands go icon-only. In a band of forty controls the label is
often the only thing that makes one findable, and that argument decides every
individual case.

### 3.2 Per-group collapse, in an authored order

A collapsed group is a single control the height of the row area, carrying the
group's **caption** and a chevron; pressing it opens the group's full layout in
a popup. The caption is kept rather than replaced by a glyph, for §3.1's reason:
a collapsed group reading **Export** is findable, an invented arrow-in-a-box is
not. The popup is drawn by `band::captioned_group` — the same renderer as the
band, with the same row split — so a group reads identically on the band and in
its popup.

- **The ranking is not right-to-left and not smallest-first.** The property
  being ranked is **importance**, which is editorial: the group that never
  collapses is the one carrying the verb the operator came to the tab for, not
  the narrowest one. A width- or item-count-based heuristic measures the wrong
  property and no amount of tuning repairs it. It is also what keeps
  `egui-shell` domain-free (R7): the shell reads a number, the application
  decides what matters.
- **A group may declare that it never collapses** — `manifest::Group::collapse`
  absent means never.
- **Every tab keeps at least one group off the ladder.** A tab whose every group
  may collapse reaches a width at which the band is a row of identical chevrons:
  every command still reachable, and the band telling the operator nothing.
  `ladder::tests::every_tab_keeps_something_expanded` enforces it and names its
  one deliberate exception, so that exception cannot be acquired by accident.

Priorities live in one table, `pdfcer-gui/src/shell/manifest/ladder.rs`, as
`(tab, group, priority)` rows applied to the built manifest, rather than on each
group — a collapse priority is a *ranking of groups against each other* and a
ranking can only be reviewed all at once. Lower collapses first; ties break on
manifest order; a group with no ladder entry never collapses. The priorities are
sparse so a group can be inserted between two rungs without renumbering a tab,
and `ladder::tests::every_ladder_entry_names_a_real_group` fails the build when
an entry names a group that no longer exists — the guard that pays for keeping
the ranking away from the definitions.

### 3.3 Scrolling

The last rung. `‹` and `›` at the band's edges shift it by **exactly one group**
per click; every group stays a group, in manifest order, and the window is a
viewport onto a row wider than itself. The band scrolls and has no dropdown:
two affordances for one job is a defect, and a group simultaneously "on the
ribbon" and "in a menu" is two mental models for one surface. The **tab strip**
is a different problem and keeps its own `⏷ N more` menu, whose one hard rule is
that it pins the active tab, because the one thing a strip must never hide is
the tab its menu cannot reach.

- **The affordance's width is subtracted first**, from the band's edge, before
  any group is measured: `overflow::arrow_width` is passed to `plan::plan_band`
  as the reservation, and the group budget is what remains. The reservation can
  therefore never be the thing that gets squeezed out, whatever the group loop
  does — the failure mode where the overflow control itself disappears and the
  hidden content has no route.
- **The left arrow is reserved too, but only while the band is scrolled.** It is
  drawn after the groups and wins the hit test wherever it overlaps one, so an
  unreserved left arrow makes the leading control unreachable and turns a click
  on it into a scroll.
- **The scroll position is an input to layout, and is clamped every frame.**
  `overflow::first` is a group index in `egui::Memory` keyed on the tab id, so a
  tab returns to where it was left and nothing is written to disk where a window
  resized on another monitor could strand it. `overflow::clamp` computes the
  furthest-left index that still fills the band **from the widths alone**; it
  never reads what was drawn, because recomputing the position from the drawn
  result is the measurement-feeding-its-own-size loop again.
- **A glyph is not an accessible name.** Each arrow announces the count it moves
  past, from `plan::overflow_label`. The right arrow keeps the region name
  `ribbon.overflow`, which is a cross-repo contract with the driven checks.
- An arrow with nowhere to go is **not drawn**, not greyed: R9 reserves greying
  for *temporarily* unavailable.

## 4. How a group wraps

`plan::wrap_group` partitions a group's items into **contiguous runs** — the
manifest's order is the operator's order — and returns the split together with
the width of the **widest** row, which is what the group costs the band.
Returning both from one call is what makes it impossible for the planner to
measure one split and the renderer to draw another.

It searches for the **narrowest** packing that fits the row ceiling it is given,
by trying every contiguous run width as a target, ascending. A ceiling is
therefore a ceiling and not a target: a group that reaches its narrowest at two
rows stays at two even when three are permitted.

| constant | what it is |
|---|---|
| `GROUP_ROWS` | the natural row split. A **constant**, never per-group, per-tab or per-theme: a knob with one sane value that every consumer must set identically gets the drift on the first one that does not |
| `MAX_GROUP_ROWS` | the re-wrap ceiling |
| `GROUP_WRAP_WIDTH` | the row width at which a group that has **not** asked for rows wraps at all. A **trigger, not a target**, from `mockups/ribbon.html`'s `.gcmds { max-width: 440px }` |
| `GROUP_PADDING` | inset each side; group width is `max(content, caption) + 2 ×` it, and the caption is measured in the font it is **drawn** in |
| `MIN_ITEM_WIDTH` | the floor under every control |
| `CUSTOM_ITEM_WIDTH` | the budget for a `Custom` item, whose drawing the shell cannot know |

```sh
grep -n "const GROUP_ROWS\|const MAX_GROUP_ROWS\|const GROUP_WRAP_WIDTH" \
    crates/egui-shell/src/ribbon/plan/mod.rs
```

The band asks for rows for **every** group — `prefer_rows`, or the ceiling the
group is being planned against — which skips `wrap_group`'s fits-already
short-circuit. Without that, a band with three rows of budget and one row of
content leaves groups collapsed while most of the band is unused.

Item widths come from `sizing::width`, one branch per size, each with a matching
branch in the renderer. A command id that is not registered measures **0** and
draws nothing: one control lost, not the band (R8).

Widths are **estimated analytically** from the item list and the same font
metrics `egui` will use, rather than by drawing and re-laying-out (a visible
flicker on every resize) or by drawing into a discarded layer (double work, and
every hover and click fires twice). An estimate that is too small costs a
clipped group; it cannot cost the affordance, because the reservation was
subtracted before the estimate was consulted.

`MIN_ITEM_WIDTH` has a second effect worth naming: `egui-shell` is built with
`default-features = false`, so a test process has no fonts and every galley
measures near zero. The floor is what keeps this arithmetic meaningful — and the
overflow tests honest — in exactly the environment they run in. It does not make
a zero-width-text test *equivalent* to a real one, which is why `width_tests`
installs a font and `height_tests` also applies a `Theme`.

**A fourth row is refused, and the arithmetic is the reason.** At the shipped
preset `ribbon_rows / 4 − BAND_ROW_SPACING` is exactly `icon_pts`, with nothing
left over: the next change to the row area or the icon size fails
`rewrap_is_legible` and disables the re-wrap rung outright. A fourth row would
otherwise have to grow the band — moving the canvas under a fit-to-page zoom —
or shrink controls below the size at which their icons are legible.

## 5. What the manifest declares

### 5.1 `ItemSize`, per item

`Medium` by default. `pdfcer-gui`'s manifest helpers are `command`, `icon_only`
and `large`; §3.1 carries the earned-`Small` rule and the `Large`-leads rule
that constrain them.

### 5.2 `Group::collapse` and `Group::prefer_rows`

`collapse: Some(n)` enters the group in the ladder at priority `n`; absent means
never. It is set from `ladder::LADDER` rather than at each group's definition,
for §3.2's reason.

`prefer_rows: Some(n)` lays the group out on several rows **even when one row
would fit**. It is a hint, not a height: the band's ceiling still wins, and the
planner still returns the narrowest packing it can find, so a group of two items
asking for two rows gets whichever of 1 × 2 and 2 × 1 is narrower. It exists
because wrapping under pressure and wrapping by design are different things —
four square icon buttons in a row is a strip, and the same four as a 2 × 2 block
is half the width and reads as one control. `group_two_rows` is the manifest
helper; `plan::tests::a_group_that_asks_for_rows_wraps_when_it_would_otherwise_fit`
holds it.

### 5.3 `visible_when`, per item

A condition name, evaluated every frame against the same `ConditionSet` that
decides enablement. **This is visibility, not enablement**, and the difference is
R9: an unavailable capability renders nothing, while greying is reserved for
*temporarily* unavailable and is always explained on hover. It is R9 extended
from "the command was never registered" to "the command does not apply here".

A hidden item is removed **before measurement** (`sizing::visible`), so its space
is reclaimed and the group re-flows. **A group with nothing visible left is not
drawn at all**, and its separator goes with it: the filter runs before the
planner sees the group list, so no width is reserved, no rule is drawn beside it,
and the groups to its right move left.

This is what lets one tab definition serve Read, Review and Edit with different
contents rather than three near-identical tabs.

## 6. Invariants

| invariant | what holds it |
|---|---|
| shown groups are a prefix of the manifest order, and `shown + hidden == n` however many of the shown are compacted | `plan::tests::the_visible_groups_are_a_prefix_and_nothing_is_lost` |
| widening never hides a group that was visible | `plan::tests::widening_the_band_never_hides_a_group_that_was_visible`, and `width_tests::widening_the_band_never_hides_a_group_under_real_metrics` |
| widening never compacts a group further — natural before re-wrapped before collapsed, at every width | `plan::collapse::tests::widening_the_band_never_compacts_a_group_further` |
| the re-wrap rung is exhausted before any group collapses, and stops as soon as it fits | `collapse::tests::every_group_rewraps_before_any_group_collapses`, `..::the_rewrap_rung_stops_as_soon_as_it_fits` |
| collapsing follows priority, and a group that declines to collapse still re-wraps | `collapse::tests::collapsing_starts_after_the_rewrap_rung_and_follows_priority`, `..::a_group_that_declines_to_collapse_still_rewraps` |
| a band that fits reserves nothing | `plan::tests::a_band_that_fits_reserves_nothing`, `collapse::tests::a_band_that_fits_is_left_alone` |
| the group budget never contains the reservation | `plan::tests::the_group_budget_never_contains_the_reservation` |
| the reservation covers the widest label it could show, by width and not by digit count | `plan::tests::the_reservation_is_sized_for_the_worst_case_label`, `..::the_reservation_covers_the_widest_label_by_width_not_by_digit_count` |
| no visible group overlaps either arrow, and the affordance is on screen and hit-testable at every width | `width_tests::no_visible_group_overlaps_the_overflow_affordance`, `..::the_overflow_affordance_is_on_screen_at_every_width`, `..::a_band_narrower_than_the_affordance_still_shows_it`, `scroll_tests::no_visible_group_overlaps_the_left_scroll_arrow` |
| a band that claims to fit really does fit, against real font metrics | `width_tests::a_band_that_claims_to_fit_really_does_fit`, `..::the_band_measures_real_text_and_not_a_floor` |
| the band is the same height on every tab and at widths where every group overflows | `height_tests::the_band_is_the_same_height_on_every_tab`, `..::the_band_keeps_its_height_at_widths_where_every_group_overflows` |
| every caption in a band shares one baseline, with clear space beneath it | `height_tests::every_caption_in_a_band_shares_one_baseline`, `..::the_band_leaves_clear_space_beneath_its_captions` |
| a group is inset by exactly the padding the plan budgeted for it | `height_tests::a_group_is_inset_by_the_padding_the_plan_budgets_for_it` |
| one oversized control does not stop the rest of its group wrapping | `plan::tests::one_oversized_control_still_lets_the_rest_wrap` |

The first two hold **by construction, not merely by testing**: `fit` always
starts from *everything natural* and advances forward in a fixed order until it
fits, and it **never starts from the previous frame's answer**. A group's state
at width `w` is a pure function of `w`, it only moves back up the ladder as `w`
grows, and there is no hysteresis to tune. That is also why the ladder is a free
function over slices rather than a method on something that remembers: **a
layout that remembers what it did last frame is how a ribbon acquires a width at
which it flickers.**

## 7. Deliberately not done

- **Galleries that shrink by showing fewer tiles.** There are none here, and
  adding a shrinking one to carry a scaling mechanism would be inventing a
  control to justify a rule.
- **Tab labels that clip without an ellipsis.** Ours truncate with `…`, and the
  tab strip's own menu handles the rest.
- **A per-group, per-tab or per-theme row count**, and **a fourth row** (§4).
- **An icon on a collapsed group** in place of its caption (§3.2).

## 8. How to check a change

```sh
cargo test -p egui-shell                     # the plan, the rhythm, the widths
cargo test -p pdfcer-gui shell::manifest     # the ladder table and the Large rule
cargo run --release -q -p ui-verify -- --exe target/release/pdfcer-gui.exe \
    --pdf D:/Dev/temp/pdfcer/SW41177.pdf --only ribbon_matches_the_mockup_geometry
```

The driven checks that cover this document are
`ribbon_matches_the_mockup_geometry`, `ribbon_group_captions_legible`,
`a_command_two_scroll_stops_away_is_still_reachable` and
`the_display_buttons_stack_in_two_rows`.

To measure the real band rather than reason about it, run the binary with
`EGUI_SHELL_DIAG=1` and read the published regions out of the trace:
`ribbon.group.<tab>.<group>`, `.caption`, `.collapsed`,
`ribbon.item.<command_id>` and `ribbon.overflow`. Their rectangles are what the
band actually drew, at the width the window actually was.
