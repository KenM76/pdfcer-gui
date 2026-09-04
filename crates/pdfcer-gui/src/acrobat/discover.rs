//! # `acrobat::discover` — turning what the registry says into a path, and
//! deciding whether the path is an Acrobat at all
//!
//! Three small functions, none of which touches a registry or a disk. They are
//! separate from [`super::resolve`] because they are the part that is *fiddly*
//! rather than the part that is *decisive*, and the fiddly part is where the
//! bugs live: a value read out of the registry is a string somebody's installer
//! wrote, and installers have written every shape of it.
//!
//! ## ★★ The three shapes, and where each one comes from
//!
//! | Source | Example value | What has to be undone |
//! |---|---|---|
//! | `App Paths` default | `C:\Program Files\Adobe\Acrobat DC\Acrobat\Acrobat.exe` | quoting, whitespace, a trailing NUL |
//! | `App Paths`, quoted | `"D:\Apps\Acrobat\Acrobat.exe"` | the quotes |
//! | `shell\open\command` | `"C:\…\Acrobat.exe" "%1"` | the quotes **and** the argument template |
//!
//! The third is the awkward one, and it has an unquoted spelling too —
//! `C:\Program Files\…\Acrobat.exe %1` — which cannot be split on whitespace
//! because the path contains some. See [`executable_from_command`].
//!
//! ## ★★★ Why [`edition_of`] is a filter and not just a label
//!
//! It answers *"is this an Acrobat?"*, and the answer is `None` far more often
//! than it looks like it should be. `super`'s §4 records the measurement that
//! made this a filter: on the operator's own machine, `HKLM\SOFTWARE\Classes\.pdf`
//! reads **`OpenPDFStudio.pdf`**. The registered PDF handler here is not Adobe's
//! product at all.
//!
//! A fallback that trusted the handler would therefore have put a button
//! labelled *Open in Acrobat* over a launcher for a competitor's editor — and
//! the operator would find out after pdfcer had already closed their document,
//! which is the worst possible moment to discover that a button meant something
//! else. So the handler is consulted for a *path* and then asked to prove it is
//! an Acrobat by its file name, which is the only evidence available without
//! reading version resources out of the binary.

use std::path::{Path, PathBuf};

use super::Edition;

/// Which Acrobat an executable path names, or `None` if it names something
/// else.
///
/// Matched on the **file name only**, case-insensitively, because the
/// directory is exactly the part that varies: `C:\Program Files\Adobe\…`,
/// `D:\Apps\…`, a network install, a per-user install under `AppData`. The
/// file name is fixed by Adobe and is what `App Paths` is keyed on.
///
/// ★ A path with no file name — `C:\`, or an empty string — is `None` rather
/// than a panic. This runs over values a person may have typed.
#[must_use]
pub fn edition_of(path: &Path) -> Option<Edition> {
    let name = path.file_name()?.to_str()?;
    [Edition::Pro, Edition::Reader]
        .into_iter()
        .find(|edition| name.eq_ignore_ascii_case(edition.executable()))
}

/// Clean an `App Paths` default value into a path.
///
/// The value is *supposed* to be a bare path, and usually is. It is
/// nonetheless unquoted and trimmed here, because:
///
/// - some installers quote it anyway, and a `PathBuf` built from
///   `"C:\…\Acrobat.exe"` **with the quote characters in it** is a path that
///   will never exist, so the button would silently never appear;
/// - `reg query` output carries whatever trailing whitespace the value had,
///   and a `REG_SZ` written by a careless installer can carry an embedded NUL
///   that survives into the string.
///
/// Returns `None` for a value that is empty once cleaned, because an empty
/// path is not a location and `Path::new("").exists()` is `false` on every
/// platform — a distinction worth making here rather than discovering as a
/// mysterious absence three functions away.
#[must_use]
pub fn executable_from_registration(raw: &str) -> Option<PathBuf> {
    let cleaned = clean(raw);
    (!cleaned.is_empty()).then(|| PathBuf::from(cleaned))
}

/// Pull the executable out of a registered `shell\open\command`.
///
/// # ★★ Why this is not `raw.split_whitespace().next()`
///
/// Because the overwhelmingly common installation directory is
/// `C:\Program Files\…`, which contains a space. Splitting on whitespace
/// yields `C:\Program`, which exists on no machine, and the failure presents
/// as *"discovery does not work"* rather than as a parsing bug.
///
/// Two shapes are handled, in this order:
///
/// 1. **Quoted** — `"C:\…\Acrobat.exe" "%1"`. Everything between the first
///    pair of double quotes is the path. This is what every modern installer
///    writes and what Windows itself requires for a path with a space.
/// 2. **Unquoted** — `C:\…\Acrobat.exe %1`. Split immediately after the first
///    case-insensitive `.exe`, which is the only reliable boundary available:
///    the extension is the last thing before the arguments begin, and a
///    directory named `…exe…` does not end a component with `.exe`.
///
/// Anything else — a command with no `.exe` at all, an empty string, a bare
/// argument template — is `None`. A wrong guess here is worse than no answer,
/// because [`super::resolve`] would carry it forward to a `Viewer` and the
/// operator would press a button that starts nothing.
#[must_use]
pub fn executable_from_command(raw: &str) -> Option<PathBuf> {
    let raw = raw.trim().trim_matches('\0');

    if let Some(rest) = raw.strip_prefix('"') {
        let (quoted, _) = rest.split_once('"')?;
        let quoted = quoted.trim();
        return (!quoted.is_empty()).then(|| PathBuf::from(quoted));
    }

    // Unquoted. Find the end of the first `.exe`, whatever its case.
    let lower = raw.to_ascii_lowercase();
    let end = lower.find(".exe")? + ".exe".len();
    let path = raw[..end].trim();
    (!path.is_empty()).then(|| PathBuf::from(path))
}

/// Strip quotes, whitespace and stray NULs off a registry string.
fn clean(raw: &str) -> &str {
    raw.trim()
        .trim_matches('\0')
        .trim()
        .trim_matches('"')
        .trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **★★★ The `.pdf` handler is filtered by file name, and this is the
    /// case that proves the filter earns its keep.**
    ///
    /// Measured on the operator's own machine, 2026-09-04:
    /// `HKLM\SOFTWARE\Classes\.pdf` reads `OpenPDFStudio.pdf`. A fallback that
    /// trusted the registered handler would launch a competitor's editor from
    /// a button that says *Open in Acrobat*.
    #[test]
    fn only_an_acrobat_executable_is_recognised_as_one() {
        assert_eq!(
            edition_of(Path::new(
                r"C:\Program Files\Adobe\Acrobat DC\Acrobat\Acrobat.exe"
            )),
            Some(Edition::Pro)
        );
        assert_eq!(
            edition_of(Path::new(
                r"C:\Program Files (x86)\Adobe\Reader\AcroRd32.exe"
            )),
            Some(Edition::Reader)
        );
        // Case is not a distinction Windows makes about a file name, and an
        // installer that wrote `ACROBAT.EXE` has still installed Acrobat.
        assert_eq!(
            edition_of(Path::new(r"D:\Apps\ACROBAT.EXE")),
            Some(Edition::Pro)
        );

        // The one that matters.
        assert_eq!(
            edition_of(Path::new(r"C:\Program Files\PDF Studio\PDFStudio.exe")),
            None,
            "a different vendor's PDF editor must not answer to `Open in Acrobat`"
        );
        assert_eq!(edition_of(Path::new(r"C:\Windows\notepad.exe")), None);
        assert_eq!(edition_of(Path::new("")), None, "no file name, no answer");
        assert_eq!(edition_of(Path::new(r"C:\")), None);
    }

    /// **★ A `Program Files` path survives parsing.**
    ///
    /// The single most likely place Acrobat is installed contains a space, so
    /// the naive `split_whitespace().next()` yields `C:\Program`. That failure
    /// presents as "discovery does not work on any normal machine", which is
    /// exactly the kind of thing a test is for.
    #[test]
    fn a_command_line_yields_the_executable_and_not_its_first_word() {
        assert_eq!(
            executable_from_command(
                r#""C:\Program Files\Adobe\Acrobat DC\Acrobat\Acrobat.exe" "%1""#
            ),
            Some(PathBuf::from(
                r"C:\Program Files\Adobe\Acrobat DC\Acrobat\Acrobat.exe"
            )),
            "the quoted form, which is what this machine actually holds"
        );
        assert_eq!(
            executable_from_command(r"C:\Program Files\Adobe\Acrobat.exe %1"),
            Some(PathBuf::from(r"C:\Program Files\Adobe\Acrobat.exe")),
            "the unquoted form: split after `.exe`, never on whitespace"
        );
        assert_eq!(
            executable_from_command(r"D:\apps\ACRORD32.EXE /A page=1 %1"),
            Some(PathBuf::from(r"D:\apps\ACRORD32.EXE")),
            "`.exe` is matched case-insensitively"
        );

        // Nothing usable is `None`, never a guess.
        assert_eq!(executable_from_command(""), None);
        assert_eq!(executable_from_command("   "), None);
        assert_eq!(executable_from_command("%1"), None, "no executable named");
        assert_eq!(
            executable_from_command(r#""" "%1""#),
            None,
            "an empty quoted path is not a location"
        );
    }

    /// An `App Paths` value is cleaned of the three things an installer may
    /// have left on it, and an empty one is refused.
    #[test]
    fn a_registration_value_is_unquoted_trimmed_and_denulled() {
        let want = PathBuf::from(r"C:\Program Files\Adobe\Acrobat DC\Acrobat\Acrobat.exe");
        assert_eq!(
            executable_from_registration(r"C:\Program Files\Adobe\Acrobat DC\Acrobat\Acrobat.exe"),
            Some(want.clone())
        );
        assert_eq!(
            executable_from_registration(
                "  \"C:\\Program Files\\Adobe\\Acrobat DC\\Acrobat\\Acrobat.exe\"  \0"
            ),
            Some(want),
            "quotes, surrounding whitespace and a trailing NUL all come off"
        );
        assert_eq!(executable_from_registration(""), None);
        assert_eq!(executable_from_registration("  \0  "), None);
        assert_eq!(
            executable_from_registration("\"\""),
            None,
            "a value that is only quotes names nothing"
        );
    }
}
