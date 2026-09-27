//! # `app::prefs::printing` — what the print dialog remembers between jobs
//!
//! Operator request **O166**, 2026-09-10: *"the printer dialogue box needs to
//! remember our last settings."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/prefs/printing.md`.

use pdfcer_render::AnnotationScope;

use crate::printspooler::{Duplex, Orientation, PageSubset, PaperChoice, ScaleMode};

/// The lowest resolution ceiling the file will accept, in DPI.
pub const MIN_PRINT_DPI: u32 = 36;

/// The highest resolution ceiling the file will accept, in DPI.
pub const MAX_PRINT_DPI: u32 = 2400;

/// The most copies the file will accept.
pub const MAX_PRINT_COPIES: u16 = 999;

/// **How this operator prints**, carried between sittings.
///
/// See the module header for the rule that decides membership: every field
/// here would still be the right answer for a *different document*, and nothing
/// that names a page, a sheet index or a window arrangement is present.
// `Eq` is deliberately absent: `ScaleMode::Custom` carries an `f64`, so the
// dialog's own scale type is only `PartialEq`. Storing the dialog's types
// rather than a mirrored set is the point of this module, and inheriting their
// trait bounds is part of the deal.
#[derive(Debug, Clone, PartialEq)]
pub struct PrintPrefs {
    /// The printer's Windows name, or `None` if none has been used yet.
    ///
    /// # By NAME, never by index
    ///
    /// `PrintDialog::selected` is an index into a list the spooler builds fresh
    /// on every open, and that list reorders whenever a printer is added,
    /// removed or renamed. Persisting `2` would mean an operator who installs a
    /// new printer silently starts printing to a different device — no error,
    /// no prompt, and the evidence is on paper in another room.
    ///
    /// A name that no longer resolves falls back to the Windows default, which
    /// is what this build did before this preference existed. Deliberately
    /// silent: a printer being gone is not a fault in the file, and a note
    /// about it on every launch would outlive its usefulness by years.
    pub printer: Option<String>,
    /// Sheet orientation.
    pub orientation: Orientation,
    /// Two-sided printing.
    ///
    /// Restored even onto a device that cannot do it. The dialog reads
    /// `DeviceFeatures::supports_duplex` and does not draw the control when the
    /// device says no — so the value sits unused and comes back the moment the
    /// operator returns to a duplex printer, which is the behaviour they would
    /// expect from having set it once.
    pub duplex: Duplex,
    /// Ask the driver to pick the input tray from each page's size.
    pub pick_tray_by_page_size: bool,
    /// The paper **policy** — never a specific form id. See the module header.
    pub paper: PaperChoice,
    /// How each page is sized onto the sheet.
    pub scale: ScaleMode,
    /// The custom percentage, kept whether or not custom is the live mode — for
    /// the same reason the dialog keeps it across mode switches, so that
    /// switching away and back does not lose a typed number.
    pub custom_percent: u32,
    /// Which classes of annotation print.
    pub scope: AnnotationScope,
    /// Rendering resolution ceiling, in DPI.
    pub max_dpi: u32,
    /// How many copies.
    pub copies: u16,
    /// `true` when copies come out uncollated.
    pub uncollated: bool,
    /// Odd/even filtering.
    ///
    /// ⚠ Arguably a property of a *job* rather than of an operator — but
    /// manual two-sided printing on a simplex device is exactly the workflow
    /// that makes it a habit, and that workflow is the reason the control
    /// exists. It is remembered, and the control shows its state plainly.
    pub subset: PageSubset,
    /// Print back to front.
    pub reverse: bool,
    /// Poster mode and its settings.
    pub poster: PosterPrefs,
    /// One fixed line width instead of the document's weights (O233).
    pub lines: LineWidthPrefs,
}

/// Print every line at one width.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineWidthPrefs {
    /// Replace the document's line weights with one width.
    pub fixed: bool,
    /// Let pdfcer choose the width; see `dialogs::print::lines::AUTO_MM`.
    pub auto: bool,
    /// The typed width on paper, millimetres, used when `auto` is off.
    pub width_mm: f64,
}

/// The typed width's range, millimetres — the dialog's and the file's.
pub const LINE_WIDTH_MM_RANGE: std::ops::RangeInclusive<f64> = 0.01..=5.0;

impl Default for LineWidthPrefs {
    /// Off, so a print carries the document's real weights (O137); Auto
    /// ticked, and 0.25 mm waiting in the box for when it is cleared.
    fn default() -> Self {
        Self {
            fixed: false,
            auto: true,
            width_mm: 0.25,
        }
    }
}

/// Poster printing: one page across many sheets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PosterPrefs {
    /// Poster mode instead of the Size mode's scale radios.
    pub on: bool,
    /// Magnification before tiling, percent.
    pub tile_percent: u32,
    /// Shared border duplicated onto neighbouring sheets, millimetres.
    pub overlap_mm: f64,
    /// Cut marks in the sheet's band.
    pub cut_marks: bool,
    /// The assembly label in the sheet's band.
    pub labels: bool,
    /// Tile only pages too large for one sheet.
    pub large_only: bool,
}

/// The tile scale's range, percent — the dialog's and the file's.
pub const POSTER_PERCENT_RANGE: std::ops::RangeInclusive<u32> = 10..=5_000;
/// The overlap's range, millimetres — the dialog's and the file's.
pub const POSTER_OVERLAP_MM_RANGE: std::ops::RangeInclusive<f64> = 0.0..=100.0;

impl Default for PosterPrefs {
    /// Off; 100 %, no overlap, no marks, no labels, every page tiled — the
    /// engine's own `PosterSpec` default.
    fn default() -> Self {
        Self {
            on: false,
            tile_percent: 100,
            overlap_mm: 0.0,
            cut_marks: false,
            labels: false,
            large_only: false,
        }
    }
}

impl Default for PrintPrefs {
    /// **Exactly what `PrintDialog::open` hard-coded before this existed.**
    fn default() -> Self {
        Self {
            printer: None,
            orientation: Orientation::default(),
            duplex: Duplex::default(),
            pick_tray_by_page_size: false,
            paper: PaperChoice::DeviceDefault,
            scale: ScaleMode::Fit,
            custom_percent: 100,
            // Not the renderer's own `DocumentAndMarkups` default, and
            // deliberately: the canvas should show markup, a print should not
            // carry review comments unless asked. Acrobat Pro defaults the
            // other way and Reader defaults to `Document`; pdfcer takes
            // Reader's, because a comment reaching paper unasked is the
            // costlier mistake. Argued in full on `PrintDialog::scope`.
            scope: AnnotationScope::Document,
            max_dpi: 300,
            copies: 1,
            uncollated: false,
            subset: PageSubset::All,
            reverse: false,
            poster: PosterPrefs::default(),
            lines: LineWidthPrefs::default(),
        }
    }
}

/// The file token for a sheet orientation.
#[must_use]
pub const fn orientation_key(value: Orientation) -> &'static str {
    match value {
        // ui-text-exempt: file VALUES, written not displayed.
        Orientation::Auto => "auto",
        // ui-text-exempt: file VALUES, written not displayed.
        Orientation::Portrait => "portrait",
        // ui-text-exempt: file VALUES, written not displayed.
        Orientation::Landscape => "landscape",
    }
}

/// A sheet orientation from its file token, or `None` if unrecognised.
#[must_use]
pub fn orientation_from_key(token: &str) -> Option<Orientation> {
    match token.trim() {
        // ui-text-exempt: file VALUES, parsed not displayed.
        "auto" => Some(Orientation::Auto),
        // ui-text-exempt: file VALUES, parsed not displayed.
        "portrait" => Some(Orientation::Portrait),
        // ui-text-exempt: file VALUES, parsed not displayed.
        "landscape" => Some(Orientation::Landscape),
        _ => None,
    }
}

/// The file token for two-sided printing.
#[must_use]
pub const fn duplex_key(value: Duplex) -> &'static str {
    match value {
        // ui-text-exempt: file VALUES, written not displayed.
        Duplex::Simplex => "off",
        // ui-text-exempt: file VALUES, written not displayed.
        Duplex::LongEdge => "long-edge",
        // ui-text-exempt: file VALUES, written not displayed.
        Duplex::ShortEdge => "short-edge",
    }
}

/// Two-sided printing from its file token, or `None` if unrecognised.
#[must_use]
pub fn duplex_from_key(token: &str) -> Option<Duplex> {
    match token.trim() {
        // ui-text-exempt: file VALUES, parsed not displayed.
        "off" => Some(Duplex::Simplex),
        // ui-text-exempt: file VALUES, parsed not displayed.
        "long-edge" => Some(Duplex::LongEdge),
        // ui-text-exempt: file VALUES, parsed not displayed.
        "short-edge" => Some(Duplex::ShortEdge),
        _ => None,
    }
}

/// The file token for a paper **policy**.
#[must_use]
pub const fn paper_key(value: PaperChoice) -> &'static str {
    match value {
        // ui-text-exempt: file VALUES, written not displayed.
        PaperChoice::DeviceDefault | PaperChoice::Form(_) => "device",
        // ui-text-exempt: file VALUES, written not displayed.
        PaperChoice::AutoFromPages => "match-pages",
    }
}

/// A paper policy from its file token, or `None` if unrecognised.
#[must_use]
pub fn paper_from_key(token: &str) -> Option<PaperChoice> {
    match token.trim() {
        // ui-text-exempt: file VALUES, parsed not displayed.
        "device" => Some(PaperChoice::DeviceDefault),
        // ui-text-exempt: file VALUES, parsed not displayed.
        "match-pages" => Some(PaperChoice::AutoFromPages),
        _ => None,
    }
}

/// The file token for a scale mode.
///
/// The custom percentage is **not** in this token; it has its own key. See the
/// module header.
#[must_use]
pub const fn scale_key(value: ScaleMode) -> &'static str {
    match value {
        // ui-text-exempt: file VALUES, written not displayed.
        ScaleMode::Fit => "fit",
        // ui-text-exempt: file VALUES, written not displayed.
        ScaleMode::ActualSize => "actual",
        // ui-text-exempt: file VALUES, written not displayed.
        ScaleMode::ShrinkOversized => "shrink",
        // ui-text-exempt: file VALUES, written not displayed.
        ScaleMode::Custom(_) => "custom",
    }
}

/// A scale mode from its file token, or `None` if unrecognised.
#[must_use]
pub fn scale_from_key(token: &str) -> Option<ScaleMode> {
    match token.trim() {
        // ui-text-exempt: file VALUES, parsed not displayed.
        "fit" => Some(ScaleMode::Fit),
        // ui-text-exempt: file VALUES, parsed not displayed.
        "actual" => Some(ScaleMode::ActualSize),
        // ui-text-exempt: file VALUES, parsed not displayed.
        "shrink" => Some(ScaleMode::ShrinkOversized),
        // ui-text-exempt: file VALUES, parsed not displayed.
        "custom" => Some(ScaleMode::Custom(1.0)),
        _ => None,
    }
}

/// The file token for odd/even filtering.
#[must_use]
pub const fn subset_key(value: PageSubset) -> &'static str {
    match value {
        // ui-text-exempt: file VALUES, written not displayed.
        PageSubset::All => "all",
        // ui-text-exempt: file VALUES, written not displayed.
        PageSubset::Odd => "odd",
        // ui-text-exempt: file VALUES, written not displayed.
        PageSubset::Even => "even",
    }
}

/// Odd/even filtering from its file token, or `None` if unrecognised.
#[must_use]
pub fn subset_from_key(token: &str) -> Option<PageSubset> {
    match token.trim() {
        // ui-text-exempt: file VALUES, parsed not displayed.
        "all" => Some(PageSubset::All),
        // ui-text-exempt: file VALUES, parsed not displayed.
        "odd" => Some(PageSubset::Odd),
        // ui-text-exempt: file VALUES, parsed not displayed.
        "even" => Some(PageSubset::Even),
        _ => None,
    }
}

/// The file token for which classes of annotation print.
#[must_use]
pub const fn scope_key(value: AnnotationScope) -> &'static str {
    match value {
        // ui-text-exempt: file VALUES, written not displayed.
        AnnotationScope::ContentOnly => "content-only",
        // ui-text-exempt: file VALUES, written not displayed.
        AnnotationScope::Document => "document",
        // ui-text-exempt: file VALUES, written not displayed.
        AnnotationScope::DocumentAndMarkups => "document-and-markups",
        // ui-text-exempt: file VALUES, written not displayed.
        AnnotationScope::DocumentAndStamps => "document-and-stamps",
        // ui-text-exempt: file VALUES, written not displayed.
        AnnotationScope::FormFieldsOnly => "form-fields-only",
        // See this function's doc: the type is `#[non_exhaustive]`, so this arm
        // is required and is the conservative direction.
        // ui-text-exempt: file VALUES, written not displayed.
        _ => "document",
    }
}

/// Which classes of annotation print, from a file token, or `None`.
#[must_use]
pub fn scope_from_key(token: &str) -> Option<AnnotationScope> {
    match token.trim() {
        // ui-text-exempt: file VALUES, parsed not displayed.
        "content-only" => Some(AnnotationScope::ContentOnly),
        // ui-text-exempt: file VALUES, parsed not displayed.
        "document" => Some(AnnotationScope::Document),
        // ui-text-exempt: file VALUES, parsed not displayed.
        "document-and-markups" => Some(AnnotationScope::DocumentAndMarkups),
        // ui-text-exempt: file VALUES, parsed not displayed.
        "document-and-stamps" => Some(AnnotationScope::DocumentAndStamps),
        // ui-text-exempt: file VALUES, parsed not displayed.
        "form-fields-only" => Some(AnnotationScope::FormFieldsOnly),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// The file format for this group — the parser and the writer, together
// ---------------------------------------------------------------------------

/// What [`parse_key`] did with a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum KeyOutcome {
    /// Not a key this group owns. The caller must keep looking.
    NotMine,
    /// Read and stored.
    Accepted,
    /// One of ours, and the value could not be read. The caller reports it and
    /// this group's field keeps its default — per-key recovery, as everywhere
    /// else in this file format.
    BadValue,
}

/// Read one `key = value` line into [`PrintPrefs`], if it belongs to this group.
pub(super) fn parse_key(prefs: &mut PrintPrefs, key: &str, value: &str) -> KeyOutcome {
    /// Store a parsed value, or report the line — the shape every arm below
    /// shares, written once so thirteen arms cannot drift from each other.
    macro_rules! store {
        ($parsed:expr, $field:expr) => {
            match $parsed {
                Some(v) => {
                    $field = v;
                    KeyOutcome::Accepted
                }
                None => KeyOutcome::BadValue,
            }
        };
    }

    match key {
        // A printer NAME, stored raw. An empty value is the legitimate way to
        // say "no printer has been chosen yet", which is what a fresh profile
        // holds. It cannot be a `BadValue` — every string is a legal printer
        // name, including one that no longer resolves. See `PrintPrefs::printer`
        // on why an absent printer falls back silently.
        // ui-text-exempt: a file KEY, parsed out of preferences.txt.
        "print_printer" => {
            prefs.printer = if value.is_empty() {
                None
            } else {
                Some(value.to_owned())
            };
            KeyOutcome::Accepted
        }
        // ui-text-exempt: file KEYS, as above.
        "print_orientation" => store!(orientation_from_key(value), prefs.orientation),
        // ui-text-exempt: file KEYS, as above.
        "print_duplex" => store!(duplex_from_key(value), prefs.duplex),
        // ui-text-exempt: file KEYS, as above.
        "print_paper" => store!(paper_from_key(value), prefs.paper),
        // ui-text-exempt: file KEYS, as above.
        "print_scale" => store!(scale_from_key(value), prefs.scale),
        // ui-text-exempt: file KEYS, as above.
        "print_markup" => store!(scope_from_key(value), prefs.scope),
        // ui-text-exempt: file KEYS, as above.
        "print_subset" => store!(subset_from_key(value), prefs.subset),
        // ui-text-exempt: file KEYS, as above.
        "print_tray_by_page_size" => store!(
            super::opening::bool_from_key(value),
            prefs.pick_tray_by_page_size
        ),
        // ui-text-exempt: file KEYS, as above.
        "print_reverse" => store!(super::opening::bool_from_key(value), prefs.reverse),
        // ⚠ The file says COLLATE and the struct holds UNCOLLATED, so this arm
        // inverts. The file's sense is the one every print dialog on the
        // machine uses and is what an operator hand-editing this expects; the
        // struct's sense is the dialog's own field. Both senses are right for
        // their own reader, so the inversion happens exactly twice — here and
        // in [`write_block`] — and
        // `every_preference_round_trips_through_the_file` is what proves the
        // two halves invert the same way.
        // ui-text-exempt: file KEYS, as above.
        "print_collate" => store!(
            super::opening::bool_from_key(value).map(|collated| !collated),
            prefs.uncollated
        ),
        // The three numbers are CLAMPED rather than rejected. A hand-edited
        // `print_copies = 5000` is a number the operator meant something by,
        // and the nearest legal value is a better answer than silently
        // reverting to 1 — the same posture `chrome::normalise_ui_scale` takes
        // on the scale it clamps. Out of range is not `BadValue`; unparseable
        // is, because there is no nearest legal value for `lots`.
        // ui-text-exempt: file KEYS, as above.
        "print_copies" => store!(
            value
                .parse::<u16>()
                .ok()
                .map(|n| n.clamp(1, MAX_PRINT_COPIES)),
            prefs.copies
        ),
        // ui-text-exempt: file KEYS, as above.
        "print_max_dpi" => store!(
            value
                .parse::<u32>()
                .ok()
                .map(|n| n.clamp(MIN_PRINT_DPI, MAX_PRINT_DPI)),
            prefs.max_dpi
        ),
        // The dialog's own `DragValue` range, so a hand-edited file cannot put
        // a number in the box that the box could not have produced.
        // ui-text-exempt: file KEYS, as above.
        "print_custom_percent" => store!(
            value.parse::<u32>().ok().map(|n| n.clamp(1, 1_000)),
            prefs.custom_percent
        ),
        // ui-text-exempt: file KEYS, as above.
        "print_poster" => store!(super::opening::bool_from_key(value), prefs.poster.on),
        // ui-text-exempt: file KEYS, as above.
        "print_poster_percent" => store!(
            value
                .parse::<u32>()
                .ok()
                .map(|n| n.clamp(*POSTER_PERCENT_RANGE.start(), *POSTER_PERCENT_RANGE.end())),
            prefs.poster.tile_percent
        ),
        // ui-text-exempt: file KEYS, as above.
        "print_poster_overlap_mm" => store!(
            value
                .parse::<f64>()
                .ok()
                .filter(|n| n.is_finite())
                .map(|n| n.clamp(
                    *POSTER_OVERLAP_MM_RANGE.start(),
                    *POSTER_OVERLAP_MM_RANGE.end()
                )),
            prefs.poster.overlap_mm
        ),
        // ui-text-exempt: file KEYS, as above.
        "print_poster_cut_marks" => {
            store!(super::opening::bool_from_key(value), prefs.poster.cut_marks)
        }
        // ui-text-exempt: file KEYS, as above.
        "print_poster_labels" => {
            store!(super::opening::bool_from_key(value), prefs.poster.labels)
        }
        // ui-text-exempt: file KEYS, as above.
        "print_poster_large_only" => store!(
            super::opening::bool_from_key(value),
            prefs.poster.large_only
        ),
        // ui-text-exempt: file KEYS, as above.
        "print_line_fixed" => store!(super::opening::bool_from_key(value), prefs.lines.fixed),
        // ui-text-exempt: file KEYS, as above.
        "print_line_auto" => store!(super::opening::bool_from_key(value), prefs.lines.auto),
        // ui-text-exempt: file KEYS, as above.
        "print_line_width_mm" => store!(
            value
                .parse::<f64>()
                .ok()
                .filter(|n| n.is_finite())
                .map(|n| n.clamp(*LINE_WIDTH_MM_RANGE.start(), *LINE_WIDTH_MM_RANGE.end())),
            prefs.lines.width_mm
        ),
        _ => KeyOutcome::NotMine,
    }
}

/// Write this group's commented block into the file.
pub(super) fn write_block(prefs: &PrintPrefs, out: &mut String) {
    out.push_str(
        "\n\
         # What the Print window opens with. These are the answers you gave\n\
         # the last time you printed -- pdfcer remembers how you print, not\n\
         # what you printed. The page range, the sheet you were looking at in\n\
         # the preview, and the preview's own zoom are deliberately NOT here:\n\
         # those are about one document and would be wrong for the next one.\n\
         #\n\
         # print_printer: the printer's Windows name, exactly as it appears in\n\
         # the Print window's list. Blank means you have not printed yet. If\n\
         # the name no longer matches a printer on this machine -- it was\n\
         # renamed, removed, or you are on a different PC -- pdfcer quietly\n\
         # falls back to the Windows default.\n",
    );
    // ui-text-exempt: a file KEY, written into preferences.txt and parsed back
    // out of it. Never displayed.
    out.push_str("print_printer = ");
    out.push_str(prefs.printer.as_deref().unwrap_or(""));
    out.push('\n');

    out.push_str(
        "\n\
         # print_orientation: auto | portrait | landscape\n\
         # auto decides from each page's own shape, so a set that mixes\n\
         # portrait text with a landscape drawing comes out upright throughout.\n",
    );
    // ui-text-exempt: a file KEY, as above.
    out.push_str("print_orientation = ");
    out.push_str(orientation_key(prefs.orientation));
    out.push('\n');

    out.push_str(
        "\n\
         # print_duplex: off | long-edge | short-edge\n\
         # long-edge is the usual book binding, short-edge the notepad one.\n\
         # Kept even on a printer that cannot do it -- the control is simply\n\
         # not drawn there, and the setting comes back when you next print to\n\
         # a printer that can.\n",
    );
    // ui-text-exempt: a file KEY, as above.
    out.push_str("print_duplex = ");
    out.push_str(duplex_key(prefs.duplex));
    out.push('\n');

    out.push_str(
        "\n\
         # print_paper: device | match-pages\n\
         # device      = say nothing about paper; the printer uses its own\n\
         #               Windows settings. This is the shipped answer.\n\
         # match-pages = pdfcer measures the pages and asks for the smallest\n\
         #               sheet the printer offers that holds them all.\n\
         #\n\
         # Picking one specific sheet by hand in the Print window is NOT\n\
         # remembered, on purpose. Sheet numbers past the standard sizes mean\n\
         # whatever one printer's driver says they mean, so a remembered one\n\
         # would quietly ask for a different sheet the day you change printer.\n\
         # If you want the same sheet every time, use match-pages: it asks for\n\
         # the right one on whatever printer is attached.\n",
    );
    // ui-text-exempt: a file KEY, as above.
    out.push_str("print_paper = ");
    out.push_str(paper_key(prefs.paper));
    out.push('\n');

    out.push_str(
        "\n\
         # print_tray_by_page_size: true | false\n\
         # Asks the printer to choose its own input tray or roll from each\n\
         # page's size. This is what makes a set mixing A1 and A3 land right on\n\
         # a machine with more than one roll.\n",
    );
    // ui-text-exempt: a file KEY, as above.
    out.push_str("print_tray_by_page_size = ");
    out.push_str(super::opening::bool_key(prefs.pick_tray_by_page_size));
    out.push('\n');

    out.push_str(
        "\n\
         # print_scale: fit | actual | shrink | custom\n\
         # fit    = fill the printable area, enlarging a small page if need be.\n\
         # actual = one PDF point to 1/72 inch, whatever that costs.\n\
         # shrink = like actual, except a page too big for the sheet is\n\
         #          reduced. Never enlarges.\n\
         # custom = the percentage on the next line.\n\
         # print_custom_percent: 1 to 1000, where 100 is actual size. Kept\n\
         # whether or not custom is the mode in force, so switching away and\n\
         # back does not lose the number you typed.\n",
    );
    // ui-text-exempt: file KEYS, as above.
    out.push_str("print_scale = ");
    out.push_str(scale_key(prefs.scale));
    out.push('\n');
    out.push_str("print_custom_percent = "); // ui-text-exempt: a file KEY, as above.
    out.push_str(&prefs.custom_percent.to_string());
    out.push('\n');

    out.push_str(
        "\n\
         # print_markup: what gets printed besides the page itself.\n\
         # document             = the drawing and its form fields and links,\n\
         #                        but no review comments. The shipped answer.\n\
         # document-and-markups = everything, comments included.\n\
         # document-and-stamps  = comments only if they are stamps.\n\
         # form-fields-only     = the filled-in form, on blank paper.\n\
         # content-only         = the page and nothing else at all.\n",
    );
    // ui-text-exempt: a file KEY, as above.
    out.push_str("print_markup = ");
    out.push_str(scope_key(prefs.scope));
    out.push('\n');

    out.push_str(
        "\n\
         # print_max_dpi: 36 to 2400. An upper bound on how finely pdfcer\n\
         # rasterises a page before sending it. This is a MEMORY limit, not a\n\
         # quality preference -- a big sheet at a high number is measured in\n\
         # gigabytes per page, and the Print window says so when it bites.\n",
    );
    // ui-text-exempt: a file KEY, as above.
    out.push_str("print_max_dpi = ");
    out.push_str(&prefs.max_dpi.to_string());
    out.push('\n');

    out.push_str(
        "\n\
         # print_copies: 1 to 999.\n\
         # print_collate: true | false. True gives you the whole set, then the\n\
         # whole set again; false gives you every copy of page 1, then every\n\
         # copy of page 2.\n\
         # print_subset: all | odd | even -- applied over the page range, so\n\
         # pages 1-10 even only is a thing you can ask for.\n\
         # print_reverse: true | false. Sends the sequence back to front.\n",
    );
    // ui-text-exempt: file KEYS, as above.
    out.push_str("print_copies = ");
    out.push_str(&prefs.copies.to_string());
    out.push('\n');
    // ⚠ Inverted on purpose -- the file says collate, the struct says
    // uncollated. See `parse_key`'s matching arm; the two must invert together
    // and the round-trip test is what proves they do.
    out.push_str("print_collate = "); // ui-text-exempt: a file KEY, as above.
    out.push_str(super::opening::bool_key(!prefs.uncollated));
    out.push('\n');
    out.push_str("print_subset = "); // ui-text-exempt: a file KEY, as above.
    out.push_str(subset_key(prefs.subset));
    out.push('\n');
    out.push_str("print_reverse = "); // ui-text-exempt: a file KEY, as above.
    out.push_str(super::opening::bool_key(prefs.reverse));
    out.push('\n');

    out.push_str(
        "\n\
         # print_poster: true | false. Prints one page across many sheets\n\
         # instead of sizing it onto one.\n\
         # print_poster_percent: 10 to 5000. How big the assembled poster is,\n\
         # where 100 is the page's own size.\n\
         # print_poster_overlap_mm: 0 to 100. How much of each sheet repeats on\n\
         # its neighbour, so the sheets can be lined up and taped.\n\
         # print_poster_cut_marks, print_poster_labels: true | false. Ticks\n\
         # showing where to trim, and each sheet's row and column, printed in\n\
         # a strip along the sheet's top and left edges.\n\
         # print_poster_large_only: true | false. Tile only the pages too big\n\
         # for one sheet; the rest print normally.\n\
         # print_line_fixed: true | false. Print every line at one width\n\
         # instead of the document's line weights.\n\
         # print_line_auto: true | false. Let pdfcer pick that width: the\n\
         # thinnest line that still prints and reads.\n\
         # print_line_width_mm: 0.01 to 5. The width used when auto is off.\n",
    );
    let p = &prefs.poster;
    let bool_key = super::opening::bool_key;
    // ui-text-exempt: file KEYS, as above.
    for (key, value) in [
        ("print_poster", bool_key(p.on).to_owned()),
        ("print_poster_percent", p.tile_percent.to_string()),
        ("print_poster_overlap_mm", p.overlap_mm.to_string()),
        ("print_poster_cut_marks", bool_key(p.cut_marks).to_owned()),
        ("print_poster_labels", bool_key(p.labels).to_owned()),
        ("print_poster_large_only", bool_key(p.large_only).to_owned()),
        ("print_line_fixed", bool_key(prefs.lines.fixed).to_owned()),
        ("print_line_auto", bool_key(prefs.lines.auto).to_owned()),
        ("print_line_width_mm", prefs.lines.width_mm.to_string()),
    ] {
        out.push_str(key);
        out.push_str(" = "); // ui-text-exempt: the file's key-value separator.
        out.push_str(&value);
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every orientation survives a trip through the file.**
    #[test]
    fn every_orientation_round_trips() {
        for value in [
            Orientation::Auto,
            Orientation::Portrait,
            Orientation::Landscape,
        ] {
            assert_eq!(orientation_from_key(orientation_key(value)), Some(value));
        }
    }

    /// **Every duplex setting survives a trip through the file.**
    #[test]
    fn every_duplex_round_trips() {
        for value in [Duplex::Simplex, Duplex::LongEdge, Duplex::ShortEdge] {
            assert_eq!(duplex_from_key(duplex_key(value)), Some(value));
        }
    }

    /// **Every subset survives a trip through the file.**
    #[test]
    fn every_subset_round_trips() {
        for value in [PageSubset::All, PageSubset::Odd, PageSubset::Even] {
            assert_eq!(subset_from_key(subset_key(value)), Some(value));
        }
    }

    /// **Every annotation scope this build can name survives the file.**
    #[test]
    fn every_scope_round_trips() {
        for value in [
            AnnotationScope::ContentOnly,
            AnnotationScope::Document,
            AnnotationScope::DocumentAndMarkups,
            AnnotationScope::DocumentAndStamps,
            AnnotationScope::FormFieldsOnly,
        ] {
            assert_eq!(scope_from_key(scope_key(value)), Some(value));
        }
    }

    /// **Every scale mode round-trips to the same MODE.**
    #[test]
    fn every_scale_mode_round_trips_to_the_same_mode() {
        for value in [
            ScaleMode::Fit,
            ScaleMode::ActualSize,
            ScaleMode::ShrinkOversized,
            ScaleMode::Custom(0.42),
        ] {
            let parsed = scale_from_key(scale_key(value)).expect("own token must parse");
            assert_eq!(
                scale_key(parsed),
                scale_key(value),
                "{value:?} came back as a different mode"
            );
        }
    }

    /// **A specific sheet is written as a policy, never as an id.**
    #[test]
    fn a_form_id_is_deliberately_not_persisted() {
        assert_eq!(
            paper_from_key(paper_key(PaperChoice::Form(257))),
            Some(PaperChoice::DeviceDefault)
        );
        for value in [PaperChoice::DeviceDefault, PaperChoice::AutoFromPages] {
            assert_eq!(paper_from_key(paper_key(value)), Some(value));
        }
    }

    /// **No two values of one enum share a token.**
    ///
    /// A round-trip test alone cannot catch a collision in the *writing*
    /// direction if the parser happens to prefer the right one — so the tokens
    /// are checked for distinctness directly. `PaperChoice` is exempt and is
    /// covered by its own test: two of its variants share `device` on purpose.
    #[test]
    fn no_two_variants_share_a_token() {
        let sets: [Vec<&str>; 4] = [
            vec![
                orientation_key(Orientation::Auto),
                orientation_key(Orientation::Portrait),
                orientation_key(Orientation::Landscape),
            ],
            vec![
                duplex_key(Duplex::Simplex),
                duplex_key(Duplex::LongEdge),
                duplex_key(Duplex::ShortEdge),
            ],
            vec![
                subset_key(PageSubset::All),
                subset_key(PageSubset::Odd),
                subset_key(PageSubset::Even),
            ],
            vec![
                scope_key(AnnotationScope::ContentOnly),
                scope_key(AnnotationScope::Document),
                scope_key(AnnotationScope::DocumentAndMarkups),
                scope_key(AnnotationScope::DocumentAndStamps),
                scope_key(AnnotationScope::FormFieldsOnly),
            ],
        ];
        for set in sets {
            let mut sorted = set.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), set.len(), "a token collision in {set:?}");
        }
    }

    /// **A token carries nothing the file format splits on.**
    #[test]
    fn no_token_carries_a_character_the_format_reserves() {
        let tokens = [
            orientation_key(Orientation::Landscape),
            duplex_key(Duplex::LongEdge),
            paper_key(PaperChoice::AutoFromPages),
            scale_key(ScaleMode::ShrinkOversized),
            subset_key(PageSubset::Even),
            scope_key(AnnotationScope::DocumentAndMarkups),
        ];
        for token in tokens {
            assert!(!token.is_empty(), "an empty token cannot be written");
            assert_eq!(token.trim(), token, "a token must survive trimming");
            assert!(
                !token.contains('=') && !token.contains('#') && !token.contains('\n'),
                "{token:?} carries a character the file format reserves"
            );
        }
    }

    /// **Every remembered field is actually read back into the dialog.**
    #[test]
    fn every_remembered_field_is_read_back_by_the_print_dialog() {
        let here = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

        // --- the field list, straight out of the struct above -------------
        let own = std::fs::read_to_string(here.join("prefs/printing.rs"))
            .expect("this module's own source");
        let (_, after) = own
            .split_once("pub struct PrintPrefs {")
            .expect("the struct declaration, verbatim");
        let (body, _) = after.split_once("\n}").expect("the end of the struct");
        let fields: Vec<&str> = body
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                // `pub name: Type,` — and nothing else. Doc comments,
                // ordinary comments and blank lines all fall out here.
                let rest = line.strip_prefix("pub ")?;
                let (name, _) = rest.split_once(':')?;
                Some(name)
            })
            .collect();
        assert!(
            fields.len() >= 13,
            "the struct parser found only {} field(s) — the declaration's shape \
             changed and this test has gone blind rather than red",
            fields.len()
        );

        // --- the body of `PrintDialog::open` -------------------------------
        let dialog =
            std::fs::read_to_string(here.join("../../pdfcer-gui/src/dialogs/print/mod.rs"))
                .expect("the print dialog's source");
        let (_, after) = dialog
            .split_once("pub(super) fn open(doc: &OpenDoc, remembered:")
            .expect("the constructor's signature, verbatim");
        // Bounded at the next item so that a mention anywhere else in this
        // 1,400-line file cannot satisfy the assertion below.
        let (open_body, _) = after
            .split_once("    fn host()")
            .expect("the item that follows the constructor");

        for field in fields {
            assert!(
                open_body.contains(&format!("remembered.{field}")),
                "★ `PrintPrefs::{field}` is written to the preferences file and \
                 never read back: `PrintDialog::open` does not mention \
                 `remembered.{field}`, so the dialog opens on its hard-coded \
                 value and this preference is inert. Seed the field there, or \
                 remove it from `PrintPrefs` — a preference that is stored and \
                 ignored is worse than one that was never offered."
            );
        }
    }

    /// **A fresh install opens the dialog exactly as every previous build did.**
    #[test]
    fn the_default_is_what_the_dialog_used_to_hard_code() {
        let prefs = PrintPrefs::default();
        assert_eq!(prefs.printer, None);
        assert_eq!(prefs.orientation, Orientation::Auto);
        assert_eq!(prefs.duplex, Duplex::Simplex);
        assert!(!prefs.pick_tray_by_page_size);
        assert_eq!(prefs.paper, PaperChoice::DeviceDefault);
        assert_eq!(prefs.scale, ScaleMode::Fit);
        assert_eq!(prefs.custom_percent, 100);
        assert_eq!(prefs.scope, AnnotationScope::Document);
        assert_eq!(prefs.max_dpi, 300);
        assert_eq!(prefs.copies, 1);
        assert!(!prefs.uncollated);
        assert_eq!(prefs.subset, PageSubset::All);
        assert!(!prefs.reverse);
        assert_eq!(prefs.poster, PosterPrefs::default());
        assert!(!prefs.poster.on);
    }
}
