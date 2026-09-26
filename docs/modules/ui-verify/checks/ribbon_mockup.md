# `ui-verify/checks/ribbon_mockup`

`ribbon_matches_the_mockup_geometry` — the band, measured against
`mockups/pdfcer-shell.html`.

# Why this check exists, and why it was written UNRUN


> *"there are still a lot of things that still look like our old layout
> including text label location and missing glyphs."*

and, at length, the biggest one: **every item in the shipped band is drawn
inside a visible button frame, and the mockup draws them frameless.**

All four were fixed in the same session, and every fix carries a unit test
that measures a rectangle or a metric. **Not one of those tests can see
whether the band LOOKS like the mockup**, and the distinction is not
pedantic — it is this project's standing finding, written into
`MODES_AND_PANELS.md`: *layout and appearance defects have exactly one
oracle, a rendered screenshot.* The two caption-less ribbon groups that
started this whole harness were found by a screenshot while every unit
test passed.

The session that wrote the fix could not take one. The operator was at his
keyboard, a watchdog kills GUI processes on sight, and raising a window
takes his focus. So this file is the check that would have settled it,
written from the same CSS the fix was translated from, and **left unrun**.
Running it is one command:

```text
cargo run -p ui-verify -- --check ribbon_matches_the_mockup_geometry \
    --exe target/debug/pdfcer-gui.exe --width 1700
```

1700 rather than the harness's usual 1100 is deliberate and is the second
half of the reason the comparison was inconclusive: the mockup was rendered
at 1700 px and the only recent capture of the real File tab
(`target/uv-icons/ribbon_captions.png`) is 1100 px wide. Some of what read
as "missing" in that comparison is `RIBBON_SCALING.md`'s collapse ladder
working correctly at a narrower width. **A fair comparison has to be at the
same width**, and a check that drove 1100 would re-file the same false
finding.

# What it asserts, and which half of each pair a unit test could already do

| # | claim | mockup | a unit test can see it? |
|---|---|---|---|
| 1 | the band's first control sits clear of the tab strip | `.ribbon { padding: 6px … }` | yes — `egui-shell`'s `the_band_draws_clear_space_above_its_first_control` |
| 2 | a Large control is 56 pt, not the full row area | `.rb.big { height: 56px }` | yes — `a_large_control_is_shorter_than_the_row_area_it_sits_in` |
| 3 | the caption hangs at the bottom of the row area | `.grp .cap { margin-top: auto }` | yes — `every_caption_in_a_band_shares_one_baseline` |
| 4 | **a resting control paints no frame** | `.rb { border: 1px solid transparent }` | **NO** |
| 5 | **every control draws a glyph** | `svg.g` | **NO** |

Rows 1–3 are re-asserted here anyway, and that is not duplication: a unit
test measures what the layout code *computed*, and this measures what the
process *published while drawing on a real screen at a real DPI*. The two
have disagreed before in this codebase — `sizing::render_large`'s
zero-height overflow-menu defect passed every unit test and was found by
this harness — and when they disagree, this one is right.

**Rows 4 and 5 are the reason the file exists.** Both are questions
about ink, and a rect cannot answer either:

* A frame is `weak_bg_fill` plus `bg_stroke` painted into a rectangle the
  control occupies **whether or not the frame is drawn** — that is
  precisely what makes `Button::frame_when_inactive(false)` safe, and
  precisely what makes it invisible to a geometry test. Every rect in the
  trace is identical before and after the fix.
* A missing glyph is `icons::paint_missing_mark`'s slashed box, which
  occupies exactly the rect a real glyph would. The band reports the item;
  the item reports its size; nothing reports what was painted inside it.

# How row 4 is measured, since "is there a frame?" needs a definition

[`is_frameless`] samples a one-pixel ring just inside a control's declared
rectangle and compares it with a ring just outside. A framed control
differs on both counts — a fill that is not the band's, and a stroke on the
boundary. A frameless one is the band's own colour right up to and across
its edge, because nothing was painted there at all.

It samples the **corners' neighbourhoods rather than the whole ring**,
and skips any sample that lands on ink: a control's icon and label are
inside its rect and are supposed to be different from the background. The
corners of a ribbon button are the one part reliably empty of content in
both designs, which is what makes them the right place to ask about the
frame and the wrong place to ask about anything else.

# Which control, and why the check picks it rather than taking one

It must be **resting**: not hovered, not focused, not selected. A selected
control draws its plate at rest by design (`.rb[aria-pressed="true"]`), and
a check that happened to sample `View ▸ Scroll` — the page-display mode
that is on by default — would report a frame and file a defect against the
one behaviour the fix deliberately kept.

So it drives the **File** tab, whose band holds no toggle at all, and it
parks the pointer at the window's bottom-left corner before capturing. Both
are stated in [`RESTING_TAB`] and [`assess`] rather than assumed.

## Item notes

### `const RESTING_TAB`

**File**, and the choice is load-bearing rather than alphabetical: it is
the tab the operator compared, it is the widest one, and — the property
row 4 depends on — **not one of its controls is a toggle**. Every other tab
carries at least one command that is selected at rest (View's page display,
View's armed tool, Markup's shape), and a selected control draws a plate on
purpose.

### `const MOCKUP_WIDTH`

See the module header: comparing a 1700 px mockup with an 1100 px capture
makes the collapse ladder look like a defect. The default is stated here so
a run that does not pass `--width` still compares like with like.

### `const SLACK`

One point. Below what anyone can see, above `egui`'s own rounding, and the
same slack `egui-shell`'s own layout tests use — deliberately, so a
disagreement between the two is a real disagreement rather than two
tolerances.

### `fn board`

The oracle needs its own test for the reason `PROJECT_PLAN.md` §4.1
keeps restating: a predicate that has only ever been seen to say "yes"
is indistinguishable from one that cannot say "no". This file's whole
value is one boolean, and that boolean is asserted here against both
answers.

### `fn a_control_off_the_capture_is_declined_rather_than_guessed`

The third answer, and the one that keeps the other two honest. A
control laid out past the window's edge — the state
`RIBBON_SCALING.md`'s scroll rung exists to make reachable, and the
state `sizing::render_large`'s zero-height defect actually shipped in —
has no pixels to sample. An oracle that answered `true` there would let
every off-screen control certify the band as frameless, which is the
exact shape of *"a check that cannot fail"*.

The rect is wholly off a 40×30 board, so every one of the four probe
pairs falls outside and `judged` stays zero.

### `fn a_control_at_the_left_edge_is_still_judged_on_its_other_corners`

This is a real state, not a fixture curiosity: the first item of the
first group sits at the band's left edge once the band has scrolled.
Its two left-hand probes would need a negative x, and the two failure
modes this pins are the two obvious ways to write that:

1. **Plain `u32` subtraction panics**, in debug, on the first
   left-edge control the harness meets — which is a driven check that
   dies rather than reporting, on a state the collapse ladder makes
   ordinary. `checked_sub` is what stops it.
2. **Declining the whole control** because one corner could not be
   probed throws away the three corners that could, and a band whose
   leftmost control is never judged is a band whose frame is never
   checked where the operator looks first.

Note what this does **not** distinguish, because a falsification
pass found it out rather than assuming: clamping the probe to zero
instead of declining it passes this test. It does so for a benign
reason — a clamped "outside" probe lands on the control's own left
column, which `far(outside)` then rejects as not-the-band, so the
corner is skipped either way. The decline is the clearer statement of
intent; the guard is what actually carries it.

### `fn a_ground_taken_from_a_collapsed_groups_plate_produces_no_verdict`

The incident, 2026-09-05: [`assess`] sampled its reference from the
first `ribbon.group.file.*` region it found, and on that run the first
one was `…export.collapsed` — a captioned button with a **plate** under
it and no items inside it. The plate is `#E8E8EA`; the band is
`#F2F2F3`; the channel-sum distance is **29** against `far`'s threshold
of 24. Every probe pair was then discarded as "not the background",
`judged` fell to zero for the whole band, and the check reported
*"0 resting band controls were judged for a frame"* — a PASS the day
before, and nothing measured the day after.

This pins the consequence rather than the cause, deliberately: the
cause is one `find` predicate in [`assess`] and would be re-broken by
any future region name that is a `ribbon.group.*` and not a band group.
What must never change is that a wrong reference **refuses** instead of
answering — `None`, not `Some(true)`. An oracle that certified the band
frameless while sampling zero pixels is the exact failure this file's
header says it exists to remove.

The two figures are the measured ones, so a theme change that narrows
the gap below `far`'s threshold turns this test red rather than turning
the check silently vacuous.
