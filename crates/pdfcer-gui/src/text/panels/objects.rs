//! # `text::panels::objects` — the Objects panel, and how a page object is
//! described
//!
//! Two jobs, and they are one module because the second is only ever read
//! through the first and through [`super::properties`]:
//!
//! 1. The Objects panel's own chrome — intro, summary, empty states, tree
//!    row labels and tooltips.
//! 2. **The wording of every fact in
//!    [`crate::panels::objects::summary::ObjectSummary`].** That type
//!    deliberately holds no prose at all; it classifies, measures and counts,
//!    and this module alone renders. The split is what makes the classifier
//!    unit-testable without an egui frame, and what makes a copy edit a
//!    one-file change.
//!
//! ## One description, several renderings
//!
//! [`object_row`] and [`super::properties::property_rows`] are two
//! **renderings of one record**. A path's fill colour is therefore never
//! described one way in a tree row and a different way in Properties, and
//! that is structural rather than careful: both read the same
//! `ObjectSummary`, and the colour resolution ("which colour does a viewer
//! actually SEE for this paint disposition?") happens once, in
//! `summary::describe_object`.
//!
//! ## [`object_note`] is rule 4's disclosure, in words
//!
//! Every one of these sentences is a fact `pdfcer-core` already computed and
//! never showed: `TextObject::approximate`, `PaintStyle::is_invisible`, an
//! exact zero-extent comparison on the bbox. **None of them is a guess**,
//! which is what makes surfacing them a disclosure rather than an inference
//! the operator would have to review.
//!
//! Each sentence says WHAT is true and WHY the operator is seeing what they
//! are seeing, because "approximate bounds" on its own is a label, not an
//! explanation — and an explanation is the entire deliverable here. They are
//! long, and that is deliberate: they answer the operator's own report,
//! *"sometimes I click and get a box highlighting on the screen that doesn't
//! seem to correspond to anything."*
//!
//! **They live in a panel, and nowhere else.** The disclosure rule is
//! explicit that inference reporting belongs off-canvas — a status line, a
//! results panel, a properties field — and equally explicit that no badge,
//! tint or dashed outline may be drawn into the page view. A panel is the
//! right home; this module supplies its words and nothing else's.
//!
//! ## What changed at salvage
//!
//! - **`object_detail` and its three helpers are private here too**, exactly
//!   as they were. [`object_row`] is the public surface; the Properties
//!   panel asks for the same facts one at a time rather than reusing the
//!   joined clause, because a vertical list and a one-line row want
//!   different punctuation and joining them would give one of the two the
//!   wrong shape.
//! - **The row-label copy no longer promises Shift+click**, and
//!   [`objects_dock_intro`] no longer promises clicking selects anything on
//!   the page. There is no selection model at S3. The old sentences return
//!   with the selection they describe; stating them now would be a control's
//!   documentation shipping ahead of the control, which is the defect the
//!   old shell's own panel header records twice.
//! - **[`objects_dock_summary`] counts kinds** rather than counting a
//!   selection, for the same reason, and reads a
//!   [`SelectionCensus`](crate::panels::objects::summary::SelectionCensus)
//!   so the selection form is a caller change and not a copy change.

use crate::panels::objects::summary::{
    Degeneracy, ObjectKind, ObjectNote, ObjectSummary, SelectionCensus,
};
use pdfcer_core::vector::{FillRule, PaintStyle, Rgb, TextBoundsBasis};

// ---------------------------------------------------------------------------
// Panel chrome
// ---------------------------------------------------------------------------

/// Intro line above the object list.
///
/// States the ordering convention, because "which end of this list is the
/// front of the page?" is otherwise a guess — and it is the convention the
/// panel's whole diagnostic value rests on. The old sentence continued
/// *"Click a row to select it on the page; Shift+click to add it to, or
/// remove it from, the selection."* Both clauses return with the selection
/// model.
#[must_use]
pub fn objects_dock_intro() -> &'static str {
    "Everything drawn on this page, front-most first — the object painted last is the first row."
}

/// Empty state: the page decomposed cleanly and holds nothing addressable.
///
/// Deliberately distinct from [`objects_dock_decompose_failed_hint`] — a
/// genuinely blank page and a page pdfcer could not read must never look
/// identical. A failure state that is visually indistinguishable from a
/// success state is the same defect as no message at all.
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
///
/// Reads a census rather than a bare total because the breakdown is the
/// orienting fact — *"1,410 objects"* on a CAD sheet says only that the
/// sheet is big, while *"1,380 paths, 30 text"* says what kind of drawing it
/// is and which of the panel's row kinds to expect.
///
/// Kinds with a zero count are omitted rather than printed as `0`: a row of
/// zeros is noise, and noise is how the numbers that matter get skimmed
/// past.
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
///
/// One entry for every row rather than a per-kind variant: the row shows the
/// same three things whatever kind it is, and assembling this sentence from
/// per-kind fragments is how a catalog acquires four subtly different
/// versions of one idea.
///
/// It names the index's meaning, which is the row's least obvious and most
/// useful property: `#412` is the number `pdfcer object-list` prints and
/// `object-delete` takes, so an operator who wants to script the same change
/// across fifty files can read the argument straight off the panel.
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

/// A part row's label — a text object's run (one show operator).
///
/// **"Run", not "label".** "Label" presumes the CAD case that motivated the
/// text half of this rung; a run is just as often a fragment of ordinary
/// prose. "Run" is the honest structural word and it is already the
/// project's vocabulary in code (`TextRun`, `hit_test_text_runs`) — a
/// UI-only synonym would be a second word for one thing.
#[must_use]
pub fn object_tree_run_row(index: usize) -> String {
    format!("Run #{index}")
}

/// A point row's label.
///
/// The number is the **object-scoped** anchor index — it keeps counting
/// across a part boundary rather than restarting at 0, because the number
/// pdfcer shows and the number `pdfcer node-move --node N` addresses have
/// to be the same number (decision 025 §1.3(b)).
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
///
/// **New at salvage, and it exists because of a measured number.** One path
/// object on a real CAD export holds **6,681 anchors**. The old tree listed
/// every one of them, relying on `ScrollArea::show_rows` to virtualize — but
/// virtualization makes a wall of rows cheap to *draw*, not useful to
/// *read*, and materialising 6,681 `ObjectTreeRow`s to find one costs a
/// frame on the sheet where it matters most.
///
/// So the point list is capped, and the cap is **stated with both numbers**
/// rather than the list being quietly shortened. A silently truncated list
/// is indistinguishable from a short one, which is the same defect
/// [`super::bookmarks_truncated`] exists to prevent one panel over.
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
///
/// The three image kinds get three different names rather than one, because
/// they are three different answers to "what is this?": an inline image
/// lives in the page's own byte stream, an image XObject is a shared
/// resource, and a form XObject is an entire nested drawing treated as one
/// opaque object — which is itself a common explanation for "why is this
/// object so much bigger than the thing I can see?".
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
///
/// **This is also where the winding rule is stated**, and it is stated by
/// naming the non-default: `even-odd` is called out, non-zero is not. That
/// is not brevity — non-zero is the rule a reader applies to `f`, and a row
/// that said "filled (non-zero)" on nine hundred ordinary shapes would bury
/// the thirty where the distinction changes what is drawn.
///
/// Words rather than the CLI's machine tokens (`fill-nonzero+stroke`): this
/// is prose an operator reads, and the two surfaces have different
/// audiences. The `n` case is spelled out at length because "paints nothing"
/// is the direct answer to "why is there an object here over blank paper?" —
/// a clip or discarded path is still a real, addressable object.
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
///
/// [`paint_style_label`] names it only when it is even-odd, which is right
/// for a one-line row and wrong for a field list: a field headed "Winding
/// rule" that is blank for nine rows in ten reads as a value pdfcer failed to
/// read. A field list has room to state both, so it does.
///
/// `None` for an object that has no fill at all — a stroke-only or `n`-op
/// path has no winding rule in effect, and printing "non-zero" there would
/// name a rule that decides nothing.
#[must_use]
pub fn winding_rule_label(style: PaintStyle) -> Option<&'static str> {
    match style.fill {
        Some(FillRule::NonZero) => Some("Non-zero"),
        Some(FillRule::EvenOdd) => Some("Even-odd"),
        None => None,
    }
}

/// Format a colour as `#RRGGBB`.
///
/// Number formatting lives in the catalog, never inline at a call site.
///
/// Components are clamped before scaling: a PDF may set a colour component
/// outside 0..1 and the decomposition records what it read rather than
/// silently repairing it, so the clamp belongs here, at the point of
/// display, not in the model.
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
///
/// **A second, shorter cap than the core model's**
/// `pdfcer_core::vector::MAX_TEXT_PREVIEW_CHARS` (64), and deliberately so:
/// that one is a memory budget for a page of 50,000 objects, this one is a
/// line-length budget for a ~320 pt dock. Tying the row width to the storage
/// cap would mean a future memory decision silently re-typesetting the
/// panel. 32 characters is enough to recognise a caption or a ce-dimension
/// label — which is the row's whole job — inside a row that also carries an
/// index, a kind and a font.
const ROW_TEXT_CHARS: usize = 32;

/// A text preview as a quoted, elided, control-character-free fragment.
///
/// Three things happen here, each for a stated reason:
///
/// 1. **Quoted**, so an empty or space-only string is visible as a string
///    rather than as a gap in the row.
/// 2. **Elided at `limit`** with a `…`, and the ellipsis is also appended
///    when the CORE model already truncated (`truncated`), so a long string
///    never presents its prefix as the whole.
/// 3. **Control characters replaced** with `·`. A `\n` or a `\t` inside a
///    show string would otherwise break the row's layout or, worse, silently
///    vanish — and an invisible character in a label is exactly the kind of
///    thing that makes an operator distrust the panel.
///
/// `limit` is a parameter rather than a constant because the row and the
/// Properties panel have different widths to spend; both still get the same
/// quoting and the same control-character treatment, which is the part that
/// must not diverge.
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
///
/// Larger than [`ROW_TEXT_CHARS`] because a field wraps and a row does not,
/// so the panel can afford the whole preview the core model kept. It is
/// still a cap rather than "everything", for the same reason the row's is:
/// the two answer different questions from
/// `pdfcer_core::vector::MAX_TEXT_PREVIEW_CHARS`, which is a memory budget
/// for a page of 50,000 objects. 64 matches that budget today, which makes
/// the panel's elision purely a function of what the model kept — the honest
/// place for it to be.
const PANEL_TEXT_CHARS: usize = 64;

/// A text preview for a Properties **field**.
///
/// Same quoting and same control-character treatment as the row's — that
/// half must not diverge, because an invisible `\n` is exactly as misleading
/// in a field as in a row — and a longer cap, because a field wraps. See
/// [`PANEL_TEXT_CHARS`].
#[must_use]
pub fn quoted_text(text: &str, truncated: bool) -> String {
    quoted_text_preview(text, truncated, PANEL_TEXT_CHARS)
}

/// A font as a fragment: the typeface if the file names one, else the
/// resource name, then the size.
///
/// `/BaseFont` is preferred over the `Tf` resource name because `F1` names
/// nothing an operator can recognise — but the resource name is shown when
/// that is all there is, rather than dropping the font entirely, since
/// "which resource" is still the handle for a later edit.
///
/// The size is the `Tf` operand, **as the file states it** — see
/// `pdfcer_core::vector::TextFont::size`, which documents why it is not
/// scaled by a `Tm`/`cm`. It is written `10 pt` rather than `10.00 pt`
/// because a type size is conventionally a whole number and the trailing
/// zeros would read as a precision this value does not claim.
#[must_use]
pub fn font_label(font: &pdfcer_core::vector::TextFont) -> String {
    let name = font
        .base_font
        .as_deref()
        .filter(|n| !n.is_empty())
        .unwrap_or(&font.resource);
    let size = font.size;
    if size.is_finite() && (size.fract().abs() < 1e-9) {
        format!("{name} {size:.0} pt")
    } else {
        format!("{name} {size:.2} pt")
    }
}

/// The detail clause for one object — everything after its kind name.
///
/// Built from whatever the summary actually carries, in a fixed order, so
/// the same object always reads the same way:
///
/// | Kind | Clause |
/// |---|---|
/// | Path | `stroked #1A73E8, 0.50 pt wide · 4 node(s)` |
/// | Text | `"Section A-A" · Helvetica 10 pt` |
/// | Image | `640 × 480 px` |
///
/// Every part is omitted when the fact is absent rather than filled with a
/// placeholder: a text object with no `Tf` shows only its string, an image
/// with unusable `/Width`/`/Height` shows no pixel clause, and an object with
/// nothing to add gets an empty clause and just its kind name.
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
///
/// `index` is the object's PAINT-ORDER index, printed verbatim so it
/// cross-references `pdfcer object-list`'s `index=` field and the
/// `object-move` / `object-delete` / `node-move` operands, which all address
/// an object by exactly this number. Showing a display-position number
/// instead (the list is drawn back-to-front) would produce a number that
/// looks equally authoritative and addresses a different object.
///
/// Takes an [`ObjectSummary`] rather than a `VectorObject`: the
/// classification (which colour is actually visible, how many nodes, which
/// disclosures apply) belongs to `summary::describe_object`, and this
/// function only words it. That is the single-source-of-truth requirement
/// made structural — this row and the Properties panel are two renderings of
/// ONE record, so a fill colour cannot be described one way here and another
/// way there.
///
/// The trailing note marker is what makes a row diagnostic rather than
/// decorative: a text row, a clip path and a hairline all look ordinary
/// until the row says out loud that its stated extent will not match what is
/// on the paper.
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

/// The one note worth putting on a single line beside an object's detail
/// clause, if any.
///
/// `PaintsNothing` is deliberately skipped: [`paint_style_label`] already
/// spells it out inside the detail clause, and a line reading "…paints
/// nothing (a clip or discarded path) · 4 node(s) · paints nothing" says it
/// twice and explains it neither time. Every other note adds a fact the
/// detail clause does not carry. The FULL sentence for `PaintsNothing` is
/// still shown in the Properties panel's disclosure list, where it earns its
/// space by saying the object is real and addressable.
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
///
/// Paired with [`object_note`]'s long form rather than replacing it: the row
/// flags that something needs explaining, the Properties panel explains it.
/// Two lengths of the same fact, never two different facts.
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
        // ★ NO CATCH-ALL ARM, deliberately. `ObjectNote` and
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
            // ★ The replacement character is **named, not shown**.
            //
            // This sentence used to contain a literal `\u{fffd}`, and the
            // bundled font stack cannot draw it — so a sentence explaining
            // that some characters are unreadable rendered its own example as
            // an unreadable box. Found by the widened glyph gate; see
            // `DEFECTS.md` D12.
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

    /// **★ Every note in the catalog has both a short form and a full
    /// sentence, and no two notes share either.**
    ///
    /// This is the sweep the `ObjectNote::ALL` array exists for. A note
    /// added without an explanation — or with one copy-pasted from its
    /// neighbour — would ship as a disclosure that discloses nothing, and
    /// that is worse than no note at all because it looks like the app
    /// answered the question. Review does not catch that reliably; a sweep
    /// does.
    ///
    /// The four `ApproximateTextBounds` bases are the ones most likely to
    /// collapse into one sentence, and they are exactly the ones where the
    /// difference matters most: "measured from the font's own metrics" and
    /// "a rough guess that may miss the letters entirely" are opposite
    /// levels of confidence about the same field.
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
    ///
    /// The three image kinds in particular: collapsing "Image", "Image
    /// (inline)" and "Form" into one word throws away the distinction that
    /// most often explains a form XObject's surprising size.
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
    ///
    /// Six combinations of `(fill, stroke)`. The `(None, false)` case is the
    /// one that must never come out blank: an object over blank paper with
    /// no description is precisely the "box over nothing" confusion.
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
    ///
    /// An invisible `\n` in a label breaks the row's layout; a silently
    /// dropped one makes the panel look like it is showing a different
    /// string from the one in the file.
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
    ///
    /// "This page really is blank" and "pdfcer could not read this page" must
    /// never look the same — a failure state indistinguishable from a
    /// success state is the same defect as no message at all.
    #[test]
    fn a_blank_page_and_an_unreadable_one_read_differently() {
        assert_ne!(
            objects_dock_empty_page_hint(),
            objects_dock_decompose_failed_hint()
        );
        assert!(objects_dock_decompose_failed_hint().contains("could not"));
    }

    /// **The intro must not promise a selection this build does not have.**
    ///
    /// The old sentence said "Click a row to select it on the page". Nothing
    /// in this build selects anything on the page, and a panel that says it
    /// does sends an operator hunting for a broken control.
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
    ///
    /// A list quietly shortened to its first N is indistinguishable from a
    /// list that is N long. Stating "the first 200 of 6,681" is the whole
    /// difference.
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
}
