//! # `app::actions::write` — the three verbs that exist only to move a file
//! picker out of the layout pass
//!
//! Split out of [`super::action`] under **R2** on 2026-08-28, when that file
//! crossed 1,500 lines for the second time in one day.
//!
//! ## ★★★ The seam, and it is the sharpest one this enum has
//!
//! Every other `Action` exists because something has to happen **after** the
//! frame that raised it -- an edit through the funnel, a dialog opened, a
//! selection changed. These three exist for a reason about **egui**:
//!
//! > **A native file dialog must not open inside a layout pass.** It is a modal
//! > OS window that blocks the thread, so opening one from a widget's
//! > `clicked()` branch leaves egui part-way through a frame that will not
//! > finish until the operator has answered.
//!
//! ⇒ That is the whole of what they share, and it is a property no other family
//! in the enum has. `super`'s declaration of `action` named **markup** as the
//! measured candidate for the next sub-enum -- 370 lines, and still the largest
//! -- and the rule it stated was *"the next family of variants to **grow**"*.
//! Today the family that grew is this one, so this one moved. The markup
//! measurement stands and is still the answer the day markup grows.
//!
//! ## ★★ Why a SAVE is in here with two exports
//!
//! `Compacted` writes the document itself rather than a derivative of it, so it
//! reads at first like the odd one out. It is not: it is here because it is an
//! `Action` for exactly the reason above and for no other. The document is not
//! changed, no undo entry is made, no epoch moves -- `app::save`'s header states
//! that a save is a **read** of the session, and all three of these are.
//!
//! ★ The alternative grouping -- *"things that produce a file"* -- would put
//! `SaveCopy` in here too, and `SaveCopy` is NOT an action: it is called
//! directly, because its picker opens from the command dispatcher rather than
//! from a widget's `clicked()`. Grouping by what a verb produces would have
//! collected a set whose members do not share the property the set exists for.

/// The three verbs that exist only to move a native file picker out of the
/// layout pass. See the module header.
#[derive(Debug, Clone, PartialEq)]
pub enum WriteAction {
    /// ★ **Write one page's vector geometry out as a DXF.**
    ///
    /// Raised by `crate::dialogs::export_dxf` and by nothing else.
    ///
    /// # Why an export is an `Action` when it changes no document
    ///
    /// The same reason [`super::action::Action::SaveCopy`] and `PageAction::ExtractPages` are:
    /// **a native file dialog must not open inside a layout pass.** It is a
    /// modal OS window that blocks the thread, so opening one from a widget's
    /// `clicked()` branch leaves egui part-way through a frame that will not
    /// finish until the operator has answered.
    ///
    /// Nothing about the document is being ordered — there is nothing to order.
    /// The funnel's *invariant* does not apply here; its **reason** does.
    ///
    /// # Why the geometry is not carried
    ///
    /// `PageObjects` is a whole page decomposed, and the shell already holds
    /// one cached on `(page, epoch)`. Carrying it would clone it for a value
    /// the apply phase can borrow — and a **stale** clone: the queue drains
    /// after the frame, so an edit raised earlier in the same frame would leave
    /// the export describing the page as it was. See `export::dxf`.
    Dxf {
        /// The 0-based page, frozen when the dialog opened.
        page: usize,
        /// The engine's own options struct, edited in place by the dialog.
        ///
        /// Carried whole rather than decomposed into scale, units and two
        /// flags, for [`super::action::Action::Dimension`]'s reason one feature along: it **is**
        /// the value the writer takes, and rebuilding it in the apply arm would
        /// put a second constructor in the path.
        options: pdfcer_core::export::dxf::DxfOptions,
    },
    /// ★★★ **Write one or more pages out as a picture** — PNG, JPEG or SVG.
    /// `OPERATOR_REQUESTS.md` **O120**.
    ///
    /// Raised by `crate::dialogs::export_image` and by nothing else.
    ///
    /// # Why it carries a whole plan, like [`Self::Dxf`] and unlike
    /// [`Self::FormData`]
    ///
    /// A dialog collected four decisions before the press — which format, which
    /// pages, what resolution, whether transparency survives — and none of them
    /// can be recovered from a save picker. `FormData` needs no plan precisely
    /// because its one decision (the format) *is* recoverable from the picker,
    /// as the extension the operator types.
    ///
    /// # ★★ Why the plan is the SHELL's type and not the engine's
    ///
    /// [`Self::Dxf`] carries `DxfOptions` because that is literally the value
    /// the writer takes. There is no engine equivalent here, and that is a fact
    /// about the feature rather than a gap: the engine offers three unrelated
    /// writers (`export::encode_png`, `export::encode_jpeg`,
    /// `svg::export_svg_view`) with three options types and three error types,
    /// and *"which of the three, over which pages"* is a question none of them
    /// asks. See `super::imageexport` for the whole argument.
    ///
    /// # ★ The pages are RESOLVED, not a scope and a string
    ///
    /// The window has already parsed the typed range — it needs the answer to
    /// decide whether Export is pressable — so re-parsing in the apply phase
    /// would be a second reading of the same box against a document that may
    /// have changed pages in between.
    Image {
        /// Everything the writer needs, frozen when Export was pressed.
        plan: super::imageexport::ImagePlan,
    },
    /// **Write the form's values out as FDF, XFDF or CSV.**
    ///
    /// # ★ It carries nothing, and that is the difference from [`Self::Dxf`]
    ///
    /// The DXF export carries a page index and an options struct because a
    /// dialog collected both before the action was raised. This one has no
    /// dialog: the format is decided by the extension the operator types in the
    /// save picker, and the picker opens inside the apply phase for the reason
    /// `actions::export`'s header gives — **a native file dialog must not open
    /// inside a layout pass**, because it blocks the thread while egui is
    /// part-way through a frame.
    ///
    /// So this is an `Action` purely to move the picker out of the layout pass.
    /// Nothing about the document is being ordered, and nothing about it
    /// changes.
    FormData,
    /// ★★★ **Write the already-serialised compacted copy to a file the operator
    /// picks.**
    ///
    /// Raised by `crate::dialogs::compact` and by nothing else.
    /// `OPERATOR_REQUESTS.md` **O48**. **`app::save::compacted` carries the
    /// argument** for why the bytes travel rather than being re-serialised here:
    /// the window quoted a measurement of them, and when a confirmation quotes a
    /// number, the thing it quoted is the operand.
    ///
    /// ★ No path. The picker opens inside the apply phase, for
    /// [`Self::FormData`]'s reason — a native file dialog must not open
    /// inside a layout pass.
    Compacted {
        /// The whole file, already written by `to_full_bytes`.
        ///
        /// Moved, never cloned: it is the document, and on a dense CAD sheet
        /// that is megabytes. The dialog closes on the frame that raises this,
        /// so nothing else is still holding it by the time the queue drains.
        bytes: Vec<u8>,
        /// What the document occupied on disk before, for the disclosure.
        before: u64,
    },
}
