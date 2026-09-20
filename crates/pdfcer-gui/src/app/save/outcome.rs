//! What a save produced, or why it produced nothing.
//!
//! Two closed vocabularies the save verbs in [`super`] match on: [`Written`]
//! names which writer ran and what it reported, [`SaveError`] names the refusal
//! when no file was made. Both are crate-private -- nothing outside this module
//! tree sees them -- and both exist so a caller branches on structured data
//! rather than on the text of a message.

use pdfcer_core::writer::{SaveReport, WriteError};
/// **Which writer produced the bytes that reached the file, and what it
/// reported.**
///
/// It exists for the staged-redaction route, and it is an enum rather
/// than `(SaveReport, Option<RedactionReport>)` because the two writers do not
/// both run: `EditSession::save_applying_redaction` produces no
/// [`SaveReport`] at all — it returns bytes and a
/// [`pdfcer_core::redact::RedactionReport`] — so a struct with both would have
/// to carry a fabricated one, and every field of a fabricated `SaveReport`
/// (`bytes_appended`, `byte_identical`, `promoted`) is a claim about a save
/// that did not happen in that shape.
///
/// ★ The trace lines differ for the same reason and that is the point. A reader
/// of a trace must be able to tell a save that appended a revision from one
/// that rewrote the whole document with content removed, and the two events
/// have no fields in common worth pretending they share.
#[derive(Debug)]
pub(super) enum Written {
    /// The ordinary §7.5.6 incremental update — §1, and everything this module
    /// promises about the previous revision staying intact.
    Ordinary(SaveReport),
    /// A staged redaction, performed — §1.1. A single-revision full rewrite
    /// with the marked content gone, proven absent from the bytes twice before
    /// they reached the disk.
    ///
    /// Boxed because `RedactionReport` is more than three times the size of
    /// `SaveReport`, and an un-boxed variant makes **every** save pay that size
    /// — the enum is as large as its largest arm, and the ordinary incremental
    /// save is the one this program performs constantly.
    RedactionApplied(Box<pdfcer_core::redact::RedactionReport>),
}

/// Why a save-a-copy produced no file.
///
/// Two variants rather than a `String`, on `crate::app::lifecycle`'s rule that
/// a branch is made on **structured error data, never by inspecting a message**
/// — and because the two are genuinely different facts about different
/// subsystems. Neither is worded to the operator separately today (the bar
/// carries one sentence for both; see §5), and keeping them apart is what makes
/// wording them separately a copy decision later rather than a re-plumbing.
#[derive(Debug)]
pub(super) enum SaveError {
    /// `pdfcer-core` could not build the update. A refusal by name from the
    /// writer — a broken provenance span, a cross-reference form that cannot
    /// express an entry it was handed.
    Serialize(WriteError),
    /// The bytes were built and the file system refused them: the folder is
    /// gone, the path is read-only, the volume is full.
    Write(std::io::Error),
    /// ★★★ **The bytes were built and pdfcer found redacted text in them.**
    ///
    /// The deferred redaction route's own guard. It means the save
    /// was refused *before any byte reached the file system*, and it is the one
    /// variant here that reports a **pdfcer defect** rather than a property of
    /// the document or of the disk: the engine's removal and pdfcer's own
    /// absence proof disagree about whether the content is gone.
    ///
    /// It is not expected to be reachable — see [`write_copy`]'s note on why
    /// the collapse makes every save mode safe by construction. It exists
    /// because a guarantee that depends on nobody ever changing the writer is
    /// not a guarantee.
    RedactionLeak {
        /// The strings that survived, for the trace. Never rendered verbatim:
        /// they are the redacted content, and putting them on screen to
        /// announce that they leaked would leak them again.
        survivors: Vec<String>,
    },
    /// ★★★ **A redaction is staged and the removal itself was refused, so no
    /// save of any kind could be built.**
    ///
    /// It is the one variant here the operator can reach by
    /// doing something perfectly reasonable, and the sequence is worth naming
    /// because it is the trap the deferred route brings with it:
    ///
    /// 1. mark, then *Review & apply* ▸ *this document* — the removal is armed;
    /// 2. **undo the marks**, which works, and is the whole point of staging
    ///    rather than collapsing;
    /// 3. press `Ctrl+S`.
    ///
    /// The staging is still armed and there is nothing left to remove, so
    /// `save_applying_redaction` refuses with `NothingToApply` — and the
    /// ordinary save modes are still refused too, so the document cannot be
    /// saved at all until the staging is cancelled. That is a real corner and
    /// the sentence for it names the remedy by name rather than reporting a
    /// failure: `crate::text::redact::save_refused_message`.
    RedactionRefused {
        /// Which refusal, so the sentence can name the remedy rather than the
        /// mechanism.
        refusal: crate::redact::RedactApplyRefusal,
    },
    /// ★★★ **The bytes were built and their page tree does not agree with
    /// itself, so writing them would hand the operator a damaged file.**
    ///
    /// From his own report: *"I tested deleting pages from a
    /// pdf. when I open the document in Acrobat there are blank pages at the
    /// end of the document equalling the number of pages I deleted."*
    ///
    /// Like [`Self::RedactionLeak`] this reports a **pdfcer defect** rather
    /// than a property of the document or of the disk — and unlike it, this one
    /// is reachable today and he reached it. `pdfcer-core` v0.38.0's
    /// `delete_pages` decrements `/Count` on the removed page's immediate
    /// parent and on no ancestor above it (measured; the appended revision
    /// defines exactly one object), so on any document whose page tree has more
    /// than one level the root goes on declaring the pre-delete page count.
    /// `page-copy --cut` produces byte-for-byte the same corruption. Filed as
    /// `request_delete_pages_leaves_ancestor_count_stale_on_a_nested_page_tree.md`.
    ///
    /// ★ The whole [`crate::pagetree::Audit`] is carried, not a pair of
    /// numbers, because the sentence differs by *which* node disagrees (see
    /// [`crate::text::pagetree`]) and because the trace wants the node ids —
    /// and because `crate::app::lifecycle`'s rule is that a branch is made on
    /// structured error data, never by inspecting a message.
    PageTreeStale {
        /// What the walk found. Never rendered verbatim: it names object ids.
        audit: crate::pagetree::Audit,
    },
}

impl From<WriteError> for SaveError {
    fn from(error: WriteError) -> Self {
        Self::Serialize(error)
    }
}

impl From<std::io::Error> for SaveError {
    fn from(error: std::io::Error) -> Self {
        Self::Write(error)
    }
}

impl std::fmt::Display for SaveError {
    /// Diagnostic prose for the trace, and for nothing else.
    ///
    /// `check-ui-strings.sh`'s exclusion 3 permits a `Display` impl to carry
    /// text that is not in the catalog **because it is diagnostic**, and states
    /// in the same breath that this "is not permission to route UI text through
    /// an error type". Nothing here reaches an operator: the bar's sentence is
    /// `crate::text::status::save_copy_failed`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialize(e) => write!(f, "the engine could not build the update: {e}"),
            Self::Write(e) => write!(f, "the file could not be written: {e}"),
            // The COUNT, never the strings. They are the redacted content, and
            // a diagnostic that announced a leak by printing the leaked text
            // into a log file would be the same failure at one remove.
            Self::RedactionLeak { survivors } => write!(
                f,
                "the save was refused: {} redacted string(s) survived in the bytes about to be \
                 written",
                survivors.len()
            ),
            // `{:?}`, deliberately: `RedactApplyRefusal` has no `Display` on
            // purpose, because it is rendered to the operator by
            // `crate::text::redact::refusal_message` and a second,
            // uncatalogued rendering is how the two drift. See
            // `crate::app::actions::redact`, which makes the same choice at the
            // other call site for the same reason.
            Self::RedactionRefused { refusal } => write!(
                f,
                "the staged redaction could not be performed, so no save was built: {refusal:?}"
            ),
            // The numbers and the node count, never the operator's sentence:
            // `crate::text::pagetree` owns that and a second, uncatalogued
            // rendering is how the two drift — the same choice
            // `RedactionRefused` makes one arm above.
            Self::PageTreeStale { audit } => write!(
                f,
                "the save was refused: the page tree declares {:?} pages over {} reachable, \
                 and {} node(s) disagree",
                audit.declared_pages,
                audit.reachable_pages,
                audit.disagreements.len()
            ),
        }
    }
}
