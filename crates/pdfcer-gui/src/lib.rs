//! # pdfcer-gui — native desktop shell (rebuild), library root
//!
//! **This crate is a library with a thin binary in front of it.** The
//! binary (`src/main.rs`) does one thing: read `argv`, call [`run`]. Every
//! module, every type and every test lives here.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/lib.md`.

#![forbid(unsafe_code)]

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
// The dock's panel bodies — Bookmarks, Layers, Signatures, Fonts, Objects and
// the properties panel. See PANELS' own header for the reachability contract
// every one of them has to satisfy.
/// **A page drag in flight** — the state four surfaces share while the
/// operator is carrying pages from one document's page list to another's, or
/// onto the page view.
pub mod panels;
/// Putting a password on a document, changing what it allows, and taking the
/// protection off — `OPERATOR_REQUESTS.md` **O119**, approved 2026-09-04.
pub mod protect;
// Redaction: the apply pipeline and its absence proof, salvaged whole from the
// old shell — the ONE place that proof exists anywhere, `pdfcer-core` included.
// See REDACT's own header for the two full rewrites, for why the proof is made
// unskippable rather than merely available, and for why a redaction never
// overwrites the file it came from.
pub mod render;
// The pdfcer shell definition — the seven-tab ribbon, three modes, QAT and
// keymap, expressed as DATA over `egui-shell`'s manifest types rather than
// as rendering code. See SHELL_FRAMEWORK.md; this module is the sole
// consumer of `text::{ribbon, commands}`.
pub mod shell;
/// **Putting the operator's own digital signature on a document** — the
/// answer to this shell's request of 2026-09-03, *"a document cannot be
/// signed"*, which `pdfcer-core` answered with `pdfcer_core::sign` and which
/// this build then failed to compile in for three days because the manifest
/// stripped the engine's default-on `signing` feature.
/// **Acrobat-compatible custom stamp collections** — the shell half of
/// engine `Pass 288.0`, and the answer to `OPERATOR_REQUESTS.md` **O169**.
pub mod stamps;
pub mod text;

pub mod viewer;

// The floor of this crate's stack, re-exported so that `crate::diag::…`,
// `crate::units::…`, `crate::secret::…` and `crate::acrobat::…` mean what
// they have always meant. THEY LIVE IN `pdfcer-gui-base` — a separate crate,
// and that is the point: cargo forbids a cycle between crates, so nothing in
// there can call back up into this one. See that crate's `Cargo.toml` for
// the admission test and DESIGNS.md for the staged plan. `ocr` (what image the
// recogniser is shown, the thread it runs on, its named refusals) joined in
// stage 2.
//
// The alternative was to rewrite ~1,200 call sites to say `pdfcer_gui_base::`.
// That would document the crossing at every use site and buy nothing else: the
// boundary is in the crate graph, not in the spelling.
#[cfg(feature = "signing")]
pub use pdfcer_gui_base::sign;
pub use pdfcer_gui_base::{
    acrobat, diag, ocr, pagedrag, pagetree, poster, redact, secret, trust, units,
};

use std::path::PathBuf;

/// The window's opening size, in egui points.
///
/// Large enough that a fit-to-page US Letter sheet is legible without any
/// resizing, which is the first thing an operator does after launching.
const INITIAL_WINDOW_SIZE: [f32; 2] = [1100.0, 800.0];

/// The smallest window the shell will let the operator make.
const MIN_WINDOW_SIZE: [f32; 2] = [640.0, 480.0];

/// Start the application, optionally opening a document.
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

    diag::trace(|| format!("start argv1={initial:?}"));

    eframe::run_native(
        "pdfcer",
        native_options,
        Box::new(move |cc| {
            app::configure_context(&cc.egui_ctx);
            let mut app = app::PdfcerApp::new();
            // The window handle, captured once. See `PdfcerApp::window` for
            // what owns what, and why an unowned driver dialog is a state the
            // operator cannot get out of.
            app.window = app::window_handle(cc);
            diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("window-handle present={}", app.window.is_some())
            });
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
