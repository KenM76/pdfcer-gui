//! # pdfcer-gui — native desktop shell (rebuild), library root
//!
//! **This crate is a library with a thin binary in front of it.** The
//! binary (`src/main.rs`) does one thing: read `argv`, call [`run`]. Every
//! module, every type and every test lives here.
//!
//! ## Why a library at all, when the product is an executable
//!
//! Three reasons, and the third is the one that bites daily.
//!
//! 1. **`tools/ui-verify` and any integration test can `use pdfcer_gui::…`.**
//!    Without a library target every assertion has to cross the process
//!    boundary, even when the question is really a unit-level one — "does
//!    this manifest validate?" does not need a window.
//! 2. **`cargo doc` documents something.** A binary crate's rustdoc is
//!    empty, and this project's standing rule is that the documentation is
//!    the logic. Docs that cannot be browsed are docs that rot.
//! 3. **`main.rs` stops being a contention point.** With the module tree in
//!    a binary, *every* new module must edit the same file — which is the
//!    one guaranteed merge conflict when work runs in parallel, and this
//!    project runs work in parallel by design. Moving the tree here does not
//!    remove the shared file, but it removes it from the path of the
//!    `argv`-and-viewport code that has nothing to do with it.
//!
//! Converted at the S2 → S3 boundary, deliberately: it changes visibility
//! across every module, so it wants a moment when nothing else is in
//! flight, and it wants to happen *before* the panel modules multiply.
//! `PROJECT_PLAN.md` §4.2b records the decision.
//!
//! ## Where everything lives
//!
//! | module | responsibility | headlessly testable |
//! |---|---|---|
//! | [`app`] | the one owner of state; frame composition; actions | partly |
//! | [`shell`] | the ribbon/mode/keymap definition, **as data** | **yes** |
//! | [`viewer`] | page index, zoom ladder, fit math, raster ceiling | **yes** |
//! | [`render`] | off-thread rasterization; pixmap → texture | worker keys only |
//! | [`canvas`] | drawing the page, wheel/ctrl-wheel/middle-drag input | geometry only |
//! | [`find`] | the search query, its options, stepping, staleness and the bar | mostly |
//! | [`panels`] | the dock's panel bodies, and the page object model behind them | mostly |
//! | [`text`] | every operator-visible string (the ui-text catalog) | n/a |
//! | [`diag`] | the opt-in `PDFCER_DIAG` trace channel | n/a |
//!
//! The split is driven by testability: a windowed UI cannot run on a CI
//! runner, so every piece of *logic* that could be wrong in a way a human
//! would notice — an off-by-one page step, a fit scale that overflows an
//! axis, a zoom that blows the rasterizer's allocation guard — is pushed
//! into a pure function with a unit test. What is left is wiring. Wiring
//! can be reviewed; arithmetic needs tests.
//!
//! ## Privacy posture, carried across unchanged
//!
//! This crate makes no network calls of any kind. The only file it opens is
//! the one it is asked to open.

#![forbid(unsafe_code)]

// O122 — finding the operator's installed Acrobat and handing the open
// document over to it. Everything that can only be true of a real machine
// (reading the registry, starting a process) is behind a trait in there, so
// the decisions are testable without either. See its header.
pub mod acrobat;
pub mod app;
pub mod canvas;
// ⚠ THE **OS** CLIPBOARD, and it is NOT `canvas::clipboard`.
//
// `canvas::clipboard` is pdfcer's own internal one — it carries selected page
// objects from one place in a document to another, in pdfcer's own types, and
// no other program can see it.
//
// THIS one is O120's second half: the bytes another application receives when
// the operator pastes into Word, Inkscape or LibreOffice, in the formats and
// the ORDER those programs read. Nothing here places anything yet, and the
// module header says at length what is missing and why shipping half of it
// would be worse than shipping none.
pub mod clipboard;
pub mod diag;
// The shell's stationary, screen-anchored surfaces — Print today, Properties
// and the settings host to come. A dialog is one transaction with a start and
// an end; a panel is somewhere you dip in and out of. See DIALOGS' own header
// for that distinction and for why a print does not push an `Action`.
pub mod dialogs;
// Find: the query and its options, the one place a search is run, the rule
// that decides what the position readout says, and the bar itself. See FIND's
// own header for the `find_text` wildcard trap it exists to avoid, for why the
// bar is docked rather than floating, and for what an edit does to a hit list.
pub mod find;
// The icon set: SVG path data, a subset parser, a tiny-skia rasterizer and
// the painter `egui-shell`'s ribbon calls back into. Supplying that painter
// is what stops the ribbon falling back to text labels — see `icons::paint`.
pub mod icons;
// OCR: what image the recogniser is shown, the thread it runs on, and the
// named refusals it can come back with. It authors no PDF — `pdfcer-core`'s
// `ocr::layer` writes the invisible mode-3 sandwich and this shell is that
// function's first caller anywhere. See OCR's own header for why recognition
// reads the document as it was OPENED, and for the y-flip it deliberately does
// not perform.
pub mod ocr;
// The dock's panel bodies — Bookmarks, Layers, Signatures, Fonts, Objects and
// the properties panel. See PANELS' own header for the reachability contract
// every one of them has to satisfy.
/// **A page drag in flight** — the state four surfaces share while the
/// operator is carrying pages from one document's page list to another's, or
/// onto the page view.
///
/// In `egui::Memory` rather than on `PdfcerApp` because switching documents
/// resets the panels' state, and switching documents is exactly what a
/// cross-document drag has to do on its way. See the module header.
pub mod pagedrag;
/// ★★★ **Does this document's page tree still agree with itself?** — the
/// structural guard every save goes through, added 2026-09-05 after the
/// operator found pages in a saved file that pdfcer did not believe were there.
///
/// It reads `/Count` **raw** off each `/Pages` node and compares it against the
/// leaves actually beneath it, because the lesson this module exists to carry
/// is that a GUI checking its own work with its own parser cannot see this
/// class of defect at all: `page_tree::pages` walks `/Kids` and reports a
/// healthy document while Acrobat, which builds its page list from the root
/// `/Count`, shows blank pages. See the module header — it also says why the
/// guard refuses rather than repairs, and why the fixture has to be nested.
pub mod pagetree;
pub mod panels;
/// Putting a password on a document, changing what it allows, and taking the
/// protection off — `OPERATOR_REQUESTS.md` **O119**, approved 2026-09-04.
///
/// The headless half of the two Security controls: what the document says
/// today, which jobs it may be offered, which of the engine's three encryption
/// verbs a choice reaches, and the atomic write at the end. The window is
/// `crate::dialogs::protect`; the split is `crate::redact`'s, and for the same
/// reason — every rule on this surface is a rule about the operator's file, and
/// a rule that can only be exercised by driving a window is one that gets
/// asserted once, by hand, and then drifts.
pub mod protect;
// Redaction: the apply pipeline and its absence proof, salvaged whole from the
// old shell — the ONE place that proof exists anywhere, `pdfcer-core` included.
// See REDACT's own header for the two full rewrites, for why the proof is made
// unskippable rather than merely available, and for why a redaction never
// overwrites the file it came from.
pub mod redact;
pub mod render;
/// A string the operator typed that must never reach a log — one type, and its
/// whole reason for existing is its `Debug`. See its header: a `{:?}` on an
/// action carrying a password writes it into the trace file `tools/ui-verify`
/// keeps as evidence.
pub mod secret;
// The pdfcer shell definition — the seven-tab ribbon, three modes, QAT and
// keymap, expressed as DATA over `egui-shell`'s manifest types rather than
// as rendering code. See SHELL_FRAMEWORK.md; this module is the sole
// consumer of `text::{ribbon, commands}`.
pub mod shell;
pub mod text;
/// ★★★ Where signature trust ANCHORS come from, and the three facts they let
/// this shell state.
///
/// The shell half of `pdfcer-core`'s `Pass 10.2`–`10.5`: locating the trust
/// list an installed Acrobat/Reader has downloaded, reading it (read-only, no
/// network, opt-in and off by default), and threading it into
/// `signature::verify_all_with_trust`.
///
/// Its header carries the rule that governs the whole subject — **this is the
/// one place in the product where a wrong answer is worse than no answer** —
/// and the consequence: integrity, coverage and trust are reported separately,
/// never folded into one badge, and `NotChecked` renders as itself.
pub mod trust;
pub mod viewer;

use std::path::PathBuf;

/// The window's opening size, in egui points.
///
/// Large enough that a fit-to-page US Letter sheet is legible without any
/// resizing, which is the first thing an operator does after launching.
const INITIAL_WINDOW_SIZE: [f32; 2] = [1100.0, 800.0];

/// The smallest window the shell will let the operator make.
///
/// Below this the canvas stops being usable rather than merely small;
/// enforcing it in the viewport builder is cheaper than defending every
/// layout against a 200×100 window.
const MIN_WINDOW_SIZE: [f32; 2] = [640.0, 480.0];

/// Start the application, optionally opening a document.
///
/// Everything from here down is the event loop. The caller has already
/// answered anything that must be decided *before* a window exists — a
/// terminal invocation must not open a window it then has to be told to
/// close, which is why argument handling belongs to the binary and not to
/// this function.
///
/// # Errors
///
/// Propagates whatever `eframe::run_native` reports: a windowing system
/// that could not be reached, a graphics backend that failed to initialise.
/// The window's icon — title bar, Alt-Tab, and the taskbar button.
///
/// # ★ Why this exists when the executable already carries an icon resource
///
/// They are two different mechanisms answering two different questions, and
/// doing only one of them leaves a visible gap.
///
/// The **resource** in `assets/pdfcer-gui.rc` is read by the shell *without
/// running the program*: Explorer, the Start menu, and the file-association
/// dialog the operator's request was about. It is the right and only answer
/// there, and it cannot be the answer here — winit creates its window class
/// without an icon, so a running window with no `window_icon` shows the
/// system's default in its title bar however good the executable's resource is.
///
/// This is the run-time half. It is the same art, at 64 px, as raw RGBA —
/// which is what `egui::IconData` takes, and why the bitmap is checked in
/// separately from the `.ico` rather than decoded out of it at start-up: the
/// alternative is carrying a PNG decoder in the binary to recover one
/// 64-pixel image. `tools/make-icon.py` writes both from one render, so they
/// cannot come to disagree.
///
/// # Why the failure mode is an empty icon rather than a panic
///
/// `include_bytes!` cannot fail — a missing file is a compile error, so the
/// bytes are always there and always the right length. The length check below
/// is therefore not defensive against absence; it is defensive against
/// `make-icon.py`'s `WINDOW_ICON_SIZE` changing without this constant
/// following. An `IconData` whose buffer does not match its dimensions is
/// rejected by winit with a log line nobody reads, so a wrong size would
/// present as "the icon silently stopped working" — the same class of quiet
/// failure the encoding bug in the `.rc` was.
fn window_icon() -> egui::IconData {
    /// Must match `WINDOW_ICON_SIZE` in `tools/make-icon.py`.
    const SIZE: u32 = 64;
    let rgba = include_bytes!("../assets/window-icon-64.rgba").to_vec();
    debug_assert_eq!(
        rgba.len(),
        (SIZE * SIZE * 4) as usize,
        // ui-text-exempt: a debug assertion message, read from a panic in a
        // developer build. Never rendered.
        "window-icon-64.rgba is not 64x64 RGBA — regenerate it with tools/make-icon.py"
    );
    egui::IconData {
        rgba,
        width: SIZE,
        height: SIZE,
    }
}

pub fn run(initial: Option<PathBuf>) -> eframe::Result {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(text::window_title())
        .with_icon(window_icon())
        .with_inner_size(INITIAL_WINDOW_SIZE)
        .with_min_inner_size(MIN_WINDOW_SIZE);

    // Test-harness placement: put the window somewhere explicit and do NOT
    // let it take focus.
    //
    // Carried across from the old shell with its reasoning intact. A GUI
    // defect has one honest oracle — the running application — but driving
    // that on the operator's own desktop takes their focus and covers their
    // work. Given a position off the visible desktop plus `with_active`
    // off, the process runs a genuine event loop that synthesized window
    // messages can drive and [`diag`] can report on, while nothing appears
    // in front of anyone. `tools/ui-verify` is the consumer.
    //
    // Deliberately NOT `with_visible(false)`: a hidden window is not merely
    // an invisible one — it stops being laid out, so the very interactions
    // under test would be skipped and the trace would show a fault that is
    // only an artefact of the harness.
    if let Some(spec) = std::env::var_os("PDFCER_DIAG_VIEWPORT") {
        let nums: Vec<f32> = spec
            .to_string_lossy()
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        if let [x, y, w, h] = nums[..] {
            viewport = viewport
                .with_position([x, y])
                .with_inner_size([w, h])
                .with_active(false);
        }
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    // Unconditional first line when tracing. Without it an empty trace has
    // two very different meanings — "the process never saw PDFCER_DIAG" and
    // "the process saw nothing worth reporting" — and a harness cannot tell
    // them apart. That ambiguity cost the old shell's investigation a round
    // trip on 2026-08-04.
    diag::trace(|| format!("start argv1={initial:?}"));

    eframe::run_native(
        "pdfcer",
        native_options,
        Box::new(move |cc| {
            app::configure_context(&cc.egui_ctx);
            let mut app = app::PdfcerApp::new();
            // ★ The window handle, captured once. See `PdfcerApp::window` for
            // what owns what, and why an unowned driver dialog is a state the
            // operator cannot get out of.
            app.window = app::window_handle(cc);
            diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("window-handle present={}", app.window.is_some())
            });
            // ★ Apply the operator's UI scale BEFORE the first frame is laid
            // out — 2026-08-17.
            //
            // `app::frame`'s step 0b applies it every frame and would reach the
            // same value on frame 2, so this is not what makes the preference
            // work. What it removes is a **visible flash**: without it, frame 1
            // is laid out at 1.0, the hook moves the factor at the end of that
            // frame, and frame 2 re-lays-out at the operator's scale. Every
            // launch of a scaled profile starts with one frame of the wrong
            // size.
            //
            // Found by `ui-verify`'s `ui_scale_resizes_the_chrome`, and found
            // sideways: the check read ribbon regions from the whole trace and
            // flagged nine controls as lying outside the window. They did — on
            // the pre-scale frame, where the window was still 1100 pt wide.
            // The overflow had correctly swallowed them by the time the scale
            // settled. So the harness's false positive was pointing at a real
            // defect one layer down, which is the more useful half of the two.
            //
            // `PdfcerApp::new` has already loaded the preferences, so this is
            // the first moment the value exists and the last moment before a
            // frame runs.
            cc.egui_ctx.set_zoom_factor(app.prefs.ui_scale);
            // Traced unconditionally, including at 1.0, and that is the point:
            // `app::frame`'s per-frame hook only traces when it MOVES the
            // factor, so once this line exists the hook is correctly silent on
            // every launch and there would otherwise be no positive evidence
            // anywhere that the preference was read at all. A diagnostic that
            // only appears when something is out of step cannot answer "was my
            // setting picked up?", which is the question an operator actually
            // has.
            diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("ui-scale-initial to={:.2}", app.prefs.ui_scale)
            });
            if let Some(path) = initial {
                app.open_path(path);
            }
            Ok(Box::new(app))
        }),
    )
}
