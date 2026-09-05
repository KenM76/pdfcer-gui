//! # `dialogs::print` — the print flow: the dialog, its preview, and the spool call
//!
//! ## Printing is the one action in pdfcer with no undo
//!
//! Carried across from the old shell verbatim, because it is the sentence
//! every other decision in this directory follows from:
//!
//! > Everything else this application does can be reverted, closed without
//! > saving, or corrected before a save. A print marks paper, occupies a
//! > device somebody else may share, and cannot be taken back. That single
//! > fact decides most of what is in this file: why the dialog is its own
//! > stationary surface rather than a dock pane, why the preview shows the
//! > printable RECTANGLE and not just the sheet, why Enter does not commit,
//! > and why no keyboard chord spools.
//!
//! ## ★ The dialog IS the confirmation. There is no second gate.
//!
//! > The CLI defaults to a dry run and requires `--send`. That is right for a
//! > scriptable tool whose operator is not watching, and wrong here: a GUI
//! > whose premise is that the operator is looking at the settings does not
//! > also need them to confirm the settings.
//! >
//! > What replaces a second gate is disclosure with teeth — the clip count is
//! > in the BUTTON'S OWN LABEL, so the uncertainty is stated in the
//! > disclosure rather than implied by a confirm step existing (rule 4).
//!
//! Two guards follow from it, and both are inherited rather than re-derived:
//!
//! - **Enter does not print.** An operator reading a dialog and pressing
//!   Enter out of habit must not commit the one action in this application
//!   with no undo. There is a text field here, which makes the habit likelier,
//!   not less. Nothing in this module reads `Key::Enter`.
//! - **No keyboard chord commits.** A chord may *open* this dialog; nothing
//!   spools a job. Reversible actions get chords; the irreversible one does
//!   not.
//!
//! ## Where this module splits, and why there
//!
//! The salvaged source was one 2,022-line file, over the project's 1,500-line
//! ceiling (`R2`). The seam is not a line count — it is the three genuinely
//! separable questions the file answers:
//!
//! | file | question | can be wrong how |
//! |---|---|---|
//! | `mod.rs` (this) | *what is the job, and how does the surface hold together?* | wiring, layout, the commit path |
//! | [`preview`] | *what will the sheet look like?* | **arithmetic** — fit, anchor, raster scale, indexing |
//! | [`tabs`] | *what are the operator's answers?* | **arithmetic** — range parsing |
//! | [`spooler`] | *what does `pdfcer-print` look like from here?* | the port, and nothing else |
//!
//! The split follows testability, exactly as the crate root's own table does:
//! everything that could be *silently* wrong — an anchor term that drifts, a
//! range that recovers from a typo, a plan list indexed by the wrong number —
//! is in a module with unit tests around it, and what is left in this file is
//! wiring that can be reviewed by reading.
//!
//! ## ★ What this build can and cannot do
//!
//! `pdfcer-print` is **not** a dependency of this crate. Every device call
//! therefore refuses, the dialog says so in one sentence, and **the commit
//! button is not drawn at all** — absent rather than greyed, because no
//! setting in this dialog would make this build reach a spooler.
//! [`spooler`]'s header carries the one manifest line that changes that, and
//! the one file it changes.
//!
//! ## What is deliberately absent: imposition
//!
//! N-up, booklet and poster are `cli [x] · gui [ ]` in `FEATURES.md`, blocked
//! on sheet composition being lifted into `pdfcer-print` so both shells share
//! one implementation. An imposition control here would be an affordance for
//! something that cannot happen. See [`spooler`]'s header for what the tab
//! owes when it lands — starting with the mutual-exclusion guard, which is
//! **CLI-local today** and which a GUI must re-implement rather than inherit.
//!
//! ## conventions: dialogs
//!
//! Corpus: `ui-conventions/dialogs.md`.
//!
//! - G1 is-an-os-window: **DONE 2026-08-20** — this was the operator's report,
//!   *"doesn't pop up in its own movable window. It is locked within the
//!   boundaries of the program's window."* It is now a real OS window through
//!   [`crate::dialogs::host`]: title bar, taskbar entry, draggable outside the
//!   application and onto a second monitor. It degrades to the old in-viewport
//!   window on a backend with no multi-viewport support, which is the web
//!   target, and egui owns that fallback rather than this file.
//! - G2 use-the-os-dialog: the file and save pickers are the system's, and
//!   `pdfcer-print` opens the native printer-properties sheet owned by our
//!   window. The dialogs in this directory are pdfcer's own because they carry
//!   choices only pdfcer has — which is the right reason to draw one, and does
//!   not excuse G1.
//! - G3 owned-by-the-app: the native pickers are. The dialog window itself is
//!   **not**, and cannot be: `eframe 0.35`'s `ViewportBuilder` has no owner
//!   option and `egui-winit` never passes egui's own `viewport_parents` down to
//!   `winit`. `crate::dialogs::host`'s header carries the whole account,
//!   including why `with_always_on_top` was considered and refused. **GAP**,
//!   and a named one.
//! - G4 enter-accepts-escape-cancels: **DONE 2026-08-20** — Escape closes, the
//!   OS close button closes, Enter presses Print, and Print is drawn in the
//!   theme's selection fill so an operator can see what Enter will do.
//!   [`crate::dialogs::host::Host::buttons`] owns all three so no dialog can
//!   implement two of them.
//! - G5 keyboard-reachable: **GAP** — egui's tab order is positional and nothing
//!   here asserts that focus starts in a sensible field or that a modal traps
//!   it.
//! - G6 remembers-position: **PARTIAL 2026-08-20** — it comes back where it was
//!   left for as long as the dialog object lives, which is the whole session it
//!   is open. It does **not** survive being closed and reopened, and it does not
//!   survive a restart: a remembered position has to be validated against the
//!   current monitor layout, and a dialog that opens on a monitor which is no
//!   longer attached is worse than one that opens where the platform puts it.
//!   See `dialogs::host`.
//! - G7 destructive-verbs-named: the unsaved-changes dialog names the file and
//!   labels its buttons with verbs rather than Yes/No.
//! - G8 cancel-is-silent: a cancelled picker is a complete, correct,
//!   uninteresting outcome and is never reported as an error.
//! - G9 nothing-blocks-silently: a native picker blocks the UI thread by design,
//!   which is what a modal file dialog is. Long work behind a pdfcer dialog is
//!   not surfaced. **GAP.**

/// **Where a rendered sheet actually carries ink** — operator request O113.
///
/// Split out of [`preview`] rather than added to it, at the seam between
/// *"what do these pixels say"* and *"how is the preview painted"*. The first
/// is pure arithmetic over a byte slice and is fully testable with no GUI at
/// all; the second needs an `egui::Ui`. Keeping them in one file would have
/// put a page of pixel-threshold reasoning in the middle of a painting
/// routine and pushed `preview.rs` toward R2's 1500-line ceiling.
pub(crate) mod ink;
pub(crate) mod layout;
/// **The preview in a window of its own** — operator request O112 ask 2. Its
/// header carries why the feature is one call and one line, and why the print
/// dialog's own column then draws nothing at all.
mod popout;
pub(crate) mod preview;
mod spooler;
pub(crate) mod tabs;

/// **What the operator has actually looked at, and what may be said about the
/// rest** — operator request O113, 2026-09-04. Its own header carries the
/// whole argument: the count, the cache key, and the four sentences.
mod verdicts;

use egui::Ui;

use crate::app::state::OpenDoc;
use crate::dialogs::print::spooler::{
    Collate, DeviceFeatures, DeviceSettings, DriverConfig, Job, JobSpec, PageBitmap, PageSubset,
    PaperChoice, PaperForm, Printer, ScaleMode, SettingsSource, SpoolReport, Unavailable,
};
use crate::dialogs::print::tabs::{PrintRange, PrintTab};
use crate::text::print as t;

/// The **Properties…** button's published region.
///
/// ★ Published for `ui-verify`, which is the only oracle this project trusts
/// for a layout claim. See `tools/ui-verify/src/checks/print_paper.rs` for
/// what it asserts, and for why a driven check reads this rect without ever
/// clicking it.
const REGION_PROPERTIES: &str = "print.properties";

/// The paper selector's published region — the combo itself, closed.
pub(super) const REGION_PAPER: &str = "print.paper";

/// One published region per entry in the OPEN paper list, indexed from zero
/// with the "from the printer's own settings" entry as index 0.
///
/// # Why the ENTRIES are published and not only the combo
///
/// Because a check that can open a list but not choose from it can only
/// assert that a control exists — and "the control exists" is exactly the
/// claim that was true of the tray checkbox for four months while it did
/// nothing. The property worth asserting is that **choosing a sheet changes
/// the plan**, and that needs a click on a specific entry.
///
/// An egui combo popup is an `Area` laid out at paint time; its entries have
/// no position anything outside the process could compute. Publishing them is
/// the only route, and it costs nothing when `PDFCER_DIAG` is unset.
pub(super) const REGION_PAPER_ITEM_PREFIX: &str = "print.paper.item.";

/// The print dialog's live state.
///
/// # Why a dialog struct rather than a dock panel
///
/// Printing is a single transaction with a start and an end, not something an
/// operator dips in and out of while working — which is what a dock pane is
/// for. It is also *modal in spirit and not in mechanism*: nothing blocks the
/// rest of the shell, but the surface is screen-anchored and stationary
/// rather than positioned relative to the page, because controls whose
/// position is derived from the page move on every zoom and scroll.
pub struct PrintDialog {
    /// Why the print system could not be reached at all, if it could not.
    ///
    /// Captured **once**, when the dialog opens. `Some` means there is no
    /// printer list to show and no printer to choose, so the whole body
    /// collapses to one sentence — a different sentence from "you have no
    /// printers", which is a claim about hardware this build cannot make.
    unavailable: Option<Unavailable>,
    /// Printers as the spooler reported them when the dialog opened.
    ///
    /// Read ONCE rather than per frame. Enumerating printers touches the
    /// spooler, and doing it sixty times a second while a dialog sits open
    /// would be rude to a service other applications share.
    printers: Vec<Printer>,
    /// Index into [`Self::printers`].
    selected: usize,
    /// What the selected device says it can do.
    features: DeviceFeatures,
    /// Every sheet size the selected device offers.
    ///
    /// Read on a change of selection, alongside [`Self::features`], and empty
    /// when the driver would not enumerate any — which is a legal answer and
    /// not a failure. An empty list means the paper control renders
    /// [`crate::text::print::paper_not_listed`] instead of a combo, rather
    /// than an empty combo: R9.
    forms: Vec<PaperForm>,
    /// The driver's own settings, once the operator has been through
    /// **Properties…** and accepted.
    ///
    /// # ★ Why `None` is not "the defaults" but "send no `DEVMODE` at all"
    ///
    /// They are genuinely different jobs. With `None`, `pdfcer-print` sends
    /// nothing and the device's own configuration applies in full — the
    /// cheapest and most conservative case, and the one this build did
    /// exclusively for its whole life. With `Some`, a real `DEVMODE` goes out
    /// carrying media type, quality, finishing and the driver's private tail,
    /// amended with the members this dialog names.
    ///
    /// # Why it is cleared on a change of printer
    ///
    /// Because a `DEVMODE`'s private tail is **one driver's** private format.
    /// Handing an EPSON's configuration to the XPS writer is not a degraded
    /// result, it is an undefined one, and the engine refuses it by name. See
    /// [`Self::refresh_device`], which drops this and the paper choice
    /// together for two different reasons.
    config: Option<DriverConfig>,
    /// Why the driver's properties dialog could not be opened, if it could
    /// not.
    ///
    /// Kept apart from [`Self::outcome`] deliberately: that field is *what
    /// happened to a print job*, and a properties failure is not one. Folding
    /// them together would let "your settings dialog would not open" appear
    /// where the operator reads "nothing was sent to the printer", which
    /// states something worse than what happened.
    ///
    /// **Cancel does not set this.** `Ok(None)` is the operator declining.
    properties_error: Option<String>,
    /// Set by the Properties… button, consumed after the window closure
    /// returns.
    ///
    /// # Why this is deferred, and it is a stronger reason than the commit's
    ///
    /// The driver's properties dialog is a **nested modal message loop** that
    /// Windows runs on this thread. Calling it from inside `Window::show`'s
    /// closure would run a foreign event loop while egui is part-way through
    /// laying this dialog out, with our own `Ui` borrowed for however long the
    /// operator spends in it. Deferring by one statement — the same shape as
    /// [`Self::commit_requested`] — means the frame has finished and nothing
    /// of egui's is borrowed when the loop starts.
    properties_requested: bool,
    /// Which selection the per-device cache — [`Self::features`],
    /// [`Self::forms`], [`Self::config`] — was filled for.
    ///
    /// ★ **This field is a fix, not salvage.** The old shell read the device
    /// features once when the dialog opened and never again — while letting
    /// the operator change printer from the combo box. On a machine with one
    /// duplex device and one simplex device, switching to the simplex one
    /// left the duplex radios on screen, and choosing two-sided produced a
    /// job that came out single-sided with nothing to say why. That is
    /// exactly the failure R83 forbids, arriving through a stale cache rather
    /// than through a missing check. Re-read on *change of selection* rather
    /// than per frame, so the discipline about not pestering the spooler is
    /// kept.
    features_for: Option<usize>,
    /// Which pages.
    range: PrintRange,
    /// The typed range, live even when [`PrintRange::Custom`] is not selected,
    /// so switching away and back does not lose it.
    range_text: String,
    /// How each page is sized onto the sheet.
    scale: ScaleMode,
    /// The custom percentage, kept across mode switches for the same reason
    /// as [`Self::range_text`].
    custom_percent: u32,
    /// Which classes of annotation print.
    ///
    /// Defaulted to `Document` for PRINTING, which differs from the
    /// renderer's own `DocumentAndMarkups` default. Deliberate on both sides:
    /// the canvas should show markup, and a print should not carry review
    /// comments unless asked. Acrobat Pro defaults the other way and Reader
    /// defaults to `Document`; pdfcer takes Reader's here, because a comment
    /// reaching paper unasked is the costlier mistake.
    scope: pdfcer_render::AnnotationScope,
    /// Rendering resolution ceiling, in DPI. A memory bound, editable because
    /// the disclosure is worth more as a control than as a warning.
    max_dpi: u32,
    /// Driver-level settings: orientation, duplex, tray choice.
    device: DeviceSettings,
    /// Odd/even filtering.
    subset: PageSubset,
    /// Print back to front.
    reverse: bool,
    /// Copy count.
    copies: u16,
    /// Copy ordering, as the checkbox holds it.
    uncollated: bool,
    /// Which sheet of the job the preview shows.
    preview_page: usize,
    /// Which group of settings is on screen.
    ///
    /// Lives on the dialog rather than on the application, so closing the
    /// dialog forgets it. That is the right lifetime: the tab an operator
    /// last used is a fact about the job they were configuring, and reopening
    /// the dialog for a different job should start where the dialog's own
    /// default says, not where an unrelated job ended.
    active_tab: PrintTab,
    /// Preview magnification, as a multiple of the fit scale. `1.0` is fit.
    ///
    /// Expressed relative to fit rather than as an absolute pt-per-pt scale so
    /// that resizing the window keeps whatever the operator chose: at `1.0` a
    /// taller window shows a bigger sheet, and at `3.0` it shows the same
    /// detail, bigger. An absolute scale would make the preview drift out of
    /// the canvas every time the window changed.
    preview_zoom: f32,
    /// How wide the preview column is, in egui points — the splitter's state.
    ///
    /// Operator request, 2026-09-03: *"the preview should be adjustable
    /// size."* Lives on the dialog rather than in `egui::Memory` because it is
    /// part of what the operator has configured about this print, alongside the
    /// zoom and pan beside it, and those three are reset together when the
    /// dialog is constructed.
    ///
    /// ★ Always read back through the clamp in [`Self::body`], never used raw:
    /// the bound depends on the window width, which changes under it.
    preview_width: f32,
    /// **Whether the preview is in its own OS window** — operator request O112
    /// ask 2, 2026-09-05.
    ///
    /// `false` is in the column beside the options, which is where every
    /// dialog opens; `true` is the second [`crate::dialogs::host::Host`] that
    /// [`popout`] draws, and the print dialog's own preview column then
    /// **renders nothing at all** and the options take its room (R9 — see
    /// `layout::Columns::split`).
    ///
    /// ★ It lives here rather than in `egui::Memory` for the same reason the
    /// width beside it does: it is part of what the operator has arranged about
    /// *this* print, and it goes away with the dialog. A remembered pop-out
    /// state would open a second window on a later print the operator had not
    /// asked for one on, which is the kind of surprise a dialog is not allowed.
    ///
    /// ★★ There is exactly ONE writer of `false`: [`PrintDialog::popped_preview`]
    /// on `Frame::closed`. Closing the window IS putting the preview back, so a
    /// second control that set this would be a second route to something the
    /// title bar already does — and the two would eventually disagree.
    preview_popped: bool,
    /// How far the sheet is displaced from centred, in egui points.
    ///
    /// Applied AFTER centring, so `Vec2::ZERO` always means "centred at the
    /// current zoom" and the Fit button is a two-field reset rather than a
    /// recomputation.
    preview_pan: egui::Vec2,
    /// The rendered page bitmap behind the preview, what it is a picture of,
    /// and **where that picture carries ink**.
    ///
    /// `None` until the first successful render, and set back to `None` when
    /// a render fails — in which case the preview falls back to a flat fill,
    /// which still shows the GEOMETRY correctly. A preview that shows the
    /// right rectangle and no content is degraded; one that shows a stale
    /// page is wrong.
    ///
    /// # ★ The ink mask rides in the SAME tuple, under the SAME key
    ///
    /// Added 2026-09-03 for operator request O113, and the placement is the
    /// point rather than an implementation detail. [`ink::InkMask`] describes
    /// **these exact pixels** — it is a downsample of this texture's source
    /// pixmap and of nothing else. A mask held in a separate field, or under a
    /// key of its own, could outlive the raster it describes, and a mask that
    /// has outlived its raster is strictly worse than no mask: it would answer
    /// *"is the overhang blank?"* about a page the operator is no longer
    /// looking at, and answer it confidently.
    ///
    /// One tuple, one [`preview::PreviewKey`], one lifetime. When the key
    /// misses, all three are replaced together; when the render fails, all
    /// three go away together.
    preview_texture: Option<(preview::PreviewKey, egui::TextureHandle, ink::InkMask)>,
    /// **What the preview found in the overhang of each sheet it has been
    /// shown** — operator request O113, 2026-09-04.
    ///
    /// The one piece of state here that accumulates as the operator works, and
    /// the reason it may: each entry was *measured*, from the same computation
    /// the hatch was drawn from, and is dropped the moment anything it depended
    /// on moves. It only ever makes the commit button's number **smaller**, and
    /// only for sheets a raster was examined for; a sheet nobody has looked at
    /// stays counted, because a claim about it would be invented. See
    /// [`verdicts`] for the key, and for why an over-strong one is the safe
    /// direction.
    verdicts: verdicts::Verdicts,
    /// The last spool attempt's outcome, once there is one.
    outcome: Option<Result<SpoolReport, String>>,
    /// Set by the commit button, consumed after the window closure returns.
    ///
    /// # Why the commit is deferred by exactly one statement
    ///
    /// Not for the borrow checker — for the frame. Committing rasterises
    /// *every page of the job* at print resolution, which on a sheet set is
    /// seconds of work; doing that inside `Window::show`'s closure runs it
    /// while egui is part-way through laying the dialog out. Deferring it to
    /// immediately after the closure returns keeps the layout pass honest and
    /// keeps the whole spool path outside any `Ui` borrow.
    ///
    /// This is the same reason the old shell routed the click through an
    /// `Action` — and an `Action` is not needed here, because a print changes
    /// no document state and so has nothing to contribute to the undo log the
    /// action funnel exists to keep coherent. See `crate::app::actions`'
    /// header for what that funnel is for.
    commit_requested: bool,
    /// Set by the footer's Close button, consumed by [`Self::show`].
    ///
    /// A flag rather than a direct close for the same reason as
    /// [`Self::commit_requested`]: the footer runs inside the window's own
    /// closure, and the window is what owns whether it is still open. Routing
    /// both closes — the button's and the title bar's — through one `open`
    /// flag means there is one close path rather than two that can disagree.
    close_requested: bool,
}

impl PrintDialog {
    /// Build the dialog for the document `doc`.
    ///
    /// # Two things happen here and nowhere else
    ///
    /// 1. **The spooler is enumerated, once, on a deliberate click.**
    ///    Enumerating printers can block briefly on a network spooler, so it
    ///    must not happen inside the frame loop.
    /// 2. **The preview opens on the page the operator is looking at.** Not
    ///    page 1: the commonest print is "this sheet", and opening the preview
    ///    somewhere else makes the operator step back to where they already
    ///    were.
    ///
    /// The guard against re-opening over a half-configured job is
    /// [`crate::dialogs::DialogsState::open_print`]'s, because it is the one
    /// place that can see whether a dialog already exists.
    pub(super) fn open(doc: &OpenDoc) -> Self {
        let (unavailable, printers) = match spooler::list_printers() {
            Ok(printers) => (None, printers),
            Err(error) => (Some(error), Vec::new()),
        };
        let selected = printers.iter().position(|p| p.is_default).unwrap_or(0);
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "print-open printers={} selected={selected} unavailable={unavailable:?} page={}",
                printers.len(),
                doc.view.page_index,
            )
        });
        Self {
            unavailable,
            printers,
            selected,
            features: DeviceFeatures::default(),
            forms: Vec::new(),
            config: None,
            properties_error: None,
            properties_requested: false,
            features_for: None,
            range: PrintRange::All,
            range_text: String::new(),
            scale: ScaleMode::Fit,
            custom_percent: 100,
            scope: pdfcer_render::AnnotationScope::Document,
            max_dpi: 300,
            device: DeviceSettings::default(),
            subset: PageSubset::All,
            reverse: false,
            copies: 1,
            uncollated: false,
            preview_page: 0,
            active_tab: PrintTab::default(),
            // Fit, centred. Both are reset here rather than carried over from
            // a previous dialog for the same reason `active_tab` is: a zoom
            // chosen while inspecting sheet 4 of last week's job says nothing
            // about this one.
            preview_zoom: 1.0,
            preview_width: layout::PREVIEW_DEFAULT_WIDTH_PTS,
            // In the column. A dialog opens with its preview where every
            // print dialog on this machine puts it; popping out is the
            // operator's act, never the program's.
            preview_popped: false,
            preview_pan: egui::Vec2::ZERO,
            preview_texture: None,
            // Empty, and emptied again by every change of context — see
            // `verdicts::Verdicts::remember`. A dialog opens knowing nothing
            // about any sheet, which is why a job printed without ever
            // stepping the preview gets the unchanged geometric count.
            verdicts: verdicts::Verdicts::default(),
            outcome: None,
            commit_requested: false,
            close_requested: false,
        }
    }

    /// Draw one frame of the dialog. Returns `false` when it should close.
    ///
    /// Everything the job depends on is recomputed here, every frame, from
    /// the operator's current answers — there is no cached plan that could
    /// describe a different job from the one the preview is showing. That is
    /// affordable because planning is arithmetic over a page-size list; the
    /// two things that are *not* affordable per frame (enumerating printers,
    /// asking a driver about duplex) are the two that are not done here.
    /// This dialog's window: what it is called, how big it opens, and the
    /// floor it may not be dragged below.
    ///
    /// ★ Built fresh each frame and owning nothing — the position the operator
    /// drags it to lives in `egui::Memory`, keyed on the id string. See
    /// [`crate::dialogs::host`]'s header for why that is what let the other
    /// thirteen dialogs be converted in one line each.
    ///
    /// ★ The size argument is unchanged and carried verbatim from the
    /// `egui::Window` this replaced. The floor is not a preference:
    /// `resizable` with no minimum lets the operator drag the window down to a
    /// title bar and a scrollbar, which is a state with no way back except
    /// closing it — and closing this dialog discards the job they were
    /// configuring. 520 x 380 is the smallest size at which one column and
    /// both scrollbars are still usable.
    fn host() -> crate::dialogs::host::Host {
        crate::dialogs::host::Host::new(
            "print", // ui-text-exempt: a viewport key, never displayed.
            t::dialog_title(),
            egui::vec2(800.0, 620.0),
            egui::vec2(520.0, 380.0),
        )
    }

    pub(super) fn show(
        &mut self,
        ctx: &egui::Context,
        doc: &OpenDoc,
        window: Option<isize>,
    ) -> bool {
        self.refresh_device();

        // ★ Page sizes come from the ROTATED device extent, not from the raw
        // `/MediaBox`.
        //
        // A divergence from the salvaged source, and a fix rather than a
        // preference. The placement is applied to a *rendered pixmap*, and
        // `pdfcer-render` rasterises a page at its rotated extent — so a page
        // carrying `/Rotate 90` renders landscape while its MediaBox reads
        // portrait. Planning from the MediaBox would place a landscape bitmap
        // into a portrait rectangle: the scale would be wrong on both axes and
        // the clip report would be wrong with it. `viewer::page_extent_pts` is
        // the same function the canvas measures with, so the preview, the
        // canvas and the paper agree by construction rather than by three
        // hand-written box subtractions.
        let page_sizes: Vec<(f64, f64)> = doc
            .pages
            .iter()
            .map(|page| {
                let (w, h) = crate::viewer::page_extent_pts(page);
                (f64::from(w), f64::from(h))
            })
            .collect();

        let spec = self.job_spec(&page_sizes, doc.view.page_index);
        let printer_name = self.printers.get(self.selected).map(|p| p.name.clone());
        let job = printer_name
            .as_deref()
            .map(|name| spooler::plan(name, self.device, self.config.as_ref(), &page_sizes, &spec))
            .and_then(Result::ok);

        // Keep the stepper inside the job. A range narrowed while the dialog
        // is open can leave `preview_page` past the end, and a preview that
        // silently shows a sheet the job no longer contains is the same class
        // of wrong as indexing page sizes by the plan position.
        if let Some(job) = &job {
            self.preview_page = self.preview_page.min(job.plans.len().saturating_sub(1));
        }

        // ★ ONE context per frame, built here and passed down — operator
        // request O113. It is the job-wide half of every cache key in this
        // dialog: the preview texture's key is *derived from it*
        // (`verdicts::Context::preview_key` is the only place a `PreviewKey` is
        // built), every remembered overhang verdict is void the moment it
        // changes, and the frame's one `Settings` clone is paid here rather
        // than at each of the three sites that need it. `None` exactly when
        // there is no job — the printable rectangle comes from the planned
        // geometry, and with no device there is no clip to report.
        let context = job
            .as_ref()
            .map(|job| verdicts::Context::new(self.scope, &doc.settings, job.device.printable_pt));

        // ★★★ THE POPPED-OUT PREVIEW, DRAWN BEFORE THIS DIALOG'S OWN WINDOW —
        // operator request O112 ask 2, 2026-09-05.
        //
        // A no-op unless the operator has pressed Pop out. The order matters
        // and [`PrintDialog::popped_preview`] carries the argument: the commit
        // button's clip count is corrected by what the preview has examined, so
        // the preview must paint before the footer reads the claim — an
        // invariant the body used to satisfy by containing it.
        self.popped_preview(ctx, doc, job.as_ref(), &page_sizes, context.as_ref());

        // ★★ A REAL OS WINDOW, as of 2026-08-20. The operator's report:
        //
        // > *"Print dialogue box doesn't pop up in its own movable window. It
        // > is locked within the boundaries of the program's window."*
        //
        // The title, the size and the floor are unchanged; what changed is the
        // host. Everything the closure below does is what it did inside
        // `egui::Window`, because a `Ui` is a `Ui` — which is the point of
        // routing this through `dialogs::host` rather than open-coding
        // `show_viewport_immediate` here: the next dialog is one line, not a
        // second implementation to keep level with this one.
        //
        // ★ The screen-anchoring note that stood here is retired rather than
        // moved. It said the window is anchored to the SCREEN and never to the
        // document, against an operator objection to controls that move on
        // every zoom and scroll. An OS window is anchored to the DESKTOP, which
        // satisfies that objection more completely than the anchor did — and
        // the anchor's other half, `CENTER_CENTER` on every frame, was G6's
        // defect: it dragged the window back to the middle the instant the
        // operator moved it.
        // Out for the duration of the draw — see the field's own docs for why,
        // and for why rebuilding it is a safe answer rather than a panic.
        let (frame, ()) = Self::host().show(ctx, |ui| {
            if let Some(unavailable) = &self.unavailable {
                // No printer list, no printer to choose, nothing to
                // preview. Two sentences and a Close button: the general
                // one an operator acts on, then the engine's own account
                // of what failed, which is the half they can quote at
                // whoever administers the machine.
                //
                // The specific line is `.small().weak()` for the same
                // reason the settings window's store-location line is:
                // it is a fact the reader may need and is not the thing
                // they are being told. Making it as loud as the sentence
                // above would put a Win32 error code at the same weight
                // as "nothing has been sent".
                ui.label(t::spooler_unavailable());
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(t::spooler_detail(&unavailable.to_string()))
                        .small()
                        .weak(),
                );
                return;
            }
            if self.printers.is_empty() {
                ui.label(t::no_printers());
                return;
            }
            self.body(ui, doc, job.as_ref(), &page_sizes, context.as_ref());
            ui.separator();
            self.footer(ui, job.as_ref(), &page_sizes, context.as_ref());
        });

        // ★ The claim is read AFTER the closure, so it includes whatever the
        // preview learned while drawing this very frame. Reading it before
        // would report the state of the previous frame on the surface that
        // exists to describe this one — the footer already avoids that by
        // running after the preview column inside the closure, and this line
        // keeps the trace level with the footer.
        let claim = match (&job, &context) {
            (Some(job), Some(context)) => self.verdicts.claim(context, job, &page_sizes),
            _ => verdicts::ClipClaim::None,
        };
        self.trace_plan(printer_name.as_deref(), job.as_ref(), claim);

        // ★ The driver's properties dialog, opened here for a stronger version
        // of the reason the commit is deferred: it is a nested modal message
        // loop, and it must not run with an egui `Ui` borrowed. See
        // [`Self::properties_requested`].
        //
        // Before the commit below, deliberately: a frame in which the operator
        // somehow triggered both should configure the device and then print
        // with what they configured, not print and then configure.
        if std::mem::take(&mut self.properties_requested) {
            self.open_properties(window);
        }

        // ★ The commit, performed here: after the window's closure has
        // returned and before the next frame begins. See
        // [`Self::commit_requested`] for why it is not done at the click site.
        if std::mem::take(&mut self.commit_requested)
            && let (Some(printer), Some(job)) = (printer_name, job)
        {
            let outcome = self.commit(&printer, doc, &job, &page_sizes);
            // ★★★ A SUCCESSFUL PRINT CLOSES THE DIALOG — 2026-09-03, and until
            // this day it did not.
            //
            // The operator: *"it doesn't close after I hit the print button
            // [...] it looks greyed out as though it doesn't do anything even
            // when I hit print - but it is working, so after many clicks I
            // checked the printer and of course there was a dozen jobs there."*
            //
            // Two defects made that, and this is the half that lives here. The
            // other is that the button was painted with a translucent canvas
            // tint and read as disabled (`dialogs::host::Host::buttons`). Fixed
            // separately, because either one alone still produces a duplicate
            // job: a button that looks dead in a window that stays open is
            // pressed again, and a working button in a window that stays open
            // is pressed again by anyone who expects the window to go.
            //
            // ★★ **The conventional interaction is the specification here.**
            // Every print dialog on this machine — Word, Acrobat, Chrome,
            // Notepad — dismisses itself the instant the job is handed to the
            // spooler. A print dialog is a transaction with an end, and the end
            // is the spooler accepting it. Leaving it open invents a model in
            // which the dialog is a printing *workspace*, and an operator who
            // has never met that model reads the window still being there as
            // the press not having landed. This shell does not get to invent an
            // interaction the whole product class agrees on.
            //
            // ★★★ A FAILURE DOES **NOT** CLOSE, and that asymmetry is the point
            // rather than a hedge. On failure the operator's next act is to
            // choose a different printer or a different range — which is what
            // this window is for — and the driver's own words are the only
            // thing that tells them which. Closing would destroy the reason and
            // the settings together, leaving *"nothing printed"* and no route
            // back. Word and Acrobat behave the same way.
            //
            // ★ THE RECEIPT IS NOT LOST, it moves. `Ok` used to be reported in
            // the footer, which is a surface that only exists while the window
            // does; on the disclosure row it outlives the dialog and sits with
            // every other consequence of an operation. Both sentences travel —
            // the page count and, when the driver held settings pdfcer does not
            // model, the `Synthesised` disclosure — through `record_notes`
            // rather than two `record_note` calls, because the slot holds one
            // disclosure and a second call REPLACES the first. That is
            // documented at `record_notes` and is exactly the trap it names.
            match Self::commit_notes(outcome.as_ref()) {
                Some(notes) => {
                    crate::app::actions::record_notes(doc.edit_epoch, notes);
                    // The outcome is still stored, for the trace and for the
                    // frame that is already in flight. Nothing will draw it.
                    self.outcome = Some(outcome);
                    self.close_requested = true;
                }
                None => self.outcome = Some(outcome),
            }
        }
        // ★ Three routes out, one outcome — G4. The OS close button and
        // Escape both arrive as `frame.closed`; the footer's own Close button
        // sets `close_requested`. Treating any of them differently would give
        // one route a different meaning from the other two, which is exactly
        // the surprise the convention exists to prevent.
        //
        // ★ A successful commit joins them rather than getting a fourth route,
        // for the same reason: "the job is away, the window is finished" is the
        // same outcome as Close, and giving it its own path is how two exits
        // come to differ.
        !frame.closed && !std::mem::take(&mut self.close_requested)
    }

    /// Re-read everything that belongs to the selected device.
    ///
    /// See [`Self::features_for`] for the defect this closes: the old shell
    /// read capabilities only for the *initially* selected device and never
    /// again, while letting the operator change printer, so a duplex control
    /// could survive onto a simplex device and produce a job that came out
    /// single-sided with nothing to say why.
    ///
    /// Called once per **change of selection**, never per frame: three of the
    /// four things it does open a device context, and doing that sixty times
    /// a second while a dialog sits open would be rude to a service other
    /// applications share.
    ///
    /// # ★ Two things are DROPPED here, for two different reasons
    ///
    /// **The configuration**, because a `DEVMODE`'s private tail is one
    /// driver's private format and handing it to another device is undefined
    /// rather than degraded. The engine refuses it by name; dropping it here
    /// means the refusal is never reached.
    ///
    /// **The paper choice**, and this one is subtler and worth stating in
    /// full. [`PaperChoice::Form`] holds a `dmPaperSize` integer, and those
    /// are only standard up to a point: the low ids are Win32 constants
    /// (`DMPAPER_LETTER` is 1, `DMPAPER_A3` is 8), but everything a vendor
    /// defines lives above `DMPAPER_USER` and means whatever that one driver
    /// says. Carrying `Form(257)` from an EPSON to a plotter would silently
    /// request a different sheet under the same number — no error, no
    /// mismatch, just the wrong paper. It resets to
    /// [`PaperChoice::DeviceDefault`], which is the only value that means the
    /// same thing on every device.
    ///
    /// A failed read of either falls back to the safe direction: no features
    /// (so no duplex control) and no forms (so no paper list).
    fn refresh_device(&mut self) {
        if self.features_for == Some(self.selected) {
            return;
        }
        let name = self.printers.get(self.selected).map(|p| p.name.clone());
        let features = name
            .as_deref()
            .and_then(|name| spooler::device_features(name).ok())
            .unwrap_or_default();
        let forms = name
            .as_deref()
            .and_then(|name| spooler::printer_forms(name).ok())
            .unwrap_or_default();

        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "print-features selected={} duplex={} max_copies={} form_source={:?} forms={}",
                self.selected,
                features.supports_duplex,
                features.max_copies,
                features.form_source,
                forms.len(),
            )
        });

        self.features = features;
        self.forms = forms;
        self.config = None;
        self.device.paper = PaperChoice::DeviceDefault;
        self.properties_error = None;
        self.features_for = Some(self.selected);
    }

    /// Open the driver's own properties dialog and keep what it produces.
    ///
    /// Runs **after** the window's closure has returned — see
    /// [`Self::properties_requested`] for why a nested modal message loop
    /// cannot be started from inside an egui layout pass.
    ///
    /// # What happens to the three outcomes
    ///
    /// | outcome | effect |
    /// |---|---|
    /// | accepted | the configuration is stored, and the paper combo adopts whatever sheet it names |
    /// | cancelled | **nothing at all** — no message, no state change. The operator declined |
    /// | refused | [`Self::properties_error`] is set and shown; whatever configuration was already held survives |
    ///
    /// # ★ Why the paper combo follows the driver's dialog
    ///
    /// Because otherwise two surfaces describe the same job differently. An
    /// operator who picks A3 in the driver's dialog and returns to a combo
    /// still reading *"From the printer's own settings"* has been told
    /// something false by a control they can see, about a setting they just
    /// changed. Adopting the id makes the combo a report of the truth rather
    /// than a competing claim — and because the engine amends rather than
    /// replaces, asserting the same value changes nothing about the job.
    ///
    /// A configuration naming a **custom** sheet has no id to adopt, so the
    /// combo stays on `DeviceDefault` — which is correct: `DeviceDefault`
    /// asserts no paper, so the configuration's own custom sheet stands. The
    /// disclosure line reports it rather than the combo.
    fn open_properties(&mut self, parent: Option<isize>) {
        let Some(printer) = self.printers.get(self.selected).map(|p| p.name.clone()) else {
            return;
        };
        match spooler::edit_printer_configuration(&printer, parent, self.config.as_ref()) {
            Ok(Some(config)) => {
                let summary = config.summary();
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "print-properties-accepted printer={printer} form={:?} custom_pt={:?} \
                         driver_extra={}",
                        summary.paper_form_id, summary.custom_paper_pt, summary.driver_extra,
                    )
                });
                if let Some(id) = summary.paper_form_id {
                    self.device.paper = PaperChoice::Form(id);
                }
                self.config = Some(config);
                self.properties_error = None;
            }
            Ok(None) => {
                // Cancel. Deliberately silent — see the table above.
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!("print-properties-cancelled printer={printer}")
                });
            }
            Err(error) => {
                let detail = error.to_string();
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "print-properties-refused printer={printer} reason={detail}"
                    )
                });
                self.properties_error = Some(detail);
            }
        }
    }

    /// Turn the operator's answers into a [`JobSpec`].
    ///
    /// The custom scale is materialised here rather than stored live, so the
    /// percentage spinner can be edited while some other sizing mode is
    /// selected without the mode changing under the operator's hand.
    fn job_spec(&self, page_sizes: &[(f64, f64)], current_page: usize) -> JobSpec {
        let mode = match self.scale {
            ScaleMode::Custom(_) => ScaleMode::Custom(f64::from(self.custom_percent) / 100.0),
            other => other,
        };
        JobSpec {
            pages: self
                .range
                .indices(&self.range_text, page_sizes.len(), current_page),
            mode,
            max_dpi: self.max_dpi,
            subset: self.subset,
            reverse: self.reverse,
            copies: self.copies,
            collate: if self.uncollated {
                Collate::Uncollated
            } else {
                Collate::Collated
            },
        }
    }

    /// **What a finished commit owes the operator, and whether the window is
    /// done** — as a pure function, so it can be tested without a printer.
    ///
    /// Returns `Some(notes)` when the job went to the spooler: the sentences to
    /// put on the application's disclosure row, in reading order. The caller
    /// records them and closes the dialog. Returns `None` on failure, which
    /// means *"say nothing here and leave the window open"* — the footer draws
    /// the driver's own words and the operator picks another printer.
    ///
    /// # ★★★ Why this is extracted rather than left inline
    ///
    /// Because the behaviour it decides is the operator's 2026-09-03 report —
    /// *"it doesn't close after I hit the print button [...] there was a dozen
    /// jobs there"* — and the only way to drive the inline version is to
    /// actually print. Spooling a real job to his printer to prove a window
    /// closes is not a test, it is the defect.
    ///
    /// So the decision is separated from the act. `ui-verify` cannot reach it
    /// (no headless route ends in a real spool), and this project's rule is
    /// that a unit test is the floor rather than the ceiling — so what is
    /// asserted here is deliberately the part that is **pure logic**: which
    /// outcome closes, and which sentences travel. The act of printing is
    /// `Self::commit`'s, and is covered by `print_dialog_reaches_the_spooler`.
    ///
    /// ★ Stated plainly because it is a real gap: *"the window closes after a
    /// successful print"* is asserted as a decision, not as an observed
    /// window disappearing. Closing that gap needs a driven check that prints
    /// to a file device — `Microsoft Print to PDF` is on this machine — and it
    /// is worth building; it is not built.
    fn commit_notes(outcome: Result<&SpoolReport, &String>) -> Option<Vec<String>> {
        let report = outcome.ok()?;
        let mut notes = vec![t::sent(report.pages)];
        // ★ The only one of the four `SettingsSource` values that is disclosed,
        // and the operator could not learn it any other way: the job printed,
        // and everything the driver held that pdfcer does not model was
        // silently absent from it. See `SettingsSource::Synthesised`.
        if report.settings_source == SettingsSource::Synthesised {
            notes.push(t::settings_synthesised().to_owned());
        }
        // ★ Both sentences in ONE call. `record_notes`' own doc comment records
        // why: the slot holds a single disclosure, so a second `record_note`
        // REPLACES the first rather than joining it, and which one survived
        // would be decided by statement order.
        Some(notes)
    }

    /// The options column: the printer, then one of three tabs.
    ///
    /// # ★ The printer selector is OUTSIDE the tabs, always visible
    ///
    /// It is not a setting like the others — it is the thing that decides
    /// which of the others exist. [`Self::features`] is read from the selected
    /// device and gates the duplex radios (R83), so a tab that could hide the
    /// printer name would let the operator change device, watch controls
    /// appear and disappear, and have no way to see what they had changed it
    /// to without going looking.
    ///
    /// # The tab strip reuses the ribbon's widget, deliberately
    ///
    /// `egui::Button::selectable` plus a bold weight on the active one is what
    /// the ribbon already draws for its own tabs. Inventing a different tab
    /// affordance for the second tabbed surface in the application would teach
    /// the operator that "tab" looks like two different things. The bold
    /// weight is not decoration: R84 forbids state carried by colour alone.
    fn options_column(&mut self, ui: &mut Ui, job: Option<&Job>, page_count: usize) {
        ui.horizontal(|ui| {
            ui.label(t::printer_label());
            egui::ComboBox::from_id_salt("print-printer")
                .selected_text(
                    self.printers
                        .get(self.selected)
                        .map_or_else(String::new, |p| p.name.clone()),
                )
                .show_ui(ui, |ui| {
                    for (index, printer) in self.printers.iter().enumerate() {
                        ui.selectable_value(&mut self.selected, index, &printer.name);
                    }
                });
            // ★ BESIDE the printer combo, which is where the operator asked
            // for it: *"pretty much every program I have ever seen lets you
            // press a properties button beside the selected printer in the
            // drop-down menu to open the printer options."*
            //
            // Outside the tabs for the same reason the selector is: it acts
            // on the DEVICE rather than on the job, and every tab's settings
            // are settings of the job. Putting it inside one would imply it
            // belonged to that tab's group.
            //
            // Not gated on anything. Every Windows printer has a properties
            // dialog — it is the driver's, not ours — and a device that
            // refused to open it says so through `properties_error` below,
            // which is a fact about this attempt rather than a capability
            // that could have been read in advance.
            let properties = ui.button(t::properties());
            crate::diag::ui_rect(REGION_PROPERTIES, properties.rect);
            if properties.on_hover_text(t::properties_tooltip()).clicked() {
                self.properties_requested = true;
            }
        });

        // What the properties dialog left behind, if anything. Both lines are
        // `.small().weak()`: they are facts the operator may need and are not
        // the thing they are being told, which is the same weighting the
        // spooler-detail line uses.
        if let Some(detail) = &self.properties_error {
            ui.label(
                egui::RichText::new(t::properties_failed(detail))
                    .small()
                    .color(ui.visuals().error_fg_color),
            );
        } else if self.config.is_some() {
            ui.label(egui::RichText::new(t::properties_held()).small().weak());
        }
        ui.add_space(6.0);

        ui.horizontal_wrapped(|ui| {
            for tab in PrintTab::ALL {
                let selected = tab == self.active_tab;
                // ★ The selected tab paints its own plate and label — the
                // same fix as the ribbon's tab strip, and the same defect.
                // See `DEFECTS.md` D11.
                //
                // `Button::selectable(true, …)` fills from
                // `visuals.selection.bg_fill`, which this theme points at the
                // translucent **canvas** object-selection tint (alpha
                // 70/255); and `.strong()` resolves to `on_accent`, which is
                // near-white. Together that is near-white text on a 27 %
                // wash. The palette's `accent` / `on_accent` pair is what
                // this state is for.
                let theme = egui_shell::theme::Theme::of(ui.ctx());
                let mut text = egui::RichText::new(tab.label());
                if selected {
                    text = text.color(theme.palette.on_accent);
                }
                let mut button = egui::Button::selectable(selected, text);
                if selected {
                    button = button.fill(theme.palette.accent);
                }
                if ui.add(button).on_hover_text(tab.tooltip()).clicked() {
                    self.active_tab = tab;
                }
            }
        });
        ui.separator();

        match self.active_tab {
            PrintTab::PagesLayout => tabs::pages_layout(
                ui,
                self,
                page_count,
                // The TURNED sheet, from the plan — so the sentence
                // names the rectangle the job was actually laid out
                // against rather than the device's un-rotated default.
                // `None` while there is no plan, which the sentence
                // handles rather than the caller guessing.
                job.map(|j| j.device.physical_pt),
            ),
            PrintTab::CopiesFinishing => tabs::copies_finishing(ui, self),
            PrintTab::CommentsResolution => {
                tabs::comments_resolution(ui, self, job.map(|j| j.resolution));
            }
        }
    }

    /// Render every planned sheet and hand them to the spooler.
    ///
    /// # ★ The one place in the GUI that starts a print job
    ///
    /// Reached only from the commit button, via [`Self::commit_requested`].
    /// Nothing here runs as a side effect of opening, previewing, saving or
    /// rendering — which is the shell's half of `pdfcer-print`'s own contract
    /// that *"`spool` is the only function that reaches `StartDoc`, and it is
    /// reached only from a control an operator deliberately clicked."*
    ///
    /// # Why the whole job is rasterised inline
    ///
    /// It blocks the UI thread for as long as the job takes. That is the
    /// honest behaviour for now and it is not an oversight: a print that
    /// proceeds in the background needs a cancel affordance, a progress
    /// surface and an answer to "what happens if the document is edited
    /// mid-job", and shipping the render off-thread without those three would
    /// replace a visible wait with an invisible race. The single-slot render
    /// worker next door is for *display*, where a cancelled render costs
    /// nothing; a cancelled print costs paper.
    fn commit(
        &self,
        printer: &str,
        doc: &OpenDoc,
        job: &Job,
        page_sizes: &[(f64, f64)],
    ) -> Result<SpoolReport, String> {
        // The SAME builder the preview calls. See `render_options` for the
        // choices it encodes and why a second copy of them here would defeat
        // the preview's purpose.
        let options = render_options(self.scope, &doc.settings);
        let view = doc.session.view();

        let mut bitmaps = Vec::with_capacity(job.plans.len());
        for plan in &job.plans {
            let (Some(page), Some(&size)) = (doc.pages.get(plan.index), page_sizes.get(plan.index))
            else {
                // A plan naming a page the document no longer has. Skipped
                // rather than refused, matching `plan_job`'s own posture: *"a
                // job that refuses wholesale because one index is stale is
                // worse than one that prints what it can and reports the
                // count."*
                continue;
            };
            let rendered = pdfcer_render::render_page_with_view(
                &view,
                page,
                plan.render_scale as f32,
                &options,
            )
            .map_err(|e| e.to_string())?;
            bitmaps.push(PageBitmap {
                width: rendered.pixmap.width(),
                height: rendered.pixmap.height(),
                // Premultiplied RGBA8, handed over unchanged — the engine's
                // stated contract. Any conversion here would be a second
                // colour convention.
                rgba: rendered.pixmap.data().to_vec(),
                placement: plan.placement,
                page_pt: size,
            });
        }

        // ★ The orientation page is the FIRST PLANNED page, taken from the
        // bitmaps rather than from the document. The sequence may be reversed
        // or range-filtered, which is exactly when `pages[0]` would be the
        // wrong page — and the driver picks its paper from whichever one it is
        // handed.
        let first_page_pt = bitmaps
            .first()
            .map_or(US_LETTER_PORTRAIT_PT, |bitmap| bitmap.page_pt);
        spooler::spool(
            printer,
            &bitmaps,
            self.device,
            self.config.as_ref(),
            first_page_pt,
        )
        .map_err(|error| error.to_string())
    }

    /// One trace line describing the job the dialog is currently showing.
    ///
    /// ★ `scale=` is on this line beside `orientation=` because they are the
    /// pair that exposes the orientation defect: a radio that changes
    /// `orientation=` and not `scale=` on a landscape page is that regression,
    /// restated. A harness can assert the relationship; a screenshot cannot.
    ///
    /// ★★ `clipped=` and `claim=` are on this line TOGETHER, and the pairing is
    /// the assertion — operator request O113. `clipped=` is the unchanged
    /// geometric count; `claim=` is what the button says, as `<state>:<count>`.
    /// A driven check asserts the *correction* between them, which no capture
    /// can supply: a button reading "Print" and a button reading "Print"
    /// because the cache silently never matched are the same photograph.
    fn trace_plan(&self, printer: Option<&str>, job: Option<&Job>, claim: verdicts::ClipClaim) {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "print-plan printer={printer:?} driver={:?} port={:?} sheets={:?} clipped={:?} \
                 claim={}:{} \
                 dpi={:?} capped={:?} uncapped_mb={:?} orientation={:?} duplex={:?} \
                 paper={:?} sheet={:?} config={} \
                 scale={:?} tab={:?}",
                self.printers.get(self.selected).map(|p| &p.driver),
                self.printers.get(self.selected).map(|p| &p.port),
                job.map(|j| j.plans.len()),
                job.map(Job::clipped),
                claim.trace_word(),
                claim.count(),
                job.map(|j| j.resolution.dpi),
                job.map(|j| j.resolution.capped),
                job.map(|j| j.resolution.uncapped_page_mb),
                self.device.orientation,
                self.device.duplex,
                // ★ `paper=` and `sheet=` are on this line TOGETHER, and the
                // pairing is the assertion. `paper=` is what was asked for;
                // `sheet=` is the physical sheet the geometry came back with.
                // A build that took the request and planned against the
                // device's default anyway would show `paper=Form(8)` beside an
                // unchanged `sheet=` — the 77 %-scale defect in a second
                // dimension, and invisible in any other evidence.
                self.device.paper,
                job.map(|j| j.device.physical_pt),
                self.config.is_some(),
                job.and_then(|j| j.plans.first()).map(|p| p.placement.scale),
                self.active_tab,
            )
        });
    }
}

/// The page size assumed for a job that plans no pages.
///
/// Mirrors `pdfcer_print::US_LETTER_PORTRAIT_PT`. Such a job spools nothing, so
/// the value never reaches paper; it exists so the commit path carries no
/// `Option` for a case that cannot print.
const US_LETTER_PORTRAIT_PT: (f64, f64) = (612.0, 792.0);

/// The render options a print job — and its preview — are drawn with.
///
/// # ★ ONE builder, called from both, and that is the point
///
/// Two independently-written builders eventually disagree about something, and
/// neither side can tell which one they are looking at. For a print preview
/// that failure is the whole feature — a preview exists to say what will come
/// out of the printer, so a preview built from its own options is a preview
/// that can be confidently wrong.
///
/// The choices it encodes, carried across with their reasoning:
///
/// - **`view_magnification` stays `None`** — the PRINT answer under §8.11.4.5,
///   which says a printing application *"shall not apply the changes based on
///   usage application dictionaries"*. Inheriting the canvas's options would
///   apply the zoom-driven optional-content states the operator happens to be
///   looking at.
/// - **The operator's layer overrides are NOT applied**, for the same clause:
///   they are a viewing choice, and §8.11.4.5 puts printing on the document's
///   own default configuration. `RenderOptions::layers` left at `None` is what
///   expresses that — and `None` is *not* an empty set, which would reveal
///   every layer the document turned off.
/// - **The annotation scope IS the operator's**, because it is a statement
///   about the job rather than about the view.
///
/// ## ★ The settings surface landed, and this paragraph is what it changed
///
/// This doc comment used to say:
///
/// > One choice the old shell encoded is missing here and its absence is not an
/// > omission: **the CMYK conversion intent**. `pdfcer-core`'s settings surface
/// > does not exist in this crate yet, so there is no operator choice to carry.
/// > When it lands, it belongs here *and* in [`preview::PreviewKey`] in the
/// > same commit — otherwise the preview keeps showing a page rendered under
/// > the previous intent, which is the exact staleness class that key exists to
/// > close.
///
/// It landed on 2026-08-17 and both halves were done together, as instructed.
/// The options now come from `crate::app::settings::SettingsExt`, which carries
/// **five** settings rather than the one that note anticipated — the CMYK
/// intent, the mask resampling filter, the minification filter, the CMYK JPEG
/// polarity, and what is drawn for an annotation with no stated appearance
/// state. That last one reaches paper as well as the screen, which is why its
/// radius line in the settings window is the only one that separately names
/// printing.
///
/// [`preview::PreviewKey`] gained the same five, for the reason that note gave.
fn render_options(
    scope: pdfcer_render::AnnotationScope,
    settings: &pdfcer_core::settings::Settings,
) -> pdfcer_render::RenderOptions {
    use crate::app::settings::SettingsExt;
    settings.render_options().with_annotation_scope(scope)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A spool report with the given page count and settings source.
    ///
    /// Built by hand rather than by printing, which is the whole reason
    /// [`PrintDialog::commit_notes`] was extracted: proving that the window
    /// closes must not require putting a job on the operator's printer.
    fn report(pages: usize, source: SettingsSource) -> SpoolReport {
        SpoolReport {
            pages,
            printed: true,
            dpi: (300, 300),
            clipped_pages: 0,
            job_id: Some(1),
            settings_source: source,
        }
    }

    /// **A successful print returns sentences, which is what closes the
    /// window** — the operator's 2026-09-03 report.
    ///
    /// > *"it doesn't close after I hit the print button [...] it looks greyed
    /// > out as though it doesn't do anything even when I hit print - but it is
    /// > working, so after many clicks I checked the printer and of course
    /// > there was a dozen jobs there."*
    ///
    /// `Some` is the signal to record and close; `None` is the signal to stay
    /// open. Asserting on the discriminant rather than on the wording, because
    /// the wording belongs to `crate::text::print` and a test that pinned it
    /// here would be a second copy of it.
    #[test]
    fn a_successful_print_asks_the_dialog_to_close() {
        let ok = report(3, SettingsSource::DriverSupplied);
        assert!(
            PrintDialog::commit_notes(Ok(&ok)).is_some(),
            "a job that reached the spooler must produce a receipt, which is what closes the \
             window. Leaving it open is how one press became a dozen queued jobs."
        );
    }

    /// **A FAILED print leaves the window open**, and the asymmetry is
    /// deliberate rather than an oversight.
    ///
    /// On failure the operator's next act is to choose a different printer or a
    /// different range — which is what this window is for — and the driver's
    /// own words in the footer are the only thing telling them which. Closing
    /// would destroy the reason and the settings together.
    #[test]
    fn a_failed_print_leaves_the_dialog_open() {
        let why = "the device is offline".to_owned();
        assert!(
            PrintDialog::commit_notes(Err(&why)).is_none(),
            "a failed job must NOT close the dialog: the footer's message is the only place the \
             reason appears, and the settings that produced it are still on screen."
        );
    }

    /// **The `Synthesised` disclosure travels WITH the receipt, in one call.**
    ///
    /// Two sentences, not two `record_note` calls. `record_notes`' doc comment
    /// records why that matters: the slot holds one disclosure, so a second
    /// call REPLACES the first and which one survived would be decided by
    /// statement order rather than by importance.
    ///
    /// The receipt is first because it is the sentence an operator reads if
    /// they read only one.
    #[test]
    fn a_synthesised_settings_source_adds_a_second_sentence_to_the_same_receipt() {
        let plain = PrintDialog::commit_notes(Ok(&report(2, SettingsSource::DriverSupplied)))
            .expect("a success returns notes");
        let synthesised = PrintDialog::commit_notes(Ok(&report(2, SettingsSource::Synthesised)))
            .expect("a success returns notes");

        assert_eq!(plain.len(), 1, "an ordinary print says one thing");
        assert_eq!(
            synthesised.len(),
            2,
            "a job printed from settings pdfcer synthesised owes the operator that fact, and it \
             is the one `SettingsSource` value they could not learn any other way"
        );
        assert_eq!(
            plain[0], synthesised[0],
            "the receipt must be the same sentence and must come FIRST in both cases"
        );
    }
}
