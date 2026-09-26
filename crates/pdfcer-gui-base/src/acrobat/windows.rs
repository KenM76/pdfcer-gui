//! # `acrobat::windows` — the two things only a real machine can answer
//!
//! Everything impure about O122 is in this file, and it is deliberately the
//! **least interesting** file of the three: it reads registry values, it
//! spawns a process, and it makes no decisions at all. Every decision —
//! which candidate wins, whether Pro beats Reader, which dialog the operator
//! sees — is in [`super`] and [`super::discover`], over values, where a test
//! can reach it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/acrobat/windows.md`.

use std::path::Path;
use std::process::Command;

use super::{Launcher, Registrations, Viewer};

/// The three registry roots that may carry an `App Paths` registration, in
/// the order Windows itself resolves them.
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
/// Consulted **before** `HKLM\SOFTWARE\Classes\.pdf`, and the order is the
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

/// `CREATE_NO_WINDOW` — see this module's header note on the console flash.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// The real machine.
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
        // No `CREATE_NO_WINDOW` here, and that is not an oversight: Acrobat
        // is a GUI program and the flag would be meaningless. It is also not
        // harmless to apply blindly — the flag suppresses a console the child
        // asks for, and a program that wanted one and did not get one behaves
        // differently.
        //
        // `spawn`, never `output` or `status`. Both of those WAIT for the
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

    /// **The real output of the real command on the real machine, parsed.**
    ///
    #[test]
    fn the_real_reg_query_output_yields_the_real_path() {
        // Assembled line by line rather than as one continued literal, and
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

    /// **The separator is whitespace of unspecified width**, which is what
    /// this module's header claims and what the parser must actually honour.
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

    /// **What this machine actually answers** — run by hand, never in the
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
