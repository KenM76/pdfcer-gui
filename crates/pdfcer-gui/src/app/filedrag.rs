//! # `app::filedrag` — **where on the window a file was dropped**
//!
//! ## What this closes
//!
//! The operator, 2026-08-31 (`OPERATOR_REQUESTS.md` **O67**):
//!
//! > *"I should be able to drag and drop documents into the thumbnails section
//! > of another pdf to import the pages."*
//!
//! [`crate::app::dropped`] already reads dropped files and opens them. What it
//! cannot do — and says so in its own header — is tell *where* the file
//! landed: *"`egui` reports drops on the **`Context`**, not on a widget."* A
//! drop on the Pages panel and a drop on the ribbon are the same event, so
//! *"drop it onto the thumbnails"* was not expressible.
//!
//! This module supplies the missing coordinate and the protocol that lets a
//! surface claim a drop that landed on it.
//!
//! ## ★★★ The coordinate does not exist in the toolkit, at all
//!
//! Not "is awkward to get" — is **discarded**, twice, on the way up:
//!
//! ```text
//! // winit 0.30.13, platform_impl/windows/drop_handler.rs
//! pub unsafe extern "system" fn DragOver(this, _grfKeyState, _pt: *const POINTL, …)
//! pub unsafe extern "system" fn Drop    (this, pDataObj, _grfKeyState, _pt: *const POINTL, …)
//! ```
//!
//! The OLE drop point arrives in `_pt` and is thrown away in both. And the
//! usual fallback is not available either: **during an OLE drag the window
//! receives no mouse-move messages**, so `egui`'s `pointer.latest_pos()` is
//! stale from before the drag started — typically wherever the operator last
//! clicked, which is exactly the kind of plausible-but-wrong coordinate this
//! project has been bitten by three times.
//!
//! ⇒ So the position is asked of the operating system directly, once per
//! frame, through [`native_window::cursor_position`]. That is the same
//! argument `native-window` was created for: the toolkit will not say
//! something the platform knows and the operator can see.
//!
//! ## ★★ The protocol: a surface CLAIMS a drop; the fallback runs last
//!
//! A dropped file is recorded at the top of the frame with the point it landed
//! on, and then sits there:
//!
//! | when | who | what |
//! |---|---|---|
//! | top of frame | [`poll`] | record the drop and the point; keep the hover alive |
//! | during the frame | any surface | [`aim`] to see if the point is over it, [`claim`] to take it |
//! | end of frame | [`crate::app::frame`] | [`unclaimed`] — whatever is left means what it always meant |
//!
//! ★ The **fallback is unconditional**, and that is the property worth
//! protecting. A drop that no surface claims still opens the document, or
//! inserts the image, or explains the refusal — exactly as before this module
//! existed. So a surface forgetting to claim costs a *feature*, never the
//! drop: the failure mode is "it opened in a new tab instead of inserting",
//! which an operator can see and undo, rather than a file that vanished.
//!
//! ## ★ One accessor for both phases, deliberately
//!
//! [`aim`] answers *"where is the file-drag pointer?"* whether the file is
//! still hovering or has just landed. That is what lets a surface draw its
//! caret and resolve its drop with **one** piece of geometry code — and the
//! alternative, two accessors, is a place for the preview and the outcome to
//! disagree about where the file was going. The caret an operator watched
//! would then be a promise the drop did not keep.
//!
//! ## Rule 4
//!
//! Everything drawn from this is a **cursor**: a caret in a gap while a button
//! (or in this case a file) is held, gone the instant it lands. Nothing here
//! marks content, tints a page, or draws a second rendering path. The words
//! half of the disclosure is the status note the insert itself records.

use std::path::PathBuf;

/// A drop that has landed and has not yet been claimed.
///
/// ★ Carries **every** path, not just the first. The claiming surface needs to
/// know how many arrived so it can say so, and `dropped`'s
/// "only the first" rule is that module's decision to make rather than this
/// one's — this module's job ends at *where*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Landed {
    /// The files, in the order the platform delivered them.
    pub paths: Vec<PathBuf>,
    /// Where the pointer was, in `egui` window points, or `None` when the
    /// operating system declined to say (see [`aim`]).
    pub at: Option<egui::Pos2>,
}

/// The memory slot holding this frame's unclaimed drop.
const LANDED_KEY: &str = "pdfcer.filedrag.landed"; // ui-text-exempt: a memory key, never displayed

/// The environment variable a harness uses to simulate one drop.
///
/// Moved here from `app::dropped` unchanged, because the simulated drop now
/// needs the same *position* the real one has, and the position is this
/// module's subject.
const DIAG_DROP_PATH: &str = "PDFCER_DIAG_DROP_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// How long to wait before firing [`DIAG_DROP_PATH`], in milliseconds.
///
/// # ★★★ Why a delay is the difference between drivable and not
///
/// A real drop carries a position, and this feature is entirely *about* the
/// position — so a check must be able to say **where** the simulated file
/// lands. It cannot pass a coordinate: the interesting points are tile
/// rectangles inside a scrolling panel, which do not exist until the
/// application has laid itself out, long after the environment was read.
///
/// So the check moves the **real cursor** over the tile it means — which it
/// can do, because that is the one thing `ui-verify` is good at — and this
/// delay gives it time to do so before the drop fires. The position is then
/// genuinely read from the operating system by the same line of code a real
/// drop uses.
///
/// ⇒ The only synthetic part left is the OLE payload, which cannot be
/// synthesised by moving a mouse and is precisely why the seam exists at all.
const DIAG_DROP_AFTER_MS: &str = "PDFCER_DIAG_DROP_AFTER_MS"; // ui-text-exempt: an environment variable name, never displayed

thread_local! {
    /// Whether [`DIAG_DROP_PATH`] has already been honoured.
    ///
    /// A property of the *process*, not of a document — the same reasoning
    /// `app::dropped` gave when this flag lived there.
    static DROPPED_ONCE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// When the process started drawing, for [`DIAG_DROP_AFTER_MS`].
    ///
    /// ★ Set on the first [`poll`] rather than at `main`, so the delay is
    /// measured from the first frame — which is what the check is waiting for
    /// too. Measuring from process start would count the time `eframe` spends
    /// creating a window against a budget meant for the harness's pointer.
    static FIRST_FRAME: std::cell::OnceCell<std::time::Instant> = const { std::cell::OnceCell::new() };
}

/// One `egui::Id` per key, spelled once.
fn id(key: &str) -> egui::Id {
    egui::Id::new(key)
}

/// **Read this frame's drops and keep a hovering drag alive.** Call once, at
/// the top of the frame, before anything is drawn.
///
/// Records a [`Landed`] when files arrived, so that a surface drawn later in
/// the same frame can claim it. Requests a repaint while a file is hovering,
/// because otherwise there would not *be* a later frame: `egui` repaints on
/// events, a hovering file produces exactly one (`HoveredFile`, on entry), and
/// a caret that appeared once and then froze would be worse than no caret.
pub fn poll(ctx: &egui::Context) {
    FIRST_FRAME.with(|c| {
        let _ = c.set(std::time::Instant::now());
    });

    if hovering(ctx) {
        // The pointer moves without producing an event; the caret has to
        // follow it. See this function's docs.
        ctx.request_repaint();
    }

    let mut paths: Vec<PathBuf> = ctx.input(|i| {
        i.raw
            .dropped_files
            .iter()
            .filter_map(|f| f.path.clone())
            .collect()
    });
    if paths.is_empty() {
        paths.extend(diag_drop());
    }
    if paths.is_empty() {
        return;
    }

    let at = aim(ctx);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "file-dropped n={} first={:?} at={}",
            paths.len(),
            paths.first().and_then(|p| p.file_name()),
            at.map_or_else(
                || "unknown".to_owned(),
                |p| format!("{:.1},{:.1}", p.x, p.y)
            )
        )
    });
    ctx.data_mut(|d| d.insert_temp(id(LANDED_KEY), Landed { paths, at }));
}

/// Whether a file is being dragged over the window right now.
///
/// `hovered_files` is **cloned** rather than taken by `egui`'s `RawInput`, so
/// it stands for as long as the drag does and this is a state rather than an
/// edge — which is what makes a per-frame caret possible.
#[must_use]
pub fn hovering(ctx: &egui::Context) -> bool {
    ctx.input(|i| !i.raw.hovered_files.is_empty())
}

/// **Where the file-drag pointer is**, in `egui` window points.
///
/// One accessor for the hover and for the landing — see the module header for
/// why that is deliberate rather than lazy.
///
/// # The conversion, and why each term
///
/// ```text
/// egui_point = desktop_physical_px / pixels_per_point − viewport.inner_rect.min
/// ```
///
/// `egui-winit` divides **both** the window's client rectangle and every
/// pointer position by the same `pixels_per_point` — which is
/// `zoom_factor × native_pixels_per_point`, not the native scale alone. Using
/// the native scale here would be right at 100 % zoom and wrong at every other
/// setting, which is the shape of bug that ships and is then reported as
/// *"it only misses when I make the text bigger"*.
///
/// Returns `None` when the platform declines to give a cursor position, or
/// when the viewport has not published its rectangle yet (the first frame, and
/// while minimised). A caller that gets `None` must fall back to
/// position-blind behaviour rather than to a guess.
#[must_use]
pub fn aim(ctx: &egui::Context) -> Option<egui::Pos2> {
    let (x, y) = native_window::cursor_position()?;
    let inner = ctx.input(|i| i.viewport().inner_rect)?;
    let ppp = ctx.pixels_per_point();
    if ppp <= 0.0 {
        return None;
    }
    #[allow(
        clippy::cast_precision_loss,
        reason = "a desktop coordinate is at most six digits; f32 carries seven" // ui-text-exempt: a lint justification, never displayed
    )]
    let desktop = egui::pos2(x as f32 / ppp, y as f32 / ppp);
    Some(desktop - inner.min.to_vec2())
}

/// The drop waiting to be claimed, if there is one. Does **not** consume it.
#[must_use]
pub fn landed(ctx: &egui::Context) -> Option<Landed> {
    ctx.data(|d| d.get_temp::<Landed>(id(LANDED_KEY)))
}

/// **Take the drop**, because this surface has decided it is theirs.
///
/// A surface calls this only once it has resolved what the drop means on its
/// own geometry — not merely because the point is inside its rectangle. A
/// claim it cannot act on would be worse than no claim, because the fallback
/// has already been skipped by then.
pub fn claim(ctx: &egui::Context) -> Option<Landed> {
    ctx.data_mut(|d| {
        let taken = d.get_temp::<Landed>(id(LANDED_KEY));
        if taken.is_some() {
            d.remove::<Landed>(id(LANDED_KEY));
        }
        taken
    })
}

/// **Whatever no surface claimed**, for the end of the frame.
///
/// The same operation as [`claim`] and a different *name*, because the two
/// call sites mean opposite things and a reader of `app::frame` should not
/// have to work out which one a bare `claim()` is. There is exactly one caller
/// of this, and it is the fallback.
pub fn unclaimed(ctx: &egui::Context) -> Option<Landed> {
    claim(ctx)
}

/// **Plant a landing**, for the tests of a surface that claims one.
///
/// ★ `#[cfg(test)]` rather than a public seam: a landing is written by exactly
/// one function in the running program ([`poll`]), and a second writer would
/// be a second answer to *"was a file dropped?"* that could disagree with the
/// input it is supposed to be reporting.
#[cfg(test)]
pub fn test_land(ctx: &egui::Context, landing: Landed) {
    ctx.data_mut(|d| d.insert_temp(id(LANDED_KEY), landing));
}

/// The harness's simulated drop, fired at most once and only after the
/// configured delay.
///
/// See [`DIAG_DROP_AFTER_MS`] for why the delay is what makes the *position*
/// half of this feature drivable at all.
fn diag_drop() -> Option<PathBuf> {
    if DROPPED_ONCE.with(std::cell::Cell::get) {
        return None;
    }
    let path = std::env::var_os(DIAG_DROP_PATH)?;
    let after = std::env::var(DIAG_DROP_AFTER_MS)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);
    let waited = FIRST_FRAME.with(|c| c.get().map(std::time::Instant::elapsed));
    if waited.is_none_or(|w| w < std::time::Duration::from_millis(after)) {
        return None;
    }
    DROPPED_ONCE.with(|c| c.set(true));
    let path = PathBuf::from(path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("dropped source=env path={path:?} after_ms={after}")
    });
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A drop is recorded, claimed once, and gone.
    #[test]
    fn a_landing_is_claimed_exactly_once() {
        let ctx = egui::Context::default();
        assert!(landed(&ctx).is_none());
        ctx.data_mut(|d| {
            d.insert_temp(
                id(LANDED_KEY),
                Landed {
                    paths: vec![PathBuf::from("a.pdf")],
                    at: Some(egui::pos2(10.0, 20.0)),
                },
            );
        });
        assert!(landed(&ctx).is_some(), "reading does not consume");
        assert!(landed(&ctx).is_some(), "…twice");
        assert!(claim(&ctx).is_some());
        assert!(
            claim(&ctx).is_none(),
            "a second surface must not act on a drop the first one took"
        );
    }

    /// ★★ **`unclaimed` sees what `claim` left**, which is the whole fallback.
    ///
    /// Written as a sequence rather than as two assertions about one function,
    /// because the property is about the ORDER: a surface claims during the
    /// frame, the fallback runs at the end, and exactly one of them acts.
    #[test]
    fn the_fallback_gets_a_drop_no_surface_took() {
        let ctx = egui::Context::default();
        let landing = Landed {
            paths: vec![PathBuf::from("drawing.pdf")],
            at: None,
        };
        ctx.data_mut(|d| d.insert_temp(id(LANDED_KEY), landing.clone()));
        assert_eq!(
            unclaimed(&ctx),
            Some(landing.clone()),
            "nobody claimed it, so the ordinary meaning of a drop still happens"
        );

        ctx.data_mut(|d| d.insert_temp(id(LANDED_KEY), landing));
        assert!(claim(&ctx).is_some(), "this time a surface takes it");
        assert!(
            unclaimed(&ctx).is_none(),
            "★ and the fallback must NOT also act — a claimed drop that still \
             opened a tab would do two things to one gesture"
        );
    }

    /// ★ A landing carries every path, not the first.
    ///
    /// `app::dropped` decides that only the first is acted on and says so to
    /// the operator. This module must not make that decision early, or the
    /// claiming surface loses the ability to say how many arrived.
    #[test]
    fn a_landing_keeps_every_path() {
        let landing = Landed {
            paths: vec![PathBuf::from("a.pdf"), PathBuf::from("b.pdf")],
            at: None,
        };
        assert_eq!(landing.paths.len(), 2);
    }
}
