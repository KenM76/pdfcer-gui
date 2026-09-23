//! # `app::prefs::exporting` — what the three export windows remember
//!
//! Operator request **O196**, 2026-09-13: *"the export windows forget every
//! setting."* Three windows, one complaint, and it is the same complaint
//! **O166** made about the Print window three days earlier — so this module is
//! deliberately a port of [`super::printing`] rather than a fresh design, down
//! to the shape of its token functions and the position of its `remember` call.
//!
//! ## ⚠ This is NOT O192, and the difference decides what gets built
//!
//! `OPERATOR_REQUESTS.md` says it in the row itself: **do not fold this into
//! O192.**
//!
//! - **O192** is *"the window reads no current value **from the document**"* —
//!   the Set Scale dialogue opening with `1 : 1` while the drawing it is about
//!   is at `1 : 50`. The fix is a **read of the open document** at construction.
//! - **O196** is *"the window remembers no previous value **across
//!   sessions**"* — Export image opening on PNG/300 dpi every time, for an
//!   operator who exports EMF at 600 every time. The fix is a **read of a file**
//!   at construction.
//!
//! They look alike from a distance and they fail in opposite directions: O192's
//! fix must be overruled by the document, O196's fix must be overruled by
//! nothing *except* the document. §"the DXF ordering rule" below is where the
//! two meet, and it is the one place in this module where getting the order
//! wrong produces a silently wrong file rather than a mildly annoying window.
//!
//! ## What the three windows were doing before, measured rather than assumed
//!
//!
//! | window | what it hard-coded |
//! |---|---|
//! | Export image | `Png`, `CurrentPage`, 300 dpi, transparent, quality 90 |
//! | Export text | `AllPages`, `FormFeed`, `AsExtracted`, no BOM |
//! | Export to DXF | `DxfOptions::default()` — `Inches`, `fit_arcs`, `Entities` |
//!
//! So an operator who exports every drawing as an EMF at 600 dpi re-answered
//! both questions on every single export, and an operator who works in
//! millimetres re-picked millimetres every time the DXF window opened.
//!
//! ## ★★★ The distinction that decides what is remembered
//!
//! [`super::printing`]'s question, unchanged, because it is the right one:
//! *would this value still be right for a **different document**?*
//!
//! | Remembered | Not remembered, and why |
//! |---|---|
//! | The image format, the resolution, transparency, JPEG quality | The page index the window froze — it names a page of *this* document |
//! | Which pages, as a **policy** (see below) | The typed page range, same reason |
//! | The text separator, the line endings, the byte-order mark | The largest-page measurement — a measurement of *this* document |
//! | The DXF units, arc fitting, whether text is written | ★★ **The DXF scale** — see below; this is the important one |
//! | | `DxfScaleSuggestion` — an inference about *this page's* ce dimension groups |
//! | | `arc_tolerance` — no control exists for it; see below |
//!
//! ### ★★★ The DXF scale is NOT remembered, and that is the whole point of the window
//!
//! `ExportDxfDialog` exists because a DXF carries no scale of its own: the
//! number in that box is the only thing standing between the operator and a
//! drawing that imports at 1/50th of its real size. It is **derived** on every
//! open from the page's own ce dimension groups (`suggest_scale_for_groups`),
//! and a remembered scale would reintroduce, silently and across sessions,
//! precisely the defect the window was built to prevent: a plausible number
//! already in the box, belonging to yesterday's drawing.
//!
//! ⇒ A remembered value is only safe when a wrong one is **visible**. A wrong
//! format is visible — the file has the wrong extension. A wrong scale is not:
//! the DXF opens, the geometry is all there, and it is the wrong size.
//!
//! ### ★★ The DXF ordering rule, which is the only way to get this wrong quietly
//!
//! `ExportDxfDialog::open` seeds `units` from the *suggestion* when the page
//! carries a calibrated ce dimension group, because — its own words —
//! *"a candidate is a group's whole opinion; a 1:50 metre group and a 1:50 inch
//! group are different answers wearing the same number."* The remembered units
//! are the operator's **habit**; the suggested units are a **measurement of the
//! page**.
//!
//! ⇒ **The habit seeds first and the measurement overrules it.** Apply them the
//! other way round and the window shows millimetres beside a scale derived from
//! an inch group, and the DXF comes out wrong by a factor of 25.4 with nothing
//! on screen to say so. Asserted in [`crate::dialogs::export_dxf`]'s own tests;
//! stated here because this module is where the temptation to "just apply the
//! preferences last" lives.
//!
//! ### `PageScope::Typed` is remembered as its own fallback, deliberately
//!
//! [`PageScope`] has three variants and only two of them are policies.
//! *"Current page"* and *"all pages"* are true of any document; *"the pages I
//! typed"* is meaningless without `range_text`, which names pages of one
//! document and is therefore not remembered.
//!
//! So `Typed` is written to the file as the window's own shipped default
//! (`current` for images, `all` for text) — a deliberate reduction, exactly like
//! [`super::printing`]'s `PaperChoice::Form(_) => "device"`. The alternative is
//! a window that opens with the **Pages** radio selected and an empty box beside
//! it, which greys the Export button on open for no reason the operator can see.
//!
//! ### ⚠ `DxfOptions::arc_tolerance` is not remembered, because nothing can set it
//!
//! It is a real field with a real default (0.05) and **no control in the
//! window**. A preference for it would be a key only a hand-editor could reach,
//! describing a setting with no UI — which is the file-format equivalent of the
//! disabled stub **R9** forbids. If a control is ever added, the key is added
//! with it, in this file, in one edit.
//!
//! ### Why the two page scopes are separate keys and not one
//!
//! Export image defaults to **this page** and Export text defaults to **all
//! pages**, and `dialogs/export_text.rs` argues that divergence at length: a
//! picture of one sheet is the common want, and a text file of one page of a
//! forty-page document almost never is. One shared `export_pages` key would
//! force those two windows to agree, which would make the divergence
//! unexpressible and silently discard one of the two decisions.
//!
//! ## Where it is stored, and why not `settings.txt`
//!
//! `preferences.txt`, beside the shell's other preferences: flat `key = value`,
//! hand-editable, per-key recovery, and already covered by the update
//! instruction *"replace the program files, keep your `userdata` folder"*.
//!
//! Not `settings.txt`. Every entry in that file cites a clause the PDF standard
//! leaves to the implementation; whether this operator likes a byte-order mark
//! is not one of them.
//!
//! ## Why the values are the windows' own types and not a mirrored set
//!
//! Same reason as [`super::printing`]: a mirrored enum is a second source of
//! truth that drifts. This module stores [`ImageFormat`], [`PageScope`],
//! [`PageSeparator`], [`LineEndings`] and the engine's own [`DxfUnits`] and
//! [`DxfText`] — the exact values the windows hold — so a variant added to any
//! of them is a compile error here rather than a silent round-trip to the
//! default.
//!
//! ★ All six are `pub` and none is `#[non_exhaustive]`, which is why every
//! `*_key` function below is an exhaustive `match` with **no `_` arm**. That is
//! deliberate and it is the difference from [`super::printing`]'s `scope_key`,
//! which needs a catch-all because `AnnotationScope` is a foreign engine enum
//! that may grow. Do not add a `_` arm here to silence a future compile error:
//! that error is the mechanism working.
//!
//! ## The token functions, and the property that binds them
//!
//! Every enum has a `*_key` (value → token) and a `*_from_key` (token → value)
//! function, and each pair is asserted to round-trip over **every variant** in
//! this module's tests. That is the property the file format actually needs: a
//! writer that emits a token its own parser rejects turns the operator's
//! settings into a `BadValue` note on the next launch, which reads as pdfcer
//! forgetting them — the very complaint this module answers.

use pdfcer_core::export::dxf::{DxfText, DxfUnits};

use crate::app::actions::exporttext::{LineEndings, PageSeparator};
use crate::app::actions::imageexport::{ImageFormat, PageScope};

use super::printing::KeyOutcome;

// ---------------------------------------------------------------------------
// Bounds — the controls' own, deliberately
// ---------------------------------------------------------------------------

/// The lowest resolution the Export-image window's own `DragValue` accepts.
///
/// ★ These four constants are **the controls' bounds**, not separately reasoned
/// limits, and [`super::printing`]'s header states why that matters: *"a file
/// that refused a value the operator could produce by dragging the box would
/// silently discard a setting they had just made."* If a control's range
/// changes, change it here in the same commit — the round-trip is only honest
/// while the two agree.
pub const MIN_EXPORT_DPI: f32 = 1.0;
/// The highest resolution the Export-image window's own `DragValue` accepts.
///
/// Generous on purpose: the real ceiling is the **pixel count**, which depends
/// on the page size and is disclosed in the window rather than enforced here.
pub const MAX_EXPORT_DPI: f32 = 4800.0;
/// The lowest JPEG quality the Export-image window's own `DragValue` accepts.
pub const MIN_JPEG_QUALITY: u8 = 1;
/// The highest JPEG quality the Export-image window's own `DragValue` accepts.
pub const MAX_JPEG_QUALITY: u8 = 100;

// ---------------------------------------------------------------------------
// The three groups
// ---------------------------------------------------------------------------

/// What the **Export image** window opens with.
///
/// Five fields, every one of them an answer to *"how does this operator export
/// pictures"* rather than to *"what is in this document"*. `PartialEq` and not
/// `Eq` because [`Self::dpi`] is an `f32`; the derive is load-bearing rather
/// than decorative — [`crate::dialogs::export_remembered::remember_image`]
/// compares with `!=` and skips the file write when nothing moved.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportImagePrefs {
    /// Which of the four writers the export goes through.
    ///
    /// The single most valuable thing in this module. An operator whose
    /// colleague's LibreOffice cannot read an SVG exports EMF **every time**,
    /// and before O196 re-picked it every time.
    pub format: ImageFormat,
    /// This page, or all of them — as a policy.
    ///
    /// [`PageScope::Typed`] is reduced to [`PageScope::CurrentPage`] on the way
    /// out; see the module header.
    pub scope: PageScope,
    /// Dots per inch for the raster formats.
    ///
    /// Clamped to [`MIN_EXPORT_DPI`]..=[`MAX_EXPORT_DPI`] on read, never
    /// refused: see [`parse_key`].
    pub dpi: f32,
    /// Whether the background is left transparent where the format supports it.
    pub transparent: bool,
    /// JPEG quality.
    ///
    /// ★ Remembered even though the control is only drawn for JPEG. The window
    /// already carries the value across a format switch for the same reason: a
    /// setting that survives being hidden is one the operator does not have to
    /// re-find when they come back to the format it belongs to.
    pub quality: u8,
    /// SVG and EMF: keep text as text instead of outlines. Off by default,
    /// because outlines look the same in every program.
    pub keep_text: bool,
}

impl Default for ExportImagePrefs {
    /// ★ Exactly what `ExportImageDialog::open` hard-coded before this existed.
    ///
    /// The specification, not a coincidence: a fresh `userdata` folder must open
    /// this window the way every previous build of pdfcer opened it. **Deleting
    /// `preferences.txt` is a way to reset pdfcer, never a way to change what it
    /// does.** Asserted rather than assumed — see
    /// `tests::the_image_default_is_what_the_dialog_used_to_hard_code`.
    fn default() -> Self {
        Self {
            format: ImageFormat::Png,
            scope: PageScope::CurrentPage,
            // 300, print grade. The same default the engine's `SvgOptions`
            // takes and for the reason it states: *an embedded raster cannot
            // be re-sampled later*. A screen-grade default would make the
            // common case — a drawing going into a document that
            // will be printed — the case the operator has to
            // remember to fix.
            dpi: 300.0,
            // ★★ Transparency ON, and that is the operator's own
            // instruction rather than a taste: *there had better be full
            // support (including transparency where supported!)*. A default
            // of white would make the feature he asked for the one he has to
            // find.
            transparent: true,
            // `JpegOptions::default()`'s own 90, and the engine states why:
            // it is where `jpeg-encoder` stops subsampling chroma, which for
            // line art and text is the difference between crisp and smeared
            // colour edges. Mirrored rather than read because `JpegOptions`
            // is `#[non_exhaustive]` and this is a `u8` in a window, not an
            // options struct.
            quality: 90,
            keep_text: false,
        }
    }
}

/// What the **Export text** window opens with.
///
/// `PartialEq` for the same dirty-check reason as [`ExportImagePrefs`]; `Eq`
/// would be derivable here, and is deliberately not derived, so the three groups
/// present one shape to a reader.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportTextPrefs {
    /// This page, or all of them — as a policy. See the module header on why
    /// this is a separate key from the image window's.
    pub scope: PageScope,
    /// Form feed, or a visible line naming the page that follows.
    pub separator: PageSeparator,
    /// The engine's own line breaks, or `\r\n` for a Windows tool.
    pub line_endings: LineEndings,
    /// Whether a UTF-8 byte-order mark is written.
    pub byte_order_mark: bool,
}

impl Default for ExportTextPrefs {
    /// ★ Exactly what `ExportTextDialog::open` hard-coded before this existed.
    ///
    /// ⚠ Written out field by field rather than `#[derive(Default)]`, and that
    /// is not style. [`PageScope`] has **no** `Default` impl at all, and the
    /// other three would take their own `#[default]` variants — which happen to
    /// agree today and are not *specified* to. This impl is the specification;
    /// the derive would have been a coincidence that compiles.
    fn default() -> Self {
        Self {
            // ★ `AllPages`, where the image window defaults to `CurrentPage`.
            // The divergence is argued in `dialogs/export_text.rs` and is the
            // reason these are two keys and not one.
            scope: PageScope::AllPages,
            separator: PageSeparator::FormFeed,
            line_endings: LineEndings::AsExtracted,
            byte_order_mark: false,
        }
    }
}

/// What the **Export to DXF** window opens with.
///
/// Three fields out of `DxfOptions`' five. The other two — `scale` and
/// `arc_tolerance` — are argued in the module header, and the argument for
/// `scale` is the most important sentence in this file.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportDxfPrefs {
    /// Inches or millimetres — the operator's habit.
    ///
    /// ⚠ **Overruled by a calibrated ce dimension group on the page.** See the
    /// module header's ordering rule; getting it backwards is wrong by 25.4×
    /// and silent.
    pub units: DxfUnits,
    /// Whether curves are written as true DXF arcs where they fit one.
    pub fit_arcs: bool,
    /// Whether text is written as entities or omitted.
    pub text: DxfText,
}

impl Default for ExportDxfPrefs {
    /// ★ Exactly what `DxfOptions::default()` gave `ExportDxfDialog::open`
    /// before this existed — read off the engine at pin `5e17017` and written
    /// here as literals rather than delegated to `DxfOptions::default()`.
    ///
    /// Literals, deliberately: if the engine changes a default, this window's
    /// behaviour must change **visibly, in a diff**, not silently on a
    /// `cargo update`. The disagreement is then a failing test rather than a
    /// different DXF.
    fn default() -> Self {
        Self {
            units: DxfUnits::Inches,
            fit_arcs: true,
            text: DxfText::Entities,
        }
    }
}

// ---------------------------------------------------------------------------
// Token functions — value <-> file token, one pair per enum
// ---------------------------------------------------------------------------

/// The file token for an image format.
#[must_use]
pub const fn image_format_key(value: ImageFormat) -> &'static str {
    match value {
        // ui-text-exempt: file VALUES, written into preferences.txt and parsed
        // back out of it. Never displayed — the display name of a format is
        // `crate::text::export_image::format_name`.
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpeg",
        ImageFormat::Svg => "svg",
        ImageFormat::Emf => "emf",
    }
}

/// An image format from its file token, or `None` if the token is not one of
/// ours.
///
/// `jpg` is **not** accepted as a synonym for `jpeg`. A second spelling is a
/// second thing the writer and the parser have to agree about, and the file
/// documents its own vocabulary in the block [`write_block`] emits.
#[must_use]
pub fn image_format_from_key(token: &str) -> Option<ImageFormat> {
    match token.trim() {
        "png" => Some(ImageFormat::Png),
        "jpeg" => Some(ImageFormat::Jpeg),
        "svg" => Some(ImageFormat::Svg),
        "emf" => Some(ImageFormat::Emf),
        _ => None,
    }
}

/// The file token for a page scope.
///
/// ★★ **Lossy on purpose.** [`PageScope::Typed`] has no token, because the
/// typed range it depends on is not remembered; a window restored into `Typed`
/// with an empty range box would open with its Export button greyed and nothing
/// on screen to explain it. The caller decides what `Typed` degrades *to* — see
/// [`page_scope_key_or`], which is what both windows actually call.
#[must_use]
pub const fn page_scope_key(value: PageScope) -> Option<&'static str> {
    match value {
        // ui-text-exempt: file VALUES, as above.
        PageScope::CurrentPage => Some("current"),
        PageScope::AllPages => Some("all"),
        PageScope::Typed => None,
    }
}

/// The file token for a page scope, with the window's own fallback for
/// [`PageScope::Typed`].
///
/// `fallback` is the window's shipped default — `CurrentPage` for the image
/// window, `AllPages` for the text window — so a reduction lands on the answer
/// that window would have opened with anyway rather than on a third behaviour.
///
/// # Panics
///
/// Never in practice, and the `expect` says which contract would have to be
/// broken first: `fallback` must itself be a scope with a token. Passing
/// `PageScope::Typed` as the fallback is the one way to reach it, and no caller
/// does — both pass a `const` default.
#[must_use]
pub fn page_scope_key_or(value: PageScope, fallback: PageScope) -> &'static str {
    page_scope_key(value).unwrap_or_else(|| {
        // ui-text-exempt: a panic message, never operator copy. An
        // `expect` string reaches a crash report, not a window, and this
        // one sits on a branch no caller can reach -- see the `# Panics`
        // note above, which names the contract that would have to be
        // broken first. Putting it in the catalog would offer a
        // translator a sentence no operator can ever be shown.
        page_scope_key(fallback).expect("the fallback scope must have a token of its own")
    })
}

/// A page scope from its file token, or `None` if the token is not one of ours.
#[must_use]
pub fn page_scope_from_key(token: &str) -> Option<PageScope> {
    match token.trim() {
        "current" => Some(PageScope::CurrentPage),
        "all" => Some(PageScope::AllPages),
        _ => None,
    }
}

/// The file token for a page separator.
#[must_use]
pub const fn separator_key(value: PageSeparator) -> &'static str {
    match value {
        // ui-text-exempt: file VALUES, as above.
        PageSeparator::FormFeed => "form-feed",
        PageSeparator::Marker => "marker",
    }
}

/// A page separator from its file token, or `None`.
#[must_use]
pub fn separator_from_key(token: &str) -> Option<PageSeparator> {
    match token.trim() {
        "form-feed" => Some(PageSeparator::FormFeed),
        "marker" => Some(PageSeparator::Marker),
        _ => None,
    }
}

/// The file token for a line-ending choice.
#[must_use]
pub const fn line_endings_key(value: LineEndings) -> &'static str {
    match value {
        // ui-text-exempt: file VALUES, as above.
        LineEndings::AsExtracted => "as-extracted",
        LineEndings::Windows => "windows",
    }
}

/// A line-ending choice from its file token, or `None`.
#[must_use]
pub fn line_endings_from_key(token: &str) -> Option<LineEndings> {
    match token.trim() {
        "as-extracted" => Some(LineEndings::AsExtracted),
        "windows" => Some(LineEndings::Windows),
        _ => None,
    }
}

/// The file token for DXF output units.
///
/// ★ `millimetres` with the British spelling, matching the engine's own variant
/// name. The file is a vocabulary of its own and consistency with the type it
/// describes beats consistency with any other file on the machine.
#[must_use]
pub const fn dxf_units_key(value: DxfUnits) -> &'static str {
    match value {
        // ui-text-exempt: file VALUES, as above.
        DxfUnits::Inches => "inches",
        DxfUnits::Millimetres => "millimetres",
    }
}

/// DXF output units from a file token, or `None`.
///
/// ★ `mm` and `millimeters` are accepted **in addition**, and this is the one
/// place in this module that takes a synonym. The reason is not symmetry with
/// the writer — the writer emits exactly one spelling — it is that this is the
/// single key in the file whose British spelling an American hand-editor will
/// get wrong, and the cost of a `BadValue` here is an operator silently
/// exporting inches. The writer's block names the canonical spelling.
#[must_use]
pub fn dxf_units_from_key(token: &str) -> Option<DxfUnits> {
    match token.trim() {
        "inches" => Some(DxfUnits::Inches),
        "millimetres" | "millimeters" | "mm" => Some(DxfUnits::Millimetres),
        _ => None,
    }
}

/// The file token for whether DXF text is written.
#[must_use]
pub const fn dxf_text_key(value: DxfText) -> &'static str {
    match value {
        // ui-text-exempt: file VALUES, as above.
        DxfText::Entities => "entities",
        DxfText::Omit => "omit",
    }
}

/// Whether DXF text is written, from a file token, or `None`.
#[must_use]
pub fn dxf_text_from_key(token: &str) -> Option<DxfText> {
    match token.trim() {
        "entities" => Some(DxfText::Entities),
        "omit" => Some(DxfText::Omit),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// The file format for this group — the parser and the writer, together
// ---------------------------------------------------------------------------

/// Every field of all three groups, in one place.
///
/// Held on [`Prefs`](super::Prefs) as three separate fields rather than one, so
/// that a window reads only its own group and a future fourth export window adds
/// a struct rather than widening one. This type exists only so [`parse_key`] and
/// [`write_block`] take one argument instead of three.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExportPrefs {
    /// The Export-image window's group.
    pub image: ExportImagePrefs,
    /// The Export-text window's group.
    pub text: ExportTextPrefs,
    /// The Export-to-DXF window's group.
    pub dxf: ExportDxfPrefs,
}

/// Read one `key = value` line into [`ExportPrefs`], if it belongs to this
/// group.
///
/// # Why the parser for this group lives HERE and not in `prefs::file`
///
/// [`super::printing::parse_key`] argues this at length and the argument carries
/// over unchanged: the rule *"adding a preference is one edit to one file"* is
/// about **the parser and the writer staying together**, not about their being
/// in `file.rs` specifically. Twelve keys, all about exporting, in the file that
/// already owns their types, defaults and token vocabulary.
///
/// `file.rs` keeps one chained call that delegates here and one that delegates
/// to [`write_block`], so the round-trip tests over the whole of
/// [`Prefs`](super::Prefs) cover this group unchanged.
///
/// # The contract
///
/// `value` arrives already trimmed, as `file.rs` trims both halves before it
/// dispatches. Returns [`KeyOutcome`]; see its variants. [`KeyOutcome`] is
/// borrowed from [`super::printing`] rather than re-declared, for the reason
/// `offpage` borrows it: a second copy is a second thing that can drift.
///
/// # ★ Out of range CLAMPS; unparseable is a `BadValue`
///
/// The numeric keys follow [`super::printing`]'s ruling exactly. `dpi = 99999`
/// becomes [`MAX_EXPORT_DPI`] silently, because the number is a legible
/// intention the control itself would have clamped. `dpi = fast` is a
/// [`KeyOutcome::BadValue`] reported at its line number, because it is not.
pub(super) fn parse_key(prefs: &mut ExportPrefs, key: &str, value: &str) -> KeyOutcome {
    /// Store a parsed value, or report the line — the shape every arm below
    /// shares, written once so twelve arms cannot drift from each other.
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
        // --- Export image ---------------------------------------------------
        // ui-text-exempt: file KEYS, parsed out of preferences.txt.
        "export_image_format" => store!(image_format_from_key(value), prefs.image.format),
        "export_image_pages" => store!(page_scope_from_key(value), prefs.image.scope),
        // ★ Parsed as `f32` rather than as an integer, because the control is a
        // float `DragValue` and a hand-editor who writes `150.5` has written
        // something the window can hold. `is_finite` rather than a bare `ok()`:
        // `"inf"` and `"NaN"` both parse successfully as `f32` and neither is a
        // resolution, and a NaN would survive `clamp` unchanged.
        "export_image_dpi" => store!(
            value
                .parse::<f32>()
                .ok()
                .filter(|n| n.is_finite())
                .map(|n| n.clamp(MIN_EXPORT_DPI, MAX_EXPORT_DPI)),
            prefs.image.dpi
        ),
        "export_image_transparent" => store!(
            super::opening::bool_from_key(value),
            prefs.image.transparent
        ),
        "export_image_quality" => store!(
            value
                .parse::<u8>()
                .ok()
                .map(|n| n.clamp(MIN_JPEG_QUALITY, MAX_JPEG_QUALITY)),
            prefs.image.quality
        ),
        "export_image_keep_text" => {
            store!(super::opening::bool_from_key(value), prefs.image.keep_text)
        }

        // --- Export text ----------------------------------------------------
        "export_text_pages" => store!(page_scope_from_key(value), prefs.text.scope),
        "export_text_separator" => store!(separator_from_key(value), prefs.text.separator),
        "export_text_line_endings" => {
            store!(line_endings_from_key(value), prefs.text.line_endings)
        }
        "export_text_byte_order_mark" => store!(
            super::opening::bool_from_key(value),
            prefs.text.byte_order_mark
        ),

        // --- Export to DXF --------------------------------------------------
        "export_dxf_units" => store!(dxf_units_from_key(value), prefs.dxf.units),
        "export_dxf_fit_arcs" => store!(super::opening::bool_from_key(value), prefs.dxf.fit_arcs),
        "export_dxf_text" => store!(dxf_text_from_key(value), prefs.dxf.text),

        _ => KeyOutcome::NotMine,
    }
}

/// Write this group's commented block into the file.
///
/// Called once by `Prefs::write_to_string`. The comments are as long as they are
/// because the file is meant to be opened in a text editor, and
/// `export_dxf_units = millimetres` tells an operator nothing about what else
/// they could write there.
///
/// ★ Written **unconditionally**, even on a fresh profile where every value is
/// the default. `offpage`'s own note is the reason: *"a preference nobody can
/// discover is a preference nobody has."*
pub(super) fn write_block(prefs: &ExportPrefs, out: &mut String) {
    out.push_str(
        "\n\
         # What the three Export windows open with. These are the answers you\n\
         # gave the last time you exported -- pdfcer remembers how you export,\n\
         # not what you exported. The page range you typed, and the page you\n\
         # happened to be looking at, are deliberately NOT here: those are about\n\
         # one document and would be wrong for the next one.\n",
    );

    // --- Export image -------------------------------------------------------
    out.push_str(
        "\n\
         # export_image_format: png | jpeg | svg | emf\n\
         # png is every pixel as rendered, with transparency.\n\
         # jpeg is every pixel, no transparency, smaller file.\n\
         # svg is vector geometry, for a program that can read it.\n\
         # emf is the same vectors as a Windows metafile -- the format that\n\
         # LibreOffice, Word, Visio and most CAD importers will take.\n",
    );
    // ui-text-exempt: a file KEY, written into preferences.txt and parsed back
    // out of it. Never displayed.
    out.push_str("export_image_format = ");
    out.push_str(image_format_key(prefs.image.format));
    out.push('\n');

    out.push_str(
        "\n\
         # export_image_pages: current | all\n\
         # Which pages the window offers when it opens. Typing a page range in\n\
         # the window is not remembered -- a range names pages of one document,\n\
         # so it would be wrong for the next one -- and a window restored into\n\
         # a range with nothing typed in it could not export anything.\n",
    );
    // ui-text-exempt: a file KEY, as above.
    out.push_str("export_image_pages = ");
    out.push_str(page_scope_key_or(
        prefs.image.scope,
        ExportImagePrefs::default().scope,
    ));
    out.push('\n');

    out.push_str(
        "\n\
         # export_image_dpi: 1 to 4800\n\
         # Dots per inch for png and jpeg. 300 is print grade. svg and emf are\n\
         # vectors and use it only for any picture the page already contained.\n\
         # A number outside the range is pulled back to the nearest end rather\n\
         # than rejected; a number that is not a number is reported.\n",
    );
    // ui-text-exempt: a file KEY, as above.
    out.push_str("export_image_dpi = ");
    out.push_str(&prefs.image.dpi.to_string());
    out.push('\n');

    out.push_str(
        "\n\
         # export_image_transparent: true | false\n\
         # Leave the paper clear instead of filling it white. png and emf can\n\
         # do this; jpeg cannot, and the window says so when you pick it.\n",
    );
    // ui-text-exempt: a file KEY, as above.
    out.push_str("export_image_transparent = ");
    out.push_str(super::opening::bool_key(prefs.image.transparent));
    out.push('\n');

    out.push_str(
        "\n\
         # export_image_quality: 1 to 100\n\
         # jpeg only. 90 is where the encoder stops smearing colour edges,\n\
         # which for line work and text is the difference that shows.\n",
    );
    // ui-text-exempt: a file KEY, as above.
    out.push_str("export_image_quality = ");
    out.push_str(&prefs.image.quality.to_string());
    out.push('\n');

    out.push_str(
        "\n\
         # export_image_keep_text: true | false\n\
         # svg and emf only. true writes text as words that can be selected\n\
         # and searched; false writes it as outlines, which look the same in\n\
         # every program. An svg carries its fonts; an emf cannot, so its\n\
         # words are drawn in whatever font of that name is installed.\n",
    );
    // ui-text-exempt: a file KEY, as above.
    out.push_str("export_image_keep_text = ");
    out.push_str(super::opening::bool_key(prefs.image.keep_text));
    out.push('\n');

    // --- Export text --------------------------------------------------------
    out.push_str(
        "\n\
         # export_text_pages: current | all\n\
         # As above, for the Export text window. It ships as all, where the\n\
         # picture window ships as current: a picture of one sheet is usually\n\
         # what you want, and one page of text out of forty usually is not.\n",
    );
    // ui-text-exempt: a file KEY, as above.
    out.push_str("export_text_pages = ");
    out.push_str(page_scope_key_or(
        prefs.text.scope,
        ExportTextPrefs::default().scope,
    ));
    out.push('\n');

    out.push_str(
        "\n\
         # export_text_separator: form-feed | marker\n\
         # form-feed is the invisible character a text reader treats as a page\n\
         # break. Nothing is added to your document's words.\n\
         # marker is a visible line naming the page that follows. Useful in an\n\
         # editor that shows nothing for a form feed -- but it is text pdfcer\n\
         # wrote, which the document does not contain, and the receipt says so.\n",
    );
    // ui-text-exempt: a file KEY, as above.
    out.push_str("export_text_separator = ");
    out.push_str(separator_key(prefs.text.separator));
    out.push('\n');

    out.push_str(
        "\n\
         # export_text_line_endings: as-extracted | windows\n\
         # as-extracted keeps exactly the breaks the page had. windows writes\n\
         # the pair of characters Notepad and older Windows tools expect.\n",
    );
    // ui-text-exempt: a file KEY, as above.
    out.push_str("export_text_line_endings = ");
    out.push_str(line_endings_key(prefs.text.line_endings));
    out.push('\n');

    out.push_str(
        "\n\
         # export_text_byte_order_mark: true | false\n\
         # Write the three-byte mark that tells a program the file is UTF-8.\n\
         # Excel wants it; some scripts choke on it.\n",
    );
    // ui-text-exempt: a file KEY, as above.
    out.push_str("export_text_byte_order_mark = ");
    out.push_str(super::opening::bool_key(prefs.text.byte_order_mark));
    out.push('\n');

    // --- Export to DXF ------------------------------------------------------
    out.push_str(
        "\n\
         # export_dxf_units: inches | millimetres\n\
         # The units the DXF declares. millimeters and mm are accepted too.\n\
         #\n\
         # IMPORTANT: this is only your usual answer. If the page carries a\n\
         # measured dimension group, pdfcer uses THAT group's units and its\n\
         # scale instead, because a group is a measurement of the drawing and\n\
         # this is only a habit. The window shows you which it used.\n",
    );
    // ui-text-exempt: a file KEY, as above.
    out.push_str("export_dxf_units = ");
    out.push_str(dxf_units_key(prefs.dxf.units));
    out.push('\n');

    out.push_str(
        "\n\
         # The DXF SCALE is deliberately not remembered, and this is the one\n\
         # omission worth explaining. A DXF carries no scale of its own, so the\n\
         # number in that box is the only thing between you and a drawing that\n\
         # imports at the wrong size. pdfcer works it out from the page every\n\
         # time. A remembered scale would put yesterday's number in the box,\n\
         # looking exactly as right as today's would.\n",
    );

    out.push_str(
        "\n\
         # export_dxf_fit_arcs: true | false\n\
         # Write curves as real DXF arcs where one fits, instead of as many\n\
         # short straight segments. Smaller files, and editable curves in CAD.\n",
    );
    // ui-text-exempt: a file KEY, as above.
    out.push_str("export_dxf_fit_arcs = ");
    out.push_str(super::opening::bool_key(prefs.dxf.fit_arcs));
    out.push('\n');

    out.push_str(
        "\n\
         # export_dxf_text: entities | omit\n\
         # entities writes the page's text as DXF text. omit leaves it out,\n\
         # which is what you want when the text is already in the title block\n\
         # of the drawing you are importing into.\n",
    );
    // ui-text-exempt: a file KEY, as above.
    out.push_str("export_dxf_text = ");
    out.push_str(dxf_text_key(prefs.dxf.text));
    out.push('\n');
}

#[cfg(test)]
mod tests;
