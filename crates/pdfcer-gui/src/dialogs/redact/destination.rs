//! `dialogs::redact::destination` — **where the redacted document goes**, and
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
//! ★ [`Destination::stages`] and [`Destination::writes_now`] are the two
//! predicates the dialog branches on, and they are deliberately **not**
//! complements of each other. See `stages`' own note: a predicate written as
//! the negation of another changes meaning silently when a variant is added,
//! which is exactly what would have happened to *apply now* — it writes no
//! file, so `!writes_now()` would have staged it and done nothing.

/// **Where the redacted document goes.**
///
/// ★★★ Added 2026-09-04, on the operator's explicit instruction, and it
/// reverses a ruling this file used to state as settled. His words:
///
/// > *"why does it have to save to a new file right away? Why can't it just
/// > wait on saving until I choose to save over the existing file or save as a
/// > new file?"*
///
/// # What this file used to say, and why it was wrong
///
/// [`RedactDialog::commit`] read, verbatim: *"There is no 'save over the
/// original' branch to find, because there is none to write, and on this
/// operation that is the difference between a copy and the destruction of the
/// only remaining source of the content being removed."*
///
/// The premise is true and the conclusion did not follow. Overwriting the
/// source **is** the destruction of the only remaining copy — but the person
/// entitled to decide that is the person who marked the content for
/// destruction in the first place, and forcing a copy does not protect him from
/// the decision, it only makes him perform it in two steps with a stray file
/// left over. Every other edit in this shell trusts him with Save and Save As
/// on exactly this reasoning; the redaction had quietly taken the decision away
/// on his behalf.
///
/// ★ What the old ruling was *actually* protecting is kept, and kept in the
/// form it belongs in: [`Self::NewFile`] is still the **default**, and
/// `crate::dialogs::redact::suggested_path` still never suggests the source. A
/// safe default is a mechanism; a warning is something to click past. The
/// change is that the safe default is now a default rather than the only
/// option.
///
/// # ★★★ CORRECTED the same evening — the deferred half SHIPPED, and this
/// section used to say it could not
///
/// What stood here, verbatim, written at about midday:
///
/// > *"⚠ What this deliberately does NOT do, and why. He asked for the write to
/// > be deferred — applied into the session, saved later by Save or Save As
/// > like any other edit. **The engine cannot express that**, and this dialog
/// > does not fake it. [`pdfcer_core::redact::apply_redactions`] takes a
/// > `&Document` and returns `Vec<u8>`; `EditSession`'s only constructor is
/// > `new(Document)` and it has no `replace_document`, no `rebase` and no
/// > `reload`."*
///
/// Every clause of that was true when it was written and was filed as an engine
/// request the same morning. **The engine answered it that afternoon**:
/// `EditSession::apply_redactions` (`Pass 250.1`, `225db51`) applies the
/// removal into the session and leaves the write to the ordinary save verbs. So
/// the paragraph is not softened, it is **replaced** — [`Self::OpenDocument`]
/// is the destination it said was impossible, and it is now the default.
///
/// ★ What the old paragraph got right and is worth keeping: the manoeuvre it
/// refused — *"building a second `EditSession` and swapping it under the open
/// document"* — is still refused, and the engine did not ship that either. Its
/// verb collapses the session in place, keeps the document identity, and clears
/// the undo log **by name** rather than by accident, which is the difference
/// between a disclosed consequence and a silent data loss. The refusal was
/// right; only its conclusion about what could exist was wrong.
///
/// The request is at `D:\Dev\FeatureRequests\pdfce_FeatureRequests\
/// open\request_apply_redactions_into_the_session.md` and the reply beside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Destination {
    /// ★★★ **The open document, with nothing written — the default since
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
    /// ★★★ **Its price was inverted on 2026-09-05, and the old price is worth
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
    /// # ★★★ Why this exists, and it is an operator report
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
    /// ★★ **The deferred destination is not a mistake and is not being
    /// replaced.** It is cheaper, it is safer, and he asked for it. What was
    /// missing is the other half: a way to apply the removal and SEE it. His
    /// own ruling on the price is on the record — *"finalizing the document and
    /// can't be undone is ok for now"* — so the cost is one he has already
    /// accepted, stated at the control rather than discovered.
    ///
    /// ★ It writes **no file**. That is what distinguishes it from
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

/// ★★★ **The destination a freshly-opened dialog starts on.**
///
/// A named constant rather than a literal inside [`RedactDialog::open`], so the
/// property that actually matters — *the default writes nothing* — can be
/// asserted without constructing a document, and so that changing it is a
/// visible edit rather than one word in a struct literal.
///
/// It moved on 2026-09-04 from [`Destination::NewFile`] to
/// [`Destination::OpenDocument`]. Both are safe defaults and for different
/// reasons: the old one never *overwrote*, the new one never *writes*.
pub(super) const DEFAULT_DESTINATION: Destination = Destination::OpenDocument;

impl Destination {
    /// Whether this destination writes a file **now**, rather than leaving the
    /// write to a later Save.
    ///
    /// A method rather than three `== ` comparisons scattered through
    /// [`RedactDialog`], because five separate places ask the same question —
    /// which permanence sentence, which button label, which acknowledgements
    /// are owed, whether the picker opens, and whether an `Action` is pushed —
    /// and a fourth destination added later must be answered once rather than
    /// found five times.
    pub(super) const fn writes_now(self) -> bool {
        matches!(self, Self::NewFile | Self::ReplaceOriginal)
    }

    /// Whether confirming **stages** the removal for the next save rather than
    /// performing it.
    ///
    /// ★★★ Added 2026-09-08 with [`Self::OpenDocumentNow`], and it is the
    /// reason that variant needed more than a radio row. The confirm handler
    /// branched on `!writes_now()` — *"anything that does not write a file is
    /// staged"* — which was true while `OpenDocument` was the only
    /// non-writing destination and became **silently wrong** the moment a
    /// second one existed: `OpenDocumentNow` writes no file either, so it would
    /// have taken the staging path and done exactly the nothing the operator
    /// reported.
    ///
    /// ⇒ Named for what it MEANS rather than derived from what it is not. A
    /// predicate written as the complement of another is a predicate that
    /// changes meaning when a variant is added, without a compile error and
    /// without a word.
    pub(super) const fn stages(self) -> bool {
        matches!(self, Self::OpenDocument)
    }
}
