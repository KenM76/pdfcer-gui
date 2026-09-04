//! # `acrobat::windows` — the two things only a real machine can answer
//!
//! Everything impure about O122 is in this file, and it is deliberately the
//! **least interesting** file of the three: it reads registry values, it
//! spawns a process, and it makes no decisions at all. Every decision —
//! which candidate wins, whether Pro beats Reader, which dialog the operator
//! sees — is in [`super`] and [`super::discover`], over values, where a test
//! can reach it.
//!
//! See [`super`]'s §3 for the seam and §6 for why the registry is read by
//! `reg.exe` rather than by a crate.
//!
//! ## ★★★ `CREATE_NO_WINDOW`, and the flash it prevents
//!
//! `reg.exe` is a console program. A GUI process on Windows that spawns one
//! **gets a console window created for it**, on top of everything, for as long
//! as the child runs. Discovery runs when the shell starts and again whenever
//! the operator changes the setting, so without this flag pdfcer would blink a
//! black rectangle over the document at exactly the moments the operator is
//! looking at it — three times in a row, since three roots are consulted.
//!
//! It is the kind of defect that never appears in a test, never appears in a
//! trace, and is reported as *"something flashes when I open a file"*.
//!
//! ## ★★ Why every read is `reg query … /ve` and never `/s`
//!
//! `/ve` asks for one key's **default value** and nothing else. A recursive
//! or wildcard query would return a tree whose size is not under pdfcer's
//! control, on the start-up path, parsed by a function that would then have to
//! decide which of several values it meant. One key, one value, one line to
//! parse.
//!
//! ## The output format, and the one thing that is fragile about parsing it
//!
//! ```text
//! HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\Acrobat.exe
//!     (Default)    REG_SZ    C:\Program Files\Adobe\Acrobat DC\Acrobat\Acrobat.exe
//! ```
//!
//! Captured from this machine on 2026-09-04. The value is whatever follows
//! the **type token**, which is why [`value_from_reg_output`] splits on
//! `REG_SZ` / `REG_EXPAND_SZ` rather than on column positions or on runs of
//! spaces: the separator is documented as whitespace of unspecified width, and
//! the *value itself* routinely contains spaces, so any split that counted
//! them would truncate `C:\Program Files\…` at the first one.
//!
//! ⚠ It is fragile in one specific way and that is worth stating rather than
//! discovering: a value whose **content** contains the literal text `REG_SZ`
//! would be split in the wrong place. That cannot occur for these keys — they
//! hold file paths written by Adobe's installer — and the alternative
//! (`reg query /f`, or reading the raw registry) costs more than the risk.
//!
//! ## A missing key is not an error
//!
//! `reg.exe` exits non-zero and prints `ERROR: The system was unable to find
//! the specified registry key or value.` when a key is absent. That is the
//! **ordinary** answer for a machine with no Acrobat, so it is mapped to
//! `None` and nothing is traced as a failure. Verified here: querying
//! `App Paths\AcroRd32.exe` on this machine exits 1, and this machine simply
//! has no Reader.

use std::path::Path;
use std::process::Command;

use super::{Launcher, Registrations, Viewer};

/// The three registry roots that may carry an `App Paths` registration, in
/// the order Windows itself resolves them.
///
/// `HKCU` last rather than first, which is the one non-obvious entry: a
/// per-user registration is the least common by far, and the two `HKLM`
/// spellings are what a machine-wide Adobe installer writes. All three are
/// consulted because a per-user install is exactly the case a hard-coded
/// `C:\Program Files\…` would miss, and missing it is the failure §4 of
/// [`super`] exists to prevent.
///
/// The `WOW6432Node` mirror is separate rather than implied: a 64-bit process
/// reading `HKLM\SOFTWARE\…` does **not** see what a 32-bit installer wrote,
/// and Acrobat has shipped 32-bit for most of its life. `reg.exe` inherits the
/// bitness of the caller, so the mirror has to be named.
const APP_PATHS_ROOTS: [&str; 3] = [
    // ui-text-exempt: registry key paths, never displayed.
    r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths",
    // ui-text-exempt: registry key paths, never displayed.
    r"HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\App Paths",
    // ui-text-exempt: registry key paths, never displayed.
    r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths",
];

/// Where the operator's own choice of `.pdf` handler is recorded.
///
/// ★ Consulted **before** `HKLM\SOFTWARE\Classes\.pdf`, and the order is the
/// whole reason both are read. On this machine the machine-wide class
/// registration says `OpenPDFStudio.pdf` while `UserChoice` says
/// `Acrobat.Document.DC` — the operator picked Acrobat and Windows recorded it
/// here, which is the answer that reflects what they actually want.
// ui-text-exempt: a registry key path, never displayed.
const USER_CHOICE: &str =
    r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.pdf\UserChoice";

/// The machine-wide `.pdf` class registration.
// ui-text-exempt: a registry key path, never displayed.
const CLASSES_PDF: &str = r"HKLM\SOFTWARE\Classes\.pdf";

/// The root every ProgId's command lives under.
// ui-text-exempt: a registry key path, never displayed.
const CLASSES: &str = r"HKLM\SOFTWARE\Classes";

/// `CREATE_NO_WINDOW` — see this module's ★★★ header note.
///
/// Spelled as a literal rather than taken from a crate because pdfcer-gui
/// depends on no Windows crate and is not permitted to gain one. The value is
/// fixed by the Win32 ABI (`processthreadsapi.h`) and has never changed.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// The real machine.
///
/// A unit struct with no state: every answer is read fresh, because the
/// operator can install Acrobat while pdfcer is running and a cached "no"
/// would outlive the fact it recorded.
#[derive(Debug, Clone, Copy, Default)]
pub struct Windows;

impl Registrations for Windows {
    fn app_path(&self, executable: &str) -> Option<String> {
        APP_PATHS_ROOTS
            .iter()
            .find_map(|root| default_value(&format!(r"{root}\{executable}")))
    }

    fn pdf_handler_command(&self) -> Option<String> {
        let prog_id = value(USER_CHOICE, "ProgId").or_else(|| default_value(CLASSES_PDF))?;
        let prog_id = prog_id.trim();
        if prog_id.is_empty() {
            return None;
        }
        // ui-text-exempt: a registry key path, never displayed.
        default_value(&format!(r"{CLASSES}\{prog_id}\shell\open\command"))
    }

    fn exists(&self, path: &Path) -> bool {
        path.is_file()
    }
}

impl Launcher for Windows {
    fn launch(&self, viewer: &Viewer, file: &Path) -> std::io::Result<()> {
        // ★ No `CREATE_NO_WINDOW` here, and that is not an oversight: Acrobat
        // is a GUI program and the flag would be meaningless. It is also not
        // harmless to apply blindly — the flag suppresses a console the child
        // asks for, and a program that wanted one and did not get one behaves
        // differently.
        //
        // ★★ `spawn`, never `output` or `status`. Both of those WAIT for the
        // child, and waiting for Acrobat means pdfcer's event loop stops
        // repainting until the operator closes it — the program appearing to
        // hang the instant it succeeds.
        Command::new(&viewer.path).arg(file).spawn().map(drop)
    }
}

/// Read a key's `(Default)` value.
fn default_value(key: &str) -> Option<String> {
    // ui-text-exempt: a `reg.exe` switch, never displayed.
    reg_query(key, &["/ve"])
}

/// Read a named value from a key.
fn value(key: &str, name: &str) -> Option<String> {
    // ui-text-exempt: a `reg.exe` switch, never displayed.
    reg_query(key, &["/v", name])
}

/// Run one `reg query` and return the value it printed, if it printed one.
///
/// Every failure is `None` and none of them is traced as a fault: `reg.exe`
/// missing, the key absent, the output unparseable and the value empty are all
/// the same fact from this module's point of view — *Windows does not register
/// that here* — and a start-up path that logged an error for the ordinary case
/// of "no Acrobat installed" would train a reader to ignore the log.
fn reg_query(key: &str, args: &[&str]) -> Option<String> {
    let mut command = Command::new("reg"); // ui-text-exempt: an executable name.
    // ui-text-exempt: a `reg.exe` sub-command, never displayed.
    command.arg("query").arg(key).args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    value_from_reg_output(&String::from_utf8_lossy(&output.stdout))
}

/// Pull the value out of `reg query` output. See this module's header for the
/// format and for the one way this is fragile.
///
/// Public to the crate rather than private so that its tests can be real: the
/// format is the thing most likely to be wrong, and it is the only part of
/// this file that can be tested without a registry.
#[must_use]
pub fn value_from_reg_output(text: &str) -> Option<String> {
    // ui-text-exempt: registry value TYPE tokens, matched literally in
    // `reg.exe` output. Never displayed.
    const TYPES: [&str; 2] = ["REG_EXPAND_SZ", "REG_SZ"];
    for line in text.lines() {
        for token in TYPES {
            if let Some((_, rest)) = line.split_once(token) {
                let value = rest.trim();
                if !value.is_empty() {
                    return Some(value.to_owned());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **★ The real output of the real command on the real machine, parsed.**
    ///
    /// Captured verbatim on 2026-09-04 from
    /// `reg query "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\Acrobat.exe" /ve`.
    /// Pinning the actual bytes rather than a tidied-up approximation is the
    /// point: a parser tested only against what its author imagined the output
    /// looks like is a parser tested against its own assumptions.
    #[test]
    fn the_real_reg_query_output_yields_the_real_path() {
        // ★ Assembled line by line rather than as one continued literal, and
        // the reason is mechanical: `check-string-gaps.sh`'s block-form
        // exemption arms exactly ONE following code line, and the padding
        // `reg.exe` prints is on the third of them. One line, one exemption.
        let captured = [
            "",
            r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\Acrobat.exe",
            // string-gap-exempt: this IS the column padding `reg.exe` prints,
            // and the whole value of the fixture is that it is byte-for-byte
            // what the real command produced on this machine. Rejoining the
            // spaces would test a format that does not exist.
            r"    (Default)    REG_SZ    C:\Program Files\Adobe\Acrobat DC\Acrobat\Acrobat.exe",
            "",
            "",
        ]
        .join("\r\n");
        assert_eq!(
            value_from_reg_output(&captured).as_deref(),
            Some(r"C:\Program Files\Adobe\Acrobat DC\Acrobat\Acrobat.exe"),
            "the value keeps the spaces in `Program Files` and in `Acrobat DC`"
        );
    }

    /// **★★ The separator is whitespace of unspecified width**, which is what
    /// this module's header claims and what the parser must actually honour.
    ///
    /// Added after a falsification: splitting on the type token followed by
    /// *exactly four spaces* passed every other test in this file, because
    /// four is what `reg.exe` printed on the machine the fixture came from. It
    /// is not a documented promise. A build of Windows, a locale, or a longer
    /// type name that padded differently would have produced a parser that
    /// silently found no value — and "no value" is indistinguishable from "no
    /// Acrobat installed", so the button would simply never appear and nothing
    /// would say why.
    #[test]
    fn the_separator_is_whitespace_of_any_width() {
        // string-gap-exempt: the runs of spaces ARE the subject of the test —
        // it exists to prove the parser does not care how wide the padding is.
        for pad in ["\t", " ", "  ", "          "] {
            let line = format!("    (Default){pad}REG_SZ{pad}D:\\Apps\\Acrobat.exe\r\n");
            assert_eq!(
                value_from_reg_output(&line).as_deref(),
                Some(r"D:\Apps\Acrobat.exe"),
                "padding {pad:?} was not read as a separator"
            );
        }
    }

    /// A missing key's output yields nothing, and so does anything else that
    /// is not a value line.
    ///
    /// `reg.exe` also exits non-zero in that case and [`reg_query`] returns
    /// before reaching here, so this is the second of two guards. It is kept
    /// because the exit code is the platform's promise and this is ours.
    #[test]
    fn output_with_no_value_line_yields_nothing() {
        assert_eq!(
            value_from_reg_output(
                "ERROR: The system was unable to find the specified registry key or value.\r\n"
            ),
            None
        );
        assert_eq!(value_from_reg_output(""), None);
        assert_eq!(
            value_from_reg_output("HKEY_LOCAL_MACHINE\\SOFTWARE\\Classes\\.pdf\r\n"),
            None,
            "a key line with no value under it"
        );
        assert_eq!(
            // string-gap-exempt: `reg.exe`'s own column padding, as above.
            value_from_reg_output("    (Default)    REG_SZ    \r\n"),
            None,
            "a present-but-empty value is not an answer"
        );
    }

    /// ★★★ **What this machine actually answers** — run by hand, never in the
    /// suite.
    ///
    /// `#[ignore]`, and the reason is the one [`super::tests`]' header gives:
    /// a test that shells out to `reg.exe` reports on the machine running it
    /// rather than on the code, and would pass here (Acrobat Pro at
    /// `C:\Program Files\Adobe\Acrobat DC\Acrobat\Acrobat.exe`) and fail on a
    /// build server with no Adobe product at all.
    ///
    /// It is kept because *"the parser is right about the format"* and *"the
    /// format is what this machine prints"* are two different claims, and only
    /// one of them can be pinned in CI. Run it with:
    ///
    /// ```text
    /// cargo test -p pdfcer-gui --lib acrobat::windows::tests::what_this_machine_answers -- --ignored --nocapture
    /// ```
    #[test]
    #[ignore = "asks the real registry; reports on the machine, not on the code"]
    fn what_this_machine_answers() {
        use super::super::{Registrations, resolve};
        let machine = Windows;
        println!(
            "App Paths Acrobat.exe  = {:?}",
            machine.app_path("Acrobat.exe")
        );
        println!(
            "App Paths AcroRd32.exe = {:?}",
            machine.app_path("AcroRd32.exe")
        );
        println!(
            ".pdf handler command   = {:?}",
            machine.pdf_handler_command()
        );
        println!("resolve(no override)   = {:?}", resolve(&machine, None));
    }

    /// `REG_EXPAND_SZ` is read too — some installers write the path with an
    /// environment variable in it.
    ///
    /// ★ pdfcer does **not** expand the variable, and does not need to:
    /// [`super::super::Registrations::exists`] will answer `false` for a path
    /// with a literal `%ProgramFiles%` in it, so such a registration is
    /// declined rather than launched. Saying so here is the honest thing —
    /// this is a known, bounded gap, not an oversight — and the operator's
    /// escape hatch is the Settings field.
    #[test]
    fn an_expandable_value_is_read_as_the_string_it_holds() {
        assert_eq!(
            value_from_reg_output(
                // string-gap-exempt: `reg.exe`'s own column padding, as above.
                "    (Default)    REG_EXPAND_SZ    %ProgramFiles%\\Adobe\\Acrobat.exe\r\n"
            )
            .as_deref(),
            Some(r"%ProgramFiles%\Adobe\Acrobat.exe")
        );
    }
}
