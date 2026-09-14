//! Tests for [`super`] — the three export windows' remembered settings.
//!
//! Four properties, in the order they matter:
//!
//! 1. **The defaults are what the windows hard-coded before O196.** A fresh
//!    `userdata` folder must behave exactly like every previous build. Deleting
//!    `preferences.txt` is a way to reset pdfcer, never a way to change it.
//! 2. **Every token round-trips, over every variant.** A writer that emits a
//!    token its own parser rejects turns the operator's settings into a
//!    `BadValue` note on the next launch — which reads as pdfcer forgetting
//!    them, the exact complaint this module answers.
//! 3. **Every field is written AND parsed.** Source-text completeness, the same
//!    instrument [`crate::app::prefs::printing`] uses, for the same reason: a
//!    field added to a struct and to nothing else is a preference that is
//!    silently inert, and no compiler diagnoses it.
//! 4. **Out-of-range clamps; unreadable is reported.** The ruling inherited
//!    from `printing`, checked at both ends of both numeric keys.
//!
//! ★★ **The inner `#![cfg(test)]` is load-bearing and is not a duplicate of
//! the outer `#[cfg(test)] mod tests;` in `exporting.rs`.** Without it,
//! `tools/gates/check-ui-strings.sh` walks this file as ordinary source and
//! reports every assertion message as a user-visible string that should live
//! in `ui_text` — exclusion 2b in that gate. It is the same line every
//! other split test file in this crate carries.
#![cfg(test)]

use super::*;

// ---------------------------------------------------------------------------
// The exhaustive variant lists these tests sweep
// ---------------------------------------------------------------------------

/// Every [`ImageFormat`], written out here rather than taken from the type.
///
/// ★ Deliberately **not** `ImageFormat::ALL`. A round-trip test that draws its
/// input from the same accessor the production code uses proves the two agree
/// with each other; it does not prove either agrees with the file format. This
/// list is the test's own statement of what the file must carry, so a variant
/// added upstream goes red here — at the `match` in [`image_format_key`] — with
/// a message about the file rather than silently expanding a sweep.
const IMAGE_FORMATS: [ImageFormat; 4] = [
    ImageFormat::Png,
    ImageFormat::Jpeg,
    ImageFormat::Svg,
    ImageFormat::Emf,
];

/// The two [`PageScope`] variants that have a token. [`PageScope::Typed`]
/// deliberately has none; see [`page_scope_key`].
const SCOPES: [PageScope; 2] = [PageScope::CurrentPage, PageScope::AllPages];

/// Every [`PageSeparator`].
const SEPARATORS: [PageSeparator; 2] = [PageSeparator::FormFeed, PageSeparator::Marker];

/// Every [`LineEndings`].
const LINE_ENDINGS: [LineEndings; 2] = [LineEndings::AsExtracted, LineEndings::Windows];

/// Every [`DxfUnits`].
const DXF_UNITS: [DxfUnits; 2] = [DxfUnits::Inches, DxfUnits::Millimetres];

/// Every [`DxfText`].
const DXF_TEXTS: [DxfText; 2] = [DxfText::Entities, DxfText::Omit];

// ---------------------------------------------------------------------------
// 1. The defaults
// ---------------------------------------------------------------------------

/// ★ The Export-image window opens exactly as it did before O196.
#[test]
fn the_image_default_is_what_the_dialog_used_to_hard_code() {
    let prefs = ExportImagePrefs::default();
    assert_eq!(prefs.format, ImageFormat::Png);
    assert_eq!(prefs.scope, PageScope::CurrentPage);
    assert!((prefs.dpi - 300.0).abs() < f32::EPSILON);
    assert!(prefs.transparent);
    assert_eq!(prefs.quality, 90);
}

/// ★ The Export-text window opens exactly as it did before O196.
///
/// The `AllPages` line is the one worth looking at: it disagrees with the image
/// window's `CurrentPage` on purpose, and that disagreement is why the two
/// scopes are separate keys.
#[test]
fn the_text_default_is_what_the_dialog_used_to_hard_code() {
    let prefs = ExportTextPrefs::default();
    assert_eq!(prefs.scope, PageScope::AllPages);
    assert_eq!(prefs.separator, PageSeparator::FormFeed);
    assert_eq!(prefs.line_endings, LineEndings::AsExtracted);
    assert!(!prefs.byte_order_mark);
}

/// ★ The Export-to-DXF window opens exactly as it did before O196.
#[test]
fn the_dxf_default_is_what_the_dialog_used_to_hard_code() {
    let prefs = ExportDxfPrefs::default();
    assert_eq!(prefs.units, DxfUnits::Inches);
    assert!(prefs.fit_arcs);
    assert_eq!(prefs.text, DxfText::Entities);
}

/// ★★ Our DXF defaults are still the engine's.
///
/// [`ExportDxfPrefs::default`] writes literals rather than delegating to
/// `DxfOptions::default()`, so that an engine change to a default cannot alter
/// this window's behaviour on a `cargo update` with nothing in the diff. This
/// test is the other half of that decision: the day the engine does change one,
/// **this goes red** and somebody decides, in a commit, whether the window
/// follows.
///
/// Without it the literals would be a silent fork rather than a deliberate one.
#[test]
fn the_dxf_defaults_still_match_the_engine_and_a_change_upstream_lands_here() {
    let engine = pdfcer_core::export::dxf::DxfOptions::default();
    let ours = ExportDxfPrefs::default();
    assert_eq!(
        ours.units, engine.units,
        "★ the engine changed its default DXF units. `ExportDxfPrefs::default` \
         holds a literal on purpose, so this is a decision to make in a commit \
         rather than a behaviour change to absorb: either follow the engine \
         here, or write down why this window does not."
    );
    assert_eq!(
        ours.fit_arcs, engine.fit_arcs,
        "★ the engine changed its default for arc fitting; see above."
    );
    assert_eq!(
        ours.text, engine.text,
        "★ the engine changed its default for DXF text; see above."
    );
}

// ---------------------------------------------------------------------------
// 2. The tokens
// ---------------------------------------------------------------------------

/// Every token pair round-trips over every variant, and no two variants of one
/// enum share a token.
///
/// The second half matters as much as the first: two values spelled the same
/// way makes one of them unreachable from a hand-edited file, and the writer
/// would keep producing a file the parser reads as the *other* value with no
/// note raised anywhere.
#[test]
fn every_token_round_trips_and_is_distinct_within_its_enum() {
    /// Round-trip and distinctness for one enum, given its writer and reader.
    macro_rules! sweep {
        ($list:expr, $to:expr, $from:expr, $name:literal) => {{
            let mut seen: Vec<&'static str> = Vec::new();
            for value in $list {
                let token = $to(value);
                assert_eq!(
                    $from(token),
                    Some(value),
                    "{}: `{}` does not read back as the value that wrote it",
                    $name,
                    token
                );
                assert!(
                    !seen.contains(&token),
                    "{}: two variants share the token `{}`, which makes one of \
                     them unreachable from a hand-edited preferences file",
                    $name,
                    token
                );
                seen.push(token);
            }
        }};
    }

    sweep!(
        IMAGE_FORMATS,
        image_format_key,
        image_format_from_key,
        "ImageFormat"
    );
    sweep!(
        SEPARATORS,
        separator_key,
        separator_from_key,
        "PageSeparator"
    );
    sweep!(
        LINE_ENDINGS,
        line_endings_key,
        line_endings_from_key,
        "LineEndings"
    );
    sweep!(DXF_UNITS, dxf_units_key, dxf_units_from_key, "DxfUnits");
    sweep!(DXF_TEXTS, dxf_text_key, dxf_text_from_key, "DxfText");

    // `page_scope_key` returns an Option, so it does not fit the macro.
    let mut seen: Vec<&'static str> = Vec::new();
    for scope in SCOPES {
        let token = page_scope_key(scope).expect("both sweepable scopes have a token");
        assert_eq!(page_scope_from_key(token), Some(scope));
        assert!(!seen.contains(&token), "PageScope: duplicate token {token}");
        seen.push(token);
    }
}

/// ★★ [`PageScope::Typed`] has no token, and degrades to the caller's own
/// default rather than to a third behaviour.
///
/// The failure this prevents is specific and invisible from the code: a window
/// restored into *Pages* with an empty range box, its Export button greyed, and
/// nothing on screen saying why.
#[test]
fn typed_has_no_token_and_degrades_to_the_windows_own_default() {
    assert_eq!(page_scope_key(PageScope::Typed), None);

    // Each window's own fallback, which is the whole point of the `_or` form:
    // the reduction lands where that window would have opened anyway.
    assert_eq!(
        page_scope_key_or(PageScope::Typed, ExportImagePrefs::default().scope),
        "current"
    );
    assert_eq!(
        page_scope_key_or(PageScope::Typed, ExportTextPrefs::default().scope),
        "all"
    );

    // A scope that HAS a token is never touched by the fallback.
    assert_eq!(
        page_scope_key_or(PageScope::AllPages, PageScope::CurrentPage),
        "all"
    );
}

/// The two American spellings of `millimetres` are accepted on read.
///
/// The writer emits exactly one spelling — asserted here too, because a writer
/// that started emitting a synonym would make the file teach a vocabulary its
/// own comment block contradicts.
#[test]
fn the_dxf_units_synonyms_are_read_but_never_written() {
    assert_eq!(
        dxf_units_from_key("millimeters"),
        Some(DxfUnits::Millimetres)
    );
    assert_eq!(dxf_units_from_key("mm"), Some(DxfUnits::Millimetres));
    assert_eq!(dxf_units_key(DxfUnits::Millimetres), "millimetres");

    // And no synonym anywhere else: `jpg` is not `jpeg`.
    assert_eq!(image_format_from_key("jpg"), None);
}

// ---------------------------------------------------------------------------
// 3. Completeness — every field is written and parsed
// ---------------------------------------------------------------------------

/// Parse one `pub name: Type,` struct body out of this module's own source.
///
/// Returns the field names in declaration order. The shape it depends on —
/// every field on one line, `pub`, no trailing comment — is the same shape
/// [`crate::app::prefs::printing`]'s equivalent depends on, and it is stated in
/// that struct's doc comment for the same reason.
fn fields_of(source: &str, decl: &str) -> Vec<String> {
    let (_, after) = source
        .split_once(decl)
        .unwrap_or_else(|| panic!("the declaration `{decl}`, verbatim"));
    let (body, _) = after.split_once("\n}").expect("the end of the struct");
    body.lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("pub ")?;
            let (name, _) = rest.split_once(':')?;
            Some(name.to_string())
        })
        .collect()
}

/// ★★★ Every field of all three groups is both written to the file and read
/// back out of it.
///
/// # Why a source-text check rather than a value round trip
///
/// A round trip over [`ExportPrefs`] — write, parse, compare — is also in this
/// file and is the stronger check *for the fields it covers*. It cannot cover a
/// field nobody wrote: a new field simply keeps its default on both sides of the
/// comparison and the round trip stays green while the preference is inert.
///
/// This test closes that hole from the other direction. It proves only that the
/// field is *mentioned* in both halves, which is weak — and is exactly the
/// difference between a preference that is wired up and one that is not.
///
/// ⚠ It is blind in one direction by construction: it cannot see a field
/// mentioned in the right place and used wrongly. That is what the round trip
/// below and the driven `ui-verify` check are for.
#[test]
fn every_field_of_every_group_is_both_written_and_parsed() {
    let own = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/app/prefs/exporting.rs"),
    )
    .expect("this module's own source");

    let (_, parser) = own
        .split_once("pub(super) fn parse_key(")
        .expect("the parser, verbatim");
    let (parser, _) = parser
        .split_once("pub(super) fn write_block(")
        .expect("the writer, which follows the parser");
    let (writer, _) = own
        .split_once("\n#[cfg(test)]")
        .expect("the end of the module's non-test source");
    let (_, writer) = writer
        .split_once("pub(super) fn write_block(")
        .expect("the writer, verbatim");

    let groups: [(&str, &str, usize); 3] = [
        ("pub struct ExportImagePrefs {", "image", 5),
        ("pub struct ExportTextPrefs {", "text", 4),
        ("pub struct ExportDxfPrefs {", "dxf", 3),
    ];

    for (decl, group, expected) in groups {
        let fields = fields_of(&own, decl);
        // ★ A FLOOR, not an equality, and the difference is which failure
        // each one reports. A field ADDED to the struct must be reported by
        // the loop below, as "nothing writes it" — which is the true
        // finding. An equality here would intercept it first and report
        // "the declaration's shape changed", sending the reader to the
        // parser rather than to the missing key.
        //
        // What the floor IS for is the opposite case: a parser that stopped
        // matching lines at all, which would otherwise sweep an empty list
        // and pass — the vacuity this project has been caught by before.
        assert!(
            fields.len() >= expected,
            "the struct parser found only {} field(s) in `{decl}` where at \
             least {expected} are declared — the declaration's shape \
             changed and this test has gone blind rather than red",
            fields.len()
        );

        for field in fields {
            let path = format!("prefs.{group}.{field}");
            assert!(
                writer.contains(&path),
                "★ `{path}` is a remembered preference that `write_block` never \
                 writes. It will be forgotten on every restart, silently, and \
                 the round-trip test cannot see it because an unwritten field \
                 keeps its default on both sides of the comparison."
            );
            assert!(
                parser.contains(&path),
                "★ `{path}` is written to the preferences file and never read \
                 back: `parse_key` has no arm that stores it. The key appears in \
                 the file, the operator can edit it, and nothing happens."
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 4. The round trip, and the numeric rulings
// ---------------------------------------------------------------------------

/// A set of values with **no field at its default**, so that a field the writer
/// skips cannot be rescued by the parser's default agreeing with it.
fn everything_changed() -> ExportPrefs {
    ExportPrefs {
        image: ExportImagePrefs {
            format: ImageFormat::Emf,
            scope: PageScope::AllPages,
            dpi: 600.0,
            transparent: false,
            quality: 72,
        },
        text: ExportTextPrefs {
            scope: PageScope::CurrentPage,
            separator: PageSeparator::Marker,
            line_endings: LineEndings::Windows,
            byte_order_mark: true,
        },
        dxf: ExportDxfPrefs {
            units: DxfUnits::Millimetres,
            fit_arcs: false,
            text: DxfText::Omit,
        },
    }
}

/// Read a block back the way `prefs::file` does: split on `=`, trim both halves,
/// skip comments and blanks, and hand each pair to [`parse_key`].
///
/// Returns the number of keys the parser **accepted**, so a caller can assert on
/// it; a key that fell through as [`KeyOutcome::NotMine`] is counted separately
/// and is a failure in every caller here, because everything in this block is
/// by construction ours.
fn parse_block(text: &str, into: &mut ExportPrefs) -> usize {
    let mut accepted = 0usize;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .expect("every non-comment line is a pair");
        match parse_key(into, key.trim(), value.trim()) {
            KeyOutcome::Accepted => accepted += 1,
            KeyOutcome::BadValue => panic!(
                "★ the writer emitted `{line}`, and this module's own parser \
                 rejects it. That is the failure mode this pair of functions \
                 exists to prevent: the operator's setting becomes a reported \
                 bad value on the next launch, which looks exactly like pdfcer \
                 forgetting it."
            ),
            KeyOutcome::NotMine => panic!(
                "★ the writer emitted `{line}`, which this module's parser does \
                 not claim. `prefs::file` would report it as an unknown key — \
                 telling the operator to check the spelling of a key pdfcer \
                 wrote itself."
            ),
        }
    }
    accepted
}

/// ★★ Every remembered value survives a trip through the file.
#[test]
fn every_export_preference_round_trips_through_the_file() {
    let written = everything_changed();
    let mut out = String::new();
    write_block(&written, &mut out);

    let mut read = ExportPrefs::default();
    let accepted = parse_block(&out, &mut read);

    assert_eq!(accepted, 12, "twelve keys are declared in this module");
    assert_eq!(read, written, "a value changed on its way through the file");
    assert_ne!(
        read,
        ExportPrefs::default(),
        "the test's own input must differ from the default in every field, or \
         a writer that skipped a key would still pass"
    );
}

/// The default set round-trips too, and is written unconditionally.
///
/// The second half is the point: a fresh profile must still find all twelve keys
/// in the file, with their comment blocks, because *a preference nobody can
/// discover is a preference nobody has.*
#[test]
fn the_defaults_are_written_too_so_the_file_teaches_its_own_vocabulary() {
    let mut out = String::new();
    write_block(&ExportPrefs::default(), &mut out);

    let mut read = ExportPrefs::default();
    assert_eq!(parse_block(&out, &mut read), 12);
    assert_eq!(read, ExportPrefs::default());

    // Each key's own comment block names it, so an operator reading the file
    // sees the vocabulary beside the value.
    for key in [
        "export_image_format",
        "export_image_pages",
        "export_image_dpi",
        "export_image_transparent",
        "export_image_quality",
        "export_text_pages",
        "export_text_separator",
        "export_text_line_endings",
        "export_text_byte_order_mark",
        "export_dxf_units",
        "export_dxf_fit_arcs",
        "export_dxf_text",
    ] {
        assert!(
            out.contains(&format!("# {key}:")),
            "`{key}` is written into the file with no comment block explaining \
             what may be written there"
        );
    }
}

/// ★ A number outside the control's own range is pulled back to the nearest
/// end; a value that is not a number is reported.
#[test]
fn out_of_range_numbers_clamp_and_unreadable_ones_are_reported() {
    let mut prefs = ExportPrefs::default();

    assert_eq!(
        parse_key(&mut prefs, "export_image_dpi", "99999"),
        KeyOutcome::Accepted
    );
    assert!((prefs.image.dpi - MAX_EXPORT_DPI).abs() < f32::EPSILON);

    assert_eq!(
        parse_key(&mut prefs, "export_image_dpi", "-40"),
        KeyOutcome::Accepted
    );
    assert!((prefs.image.dpi - MIN_EXPORT_DPI).abs() < f32::EPSILON);

    // A fractional resolution is legible and the control can hold it.
    assert_eq!(
        parse_key(&mut prefs, "export_image_dpi", "150.5"),
        KeyOutcome::Accepted
    );
    assert!((prefs.image.dpi - 150.5).abs() < f32::EPSILON);

    assert_eq!(
        parse_key(&mut prefs, "export_image_quality", "255"),
        KeyOutcome::Accepted
    );
    assert_eq!(prefs.image.quality, MAX_JPEG_QUALITY);
    assert_eq!(
        parse_key(&mut prefs, "export_image_quality", "0"),
        KeyOutcome::Accepted
    );
    assert_eq!(prefs.image.quality, MIN_JPEG_QUALITY);

    // ★ Unreadable is a BadValue, and the field keeps whatever it had —
    // per-key recovery, as everywhere else in this file format.
    let before = prefs.image.dpi;
    assert_eq!(
        parse_key(&mut prefs, "export_image_dpi", "fast"),
        KeyOutcome::BadValue
    );
    assert!((prefs.image.dpi - before).abs() < f32::EPSILON);
}

/// ★★ `inf` and `NaN` parse as `f32` and neither is a resolution.
///
/// This is the arm most likely to be written as a bare `.parse().ok()`, and a
/// NaN is the worse of the two: it survives `clamp` unchanged, so it would be
/// *stored*, handed to the window, and every comparison against it would answer
/// `false`.
#[test]
fn a_non_finite_resolution_is_a_bad_value_and_never_reaches_the_window() {
    for spelling in ["inf", "-inf", "NaN", "infinity"] {
        let mut prefs = ExportPrefs::default();
        assert_eq!(
            parse_key(&mut prefs, "export_image_dpi", spelling),
            KeyOutcome::BadValue,
            "`{spelling}` parses as an f32 and is not a resolution"
        );
        assert!(
            prefs.image.dpi.is_finite(),
            "`{spelling}` reached the stored value"
        );
    }
}

/// A key from another group falls through untouched, so `prefs::file` can keep
/// looking.
#[test]
fn a_key_from_another_group_is_not_mine() {
    let mut prefs = ExportPrefs::default();
    for key in ["print_copies", "show_rulers", "export_image", "exporting"] {
        assert_eq!(
            parse_key(&mut prefs, key, "1"),
            KeyOutcome::NotMine,
            "`{key}` is not one of this group's twelve"
        );
    }
    assert_eq!(prefs, ExportPrefs::default());
}

/// ★ A resolution survives the file exactly, at every value the control can
/// reach in one drag.
///
/// `f32::to_string` is shortest-round-trip, which is why the file can hold a
/// float at all — but *"the standard library says so"* is a claim worth one
/// cheap measurement, because a change to how this module writes the number
/// (a format specifier, a rounding step) would break it silently and the
/// operator's 150.5 would come back as 150.
#[test]
fn a_fractional_resolution_survives_the_file_exactly() {
    for dpi in [1.0_f32, 72.0, 96.5, 150.5, 300.0, 599.25, 4800.0] {
        let written = ExportPrefs {
            image: ExportImagePrefs {
                dpi,
                ..ExportImagePrefs::default()
            },
            ..ExportPrefs::default()
        };
        let mut out = String::new();
        write_block(&written, &mut out);

        let mut read = ExportPrefs::default();
        parse_block(&out, &mut read);
        assert!(
            (read.image.dpi - dpi).abs() < f32::EPSILON,
            "{dpi} came back as {}",
            read.image.dpi
        );
    }
}
