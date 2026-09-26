//! # pdfcer-gui-base — the floor of the pdfcer-gui crate stack
//!
//! Modules that reference no other module of `pdfcer-gui`, lifted out of
//! it so that the compiler, rather than a review, is what keeps them beneath
//! the application.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/lib.md`.

#![forbid(unsafe_code)]

// Finding the operator's installed Acrobat and handing the open document over
// to it. Everything that can only be true of a real machine — reading the
// registry, starting a process — is behind a trait in there, so the decisions
// are testable without either.
pub mod acrobat;

/// The one place the program reads a wall clock: PDF and ISO dates in UTC.
/// `pdfcer-core` refuses to supply a timestamp, for determinism and because a
/// date is a claim; its header carries why the answer is UTC and never a local
/// time labelled as one.
pub mod clock;

/// Where the view is, when the scroll offset can no longer say.
pub mod deepanchor;

/// The opt-in trace of what the shell actually received.
pub mod diag;

/// Which file a document came from, and which form field is selected.
pub mod docidentity;

/// OCR: what image the recogniser is shown, the thread it runs on, and the
/// named refusals it can come back with. It authors no PDF — `pdfcer-core`'s
/// `ocr::layer` writes the invisible mode-3 sandwich. See its header for why
/// recognition reads the document as it was OPENED, and for the y-flip it
/// deliberately does not perform.
pub mod ocr;

pub mod pagedrag;

/// The pixel proof that "line weights off" thins a drawing, in the right
/// direction. Tests only.
mod hairline;

/// A revision number per page, so a change to one page repaints only it.
pub mod pageepoch;

/// Does a document's page tree still agree with itself: the raw `/Count`
/// against the leaves actually reachable, audited on every save, and which
/// refusal a disagreement owes. The wording lives in `pdfcer-gui`'s `text`.
pub mod pagetree;

/// The per-page raster scale this document has been measured unable to reach,
/// so zoom stops there instead of showing an error.
pub mod rasterceiling;

/// The ground outside the sheet: the box to rasterize so an object past the
/// page edge is painted.
pub mod rasterhalo;

/// The window's rectangle in PDF user space: the region tier's one conversion.
pub mod rasterregion;

/// Whole page, or just the window: the tier decision, made from numbers.
pub mod rasterstrategy;

/// Rendered pixmaps into egui textures, and what each upload is a picture of.
pub mod raster;

/// A string the operator typed that must never reach a log — one type, and its
/// whole reason for existing is its `Debug`. See its header: a `{:?}` on an
/// action carrying a password writes it into the trace file `tools/ui-verify`
/// keeps as evidence.
pub mod secret;

pub mod settings;

/// **Acrobat-compatible custom stamp collections** — the shell half of
/// engine `Pass 288.0`, and the answer to `OPERATOR_REQUESTS.md` **O169**.
pub mod stamps;

/// The texture cache behind a continuous strip of pages, under a texel budget.
pub mod stripcache;

/// Poster printing: one page across many sheets, with a band along each
/// sheet's top and left for cut marks and the assembly label, and the drawing
/// of both. The tiling is `pdfcer_print::imposition::plan_poster`'s.
pub mod poster;

pub mod pressure;

/// The icon set: SVG path data, a subset parser, a tiny-skia rasterizer and
/// the painter `egui-shell`'s ribbon calls back into. Supplying that painter
/// is what stops the ribbon falling back to text labels — see `icons::paint`.
pub mod icons;

pub mod redact;

/// How sharply a page is drawn, and how long zoom waits before drawing it.
pub mod renderquality;

/// The background render thread: one page raster at a time, cancellable, and
/// a typed refusal when there are no pixels.
pub mod renderworker;

/// Putting the operator's own digital signature on a document.
#[cfg(feature = "signing")]
pub mod sign;

/// Where signature trust ANCHORS come from, and the three facts they let
/// this shell state.
pub mod trust;

/// **The one length-conversion table for this program.**
///
/// Points to millimetres, inches, metres or any other engine [`Unit`], and
/// back. Every operator-facing length goes through it, and the gate
/// `tools/gates/check-unit-conversion.sh` fails the build when a second copy
/// of the constant appears anywhere under either crate's `src/`.
///
/// It exists because of a measured defect, not for tidiness: a sheet of
/// exactly 210.5 mm rendered `210` in the page-thumbnail tooltip and `211`
/// in the print dialogue. Both surfaces computed the same value; they disagreed
/// on the ROUNDING RULE, because half the program wrote `.round()` (half away
/// from zero, the CAD convention) and the other half wrote `{:.0}` in a format
/// string, which is Rust's default and rounds half to even. The module's header
/// argues that choice rather than leaving it to whichever spelling a caller
/// reached for.
///
/// [`Unit`]: pdfcer_core::dimension::Unit
pub mod units;

/// What one canvas remembers between frames.
pub mod viewframe;

/// Which page is shown, at what zoom, in what arrangement, and where.
pub mod viewer;
