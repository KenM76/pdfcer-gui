//! # `prefs::tests` — the preferences record's own assertions
//!
//! **The inner `#![cfg(test)]` is load-bearing and is not a duplicate of
//! the outer `#[cfg(test)] mod tests;`.** Without it, `tools/gates/check-ui-strings.sh`
//! walks this file as ordinary source and reports every assertion message as a
//! user-visible string that should live in `ui_text` — exclusion 2b in that
//! gate. It is the same line every other split test file in this crate carries.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/prefs/tests.md`.
#![cfg(test)]

use super::*;
// O166's group stores the print dialog's OWN types rather than a mirrored set
// -- see `prefs::printing`'s header on why -- so the tests that build a
// non-default `PrintPrefs` name them at their real home.
use crate::printspooler::{Duplex, Orientation, PageSubset, PaperChoice, ScaleMode};
// O196's three groups store the export windows' OWN types, identically, so the
// tests that build a non-default `ExportPrefs` name them at their real homes --
// two of which are the engine's, because a DXF's units and its text policy are
// the engine's vocabulary and mirroring them here is what `exporting`'s header
// forbids.
use crate::exporttext::{LineEndings, PageSeparator};
use crate::imageexport::{ImageFormat, PageScope};
use crate::viewer::{FitMode, ViewState};
use pdfcer_core::export::dxf::{DxfText, DxfUnits};

/// Every value round-trips through the file.
#[test]
fn every_preference_round_trips_through_the_file() {
    for quality in RenderQuality::ALL {
        for fit in OpeningFit::ALL {
            for reach in RedactionReach::ALL {
                let original = Prefs {
                    ocr_models: {
                        let mut o = crate::prefs::OcrModelPrefs::default();
                        o.add_folder(std::path::Path::new("E:/extra models"));
                        o.model = Some("paddle-vl".to_owned());
                        o
                    },
                    snapshot: {
                        let mut s = crate::prefs::SnapshotPrefs::default();
                        s.set_dpi(150);
                        s
                    },
                    shortcuts: {
                        let mut s = crate::prefs::ShortcutPrefs::default();
                        s.set("edit.find", &["Ctrl+K".to_owned()]);
                        s.set("format.bold", &[]);
                        s
                    },
                    // Non-default, like every field here. O70: `false`, because
                    // the shipped default is `true` and a writer that emitted a
                    // constant would otherwise pass.
                    smart_select: false,
                    text_chunks: false,
                    // Non-default, identically. O163 ships `true`, so a
                    // writer emitting a constant would round-trip a preference
                    // the operator had turned OFF without this test noticing.
                    find_zoom_on_jump: false,
                    // Non-default, identically. O180 ships `true`.
                    find_trim_query: false,
                    page_previews: false,
                    // Non-default, and specifically NOT `0`: O225 ships
                    // *no limit*, which is spelled `0`, so a writer emitting a
                    // constant zero would round-trip an operator's chosen time
                    // limit into no limit at all and pass.
                    page_preview_budget_ms: 2000,
                    ribbon_auto_hide: false,
                    rail_auto_hide: false,
                    // Non-default for the identical reason — O96 ships `true`,
                    // so a writer emitting a constant `true` would round-trip a
                    // preference the operator had turned OFF and this test would
                    // not notice.
                    shade_form_fields: false,
                    // Non-default, and with no repeated byte and no digit
                    // that is its own nibble doubled — O229. A writer that
                    // emitted the shorthand notation, or a parser that shifted
                    // a nibble instead of doubling it, both round-trip
                    // `#CC0099` and neither round-trips this.
                    ocr_layer_colour: [1, 130, 255],
                    // Non-default, and OPTIONAL: a writer that omitted it fails here.
                    ocr_engine: Some(crate::ocr::EngineId::Ocrcer),
                    sign_timestamp_server: Some("http://tsa.example/rfc3161".to_owned()),
                    // Non-default, like every field here — and this one is
                    // the only OPTIONAL key in the file, so a writer that emitted
                    // nothing for it would fail this round trip. `Facing` rather
                    // than `Single` so it cannot coincide with the compiled-in
                    // per-mode answer either. O80.
                    default_page_display: Some(crate::viewer::PageDisplay::Facing),
                    font_folders: vec![std::path::PathBuf::from("C:/Fonts")],
                    use_os_fonts: true,
                    // Non-default, like every field here, and with a SPACE in
                    // it — a path a person would really type on Windows. O122.
                    // Non-default, like every field here. O173 ships `true`, so a
                    // writer that emitted a constant would round-trip the offer as
                    // still-wanted after the operator had ticked Do not ask again -
                    // which is the one failure this preference exists to prevent.
                    ask_default_app: false,
                    acrobat_path: r"D:\Apps\Acrobat DC\Acrobat.exe".to_owned(),
                    // Non-default, and deliberately a DIFFERENT string from the
                    // field above: two paths that happened to be equal would pass on a
                    // writer that emitted one of them twice.
                    acrobat_trust_store_path: r"D:\Certs\addressbook.acrodata".to_owned(),
                    // Non-default, like every other field here: a `None`
                    // would pass on a build whose writer emitted no
                    // `chosen_standard` key at all.
                    chosen_standard: Some("pdf-x1a".to_owned()),
                    // Non-default, and with a SPACE in it: the writer emits
                    // the value raw and the parser trims, so a name of one word
                    // would pass on a reader that split on whitespace.
                    author_name: "Ken Mantle".to_owned(),
                    // The migration marker round-trips like every other key.
                    // `true` rather than the default `false`, per this test's
                    // own rule: a non-default in every field, so no emitted
                    // value can coincide with what a failed parse left behind.
                    render_quality: *quality,
                    // Swept rather than pinned, like the two enums above, and
                    // this is the one where a constant token would be worst: the
                    // three values differ in what a redaction DESTROYS, so a
                    // writer that emitted `hidden-carriers` whatever the operator
                    // chose would silently return the widest choice to the middle
                    // and the narrowest to the middle too.
                    redaction_reach: *reach,
                    page_cache: PageCache::default(),
                    zoom_settle_ms: 275,
                    // A non-default well past the shipped ceiling, so the
                    // round trip is proved on a value that MATTERS rather
                    // than on 800.
                    max_zoom_percent: 1_000_000.0,
                    opening_fit: *fit,
                    // The non-default, so a writer that emitted no
                    // `wheel_paging` key at all would fail here rather than
                    // pass by landing back on `Scroll`.
                    paste_chords: PasteChords::AcrobatOrder,
                    wheel_paging: WheelPaging::FlipPages,
                    remote_control: RemoteControl::Never,
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
                    // Non-default: O265 ships `true`.
                    colour_icons: false,
                    // O166's thirteen keys, every one non-default, per this
                    // test's own rule. This is the only group whose parser and
                    // writer live outside `prefs::file` (they are in
                    // `prefs::printing`, together), so this assertion is the one
                    // that proves the delegation reaches both halves.
                    print: PrintPrefs {
                        // A name with spaces AND parentheses — what a real second
                        // plotter is actually called on a Windows machine, and the
                        // string most likely to break a naive writer.
                        printer: Some("HP DesignJet T1600 (Copy 2)".to_owned()),
                        orientation: Orientation::Landscape,
                        duplex: Duplex::ShortEdge,
                        pick_tray_by_page_size: true,
                        paper: PaperChoice::AutoFromPages,
                        // `Custom(1.0)` and not `Custom(2.5)`, deliberately, and
                        // this is the one place in this test where a "more
                        // non-default" value would be WRONG. `ScaleMode::Custom`'s
                        // payload is not persisted — the percentage has its own
                        // key and the payload is re-derived from it at the point
                        // of use — so `1.0` is what `scale_from_key("custom")`
                        // returns and any other payload would fail this round trip
                        // for a duplication the design deliberately avoids. The
                        // MODE is still non-default; the shipped one is `Fit`.
                        scale: ScaleMode::Custom(1.0),
                        // The number the payload above would be derived from, and
                        // deliberately not 100: a writer emitting a constant would
                        // otherwise pass.
                        custom_percent: 250,
                        scope: pdfcer_render::AnnotationScope::DocumentAndMarkups,
                        max_dpi: 600,
                        copies: 3,
                        // Non-default, and it is the field that proves the
                        // INVERSION: the file says `print_collate` and the struct
                        // holds `uncollated`, so a writer and parser that inverted
                        // differently would land back on `false` here.
                        uncollated: true,
                        subset: PageSubset::Even,
                        reverse: true,
                        // Every poster field off its default.
                        poster: crate::prefs::printing::PosterPrefs {
                            on: true,
                            tile_percent: 275,
                            overlap_mm: 12.5,
                            cut_marks: true,
                            labels: true,
                            large_only: true,
                        },
                        // Every line-width field off its default.
                        lines: crate::prefs::printing::LineWidthPrefs {
                            fixed: true,
                            auto: false,
                            width_mm: 0.35,
                        },
                    },
                    // Off-page display, and every one of these three is
                    // the OPPOSITE of what that mode ships — Read ships off,
                    // Review and Edit ship on. That is not decoration: this
                    // group stores only the answers actually GIVEN, so a
                    // writer that skipped an answer equal to its default, or a
                    // parser that dropped one, would still round-trip cleanly
                    // if the values here agreed with the defaults. Inverted,
                    // either bug lands on the wrong bool and fails.
                    //
                    // The fourth key is a mode this build does not ship. The
                    // family is a PREFIX rather than three fixed keys, because
                    // ribbon modes come from the manifest and an operator may
                    // customize it; a parser that closed over the three shipped
                    // modes would silently drop this line and fail here, which
                    // is the whole point of it being present.
                    off_page: {
                        let mut p = OffPagePrefs::default();
                        p.set("read", true);
                        p.set("review", false);
                        p.set("edit", false);
                        p.set("proof", true);
                        p
                    },
                    // O196's twelve keys, every one non-default, per this
                    // test's own rule. The second group whose parser and writer
                    // live outside `prefs::file` -- in `prefs::exporting`, together
                    // -- so this assertion is what proves the THIRD link of the
                    // delegation chain reaches both halves.
                    //
                    // The two page scopes are set to OPPOSITE values, and that
                    // is the pair worth looking at: the image window ships
                    // `CurrentPage` and the text window ships `AllPages`, so a
                    // writer or parser that collapsed them onto one key would land
                    // each group on the other's default and still look plausible.
                    // Crossed over, it fails.
                    export: ExportPrefs {
                        image: ExportImagePrefs {
                            format: ImageFormat::Emf,
                            scope: PageScope::AllPages,
                            // Off the hundred, so a writer that rounded or
                            // truncated to an integer would land on 601 or 600.
                            dpi: 601.5,
                            transparent: false,
                            quality: 55,
                            keep_text: true,
                        },
                        text: ExportTextPrefs {
                            scope: PageScope::CurrentPage,
                            separator: PageSeparator::Marker,
                            order: crate::exporttext::TextOrder::Reading,
                            line_endings: LineEndings::Windows,
                            byte_order_mark: true,
                        },
                        tables: ExportTablePrefs {
                            scope: PageScope::CurrentPage,
                            format: crate::tableexport::TableFormat::Xlsx,
                        },
                        dxf: ExportDxfPrefs {
                            units: DxfUnits::Millimetres,
                            fit_arcs: false,
                            text: DxfText::Omit,
                            version: pdfcer_core::export::dxf::DxfVersion::R2004,
                        },
                    },
                };
                let (read_back, notes) = Prefs::parse(&original.write_to_string());
                assert!(
                    notes.is_empty(),
                    "a written file did not read cleanly: {notes:?}"
                );
                assert_eq!(
                    read_back, original,
                    "{quality:?}/{fit:?}/{reach:?} did not survive the trip"
                );
            }
        }
    }
}

/// The three overlays are three independent keys.
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
#[test]
fn the_defaults_are_the_constants_they_replaced() {
    let prefs = Prefs::default();
    assert_eq!(prefs.zoom_settle_ms, 150);
    assert!((prefs.render_quality.multiplier() - 1.0).abs() < f32::EPSILON);
    assert_eq!(prefs.opening_fit, OpeningFit::Page);
    assert!(prefs.chrome.all_hidden());
}

/// **The shipped preferences change nothing about a freshly opened view.**
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

/// **A document's remembered guides survive a preference that hides them.**
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

/// One bad line never discards the rest of the file.
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

/// **A trillion percent is accepted**, which is the figure the
/// operator named — `OPERATOR_REQUESTS.md` O24.
#[test]
fn a_trillion_percent_is_accepted_and_the_page_actually_draws_there() {
    let (prefs, notes) = Prefs::parse(
        "max_zoom_percent = 1000000000000
",
    );
    assert!((prefs.max_zoom_percent - 1e12).abs() / 1e12 < 1e-6);
    assert!(
        notes.is_empty(),
        "a stated maximum the shell can honour must not be second-guessed"
    );
}

/// **A non-finite value is refused, not clamped.**
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

/// **The file says a whole number, not `1e12`.**
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

/// **A UI scale of `nan` or `inf` is refused, not clamped.**
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
#[test]
fn an_empty_file_produces_defaults_and_no_notes() {
    let (prefs, notes) = Prefs::parse("");
    assert_eq!(prefs, Prefs::default());
    assert!(notes.is_empty());
}

/// Every key the writer emits is a key the parser knows.
#[test]
fn the_writer_emits_no_key_the_parser_rejects() {
    // A non-default in every field, so no emitted value can coincide with
    // what a failed parse would have left behind.
    let prefs = Prefs {
        ocr_models: {
            let mut o = crate::prefs::OcrModelPrefs::default();
            o.add_folder(std::path::Path::new("E:/extra models"));
            o.model = Some("paddle-vl".to_owned());
            o
        },
        snapshot: {
            let mut s = crate::prefs::SnapshotPrefs::default();
            s.set_dpi(150);
            s
        },
        shortcuts: {
            let mut s = crate::prefs::ShortcutPrefs::default();
            s.set("edit.find", &["Ctrl+K".to_owned()]);
            s.set("format.bold", &[]);
            s
        },
        // Non-default, for this test's stated reason. O70.
        text_chunks: false,
        smart_select: false,
        // …and O163, which also ships `true`.
        find_zoom_on_jump: false,
        // …and O180, which also ships `true`.
        find_trim_query: false,
        page_previews: false,
        // Non-default, for this test's stated reason — and specifically
        // not `0`, which O225 ships.
        page_preview_budget_ms: 2000,
        ribbon_auto_hide: false,
        rail_auto_hide: false,
        // …and O96, which also ships `true`.
        shade_form_fields: false,
        // Non-default, for this test's stated reason. O229.
        ocr_layer_colour: [1, 130, 255],
        ocr_engine: Some(crate::ocr::EngineId::Ocrcer),
        sign_timestamp_server: Some("http://tsa.example/rfc3161".to_owned()),
        // Non-default, and the only OPTIONAL key in the file. O80.
        default_page_display: Some(crate::viewer::PageDisplay::Facing),
        // Non-default: the Acrobat order, so a writer emitting a constant
        // token would fail here rather than pass.
        paste_chords: PasteChords::AcrobatOrder,
        // Non-default, like every field here — and this one is the only
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
        // Non-default and with a space in it — O122. A path is the one
        // value in this file most likely to contain the character that
        // breaks a naive writer.
        // Non-default, for the reason this test states about every field, and
        // in the direction that matters: O173 ships `true`.
        ask_default_app: false,
        acrobat_path: r"D:\Apps\Acrobat DC\Acrobat.exe".to_owned(),
        // Non-default, and deliberately a DIFFERENT string from the
        // field above: two paths that happened to be equal would pass on a
        // writer that emitted one of them twice.
        acrobat_trust_store_path: r"D:\Certs\addressbook.acrodata".to_owned(),
        // Non-default, with a space and a non-ASCII character. The file
        // is UTF-8 and a name is the one field an operator will put an
        // accent in; a writer or reader that mangled it would put mojibake
        // into every comment they sign.
        author_name: "Ken Mantlé".to_owned(),
        // Non-default, for the reason stated below about every other field.
        render_quality: RenderQuality::Sharper,
        // Non-default, for this test's stated reason, and `WholeDocument`
        // rather than `MarkedOnly` because it is the value whose loss would
        // be silent in the dangerous direction: a build that failed to write
        // it would leave the operator at the middle, which still edits.
        redaction_reach: RedactionReach::WholeDocument,
        // Not the default, deliberately, and this test's own comment says
        // why: "a non-default in every field, so no emitted value can
        // coincide with what a failed parse would have left behind". A
        // `PageCache::Large` here would pass on a build whose writer emitted
        // no `page_cache` key at all.
        page_cache: PageCache::Maximum,
        zoom_settle_ms: 400,
        max_zoom_percent: 25_000.0,
        opening_fit: OpeningFit::ActualSize,
        wheel_paging: WheelPaging::FlipPages,
        remote_control: RemoteControl::Always,
        chrome: PageChrome {
            rulers: true,
            grid: true,
            guides: true,
        },
        ui_scale: 1.65,
        colour_icons: true,
        // O166. Deliberately a DIFFERENT set of values from the round-trip
        // test above — a shared constant would make both tests depend on one
        // combination, and this one is asking a different question: does every
        // token this writer can emit parse back cleanly? So the enums are the
        // arms the other test does not use, and the numbers are at the ENDS of
        // their ranges, which is where a writer's formatting breaks.
        print: PrintPrefs {
            // A name with a non-ASCII character. The file is UTF-8 and a
            // printer's name is set by whoever installed it; a writer or reader
            // that mangled it would silently print to the Windows default
            // forever after, with nothing on screen to say so.
            printer: Some("Konica Minolta bizhub — Atelier".to_owned()),
            orientation: Orientation::Portrait,
            duplex: Duplex::LongEdge,
            pick_tray_by_page_size: true,
            paper: PaperChoice::AutoFromPages,
            scale: ScaleMode::ShrinkOversized,
            // The top of the dialog's own `DragValue` range.
            custom_percent: 1_000,
            scope: pdfcer_render::AnnotationScope::FormFieldsOnly,
            // The top of the range the file accepts. A number this large is
            // exactly where a writer that formatted through an `f32` would
            // start emitting something its own parser could not read.
            max_dpi: 2_400,
            copies: 999,
            uncollated: true,
            subset: PageSubset::Odd,
            reverse: true,
            // The ends of the ranges, where a writer's formatting breaks.
            poster: crate::prefs::printing::PosterPrefs {
                on: true,
                tile_percent: 5_000,
                overlap_mm: 0.1,
                cut_marks: false,
                labels: true,
                large_only: false,
            },
            lines: crate::prefs::printing::LineWidthPrefs {
                fixed: true,
                auto: true,
                width_mm: 5.0,
            },
        },
        // Off-page display. A DIFFERENT set from the round-trip test
        // above, per this test's own rule, and one of the mode ids carries
        // a SPACE — which a manifest's mode id legitimately may, and which
        // is the character most likely to break a writer or a parser that
        // splits a line by whitespace rather than at the first `=`. The
        // round trip above would not see that: it compares structs, and a
        // key that failed to parse would be reported in `notes`, which is
        // exactly what this test asserts is empty.
        off_page: {
            let mut p = OffPagePrefs::default();
            p.set("read", true);
            p.set("drawing review", false);
            p
        },
        // O196. A DIFFERENT set from the round-trip test above, per this
        // test's own rule, and chosen for what it asks: does every token this
        // writer can emit parse back cleanly? So the enums are the arms the
        // other test does not use, and the numbers sit at the ENDS of their
        // ranges -- which is where a writer's formatting breaks.
        export: ExportPrefs {
            image: ExportImagePrefs {
                // The one format whose token the other test does not emit.
                format: ImageFormat::Svg,
                scope: PageScope::CurrentPage,
                // The very top of the control's range. A resolution this
                // large is exactly where a writer that formatted through an
                // exponent would start emitting `4.8e3`, which this module's
                // own parser reads fine -- and which the file's comment block
                // does not describe, so a hand-editor would meet a spelling
                // pdfcer taught them nowhere.
                dpi: MAX_EXPORT_DPI,
                transparent: false,
                // The bottom of the range, so both ends are exercised across
                // the two numeric keys.
                quality: MIN_JPEG_QUALITY,
                keep_text: true,
            },
            text: ExportTextPrefs {
                scope: PageScope::AllPages,
                separator: PageSeparator::FormFeed,
                order: crate::exporttext::TextOrder::AsDrawn,
                line_endings: LineEndings::AsExtracted,
                byte_order_mark: true,
            },
            tables: ExportTablePrefs {
                scope: PageScope::AllPages,
                format: crate::tableexport::TableFormat::Ods,
            },
            dxf: ExportDxfPrefs {
                units: DxfUnits::Inches,
                fit_arcs: true,
                text: DxfText::Entities,
                version: pdfcer_core::export::dxf::DxfVersion::R2000,
            },
        },
    };
    let (_, notes) = Prefs::parse(&prefs.write_to_string());
    assert!(
        notes.is_empty(),
        "pdfcer's own preferences file does not read cleanly: {notes:?}"
    );
}

/// The preferences file sits beside the settings file.
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

/// **An absent key means "he has not said", and the writer must not
/// invent one** — `OPERATOR_REQUESTS.md` O80.
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

/// **The three tiers resolve in the order the design states.**
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
