//! # `text::compact` — what the Save-a-compacted-copy window says before it
//! throws anything away
//!
//! The copy for [`crate::dialogs::compact`].
//!
//! ## ★★★ Why this command exists, and it is the operator's own request
//!
//! `OPERATOR_REQUESTS.md` **O48**, answered *"yes to all three"* on 2026-08-28.
//! It was raised by this project rather than by him, from a limit found while
//! wiring Remove-embedded-fonts:
//!
//! > **Removing fonts does not make the file smaller.** pdfcer saves by adding
//! > your changes to the end of the file and leaving the earlier version
//! > intact, so the outlines stop being used and are still there.
//!
//! §7.5.6's update section is *appended*. Deleted objects get free
//! cross-reference entries in a **new** section and their bytes remain in the
//! prior revision, which is still in the file. So every space-reclaiming
//! operation pdfcer has — unembedding a font, deleting a page, deleting an image
//! — produces a file that is very slightly **larger**.
//!
//! Only a full rewrite drops the bytes, and until now this shell had no way to
//! ask for one.
//!
//! ## ★★★ Why it is a SEPARATE command and not a better Save
//!
//! Because incremental saving is not a limitation pdfcer is working around — it
//! is a promise it makes, on an operator-visible surface, and has since the day
//! Save-a-copy was registered:
//!
//! > *"…the edits are appended as an update so the previous version stays intact
//! > inside the file."*
//!
//! That promise buys two things a full rewrite destroys: **the earlier revision
//! is recoverable from inside the file**, and **every digital signature stays
//! valid** (§12.8.1 — a full rewrite invalidates them all). Neither is small,
//! and neither should be traded away by default because somebody wanted a
//! smaller file once.
//!
//! ⇒ So the whole design is: a second command, named so it cannot be pressed by
//! accident, **always to a new file**, with what it discards said plainly
//! before the picker opens.
//!
//! ## ★★ Every sentence here is about a LOSS, and that is deliberate
//!
//! The gain — a smaller file — is why the operator pressed it and needs no
//! advocacy. What they cannot see is what is being dropped: a revision history
//! they may never have known was there, and signatures whose invalidation shows
//! up in somebody else's reader rather than in this one. Rule 4's surviving
//! half, applied to a save.

/// The window's title bar.
#[must_use]
pub const fn window_title() -> &'static str {
    "Save a compacted copy"
}

/// The opening sentence.
///
/// ★★ It leads with the **mechanism**, not the benefit, because the mechanism is
/// the part that explains every consequence below it. An operator who
/// understands *"pdfcer normally adds to the end of the file"* can predict what a
/// rewrite costs; one told only *"this makes your file smaller"* cannot.
#[must_use]
pub const fn intro() -> &'static str {
    "pdfcer normally saves by adding your changes to the end of the file, which keeps the earlier \
     version of your drawing inside it. This writes the whole file fresh instead, so anything no \
     longer used is dropped."
}

/// How much smaller the file is expected to be.
///
/// ★★★ A **measurement of this document**, taken by writing it — not an
/// estimate. The window computes the compacted bytes before it opens, because a
/// prediction that turned out wrong on the operator's own file would be worse
/// than saying nothing: they would have accepted the losses below for a saving
/// that did not arrive.
///
/// ★★ The **no-saving** case is a real outcome and gets its own sentence. A file
/// that has never been edited, or one whose deletions were all in the current
/// revision anyway, has nothing to reclaim — and *"this will save 0 KB"* reads
/// as a failure when it is an accurate answer about a tidy file.
#[must_use]
pub fn size_change(before: u64, after: u64) -> String {
    if after >= before {
        // ★ `>=`, not `>`. A rewrite can legitimately come out very slightly
        // LARGER — §7.5.4 requires a single-section cross-reference table with
        // one entry per object number from zero, so a file whose objects are
        // sparsely numbered pays for the gaps. Saying "no smaller" covers both
        // and does not invite the question a byte count would.
        return "This file has nothing unused in it, so a compacted copy would be no smaller. \
                The copy is still written if you want one."
            .to_owned();
    }
    let saved = before - after;
    format!(
        "This file is {}. A compacted copy would be {} — about {} smaller.",
        bytes(before),
        bytes(after),
        bytes(saved)
    )
}

/// A byte count in the units an operator thinks in.
fn bytes(n: u64) -> String {
    let mib = n as f64 / (1024.0 * 1024.0);
    if mib >= 0.1 {
        format!("{mib:.1} MB")
    } else {
        format!("{:.0} KB", n as f64 / 1024.0)
    }
}

/// The previous revision is discarded.
///
/// ★★ Said to everybody, because everybody loses it and almost nobody knows it
/// was there. An incremental save leaves the file's earlier state recoverable
/// from inside the file itself; a rewrite is the moment that stops being true,
/// and it is the only moment at which saying so is any use.
#[must_use]
pub const fn revisions_line() -> &'static str {
    "The earlier version of the drawing that pdfcer keeps inside the file is dropped. Your own \
     Undo is not affected, and neither is the original file — this always writes a new one."
}

/// The document is signed and the copy will not be.
///
/// ★★★ The loudest sentence in this window, and the only one that is
/// **conditional**, for `text::unembed`'s reason: a warning about signatures on
/// every unsigned drawing is noise that teaches an operator to skip the window.
///
/// ★★ It says *"cannot be repaired"*, which is the part that distinguishes this
/// from every other loss pdfcer discloses. A signature covers a byte range
/// (§12.8.1); rewriting the file moves everything, and no later save puts it
/// back. The operator is being asked to accept something irreversible and is
/// entitled to be told that in the word that means it.
#[must_use]
pub fn signature_line(count: usize) -> String {
    format!(
        "This document carries {count} digital signature(s). A compacted copy CANNOT keep them — \
         rewriting the file moves every byte they cover, and nothing later can repair that. Your \
         original file keeps its signatures."
    )
}

/// The button that writes it.
///
/// ★ It names the **act**, not "OK". The operator has just read three sentences
/// about losses, and a button reading `OK` after that asks them to agree to a
/// question rather than to perform a thing they chose.
#[must_use]
pub const fn save_button() -> &'static str {
    "Choose where to save…"
}

/// The button that does not.
#[must_use]
pub const fn cancel_button() -> &'static str {
    "Cancel"
}

/// The disclosure after a compacted copy is written.
///
/// ★ It repeats the size because that is the outcome, and names the **file**
/// because a copy an operator cannot find is a copy they will make twice.
#[must_use]
pub fn written(path: &str, before: u64, after: u64) -> String {
    if after >= before {
        return format!("Wrote a compacted copy to {path}. It is no smaller — nothing was unused.");
    }
    format!(
        "Wrote a compacted copy to {path} — {} instead of {}.",
        bytes(after),
        bytes(before)
    )
}

/// The engine refused to rewrite this file.
///
/// ★★ A real refusal with a named cause, not a fallback. `pdfcer-core` refuses a
/// full rewrite of a **hybrid-reference** file by name and points at incremental
/// as the supported path — and `app::save`'s header states the rule this obeys:
/// *"if a future change finds incremental genuinely impossible for some input,
/// the honest response is to refuse and say so, not to fall back."* This is that
/// rule read in the other direction, and quietly writing an incremental copy
/// here would give the operator a file that is not what the command promised.
#[must_use]
pub fn refused(detail: &str) -> String {
    format!("pdfcer cannot rewrite this file: {detail}. Save a copy the ordinary way instead.")
}

/// The operating system refused the write.
///
/// ★ The reason is passed through verbatim, on `export_dxf::export_failed`'s
/// stated rule: *"access is denied"* and *"the device is not ready"* are
/// different problems with different remedies, and a shell that collapsed them
/// to *"could not save"* would leave the operator with the one fact they cannot
/// derive.
#[must_use]
pub fn write_failed(detail: &str) -> String {
    format!("pdfcer could not write the compacted copy: {detail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **A file with nothing to reclaim is told so, and is not told a
    /// number.**
    ///
    /// The failure this guards is *"this will save 0 KB"*, which is accurate and
    /// reads as the feature failing. It is also the case a rewrite can come out
    /// LARGER — §7.5.4's table pays for gaps in object numbering — so a sentence
    /// built from a subtraction would underflow or print a negative saving.
    #[test]
    fn a_tidy_file_is_told_it_is_tidy() {
        for (before, after) in [(1000_u64, 1000_u64), (1000, 1200)] {
            let line = size_change(before, after);
            assert!(line.contains("no smaller"), "{line}");
            assert!(!line.contains('0'), "it quoted a number: {line}");
        }
        let saving = size_change(4 * 1024 * 1024, 1024 * 1024);
        assert!(saving.contains("smaller"), "{saving}");
        assert!(saving.contains("3.0 MB"), "the saving is named: {saving}");
    }

    /// ★★ **The signature sentence says the loss cannot be repaired.**
    ///
    /// Every other disclosure in this shell describes something an operator can
    /// undo or redo. This one cannot, and the word that says so is the whole
    /// difference between a warning and a note.
    #[test]
    fn the_signature_warning_says_it_is_irreversible() {
        let line = signature_line(2);
        assert!(line.contains("CANNOT keep"), "{line}");
        assert!(line.contains("repair"), "{line}");
        assert!(line.contains("original file keeps"), "{line}");
    }

    /// **The written disclosure names the file and the outcome.**
    #[test]
    fn the_result_names_where_it_went() {
        let line = written("C:/out/plan.pdf", 4 * 1024 * 1024, 1024 * 1024);
        assert!(line.contains("C:/out/plan.pdf"), "{line}");
        assert!(line.contains("1.0 MB"), "{line}");
        assert!(written("C:/out/p.pdf", 100, 100).contains("no smaller"));
    }
}
