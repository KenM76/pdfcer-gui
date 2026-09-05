//! Layout tests for the band's **rows** and its **constant height**.
//!
//! ★ It said *"two rows"* until 2026-09-05, when `plan::GROUP_ROWS` became
//! **three** to match `mockups/pdfcer-shell.html`'s own arithmetic
//! (`.rb { height: 22px }` + `.grp .col { gap: 1px }` = the theme's 68 pt row
//! area). Nothing about the CLAIMS below moved: the band is still one height on
//! every tab, the captions still share one baseline, and the padding is still
//! asserted against drawn rectangles. Only the row count did, and the
//! precondition guards in three of these tests now read `plan::GROUP_ROWS`
//! rather than a literal `2`, so the next change to it fails them loudly
//! instead of leaving them comparing two identical one-row bands.
//!
//! # Why this is a file of its own
//!
//! Three reasons, and they are the same three [`super`]'s module list gives
//! for [`super::width_tests`] existing separately from [`super::tests`].
//!
//! 1. **R2 caps a source file at 1,500 lines.** These tests arrived into a
//!    `width_tests.rs` that was already at 1,412 and would have pushed it to
//!    1,757.
//! 2. **They are a different claim.** `width_tests` asks *"is the overflow
//!    affordance reachable"* — a horizontal question about failure mode #8.
//!    Everything here is vertical: how many rows a group uses, how tall the
//!    band is, and whether that height is the same on two different tabs.
//! 3. **They have a different reason to exist.** `width_tests` is a
//!    regression file: every test in it pins a defect that shipped. This one
//!    pins `PROJECT_PLAN.md`'s **R128** — a rule, in front of a change that
//!    could break it — which is a claim about the future rather than about
//!    the past.
//!
//! # The harness is [`super::width_tests`]', deliberately
//!
//! [`context`], [`render_shell_with`] and its [`super::width_tests::Rendered`]
//! are imported rather
//! than re-implemented. A second synthetic-font installation and a second
//! two-frame render loop would be two harnesses that agree today and
//! disagree after whichever one somebody next edits — and the whole reason
//! `width_tests` installs [`super::testfont`] is that a test measuring
//! nothing looks exactly like a test measuring the right thing. That trap
//! does not get better for being duplicated.
//!
//! # What is asserted here
//!
//! | Test | Claim |
//! |---|---|
//! | [`the_band_is_the_same_height_on_every_tab`] | **R128.** A one-row tab and a full-height tab produce one height. |
//! | [`the_band_keeps_its_height_at_widths_where_every_group_overflows`] | …including where the band draws no group at all. |
//! | [`a_group_wider_than_the_cap_is_drawn_on_two_rows`] | The renderer obeys [`super::plan::wrap_group`], and the group gets narrower for it. |
//! | [`every_caption_in_a_band_shares_one_baseline`] | The mockup's `justify-content: space-between`. |
//! | [`a_group_is_inset_by_the_padding_the_plan_budgets_for_it`] | The mockup's `.group { padding: 0 13px }` — **drawn**, not merely reserved. |
//! | [`the_band_leaves_clear_space_beneath_its_captions`] | The mockup's `.band { padding: … 4px }`. |
//!
//! # ★ Why the last two are here at all
//!
//! Both pin a defect of the same *shape*, and it is a shape that no test in
//! this crate could previously see: **the plan reserved space the renderer
//! never drew.** [`super::plan::GROUP_PADDING`] budgeted 12 pt per group from
//! the day the planner was written; nothing drew a point of it, every test
//! passed, and the only evidence was a screenshot in which a group's box and
//! its first control began at the same x. `MODES_AND_PANELS.md` is blunt about
//! that class — *layout/clipping defects have exactly one oracle, a rendered
//! screenshot* — and the response to a defect an oracle found is a test that
//! would have found it too, not a note saying the oracle exists.
//!
//! So both assert against **drawn rectangles**: the inset is measured as the
//! distance from a group's published box to the leftmost control published
//! inside it, and the bottom padding as the distance from the ribbon's bottom
//! edge to the lowest caption in it. Delete either `add_space` and the numbers
//! go to zero.

use egui::Rect;

use crate::commands::{Command, CommandRegistry, ConditionSet, HandlerToken};
use crate::manifest::{Group, Item, Mode, Shell, Tab};

use super::report;
use super::tests::registry;
use super::width_tests::{SLACK, render_shell_with};

/// [`super::width_tests`]' context, **with a [`crate::theme::Theme`]
/// applied**.
///
/// # ★ Why the theme has to be applied here and is not next door
///
/// `Theme::apply` writes `spacing.interact_size.y = control_height`, which
/// is what makes a band control exactly as tall as the metric
/// [`super::rhythm::rows_height`] budgets for it. Without it `egui`'s default
/// `interact_size.y` is 18 pt against a 24 pt `control_height`, and **every
/// row carries 6 pt of slack that a spacing error can hide in**.
///
/// One did. A first cut of the two-row band padded its rows against
/// `GROUP_ROWS × height + (GROUP_ROWS − 1) × spacing` — correct for the ink
/// and one gap short for the cursor, which `egui` advances past *every* rect
/// including the last. Under the un-themed context the slack absorbed it and
/// these tests passed; the running binary's own trace showed a two-row group
/// at 68 pt beside a one-row group at 64.
///
/// `HANDOFF.md` §10: *a fixture can flatter the thing it measures, and the
/// numbers will look fine.* The theme is applied so this fixture cannot.
/// [`the_fixture_is_themed_like_the_running_application`] asserts it took.
fn context() -> egui::Context {
    let ctx = super::width_tests::context();
    crate::theme::Theme::default().apply(&ctx);
    ctx
}

/// **★ The guard on every measurement below: this context is spaced the way
/// the application is.**
///
/// Asserted rather than assumed, because the failure it prevents is silent —
/// see [`context`]. A slack row makes a band-height test pass against a band
/// whose groups are different heights.
#[test]
fn the_fixture_is_themed_like_the_running_application() {
    let ctx = context();
    let theme = crate::theme::Theme::of(&ctx);
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::Vec2::new(900.0, 400.0),
        )),
        ..Default::default()
    };
    let mut seen = None;
    let _ = ctx.run_ui(input, |ui| {
        seen = Some((ui.spacing().interact_size.y, ui.spacing().item_spacing.y));
    });
    let (interact, spacing) = seen.expect(
        "the closure never ran, so nothing was measured and the assertions below would be about a context nobody looked at",
    );
    assert!(
        (interact - theme.metrics.control_height).abs() <= f32::EPSILON,
        "a band control lays out {interact} pt tall against a {} pt metric — this context has slack the running application does not, and a height test under it can pass against a ragged band",
        theme.metrics.control_height
    );
    assert!(
        (spacing - theme.metrics.gutter).abs() <= f32::EPSILON,
        "the layout gap is {spacing} pt against the theme's {} pt gutter",
        theme.metrics.gutter
    );
}

/// A registry of eight controls with labels long enough to make a group
/// wider than [`super::plan::GROUP_WRAP_WIDTH`] under the synthetic face.
///
/// Deliberately **unequal** label lengths, for the reason [`super::width_tests`]'s `strip_shell`'s
/// labels are unequal: an even split of eight identical widths is
/// arithmetic anyone could get right by accident, and it would not exercise
/// [`super::plan::wrap_group`]'s search over contiguous runs at all.
fn wide_registry() -> CommandRegistry {
    let mut r = registry();
    for (i, label) in [
        "Rectangle",
        "Ellipse",
        "Arrow line",
        "Polyline run",
        "Polygon run",
        "Freehand ink",
        "Finish shape",
        "Clear the page",
    ]
    .into_iter()
    .enumerate()
    {
        r.register(
            Command::new(
                format!("wide.c{i}"),
                label,
                HandlerToken::new(100 + i as u64),
            )
            .with_icon("open"),
        )
        .expect("distinct ids");
    }
    r
}

/// Two tabs that differ in exactly the way R128 cares about: one whose
/// group fits on a single row, and one whose group cannot.
///
/// The `narrow` tab holds two controls; the `wide` tab holds eight. Under
/// the one-row band these two tabs were **different heights**, and the
/// difference moved the canvas underneath on every tab click.
fn two_row_shell() -> Shell {
    let wide: Vec<Item> = (0..8)
        .map(|i| Item::command(format!("wide.c{i}")))
        .collect();
    Shell::new()
        .with_mode(Mode::new("edit", "Edit", ["narrow", "wide"]))
        .with_tab(
            Tab::new("narrow", "Narrow").with_groups([Group::new("small", "Small")
                .with_items([Item::command("view.single"), Item::command("view.facing")])]),
        )
        .with_tab(
            Tab::new("wide", "Wide").with_groups([Group::new("shapes", "Shapes").with_items(wide)]),
        )
}

/// Group rectangles by their top edge, and report the distinct rows.
///
/// A row is a set of controls whose tops agree to within [`SLACK`]; `egui`
/// rounds widget rects to whole physical pixels, so an exact equality would
/// make this brittle for a reason that has nothing to do with layout.
fn row_tops(rects: &[Rect]) -> Vec<f32> {
    let mut tops: Vec<f32> = Vec::new();
    for rect in rects {
        if !tops.iter().any(|t| (t - rect.top()).abs() <= SLACK) {
            tops.push(rect.top());
        }
    }
    tops.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    tops
}

/// **★ R128: two different tabs produce the same band height.**
///
/// `PROJECT_PLAN.md`'s rule and this project's most-repeated bug — a
/// content-driven height beside a fit-to-viewport zoom is a feedback loop,
/// measured at 230 % → 224 % → 215 % of drift. The ribbon sits directly
/// above the canvas, so a band that is one row tall on one tab and two on
/// another re-fits the page every time the operator changes tab, and the
/// zoom walks.
///
/// The fixture is built so the *content* really does differ: `narrow` holds
/// one two-control group and `wide` holds one eight-control group that
/// [`super::plan::wrap_group`] splits over two rows. A band sized to its
/// content would report two different numbers here; a band sized from the
/// theme reports one.
///
/// # Why the vacuity guards are as long as the assertion
///
/// `HANDOFF.md` §10: **`cargo test -p egui-shell` and `cargo test
/// --workspace` compile with different `egui` features**, and a layout test
/// can be entirely vacuous under one of them. Two things could go quiet
/// here — a `None` ribbon height (the measuring closure never ran) and a
/// band that never wrapped (both tabs one row, so "the same height" is true
/// and says nothing). Both are asserted as facts before the equality is
/// asserted at all.
#[test]
fn the_band_is_the_same_height_on_every_tab() {
    let ctx = context();
    let shell = two_row_shell();
    let registry = wide_registry();
    let conditions = ConditionSet::new();
    let width = 900.0;

    let narrow = render_shell_with(&ctx, &shell, &registry, "narrow", &conditions, width);
    let wide = render_shell_with(&ctx, &shell, &registry, "wide", &conditions, width);

    // --- the measurement happened -------------------------------------
    let narrow_band = narrow.band_height("narrow").expect(
        "the `narrow` tab published no group rect, so nothing was measured and the \
         equality below would hold vacuously",
    );
    let wide_band = wide.band_height("wide").expect(
        "the `wide` tab published no group rect, so nothing was measured and the \
         equality below would hold vacuously",
    );
    let narrow_total = narrow
        .ribbon_height
        .expect("the render closure never ran, so no ribbon height exists to compare");
    let wide_total = wide
        .ribbon_height
        .expect("the render closure never ran, so no ribbon height exists to compare");

    // --- the fixture really does differ in content --------------------
    assert_eq!(
        narrow.all("ribbon.item.").len(),
        2,
        "the narrow tab must draw its two controls"
    );
    assert_eq!(
        wide.all("ribbon.item.").len(),
        8,
        "the wide tab must draw all eight controls"
    );
    assert_eq!(
        row_tops(&wide.all("ribbon.item.")).len(),
        super::plan::GROUP_ROWS,
        "the wide tab's group was NOT wrapped onto its full row count, so this test is \
         comparing two one-row bands and would pass against the very layout it \
         exists to refuse"
    );
    assert_eq!(
        row_tops(&narrow.all("ribbon.item.")).len(),
        1,
        "the narrow tab's group must stay on one row, or there is no difference in \
         content for the fixed height to absorb"
    );

    // --- the claim ----------------------------------------------------
    assert!(
        (narrow_band - wide_band).abs() <= SLACK,
        "R128: a one-row tab's band is {narrow_band} pt and a two-row tab's is \
         {wide_band} pt. A band whose height follows its content moves the canvas \
         on every tab click, and a fit-to-viewport zoom chases it — measured at \
         230 % -> 224 % -> 215 % when a status line did the same thing"
    );
    assert!(
        (narrow_total - wide_total).abs() <= SLACK,
        "R128: the whole ribbon is {narrow_total} pt tall on `narrow` and \
         {wide_total} pt on `wide`. That is the number the canvas below sees, \
         rather than any rect the ribbon publishes"
    );

    // …and it is genuinely two rows tall, not one row that happens to match.
    let one_row = crate::theme::Theme::of(&ctx).metrics.control_height;
    assert!(
        wide_band > one_row * 1.5,
        "the band measured {wide_band} pt against a {one_row} pt control — that is \
         a one-row band, so the two-row reservation never took effect"
    );
}

/// **★ R128 holds at every width, including the ones where a tab shows no
/// group at all.**
///
/// The case a height derived from drawn content gets silently wrong: below
/// the width at which one group fits beside the overflow reservation, the
/// band draws *nothing but the affordance*. A band measured from what it
/// drew would then be one control tall on that tab and two rows tall on the
/// next one — R128 arriving through the overflow machinery rather than
/// through the manifest.
///
/// Swept rather than spot-checked, because "at *some* width the answer is
/// wrong" is the shape of every defect this file exists for.
#[test]
fn the_band_keeps_its_height_at_widths_where_every_group_overflows() {
    let ctx = context();
    let shell = two_row_shell();
    let registry = wide_registry();
    let conditions = ConditionSet::new();

    let mut saw_an_empty_band = false;
    for width in (60..1000).step_by(19).map(|w| w as f32) {
        let narrow = render_shell_with(&ctx, &shell, &registry, "narrow", &conditions, width);
        let wide = render_shell_with(&ctx, &shell, &registry, "wide", &conditions, width);

        let a = narrow
            .ribbon_height
            .unwrap_or_else(|| panic!("the render closure never ran at {width} pt"));
        let b = wide
            .ribbon_height
            .unwrap_or_else(|| panic!("the render closure never ran at {width} pt"));
        assert!(
            (a - b).abs() <= SLACK,
            "at {width} pt the ribbon is {a} pt tall on `narrow` and {b} pt on \
             `wide`. Every tab click at that window size would move the canvas"
        );

        saw_an_empty_band |= wide.state.last_frame().groups_in_band == 0;
    }
    assert!(
        saw_an_empty_band,
        "the sweep never reached a width at which the band drew no group, so it \
         never exercised the case a content-derived height gets wrong"
    );
}

/// **★ A group wider than the cap really is drawn on two rows, and costs
/// the band less for it.**
///
/// [`super::plan`]'s own tests prove the arithmetic; this proves the
/// renderer obeys it, which is a different claim and the one that could not
/// hold before this change — `captioned_group` emitted a single
/// `ui.horizontal`, so no manifest could ever produce a second row however
/// wide it got.
///
/// The width claim is stated against the **items**, not against a constant:
/// the group's rect must be narrower than laying its eight controls end to
/// end would be. That is checkable without knowing a single measurement,
/// and it is the property that lets a fifth group fit where three used to be
/// pushed behind the affordance.
#[test]
fn a_group_wider_than_the_cap_is_drawn_on_two_rows() {
    let ctx = context();
    let shell = two_row_shell();
    let registry = wide_registry();
    let frame = render_shell_with(
        &ctx,
        &shell,
        &registry,
        "wide",
        &ConditionSet::new(),
        1200.0,
    );

    let items = frame.all("ribbon.item.");
    assert_eq!(items.len(), 8, "every control must be drawn, on some row");

    let tops = row_tops(&items);
    assert_eq!(
        tops.len(),
        super::plan::GROUP_ROWS,
        "eight controls that do not fit the cap were laid out on {} row(s): {tops:?}",
        tops.len()
    );

    let group = frame
        .rect(&report::group("wide", "shapes"))
        .expect("the band publishes its groups");
    let end_to_end: f32 = items.iter().map(|r| r.width()).sum();
    assert!(
        group.width() < end_to_end,
        "the wrapped group is {} pt wide and its eight controls total {end_to_end} \
         pt — wrapping bought the band nothing, which means the plan wrapped and the \
         renderer did not",
        group.width()
    );
    assert!(
        group.width() <= super::plan::GROUP_WRAP_WIDTH + SLACK,
        "the group is {} pt wide, over the {} pt cap it tripped — an even split must \
         come in under the width that triggered it",
        group.width(),
        super::plan::GROUP_WRAP_WIDTH
    );

    // Every control is inside its own group, on whichever row it landed.
    for item in &items {
        assert!(
            group.contains_rect(*item),
            "a control at {item:?} was drawn outside its group at {group:?}"
        );
    }
}

/// **★ Every caption in a band sits on one baseline**, whether the group
/// above it used one row or two.
///
/// The mockup's `justify-content: space-between`, and the half of "staged
/// nicer" that has nothing to do with wrapping: with each group as tall as
/// its own content, a two-row group's caption hangs a whole control-row
/// below its neighbours' and the band reads as ragged. Pinning the captions
/// is what makes the row of group names scan as a row.
#[test]
fn every_caption_in_a_band_shares_one_baseline() {
    let ctx = context();
    let wide: Vec<Item> = (0..8)
        .map(|i| Item::command(format!("wide.c{i}")))
        .collect();
    let shell = Shell::new()
        .with_mode(Mode::new("edit", "Edit", ["mixed"]))
        .with_tab(Tab::new("mixed", "Mixed").with_groups([
            Group::new("shapes", "Shapes").with_items(wide),
            Group::new("small", "Small").with_items([Item::command("view.single")]),
        ]));
    let frame = render_shell_with(
        &ctx,
        &shell,
        &wide_registry(),
        "mixed",
        &ConditionSet::new(),
        1400.0,
    );

    // The fixture is only meaningful if the two groups really differ.
    assert_eq!(
        row_tops(&frame.all("ribbon.item.")).len(),
        super::plan::GROUP_ROWS,
        "the mixed band must contain a FULL-height group beside a one-row one, or the \
         baseline claim below is made about two identical groups. Written against \
         GROUP_ROWS rather than a literal, because the eight-item fixture wraps to \
         whatever the shipped ceiling is"
    );

    let captions: Vec<(&str, Rect)> = frame
        .rects
        .iter()
        .filter(|(n, _)| n.ends_with(".caption"))
        .map(|(n, r)| (n.as_str(), *r))
        .collect();
    assert_eq!(captions.len(), 2, "both groups must publish a caption");

    let (first_name, first) = captions[0];
    for (name, rect) in &captions[1..] {
        assert!(
            (rect.top() - first.top()).abs() <= SLACK,
            "`{name}` captions at y={} and `{first_name}` at y={} — a two-row group \
             pushed its caption below its neighbour's, which is the ragged band the \
             fixed height exists to prevent",
            rect.top(),
            first.top()
        );
    }
}

/// A band of two groups whose controls are the wider half of each of them.
///
/// Both fixture properties are load-bearing for
/// [`a_group_is_inset_by_the_padding_the_plan_budgets_for_it`]:
///
/// - **Two groups**, so the *gap between* them can be measured as well as the
///   inset within each. The inset is invisible in a one-group band — a group
///   at the band's left edge with padding looks exactly like a group without
///   it drawn 6 pt further right.
/// - **Controls wider than captions.** [`super::band::captioned_group`]
///   centres the caption on the widest control row, so in a group whose
///   caption is the wider half the caption overhangs the rows on both sides
///   and the group's box is caption-driven rather than row-driven. That is
///   correct behaviour and it makes the inset unmeasurable from the control
///   rects, which is the only thing published. Eight- and four-control groups
///   under the synthetic face are comfortably row-driven.
fn two_padded_groups() -> Shell {
    let shapes: Vec<Item> = (0..8)
        .map(|i| Item::command(format!("wide.c{i}")))
        .collect();
    let notes: Vec<Item> = (0..4)
        .map(|i| Item::command(format!("wide.c{i}")))
        .collect();
    Shell::new()
        .with_mode(Mode::new("edit", "Edit", ["padded"]))
        .with_tab(Tab::new("padded", "Padded").with_groups([
            Group::new("shapes", "Shapes").with_items(shapes),
            Group::new("notes", "Notes").with_items(notes),
        ]))
}

/// **★ A group's controls are inset from its own box by exactly the padding
/// the planner budgets — the mockup's `.group { padding: 0 13px }`.**
///
/// # The defect, measured
///
/// [`super::plan::GROUP_PADDING`] has added 2 × 6 pt to every group's planned
/// width since the planner was written, and until 2026-08-14 the renderer drew
/// none of it: `captioned_group` laid the group out as a bare `ui.vertical`
/// with no horizontal inset. Measured in the running application at 1,100 pt
/// on the Markup tab, the Text-markup group box began at **x = 322.5 and its
/// first control began at x = 322.5** — a zero-point inset. Controls sat flush
/// against the group boundary and against the rule dividing them from the next
/// group, which is most of what the operator meant by *"cluttered"*.
///
/// Nothing failed. The plan was right, the width fitted, every test passed,
/// and the space was spent as an accidental margin on the outside of the box
/// instead of as padding on the inside.
///
/// # What is asserted, and why the second assertion is the interesting one
///
/// 1. **The inset**, on both sides of both groups, exactly
///    [`super::plan::GROUP_PADDING`]. This is what fails if either
///    `add_space` is deleted.
/// 2. **The gap between the two groups** — group edge to group edge — is
///    `2 × GROUP_PADDING + `[`super::measure::separator_width`]. That is the
///    number the mockup actually specifies: its `.group` padding is 13 px each
///    side and its divider is a zero-width `border-right`, giving 26 px, and
///    this build reaches the same 26 pt as 6 + 14 + 6 because its divider is a
///    real `ui.separator()` with real width. Asserting the *sum* is what makes
///    this test a statement about the mockup rather than about a constant it
///    could have copied. See [`super::plan::GROUP_PADDING`] for the full
///    reconciliation.
#[test]
fn a_group_is_inset_by_the_padding_the_plan_budgets_for_it() {
    let ctx = context();
    let frame = render_shell_with(
        &ctx,
        &two_padded_groups(),
        &wide_registry(),
        "padded",
        &ConditionSet::new(),
        1400.0,
    );

    // --- the fixture drew what it claims to have drawn ------------------
    let boxes: Vec<(String, Rect)> = frame
        .rects
        .iter()
        .filter(|(n, _)| n.starts_with("ribbon.group.padded.") && !n.ends_with(".caption"))
        .map(|(n, r)| (n.clone(), *r))
        .collect();
    assert_eq!(
        boxes.len(),
        2,
        "both groups must be in the band, not behind the overflow affordance, or \
         there is nothing to measure an inset against: {boxes:?}"
    );
    let items = frame.all("ribbon.item.");
    assert!(
        !items.is_empty(),
        "no control published a rect, so every assertion below would range over an \
         empty set and hold vacuously"
    );

    let pad = super::plan::GROUP_PADDING;
    for (name, box_) in &boxes {
        // Attributed by containment rather than by id: a group's box contains
        // its own controls and no others, and that is true without this test
        // knowing which fixture command landed in which group.
        let inside: Vec<Rect> = items
            .iter()
            .copied()
            .filter(|i| box_.contains_rect(*i))
            .collect();
        assert!(
            !inside.is_empty(),
            "`{name}` published a box at {box_:?} with no control inside it"
        );
        let left = inside.iter().fold(f32::INFINITY, |a, r| a.min(r.left()));
        let right = inside
            .iter()
            .fold(f32::NEG_INFINITY, |a, r| a.max(r.right()));

        assert!(
            (left - box_.left() - pad).abs() <= SLACK,
            "`{name}`: the group box starts at x={} and its first control at x={left} \
             — an inset of {}, against the {pad} pt `plan::GROUP_PADDING` reserves for \
             it. Zero is the defect this test exists for: the controls sit flush \
             against the group boundary and against the rule dividing them from the \
             next group, while the planner charges the band for space nobody drew",
            box_.left(),
            left - box_.left()
        );
        assert!(
            (box_.right() - right - pad).abs() <= SLACK,
            "`{name}`: the group box ends at x={} and its last control at x={right} — \
             a trailing inset of {}, against {pad} pt. An asymmetric group is what \
             forgetting `item_spacing.x = 0` in the wrapper produces: `egui` advances \
             the cursor past the body by `item_spacing` and `add_space` adds to that",
            box_.right(),
            box_.right() - right
        );
    }

    // --- and the gap the mockup actually specifies ----------------------
    let mut ordered = boxes.clone();
    ordered.sort_by(|a, b| {
        a.1.left()
            .partial_cmp(&b.1.left())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let gap = ordered[1].1.left() - ordered[0].1.right();
    let separator = separator_width(&ctx);
    assert!(
        (gap - separator).abs() <= SLACK,
        "the two group boxes are {gap} pt apart against a {separator} pt separator — \
         the boxes must abut the rule between them, because the padding is now INSIDE \
         them"
    );
    let edge_to_edge = ordered[1].1.left() + pad - (ordered[0].1.right() - pad);
    assert!(
        (edge_to_edge - (separator + 2.0 * pad)).abs() <= SLACK,
        "control to control across the divider is {edge_to_edge} pt. The mockup \
         specifies 13 px of group padding either side of a zero-width \
         `border-right`, i.e. 26 px; this build reaches the same figure as \
         {pad} + {separator} + {pad} because its divider is a real `ui.separator()` \
         with real width. If those stop summing to the same number, one of the two \
         was changed without the other being reconsidered"
    );
}

/// The inter-group separator's cost, measured from a real `Ui` rather than
/// re-derived from constants.
///
/// [`super::measure::separator_width`] takes a `&Ui` because it reads
/// `item_spacing` from the live style, and a test that spelled the sum out
/// again would pass while disagreeing with the renderer — which is the exact
/// failure `super::plan`'s header warns about for the *group* width.
fn separator_width(ctx: &egui::Context) -> f32 {
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::Vec2::new(900.0, 400.0),
        )),
        ..Default::default()
    };
    let mut seen = None;
    let _ = ctx.run_ui(input, |ui| seen = Some(super::measure::separator_width(ui)));
    seen.expect("the closure never ran, so no separator width was measured")
}

/// **★ The band stops a clear [`super::band::BAND_PADDING_BOTTOM`] below its
/// lowest caption — the mockup's `.band { padding: 8px 10px 4px }`.**
///
/// # The defect, measured
///
/// Before 2026-08-14 the band's reserved height was exactly the rows, the gap
/// and one line of caption, so the ribbon ended on the caption's own baseline.
/// In the running application at 1,100 pt the captions ended at y = 103 and
/// the dock's tab bar began at y = 105.3: a 10 pt line of `weak()`, `small()`
/// text separated from the panel header beneath it by less than its own
/// leading. The caption is the one piece of text that says what a block of
/// controls is *for*, and a caption sitting on a seam reads as a label for
/// whatever is on the other side of it.
///
/// # Why this is asserted against the ribbon's bottom edge
///
/// Because that is the edge the operator sees and the edge the application
/// puts a panel against. It is deliberately **not** asserted as
/// `band_height() == rows + gap + caption + padding`, which would be the
/// derivation restated — true by construction, and equally true of a build in
/// which the reservation is made and then not honoured. The reservation is
/// made by `ui.set_min_height`, whose whole job is to be larger than the
/// content; a test that never measures the content cannot tell whether it took
/// effect.
///
/// R128 is *not* re-asserted here — [`the_band_is_the_same_height_on_every_tab`]
/// and [`the_band_keeps_its_height_at_widths_where_every_group_overflows`] own
/// that claim, and both still hold with the padding folded into the
/// derivation, which is the point of folding it in rather than emitting it
/// after the last group.
#[test]
fn the_band_leaves_clear_space_beneath_its_captions() {
    let ctx = context();
    let frame = render_shell_with(
        &ctx,
        &two_padded_groups(),
        &wide_registry(),
        "padded",
        &ConditionSet::new(),
        1400.0,
    );

    // The vacuity guard. The assertion below is stated against the constant
    // rather than against a literal, so that changing the padding does not
    // require editing this test — but a constant of zero would make it a
    // comparison of a number against itself minus SLACK, which every band
    // passes. Zero is exactly the state before this change, so the test would
    // go quiet in the one case it exists to catch.
    //
    // A `const` block, so it is a compile error rather than a test failure:
    // clippy refuses a runtime assertion on a constant, and it is right to —
    // the fact is known without running anything, and a build in which the
    // guard could not hold should not produce a test binary at all.
    const {
        assert!(
            super::band::BAND_PADDING_BOTTOM > 0.0,
            "the band reserves no space beneath its captions, so the clearance \
             assertion in `the_band_leaves_clear_space_beneath_its_captions` holds \
             for any band at all"
        );
    }

    let ribbon = frame.ribbon_rect.expect(
        "the render closure never ran, so there is no ribbon rectangle and the \
         comparison below would have nothing to compare",
    );
    let captions: Vec<(&str, Rect)> = frame
        .rects
        .iter()
        .filter(|(n, _)| n.starts_with("ribbon.group.padded.") && n.ends_with(".caption"))
        .map(|(n, r)| (n.as_str(), *r))
        .collect();
    assert_eq!(
        captions.len(),
        2,
        "both groups must caption in the band, or the clearance below is measured \
         against a caption that is not the lowest one"
    );

    let (name, lowest) =
        captions.iter().copied().fold(
            captions[0],
            |a, b| if b.1.bottom() > a.1.bottom() { b } else { a },
        );
    let clearance = ribbon.bottom() - lowest.bottom();
    assert!(
        clearance >= super::band::BAND_PADDING_BOTTOM - SLACK,
        "`{name}` ends at y={} and the ribbon ends at y={} — {clearance} pt of \
         clearance against the {} pt the band reserves. Zero is the defect: the \
         caption sits on the seam with whatever the application docks beneath the \
         ribbon",
        lowest.bottom(),
        ribbon.bottom(),
        super::band::BAND_PADDING_BOTTOM
    );
}

// ===========================================================================
// ★★★ THE MOCKUP'S VERTICAL RHYTHM — 2026-09-04
//
// Three tests added when `mockups/pdfcer-shell.html` was adopted as the
// band's specification rather than as a sketch of it. The operator's words
// were *"I want everything to look exactly like that including sizing"*, and
// his fourth complaint was the one these are about: *"the mock's band is
// visibly taller with more generous rows and the group caption sitting
// lower."*
//
// ★ None of them asserts a total height against a literal, and that is
// deliberate. A literal would pin the number and say nothing about where it
// comes from, so the next font change would fail a test that reads as
// arbitrary. Each of these pins a **relationship** the mockup states:
// clearance above the rows, room for the rows themselves, and a caption
// drawn at the size the height was predicted from.
// ===========================================================================

/// ★★★ **Every preset reserves enough row area for its own two rows.**
///
/// The one invariant that makes [`crate::theme::Metrics::ribbon_rows`] safe as
/// a stated number rather than a derived one, and the reason the ribbon
/// rhythm is a *metric* at all rather than a constant in `band`.
///
/// The band's row area is now a **budget** — the mockup's 68 px, into which
/// [`super::plan::GROUP_ROWS`] rows are laid, with the caption hanging off the
/// bottom of it (`.grp .cap { margin-top: auto }`). A budget smaller than what
/// two rows cost does not fail loudly: the second row simply draws over the
/// caption, in one preset, which is the class of defect
/// `MODES_AND_PANELS.md` says has exactly one oracle and this project would
/// rather not need it for.
///
/// `Airy` is the preset that makes this real, and it is why transcribing the
/// mockup's `68` into all three would have been wrong: its `control_height` is
/// 28 pt and its `gutter` 8, so two of its rows cost 72 — **more than the
/// mockup's whole area.**
///
/// ★ Asserted over `Preset::ALL` rather than over the three by name, so a
/// preset added later cannot ship unmeasured. That is the same discipline
/// `Preset::ALL`'s own doc comment asks for.
#[test]
fn every_preset_reserves_room_for_its_own_rows() {
    #[allow(clippy::cast_precision_loss)] // single digits
    let rows = super::plan::GROUP_ROWS as f32;
    for &preset in crate::theme::Preset::ALL {
        let m = crate::theme::Theme::new(preset).metrics;

        // The height the band actually draws a small control at. The same
        // arithmetic `band::band_row_height` performs, restated in the test's
        // own terms rather than by calling it — an assertion that invokes the
        // implementation it is checking agrees with itself by construction.
        let control = m.ribbon_rows / rows - super::rhythm::BAND_ROW_SPACING;

        // (a) The rows fit the area they are laid into, counting the gap
        //     between them AND the one `egui` adds after the last. Arithmetic
        //     on the line above, so it cannot fail while (b) holds — and that
        //     is the point: it is the equation the preset table has to satisfy,
        //     written where a reader looking for it will find it.
        let occupied = rows.mul_add(control, rows * super::rhythm::BAND_ROW_SPACING);
        assert!(
            occupied <= m.ribbon_rows + 0.01,
            "preset {preset:?} budgets {} pt of band row area and its {rows} rows \
             occupy {occupied} pt, so the last row would be drawn over the group \
             caption",
            m.ribbon_rows
        );

        // (b) ★★★ THE HALF THAT CAN ACTUALLY FAIL, and the reason this test
        //     survived the 2026-09-05 row-count change rather than going with
        //     the invariant it used to state.
        //
        //     `band_row_height` floors the control at `icon_pts`, because a
        //     control that cannot show its icon is not a smaller control, it is
        //     a clipped one. The floor is a **safety, not a target**: the moment
        //     it binds, (a) stops holding, the band grows on every tab, and the
        //     canvas beneath it moves — R128, arriving through the theme table.
        //     A preset whose `ribbon_rows` was tightened or whose `icon_pts` was
        //     raised would trip this and **nothing else in the crate would
        //     notice**, because a taller band is not an error anywhere.
        //
        //     Margins as shipped: `Quiet` 68/3 − 1 = 21.67 against a 16 pt icon;
        //     `Airy` 84/3 − 1 = 27 against 17.
        assert!(
            control >= m.icon_pts,
            "preset {preset:?} draws a band row {control} pt tall and its icons are \
             {} pt, so `band_row_height`'s floor would bind and the band would grow \
             past its own {} pt budget",
            m.icon_pts,
            m.ribbon_rows
        );
    }
}

/// ★★ **…and the re-wrap rung survives the budget**, which is the half that
/// could have been broken silently.
///
/// `RIBBON_SCALING.md`'s ladder is re-wrap → collapse → scroll, and rung one
/// divides the *same* row area into [`super::plan::MAX_GROUP_ROWS`] instead of
/// [`super::plan::GROUP_ROWS`]. It has a self-disabling guard —
/// `band::rewrap_is_legible` — that turns the rung off rather than clipping
/// icons when the arithmetic does not clear, and **a rung that has switched
/// itself off looks exactly like a rung that was never needed.** So a change
/// to the row area could disable the first rung of the ladder in one preset
/// and no other test in this crate would say a word.
///
/// The margin is stated, not just the sign: with `Quiet` the compressed row is
/// `68/3 − 2 = 20.67` pt against a 16 pt icon. Before the mockup pass it was
/// `56/3 − 2 = 16.67` against the same 16 — a margin of two thirds of a point,
/// i.e. the rung was one theme tweak from vanishing.
#[test]
fn every_preset_can_still_re_wrap_a_group() {
    #[allow(clippy::cast_precision_loss)] // single digits
    let n = super::plan::MAX_GROUP_ROWS as f32;
    for &preset in crate::theme::Preset::ALL {
        let m = crate::theme::Theme::new(preset).metrics;
        // The same expression `band::compressed_control_height` evaluates,
        // written out rather than called because that function needs a `Ui`
        // and this claim is about the numbers alone.
        let compressed = m.ribbon_rows / n - 2.0;
        assert!(
            compressed >= m.icon_pts,
            "preset {preset:?} compresses a re-wrapped row to {compressed} pt, which \
             cannot show its own {} pt icon — so `rewrap_is_legible` returns false, \
             the collapse ladder loses its first rung in this preset, and every \
             group jumps straight from natural to collapsed",
            m.icon_pts
        );
    }
}

/// ★ **The band draws clear space above its first control** — the mockup's
/// `.ribbon { padding: 6px 8px 0 }`, the first figure.
///
/// The exact mirror of [`the_band_leaves_clear_space_beneath_its_captions`],
/// and it exists for the same reason: a padding that is *budgeted* and not
/// *drawn* is invisible to every test that measures a total height, because
/// the total is right and the ink is in the wrong place. That is the shipped
/// defect `super::plan::GROUP_PADDING` produced horizontally, and this is the
/// vertical version of the tripwire.
///
/// Measured from the **group's own published rectangle** rather than from the
/// ribbon's, because the ribbon rect includes the tab strip above the band and
/// would answer a different question. A group's rect starts at the band's top
/// edge; its first control starts `ribbon_pad_top` below that, and nowhere
/// else in `group_body` is any space emitted before the rows.
#[test]
fn the_band_draws_clear_space_above_its_first_control() {
    let ctx = context();
    let pad = crate::theme::Theme::of(&ctx).metrics.ribbon_pad_top;
    // The vacuity guard, exactly as the bottom-clearance test carries one: a
    // zero padding makes the assertion below `x >= -SLACK`, which every band
    // in every state satisfies.
    assert!(
        pad > 0.0,
        "this preset reserves no space above the band's first row, so the \
         clearance assertion below holds for any band at all"
    );

    let frame = render_shell_with(
        &ctx,
        &two_padded_groups(),
        &wide_registry(),
        "padded",
        &ConditionSet::new(),
        1400.0,
    );

    let group = frame
        .rect("ribbon.group.padded.shapes")
        .expect("the Shapes group must be drawn in the band, or there is nothing to measure from");
    let items = frame.all("ribbon.item.");
    assert!(
        !items.is_empty(),
        "the band published no control rectangles, so the clearance below would be \
         measured against nothing"
    );
    let highest = items
        .iter()
        .filter(|r| group.contains_rect(**r))
        .fold(f32::INFINITY, |a, r| a.min(r.top()));
    assert!(
        highest.is_finite(),
        "no control was published inside `ribbon.group.padded.shapes` — the group \
         drew nothing, and a clearance over an empty group is not the claim"
    );

    let clearance = highest - group.top();
    assert!(
        clearance >= pad - SLACK,
        "the Shapes group begins at y={} and its first control at y={highest} — \
         {clearance} pt of clearance against the {pad} pt the theme reserves. Zero \
         is the defect: the band's first row sits on the seam with the tab strip \
         above it",
        group.top()
    );
}

// =====================================================================
// AUTO-HIDE — 2026-09-05. The operator: *"we should also add the capability
// to auto hide the ribbon until we hover over top of it."*
//
// ★★★ These belong in this file rather than beside `peek`'s own tests, and
// the reason is the file's own subject. `peek` is arithmetic over rectangles
// and is tested exhaustively there; what CANNOT be asserted there is the one
// property the operator will actually feel, which is **vertical and belongs
// to the ribbon**: the number the canvas below sees must not change when the
// band comes and goes. That is R128 for a surface that appears and
// disappears, and R128 is what this whole file pins.
// =====================================================================

/// A one-tab shell with a group in it, for the auto-hide tests.
fn hideable_shell() -> Shell {
    Shell::new()
        .with_mode(Mode::new("edit", "Edit", ["only"]))
        .with_tab(
            Tab::new("only", "Only").with_groups([Group::new("g", "Group")
                .with_items([Item::command("wide.c0"), Item::command("wide.c1")])]),
        )
}

/// Render one auto-hidden ribbon, optionally with the pointer somewhere.
///
/// Two frames, like `render_shell_with`, and for one more reason besides its:
/// [`crate::peek::Peek`] answers from the pointer AND from last frame's state,
/// so the frame that reveals and the frame that draws the revealed band are
/// different frames.
fn render_auto_hidden(
    ctx: &egui::Context,
    pointer: Option<egui::Pos2>,
    width: f32,
) -> (Option<f32>, crate::peek::Show, Vec<(String, Rect)>) {
    let shell = hideable_shell();
    let registry = wide_registry();
    let mut state = crate::ribbon::RibbonState::new();
    state.set_active_tab("only");
    state.set_auto_hide(crate::peek::AutoHide::OnHover);

    let mut height = None;
    let mut rects: Vec<(String, Rect)> = Vec::new();
    let conditions = ConditionSet::new();
    for _ in 0..2 {
        rects.clear();
        height = None;
        let mut sink = |name: &str, rect: Rect| rects.push((name.to_owned(), rect));
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::Vec2::new(width, 400.0),
            )),
            events: pointer.map(egui::Event::PointerMoved).into_iter().collect(),
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            let _ = crate::ribbon::Ribbon::new()
                .with_conditions(&conditions)
                .reporting_rects_to(&mut sink)
                .render(ui, &shell, &registry, &mut state);
            // The number the canvas below the ribbon actually sees.
            height = Some(ui.min_rect().height());
        });
    }
    (height, state.last_frame().band_show, rects)
}

/// ★★★ **THE CANVAS DOES NOT MOVE.** An auto-hidden ribbon takes the same room
/// whether its band is showing or not.
///
/// This is the property that decides whether the setting is usable. The band is
/// drawn into an `egui::Area` when it is revealed, which allocates nothing, so
/// the `Ui` the application handed the ribbon is exactly as tall as the tab
/// strip in both states. The plausible wrong implementation draws the band
/// inline and simply skips it when hidden — every test in this file would still
/// pass, and the operator's drawing would jump by ninety points every time the
/// pointer crossed the tab row.
///
/// ⚠ Both states are DRIVEN rather than assumed. An absence test that never
/// reached the revealed state would be satisfied by a build that never reveals
/// anything, which is the vacuous shape this project has shipped twice; so the
/// second reading asserts `Show::Overlay` before comparing, and says so if the
/// plant did not land.
#[test]
fn an_auto_hidden_ribbon_takes_the_same_room_whether_its_band_shows_or_not() {
    let ctx = context();

    let (hidden_h, hidden_show, hidden_rects) = render_auto_hidden(&ctx, None, 900.0);
    assert_eq!(
        hidden_show,
        crate::peek::Show::Hidden,
        "with no pointer in the frame the band must be at rest"
    );
    let hidden_h = hidden_h.expect("the measuring closure never ran");

    // The trigger is the tab strip, and the pointer is put in the middle of it
    // — read out of the published region rather than guessed, because a guessed
    // coordinate that happens to miss produces exactly this test passing for
    // the wrong reason.
    let trigger = hidden_rects
        .iter()
        .rev()
        .find(|(n, _)| n == "ribbon.autohide.trigger")
        .map(|(_, r)| *r)
        .expect("a hidden ribbon must publish its trigger, or there is no way back");
    assert!(
        trigger.height() >= crate::peek::Peek::MIN_TRIGGER_PTS,
        "the way back to the band is {} pt tall, under the {} pt floor",
        trigger.height(),
        crate::peek::Peek::MIN_TRIGGER_PTS
    );

    let (shown_h, shown_show, shown_rects) =
        render_auto_hidden(&ctx, Some(trigger.center()), 900.0);
    assert_eq!(
        shown_show,
        crate::peek::Show::Overlay,
        "the plant did not land: the pointer is on the tab strip and the band is \
         still hidden, so the comparison below is between two identical frames"
    );
    let shown_h = shown_h.expect("the measuring closure never ran");

    assert!(
        (hidden_h - shown_h).abs() < 0.01,
        "the ribbon occupied {hidden_h} pt with its band hidden and {shown_h} pt \
         with it revealed, so the canvas beneath jumped by {} pt as the pointer \
         crossed the tab row",
        (hidden_h - shown_h).abs()
    );

    // …and the band really was drawn on the second reading. Without this the
    // equality above is satisfied by a build that reveals nothing at all.
    assert!(
        shown_rects
            .iter()
            .any(|(n, _)| n == "ribbon.autohide.overlay"),
        "no overlay rectangle was published, so nothing was drawn over the document"
    );
    assert!(
        !hidden_rects
            .iter()
            .any(|(n, _)| n == "ribbon.autohide.overlay"),
        "an overlay was published on the frame the band is supposed to be hidden"
    );
}

/// ★★ **The tab strip is the same height either way**, which is what makes it
/// a legal trigger.
///
/// `peek`'s direction bound rests on the trigger being independent of the
/// surface it reveals. Here that is a claim about *this* ribbon rather than
/// about `peek`: the strip is drawn by [`super::strip::render`] before the band
/// exists, so nothing the band does can reach it. Asserted across a width
/// series rather than at one width, because the strip's own overflow machinery
/// changes what it contains and a single width could agree by luck.
#[test]
fn the_tab_strip_is_the_same_height_with_the_band_shown_and_hidden() {
    let ctx = context();
    for width in [1400.0_f32, 1100.0, 900.0, 700.0, 520.0] {
        let (_, _, hidden) = render_auto_hidden(&ctx, None, width);
        let trigger = hidden
            .iter()
            .rev()
            .find(|(n, _)| n == "ribbon.autohide.trigger")
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("no trigger published at {width} pt"));
        let (_, show, shown) = render_auto_hidden(&ctx, Some(trigger.center()), width);
        assert_eq!(show, crate::peek::Show::Overlay, "at {width} pt");
        let revealed = shown
            .iter()
            .rev()
            .find(|(n, _)| n == "ribbon.autohide.trigger")
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("no trigger published at {width} pt while revealed"));
        assert!(
            (trigger.height() - revealed.height()).abs() < 0.01,
            "at {width} pt the tab strip is {} pt tall hidden and {} pt revealed — \
             the trigger moves with the thing it triggers, which is R128's loop",
            trigger.height(),
            revealed.height()
        );
    }
}
