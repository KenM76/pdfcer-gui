//! **What the status bar says when a resize is refused** — two refusals,
//! two sentences each, and the rule they share: *name the next click, not the
//! matrix.*
//!
//! Split out of `text/status/mod.rs` on 2026-09-09, when the second refusal
//! (`resize_fixed_size_marker`, engine `Pass 277.0`) pushed that file past
//! R2's 1,500 lines. The seam is the one `refused.rs` and `formdelete.rs`
//! already draw in this directory: one refusal family per file, the sentence
//! and the argument for its wording together. Nothing here decides whether
//! a resize is refused — `app::actions::annots::resize` learns that from the
//! engine's `EditError` and `app::status::decline` records which sentence;
//! this file is only the words.
//!
//! Both functions are `const` and return `&'static str` because every
//! sentence in the decline catalog is static today; the day one needs a
//! computed remedy (the font-coverage face list, `ENGINE_BACKLOG.md`) is the
//! day `Declined::line` grows a `Cow`, and that decision is not made here.

/// **A resize was refused because the artwork cannot be rebuilt.**
///
/// `OPERATOR_REQUESTS.md` O51. Two sentences, chosen by whether the drag was
/// proportional, and the split is the whole value of the message: **only one of
/// the two switches helps in each case**, and naming the wrong one would send
/// the operator to a control that changes nothing.
///
/// | drag | what fixes it |
/// |---|---|
/// | proportional | *Scale line weight* — the resize then comes out **exact** |
/// | not proportional | nothing fixes it; only *Allow the artwork to distort* proceeds |
///
/// ★★★ **It does not say "cannot".** The operator resized a shape and got
/// nothing; what they need is the next click, not a diagnosis. Both sentences
/// name a switch by the words on it, and the non-uniform one is honest that the
/// result will be imperfect rather than dressing the option up.
///
/// ★★ Neither sentence mentions appearance streams, placement matrices or
/// §12.5.5. The *reason* is real and is written down in `canvas::scaling`; what
/// belongs in a status bar is what to do. A sentence that explained the matrix
/// would be correct, unactionable, and too long to read where it appears.
///
/// ★ *"pdfcer did not draw this shape"* is in the uniform sentence because it is
/// the part an operator can verify and act on — shapes pdfcer drew resize
/// perfectly, so the message quietly tells them the difference between the two
/// kinds of object on their page.
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
///
/// # ★ Move is the remedy, and the sentence says so first
///
/// A sticky note's box has no size a reader honours — it draws the icon at one
/// size and reads the box only for **where**. The operator who dragged a
/// geometry field and got nothing needs the verb that does work on this
/// object, which is a drag of the note itself. Neither sentence says
/// "cannot": the object has a property, position, and the sentence names it.
///
/// # ★★ Why two sentences
///
/// The engine's error carries a `why` that is either the subtype's rule (a
/// `/Text` is always fixed-size, 12.5.6.4) or the annotation's own `NoZoom`
/// flag (12.5.3). The first is not the operator's to change; the second is,
/// in principle — a flag can be cleared — but this shell offers no flag
/// editor, so the sentence states the fact without promising a switch. When
/// one exists it belongs in the `by_flag` sentence and nowhere else.
///
/// # ★ Neither sentence mentions `NoZoom` by its PDF name for a sticky
///
/// An operator who placed a sticky note did not set a flag and would not know
/// what one is; "drawn at one fixed size" is the fact in their terms. For the
/// flag case the name is kept, because a foreign producer set it and the
/// operator may be looking at the file elsewhere.
#[must_use]
pub const fn resize_fixed_size_marker(by_flag: bool) -> &'static str {
    if by_flag {
        "This annotation is flagged NoZoom, so readers draw it at one fixed size and its box only says where it sits. Nothing was resized; drag it to move it."
    } else {
        "A sticky note is drawn at one fixed size, so its box only says where the note sits. Nothing was resized; drag the note to move it."
    }
}
