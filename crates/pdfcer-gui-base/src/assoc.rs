//! # `assoc` — making pdfcer the program Windows opens a PDF with
//!
//! `OPERATOR_REQUESTS.md` **O173**: *"we should have an easy way to make
//! pdfce-gui our default opener for pdfs. Ask once with a don't show me again (old-name-exempt: HIS words, quoted verbatim from O173 — correcting an operator's own sentence would stop this being a quotation)
//! check box option. Then it should be in the top of our settings as a button
//! to execute the changeover."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/assoc.md`.

/// The ProgID pdfcer registers itself under.
///
/// `pdfcer.pdf`, not `pdfcer-gui.pdf` and not `PDFCERFile`. The convention
/// Windows expects is `Vendor.Type`, it must be globally unique on the machine,
/// and it is the string that appears in `UserChoice` afterwards — so
/// [`current_owner`] can compare against it and say, in one word, whether the
/// changeover actually happened.
// ui-text-exempt: a registry ProgID, never displayed.
pub const PROGID: &str = "pdfcer.pdf";

/// The name under `RegisteredApplications`, and the entry Windows Settings
/// deep-links to.
// ui-text-exempt: a registry key name; the operator-facing name is in `text::assoc`.
const REGISTERED_APP: &str = "pdfcer";

/// Where the capability block lives.
// ui-text-exempt: a registry path, never displayed.
const CAPABILITIES_PATH: &str = r"Software\pdfcer\Capabilities";

/// What Windows currently opens a `.pdf` with, as a ProgID.
#[must_use]
pub fn current_owner() -> Option<String> {
    let out = reg(&[
        "query",
        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.pdf\UserChoice",
        "/v",
        "ProgId",
    ])?;
    // `reg query` prints `    ProgId    REG_SZ    AppXd4nrz…`; the value is the
    // last whitespace-separated field of the line naming it.
    out.lines()
        .find(|line| line.contains("ProgId"))
        .and_then(|line| line.split_whitespace().last())
        .map(str::to_owned)
}

/// Whether pdfcer is the ProgID Windows currently opens PDFs with.
#[must_use]
pub fn is_default() -> bool {
    current_owner().as_deref() == Some(PROGID)
}

/// **Where Windows' list of PDF programs points**, as far as this build is
/// concerned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Registration {
    /// pdfcer is not in Windows' list. The state every portable build starts
    /// in, because no installer ever ran.
    Absent,
    /// The list names **this** executable.
    Here,
    /// The list names a pdfcer somewhere else. See the type's own note.
    Elsewhere,
}

/// **Everything the two surfaces need to know about the machine, read once.**
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status {
    /// The ProgID Windows currently opens `.pdf` with, if the user has ever
    /// chosen. `None` is the ordinary state on a fresh account, not a fault.
    pub owner: Option<String>,
    /// Whether pdfcer is in the list, and whether it is *this* pdfcer.
    pub registration: Registration,
}

impl Status {
    /// Whether Windows currently opens PDFs with pdfcer.
    #[must_use]
    pub fn is_default(&self) -> bool {
        self.owner.as_deref() == Some(PROGID)
    }
}

/// Read the machine. See [`Status`] for why this is not called from a paint.
#[must_use]
pub fn probe() -> Status {
    Status {
        owner: current_owner(),
        registration: registration(),
    }
}

/// Which of the three [`Registration`] states this machine is in.
#[must_use]
pub fn registration() -> Registration {
    let Some(out) = reg(&[
        "query",
        &format!(r"HKCU\Software\Classes\{PROGID}\shell\open\command"),
        "/ve",
    ]) else {
        return Registration::Absent;
    };
    let Some(exe) = exe_path() else {
        return Registration::Elsewhere;
    };
    if out.to_lowercase().contains(&exe.to_lowercase()) {
        Registration::Here
    } else {
        Registration::Elsewhere
    }
}

/// **Register this executable as a candidate for `.pdf`.**
pub fn register() -> Result<(), String> {
    let exe = exe_path().ok_or_else(crate::text::assoc::no_exe_path)?;
    // ui-text-exempt: a Windows registry command line, read by Explorer and
    // never by a person. The quoting is not cosmetic - an unquoted `%1` breaks
    // on the first space in the path, which is every path under
    // `C:\Program Files`, and an unquoted exe breaks the same way.
    let command = format!("\"{exe}\" \"%1\"");
    let icon = format!("{exe},0");
    let leaf = exe
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or("pdfcer-gui.exe")
        .to_owned();
    let apps = format!(r"HKCU\Software\Classes\Applications\{leaf}");

    let writes: Vec<Vec<String>> = vec![
        set(
            &format!(r"HKCU\Software\Classes\{PROGID}"),
            None,
            &crate::text::assoc::progid_description(),
        ),
        set(
            &format!(r"HKCU\Software\Classes\{PROGID}\DefaultIcon"),
            None,
            &icon,
        ),
        set(
            &format!(r"HKCU\Software\Classes\{PROGID}\shell\open\command"),
            None,
            &command,
        ),
        set(
            r"HKCU\Software\Classes\.pdf\OpenWithProgids",
            Some(PROGID),
            "",
        ),
        set(&format!(r"{apps}\shell\open\command"), None, &command),
        set(&format!(r"{apps}\SupportedTypes"), Some(".pdf"), ""),
        set(
            &format!(r"HKCU\{CAPABILITIES_PATH}"),
            Some("ApplicationName"),
            &crate::text::assoc::application_name(),
        ),
        set(
            &format!(r"HKCU\{CAPABILITIES_PATH}"),
            Some("ApplicationDescription"),
            &crate::text::assoc::application_description(),
        ),
        set(
            &format!(r"HKCU\{CAPABILITIES_PATH}\FileAssociations"),
            Some(".pdf"),
            PROGID,
        ),
        set(
            r"HKCU\Software\RegisteredApplications",
            Some(REGISTERED_APP),
            CAPABILITIES_PATH,
        ),
    ];

    for args in &writes {
        let borrowed: Vec<&str> = args.iter().map(String::as_str).collect();
        run(&borrowed)?;
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("assoc-registered exe={exe:?} progid={PROGID}")
    });
    Ok(())
}

/// **Open Windows' own *Default apps* page, deep-linked to pdfcer.**
pub fn open_settings_page() -> Result<(), String> {
    let url = format!("ms-settings:defaultapps?registeredAppUser={REGISTERED_APP}");
    open_url(&url)
}

// ---------------------------------------------------------------------------
// The `reg.exe` plumbing
// ---------------------------------------------------------------------------

/// One `reg add` invocation, as owned words.
fn set(key: &str, value: Option<&str>, data: &str) -> Vec<String> {
    // ui-text-exempt: `reg.exe` switch names, never displayed.
    let mut args = vec!["add".to_owned(), key.to_owned()];
    match value {
        Some(name) => {
            args.push("/v".to_owned());
            args.push(name.to_owned());
        }
        None => args.push("/ve".to_owned()),
    }
    args.push("/t".to_owned());
    args.push("REG_SZ".to_owned());
    args.push("/d".to_owned());
    args.push(data.to_owned());
    args.push("/f".to_owned());
    args
}

/// Run `reg.exe` and require success.
#[cfg(windows)]
fn run(args: &[&str]) -> Result<(), String> {
    let out = command().args(args).output().map_err(|e| e.to_string())?;
    if out.status.success() {
        return Ok(());
    }
    let said = String::from_utf8_lossy(&out.stderr).trim().to_owned();
    Err(if said.is_empty() {
        crate::text::assoc::refused(args.get(1).copied().unwrap_or_default())
    } else {
        said
    })
}

/// Nothing to do off Windows: there is no registry and no `.pdf` ProgID.
#[cfg(not(windows))]
fn run(_args: &[&str]) -> Result<(), String> {
    Err(crate::text::assoc::settings_page_refused())
}

/// Run `reg.exe` for its **output**, or `None` if it failed for any reason.
#[cfg(windows)]
fn reg(args: &[&str]) -> Option<String> {
    let out = command().args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Off Windows there is nothing to read.
#[cfg(not(windows))]
fn reg(_args: &[&str]) -> Option<String> {
    None
}

/// `reg.exe`, with no console window.
#[cfg(windows)]
fn command() -> std::process::Command {
    use std::os::windows::process::CommandExt;
    /// `CREATE_NO_WINDOW`. See the module header: ten console flashes is how a
    /// deliberate act looks like malware.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    // ui-text-exempt: a program name, never displayed.
    let mut c = std::process::Command::new("reg.exe");
    c.creation_flags(CREATE_NO_WINDOW);
    c
}

/// Where this executable is, as a string.
///
/// `None` when the platform will not say — which `std` documents as possible,
/// and which every caller treats as *"cannot register"* rather than guessing.
#[must_use]
pub fn exe_path() -> Option<String> {
    std::env::current_exe()
        .ok()
        .map(|p| p.display().to_string())
}

/// Hand a URL to the shell.
#[cfg(windows)]
fn open_url(url: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    /// See [`command`].
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    // `cmd /c start "" <url>` rather than `ShellExecute`: the same
    // forbid-unsafe reasoning as the rest of this module. `start`'s first
    // quoted argument is the window TITLE — omitting the empty pair makes
    // `start` read the URL as a title and open a console instead.
    // ui-text-exempt: program and switch names, never displayed.
    std::process::Command::new("cmd")
        .args(["/c", "start", "", url])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|e| e.to_string())
        .and_then(|s| {
            if s.success() {
                Ok(())
            } else {
                Err(crate::text::assoc::settings_page_refused())
            }
        })
}

/// Off Windows there is no such page.
#[cfg(not(windows))]
fn open_url(_url: &str) -> Result<(), String> {
    Err(crate::text::assoc::settings_page_refused())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The ProgID is the `Vendor.Type` shape Windows requires**, and it is
    /// the string [`current_owner`] compares against — so a change here is a
    /// change to what *"pdfcer is the default"* means.
    #[test]
    fn the_progid_is_vendor_dot_type() {
        assert!(PROGID.contains('.'), "a ProgID must be Vendor.Type");
        assert!(!PROGID.contains(' '), "a ProgID may not contain spaces");
    }

    /// **A default-value write uses `/ve`, a named one uses `/v <name>`.**
    ///
    /// Mixing them writes the right data under the wrong name — a registration
    /// that looks complete and does nothing.
    #[test]
    fn a_default_value_write_uses_ve() {
        let default = set(r"HKCU\Software\Classes\x", None, "data");
        assert!(default.contains(&"/ve".to_owned()));
        assert!(!default.contains(&"/v".to_owned()));

        let named = set(r"HKCU\Software\Classes\x", Some(".pdf"), "");
        assert!(named.contains(&"/v".to_owned()));
        assert!(named.contains(&".pdf".to_owned()));
        assert!(!named.contains(&"/ve".to_owned()));
    }

    /// **Every write is forced**, because a prompt from a subprocess with no
    /// console is a hang with no symptom.
    #[test]
    fn every_write_is_unattended() {
        let args = set(r"HKCU\Software\Classes\x", None, "data");
        assert!(
            args.contains(&"/f".to_owned()),
            "reg.exe prompts to overwrite unless forced"
        );
    }

    /// **The `.pdf` `OpenWithProgids` entry carries the ProgID as a NAME with
    /// empty data**, which is the shape Windows reads.
    #[test]
    fn open_with_progids_names_the_progid_rather_than_storing_it() {
        let args = set(
            r"HKCU\Software\Classes\.pdf\OpenWithProgids",
            Some(PROGID),
            "",
        );
        let joined = args.join(" ");
        assert!(joined.contains(&format!("/v {PROGID}")));
        assert!(
            joined.ends_with("/d  /f"),
            "the data must be empty: {joined}"
        );
    }
}
