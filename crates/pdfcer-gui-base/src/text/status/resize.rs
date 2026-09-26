//! **What the status bar says when a resize is refused** — two refusals,
//! two sentences each, and the rule they share: *name the next click, not the
//! matrix.*
//!
//!
//! Both functions are `const` and return `&'static str` because every
//! sentence in the decline catalog is static today; the day one needs a
//! computed remedy (the font-coverage face list, `ENGINE_BACKLOG.md`) is the
//! day `Declined::line` grows a `Cow`, and that decision is not made here.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/status/resize.md`.

/// **A resize was refused because the artwork cannot be rebuilt.**
#[must_use]
pub const fn resize_not_rebuildable(uniform: bool) -> &'static str {
    if uniform {
        "pdfcer did not draw this shape, so its border will thicken as the shape grows. Turn on Scale line weight in the Tool panel and the resize comes out exactly right."
    } else {
        "pdfcer did not draw this shape, and stretching it more in one direction than the other would leave its border uneven — no PDF can describe that. Resize it proportionally, or turn on Allow the artwork to distort in the Tool panel to go ahead anyway."
    }
}

/// **A resize was refused because the annotation is a fixed-size marker**
/// (`EditError::ResizeFixedSizeMarker`, engine `Pass 277.0`).
#[must_use]
pub const fn resize_fixed_size_marker(by_flag: bool) -> &'static str {
    if by_flag {
        "This annotation is flagged NoZoom, so readers draw it at one fixed size and its box only says where it sits. Nothing was resized; drag it to move it."
    } else {
        "A sticky note is drawn at one fixed size, so its box only says where the note sits. Nothing was resized; drag the note to move it."
    }
}
