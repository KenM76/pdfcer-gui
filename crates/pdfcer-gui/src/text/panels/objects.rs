//! # `text::panels::objects` — the Objects panel, and how a page object is
//! described
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/panels/objects.md`.

use crate::panels::objects::summary::{
    Degeneracy, ObjectKind, ObjectNote, ObjectSummary, SelectionCensus,
};
use pdfcer_core::vector::{FillRule, PaintStyle, Rgb, TextBoundsBasis};

// ---------------------------------------------------------------------------
// Panel chrome
// ---------------------------------------------------------------------------

/// Intro line above the object list.
#[must_use]
pub fn objects_dock_intro() -> &'static str {
    "Everything drawn on this page, front-most first — the object painted last is the first row."
}

/// Empty state: the page decomposed cleanly and holds nothing addressable.
#[must_use]
pub fn objects_dock_empty_page_hint() -> &'static str {
    "This page has nothing pdfcer can address individually — no shapes, text or images."
}

/// Empty state: the page's content could not be analysed.
#[must_use]
pub fn objects_dock_decompose_failed_hint() -> &'static str {
    "pdfcer could not analyse this page's contents, so it cannot list its objects. The page may still display correctly."
}

/// Summary line under the intro: what this page is made of.
#[must_use]
pub fn objects_dock_summary(census: SelectionCensus) -> String {
    let mut parts: Vec<String> = Vec::new();
    if census.paths > 0 {
        parts.push(format!("{} path(s)", census.paths));
    }
    if census.texts > 0 {
        parts.push(format!("{} text object(s)", census.texts));
    }
    if census.images > 0 {
        parts.push(format!("{} image(s)", census.images));
    }
    if census.forms > 0 {
        parts.push(format!("{} form(s)", census.forms));
    }
    if parts.is_empty() {
        return format!("{} object(s) on this page.", census.total);
    }
    format!(
        "{} object(s) on this page — {}.",
        census.total,
        parts.join(", ")
    )
}

/// Tooltip on an object row.
#[must_use]
pub fn objects_dock_row_tooltip() -> &'static str {
    "The number is this object's position in the page's paint order — the same number pdfcer's command-line tools use to address it."
}

/// Width reserved where a leaf object would show an expander, so every row's
/// label starts at the same x.
///
/// The space is held and no dead control is drawn: a leaf has nothing to
/// expand and must not offer to (R83).
pub const OBJECT_TREE_EXPANDER_WIDTH: f32 = 18.0;

/// One level of indent in the object tree.
pub const OBJECT_TREE_INDENT: f32 = 14.0;

/// Tooltip on an object row's expander — says what expanding REVEALS, not
/// that it expands.
#[must_use]
pub fn object_tree_expander_tooltip() -> &'static str {
    "Show the parts this object is drawn from - its separate lines, and the points on them."
}

/// A part row's label — a path's subpath.
#[must_use]
pub fn object_tree_subpath_row(index: usize) -> String {
    format!("Part #{index}")
}

/// A part row's label — one visual line of a text object.
#[must_use]
pub fn object_tree_run_row(index: usize) -> String {
    format!("Line #{index}")
}

/// A point row's label.
#[must_use]
pub fn object_tree_node_row(index: usize) -> String {
    format!("Point #{index}")
}

/// Tooltip on a part row.
#[must_use]
pub fn object_tree_part_tooltip() -> &'static str {
    "One of the separate pieces this object is drawn from. Drawings exported from CAD often put a whole view into a single object."
}

/// Tooltip on a point row.
#[must_use]
pub fn object_tree_node_tooltip() -> &'static str {
    "One anchor point of this part. The number counts across the whole object, so it matches what pdfcer's command-line tools address."
}

/// Disclosure when a part holds more points than the tree will list.
#[must_use]
pub fn object_tree_points_capped(shown: usize, total: usize) -> String {
    format!("Showing the first {shown} of {total} points in this part.")
}

// ---------------------------------------------------------------------------
// Describing one object
// ---------------------------------------------------------------------------

/// The plain-language name of an object kind — the ONE place each kind is
/// named, so the tree row and the Properties panel cannot drift into calling
/// the same thing two names.
#[must_use]
pub fn object_kind_label(kind: ObjectKind) -> &'static str {
    match kind {
        ObjectKind::Path => "Path",
        ObjectKind::Text => "Text",
        ObjectKind::InlineImage => "Image (inline)",
        ObjectKind::ImageXObject => "Image",
        ObjectKind::FormXObject => "Form",
    }
}

/// Plain-language name for a path's painting disposition (§8.5.3, Table 60).
#[must_use]
pub fn paint_style_label(style: PaintStyle) -> &'static str {
    match (style.fill, style.stroke) {
        (Some(FillRule::NonZero), true) => "filled and stroked",
        (Some(FillRule::NonZero), false) => "filled",
        (Some(FillRule::EvenOdd), true) => "filled (even-odd) and stroked",
        (Some(FillRule::EvenOdd), false) => "filled (even-odd)",
        (None, true) => "stroked",
        (None, false) => "paints nothing (a clip or discarded path)",
    }
}

/// The winding rule on its own, for the Properties panel's field list.
#[must_use]
pub fn winding_rule_label(style: PaintStyle) -> Option<&'static str> {
    match style.fill {
        Some(FillRule::NonZero) => Some("Non-zero"),
        Some(FillRule::EvenOdd) => Some("Even-odd"),
        None => None,
    }
}

/// Format a colour as `#RRGGBB`.
#[must_use]
pub fn rgb_hex(colour: Rgb) -> String {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "clamped to 0..=1 immediately before scaling, so the product is 0..=255" // ui-text-exempt: clippy lint justification, never displayed
    )]
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!(
        "#{:02X}{:02X}{:02X}",
        byte(colour.r),
        byte(colour.g),
        byte(colour.b)
    )
}

/// How many characters of a text object's string a one-line row shows before
/// eliding.
const ROW_TEXT_CHARS: usize = 32;

/// A text preview as a quoted, elided, control-character-free fragment.
fn quoted_text_preview(text: &str, truncated: bool, limit: usize) -> String {
    let cleaned: String = text
        .chars()
        .map(|c| if c.is_control() { '·' } else { c })
        .collect();
    let mut shown: String = cleaned.chars().take(limit).collect();
    let elided = truncated || cleaned.chars().count() > limit;
    if elided {
        shown.push('…');
    }
    format!("\"{shown}\"")
}

/// How many characters of a text object's string a Properties **field**
/// shows before eliding.
const PANEL_TEXT_CHARS: usize = 64;

/// A text preview for a Properties **field**.
#[must_use]
pub fn quoted_text(text: &str, truncated: bool) -> String {
    quoted_text_preview(text, truncated, PANEL_TEXT_CHARS)
}

/// **THE SUBSET TAG IS STRIPPED HERE AND NOWHERE ELSE — 2026-09-04.**
#[must_use]
fn without_subset_tag(name: &str) -> &str {
    match name.split_once('+') {
        Some((tag, rest))
            if tag.len() == 6
                && !rest.is_empty()
                && tag.bytes().all(|b| b.is_ascii_uppercase()) =>
        {
            rest
        }
        _ => name,
    }
}

/// A font as a fragment: the typeface if the file names one, else the
/// resource name, then the size.
#[must_use]
pub fn font_label(font: &pdfcer_core::vector::TextFont) -> String {
    let name = without_subset_tag(
        font.base_font
            .as_deref()
            .filter(|n| !n.is_empty())
            .unwrap_or(&font.resource),
    );
    let size = font.size;
    if size.is_finite() && (size.fract().abs() < 1e-9) {
        format!("{name} {size:.0} pt")
    } else {
        format!("{name} {size:.2} pt")
    }
}

/// The detail clause for one object — everything after its kind name.
fn object_detail(summary: &ObjectSummary) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(paint) = summary.paint {
        let mut detail = paint_style_label(paint).to_owned();
        if let Some(colour) = summary.colour {
            detail.push(' ');
            detail.push_str(&rgb_hex(colour));
        }
        if let Some(width) = summary.line_width {
            detail.push_str(&format!(", {width:.2} pt wide"));
        }
        if let Some(nodes) = summary.nodes {
            detail.push_str(&format!(" · {nodes} node(s)"));
        }
        parts.push(detail);
    }

    if let Some(text) = summary.text.as_deref() {
        parts.push(quoted_text_preview(
            text,
            summary.text_truncated,
            ROW_TEXT_CHARS,
        ));
    }
    if let Some(font) = summary.font.as_ref() {
        parts.push(font_label(font));
    }
    if let Some((w, h)) = summary.pixels {
        // "px" rather than "pt": these are SAMPLES (§8.9.5 Table 89), and
        // the size clause elsewhere in the readout is in points. The two
        // numbers describe different things and must not look alike.
        parts.push(format!("{w} × {h} px"));
    }

    parts.join(" · ")
}

/// One-line row text for any object — the Objects panel's row label.
#[must_use]
pub fn object_row(index: usize, summary: &ObjectSummary) -> String {
    let kind = object_kind_label(summary.kind);
    let detail = object_detail(summary);
    let head = if detail.is_empty() {
        format!("#{index}  {kind}")
    } else {
        format!("#{index}  {kind} · {detail}")
    };
    match headline_note(summary) {
        Some(note) => format!("{head} · {}", object_note_short(note)),
        None => head,
    }
}

/// **The mark a row wears when the object it names carries a
/// disclosure** — added 2026-09-05 with [`object_row_headline`].
pub const OBJECT_ROW_DISCLOSURE_MARK: char = '\u{26a0}';

/// **The MASTER row's label — what the Objects tree actually draws**,
/// since 2026-09-05. `OPERATOR_REQUESTS.md` **O123** part 6, defect 2.
#[must_use]
pub fn object_row_headline(index: usize, summary: &ObjectSummary) -> String {
    let kind = object_kind_label(summary.kind);
    let mut head = format!("#{index}  {kind}");
    let identity = headline_identity(summary);
    if !identity.is_empty() {
        head.push_str(" · ");
        head.push_str(&identity);
    }
    if !summary.notes.is_empty() {
        head.push(' ');
        head.push(OBJECT_ROW_DISCLOSURE_MARK);
    }
    head
}

/// The clauses that tell one object of a kind from another of the same kind.
fn headline_identity(summary: &ObjectSummary) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(text) = summary.text.as_deref() {
        parts.push(quoted_text_preview(
            text,
            summary.text_truncated,
            ROW_TEXT_CHARS,
        ));
    }
    if let Some(colour) = summary.colour {
        parts.push(rgb_hex(colour));
    }
    if let Some(nodes) = summary.nodes {
        parts.push(format!("{nodes} node(s)"));
    }
    if let Some((w, h)) = summary.pixels {
        // "px" rather than "pt", for [`object_detail`]'s stated reason: these
        // are samples, and the size clause elsewhere is in points.
        parts.push(format!("{w} × {h} px"));
    }
    parts.join(" · ")
}

/// The one note worth putting on a single line beside an object's detail
/// clause, if any.
#[must_use]
pub fn headline_note(summary: &ObjectSummary) -> Option<ObjectNote> {
    summary
        .notes
        .iter()
        .copied()
        .find(|note| !matches!(note, ObjectNote::PaintsNothing))
}

/// The SHORT form of a disclosure, for a one-line row where a full sentence
/// would not fit.
#[must_use]
pub fn object_note_short(note: ObjectNote) -> &'static str {
    match note {
        // Four short forms, not one, because the row's job is to flag which
        // KIND of doubt applies — "may miss the letters" and "measured from
        // the font's metrics" are different warnings, and a row that gave
        // both the same two words would leave the operator no reason to open
        // the full explanation for the one that matters.
        ObjectNote::ApproximateTextBounds(TextBoundsBasis::FontMetrics) => "bounds from metrics",
        ObjectNote::ApproximateTextBounds(TextBoundsBasis::MetricAdvancesNominalHeight) => {
            "estimated height"
        }
        ObjectNote::ApproximateTextBounds(TextBoundsBasis::EstimatedAdvances) => "estimated widths",
        ObjectNote::ApproximateTextBounds(TextBoundsBasis::EmBox) => {
            "rough bounds \u{2014} may miss"
        }
        ObjectNote::PaintsNothing => "paints nothing",
        ObjectNote::DegenerateBounds(Degeneracy::VerticalRule) => "zero width",
        ObjectNote::DegenerateBounds(Degeneracy::HorizontalRule) => "zero height",
        ObjectNote::DegenerateBounds(Degeneracy::Point) => "a single point",
        ObjectNote::NoBounds => "no measurable bounds",
        ObjectNote::FormNotDecomposed => "a whole nested drawing",
        ObjectNote::TextUndecodable => "text cannot be read",
        ObjectNote::TextPartlyUndecodable => "some characters cannot be read",
        // NO CATCH-ALL ARM, deliberately. `ObjectNote` and
        // `TextBoundsBasis` are both closed enums, so this match is
        // exhaustive and adding a variant to either **breaks the build**.
        // That is a stronger guard than a `_` arm returning a placeholder,
        // which would let a note ship as a disclosure that discloses
        // nothing — worse than no note at all, because it looks like the app
        // answered the question.
    }
}

/// The FULL disclosure sentence for one fact about an object — the direct
/// answer to the operator's *"sometimes I click and get a box highlighting
/// on the screen that doesn't seem to correspond to anything."*
///
/// See the module header on why these are long, and why they are facts
/// rather than inferences.
#[must_use]
pub fn object_note(note: ObjectNote) -> &'static str {
    match note {
        ObjectNote::ApproximateTextBounds(TextBoundsBasis::FontMetrics) => {
            "The area given for text is laid out from the font's own metrics: pdfcer adds up the \
width of every character the run shows, and takes the height from the font's designed ascent \
and descent. That is exactly how a PDF reader places the text, so the area is where the text \
is. It is still not traced around the letters themselves, so it can be slightly generous \
above short lowercase words, and slightly tight around an italic's overhang or a swash."
        }
        ObjectNote::ApproximateTextBounds(TextBoundsBasis::MetricAdvancesNominalHeight) => {
            "The area given for text is laid out from the font's own character widths, so its \
LEFT and RIGHT edges are where the text really starts and ends. Its HEIGHT is a standing \
estimate: this font declares no ascent or descent for pdfcer to read, so the area is one type \
size tall above the baseline and a quarter of one below. Expect it to be taller than the \
letters rather than shorter."
        }
        ObjectNote::ApproximateTextBounds(TextBoundsBasis::EstimatedAdvances) => {
            "The area given for text is the right shape but an estimated size: this font carries \
no width table of its own, and is not one of the 14 standard faces whose metrics pdfcer has \
built in, so the width of each character was estimated from a similar face. The area starts \
where the text starts and grows with the run, but its right-hand edge can be off by a few \
points either way."
        }
        ObjectNote::ApproximateTextBounds(TextBoundsBasis::EmBox) => {
            "The area given for this text is a rough guess, and it can sit in the wrong place. \
pdfcer could not read the font behind at least part of this run, so it has no character widths \
to lay the text out with; it falls back to marking where the run STARTS and padding that point \
by the largest type size it saw. The result is roughly a square centred on the start of the \
text, not a box around the ink — so it reaches into blank paper before the text, and usually \
stops short of the end of a long run."
        }
        ObjectNote::PaintsNothing => {
            "This path paints nothing at all — it is a clipping path or a shape that was built \
and then discarded without being filled or stroked. It is a real object, and it is listed \
here, but there is nothing on the paper to see."
        }
        ObjectNote::DegenerateBounds(Degeneracy::VerticalRule) => {
            "This object has zero width — it is a vertical rule. The object itself is a line, \
not a box, which is why its width reads as 0.0 pt."
        }
        ObjectNote::DegenerateBounds(Degeneracy::HorizontalRule) => {
            "This object has zero height — it is a horizontal rule. The object itself is a line, \
not a box, which is why its height reads as 0.0 pt."
        }
        ObjectNote::DegenerateBounds(Degeneracy::Point) => {
            "This object is a single point — it has no width and no height."
        }
        ObjectNote::NoBounds => {
            "pdfcer could not work out where this object is on the page, so it has no position or \
size to report. It is still a real object and it is still listed here."
        }
        ObjectNote::FormNotDecomposed => {
            "This is a form XObject — a whole nested drawing that pdfcer treats as ONE object. \
Its size covers the entire nested drawing, and the shapes inside it are not listed \
individually."
        }
        ObjectNote::TextUndecodable => {
            "pdfcer cannot read this text. The font gives no way to work out which characters its \
codes stand for — it carries no /ToUnicode table and uses an encoding that is only meaningful \
inside the font itself. The text still displays and prints correctly; it simply cannot be \
turned back into letters. Rather than show a row of question marks, pdfcer says so."
        }
        ObjectNote::TextPartlyUndecodable => {
            // The replacement character is **named, not shown**.
            //
            //
            // Naming it is better than substituting a drawable stand-in
            // anyway: the operator is being told what they will see *in the
            // text on the page*, which is drawn from the document's own fonts
            // and has nothing to do with what this panel's font can render.
            // Showing a mark here would have implied the two were the same.
            "Some characters in this text could not be read, and are shown as the Unicode \
replacement character. Their font gives no mapping for those particular codes, so pdfcer has no \
way to tell what they stand for. The characters around them are correct, and everything still \
displays and prints as it should."
        } // No catch-all — see `object_note_short`'s closing comment. The
          // exhaustive match IS the guard: a new note cannot reach an operator
          // without someone writing its sentence, because the crate will not
          // compile until they do.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::vector::TextFont;

    /// **Every note in the catalog has both a short form and a full
    /// sentence, and no two notes share either.**
    #[test]
    fn every_note_has_its_own_short_form_and_its_own_sentence() {
        let mut shorts: Vec<&str> = Vec::new();
        let mut longs: Vec<&str> = Vec::new();
        for note in ObjectNote::ALL {
            let short = object_note_short(note);
            let long = object_note(note);
            assert!(!short.is_empty(), "{note:?} has no short form");
            assert!(
                short.chars().count() <= 32,
                "{note:?}'s short form is a sentence, not a marker: {short}"
            );
            assert!(
                long.len() > 60,
                "{note:?}'s sentence is too short to explain anything: {long}"
            );
            assert!(
                !shorts.contains(&short),
                "{note:?} reuses another note's short form: {short}"
            );
            assert!(
                !longs.contains(&long),
                "{note:?} reuses another note's sentence"
            );
            shorts.push(short);
            longs.push(long);
        }
        assert_eq!(shorts.len(), ObjectNote::ALL.len());
    }

    /// **Every object kind has its own name.**
    #[test]
    fn every_object_kind_has_a_distinct_name() {
        let mut seen: Vec<&str> = Vec::new();
        for kind in ObjectKind::ALL {
            let label = object_kind_label(kind);
            assert!(!label.is_empty());
            assert!(!seen.contains(&label), "{kind:?} reuses the name {label}");
            seen.push(label);
        }
    }

    /// **Every paint disposition is named, including the one that paints
    /// nothing.**
    #[test]
    fn every_paint_disposition_is_named_and_the_no_paint_case_says_so() {
        let mut seen: Vec<&str> = Vec::new();
        for fill in [None, Some(FillRule::NonZero), Some(FillRule::EvenOdd)] {
            for stroke in [false, true] {
                let style = PaintStyle { fill, stroke };
                let label = paint_style_label(style);
                assert!(!label.is_empty());
                assert!(!seen.contains(&label), "two dispositions share {label}");
                seen.push(label);
            }
        }
        let nothing = PaintStyle {
            fill: None,
            stroke: false,
        };
        assert!(paint_style_label(nothing).contains("paints nothing"));
    }

    /// The winding rule is stated in full for a field list, and absent when
    /// no fill is in effect.
    ///
    /// Naming "non-zero" for a stroke-only path would name a rule that
    /// decides nothing about what is drawn.
    #[test]
    fn the_winding_rule_is_named_only_when_a_fill_is_in_effect() {
        let filled = PaintStyle {
            fill: Some(FillRule::NonZero),
            stroke: false,
        };
        let even_odd = PaintStyle {
            fill: Some(FillRule::EvenOdd),
            stroke: true,
        };
        let stroked = PaintStyle {
            fill: None,
            stroke: true,
        };
        assert_eq!(winding_rule_label(filled), Some("Non-zero"));
        assert_eq!(winding_rule_label(even_odd), Some("Even-odd"));
        assert_eq!(winding_rule_label(stroked), None);
    }

    /// A control character never reaches a row, and an elision is always
    /// marked.
    #[test]
    fn a_text_preview_is_quoted_de_controlled_and_marked_when_elided() {
        let s = quoted_text_preview("a\nb\tc", false, 32);
        assert_eq!(s, "\"a·b·c\"");
        assert!(!s.contains('\n') && !s.contains('\t'));

        // Longer than the limit: elided, and the ellipsis says so.
        let long: String = "x".repeat(40);
        let e = quoted_text_preview(&long, false, 32);
        assert!(e.ends_with("…\""));
        assert_eq!(e.chars().filter(|c| *c == 'x').count(), 32);

        // Short, but the CORE already truncated: still marked, because the
        // prefix must never present itself as the whole.
        let t = quoted_text_preview("abc", true, 32);
        assert!(t.ends_with("…\""), "{t}");

        // An empty string is visible as a string, not as a gap.
        assert_eq!(quoted_text_preview("", false, 32), "\"\"");
    }

    /// A font is named by its typeface when the file gives one, and by its
    /// resource name when that is all there is.
    /// A subset tag is six arbitrary letters and a plus, and it is stripped
    /// from the ROW because it means nothing and costs seven characters on
    /// every text row.
    #[test]
    fn a_subset_tag_is_stripped_from_the_row() {
        assert_eq!(
            without_subset_tag("AAAAAA+JetBrainsMono-Regular"),
            "JetBrainsMono-Regular"
        );
        assert_eq!(
            without_subset_tag("BAAAAA+SpaceGrotesk-Bold"),
            "SpaceGrotesk-Bold"
        );
    }

    /// And a `+` that is not a subset tag is left alone — losing half of a
    /// real face name would be a silent corruption of the one thing this label
    /// exists to say.
    #[test]
    fn a_plus_that_is_not_a_subset_tag_survives() {
        for name in [
            "Arial+Bold",            // five letters, not six
            "AAAAAAA+Thing",         // seven
            "aaaaaa+Thing",          // lower case
            "AAAA1A+Thing",          // not all letters
            "AAAAAA+",               // nothing after the plus
            "JetBrainsMono-Regular", // no plus at all
        ] {
            assert_eq!(without_subset_tag(name), name, "{name} must survive");
        }
    }

    #[test]
    fn a_font_falls_back_to_its_resource_name_and_formats_a_whole_size() {
        let named = TextFont {
            resource: "F1".to_owned(),
            base_font: Some("Helvetica".to_owned()),
            size: 10.0,
        };
        assert_eq!(font_label(&named), "Helvetica 10 pt");

        // No `/BaseFont`: the resource name is still a handle, so it is
        // shown rather than the font being dropped.
        let unnamed = TextFont {
            resource: "F7".to_owned(),
            base_font: None,
            size: 10.5,
        };
        assert_eq!(font_label(&unnamed), "F7 10.50 pt");

        // An empty `/BaseFont` is the same as none — a zero-length name is
        // not a name.
        let empty = TextFont {
            resource: "F2".to_owned(),
            base_font: Some(String::new()),
            size: 8.0,
        };
        assert_eq!(font_label(&empty), "F2 8 pt");
    }

    /// Colour components outside 0..1 are clamped at display time, not
    /// repaired in the model.
    #[test]
    fn a_colour_out_of_range_clamps_rather_than_wrapping() {
        assert_eq!(
            rgb_hex(Rgb {
                r: 0.0,
                g: 0.0,
                b: 1.0
            }),
            "#0000FF"
        );
        assert_eq!(
            rgb_hex(Rgb {
                r: 2.0,
                g: -1.0,
                b: 0.5
            }),
            "#FF0080"
        );
    }

    /// The census line omits kinds with no members, and still states a total
    /// when nothing matched.
    #[test]
    fn the_summary_line_omits_empty_kinds() {
        let census = SelectionCensus {
            total: 3,
            paths: 2,
            texts: 1,
            images: 0,
            forms: 0,
        };
        let line = objects_dock_summary(census);
        assert!(line.contains("2 path(s)") && line.contains("1 text object(s)"));
        assert!(
            !line.contains("image"),
            "an empty kind must not print: {line}"
        );
        assert_eq!(
            objects_dock_summary(SelectionCensus::default()),
            "0 object(s) on this page."
        );
    }

    /// The two empty states are different sentences.
    #[test]
    fn a_blank_page_and_an_unreadable_one_read_differently() {
        assert_ne!(
            objects_dock_empty_page_hint(),
            objects_dock_decompose_failed_hint()
        );
        assert!(objects_dock_decompose_failed_hint().contains("could not"));
    }

    /// **The intro must not promise a selection this build does not have.**
    #[test]
    fn the_intro_does_not_promise_click_to_select() {
        let intro = objects_dock_intro();
        assert!(
            !intro.contains("select"),
            "there is no selection model at S3; the intro must not name one: {intro}"
        );
        // It must still state the ordering, which is what the panel's
        // diagnostic value rests on.
        assert!(intro.contains("front-most first"));
    }

    /// The truncation disclosure names BOTH numbers.
    #[test]
    fn the_point_cap_states_both_numbers() {
        let s = object_tree_points_capped(200, 6681);
        assert!(s.contains("200") && s.contains("6681"), "{s}");
    }

    /// The row's index is the paint-order index, printed verbatim.
    ///
    /// The tooltip is the only place that says what the number means, and
    /// the number is the handle for every command-line verb.
    #[test]
    fn a_row_leads_with_its_paint_order_index() {
        use crate::panels::objects::summary::describe_object;
        use pdfcer_core::content::ContentStream;
        use pdfcer_core::vector::{Matrix, NoXObjects, decompose};

        let cs = ContentStream::parse(b"0 0 1 rg 10 10 80 80 re f".to_vec()).expect("parse");
        let objects = decompose(&cs, Matrix::IDENTITY, &NoXObjects);
        let summary = describe_object(&objects.objects[0]);

        let row = object_row(412, &summary);
        assert!(row.starts_with("#412"), "{row}");
        assert!(row.contains("Path"));
        assert!(row.contains("filled"));
        assert!(
            row.contains("#0000FF"),
            "the visible colour is named: {row}"
        );
        assert!(row.contains("4 node(s)"));
        assert!(objects_dock_row_tooltip().contains("paint order"));
    }

    /// Decompose one content stream and describe its first object.
    fn described(content: &[u8]) -> ObjectSummary {
        use crate::panels::objects::summary::describe_object;
        use pdfcer_core::content::ContentStream;
        use pdfcer_core::vector::{Matrix, NoXObjects, decompose};

        let cs = ContentStream::parse(content.to_vec()).expect("parse");
        let objects = decompose(&cs, Matrix::IDENTITY, &NoXObjects);
        describe_object(&objects.objects[0])
    }

    /// **The headline keeps the identity and drops the description** —
    /// O123 defect 2, asserted clause by clause.
    #[test]
    fn the_headline_is_the_description_with_the_describing_clauses_dropped() {
        let summary = described(b"0 0 1 rg 1 0 0 RG 0.5 w 10 20 m 300 20 l B*");
        let head = object_row_headline(412, &summary);
        let full = object_row(412, &summary);

        // Kept: the operand, the kind, and the two facts that tell one rule
        // from another.
        assert!(head.starts_with("#412"), "{head}");
        assert!(head.contains("Path"), "{head}");
        assert!(head.contains("#0000FF"), "the visible colour: {head}");
        assert!(head.contains("2 node(s)"), "{head}");

        // Dropped: everything the detail pane an inch below already states.
        assert!(
            !head.contains("filled"),
            "the paint-style phrase belongs to the detail pane: {head}"
        );
        assert!(
            !head.contains("pt wide"),
            "the stroke width belongs to the detail pane: {head}"
        );
        // …and it is still in the long form, which the row hovers.
        assert!(
            full.contains("filled") && full.contains("pt wide"),
            "{full}"
        );
        assert!(
            head.chars().count() < full.chars().count(),
            "the headline must be shorter than what it summarises: {head} / {full}"
        );
    }

    /// **An object that paints nothing wears the mark**, which
    /// [`headline_note`] alone would not give it.
    #[test]
    fn a_path_that_paints_nothing_still_wears_the_mark() {
        let summary = described(b"10 10 100 100 re n");
        assert!(
            headline_note(&summary).is_none(),
            "the precondition: the only note here is the one `headline_note` skips"
        );
        let head = object_row_headline(7, &summary);
        assert!(
            head.contains(OBJECT_ROW_DISCLOSURE_MARK),
            "an invisible path must be marked: {head}"
        );
    }

    /// …and an ordinary object with nothing to disclose wears no mark.
    ///
    /// Without this the test above passes on a build that marks every row,
    /// which would make the mark mean nothing.
    #[test]
    fn an_object_with_nothing_to_disclose_wears_no_mark() {
        let summary = described(b"0 0 1 rg 10 10 80 80 re f");
        assert!(summary.notes.is_empty(), "the precondition");
        let head = object_row_headline(1, &summary);
        assert!(
            !head.contains(OBJECT_ROW_DISCLOSURE_MARK),
            "a mark on every row is a mark that says nothing: {head}"
        );
    }
}
