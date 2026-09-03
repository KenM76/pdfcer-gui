//! # `dialogs::print::spooler` — the one module that knows `pdfcer-print` exists
//!
//! ## ★ Read this first: this module is the ADAPTER, and it is now live
//!
//! Everything else in [`crate::dialogs::print`] — the three tabs, the range
//! parser, the zoom anchor, the preview raster cache, the clip disclosure,
//! the commit button's label — is written against the types *in here*.
//! Nothing else in the dialog names a printing type, which is what confined
//! the whole "make printing work" change to this one module.
//!
//! ## Two files, and where the seam is
//!
//! This was one file until 2026-08-18, when the paper-selection work would
//! have carried it past R2's 1,500-line limit. It split on a seam that was
//! already there:
//!
//! | file | subject | changes when |
//! |---|---|---|
//! | this one | **the job** — which pages, at what size, in what order, placed where | the layout arithmetic changes |
//! | [`device`] | **the device** — which printers exist, what each can do, which sheets it offers, what its driver holds | the way a device is interrogated changes |
//!
//! [`device`]'s types are re-exported here, so every caller still says
//! `spooler::Printer` and `spooler::device_features`. See the re-export's own
//! note for why the seam is not pushed out to the call sites.
//!
//! ## ★ The defect this file carried for the whole of v0.1.0, recorded
//!
//! This header used to open with the sentence *"`pdfcer-print` is NOT a
//! dependency of this crate"* and then set out, in full, the two edits that
//! would make the build print: add the manifest line, then fill the four
//! holes below. **The manifest line landed and the four holes were never
//! filled.** `pdfcer-print` sat in `Cargo.toml` and in `Cargo.lock`, was
//! compiled and linked into every shipped binary, and no source file in the
//! crate contained the identifier `pdfcer_print` outside a doc comment. So
//! [`list_printers`] kept returning a refusal, the dialog kept rendering
//! *"This build cannot reach a print device"*, and the commit button was
//! never drawn.
//!
//! The operator's report was *"the print dialogue didn't work"*, and it was
//! exactly right. Two things are worth carrying forward from it:
//!
//! 1. **The whole test suite was green throughout.** It had to be: the tests
//!    asserted that every hole *refuses*, which is the correct assertion for
//!    an unlinked build and becomes a lock on the defect the moment the
//!    manifest line lands. A test that pins a refusal must name the condition
//!    the refusal is conditional on, or it outlives its own premise. The
//!    replacement — `every_call_reaches_the_engine_rather_than_refusing` —
//!    asserts the opposite property, and it is written so that it cannot pass
//!    on a machine with no printers.
//! 2. **A doc comment that describes future work is a liability with a shelf
//!    life.** This one was precise, correct, and read by nobody at the moment
//!    it became actionable. Where a plan like that is written down again it
//!    belongs in `GUI_ROADMAP.md`, where something sweeps it, and not only in
//!    the header of the file it happens to be about.
//!
//! ## What this build does now
//!
//! [`list_printers`] enumerates the system's printers, [`device_features`]
//! reads one device's duplex and copy support, [`plan`] turns the geometry
//! and places every page, and [`spool`] hands the rendered sheets to the
//! Windows spooler. The commit button is drawn whenever there is a device and
//! a non-empty plan, and pressing it consumes paper.
//!
//! Three ways to have no printer are still said three ways — `pdfcer-print`
//! refuses to collapse them, because non-Windows `list_printers` returns
//! `Err(Unsupported)` rather than an empty `Vec`, since *"reporting the same
//! value for 'this platform cannot enumerate printers at all' would collapse
//! two different facts into one and send a caller looking for hardware"*
//! (`lib.rs:1859-1866`). [`Unavailable`] carries that distinction across the
//! port; see [`crate::text::print`]'s header for the three sentences it
//! feeds.
//!
//! ## Why mirror the types rather than re-export them
//!
//! Two reasons, and the second is the one that matters.
//!
//! 1. The dialog needs a handful of values (a placement, a sheet size, a
//!    resolution verdict), not the whole crate. Mirroring the values it
//!    reads makes the seam small enough to hold in the head.
//! 2. **No arithmetic is mirrored.** There is no `place_page` here, no
//!    `sequence()`, no `job_resolution`, no `plan_job` — those are
//!    `pdfcer-print`'s, they are tested there, and a second copy would be the
//!    failure this project already names about range parsers: *"two range
//!    parsers would eventually disagree about something like `5,1-2` … and an
//!    operator moving between the GUI and a script would have no way to know
//!    which one they were talking to."* The same is true, with paper at
//!    stake, of two placement calculations. So [`plan`] is a **hole**, not an
//!    implementation: it either calls the engine or it refuses.
//!
//! ## What is deliberately NOT here
//!
//! **Imposition — n-up, booklet, poster.** `FEATURES.md` records it as
//! `core — · cli [x] · gui [ ]`, and the roadmap names the prerequisite:
//! *"needs the sheet composition extracted into `pdfcer-print` so both shells
//! share one implementation."* Until that lands, an imposition control in
//! this dialog would be an affordance for something that cannot happen, which
//! is precisely what the no-placeholders rule forbids. When it does land it
//! is **one new tab**, not a change to the three that exist: n-up, booklet
//! and poster remap the *job* rather than scale a page, and
//! `docs/core-api/03` §6.4 records that the mutual-exclusion guard between
//! them is **CLI-local** — *"`pdfcer-print` will not stop you. A new GUI shell
//! must re-implement this guard."* That guard is the first thing that tab
//! owes.

use std::fmt;

mod device;

// ★ Re-exported rather than left as `spooler::device::…`.
//
// The dialog is written against `spooler::` as ONE vocabulary, and it was
// written that way before this module became a directory. A split that made
// forty call sites choose between two paths would have moved a private
// implementation detail — where a type happens to live — into every file
// that names one. The seam is real and worth having; it is not worth
// spending the caller's attention on.
pub(crate) use device::{
    DeviceFeatures, DriverConfig, FormSourceSupport, PaperForm, Printer, device_features,
    edit_printer_configuration, list_printers, printer_forms,
};

// ---------------------------------------------------------------------------
// Failure
// ---------------------------------------------------------------------------

/// Why the print system could not be reached.
///
/// # Two variants, mapping onto two of the three sentences
///
/// [`crate::text::print`]'s header sets out three ways to have no printer,
/// deliberately said three ways. Two of them are failures and live here; the
/// third is not a failure at all and therefore has no variant:
///
/// | condition | represented by | sentence |
/// |---|---|---|
/// | pdfcer could not ask this system about printers **at all** | [`Unavailable::Spooler`] | [`crate::text::print::spooler_unavailable`] |
/// | this *particular* device would not describe itself | [`Unavailable::Device`] | [`crate::text::print::device_unavailable`] |
/// | the spooler answered and reported none installed | `Ok(vec![])` — **not an error** | [`crate::text::print::no_printers`] |
///
/// The third row is the one worth stating explicitly, because collapsing it
/// into the first is the exact defect `pdfcer-print` names: a machine with no
/// printers installed is a *normal machine*, and reporting that as a failure
/// sends an operator looking for a fault that does not exist. The engine
/// returns an empty `Vec` there and this type has nowhere to put one, which
/// is the type system holding the distinction rather than a convention.
///
/// # Why a `String` rather than the engine's `PrintError`
///
/// `PrintError` is `Debug + Clone` and neither `Copy` nor `Eq`, and this
/// value is stored in dialog state, compared in tests, and copied into trace
/// lines. Carrying the engine's own `Display` output — which is written as
/// operator-facing prose, complete with the remedy — keeps every one of those
/// cheap while losing nothing: nothing in the shell branches on *which*
/// `PrintError` it was, only on which of the two rows above applies, and that
/// is what the variant already encodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Unavailable {
    /// The spooler itself could not be queried — `list_printers` or
    /// `device_features` returned `Err`. Carries `PrintError`'s own sentence.
    Spooler(String),
    /// One device would not describe itself — `printer_caps` returned `Err`
    /// for this printer. Carries `PrintError`'s own sentence.
    ///
    /// Distinct from [`Self::Spooler`] because the remedy is different:
    /// *pick another printer* rather than *there is nothing to pick from*.
    Device(String),
}

impl fmt::Display for Unavailable {
    /// **Diagnostic text, not operator copy.**
    ///
    /// It reaches a `PDFCER_DIAG` trace line and
    /// [`crate::text::print::failed`]'s `detail` argument — which is the same
    /// passing-through of a structured engine error that
    /// [`crate::text::canvas_render_failed`] does, and for the same reason:
    /// the engine's own sentence is the specific half.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spooler(detail) | Self::Device(detail) => f.write_str(detail),
        }
    }
}

// ---------------------------------------------------------------------------
// Job description — what the operator's answers become
// ---------------------------------------------------------------------------

/// How a page is sized onto the sheet.
///
/// **Four modes, not three**, and the fourth is not a rounding error:
/// `pdfcer-print` keeps `Fit` and `ShrinkOversized` apart because collapsing
/// them — *"the natural simplification"* — *"silently blows a business card
/// up to A4"* (`lib.rs:490-494`). Fit scales in both directions; Shrink only
/// ever reduces.
///
/// Maps to `pdfcer_print::ScaleMode`, variant for variant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ScaleMode {
    /// Scale to fill the printable area, up or down, preserving aspect.
    Fit,
    /// One PDF point to 1/72 inch of paper, whatever that costs.
    ActualSize,
    /// Like [`Self::ActualSize`], except an oversized page is reduced.
    ShrinkOversized,
    /// An explicit multiplier, where `1.0` is actual size.
    Custom(f64),
}

/// Odd/even filtering, applied **over** a page range rather than instead of
/// one — "pages 1-10, even only" is a thing operators ask for.
///
/// Maps to `pdfcer_print::PageSubset`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum PageSubset {
    /// No filtering.
    #[default]
    All,
    /// Document pages 1, 3, 5 … — **the numbers printed on the paper**.
    Odd,
    /// Document pages 2, 4, 6 ….
    Even,
}

/// Copy ordering. Maps to `pdfcer_print::Collate`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Collate {
    /// The whole set, then the whole set again.
    #[default]
    Collated,
    /// Every copy of page 1, then every copy of page 2.
    Uncollated,
}

/// Which way up the sheet is fed. Maps to `pdfcer_print::Orientation`.
///
/// `Auto` is resolved **per page** from the page's own aspect, which is what
/// keeps a document mixing portrait text with a landscape drawing upright
/// throughout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Orientation {
    /// Choose from each page's own shape.
    #[default]
    Auto,
    /// Force portrait.
    Portrait,
    /// Force landscape.
    Landscape,
}

/// Two-sided printing. Maps to `pdfcer_print::Duplex`.
///
/// **Driver-gated, never simulated.** pdfcer will not fake duplex by
/// reordering pages and asking the operator to reinsert the stack: *"that is
/// a workflow with a documented mis-assembly failure mode, and offering it as
/// though it were duplex would be claiming a capability the hardware does not
/// have."* [`DeviceFeatures::supports_duplex`] is what the dialog consults
/// before drawing the control at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Duplex {
    /// One side only — also what a device that cannot duplex does.
    #[default]
    Simplex,
    /// Two-sided, flipped on the long edge: the usual book binding.
    LongEdge,
    /// Two-sided, flipped on the short edge: notepad binding.
    ShortEdge,
}

/// The arithmetic half of a job: which pages, at what size, in what order.
///
/// Maps to `pdfcer_print::JobSpec`, field for field. **Kept separate from
/// [`DeviceSettings`]** for the engine's own reason: everything here is
/// arithmetic pdfcer performs and can be exact about, and everything there is
/// a *request to the driver* which the driver may quietly decline. Presenting
/// both as though pdfcer controlled them is what makes a job silently come out
/// single-sided with nothing to say so.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct JobSpec {
    /// Zero-based page indices, in document order.
    pub(crate) pages: Vec<usize>,
    /// How each page is sized onto the sheet.
    pub(crate) mode: ScaleMode,
    /// Upper bound on rendering resolution, in DPI. **A memory bound, not a
    /// quality preference** — and one pdfcer chose rather than the operator,
    /// which is why [`JobResolution::capped`] must be disclosed.
    pub(crate) max_dpi: u32,
    /// Odd/even filtering, applied over [`Self::pages`].
    pub(crate) subset: PageSubset,
    /// Send the sequence back to front.
    pub(crate) reverse: bool,
    /// How many copies.
    pub(crate) copies: u16,
    /// Copy ordering.
    pub(crate) collate: Collate,
}

/// Which sheet the driver is asked to feed. Maps to
/// `pdfcer_print::PaperSelection`.
///
/// # ★ Why choosing paper is a REQUEST and not a setting
///
/// `pdfcer-print` reported, while building this: **two drivers were found
/// silently ignoring a paper request.** The `DEVMODE` is handed over with
/// `DM_PAPERSIZE` asserted, the driver is free to do as it likes with it, and
/// nothing comes back to say it declined. There is no acknowledgement in the
/// Win32 API to read and none to invent.
///
/// That is a fact pdfcer cannot verify and the operator cannot see, which puts
/// it squarely under rule 4 — *fuzzy, never sneaky*. The disclosure is
/// [`crate::text::print::paper_is_a_request`], off-canvas, in words, beside
/// the control that makes the choice. It is **not** a warning icon on the
/// preview and **not** a differently-styled sheet outline: the preview draws
/// the sheet the job was planned for, exactly as it would draw any other, and
/// pdfcer's uncertainty about the driver is reported in text next to it.
///
/// # Why there is no `Custom` variant here when the engine has one
///
/// Because there is no surface to type a size into. The engine's
/// `PaperSelection::Custom` takes a sheet in tenths of a millimetre and is
/// reachable through the driver's own properties dialog — an operator who
/// needs a 900 mm roll length sets it there, and
/// [`super::device::ConfigSummary::custom_paper_pt`] is read back so the
/// dialog can say what it holds. Mirroring a variant this shell cannot
/// construct would be a value with no producer; recorded in `NO_SURFACE.md`
/// rather than half-built here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum PaperChoice {
    /// Say nothing about paper.
    ///
    /// **Not "Letter" and not "whatever page 1 is"** — genuinely silent. The
    /// device prints on whatever its own Windows settings name, which is what
    /// this build did exclusively until 2026-08-18 and is still the default.
    ///
    /// When a [`super::device::DriverConfig`] is also held, this means "the
    /// sheet that configuration holds", since the configuration is amended
    /// rather than replaced and pdfcer asserts nothing over it.
    #[default]
    DeviceDefault,
    /// A form the driver enumerates — a [`super::device::PaperForm::id`].
    Form(u16),
}

/// The driver half of a job: what pdfcer asks the device to do.
///
/// Maps to `pdfcer_print::DeviceSettings`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct DeviceSettings {
    /// Sheet orientation.
    pub(crate) orientation: Orientation,
    /// Two-sided printing, if the device supports it.
    pub(crate) duplex: Duplex,
    /// Ask the driver to pick the input tray from each page's size.
    pub(crate) pick_tray_by_page_size: bool,
    /// Which sheet to feed. **A request the driver may decline** — see
    /// [`PaperChoice`].
    pub(crate) paper: PaperChoice,
}

// ---------------------------------------------------------------------------
// Job description — what comes back
// ---------------------------------------------------------------------------

/// The sheet, the printable area within it, and the resolution — **already
/// turned for this job's orientation**.
///
/// Maps to `pdfcer_print::DeviceGeometry`.
///
/// # ★ Turned, and that word is the whole defect this type prevents
///
/// `printer_caps` reports the device's *default* `DEVMODE`. On a
/// portrait-default printer that is a portrait printable area — so a
/// landscape job planned against it under-scales every page to about 77 % of
/// correct size, leaves a wide empty margin, and **reports no clip**, so
/// nothing says it happened. The engine removed the `From` impl that made
/// that mistake reachable, *"because a wrong answer that is one `.into()`
/// away will be reached again"*, leaving `DeviceGeometry::from_caps` as the
/// only route — and it cannot be called without stating the orientation and
/// the first page.
///
/// The port honours that by not exposing raw capabilities at all: [`plan`]
/// takes the orientation and the page sizes and hands back a geometry that
/// has already been turned, so the picture the preview draws and the paper
/// the job lands on are the same claim.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DeviceGeometry {
    /// Resolution in dots per inch, horizontal and vertical.
    ///
    /// A pair rather than one number because asymmetric devices are real
    /// (600×300 on some plotters), and the engine renders at the **smaller**
    /// axis so the driver is not left to resample.
    pub(crate) dpi: (u32, u32),
    /// The printable area in points — smaller than the sheet by the
    /// unprintable margins the driver reports.
    pub(crate) printable_pt: (f64, f64),
    /// The full sheet in points.
    pub(crate) physical_pt: (f64, f64),
    /// Where the printable area starts relative to the sheet corner, in
    /// points: the top-left unprintable margin.
    pub(crate) offset_pt: (f64, f64),
}

/// Where and how big one page lands on the sheet.
///
/// Maps to `pdfcer_print::Placement`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Placement {
    /// Multiplier from PDF points to paper points.
    pub(crate) scale: f64,
    /// Offset within the *printable area*, in paper points.
    pub(crate) offset_x_pt: f64,
    /// Vertical offset, same units.
    pub(crate) offset_y_pt: f64,
    /// **The scaled page does not fit and will lose content off the edges.**
    ///
    /// Acrobat clips silently here. pdfcer reports it — the operator's
    /// standing ruling that parity is a floor — and this flag is the whole
    /// reason the preview hatches, the caption counts, and the commit
    /// button's own label carries the number.
    pub(crate) clipped: bool,
}

/// Where one page lands, and how big to render it.
///
/// Maps to `pdfcer_print::PagePlan`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PagePlan {
    /// The **document** page this describes, zero-based.
    ///
    /// ★ Not a position in the plan list. The plan list is the job's
    /// *sequence* — subset-filtered, possibly reversed, possibly repeated for
    /// copies — so the two coincide only for a whole-document forward job.
    /// Indexing page sizes by a plan's position rather than by this field is
    /// a live defect the salvaged preview carries a comment about; see
    /// [`super::preview`].
    pub(crate) index: usize,
    /// Placement on the sheet.
    pub(crate) placement: Placement,
    /// The scale to rasterise at, in device pixels per PDF point.
    ///
    /// **Already carries the print scale** (`dpi / 72 × placement.scale`), so
    /// the pixels handed to the spooler are the size they will occupy on
    /// paper and the blit is a 1:1 copy. Rendering at device resolution and
    /// letting the driver stretch resamples twice, and on a CAD drawing —
    /// whose value is thin lines — that is the difference an operator notices
    /// first.
    pub(crate) render_scale: f64,
}

/// The resolution a job will render at, and whether pdfcer's cap bound.
///
/// Maps to `pdfcer_print::JobResolution`, plus one value flattened: the engine
/// exposes `uncapped_page_mb()` as a method, and it is carried here as a
/// field so no formula of the engine's is restated in this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct JobResolution {
    /// The DPI actually used.
    pub(crate) dpi: u32,
    /// The device's own resolution, before the cap.
    pub(crate) device_dpi: u32,
    /// Whether [`JobSpec::max_dpi`] reduced it.
    ///
    /// **The case that must be disclosed**: pdfcer chose a number the operator
    /// did not, by pdfcer's own memory judgement.
    pub(crate) capped: bool,
    /// Roughly what one page at the *device's* resolution would cost, in
    /// megabytes — the number that justifies the cap, from
    /// `JobResolution::uncapped_page_mb`.
    pub(crate) uncapped_page_mb: u64,
}

/// A job, planned: the turned geometry, the resolution verdict, and one entry
/// per sheet in the order it will be sent.
///
/// # Why one struct rather than three calls
///
/// The three come from the same three engine calls, in a fixed order, against
/// the same inputs — and getting the order wrong is exactly the orientation
/// defect described on [`DeviceGeometry`]. Returning them together means the
/// dialog cannot plan against one geometry and preview against another.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Job {
    /// The sheet this job was planned against, already turned.
    pub(crate) device: DeviceGeometry,
    /// What resolution it will render at, and whether pdfcer capped it.
    pub(crate) resolution: JobResolution,
    /// One entry per sheet, in send order.
    pub(crate) plans: Vec<PagePlan>,
}

impl Job {
    /// How many sheets of this job will lose content off an edge.
    ///
    /// Counted over the **whole job**, not the sheet on screen, because a
    /// multi-page job's clip is usually on a sheet the operator is not
    /// looking at. This one number reaches three surfaces — the preview
    /// caption, the commit button's label, and the trace — and it is computed
    /// in one place so they cannot disagree.
    pub(crate) fn clipped(&self) -> usize {
        self.plans.iter().filter(|p| p.placement.clipped).count()
    }
}

/// One rendered page, ready to blit.
///
/// Maps to `pdfcer_print::PageBitmap`. **RGBA8, row-major, top row first** —
/// i.e. `pixmap.data().to_vec()` handed over unchanged, premultiplied, with
/// no conversion in between. The engine is explicit that this is the
/// contract; re-encoding it here would be a second colour convention of
/// exactly the kind [`crate::render::raster`]'s header exists to prevent.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PageBitmap {
    /// Width in device pixels.
    pub(crate) width: u32,
    /// Height in device pixels.
    pub(crate) height: u32,
    /// The pixels, premultiplied RGBA8.
    pub(crate) rgba: Vec<u8>,
    /// Where this page lands on the sheet.
    pub(crate) placement: Placement,
    /// The page's own size in PDF points — what the driver picks paper from.
    pub(crate) page_pt: (f64, f64),
}

/// What a spool attempt did. Maps to `pdfcer_print::SpoolReport`.
///
/// **Never constructed in this build**, because [`spool`] cannot succeed
/// here. The `allow` is scoped to this one type and names the condition that
/// removes it, following the precedent `crate::viewer` sets for salvaged
/// items whose first consumer arrives in a later stage. Deleting the type
/// instead would mean the footer had no shape to render a success into, and
/// the day the manifest line lands the success path would be written from
/// scratch rather than reviewed.
#[allow(
    dead_code,
    reason = "constructed by the adapter once pdfcer-print is linked; see the module header" // ui-text-exempt: lint justification, never displayed
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpoolReport {
    /// Pages sent.
    pub(crate) pages: usize,
    /// Whether a job was actually started.
    pub(crate) printed: bool,
    /// The device's reported resolution.
    pub(crate) dpi: (i32, i32),
    /// Pages whose placement reported [`Placement::clipped`].
    pub(crate) clipped_pages: usize,
    /// The spooler's job ID, when one was started.
    pub(crate) job_id: Option<u32>,
    /// Where the `DEVMODE` this job was sent with came from.
    ///
    /// ★ One of its four values is a disclosure the operator would otherwise
    /// never learn — see [`SettingsSource`].
    pub(crate) settings_source: SettingsSource,
}

/// Where the `DEVMODE` a job was sent with came from. Maps to
/// `pdfcer_print::SettingsSource`.
///
/// # ★ Why a shell must report this, and why it cannot be inferred
///
/// pdfcer writes at most four members of a `DEVMODE`. Everything else a device
/// does — media type, print quality, colour handling, stapling, output bin,
/// the entire vendor-private half — lives in the driver's own configuration,
/// which pdfcer carries through untouched **when it has one**.
///
/// [`Self::Synthesised`] is the case where it did not. The driver refused to
/// report its settings, so the job went out carrying only what pdfcer sets
/// itself and everything the driver held was lost. **The job still prints**,
/// which is exactly what makes this dangerous: the operator gets paper, and
/// the paper is wrong in ways — plain instead of glossy, draft instead of
/// best — that look like a printer problem rather than a pdfcer one.
///
/// It is not visible from the printed page, not visible from the dialog, and
/// not derivable from anything the shell knows before the call. The engine
/// reports it because it is the only party that can, and the shell says it
/// out loud for the same reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SettingsSource {
    /// No `DEVMODE` was sent at all — nothing pdfcer controls differs from
    /// what the device is already set to. The cheapest, most conservative
    /// case, and the common one. Nothing to disclose.
    #[default]
    DeviceDefault,
    /// The driver's own current settings were fetched and amended. The normal
    /// case for a job that changes anything. Nothing to disclose.
    DriverSupplied,
    /// A configuration the operator produced — from the driver's own
    /// properties dialog — was amended. Nothing to disclose; it is what they
    /// asked for.
    CallerSupplied,
    /// ★ The driver would not report its settings and pdfcer synthesised one.
    /// **Disclosed** — see the type's own docs.
    Synthesised,
}

// ---------------------------------------------------------------------------
// Conversions — this crate's view of a job, into the engine's
// ---------------------------------------------------------------------------
//
// Every function in this block is a total, field-for-field mapping with no
// arithmetic in it. That is the whole discipline: if a conversion here ever
// grows a computation, the second implementation the module header warns
// about has arrived and it has arrived in the least visible place.
//
// They are written as free functions rather than `From` impls deliberately.
// `DeviceGeometry` is the reason: `pdfcer-print` REMOVED its own
// `From<&PrinterCaps>` impl because a wrong answer that is one `.into()` away
// will be reached again — the geometry must be turned for the job's
// orientation, and an ergonomic conversion is exactly what hides that it was
// not. Following the same posture for the whole set keeps the two directions
// looking alike.

/// The operator's scaling choice, into the engine's.
const fn to_engine_scale(mode: ScaleMode) -> pdfcer_print::ScaleMode {
    match mode {
        ScaleMode::Fit => pdfcer_print::ScaleMode::Fit,
        ScaleMode::ActualSize => pdfcer_print::ScaleMode::ActualSize,
        ScaleMode::ShrinkOversized => pdfcer_print::ScaleMode::ShrinkOversized,
        ScaleMode::Custom(factor) => pdfcer_print::ScaleMode::Custom(factor),
    }
}

/// The odd/even filter, into the engine's.
const fn to_engine_subset(subset: PageSubset) -> pdfcer_print::PageSubset {
    match subset {
        PageSubset::All => pdfcer_print::PageSubset::All,
        PageSubset::Odd => pdfcer_print::PageSubset::Odd,
        PageSubset::Even => pdfcer_print::PageSubset::Even,
    }
}

/// The copy ordering, into the engine's.
const fn to_engine_collate(collate: Collate) -> pdfcer_print::Collate {
    match collate {
        Collate::Collated => pdfcer_print::Collate::Collated,
        Collate::Uncollated => pdfcer_print::Collate::Uncollated,
    }
}

/// The sheet orientation, into the engine's.
const fn to_engine_orientation(orientation: Orientation) -> pdfcer_print::Orientation {
    match orientation {
        Orientation::Auto => pdfcer_print::Orientation::Auto,
        Orientation::Portrait => pdfcer_print::Orientation::Portrait,
        Orientation::Landscape => pdfcer_print::Orientation::Landscape,
    }
}

/// The duplex request, into the engine's.
const fn to_engine_duplex(duplex: Duplex) -> pdfcer_print::Duplex {
    match duplex {
        Duplex::Simplex => pdfcer_print::Duplex::Simplex,
        Duplex::LongEdge => pdfcer_print::Duplex::LongEdge,
        Duplex::ShortEdge => pdfcer_print::Duplex::ShortEdge,
    }
}

/// The driver half of a job, into the engine's.
const fn to_engine_settings(settings: DeviceSettings) -> pdfcer_print::DeviceSettings {
    pdfcer_print::DeviceSettings {
        orientation: to_engine_orientation(settings.orientation),
        duplex: to_engine_duplex(settings.duplex),
        pick_tray_by_page_size: settings.pick_tray_by_page_size,
        paper: to_engine_paper(settings.paper),
    }
}

/// The paper request, into the engine's.
///
/// # ★ What pdfcer asserts over a driver configuration, and what it leaves
///
/// The engine amends a `DEVMODE` rather than replacing it, and the members
/// named in [`DeviceSettings`] win over whatever the configuration held. So
/// it matters which members those are, and the list is not symmetrical:
///
/// | member | asserted | consequence |
/// |---|---|---|
/// | orientation | **always** | an orientation the operator set in the driver's own dialog is overridden by this dialog's radios. `Auto` resolves per page, which is nearly always what a mixed CAD set wants — but it *is* an override, and the properties disclosure says so |
/// | paper | only when [`PaperChoice::Form`] | `DeviceDefault` leaves the configuration's own sheet standing |
/// | duplex | only when non-default | asserting `DMDUP_SIMPLEX` unconditionally would silently cancel a driver's own duplex default — a defect the engine names in `apply`'s notes |
/// | tray | only when the checkbox is on | same reasoning |
///
/// Everything else — media type, quality, colour handling, stapling, output
/// bin, the whole driver-private tail — is carried through untouched.
const fn to_engine_paper(paper: PaperChoice) -> pdfcer_print::PaperSelection {
    match paper {
        PaperChoice::DeviceDefault => pdfcer_print::PaperSelection::DeviceDefault,
        PaperChoice::Form(id) => pdfcer_print::PaperSelection::Form(id),
    }
}

/// The arithmetic half of a job, into the engine's.
///
/// `pages` is cloned rather than moved because [`plan`] takes `&JobSpec` — the
/// dialog rebuilds its spec every frame from the operator's current answers
/// and keeps ownership of it, and a signature that consumed the spec would
/// force a clone at every call site instead of the one here.
fn to_engine_spec(spec: &JobSpec) -> pdfcer_print::JobSpec {
    pdfcer_print::JobSpec {
        pages: spec.pages.clone(),
        mode: to_engine_scale(spec.mode),
        max_dpi: spec.max_dpi,
        subset: to_engine_subset(spec.subset),
        reverse: spec.reverse,
        copies: spec.copies,
        collate: to_engine_collate(spec.collate),
    }
}

/// A placement, out of the engine.
const fn from_engine_placement(placement: pdfcer_print::Placement) -> Placement {
    Placement {
        scale: placement.scale,
        offset_x_pt: placement.offset_x_pt,
        offset_y_pt: placement.offset_y_pt,
        clipped: placement.clipped,
    }
}

/// One rendered sheet, into the engine's.
///
/// The pixel buffer is **cloned**, and that is a deliberate cost rather than
/// an oversight. [`spool`] takes `&[PageBitmap]` because the dialog's commit
/// path builds the whole set and then hands it over; taking the vector by
/// value would let the copy be avoided, but it would also mean the commit
/// path could not re-attempt a spool without re-rendering every page. On a
/// job large enough for the copy to matter, re-rendering is the far larger
/// cost.
fn to_engine_bitmap(bitmap: &PageBitmap) -> pdfcer_print::PageBitmap {
    pdfcer_print::PageBitmap {
        width: bitmap.width,
        height: bitmap.height,
        rgba: bitmap.rgba.clone(),
        placement: pdfcer_print::Placement {
            scale: bitmap.placement.scale,
            offset_x_pt: bitmap.placement.offset_x_pt,
            offset_y_pt: bitmap.placement.offset_y_pt,
            clipped: bitmap.placement.clipped,
        },
        page_pt: bitmap.page_pt,
    }
}

// ---------------------------------------------------------------------------
// The two calls that DO the job
// ---------------------------------------------------------------------------
//
// These, and the two device queries in [`device`], were four holes with
// refusals in them for the whole of v0.1.0; see the module header for what
// that cost and why. Nothing here computes a placement, a sequence or a
// resolution — see the module docs on why a second implementation of any of
// those is worse than no implementation at all.

/// Plan the whole job: turn the geometry, resolve the resolution, place every
/// page.
///
/// # The three engine calls this stands in for, and their order
///
/// ```text
/// let caps   = pdfcer_print::printer_caps(printer)?;
/// let device = pdfcer_print::DeviceGeometry::from_caps(       // 1. TURN FIRST
///     &caps, settings.orientation, spec.first_page_pt(page_sizes));
/// let res    = pdfcer_print::job_resolution(&device, &spec);  // 2.
/// let plans  = pdfcer_print::plan_job(&device, page_sizes, &spec);   // 3.
/// ```
///
/// The order is not stylistic. Steps 2 and 3 both take `&device`, so a
/// geometry turned *after* them would leave the dialog previewing a sheet the
/// job was not planned for — the 77 %-scale defect described on
/// [`DeviceGeometry`], reintroduced by sequencing rather than by a `From`
/// impl.
///
/// `spec.first_page_pt(page_sizes)` is the **first page the job sends**, not
/// `page_sizes[0]`: the sequence may be subset-filtered or reversed, and the
/// `DEVMODE` and the geometry rotation must resolve `Auto` from the same
/// page. Taking an index of our own here is how the two come to disagree.
///
/// # Errors
///
/// [`Unavailable::Device`] when this *particular* device would not describe
/// itself, which is a different sentence from having no printers at all —
/// [`crate::text::print::device_unavailable`] rather than
/// [`crate::text::print::spooler_unavailable`].
pub(crate) fn plan(
    printer: &str,
    settings: DeviceSettings,
    config: Option<&DriverConfig>,
    page_sizes: &[(f64, f64)],
    spec: &JobSpec,
) -> Result<Job, Unavailable> {
    let engine_spec = to_engine_spec(spec);

    // 1. CAPABILITIES, ★ FOR THE SHEET THIS JOB WILL ACTUALLY USE.
    //
    //    `printer_caps` — the function this used to call — opens an
    //    information DC with the device's DEFAULT `DEVMODE` and reports the
    //    geometry of whatever sheet THAT names. Every placement below is
    //    computed against it. So the moment paper became choosable, a job
    //    asking for A3 on a Letter-default device would have been PLANNED for
    //    Letter and PRINTED on A3 — the preview and the paper describing
    //    different sheets, with no clip reported and nothing to explain it.
    //
    //    That is the same failure as the un-turned geometry described on
    //    [`DeviceGeometry`], arriving through a second dimension. The engine
    //    closed it by giving the query the sheet: `printer_caps_for` takes the
    //    configuration and the paper request and reports the geometry of the
    //    result. Passing them here is not optional and is not a refinement.
    //
    //    It is also the only fallible step: everything after it is arithmetic
    //    over values already in hand.
    let caps = match pdfcer_print::printer_caps_for(
        printer,
        config.map(DriverConfig::engine),
        to_engine_paper(settings.paper),
    ) {
        Ok(caps) => caps,
        Err(error) => {
            // Traced rather than discarded. A refusal is exactly the event a
            // harness needs to see, and reading every operand here is also
            // what keeps the shape of [`JobSpec`] honest — a field nothing
            // ever reads is a field that can quietly acquire the wrong units
            // or stop being filled at all.
            let detail = error.to_string();
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "print-plan-refused printer={printer} pages={:?} mode={:?} max_dpi={} \
                     subset={:?} reverse={} copies={} collate={:?} orientation={:?} \
                     duplex={:?} tray={} sizes={} reason={detail}",
                    spec.pages,
                    spec.mode,
                    spec.max_dpi,
                    spec.subset,
                    spec.reverse,
                    spec.copies,
                    spec.collate,
                    settings.orientation,
                    settings.duplex,
                    settings.pick_tray_by_page_size,
                    page_sizes.len(),
                )
            });
            return Err(Unavailable::Device(detail));
        }
    };

    // 2. ★ TURN THE GEOMETRY FIRST. Steps 3 and 4 both take `&device`, so a
    //    geometry turned after them would leave the dialog previewing a sheet
    //    the job was not planned for — the 77 %-scale defect described on
    //    [`DeviceGeometry`], reintroduced by sequencing rather than by a
    //    `From` impl.
    //
    //    `first_page_pt` is the FIRST PAGE THE JOB SENDS, not `page_sizes[0]`:
    //    the sequence may be subset-filtered or reversed, and the `DEVMODE`
    //    and the geometry rotation must resolve `Auto` from the same page.
    //    Taking an index of our own here is how the two come to disagree, so
    //    the engine's own accessor is used rather than an index.
    let device = pdfcer_print::DeviceGeometry::from_caps(
        &caps,
        to_engine_orientation(settings.orientation),
        engine_spec.first_page_pt(page_sizes),
    );

    // 3. Resolution, against the turned geometry.
    let resolution = pdfcer_print::job_resolution(&device, &engine_spec);

    // 4. Every page placed, against the same turned geometry.
    let plans = pdfcer_print::plan_job(&device, page_sizes, &engine_spec);

    Ok(Job {
        device: DeviceGeometry {
            dpi: device.dpi,
            printable_pt: device.printable_pt,
            physical_pt: device.physical_pt,
            offset_pt: device.offset_pt,
        },
        resolution: JobResolution {
            dpi: resolution.dpi,
            device_dpi: resolution.device_dpi,
            capped: resolution.capped,
            // Flattened from the engine's method to a field, so no formula of
            // the engine's is restated in this crate. See [`JobResolution`].
            uncapped_page_mb: resolution.uncapped_page_mb(),
        },
        plans: plans
            .into_iter()
            .map(|plan| PagePlan {
                index: plan.index,
                placement: from_engine_placement(plan.placement),
                render_scale: plan.render_scale,
            })
            .collect(),
    })
}

/// Hand the rendered sheets to the spooler.
///
/// Fill with
/// `pdfcer_print::spool(printer, &bitmaps, DryRun::No, None, settings, first_page_pt)`.
///
/// # ★ This is the one call in the application that consumes paper
///
/// `pdfcer-print`'s own header: *"Printing consumes paper, occupies a device
/// other people may share, and cannot be undone. Nothing in this crate starts
/// a job as a side effect of anything else: `spool` is the only function that
/// reaches `StartDoc`, and it is reached only from a control an operator
/// deliberately clicked."* The shell's half of that contract is that this
/// function is reached from **one** place — the commit button — and from no
/// keyboard chord, no dispatch arm and no frame-loop condition.
///
/// `first_page_pt` must come from `bitmaps.first()`, never from the
/// document's page 0: a reversed or range-filtered job sends a different page
/// first, and the driver picks its paper from whichever one it is handed.
///
/// # Errors
///
/// [`Unavailable::Spooler`] carrying whatever the spooler reported — passed
/// through to the operator verbatim by [`crate::text::print::failed`],
/// because a structured spooler error is the specific half of that sentence.
pub(crate) fn spool(
    printer: &str,
    bitmaps: &[PageBitmap],
    settings: DeviceSettings,
    config: Option<&DriverConfig>,
    first_page_pt: (f64, f64),
) -> Result<SpoolReport, Unavailable> {
    // Traced BEFORE the call, not after, and that ordering is the point: this
    // is the one call in the application that consumes paper, and if it hangs
    // or takes the process down, the trace line describing what was sent is
    // the only record that survives. Reading every operand here is also what
    // keeps the shape of `PageBitmap` honest — a field nothing ever reads is
    // a field that can quietly acquire the wrong units.
    crate::diag::trace(|| {
        let bytes: usize = bitmaps.iter().map(|b| b.rgba.len()).sum();
        let first = bitmaps.first();
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "print-spool printer={printer} sheets={} px={:?} bytes={bytes} \
             first_page_pt={first_page_pt:?} placement={:?} page_pt={:?} \
             orientation={:?} duplex={:?} tray={}",
            bitmaps.len(),
            first.map(|b| (b.width, b.height)),
            first.map(|b| b.placement),
            first.map(|b| b.page_pt),
            settings.orientation,
            settings.duplex,
            settings.pick_tray_by_page_size,
        )
    });

    let pages: Vec<pdfcer_print::PageBitmap> = bitmaps.iter().map(to_engine_bitmap).collect();

    // ★ `DryRun::No` and `output: None`, both hard-coded, both deliberate.
    //
    // The CLI defaults to a dry run and requires `--send`, and that is right
    // *there*: a command line has no confirmation step of its own, so the
    // flag is the confirmation. Here the DIALOG is the confirmation — the
    // operator chose a printer, read a clip count in the button's own label,
    // and pressed it. A dry-run toggle on this surface would be a second gate
    // whose only effect is to make the first one mean less.
    // ★ `spool_with_config` rather than `spool`, always — including when
    // there is no configuration, where `None` makes it the same call.
    //
    // One call site rather than two branches: a shell that chose between two
    // spool functions would have two paths to the one irreversible operation
    // in the application, and the rarer one would be the one nobody drove.
    let outcome = pdfcer_print::spool_with_config(
        printer,
        &pages,
        pdfcer_print::DryRun::No,
        None,
        to_engine_settings(settings),
        first_page_pt,
        config.map(DriverConfig::engine),
    );

    match outcome {
        Ok(report) => {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "print-spool-done printer={printer} pages={} printed={} dpi={:?} \
                     clipped={} job_id={:?}",
                    report.pages, report.printed, report.dpi, report.clipped_pages, report.job_id,
                )
            });
            Ok(SpoolReport {
                pages: report.pages,
                printed: report.printed,
                dpi: report.dpi,
                clipped_pages: report.clipped_pages,
                job_id: report.job_id,
                settings_source: match report.settings_source {
                    pdfcer_print::SettingsSource::DeviceDefault => SettingsSource::DeviceDefault,
                    pdfcer_print::SettingsSource::DriverSupplied => SettingsSource::DriverSupplied,
                    pdfcer_print::SettingsSource::CallerSupplied => SettingsSource::CallerSupplied,
                    pdfcer_print::SettingsSource::Synthesised => SettingsSource::Synthesised,
                },
            })
        }
        Err(error) => {
            let detail = error.to_string();
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "print-spool-failed printer={printer} reason={detail}"
                )
            });
            Err(Unavailable::Spooler(detail))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A spec the plan tests can share.
    fn one_page_spec() -> JobSpec {
        JobSpec {
            pages: vec![0],
            mode: ScaleMode::Fit,
            max_dpi: 300,
            subset: PageSubset::All,
            reverse: false,
            copies: 1,
            collate: Collate::Collated,
        }
    }

    /// ★ **The regression test for the defect this module carried for the
    /// whole of v0.1.0.** See the module header.
    ///
    /// # What it asserts, and why the obvious assertion is the wrong one
    ///
    /// The test this replaces asserted that every one of these four functions
    /// **refused**. That was correct while `pdfcer-print` was not a dependency
    /// and became a lock on the defect the moment it was — the manifest line
    /// landed, the refusals stayed, the suite stayed green, and the operator
    /// found out by opening the dialog.
    ///
    /// So this asserts the opposite property, and the wording matters: it
    /// asserts that a call **reaches the engine**, not that it succeeds.
    /// Success is not available to assert. This suite runs on machines with
    /// printers and machines without, on Windows and (at fold-in) not, and a
    /// test that needed a device would be a test that got `#[ignore]`d and
    /// then stopped being run at all.
    ///
    /// The distinguishing evidence is the **failure text**. Every refusal
    /// this module can now produce carries `PrintError`'s own `Display`,
    /// which is written as operator-facing prose; the string the deleted code
    /// produced was `"pdfcer-print is not linked into this build"`. That
    /// sentence can no longer be constructed — the variant that held it does
    /// not exist — so this test is really asserting that the type has the
    /// shape a linked build gives it, which no amount of environment can
    /// fake.
    #[test]
    fn every_call_reaches_the_engine_rather_than_refusing_unconditionally() {
        // Enumeration either succeeds (any number of printers, INCLUDING
        // none — an empty list is a normal machine, not a failure) or fails
        // with the spooler's own words.
        match list_printers() {
            Ok(_) => {}
            Err(Unavailable::Spooler(detail)) => {
                assert!(
                    !detail.is_empty(),
                    "a spooler refusal must carry the engine's sentence, not an empty string"
                );
            }
            Err(Unavailable::Device(detail)) => {
                panic!("enumeration cannot fail for a single device: {detail}")
            }
        }

        // The remaining three are addressed to a printer name chosen so that
        // it cannot resolve on any machine. Each must come back with the
        // ENGINE's refusal, which proves the call was made — a stubbed
        // function could not produce this text.
        const ABSENT: &str = "pdfcer-ui-verify-no-such-printer";

        let features = device_features(ABSENT);
        assert!(
            matches!(features, Err(Unavailable::Spooler(_))),
            "a device that cannot be opened is a spooler-level refusal: {features:?}"
        );

        let planned = plan(
            ABSENT,
            DeviceSettings::default(),
            None,
            &[(612.0, 792.0)],
            &one_page_spec(),
        );
        assert!(
            matches!(planned, Err(Unavailable::Device(_))),
            "a plan against an absent device must refuse as a DEVICE failure, so the dialog \
             says \"choose another printer\" rather than \"there are none\": {planned:?}"
        );

        // ★ Spooling is called with an EMPTY page list. That is not laziness:
        // it is the one input for which `spool` cannot start a job whatever
        // else is true, so a test suite may address it to a real printer name
        // without any risk of consuming paper. The refusal still comes from
        // the engine, because the printer name is resolved before the page
        // count is looked at.
        let spooled = spool(ABSENT, &[], DeviceSettings::default(), None, (612.0, 792.0));
        assert!(
            matches!(spooled, Err(Unavailable::Spooler(_))),
            "spooling to an absent printer must refuse with the spooler's own words: {spooled:?}"
        );
    }

    /// ★ No refusal this module produces is the string the defect produced.
    ///
    /// The narrowest possible statement of the regression, and the one that
    /// would fail if somebody restored a `NotLinked`-shaped shortcut — for
    /// instance by wrapping the four calls in a `cfg` that compiled them out
    /// on a machine where `pdfcer-print` was inconvenient.
    #[test]
    fn no_refusal_claims_the_engine_is_unlinked() {
        let sentences = [
            list_printers().err().map(|e| e.to_string()),
            device_features("pdfcer-ui-verify-no-such-printer")
                .err()
                .map(|e| e.to_string()),
            plan(
                "pdfcer-ui-verify-no-such-printer",
                DeviceSettings::default(),
                None,
                &[(612.0, 792.0)],
                &one_page_spec(),
            )
            .err()
            .map(|e| e.to_string()),
        ];
        for sentence in sentences.into_iter().flatten() {
            assert!(
                !sentence.contains("not linked"),
                "a refusal still claims pdfcer-print is unlinked: {sentence:?}"
            );
        }
    }

    /// Every conversion is total and round-trips the values it carries.
    ///
    /// Not a change-detector: these are the functions through which an
    /// operator's answer becomes paper, and a mapping that sent
    /// `ShrinkOversized` where `Fit` was meant would enlarge a business card
    /// to A4 — the exact collapse `pdfcer-print` keeps two variants apart to
    /// prevent. A wrong arm here is silent until the sheet comes out.
    #[test]
    fn the_conversions_map_every_variant_to_its_own() {
        assert!(matches!(
            to_engine_scale(ScaleMode::Fit),
            pdfcer_print::ScaleMode::Fit
        ));
        assert!(matches!(
            to_engine_scale(ScaleMode::ActualSize),
            pdfcer_print::ScaleMode::ActualSize
        ));
        assert!(matches!(
            to_engine_scale(ScaleMode::ShrinkOversized),
            pdfcer_print::ScaleMode::ShrinkOversized
        ));
        match to_engine_scale(ScaleMode::Custom(0.5)) {
            pdfcer_print::ScaleMode::Custom(factor) => assert!((factor - 0.5).abs() < f64::EPSILON),
            other => panic!("a custom scale lost its multiplier: {other:?}"),
        }

        assert!(matches!(
            to_engine_subset(PageSubset::Odd),
            pdfcer_print::PageSubset::Odd
        ));
        assert!(matches!(
            to_engine_subset(PageSubset::Even),
            pdfcer_print::PageSubset::Even
        ));
        assert!(matches!(
            to_engine_collate(Collate::Uncollated),
            pdfcer_print::Collate::Uncollated
        ));
        assert!(matches!(
            to_engine_orientation(Orientation::Landscape),
            pdfcer_print::Orientation::Landscape
        ));
        assert!(matches!(
            to_engine_duplex(Duplex::ShortEdge),
            pdfcer_print::Duplex::ShortEdge
        ));

        // The spec carries its operands across unchanged. `copies` and
        // `max_dpi` are the two that a transposed field assignment would
        // swap without the compiler noticing, since both are integers.
        let spec = JobSpec {
            pages: vec![3, 1, 4],
            mode: ScaleMode::ActualSize,
            max_dpi: 200,
            subset: PageSubset::Even,
            reverse: true,
            copies: 7,
            collate: Collate::Uncollated,
        };
        let engine = to_engine_spec(&spec);
        assert_eq!(engine.pages, vec![3, 1, 4]);
        assert_eq!(engine.max_dpi, 200);
        assert_eq!(engine.copies, 7);
        assert!(engine.reverse);

        let settings = DeviceSettings {
            orientation: Orientation::Portrait,
            duplex: Duplex::LongEdge,
            pick_tray_by_page_size: true,
            paper: PaperChoice::Form(8),
        };
        let engine = to_engine_settings(settings);
        assert!(matches!(
            engine.orientation,
            pdfcer_print::Orientation::Portrait
        ));
        assert!(matches!(engine.duplex, pdfcer_print::Duplex::LongEdge));
        assert!(engine.pick_tray_by_page_size);
        // ★ The paper id must survive the mapping UNCHANGED. It is a
        // `dmPaperSize` the driver defined, and a conversion that shifted it
        // by one would request a different sheet from the same list — the
        // failure would be paper, not an error.
        assert!(matches!(
            engine.paper,
            pdfcer_print::PaperSelection::Form(8)
        ));
        assert!(matches!(
            to_engine_paper(PaperChoice::DeviceDefault),
            pdfcer_print::PaperSelection::DeviceDefault
        ));
    }

    /// The clip count is over the whole job, and counts sheets not pages.
    ///
    /// Pinned because three surfaces read it — the preview caption, the
    /// commit button's label and the trace — and the entire point of
    /// computing it once is that the button cannot promise a different number
    /// from the caption above it.
    #[test]
    fn the_clip_count_covers_the_whole_job() {
        let placed = |clipped| Placement {
            scale: 1.0,
            offset_x_pt: 0.0,
            offset_y_pt: 0.0,
            clipped,
        };
        let job = Job {
            device: DeviceGeometry {
                dpi: (600, 600),
                printable_pt: (600.0, 780.0),
                physical_pt: (612.0, 792.0),
                offset_pt: (6.0, 6.0),
            },
            resolution: JobResolution {
                dpi: 300,
                device_dpi: 600,
                capped: true,
                uncapped_page_mb: 139,
            },
            plans: vec![
                PagePlan {
                    index: 4,
                    placement: placed(false),
                    render_scale: 4.0,
                },
                PagePlan {
                    index: 0,
                    placement: placed(true),
                    render_scale: 4.0,
                },
                PagePlan {
                    index: 2,
                    placement: placed(true),
                    render_scale: 4.0,
                },
            ],
        };
        assert_eq!(job.clipped(), 2);
        // And a job with nothing clipped reports zero rather than being
        // treated as "unknown" — the commit button's plain label depends on
        // the difference.
        let clean = Job {
            plans: vec![PagePlan {
                index: 0,
                placement: placed(false),
                render_scale: 1.0,
            }],
            ..job
        };
        assert_eq!(clean.clipped(), 0);
    }
}
