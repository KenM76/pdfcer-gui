//! # `text::compact` — what the Save-a-compacted-copy window says before it
//! throws anything away
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/compact.md`.

/// The window's title bar.
#[must_use]
pub const fn window_title() -> &'static str {
    "Save a compacted copy"
}

/// The opening sentence.
#[must_use]
pub const fn intro() -> &'static str {
    "pdfcer normally saves by adding your changes to the end of the file, which keeps the earlier \
     version of your drawing inside it. This writes the whole file fresh instead, so anything no \
     longer used is dropped."
}

/// How much smaller the file is expected to be.
#[must_use]
pub fn size_change(before: u64, after: u64) -> String {
    if after >= before {
        // `>=`, not `>`. A rewrite can legitimately come out very slightly
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
#[must_use]
pub const fn revisions_line() -> &'static str {
    "The earlier version of the drawing that pdfcer keeps inside the file is dropped. Your own \
     Undo is not affected, and neither is the original file — this always writes a new one."
}

/// The document is signed and the copy will not be.
#[must_use]
pub fn signature_line(count: usize) -> String {
    format!(
        "This document carries {count} digital signature(s). A compacted copy CANNOT keep them — \
         rewriting the file moves every byte they cover, and nothing later can repair that. Your \
         original file keeps its signatures."
    )
}

/// The button that writes it.
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
/// It repeats the size because that is the outcome, and names the **file**
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
#[must_use]
pub fn refused(detail: &str) -> String {
    format!("pdfcer cannot rewrite this file: {detail}. Save a copy the ordinary way instead.")
}

/// The operating system refused the write.
#[must_use]
pub fn write_failed(detail: &str) -> String {
    format!("pdfcer could not write the compacted copy: {detail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A file with nothing to reclaim is told so, and is not told a
    /// number.**
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

    /// **The signature sentence says the loss cannot be repaired.**
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
