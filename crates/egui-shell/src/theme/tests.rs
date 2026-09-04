//! Tests for the theme: the palette-level assertions, the rendered-pair
//! contrast gate's application to the shipped presets, and the two roles
//! `egui` drives from `visuals.selection`.
//!
//! # ★ Why these live in their own file
//!
//! Rule **R2**: no `.rs` file over 1500 lines. `theme/mod.rs` carries the
//! palette, the presets, the accessors and `write_style`, every one of which
//! is documented at this project's deliberately verbose register (rule R5) —
//! and the colour decisions in it each carry their arithmetic, which is the
//! part a future reader cannot reconstruct. Splitting the tests out is the
//! seam that costs the least: the crate already does it for
//! `dock::width_tests`, `menu::tests` and `ribbon::height_tests`.
//!
//! `use super::*` keeps every one of these assertions able to see private
//! items — `clamp_to_i8`, `Theme::quiet` and friends — exactly as it could
//! when the module was inline. Nothing was weakened to move it.

use super::*;

/// Crude relative luminance, matching [`contrast::luma`].
///
/// Duplicated as a one-line local so the palette-level tests below
/// read on their own; the rendered-pair gate uses the real one.
fn luma(c: Color32) -> f32 {
    contrast::luma(c)
}

/// **Text is legible on the surface it is drawn on, in every
/// preset.**
///
/// Salvaged verbatim in intent. A crude relative-luminance gap rather
/// than a full WCAG contrast ratio: the point is to catch a preset
/// where someone set a light text colour against a light panel — which
/// is what a `..quiet` spread does the moment a surface is darkened
/// and the text is not — and a coarse check that always fires beats a
/// precise one nobody runs.
///
/// **This test is kept even though the rendered-pair gate subsumes
/// most of it**, because it fails with a better message: it names the
/// palette role that is wrong, where the gate names the widget state
/// that renders wrong. Both are worth having when a preset is being
/// edited.
#[test]
fn text_contrasts_with_its_background_in_every_preset() {
    for preset in Preset::ALL {
        let p = Theme::new(*preset).palette;
        for (name, bg) in [("surface", p.surface), ("panel", p.panel)] {
            let gap = (luma(p.text) - luma(bg)).abs();
            assert!(
                gap > 90.0,
                "{preset:?}: `text` on `{name}` has a luminance gap of {gap:.0}, \
                 which is not readable"
            );
        }
        let muted = (luma(p.text_muted) - luma(p.surface)).abs();
        assert!(
            muted > 45.0,
            "{preset:?}: `text_muted` on `surface` is too faint (gap {muted:.0})"
        );
    }
}

/// **The label backdrop stays light in every preset, including the
/// dark one.**
///
/// Salvaged. Labels sit over CONTENT, not over chrome, and the content
/// is whatever colour the document says — overwhelmingly white. A dark
/// theme that darkened the label backdrop would put dark text on a
/// dark plate on a white page, which is unreadable in the one place it
/// matters most.
///
/// Worth a test because it is precisely the field a careless "make
/// everything dark" edit would flip.
///
/// **Note what this test does NOT do**, because it is half of why D2
/// shipped: it asserts `label_backdrop` is light *and stops there*. It
/// says nothing about what is behind `label_backdrop` when something
/// draws with it, and in the salvage source something did — the active
/// widget state's foreground. A test that pins a colour without
/// pinning its pairing is a test that will agree with the bug.
#[test]
fn label_plates_stay_content_facing_not_chrome_facing() {
    for preset in Preset::ALL {
        let p = Theme::new(*preset).palette;
        assert!(
            p.label_backdrop.r() > 200 && p.label_backdrop.b() > 200,
            "{preset:?}: the label backdrop follows the content, not the chrome"
        );
        assert!(
            p.label_text.r() < 80,
            "{preset:?}: label text must be dark, to sit on that backdrop"
        );
    }
}

/// **★ `DEFECTS.md` D2's regression test: every foreground `egui`
/// will actually paint is readable on the background it will actually
/// paint it on — for all five widget states, both fills, all three
/// presets.**
///
/// # Why the two tests above could not have caught D2, and this can
///
/// This is the important part of this test, and it generalises past
/// theming.
///
/// D2 was: `widgets.active.fg_stroke` was set to a near-white plate
/// colour while `widgets.active.bg_fill` was never assigned the
/// accent, so `CollapsingHeader` headers and dock tab labels rendered
/// near-white on light grey. Two tests sat directly adjacent to it:
///
/// - `text_contrasts_with_its_background_in_every_preset` checked
///   `text` against `surface` and `panel`. It never touched
///   `label_backdrop`, and `label_backdrop` was the foreground that
///   failed.
/// - `label_plates_stay_content_facing_not_chrome_facing` **asserted
///   `label_backdrop` stays light** — correct for its stated purpose,
///   and it therefore *agreed with the defect*.
///
/// Both are palette-vs-palette tests: they compare two colours a human
/// deliberately wrote down next to each other. The defect was not in
/// the palette. It was in the **assignment** — which palette entry
/// ends up as a foreground and which as a background on the
/// `egui::Style` that gets painted. No amount of checking the palette
/// against itself can see that, because the pair that renders was
/// never a pair anyone wrote down.
///
/// A structural gate could not see it either. The project's
/// `check-theme-colors.sh` bans raw `Color32` literals outside the
/// theme module — a real and useful rule that says nothing about
/// whether the named colours are legible together. As `DEFECTS.md`
/// puts it: *the gate is structural, not perceptual.*
///
/// So this test reads the **rendered style** back and enumerates the
/// pairs as `egui` resolves them. Its coverage is defined by the
/// widget-state matrix rather than by anyone's list, which means a
/// fill someone forgets to assign in a future preset is caught by the
/// same assertion that catches a fill someone assigns wrongly. That
/// property — *the test enumerates the render surface, not the
/// author's intentions* — is the transferable lesson.
///
/// # On the threshold
///
/// 90 on a 0–255 crude luminance scale, the same figure the salvaged
/// text test uses, and for the same reason: a coarse check that always
/// fires beats a precise one nobody runs. It is not a WCAG ratio and
/// does not claim to be. The values it passes are comfortable — the
/// tightest real pair in the shipped presets is `on_accent` on
/// `accent` in the dark preset, at roughly 125.
#[test]
fn every_rendered_widget_pair_is_readable_in_every_preset() {
    for preset in Preset::ALL {
        let theme = Theme::new(*preset);
        if let Err(failures) = theme.check_contrast(contrast::READABLE_LUMA_GAP) {
            let detail = failures
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n  ");
            panic!(
                "{preset:?}: {} rendered widget pair(s) are not readable. \
                 A pair here is a foreground egui WILL paint on a background egui \
                 WILL paint it on — not two palette entries someone chose together, \
                 which is the distinction that let DEFECTS.md D2 ship past two \
                 adjacent tests:\n  {detail}",
                failures.len()
            );
        }
    }
}

/// **The gate is not vacuous: it fails on the defect it was written
/// for.**
///
/// Without this, `every_rendered_widget_pair_is_readable_in_every_preset`
/// would pass identically if [`contrast::check`] returned `Ok` for
/// everything, and would be asserting nothing at all. So this
/// reconstructs D2 exactly — a light foreground on the active state
/// with its `bg_fill` left at `egui`'s default — and asserts the gate
/// catches it and *names the state*.
///
/// This is the same discipline the salvage source applied to its
/// script-parser tests: a test that proves a typo is rejected is worth
/// nothing beside a test that proves the correct spelling is accepted.
#[test]
fn the_contrast_gate_catches_the_exact_defect_it_was_written_for() {
    let mut style = egui::Style::default();
    // egui's default light `bg_fill` for the active state, i.e. what
    // D2 left in place because nothing assigned it.
    style.visuals.widgets.active.bg_fill = Color32::from_gray(0xC8);
    style.visuals.widgets.active.fg_stroke =
        egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(250, 250, 250, 220));

    let failures = contrast::check(&style, contrast::READABLE_LUMA_GAP)
        .expect_err("near-white on light grey must fail the gate");
    assert!(
        failures
            .iter()
            .any(|f| f.state == contrast::WidgetState::Active
                && f.fill == contrast::FillKind::BgFill),
        "the gate must name the widget state and the fill that failed, \
         so the message points at the line to change; got: {failures:?}"
    );
}

/// **`on_accent` is a role, and the presets prove it has to be.**
///
/// If every preset's `on_accent` were the same light colour, the field
/// would be a constant wearing a role's clothes and the next editor
/// would be right to inline it — reintroducing exactly the coupling
/// that made D2 invisible. The dark preset inverts it, and that is the
/// standing evidence the separation is load-bearing.
#[test]
fn on_accent_inverts_where_the_accent_is_light() {
    let quiet = Theme::new(Preset::Quiet).palette;
    let dark = Theme::new(Preset::Dark).palette;
    assert!(
        luma(quiet.on_accent) > luma(quiet.accent),
        "a dark accent takes a light foreground"
    );
    assert!(
        luma(dark.on_accent) < luma(dark.accent),
        "a light accent takes a dark foreground — which a single shared \
         plate colour could never have expressed, and is why `on_accent` \
         is a role rather than a constant"
    );
}

/// **★★★ Defect T2's regression test: a SELECTED WIDGET's text is
/// readable on the fill `egui` will paint behind it, in every preset.**
///
/// # Why the gate above cannot see this pair, and why that mattered
///
/// `every_rendered_widget_pair_is_readable_in_every_preset` enumerates
/// `WidgetState::ALL` × `FillKind::ALL` — ten pairs, foreground always
/// `fg_stroke.color`, background always one of the state's own fills. The
/// selected pair is in none of them, because `egui` does not *store* it:
/// `Style::button_style` substitutes it at paint time
/// (`egui-0.35.0/src/widget_style.rs:151-154`), overwriting both fills
/// and the text colour from `visuals.selection`. Reading the `Style` back
/// therefore cannot reach it — the pair that renders was never a pair in
/// the struct.
///
/// So this test reproduces `egui`'s substitution arithmetic explicitly.
/// It is the same discipline as the rendered-pair gate one level up:
/// measure what will be *painted*, not what somebody wrote down.
///
/// # The background is composited, and that is load-bearing
///
/// The fill is measured **over `panel`**, through [`contrast::over`],
/// because the value that shipped here was `selection_fill` — 27 % alpha.
/// Measuring a translucent fill as if it were opaque is exactly the error
/// that hides this class of defect: it overstates the gap against a dark
/// background and understates it against a light one.
///
/// # The numbers this pins, measured 2026-09-04
///
/// | preset | `accent` on 27 % wash | `on_accent` on `accent` | `accent` on `selected_plate` |
/// |---|---:|---:|---:|
/// | Quiet | 120 | 165 | 103.1 |
/// | Airy  | 137 | 165 | 118.9 |
/// | Dark  | **72.5** | 125 | 123.2 |
///
/// The third column is what ships. It is lower than the second and that is
/// a deliberate trade, not a regression: the second column's ink was
/// `on_accent`, which made the focused-`TextEdit` ring `egui` draws from
/// the *same* channel unreadable (17.9 / 5.0 / 29.1). See
/// `both_roles_the_selection_channel_serves_are_readable_in_every_preset`,
/// which is the test that now holds both ends at once.
///
/// Only Dark actually breached the floor, and Dark is the preset the
/// reviewer's screenshots came from. Quiet and Airy passed *by accident of
/// their panel being light* — the same colours over a dark panel failed.
/// A pair that is readable only because of what happens to be behind it is
/// not readable; it is lucky. Pinning all three presets is what converts
/// the luck into an assertion.
#[test]
fn a_selected_widgets_text_is_readable_on_the_fill_egui_paints_behind_it() {
    for preset in Preset::ALL {
        let theme = Theme::new(*preset);
        let style = theme.rendered_style();
        let v = &style.visuals;

        // Exactly what `Style::button_style` does for a selected button:
        // the fill becomes `selection.bg_fill` and the text becomes
        // `selection.stroke.color`. The panel is what the fill is
        // composited over, because a selected control lives on a panel.
        let fg = v.selection.stroke.color;
        let bg = contrast::over(v.selection.bg_fill, theme.palette.panel);
        let gap = contrast::gap(fg, bg);

        assert!(
            gap >= contrast::READABLE_LUMA_GAP,
            "{preset:?}: a selected widget renders text of luminance {:.1} on a \
             fill of luminance {:.1} — a gap of {gap:.1}, under the floor of {:.0}. \
             This is every bare `ui.selectable_label(true, …)` and every \
             `Button::selected(true)` in the application at once: egui takes BOTH \
             the fill and the text colour from `visuals.selection`, so a theme that \
             points that channel anywhere but at a fill-and-its-ink pair makes all \
             of them unreadable together. See `Theme::write_style`.",
            contrast::luma(fg),
            contrast::luma(bg),
            contrast::READABLE_LUMA_GAP,
        );
    }
}

/// **The selection channel carries the CHROME pair, not the canvas one.**
///
/// The test above measures a *property* (readable), which several wrong
/// answers could satisfy. This pins the *identity*: `selection.bg_fill` is
/// [`Palette::selected_plate`] and `selection.stroke.color` is
/// [`Palette::accent`], in every preset.
///
/// # ★★★ Why the identity is worth pinning separately, and it is T1's lesson
///
/// Because named accessors elsewhere promise it.
/// [`Theme::selected_widget_ink`] and [`Theme::selected_widget_pair`] tell
/// their callers they hand back *what `egui` will actually paint* — that is
/// the whole reason a call site is allowed to use them instead of reading
/// `visuals.selection` (which `tools/gates/check-selection-channel.sh` now
/// forbids outright, with no file-level exemption left). A promise like
/// that is only worth what it is checked by: if `write_style` and the
/// accessors drift, `icons::selected_image` tints a glyph for a plate that
/// is no longer behind it, and nothing else in the tree would notice.
///
/// `REVIEW_TRIAGE.md` T1 is what happens when such a premise lives only in
/// prose: `check-strong-text.sh` blessed two sites on the ground that
/// *"both are drawn ON the accent fill"*, which was true of one of them
/// and had silently stopped being true of the other. The gate passed a
/// defective site for a reason that had expired. **A blessing whose
/// premise is a fact about the code should be held by an assertion about
/// the code**, and this is that assertion.
#[test]
fn the_selection_channel_resolves_to_the_selected_plate_pair_in_every_preset() {
    for preset in Preset::ALL {
        let theme = Theme::new(*preset);
        let v = &theme.rendered_style().visuals;
        assert_eq!(
            v.selection.bg_fill, theme.palette.selected_plate,
            "{preset:?}: `visuals.selection.bg_fill` is egui's SELECTED-WIDGET plate \
             and must be `selected_plate` — the OPAQUE diluted accent, not `accent` \
             itself and emphatically not `selection_fill`, the 27 % canvas wash. \
             `Theme::selected_widget_pair` promises this value to every call site \
             that hand-draws a mark on a selected control."
        );
        assert_eq!(
            v.selection.stroke.color, theme.palette.accent,
            "{preset:?}: `visuals.selection.stroke.color` is the ink egui paints on \
             that plate — the label of every selected button, the highlight text in \
             a TextEdit, and the glyph `icons::selected_image` tints — AND it is the \
             frame stroke of a focused TextEdit, drawn on `panel`. `accent` is the \
             one role that reads on both. `on_accent` was tried here and made the \
             focus ring near-white on a near-white panel (gaps 17.9 / 5.0 / 29.1)."
        );
    }
}

/// **BOTH roles `egui` drives from `visuals.selection` are readable, in
/// every preset.** Six numbers, one floor.
///
/// # ★★★ Why one channel has two backgrounds, and why that is the whole
/// difficulty
///
/// `egui` spends `selection.stroke.color` twice, on two different grounds,
/// and offers no way to separate them:
///
/// 1. as the **ink on a selected widget's plate** — `widget_style.rs:151-154`
///    substitutes `selection.bg_fill` into both fills and
///    `selection.stroke.color` into the text of anything `.selected(true)`;
/// 2. as the **frame stroke of a focused, mutable `TextEdit`** —
///    `widgets/text_edit/builder.rs:699-706`, drawn over
///    `text_edit_bg_color()`, which falls back to `extreme_bg_color`, which
///    [`Theme::write_style`] points at [`Palette::panel`]. `TextEdit` has
///    **no `.frame_stroke()`**: there is no per-widget escape hatch.
///
/// A test that measured only (1) is how the ring was lost. It existed, it
/// was green, and it was green about the wrong half of the channel. So this
/// measures both, from the `Style` that actually ships.
///
/// # The numbers, measured 2026-09-04, floor 90
///
/// | preset | selected pair (`accent` on `selected_plate`) | focus ring (`accent` on `panel`) |
/// |---|---:|---:|
/// | Quiet | 103.1 | 147.3 |
/// | Airy  | 118.9 | 170.2 |
/// | Dark  | 123.2 |  **96.0** |
///
/// For comparison, the arrangement this replaced (`on_accent` on `accent`,
/// `on_accent` on `panel`) measured 165 / 165 / 125 on the first column and
/// **17.9 / 5.0 / 29.1** on the second. Airy's ring was white on white to
/// within five levels of luminance.
///
/// ★ Dark's ring, at 96.0, is the tightest pair in the theme. It is also
/// the reason this is a loop over `Preset::ALL` and not a spot check: the
/// light presets clear both columns by fifty or more and would happily
/// bless an accent that Dark cannot use.
///
/// ★★ The ring is measured against `panel` and NOT against
/// `widgets.*.bg_fill`, because that is genuinely where `egui` draws it —
/// `text_edit_bg_color()`, not the widget state's own fill. Measuring the
/// convenient background instead of the real one is the error this whole
/// family of tests exists to avoid; see [`contrast`]'s module header.
#[test]
fn both_roles_the_selection_channel_serves_are_readable_in_every_preset() {
    for preset in Preset::ALL {
        let theme = Theme::new(*preset);
        let style = theme.rendered_style();
        let v = &style.visuals;
        let ink = v.selection.stroke.color;

        // Role 1 — the selected widget. `over` rather than a bare read
        // because a translucent plate must be measured composited; the
        // shipped `selected_plate` is opaque, and this is what would catch
        // a future edit that made it a wash again.
        let plate = contrast::over(v.selection.bg_fill, theme.palette.panel);
        let selected_gap = contrast::gap(ink, plate);
        assert!(
            selected_gap >= contrast::READABLE_LUMA_GAP,
            "{preset:?}: a SELECTED WIDGET renders ink of luminance {:.1} on a plate \
             of luminance {:.1} — a gap of {selected_gap:.1}, under the floor of \
             {:.0}. egui takes both the plate and the ink from \
             `visuals.selection`, so this is every `selectable_label(true, …)`, \
             every `Button::selected(true)`, the highlight behind selected text and \
             a ProgressBar's fill, all at once. See `Theme::write_style`.",
            contrast::luma(ink),
            contrast::luma(plate),
            contrast::READABLE_LUMA_GAP,
        );

        // Role 2 — the focused TextEdit's ring, on `text_edit_bg_color()`.
        let ring_bg = v.text_edit_bg_color();
        let ring_gap = contrast::gap(ink, ring_bg);
        assert!(
            ring_gap >= contrast::READABLE_LUMA_GAP,
            "{preset:?}: a FOCUSED TextEdit draws its frame with \
             `visuals.selection.stroke` (luminance {:.1}) over \
             `text_edit_bg_color()` (luminance {:.1}) — a gap of {ring_gap:.1}, \
             under the floor of {:.0}. A focused field then looks unfocused, and \
             `TextEdit` has no `.frame_stroke()` to override it with. This is the \
             half of the channel that was lost the first time it was re-pointed: \
             `on_accent` here measured 17.9 / 5.0 / 29.1. If the selected-widget \
             pair above is what you were tuning, tune `Palette::selected_plate` \
             instead — diluting the PLATE leaves the ink free to be `accent`, which \
             is far from `panel` by construction.",
            contrast::luma(ink),
            contrast::luma(ring_bg),
            contrast::READABLE_LUMA_GAP,
        );
    }
}

/// **The selected-widget accessors hand back exactly what `egui` will
/// paint.**
///
/// [`Theme::selected_widget_ink`] exists so that
/// `icons::selected_image` can tint a glyph to match a plate it did not
/// paint, without reading `visuals.selection` (which the selection-channel
/// gate forbids). That is only safe while the accessor and `write_style`
/// agree — otherwise the glyph is coloured for a background that is no
/// longer behind it, which is D2 in miniature and would be invisible in
/// every other test.
///
/// Read through a real `Context` with the theme applied, because the
/// accessors go via [`Theme::of`] and a stash that never happened would
/// make them silently return the default preset's colours.
#[test]
fn the_selected_widget_accessors_agree_with_the_style_egui_will_paint() {
    for preset in Preset::ALL {
        let theme = Theme::new(*preset);
        let ctx = egui::Context::default();
        theme.apply(&ctx);
        let v = &theme.rendered_style().visuals;

        assert_eq!(
            Theme::selected_widget_ink(&ctx),
            v.selection.stroke.color,
            "{preset:?}: `selected_widget_ink` must equal what egui paints as the \
             ink on a selected widget, or every glyph tinted with it is coloured \
             for a plate that is not there."
        );
        assert_eq!(
            Theme::selected_widget_pair(&ctx),
            (v.selection.bg_fill, v.selection.stroke.color),
            "{preset:?}: the pair form must agree with the shipped style, and in \
             the order `(plate, ink)` — the same order `Theme::accent_pair` uses, \
             since both answer \"a chrome surface and the ink on it\". Reversed, it \
             is a silent colour swap at every call site that destructures it."
        );
    }
}

/// **The canvas accessors return the canvas roles — the values the
/// content area used to reach through `visuals.selection` for.**
///
/// The whole safety argument for defect T2's fix is that re-pointing the
/// widget channel cost the canvas **nothing**: the ~33 content-area
/// readers were moved to [`Theme::canvas_selection_ink`] and
/// [`Theme::canvas_selection_fill`], which hand back `accent` and
/// `selection_fill` — bit-for-bit the two values that channel used to
/// carry. This asserts it rather than asserting it in a comment, so a
/// later edit that "tidies" one of these into the chrome pair turns the
/// overlay a different colour *and goes red* instead of shipping.
///
/// Read through a real `Context` with the theme applied, not off the
/// struct, because the accessors go via [`Theme::of`] and a stash that
/// never happened would make every one of them silently return the
/// default preset's colours — the failure [`Theme::apply`]'s own doc
/// comment names.
#[test]
fn the_canvas_accessors_hand_back_the_values_the_canvas_used_to_read() {
    for preset in Preset::ALL {
        let theme = Theme::new(*preset);
        let ctx = egui::Context::default();
        theme.apply(&ctx);

        assert_eq!(
            Theme::canvas_selection_ink(&ctx),
            theme.palette.accent,
            "{preset:?}: canvas selection ink must stay `accent` — it is what \
             `visuals.selection.stroke.color` carried before T2, and every outline, \
             grip, node mark and drop caret in the content area is drawn with it."
        );
        assert_eq!(
            Theme::canvas_selection_fill(&ctx),
            theme.palette.selection_fill,
            "{preset:?}: canvas selection fill must stay the translucent \
             `selection_fill` — seeing the object through the tint is the point, \
             and that is exactly why it was unfit for the widget channel."
        );
        assert_eq!(
            Theme::canvas_selection_pair(&ctx),
            (theme.palette.accent, theme.palette.selection_fill),
            "{preset:?}: the pair form must agree with the two singles, or a call \
             site that draws a washed rect with an outline gets two colours from \
             two different stories."
        );
    }
}

/// Settings keys round-trip, and an unknown key is `None` rather than
/// a silent default. Salvaged.
#[test]
fn preset_keys_round_trip_and_unknown_keys_are_refused() {
    for preset in Preset::ALL {
        assert_eq!(Preset::from_key(preset.key()), Some(*preset));
    }
    assert_eq!(Preset::from_key("solarized"), None);
    assert_eq!(Preset::from_key(""), None);
}

/// **An absurd metric produces a sane margin.**
///
/// `egui::Margin` is `i8`. A plain cast turns 200 pt of padding into
/// −56, which paints content outside its own panel: a silent geometry
/// defect produced by a number that looked fine where it was typed.
#[test]
fn an_out_of_range_panel_padding_saturates_rather_than_wrapping() {
    assert_eq!(clamp_to_i8(6.0), 6);
    assert_eq!(clamp_to_i8(200.0), 127);
    assert_eq!(clamp_to_i8(-200.0), -128);
    assert_eq!(clamp_to_i8(f32::NAN), 0);
}
