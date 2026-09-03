//! Capture the target window to an [`Image`], and refuse a capture that is not
//! evidence.
//!
//! ## The guard is the point
//!
//! Grabbing pixels is four lines. The reason this is a module is the check at
//! the end of [`window`]: **a near-uniform capture is refused rather than
//! returned.**
//!
//! A blank capture is indistinguishable from a real one at the call site. The
//! file exists, the call succeeded, and the only thing that says "this is not
//! evidence" is a human looking at it. pdfcer's predecessor recorded exactly
//! what happens without the guard: a run of blank screenshots got a
//! plausible-sounding cause invented for them (a compositor recomposite race),
//! the fix appeared to work because the real cause happened to go away at the
//! same time, and the actual cause — the operator's **display had powered
//! down**, and `CopyFromScreen` reads the composited desktop, which has
//! nothing to read from a sleeping display — was found later, by the operator.
//!
//! The lesson kept from that, and the reason this function returns an error
//! instead of a picture: a change that appears to fix something is not
//! evidence for the story told about it, and the fix that mattered was not the
//! sleep that was tried first — it was refusing to treat a uniform capture as
//! evidence at all.
//!
//! ## Known causes of a uniform capture, in the order they have occurred
//!
//! 1. **The display is asleep or powered off.** Nothing to read.
//! 2. **The window was not raised**, so the region belongs to another
//!    application. This usually looks like a screenshot *of that application*
//!    rather than a blank — which is why [`window`] raises first and why the
//!    check is for uniformity rather than for blankness specifically.
//! 3. **The application died** before the shot. Check its trace.

use std::path::Path;

use crate::error::{Error, Result};
use crate::geom::PixRect;
use crate::image::Image;
use crate::launch::Session;
use crate::pixels;
use crate::sys;

/// How long to let a raised window finish painting before reading the desktop.
///
/// 700 ms, measured rather than guessed — and the number matters less than the
/// note attached to it in the predecessor script, which briefly read 2500 ms
/// with an invented explanation. Three consecutive captures at 700 ms produced
/// identical non-blank content; the longer sleep bought nothing and cost 1.8 s
/// per capture.
const RAISE_SETTLE_MS: u64 = 700;

/// Capture the session's window client area.
///
/// Raises the window, waits for it to paint, reads the desktop region it
/// occupies, and refuses the result if it is near-uniform.
///
/// # Errors
///
/// * The window cannot be measured (it has closed).
/// * The screen grab failed.
/// * The capture is near-uniform — see the module docs. This is reported as an
///   error, not as a picture, because the caller would otherwise assert on it
///   and produce a confident verdict about nothing.
pub fn window(session: &Session) -> Result<Image> {
    let frame = session.frame()?;
    frame_capture(session, &frame, true)
}

/// **Capture an arbitrary window of the application under test**, given the
/// frame that describes it.
///
/// # ★★ Why a pixel check needed this on 2026-08-21
///
/// Because [`window`] captures the application's own window, and as of that day
/// the surface a pixel check is measuring is often **not in it**. Thirteen
/// dialogs became real OS windows; a capture of the application shows the page
/// where the dialog used to be, and the contrast sampler goes on sampling —
/// reporting a confident 1.51:1 about a piece of the drawing.
///
/// That is the worst available failure: a measurement of the wrong surface is
/// indistinguishable from a measurement of a broken one. `settings_headings_legible`
/// produced exactly that, naming two headings that render perfectly well.
///
/// `raise` is the caller's choice here rather than unconditional, because the
/// caller may already have raised the dialog to click something in it and a
/// second raise of the MAIN window would put it behind again — which is the
/// same z-order trap `Driver::window_owning` exists for.
pub fn frame_capture(
    session: &Session,
    frame: &crate::coords::WindowFrame,
    raise: bool,
) -> Result<Image> {
    // ★★ RAISE THE WINDOW THIS FRAME DESCRIBES, which is not always the
    // application's own. A screen grab reads the COMPOSITED DESKTOP, so a
    // dialog sitting behind the main window is captured as the main window —
    // plausible pixels, wrong surface. `session.raise()` would make that
    // certain rather than likely, by putting the main window in front.
    //
    // The window is found by matching client origins rather than by z-order,
    // for the reason `Driver::window_owning` states at length: the raise is
    // about to change z-order, so z-order cannot be the way in.
    if raise {
        let found = session
            .window()
            .and_then(crate::sys::pid_of_window)
            .map(crate::sys::windows_for_pid)
            .and_then(|windows| {
                windows.into_iter().find(|w| {
                    crate::sys::window_frame(*w)
                        .is_ok_and(|f| f.client_origin == frame.client_origin)
                })
            });
        match found {
            Some(w) => crate::sys::raise_window(w),
            None => session.raise(),
        }
        std::thread::sleep(std::time::Duration::from_millis(RAISE_SETTLE_MS));
    }
    let region = frame.client_pixels();
    if region.area() == 0 {
        return Err(Error::new(
            "the window's client area has no size; it may be minimised",
        ));
    }
    let bgra = sys::capture_screen(region)?;
    let image = Image::from_bgra(region.w, region.h, bgra)?;

    let whole = PixRect::new(0, 0, image.width(), image.height());
    let uniformity = pixels::region_not_uniform(&image, whole);
    if uniformity.is_uniform() {
        return Err(Error::new(format!(
            "the capture is near-UNIFORM ({}). This is almost certainly NOT a picture of the \
             application; do not treat it as evidence. Known causes, in the order they have \
             actually occurred: (1) the display is asleep or powered off — the grab reads the \
             composited desktop and there is nothing there to read; (2) the window was not \
             raised, so the region belongs to another application; (3) the application died \
             before the shot — check its trace at {}.",
            uniformity.summary(),
            session.trace_path().display()
        )));
    }
    Ok(image)
}

/// Capture and also write the PNG, returning both.
///
/// Every check that looks at pixels saves its evidence, pass or fail. On a
/// failure it is what the reader needs; on a pass it is what makes the *next*
/// failure diagnosable, because there is something to compare against.
pub fn window_to_png(session: &Session, path: &Path) -> Result<Image> {
    let image = window(session)?;
    image.save_png(path)?;
    Ok(image)
}

/// [`frame_capture`] with the PNG written, for the same reason
/// [`window_to_png`] writes one.
pub fn frame_to_png(
    session: &Session,
    frame: &crate::coords::WindowFrame,
    path: &Path,
) -> Result<Image> {
    let image = frame_capture(session, frame, true)?;
    image.save_png(path)?;
    Ok(image)
}
