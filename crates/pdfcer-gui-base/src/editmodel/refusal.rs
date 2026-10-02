//! # `editmodel::refusal` — why a click could not open a caret
//!
//! Produced by `pdfcer_gui::canvas::textedit`; worded for the operator by
//! [`crate::text::textedit::refusal`].

/// Why a click could not start a draft, in a form the status bar can render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The click landed on no text at all.
    NoRun,
    /// The page's text could not be extracted (an image-only page, a damaged
    /// content stream).
    NoText,
    /// **The run is real and readable, and it covers no show operator the
    /// surgery could anchor on** — its glyphs come from `/ActualText`, which
    /// supplies the text without drawing it.
    ///
    /// This is not *"out of reach"*; it is *"there is nothing to reach for"*,
    /// and the engine carries it as its own [`Editability::NoAnchor`] variant
    /// precisely so a shell can say something different about it. The shell
    /// asks that predicate rather than modelling the surgery's internals — a
    /// guard built here out of what the engine happens not to support yet is a
    /// workaround that goes on refusing after the support lands.
    ///
    /// # Why refusing at the click rather than at the commit
    ///
    /// Because the attempt cannot succeed, and a control that accepts input it
    /// will discard is this project's defining defect class. Without this, the
    /// click is accepted, a caret placed, keystrokes taken, a plan built, and
    /// the engine refuses the commit **to the trace only** — so the operator
    /// sees a caret that took their sentence and threw it away in silence. An
    /// honest refusal at the click costs them one click instead.
    ///
    /// [`Editability::NoAnchor`]: pdfcer_core::text_extract::Editability::NoAnchor
    NoAnchor,
    /// **The run is real, addressable, and its font can spell nothing at
    /// all** — so the caret declines to open rather than open and refuse every
    /// key.
    ///
    /// # The engine built this distinction because this shell asked for it
    ///
    /// `EditSession::run_repertoire` could have reported an un-invertible font
    /// as an `Err`. It does not: it answers an **empty**
    /// `RunRepertoire` with `RunRepertoire::reason` set, and the engine's own
    /// rustdoc says why in as many words — *"so an editor can decline to open
    /// rather than open and refuse every key. The requesting shell asked for
    /// that distinction by name."*
    ///
    /// A run that cannot be **located** is still an `Err` and lands on
    /// [`Self::NoRun`] or [`Self::NoText`]. The two are not interchangeable:
    /// one says *there is nothing here*, this one says *this text is here and
    /// pdfcer cannot spell into it*.
    ///
    /// # Why a refusal and not a caret that greys out
    ///
    /// Because there is no key it would accept. [`Self::NoAnchor`]'s argument
    /// applies unchanged and is the module's oldest lesson: *a control that
    /// accepts input it will discard is this project's defining defect class.*
    /// A caret in a run with an empty repertoire is exactly that control — it
    /// blinks, it takes clicks, it moves with the arrow keys, and every printable
    /// key does nothing. One honest refusal at the click is cheaper than
    /// discovering it one keystroke at a time.
    ///
    /// # ⚠ The engine's `reason` string does NOT reach the operator
    ///
    /// It is good prose and it is in the engine's vocabulary: *"none of the 217
    /// character(s) this font addresses can be shown by this run — an embedded
    /// subset carries only the codes already drawn on this page (R-INV-1)"*.
    /// `text::redact`'s wording rule, generalised: **say where in HIS
    /// vocabulary, never in the engine's.** So the `reason` goes to
    /// `PDFCER_DIAG` — where the clause number is exactly what a reader wants
    /// — and `crate::text::textedit::refusal` says the same fact in his terms.
    ///
    /// It carries no payload for that reason. A `String` here would be the
    /// engine's sentence travelling towards a surface that must not show it,
    /// and `Refusal` would stop being `Copy` to carry it.
    NoUsableEncoding,
    /// **The click landed on a picture on a page that has no text at all** —
    /// a scan, or a page printed to an image.
    ///
    /// Refused rather than turned into Add text: a caret here would type new
    /// words over a picture of the old ones, which is never what a click on
    /// words in a scan means. The remedy is File ▸ Recognise text…, which
    /// `crate::declined::Declined::remedy` offers as a button.
    PictureOfText,
}
