//! # `app::prefs::tests` — split out under R2 on 2026-08-28
//!
//! ★★ **The inner `#![cfg(test)]` is load-bearing and is not a duplicate of
//! the outer `#[cfg(test)] mod tests;`.** Without it, `tools/gates/check-ui-strings.sh`
//! walks this file as ordinary source and reports every assertion message as a
//! user-visible string that should live in `ui_text` — exclusion 2b in that
//! gate. It is the same line every other split test file in this crate carries.
#![cfg(test)]

use super::*;
use crate::viewer::{FitMode, ViewState};

/// ★ Every value round-trips through the file.
///
/// The property a preferences store exists for, and the one a hand-written
/// writer and a hand-written parser get wrong first: they are two spellings
/// of the same vocabulary, and this is what stops them drifting.
///
/// Every field is varied, and the two enums are varied over **all** their
/// values rather than one apiece — a writer that emitted a constant token
/// would pass a single-value check.
#[test]
fn every_preference_round_trips_through_the_file() {
    for quality in RenderQuality::ALL {
        for fit in OpeningFit::ALL {
            let original = Prefs {
                // ★ Non-default, like every field here. O70: `false`, because
                // the shipped default is `true` and a writer that emitted a
                // constant would otherwise pass.
                smart_select: false,
                // ★ Non-default for the identical reason — O96 ships `true`,
                // so a writer emitting a constant `true` would round-trip a
                // preference the operator had turned OFF and this test would
                // not notice.
                shade_form_fields: false,
                // ★ Non-default, like every field here — and this one is
                // the only OPTIONAL key in the file, so a writer that emitted
                // nothing for it would fail this round trip. `Facing` rather
                // than `Single` so it cannot coincide with the compiled-in
                // per-mode answer either. O80.
                default_page_display: Some(crate::viewer::PageDisplay::Facing),
                font_folders: vec![std::path::PathBuf::from("C:/Fonts")],
                use_os_fonts: true,
                // ★ Non-default, like every field here, and with a SPACE in
                // it — a path a person would really type on Windows. O122.
                acrobat_path: r"D:\Apps\Acrobat DC\Acrobat.exe".to_owned(),
                // ★ Non-default, like every other field here: a `None`
                // would pass on a build whose writer emitted no
                // `chosen_standard` key at all.
                chosen_standard: Some("pdf-x1a".to_owned()),
                // ★ Non-default, and with a SPACE in it: the writer emits
                // the value raw and the parser trims, so a name of one word
                // would pass on a reader that split on whitespace.
                author_name: "Ken Mantle".to_owned(),
                // The migration marker round-trips like every other key.
                // `true` rather than the default `false`, per this test's
                // own rule: a non-default in every field, so no emitted
                // value can coincide with what a failed parse left behind.
                render_quality: *quality,
                page_cache: PageCache::default(),
                zoom_settle_ms: 275,
                // A non-default well past the shipped ceiling, so the
                // round trip is proved on a value that MATTERS rather
                // than on 800.
                max_zoom_percent: 1_000_000.0,
                opening_fit: *fit,
                // ★ The non-default, so a writer that emitted no
                // `wheel_paging` key at all would fail here rather than
                // pass by landing back on `Scroll`.
                paste_chords: PasteChords::AcrobatOrder,
                wheel_paging: WheelPaging::FlipPages,
                // Deliberately not all-true and not all-false: an assignment
                // that crossed two of the three fields would survive either.
                chrome: PageChrome {
                    rulers: true,
                    grid: false,
                    guides: true,
                },
                // A non-default that is ON the control's step, so the round
                // trip tests the writer's formatting rather than the
                // loader's rounding — that is `an_off_step_ui_scale_is_rounded_and_reported`'s job.
                ui_scale: 1.25,
            };
            let (read_back, notes) = Prefs::parse(&original.write_to_string());
            assert!(
                notes.is_empty(),
                "a written file did not read cleanly: {notes:?}"
            );
            assert_eq!(
                read_back, original,
                "{quality:?}/{fit:?} did not survive the trip"
            );
        }
    }
}

/// ★ The three overlays are three independent keys.
///
/// The failure this catches is a copy-paste in either the writer or the
/// parser sending two overlays to one field — which the round-trip above
/// would only catch for the specific combination it happens to use. Here
/// each is set alone and the other two are asserted to have stayed off.
#[test]
fn each_overlay_is_written_and_read_on_its_own_key() {
    for (name, build) in [
        (
            "rulers",
            PageChrome {
                rulers: true,
                ..PageChrome::default()
            },
        ),
        (
            "grid",
            PageChrome {
                grid: true,
                ..PageChrome::default()
            },
        ),
        (
            "guides",
            PageChrome {
                guides: true,
                ..PageChrome::default()
            },
        ),
    ] {
        let original = Prefs {
            chrome: build,
            ..Prefs::default()
        };
        let (read_back, notes) = Prefs::parse(&original.write_to_string());
        assert!(notes.is_empty(), "{name}: {notes:?}");
        assert_eq!(read_back.chrome, build, "{name} landed in the wrong field");
    }
}

/// The shipped defaults are what the constants they replaced held.
///
/// `ZOOM_SETTLE` was a compiled-in 150 ms, `raster_scale` had no
/// multiplier at all, and `ViewState::default` was fit-page with all three
/// overlays off. A build that never opens the Settings window has to
/// behave exactly as the build before this module did — the standing rule
/// for a capability becoming choosable.
#[test]
fn the_defaults_are_the_constants_they_replaced() {
    let prefs = Prefs::default();
    assert_eq!(prefs.zoom_settle_ms, 150);
    assert!((prefs.render_quality.multiplier() - 1.0).abs() < f32::EPSILON);
    assert_eq!(prefs.opening_fit, OpeningFit::Page);
    assert!(prefs.chrome.all_hidden());
}

/// ★ **The shipped preferences change nothing about a freshly opened view.**
///
/// The strongest form of the rule above, and the one a reordering or a
/// typo in [`Prefs::seed_view`] would break: seeding a default `ViewState`
/// from default preferences must leave it **byte-identical**. Asserting
/// the fields one at a time would pass while a fourth field was silently
/// clobbered; asserting the whole struct will not.
#[test]
fn seeding_from_the_shipped_preferences_changes_nothing() {
    let mut view = ViewState::default();
    Prefs::default().seed_view(&mut view);
    assert_eq!(
        view,
        ViewState::default(),
        "the shipped preferences moved a freshly opened view"
    );
}

/// Each opening fit reaches the view it names.
#[test]
fn the_opening_fit_reaches_the_view() {
    for (fit, expected) in [
        (OpeningFit::Page, FitMode::Page),
        (OpeningFit::Width, FitMode::Width),
        (OpeningFit::Height, FitMode::Height),
        (OpeningFit::ActualSize, FitMode::None),
    ] {
        let mut view = ViewState::default();
        Prefs {
            opening_fit: fit,
            ..Prefs::default()
        }
        .seed_view(&mut view);
        assert_eq!(view.fit, expected, "{fit:?}");
        assert!(view.zoom > 0.0, "{fit:?} seeded a zoom of {}", view.zoom);
    }
}

/// ★ **A document's remembered guides survive a preference that hides them.**
///
/// Row two of [`Prefs::seed_view`]'s table, and the whole reason that one
/// field ORs. `canvas::guides::opening` turns the layer on for a document
/// that has guides saved against it, because *"the presence of the work is
/// the preference"* — and an assignment here would hide work the operator
/// did, on the document they did it on, because of a switch they set weeks
/// earlier about documents in general.
///
/// This is the failing direction: preference **off**, view already **on**.
#[test]
fn a_preference_that_hides_guides_does_not_hide_remembered_ones() {
    // What `OpenDoc::assemble` hands over for a document with saved guides.
    let mut view = ViewState {
        guides: true,
        ..ViewState::default()
    };
    Prefs {
        chrome: PageChrome {
            guides: false,
            ..PageChrome::default()
        },
        ..Prefs::default()
    }
    .seed_view(&mut view);
    assert!(
        view.guides,
        "a document's own remembered guides were hidden by a global default"
    );
}

/// …and rulers and grid do NOT get that treatment.
///
/// The counterpart, and it is what stops the OR being copied to all three
/// out of symmetry. Neither has any per-document memory, so a `true`
/// arriving in the view is not evidence of anything the operator did — it
/// would just be a stale value that the preference could then never turn
/// off.
#[test]
fn rulers_and_grid_follow_the_preference_in_both_directions() {
    let mut view = ViewState {
        rulers: true,
        grid: true,
        ..ViewState::default()
    };
    Prefs::default().seed_view(&mut view);
    assert!(
        !view.rulers,
        "the rulers preference could not turn them off"
    );
    assert!(!view.grid, "the grid preference could not turn it off");
}

/// ★ One bad line never discards the rest of the file.
///
/// The fail-soft contract, and the reason it matters here rather than being
/// inherited politeness: this file is *meant* to be hand-edited, and a
/// parser that failed a whole document over one typo would punish the
/// operator for doing the thing the file invites.
#[test]
fn a_bad_line_costs_only_its_own_key() {
    let (prefs, notes) = Prefs::parse(
        "render_quality = sharper\n\
         this line is not a setting\n\
         zoom_settle_ms = purple\n\
         show_rulers = ture\n\
         opening_fit = width\n\
         unknown_key = 3\n",
    );
    assert_eq!(
        prefs.render_quality,
        RenderQuality::Sharper,
        "a good key was discarded because a later line was bad"
    );
    assert_eq!(
        prefs.zoom_settle_ms, DEFAULT_SETTLE_MS,
        "an unreadable value must fall back for its own key"
    );
    assert!(
        !prefs.chrome.rulers,
        "a misspelt bool must fall back, not be read as true"
    );
    assert_eq!(
        prefs.opening_fit,
        OpeningFit::Width,
        "a good key AFTER a bad one was discarded"
    );
    assert!(
        notes
            .iter()
            .any(|n| matches!(n, PrefNote::Malformed { .. }))
    );
    // Two bad values, not one: the settle and the misspelt bool.
    assert_eq!(
        notes
            .iter()
            .filter(|n| matches!(n, PrefNote::BadValue { .. }))
            .count(),
        2,
        "{notes:?}"
    );
    assert!(
        notes
            .iter()
            .any(|n| matches!(n, PrefNote::UnknownKey { .. }))
    );
}

/// ★★★ **A trillion percent is accepted**, which is the figure the
/// operator named — `OPERATOR_REQUESTS.md` O24.
///
/// The point of the setting is that the performance trade is his; a ceiling
/// exists only because `f32` must stay finite — and since both precision
/// ceilings were removed, the page actually draws there.
#[test]
fn a_trillion_percent_is_accepted_and_the_page_actually_draws_there() {
    let (prefs, notes) = Prefs::parse(
        "max_zoom_percent = 1000000000000
",
    );
    // ★ Accepted in full, and NOT clamped. It was clamped for part of
    // 2026-08-22, while a trillion percent rendered cleanly and showed a
    // blank page; removing the two precision ceilings made the figure he
    // named actually draw, so the clamp went with them.
    assert!((prefs.max_zoom_percent - 1e12).abs() / 1e12 < 1e-6);
    assert!(
        notes.is_empty(),
        "a stated maximum the shell can honour must not be second-guessed"
    );
}

/// ★★ **A non-finite value is refused, not clamped.**
///
/// `inf` would propagate into a scroll extent and blank the canvas, which is
/// the failure `canvas::geometry`'s guards exist for. Reporting it as
/// *clamped* would also imply the operator wrote something reasonable.
#[test]
fn an_infinite_maximum_is_a_bad_value_rather_than_a_clamp() {
    for text in [
        "max_zoom_percent = inf
",
        "max_zoom_percent = NaN
",
    ] {
        let (prefs, notes) = Prefs::parse(text);
        assert_eq!(
            prefs.max_zoom_percent, DEFAULT_MAX_ZOOM_PERCENT,
            "{text:?} must leave the default in place"
        );
        assert!(
            notes.iter().any(|n| matches!(n, PrefNote::BadValue { .. })),
            "{text:?} should be reported as a bad value"
        );
    }
}

/// The default is the MAXIMUM, on the operator's instruction of the
/// shell behaved before this setting existed.
#[test]
fn the_default_maximum_is_the_highest_available() {
    let (prefs, _) = Prefs::parse("");
    assert!(
        (prefs.max_zoom_percent - MAX_MAX_ZOOM_PERCENT).abs() < f32::EPSILON,
        "the operator asked for the default to reach the maximum"
    );
}

/// ★ **The file says a whole number, not `1e12`.**
///
/// The preferences file is the operator's to read and edit; a machine-shaped
/// number there means he cannot tell at a glance what he set.
///
/// ★★ And it records something the operator will otherwise discover by
/// reading his own file: **`f32` cannot hold a trillion exactly.** It
/// stores `999,999,995,904` — a rounding of four thousand parts in a
/// trillion, four ten-millionths of one percent. At a zoom where one screen
/// pixel is a millionth of a point, that difference is unobservable; but a
/// value written back as a number he did not type is worth knowing about
/// rather than being mistaken for a bug.
#[test]
fn the_file_writes_a_readable_number_rather_than_an_exponent() {
    let prefs = Prefs {
        font_folders: Vec::new(),
        use_os_fonts: false,
        max_zoom_percent: 1e12,
        ..Prefs::default()
    };
    let text = prefs.write_to_string();
    assert!(
        text.contains("max_zoom_percent = 999999995904"),
        "the file should spell the number out rather than using an exponent: {text}"
    );
    assert!(
        !text.contains("e12"),
        "no exponent should reach the file: {text}"
    );
}

/// An out-of-range settle is clamped and the clamp is reported.
///
/// Reported, not silent: the operator wrote a number and is getting a
/// different one, which is exactly the kind of quiet substitution the
/// engine's store spends a note variant on.
#[test]
fn an_out_of_range_settle_clamps_and_says_so() {
    let (prefs, notes) = Prefs::parse("zoom_settle_ms = 99999\n");
    assert_eq!(prefs.zoom_settle_ms, MAX_SETTLE_MS);
    assert!(notes.iter().any(|n| matches!(n, PrefNote::Clamped { .. })));

    let (prefs, notes) = Prefs::parse("zoom_settle_ms = 0\n");
    assert_eq!(prefs.zoom_settle_ms, MIN_SETTLE_MS);
    assert!(notes.iter().any(|n| matches!(n, PrefNote::Clamped { .. })));
}

/// An off-step UI scale is rounded to one the control can produce, and the
/// substitution is reported.
///
/// Rounding rather than accepting, because the file is hand-editable and
/// the slider is not: a value of `1.234` would sit in the control until the
/// operator touched it, at which point it would jump — a change they did
/// not make, to a setting they did. Reported for the same reason the settle
/// clamp is: the operator wrote a number and is getting a different one.
#[test]
fn an_off_step_ui_scale_is_rounded_and_reported() {
    let (prefs, notes) = Prefs::parse("ui_scale = 1.234\n");
    assert!(
        (prefs.ui_scale - 1.25).abs() < 1e-5,
        "1.234 became {}",
        prefs.ui_scale
    );
    assert!(notes.iter().any(|n| matches!(n, PrefNote::Clamped { .. })));

    // …and a value already on the step is NOT reported. The other half:
    // a note on every clean file would train the operator to ignore notes.
    let (prefs, notes) = Prefs::parse("ui_scale = 1.25\n");
    assert!((prefs.ui_scale - 1.25).abs() < 1e-5);
    assert!(notes.is_empty(), "a clean value was reported: {notes:?}");
}

/// ★ **A UI scale of `nan` or `inf` is refused, not clamped.**
///
/// The one parse arm in this file that needs a guard beyond `parse()`
/// succeeding. `"nan"` and `"inf"` are both valid `f32` literals, and
/// `f32::clamp` **propagates** NaN rather than rejecting it — so without
/// the `is_finite` check a hand-edited `ui_scale = nan` would flow through
/// `normalise_ui_scale` untouched and reach `Context::set_zoom_factor`,
/// which is a window that draws nothing.
///
/// Reported as a bad value rather than clamped to an end, because the
/// operator did not name an end. `inf` is included for the same reason
/// even though clamping would in fact handle it: two spellings of "this is
/// not a size" should not get two different treatments.
#[test]
fn a_non_finite_ui_scale_is_refused_rather_than_clamped() {
    for spelling in ["nan", "NaN", "inf", "-inf", "infinity"] {
        let (prefs, notes) = Prefs::parse(&format!("ui_scale = {spelling}\n"));
        assert!(
            (prefs.ui_scale - DEFAULT_UI_SCALE).abs() < 1e-6,
            "{spelling:?} produced a scale of {}",
            prefs.ui_scale
        );
        assert!(
            prefs.ui_scale.is_finite(),
            "{spelling:?} reached the zoom factor"
        );
        assert!(
            notes.iter().any(|n| matches!(n, PrefNote::BadValue { .. })),
            "{spelling:?} was substituted silently: {notes:?}"
        );
    }
}

/// A missing file is silent.
///
/// A first run is the expected state, not a fault. Reporting it would train
/// the operator to ignore the channel that carries the real problems — the
/// engine's store makes the same distinction and states it in a table.
#[test]
fn an_empty_file_produces_defaults_and_no_notes() {
    let (prefs, notes) = Prefs::parse("");
    assert_eq!(prefs, Prefs::default());
    assert!(notes.is_empty());
}

/// ★ Every key the writer emits is a key the parser knows.
///
/// The drift this catches is the one that would be silent in both
/// directions: a key added to [`Prefs::write_to_string`] and not to
/// [`Prefs::parse`] makes pdfcer report its **own** file as containing an
/// unknown key, on every start, forever — and the operator would have no
/// way to tell that the file they never edited was written by the program
/// complaining about it.
///
/// The round-trip test above cannot see this: it compares the parsed struct
/// and would pass on a key that was written, unread and defaulted back to
/// the same value.
#[test]
fn the_writer_emits_no_key_the_parser_rejects() {
    // A non-default in every field, so no emitted value can coincide with
    // what a failed parse would have left behind.
    let prefs = Prefs {
        // ★ Non-default, for this test's stated reason. O70.
        smart_select: false,
        // ★ …and O96, which also ships `true`.
        shade_form_fields: false,
        // ★ Non-default, and the only OPTIONAL key in the file. O80.
        default_page_display: Some(crate::viewer::PageDisplay::Facing),
        // ★ Non-default: the Acrobat order, so a writer emitting a constant
        // token would fail here rather than pass.
        paste_chords: PasteChords::AcrobatOrder,
        // ★ Non-default, like every field here — and this one is the only
        // REPEATED key in the file, so it is the only field whose writer
        // emits a variable number of lines. Two entries rather than one,
        // so a writer that emitted only the first would fail here.
        use_os_fonts: true,
        font_folders: vec![
            std::path::PathBuf::from("C:/Fonts"),
            std::path::PathBuf::from("D:/More Fonts"),
        ],
        // Non-default, for the reason this test states about every field.
        chosen_standard: Some("pdf-x4".to_owned()),
        // ★ Non-default and with a space in it — O122. A path is the one
        // value in this file most likely to contain the character that
        // breaks a naive writer.
        acrobat_path: r"D:\Apps\Acrobat DC\Acrobat.exe".to_owned(),
        // ★ Non-default, with a space and a non-ASCII character. The file
        // is UTF-8 and a name is the one field an operator will put an
        // accent in; a writer or reader that mangled it would put mojibake
        // into every comment they sign.
        author_name: "Ken Mantlé".to_owned(),
        // Non-default, for the reason stated below about every other field.
        render_quality: RenderQuality::Sharper,
        // ★ Not the default, deliberately, and this test's own comment says
        // why: "a non-default in every field, so no emitted value can
        // coincide with what a failed parse would have left behind". A
        // `PageCache::Large` here would pass on a build whose writer emitted
        // no `page_cache` key at all.
        page_cache: PageCache::Maximum,
        zoom_settle_ms: 400,
        max_zoom_percent: 25_000.0,
        opening_fit: OpeningFit::ActualSize,
        wheel_paging: WheelPaging::FlipPages,
        chrome: PageChrome {
            rulers: true,
            grid: true,
            guides: true,
        },
        ui_scale: 1.65,
    };
    let (_, notes) = Prefs::parse(&prefs.write_to_string());
    assert!(
        notes.is_empty(),
        "pdfcer's own preferences file does not read cleanly: {notes:?}"
    );
}

/// The preferences file sits beside the settings file.
///
/// Asserted against `pdfcer-core`'s own answer rather than by re-deriving a
/// path, so the two cannot drift — which is the failure this project
/// already found once, when two callers in one process disagreed about
/// which home was live.
#[test]
fn the_preferences_file_lives_beside_the_settings_file() {
    let store = pdfcer_core::settings::resolve_store();
    let (Some(settings), Some(prefs)) = (store.path.as_deref(), Prefs::path()) else {
        // No writable location on this machine — the session still runs,
        // and there is nothing to compare. Not a failure.
        return;
    };
    assert_eq!(settings.parent(), prefs.parent());
}

// ---- O80: the standing page-display preference ------------------------

/// ★★★ **An absent key means "he has not said", and the writer must not
/// invent one** — `OPERATOR_REQUESTS.md` O80.
///
/// The whole reason `default_page_display` is an `Option` rather than a
/// `PageDisplay`. `None` has to be expressible on disk, because
/// `MODES_AND_PANELS.md`'s per-mode rule — Read opens continuous — must keep
/// applying to an operator who has never stated a preference. A writer that
/// emitted `default_page_display = single` for `None` would silently override
/// that rule for everybody, and it would do it the first time anybody's
/// preferences file was rewritten for an unrelated reason.
#[test]
fn an_unstated_page_display_preference_writes_no_key_and_reads_back_as_unstated() {
    let mut prefs = Prefs::default();
    assert_eq!(
        prefs.default_page_display, None,
        "a fresh profile has not stated one"
    );

    let text = prefs.write_to_string();
    assert!(
        !text.contains("default_page_display ="),
        "the writer must emit no VALUE for an unstated preference:
{text}"
    );
    // …but it must still document that the setting exists, or a hand-editable
    // file hides half of what it can carry.
    assert!(
        text.contains("default_page_display"),
        "the file must still name the setting in its comments:
{text}"
    );

    let (back, _) = Prefs::parse(&text);
    assert_eq!(
        back.default_page_display, None,
        "and it round-trips as unstated"
    );

    // …and once stated, it is written and read back.
    prefs.default_page_display = Some(crate::viewer::PageDisplay::Continuous);
    let text = prefs.write_to_string();
    assert!(text.contains("default_page_display = continuous"), "{text}");
    let (back, _) = Prefs::parse(&text);
    assert_eq!(
        back.default_page_display,
        Some(crate::viewer::PageDisplay::Continuous)
    );
}

/// ★★ **The three tiers resolve in the order the design states.**
///
/// | tier | wins over |
/// |---|---|
/// | this document's own remembered arrangement | everything |
/// | his standing preference | the per-mode default |
/// | the per-mode default | nothing |
///
/// Asserted as the expression `lifecycle` actually evaluates, because the
/// order is the whole design and a reversed `or` would compile, run, and
/// silently make a global preference override a document the operator had
/// deliberately arranged.
#[test]
fn a_remembered_document_outranks_the_standing_preference_which_outranks_the_mode() {
    use crate::viewer::PageDisplay;
    let resolve = |remembered: Option<PageDisplay>, standing: Option<PageDisplay>, mode: &str| {
        remembered
            .or(standing)
            .unwrap_or_else(|| PageDisplay::default_for_mode(mode))
    };

    assert_eq!(
        resolve(Some(PageDisplay::Facing), Some(PageDisplay::Single), "read"),
        PageDisplay::Facing,
        "a document he has arranged keeps its arrangement"
    );
    assert_eq!(
        resolve(None, Some(PageDisplay::Single), "read"),
        PageDisplay::Single,
        "a document he has NOT arranged takes his standing preference, even in Read"
    );
    assert_eq!(
        resolve(None, None, "read"),
        PageDisplay::Continuous,
        "and an operator who has stated nothing keeps the mode rule"
    );
    assert_eq!(
        resolve(None, None, "edit"),
        PageDisplay::Single,
        "…which is single everywhere but Read"
    );
}
