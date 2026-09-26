//! `redactdestination` — **where the redacted document goes**, and
//! the three questions that answer distinguishes.
//!
//! # Why this is its own file
//!
//! **R2.** `dialogs/redact.rs` reached 1,609 lines on 2026-09-08 when
//! [`Destination::OpenDocumentNow`] was added on the operator's report that the
//! dialog offered only a *"don't apply yet"* button. Its tests were already
//! split out, so the seam had to be production code — and this is a real
//! subject rather than a convenient cut: **four destinations, and every
//! behavioural difference in the dialog is a function of which one is chosen.**
//!
//! The permanence sentence, the confirm label, whether a picker opens, whether
//! a file is written, whether the page changes, and which of two acknowledgement
//! checkboxes appear — all of them branch here. A reader asking *"what does
//! confirming actually do?"* is asking about this enum.
//!
//! [`Destination::stages`] and [`Destination::writes_now`] are the two
//! predicates the dialog branches on, and they are deliberately **not**
//! complements of each other. See `stages`' own note: a predicate written as
//! the negation of another changes meaning silently when a variant is added,
//! which is exactly what would have happened to *apply now* — it writes no
//! file, so `!writes_now()` would have staged it and done nothing.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/redactdestination.md`.

/// **Where the redacted document goes.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Destination {
    /// **The open document, with nothing written — the default since
    /// 2026-09-04 (evening), and the thing he actually asked for.**
    ///
    /// `crate::redact::stage_into_session`: the removal is armed, and
    /// `file.save` / `file.save_as` / `file.save_copy` carry it out, exactly as
    /// they carry every other edit to a file.
    ///
    /// It is the **default** because it is the only one of the three that
    /// writes nothing. The old default ([`Self::NewFile`]) was safe because it
    /// never overwrote; this is safer still, because it never writes.
    ///
    /// **Its price was inverted on 2026-09-05, and the old price is worth
    /// recording because it is what the operator agreed to.** Under
    /// `Pass 250.1` this destination **finalized**: the removal happened at the
    /// click and the whole undo log went with it, disclosed above the confirm
    /// control as a step count, on his ruling *"finalizing the document and
    /// can't be undone is ok **for now**"*. `Pass 250.2` charges nothing —
    /// base, overlay and the entire undo/redo stack survive — and what is
    /// disclosed in the same place is the surprise that replaced the price:
    /// [`crate::text::redact::removal_happens_at_save`], because **the page
    /// does not change**.
    OpenDocument,
    /// **The open document, now** — the removal happens at the click and the
    /// page changes on screen.
    ///
    /// # Why this exists, and it is an operator report
    ///
    /// The operator, 2026-09-08: *"the redaction feature regressed back to just
    /// giving me the 'don't apply yet' button."*
    ///
    /// Nothing had regressed in code. [`Self::OpenDocument`] became the default
    /// on 2026-09-04 because he asked for it — *"why can't it just wait on
    /// saving until I choose to save"* — and on 2026-09-05 `Pass 250.2` made it
    /// **cost nothing**: base, overlay and the whole undo stack survive. The
    /// price it charges instead is stated in that variant's own doc: **the page
    /// does not change.**
    ///
    /// ⇒ So he pressed the one button the default offered and watched nothing
    /// happen, which is his 2026-09-04 complaint in a new form — *"what is the
    /// purpose of a redaction tool that refuses every time to do any work?"*
    ///
    /// **The deferred destination is not a mistake and is not being
    /// replaced.** It is cheaper, it is safer, and he asked for it. What was
    /// missing is the other half: a way to apply the removal and SEE it. His
    /// own ruling on the price is on the record — *"finalizing the document and
    /// can't be undone is ok for now"* — so the cost is one he has already
    /// accepted, stated at the control rather than discovered.
    ///
    /// It writes **no file**. That is what distinguishes it from
    /// [`Self::ReplaceOriginal`], and it is why it is *"this document"* rather
    /// than *"save"*: the redacted bytes replace the open session, and where
    /// they go on disk stays his decision.
    OpenDocumentNow,
    /// A new file, chosen in the save picker.
    NewFile,
    /// The document that is open, replaced in place.
    ///
    /// Offered only when the source is a real file on disk — a document
    /// created in this session has no original to replace, and a control
    /// meaning "replace nothing" is worse than an absent one.
    ReplaceOriginal,
}

/// **The destination a freshly-opened dialog starts on.**
pub const DEFAULT_DESTINATION: Destination = Destination::OpenDocument;

impl Destination {
    /// Whether this destination writes a file **now**, rather than leaving the
    /// write to a later Save.
    pub const fn writes_now(self) -> bool {
        matches!(self, Self::NewFile | Self::ReplaceOriginal)
    }

    /// Whether confirming **stages** the removal for the next save rather than
    /// performing it.
    pub const fn stages(self) -> bool {
        matches!(self, Self::OpenDocument)
    }
}
