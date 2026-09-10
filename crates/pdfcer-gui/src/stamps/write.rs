//! # `stamps::write` — turning a [`Plan`] into a file Acrobat will load
//!
//! Four engine calls in a fixed order, and the order is the whole content of
//! this module. Everything else here is disclosure.
//!
//! ```text
//! pageops::extract(view, plan.pages_to_extract())  →  bytes of a NEW document
//! Document::from_bytes(bytes)                      →  reopen those bytes
//! settings.open_session(doc)                        →  the SESSION funnel
//!   .set_info_field(InfoField::Title, category)    →  the CATEGORY
//! stamp_file::name_stamp_pages(&mut session, …)    →  the /Names → /Pages tree
//! session.to_full_bytes(options)                   →  the file
//! ```
//!
//! ## ★★ Why it extracts instead of editing the open document
//!
//! Because the open document must survive the operation completely untouched.
//! `Save as stamp collection…` is a **Read-mode-legal act** by the operator's
//! own standing rule — *Read may produce a new document; it may not modify
//! this one* — and the cheapest way to honour that is never to have a mutable
//! handle on his document at all. `pageops::extract` reads a `DocumentView`
//! and returns bytes; nothing upstream of it can be changed by anything
//! downstream of it.
//!
//! ★ The view is the **session's**, not the loaded file's, which carries his
//! unsaved edits into the collection. `app::actions::extract` makes the same
//! choice for the same reason (decision 018), and the alternative — silently
//! writing the file as it was opened — is the kind of wrong answer that looks
//! completely right.
//!
//! ## ★★ Why it reopens the bytes rather than reusing a session
//!
//! `name_stamp_pages` names `stamps[i]` to **page `i` of the session it is
//! given**. Handing it the operator's session would name his drawing's pages.
//! The extracted bytes are the only document whose page numbering matches the
//! plan, so they have to become a document before they can be named. The
//! reopen costs one parse of a file we just built and buys the positional
//! contract for free.
//!
//! ⚠ It also means **a failure to reparse our own output is reachable**, and
//! is reported as its own outcome rather than folded into "extraction failed".
//! If pdfcer ever writes bytes pdfcer cannot read, that is a finding about the
//! engine and the operator should not see it described as something his file
//! did.
//!
//! ## What is disclosed, and why each one
//!
//! Under R8b rule 4 every inference this path makes is reported **off-canvas**
//! — here, in the dialog that is about to write the file, before it writes.
//! [`Written`] carries them; the dialog turns them into sentences. Nothing in
//! this module marks a page, because nothing in this module has a page to
//! mark.

use std::path::Path;

use pdfcer_core::document::Document;
use pdfcer_core::edit::InfoField;
use pdfcer_core::settings::Settings;
use pdfcer_core::stamp_file;

use super::Plan;
use crate::app::settings::SettingsExt;

/// Everything that happened on the way to the bytes.
///
/// Returned rather than logged because the operator is entitled to all of it
/// **before** the file lands somewhere Acrobat will read it — a stamp
/// collection that silently dropped one stamp is a picker with a hole in it,
/// discovered weeks later in the middle of signing something.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Written {
    /// The file's bytes, ready for `std::fs::write`.
    pub bytes: Vec<u8>,
    /// How many stamps the name tree ended up holding.
    ///
    /// Compared against [`Plan::included`] by the caller: a shortfall is the
    /// engine's `skipped` list and is disclosed, never rounded off.
    pub stamps_named: usize,
    /// Stamps the engine refused to name, in its own `internal=display` form.
    ///
    /// Should be empty by construction — [`Plan::for_engine`] and
    /// [`Plan::pages_to_extract`] are the same filter — so a non-empty list
    /// here is a **defect in this module**, not an operator error, and the
    /// disclosure says so rather than blaming his document.
    pub skipped: Vec<String>,
    /// How many pages the extraction actually carried.
    ///
    /// ★ Kept separately from `stamps_named` because they answer different
    /// questions and a build where they disagree is precisely the build that
    /// names the wrong artwork. The caller asserts they match; the trace
    /// prints both so a driven check can see the disagreement rather than
    /// infer it.
    pub pages_carried: usize,
}

/// Why a collection could not be written.
///
/// Four variants because there are four genuinely different sentences to say,
/// and collapsing them would hand the operator the shrug this project's text
/// conventions forbid.
#[derive(Debug, Clone)]
pub enum WriteFailure {
    /// `pageops::extract` refused. His document's pages could not be carried.
    Extract(String),
    /// pdfcer could not reparse the bytes it had just written.
    ///
    /// ⚠ Not an operator problem. See the module header.
    Reopen(String),
    /// The category could not be written to `/Info` `/Title`.
    ///
    /// Reachable on an encrypted source whose `/Info` is not a dictionary —
    /// `set_info_field` returns `NotADictionary` rather than overwriting
    /// whatever is really there, which is the correct refusal and worth
    /// repeating to the operator verbatim.
    Category(String),
    /// The name tree could not be written.
    Names(String),
    /// The final serialisation failed.
    Serialise(String),
}

impl WriteFailure {
    /// The engine's own words for what went wrong.
    ///
    /// ★ Returned rather than re-worded. `crate::text` owns the *frame* —
    /// which of the four things failed — and the engine owns the detail, for
    /// the same reason `app::save`'s refusals quote rather than paraphrase: a
    /// sentence this shell invents about a failure it did not diagnose is a
    /// sentence that will eventually be wrong.
    #[must_use]
    pub fn detail(&self) -> &str {
        match self {
            Self::Extract(d)
            | Self::Reopen(d)
            | Self::Category(d)
            | Self::Names(d)
            | Self::Serialise(d) => d,
        }
    }

    /// A stable token for the trace, so a driven check can assert *which*
    /// stage refused without matching on prose that is allowed to change.
    #[must_use]
    pub const fn token(&self) -> &'static str {
        match self {
            // ui-text-exempt: diagnostic tokens, never displayed in the UI.
            Self::Extract(_) => "extract",
            Self::Reopen(_) => "reopen",
            Self::Category(_) => "category",
            Self::Names(_) => "names",
            Self::Serialise(_) => "serialise",
        }
    }
}

/// Build the bytes of the stamp collection described by `plan`.
///
/// `view` is the source document — the operator's open session's view, so his
/// unsaved edits are carried.
///
/// # ★★ Why this takes `Settings` and not a `SaveOptions`
///
/// It takes **both** funnels, and it has to, because this function does two
/// things the operator has a persisted preference about: it opens an editing
/// session, and it serialises one. An earlier draft took a ready-made
/// `SaveOptions` — which honoured the write funnel and quietly bypassed the
/// session one, calling `EditSession::new` directly. `app::settings`' `syn`
/// check caught it on the first run, which is the whole reason
/// `EditSession::new` is on its forbidden list: **a guard shaped around one
/// delivery mechanism cannot see a second one**, and the second mechanism here
/// is `set_quad_point_order`, delivered by a setter rather than by a field.
///
/// ⚠ It is true that this particular session authors no annotation, so
/// `quad_point_order` changes nothing about the bytes it writes today. That is
/// an argument for an exemption and it is deliberately **not** taken: the
/// exemption would be a claim about what this module does now, re-checked by
/// nobody the day somebody adds a stamp-authoring verb here. Taking the funnel
/// costs one line and is true forever.
///
/// # Errors
///
/// [`WriteFailure`], one variant per stage. Nothing is written to disk by this
/// function — the caller owns the path, the picker and the syscall, which is
/// what makes this half unit-testable without a dialog.
pub fn build(
    view: &pdfcer_core::view::DocumentView<'_>,
    plan: &Plan,
    settings: &Settings,
) -> Result<Written, WriteFailure> {
    let pages = plan.pages_to_extract();

    // 1 — carry the wanted pages into a document of their own.
    let (extracted, report) = pdfcer_core::pageops::extract(view, &pages)
        .map_err(|e| WriteFailure::Extract(e.to_string()))?;

    // 2 — reopen them, because the names must be positional against THESE
    //     pages and no others. See the module header.
    let doc = Document::from_bytes(extracted).map_err(|e| WriteFailure::Reopen(e.to_string()))?;
    // ★ The session funnel, never `EditSession::new`. See the doc comment.
    let mut session = settings.open_session(doc);

    // 3 — the category. This is an ordinary /Info edit and the engine says so:
    //     `name_stamp_pages` deliberately owns the name tree only.
    session
        .set_info_field(InfoField::Title, Some(plan.category.trim()))
        .map_err(|e| WriteFailure::Category(e.to_string()))?;

    // 4 — the name tree. Sorting into §7.9.6's lexicographic order happens in
    //     here, in the engine, where the requirement is documented.
    let named = stamp_file::name_stamp_pages(&mut session, &plan.for_engine())
        .map_err(|e| WriteFailure::Names(e.to_string()))?;

    // 5 — a FULL rewrite, not an incremental one. The extracted bytes are a
    //     document nobody has ever seen; appending a revision to them would
    //     ship the operator a file with a history it does not have, and a
    //     stamp collection is one of the few files where small and plain is
    //     worth something — Acrobat reparses the whole folder on startup.
    // ★ And the save funnel for the serialisation, so a stamp collection and
    //   `Save a copy` never disagree about line endings in the xref table.
    let (bytes, _save) = session
        .to_full_bytes(&settings.save_options())
        .map_err(|e| WriteFailure::Serialise(e.to_string()))?;

    Ok(Written {
        bytes,
        stamps_named: named.stamps_named,
        skipped: named.skipped,
        pages_carried: report.pages,
    })
}

/// Build the collection and put it on disk, reporting on the trace.
///
/// Split from [`build`] on `app::actions::extract`'s reasoning: the half that
/// touches the filesystem and the half that does the work are separable in the
/// reading as well as in the testing, and only one of them needs a temporary
/// directory to exercise.
///
/// # Errors
///
/// [`WriteFailure`] from [`build`]. A failed `std::fs::write` is traced and
/// reported through the return value's `Ok(None)`-shaped absence — see below.
pub fn build_and_write(
    view: &pdfcer_core::view::DocumentView<'_>,
    plan: &Plan,
    settings: &Settings,
    target: &Path,
) -> Result<Written, WriteFailure> {
    let written = build(view, plan, settings).inspect_err(|failure| {
        let token = failure.token();
        let detail = failure.detail().to_owned();
        crate::diag::trace(move || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            format!("stamp-collection-failed stage={token} detail={detail}")
        });
    })?;

    match std::fs::write(target, &written.bytes) {
        Ok(()) => {
            // ★ `stamps=` beside `pages=` on HANDOFF.md §2's ink-trail advice.
            // A build that named the wrong pages writes a perfectly good PDF
            // of the right size with the right stamp count; these two fields
            // disagreeing is the only thing in the line that would show it.
            // `skipped=` is expected to be 0 forever, which is exactly why it
            // is printed — a number that is always zero is a tripwire.
            let (stamps, pages, skipped, bytes) = (
                written.stamps_named,
                written.pages_carried,
                written.skipped.len(),
                written.bytes.len(),
            );
            let path = target.to_owned();
            crate::diag::trace(move || {
                // ui-text-exempt: diagnostic trace, never displayed in the UI.
                format!(
                    "stamp-collection path={path:?} stamps={stamps} pages={pages} \
                     skipped={skipped} bytes={bytes}"
                )
            });
        }
        Err(error) => {
            let path = target.to_owned();
            let detail = error.to_string();
            crate::diag::trace(move || {
                // ui-text-exempt: diagnostic trace, never displayed in the UI.
                format!("stamp-collection-failed stage=disk path={path:?} detail={detail}")
            });
            return Err(WriteFailure::Serialise(error.to_string()));
        }
    }

    Ok(written)
}
