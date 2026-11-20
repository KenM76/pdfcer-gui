//! # `text::status::card` — which way the page on screen was drawn
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/status/card.md`.

/// The render notes `rest`, led by saying the card drew the page.
#[must_use]
pub fn card_drew(rest: &str) -> String {
    format!("Drawn on the graphics card. {rest}")
}

/// The render notes `rest`, led by saying the processor drew the page with
/// the switch on; `why` is one of the reasons below.
#[must_use]
pub fn card_processor(why: &str, rest: &str) -> String {
    format!("Drawn on the processor: {why}. {rest}")
}

/// No graphics context to draw with.
#[must_use]
pub const fn card_no_context() -> &'static str {
    "this window has no graphics card drawing available"
}

/// The engine cannot record the page for replay.
#[must_use]
pub const fn card_not_recordable() -> &'static str {
    "the page uses content the graphics card path cannot record"
}

/// `GpuRefusal::Image`.
#[must_use]
pub const fn card_refused_image() -> &'static str {
    "the page has a picture the graphics card cannot sample exactly"
}

/// `GpuRefusal::Shading`.
#[must_use]
pub const fn card_refused_shading() -> &'static str {
    "the page has a smooth shading, which the graphics card does not draw yet"
}

/// `GpuRefusal::Transparency`.
#[must_use]
pub const fn card_refused_transparency() -> &'static str {
    "the page uses a soft mask or a blend mode the graphics card does not draw"
}

/// `GpuRefusal::Curves`.
#[must_use]
pub const fn card_refused_curves() -> &'static str {
    "the page has curves the graphics card path does not flatten"
}

/// `GpuRefusal::TooMuchData`.
#[must_use]
pub const fn card_refused_size() -> &'static str {
    "the page is too large to hand to the graphics card"
}

/// The drawer failed; it is retired for the session. `driver` is the
/// driver's own words.
#[must_use]
pub fn card_failed(driver: &str) -> String {
    format!("the graphics card failed to draw ({driver}), so it is off until pdfcer is restarted")
}
