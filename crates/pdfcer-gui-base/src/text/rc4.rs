//! Every sentence about edits to a document kept under the old RC4
//! encryption: the command, the open-time note, the decline, the switch and
//! what a save reports.
//!
//! Each names the cost — a changed part is encrypted again with the key
//! stream its earlier version used — and the strong alternative, Encrypt….

use super::commands::CommandText;

/// `file.allow_rc4_edits`.
#[must_use]
pub const fn allow_edits() -> CommandText {
    CommandText::new(
        "Allow edits under RC4",
        "This file uses RC4, an old and weak encryption, and pdfcer changes nothing in it \
         until you allow it. Allowed, your edits are saved under the same password and key, \
         so every signature stays valid, but each part you change is encrypted again with the \
         key stream it had before, which makes that part easier to break. Encrypt… writes a \
         strongly protected copy instead. Lasts until the file is closed.",
    )
}

/// The status-row sentence when an RC4 file opens.
#[must_use]
pub const fn on_open() -> &'static str {
    "This file uses the old RC4 encryption, so pdfcer will not change it until you allow \
     it: File > Security > Allow edits under RC4."
}

/// The decline when an edit is refused for RC4 alone.
#[must_use]
pub const fn refused() -> &'static str {
    "Not changed: this file uses the old RC4 encryption. Allow edits under RC4 to change it \
     anyway, or use Encrypt… to make a strongly protected copy."
}

/// What the switch now does.
#[must_use]
pub const fn policy(allowed: bool) -> &'static str {
    if allowed {
        "Edits allowed under RC4 until this file is closed. Each save says how many changed \
         parts reuse their old key stream."
    } else {
        "Edits under RC4 are off again: pdfcer will not change this file."
    }
}

/// After a save under RC4: `n` changed parts were encrypted with the key
/// stream their earlier version used.
#[must_use]
pub fn reused(n: usize) -> String {
    match n {
        0 => "Saved under the file's old RC4 encryption; no changed part reuses its old key \
              stream."
            .to_owned(),
        1 => "Saved under the file's old RC4 encryption: 1 changed part is encrypted with the \
              same key stream as before, which weakens it."
            .to_owned(),
        n => format!(
            "Saved under the file's old RC4 encryption: {n} changed parts are encrypted with \
             the same key stream as before, which weakens them."
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_save_that_reused_a_key_stream_says_how_many() {
        assert!(reused(3).contains('3'));
        assert_ne!(reused(0), reused(1));
    }

    #[test]
    fn the_open_note_names_the_command_by_its_label() {
        assert!(on_open().contains(allow_edits().label));
    }
}
