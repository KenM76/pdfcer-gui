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
/// # ★★★ The widening, 2026-09-04 — `REVIEW_TRIAGE.md` A15e
///
/// This test was called `..._widget_pair_...` until that date, and the
/// word was load-bearing in the wrong direction: it enumerated ten pairs
/// and **was green through three separately shipped contrast defects**,
/// because none of the three was a widget pair. [`contrast::pairs`] now
/// returns **twenty-seven**, and its module header states which of the
/// three the widening reaches (the selection channel) and which two it
/// deliberately still cannot (a caller's `RichText::color`, and a
/// background produced by geometry) with the gates that do cover those.
///
/// The seventeen new pairs are the foregrounds `egui` resolves through a
/// `Visuals` *accessor* rather than storing in a `WidgetVisuals` — body
/// text, weak text, strong text, hyperlink, warn, error — each measured
/// on the grounds it is really drawn on, plus the two roles the selection
/// channel serves.
///
/// # The numbers, measured 2026-09-04, floor 90
///
/// Every new pair, per preset. The widget ten are omitted; they were
/// already comfortable and are unchanged.
///
/// | pair | Quiet | Airy | Dark |
/// |---|---:|---:|---:|
/// | `text_color()` on `panel_fill` | 213.9 | 212.1 | 194.0 |
/// | `text_color()` on `window_fill` | 204.0 | 217.1 | 183.1 |
/// | `text_color()` on `text_edit_bg_color()` | 204.0 | 217.1 | 183.1 |
/// | `weak_text_color()` on `panel_fill` | 128.9 | 128.0 | 116.0 |
/// | `weak_text_color()` on `window_fill` | 123.0 | 131.0 | 109.2 |
/// | `weak_text_color()` on `text_edit_bg_color()` | 123.0 | 131.0 | 109.2 |
/// | `strong_text_color()` on `panel_fill` | **7.9** | **0.1** | **18.3** |
/// | `strong_text_color()` on `window_fill` | **17.9** | **5.0** | **29.1** |
/// | `hyperlink_color` on `panel_fill` | 157.2 | 165.2 | 106.8 |
/// | `hyperlink_color` on `window_fill` | 147.3 | 170.2 | **96.0** |
/// | `warn_fg_color` on `panel_fill` | 127.0 | 135.0 | 128.8 |
/// | `warn_fg_color` on `window_fill` | 117.0 | 139.9 | 118.0 |
/// | `error_fg_color` on `panel_fill` | 168.2 | 176.2 | 113.2 |
/// | `error_fg_color` on `window_fill` | 158.3 | 181.1 | 102.3 |
/// | selected plate over `panel_fill` | 103.1 | 118.9 | 123.2 |
/// | selected plate over `window_fill` | 103.1 | 118.9 | 123.2 |
/// | focus ring on `text_edit_bg_color()` | 147.3 | 170.2 | **96.0** |
///
/// ★ Two rows need reading rather than scanning.
///
/// **`strong_text_color()` is the theme's only exemption**, and it is
/// structural rather than a tuning miss: that accessor **is**
/// `widgets.active.fg_stroke.color`, the ink chosen for the accent FILL,
/// and no value reads on both the accent and a panel. It is measured,
/// excused by [`contrast::EXEMPTIONS`] with the argument written out, and
/// expired by `every_contrast_exemption_still_has_a_subject`. What covers
/// it instead is `tools/gates/check-strong-text.sh`, at the call site.
///
/// **`error_fg_color` on `window_fill` in Dark is the pair the widening
/// actually caught.** It measured **89.7** against this floor on the day
/// it was first enumerated — a real, if marginal, shortfall in the colour
/// every dialog uses to say the operator must act. The fix was to move the
/// role (`Palette::danger` in the dark preset, `#FF6B6B` → `#FF7B7B`), not
/// the threshold and not the pair; `Theme::dark` carries the arithmetic.
///
/// # On the threshold
///
/// 90 on a 0–255 crude luminance scale, the same figure the salvaged
/// text test uses, and for the same reason: a coarse check that always
/// fires beats a precise one nobody runs. It is not a WCAG ratio and
/// does not claim to be. The values it passes are comfortable — the
/// tightest two real pairs in the shipped presets are Dark's focus ring
/// and Dark's hyperlink, both at 96.0.
#[test]
fn every_rendered_pair_is_readable_in_every_preset() {
    for preset in Preset::ALL {
        let theme = Theme::new(*preset);
        if let Err(failures) = theme.check_contrast(contrast::READABLE_LUMA_GAP) {
            let detail = failures
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n  ");
            panic!(
                "{preset:?}: {} rendered pair(s) are not readable. \
                 A pair here is a foreground egui WILL paint on a background egui \
                 WILL paint it on — not two palette entries someone chose together, \
                 which is the distinction that let DEFECTS.md D2 ship past two \
                 adjacent tests:\n  {detail}",
                failures.len()
            );
        }
    }
}

/// **★★ Every contrast exemption still describes a pair that would
/// otherwise fail.**
///
/// # Why the other direction needs its own test
///
/// [`contrast::check`] skips the entries in [`contrast::EXEMPTIONS`]. It
/// is silent about an entry with nothing behind it — so a blessing
/// survives its own subject, and what survives is a paragraph arguing
/// that a pair *cannot* be made readable, about a pair that now is.
///
/// That is not tidiness. A stale exemption is a **licence**: the next
/// person who wants a contrast failure waved through finds a precedent in
/// the list, and the precedent is a state that has not existed for months.
/// This project has already paid for the same shape twice in one week — a
/// gate exemption whose premise expired within a day, and
/// `check-strong-text.sh` blessing a site on a sentence that had stopped
/// being true (`REVIEW_TRIAGE.md` T1). ⇒ **Blessings expire with their
/// subject, and something has to notice.**
///
/// The shape is `icons::catalog::tests::every_declared_share_is_still_a_share`,
/// which is where this project settled the pattern.
///
/// # What "would otherwise fail" means here, precisely
///
/// At least **one** shipped preset, not all three. An exemption exists
/// because *some* real theme cannot satisfy the pair; requiring every
/// preset to fail would delete an entry the moment one preset's palette
/// happened to drift into the clear, which is the opposite of the point.
/// The failure message lists the per-preset numbers either way, so a
/// reader deleting an entry can see exactly how much room there now is.
///
/// The companion half — "and the origin still exists" — is
/// `contrast::tests::every_exemption_names_an_origin_the_gate_produces`,
/// which needs no presets and lives with the module.
#[test]
fn every_contrast_exemption_still_has_a_subject() {
    for exemption in contrast::EXEMPTIONS {
        let mut measured: Vec<(Preset, f32)> = Vec::new();
        for preset in Preset::ALL {
            let style = Theme::new(*preset).rendered_style();
            let pair = contrast::pairs(&style)
                .into_iter()
                .find(|p| p.origin == exemption.origin)
                .unwrap_or_else(|| {
                    panic!(
                        "the exemption granted {} covers {:?}, which `contrast::pairs` \
                         no longer produces",
                        exemption.granted, exemption.origin
                    )
                });
            measured.push((*preset, pair.gap));
        }
        assert!(
            measured
                .iter()
                .any(|(_, gap)| *gap < contrast::READABLE_LUMA_GAP),
            "the contrast exemption granted {} for {:?} now passes the gate in EVERY \
             preset ({}), so it excuses nothing. Delete it. An allow-list entry left \
             behind after its subject is gone reads as a decision somebody made and is \
             a precedent for a failure nobody argued for. Its argument was:\n  {}",
            exemption.granted,
            exemption.origin,
            measured
                .iter()
                .map(|(p, g)| format!("{p:?} {g:.1}"))
                .collect::<Vec<_>>()
                .join(", "),
            exemption.reason,
        );
    }
}

/// **The gate is not vacuous: it fails on the defect it was written
/// for.**
///
/// Without this, `every_rendered_pair_is_readable_in_every_preset`
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
        failures.iter().any(|f| f.origin
            == contrast::Origin::Widget {
                state: contrast::WidgetState::Active,
                fill: contrast::FillKind::BgFill,
            }),
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

// ★ **`a_selected_widgets_text_is_readable_on_the_fill_egui_paints_behind_it`
// used to live here, and was deleted on 2026-09-04 during the A15e
// widening.** It measured one half of what
// `both_roles_the_selection_channel_serves_are_readable_in_every_preset`
// measures, with the same arithmetic and a weaker message, and both of
// them now read their numbers out of the one gate rather than
// reproducing `egui`'s substitution by hand. Its own table — `accent` on
// the 27 % wash at 120 / 137 / **72.5**, which is what defect T2
// actually shipped — is preserved in that test's doc comment below, so
// nothing it recorded was lost with it.
//
// The reason it is worth a note rather than a silent deletion: three
// tests measuring one pair is how a project ends up unable to say which
// of them is the contract.

/// **The selection channel carries the CHROME pair, not the canvas one.**
///
/// `both_roles_the_selection_channel_serves_are_readable_in_every_preset`
/// below, and the widened gate under it, measure a *property* (readable),
/// which several wrong answers could satisfy. This pins the *identity*:
/// `selection.bg_fill` is
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
        let measured = contrast::pairs(&style);
        let ink = style.visuals.selection.stroke.color;

        // ★ Both numbers come out of `contrast::pairs` rather than being
        // recomputed here. Until the A15e widening this test reproduced
        // `egui`'s substitution arithmetic by hand, because the gate could
        // not reach it; the gate reaches it now, and a second hand-rolled
        // copy of an arithmetic this delicate is a place for the two to
        // disagree. What this test still owns — and the reason it was not
        // simply deleted into the gate — is the two MESSAGES below, which
        // say what to tune and in which direction. The gate's own message
        // names the fields; these name the move.
        let find = |origin: contrast::Origin| {
            measured
                .iter()
                .find(|p| p.origin == origin)
                .copied()
                .unwrap_or_else(|| {
                    panic!(
                        "{preset:?}: `contrast::pairs` no longer produces {origin:?}. \
                         Both roles this channel serves must stay in the one gate — \
                         a role that leaves the enumeration is a role nothing measures."
                    )
                })
        };

        // Role 1 — the selected widget, its plate composited over the
        // window ground. The shipped `selected_plate` is opaque, so the
        // compositing is a no-op today; it is what would catch a future
        // edit that made it a wash again.
        let selected = find(contrast::Origin::SelectedWidget {
            ground: contrast::Ground::WindowFill,
        });
        assert!(
            selected.gap >= contrast::READABLE_LUMA_GAP,
            "{preset:?}: a SELECTED WIDGET renders ink of luminance {:.1} on a plate \
             of luminance {:.1} — a gap of {:.1}, under the floor of {:.0}. egui takes \
             both the plate and the ink from `visuals.selection`, so this is every \
             `selectable_label(true, …)`, every `Button::selected(true)`, the highlight \
             behind selected text and a ProgressBar's fill, all at once. See \
             `Theme::write_style`.",
            contrast::luma(ink),
            contrast::luma(selected.bg),
            selected.gap,
            contrast::READABLE_LUMA_GAP,
        );

        // Role 2 — the focused TextEdit's ring, on `text_edit_bg_color()`.
        let ring = find(contrast::Origin::FocusRing);
        assert!(
            ring.gap >= contrast::READABLE_LUMA_GAP,
            "{preset:?}: a FOCUSED TextEdit draws its frame with \
             `visuals.selection.stroke` (luminance {:.1}) over \
             `text_edit_bg_color()` (luminance {:.1}) — a gap of {:.1}, under the floor \
             of {:.0}. A focused field then looks unfocused, and `TextEdit` has no \
             `.frame_stroke()` to override it with. This is the half of the channel \
             that was lost the first time it was re-pointed: `on_accent` here measured \
             17.9 / 5.0 / 29.1. If the selected-widget pair above is what you were \
             tuning, tune `Palette::selected_plate` instead — diluting the PLATE leaves \
             the ink free to be `accent`, which is far from `panel` by construction.",
            contrast::luma(ink),
            contrast::luma(ring.bg),
            ring.gap,
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
