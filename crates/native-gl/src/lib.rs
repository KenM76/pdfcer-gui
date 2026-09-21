//! # `native-gl` — **the OpenGL error flag, read**
//!
//! ## The defect this exists to make visible
//!
//! A texture upload has two failure modes and they look nothing alike.
//!
//! * **Too large on one axis** — `egui_glow`'s `Painter::upload_texture_srgb`
//!   carries a bare `assert!` on `max_texture_side`, and `egui`'s own
//!   `Context::load_texture` guards size with a `debug_assert!` only. A release
//!   build therefore hands an oversized image straight through to a panic.
//! * **Legal size, no memory** — nothing panics, nothing logs, and nothing
//!   returns an error. GL raises `GL_OUT_OF_MEMORY` on a flag and carries on;
//!   the texture object stays bound with no storage; the program draws a
//!   **blank rectangle at full frame rate**.
//!
//! The second is silent because `egui_glow`'s `check_for_gl_error!` wraps its
//! whole body in `if cfg!(debug_assertions)` — so in the build the operator
//! actually runs, the error flag is never read by anything in the render
//! stack. `check_for_gl_error_even_in_release!` is public and ungated but is
//! not called from the paint path, so it cannot be reached by configuration.
//!
//! ⇒ This crate is the instrument. It reads the flag, and that is all it does.
//!
//! ## ★★ Why `unsafe` is here rather than at the call site
//!
//! `crates/pdfcer-gui/src/lib.rs` and `main.rs` both open with
//! `#![forbid(unsafe_code)]`. `forbid` cannot be relaxed by an inner `allow` —
//! that is precisely why it was chosen over `deny` — so an `unsafe` block in
//! the application is not a lint to quieten, it is a claim to give up.
//! `crates/native-window` and `crates/native-clipboard` answer the same
//! question for `user32`; this is that answer applied to `glow`, whose every
//! entry point is an `unsafe fn`.
//!
//! ## What this crate refuses to know
//!
//! Which upload failed, whose zoom should come down, and whether a blank
//! canvas matters. All three are policy and all three live with the caller.
//! This crate returns a [`Drained`] and has never heard of a document.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]

use glow::HasContext as _;

/// How many times [`drain`] will call `glGetError` before giving up.
///
/// ★ **A drain loop must be bounded, and this is not defensive
/// decoration.** The specified contract is that the flag set empties and
/// `glGetError` then returns `GL_NO_ERROR` — but this runs once per frame on
/// the UI thread against a third-party driver, and a context that has been
/// lost or a driver that is misbehaving turns "loop until it says no error"
/// into a hung program with no message. A bound converts the worst case from a
/// freeze into a wrong number.
///
/// Sixteen because the flag set is small in every implementation and a frame
/// that genuinely raised sixteen distinct errors has a problem this crate is
/// not going to characterise anyway.
const MAX_DRAIN: usize = 16;

/// What one call to [`drain`] took off the error flag.
///
/// `Copy` and allocation-free: this is read every frame, and on every frame
/// where nothing went wrong it is [`Drained::CLEAN`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Drained {
    /// `GL_OUT_OF_MEMORY` was among the codes drained.
    ///
    /// ★★ **This is the only field a caller should branch on.** It is the one
    /// failure that presents as a blank picture rather than as a crash or a
    /// wrong picture, and therefore the one a user reports as *"the view went
    /// blank"* with nothing in any log to corroborate it.
    pub out_of_memory: bool,
    /// How many codes were drained in total, `out_of_memory` included.
    ///
    /// Zero means the flag was already clear. Equal to [`MAX_DRAIN`] means the
    /// drain hit its bound and the flag may still hold more — see
    /// [`Self::truncated`].
    pub count: usize,
    /// The first code drained that was **not** `GL_OUT_OF_MEMORY`, if any.
    ///
    /// Kept for the trace rather than for logic. An `INVALID_OPERATION` from
    /// somewhere else in the frame is not this crate's business, but a caller
    /// attributing a blank canvas to memory pressure wants to know the flag
    /// also held something unrelated — that is the difference between one
    /// explanation and two.
    pub first_other: Option<u32>,
}

impl Drained {
    /// Nothing was on the flag.
    pub const CLEAN: Self = Self {
        out_of_memory: false,
        count: 0,
        first_other: None,
    };

    /// The drain stopped at [`MAX_DRAIN`] and the flag may still hold codes.
    ///
    /// ⚠ A caller reporting *"no errors this frame"* must check this first: a
    /// truncated drain that happened not to contain `GL_OUT_OF_MEMORY` in its
    /// first sixteen codes is **not** evidence that memory was fine.
    #[must_use]
    pub const fn truncated(self) -> bool {
        self.count >= MAX_DRAIN
    }

    /// Nothing at all was drained.
    #[must_use]
    pub const fn is_clean(self) -> bool {
        self.count == 0
    }
}

/// Take every code currently on the context's error flag.
///
/// Returns what was there. The flag is **empty afterwards**, which is the
/// whole mechanism and also the whole hazard — see below.
///
/// # ⚠ `glGetError` returns ONE code and clears it
///
/// A single call is a bug. The flag is a *set*, and reading one member hides
/// the rest — so a frame that raised `GL_INVALID_OPERATION` first and
/// `GL_OUT_OF_MEMORY` second reports as "an invalid operation" and the blank
/// canvas goes unexplained. Hence the loop, and hence [`Drained::count`].
///
/// # ⚠ The flag is GLOBAL to the context, so this is destructive
///
/// Draining removes the codes for every other reader too. In a release build
/// that is safe and is why this is only wired up there: `egui_glow`'s
/// `check_for_gl_error!` compiles to nothing, so no other part of the render
/// stack is looking. In a **debug** build `egui_glow` *is* looking, and a
/// drain scheduled before its check would silently swallow the diagnostics it
/// would otherwise print.
///
/// ⇒ Call this at a point where the frame's GL work is finished and nothing
/// else will read the flag. The caller owns that ordering; this function
/// cannot enforce it.
///
/// # What a code does NOT tell you
///
/// Which call raised it. GL's error flag carries no provenance whatsoever, so
/// `out_of_memory` means *"something in the work since the last drain ran out
/// of memory"* and nothing more precise. A caller that acts on it — by
/// lowering a zoom ceiling, say — must establish for itself that the work
/// since the last drain included the upload it intends to blame, or it will
/// attribute a failed glyph atlas to the page the operator is looking at.
#[must_use]
pub fn drain(gl: &glow::Context) -> Drained {
    let mut out = Drained::CLEAN;
    for _ in 0..MAX_DRAIN {
        // SAFETY: `get_error` is `unsafe` only because every `glow` entry
        // point is — the trait cannot know a caller holds a current context.
        // It reads and clears a flag, takes no pointer, allocates nothing, and
        // has no failure mode of its own. The caller's `&glow::Context` is the
        // one `eframe` created and made current for this thread, so the
        // precondition the trait cannot express is satisfied by construction.
        let code = unsafe { gl.get_error() };
        if code == glow::NO_ERROR {
            break;
        }
        out.count += 1;
        if code == glow::OUT_OF_MEMORY {
            out.out_of_memory = true;
        } else if out.first_other.is_none() {
            out.first_other = Some(code);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The clean value is the default, so a caller that builds one either way
    /// gets the same thing. Cheap to assert and it pins the two together.
    #[test]
    fn clean_is_the_default() {
        assert_eq!(Drained::CLEAN, Drained::default());
        assert!(Drained::CLEAN.is_clean());
        assert!(!Drained::CLEAN.truncated());
    }

    /// ★ `truncated` must be true exactly at the bound, not one short of it.
    ///
    /// An off-by-one here is a false "the flag was fully read", which is the
    /// one wrong answer this field exists to prevent.
    #[test]
    fn truncated_is_true_at_the_bound_and_not_below_it() {
        let below = Drained {
            count: MAX_DRAIN - 1,
            ..Drained::CLEAN
        };
        let at = Drained {
            count: MAX_DRAIN,
            ..Drained::CLEAN
        };
        assert!(!below.truncated());
        assert!(at.truncated());
    }

    /// A drained set that is not clean is not clean even with no OOM in it.
    ///
    /// `is_clean` asks about the COUNT, deliberately: a frame that raised an
    /// unrelated error still had something happen, and a caller tracing
    /// "nothing to report" must not be told that by a set containing an
    /// `INVALID_OPERATION`.
    #[test]
    fn an_unrelated_error_is_not_clean() {
        let d = Drained {
            out_of_memory: false,
            count: 1,
            first_other: Some(glow::INVALID_OPERATION),
        };
        assert!(!d.is_clean());
        assert!(!d.out_of_memory);
    }
}
