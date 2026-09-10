//! # `app::assoc` — making pdfcer the program Windows opens a PDF with
//!
//! `OPERATOR_REQUESTS.md` **O173**: *"we should have an easy way to make
//! pdfce-gui our default opener for pdfs. Ask once with a don't show me again (old-name-exempt: HIS words, quoted verbatim from O173 — correcting an operator's own sentence would stop this being a quotation)
//! check box option. Then it should be in the top of our settings as a button
//! to execute the changeover."*
//!
//! ## ★★★ What an application can and cannot do here, because it shapes every
//! ## word on the two surfaces above this module
//!
//! **No program can silently take the `.pdf` association on Windows 10 or 11.**
//! It could once, through `IApplicationAssociationRegistration::SetAppAsDefault`,
//! and Microsoft removed that ability on purpose — an unattended "make me the
//! default" was the single most abused call on the platform. What replaced it
//! is a **hash** stored under `…\Explorer\FileExts\.pdf\UserChoice`, computed
//! from the extension, the ProgID, the user's SID and a salt Microsoft does not
//! publish, and refreshed by Explorer. A value written there by anything but
//! the shell is detected and thrown away, silently, and on some builds it also
//! resets the association to Edge.
//!
//! ⇒ So the honest decomposition is **two halves, and this module owns the
//! first one entirely**:
//!
//! 1. **Become a candidate.** Register a ProgID, an `OpenWithProgids` entry
//!    against `.pdf`, an `Applications\…` block and a
//!    `RegisteredApplications` capability set. After this pdfcer appears in
//!    *Open with*, in *Choose a default app*, and as an application Windows
//!    Settings can be deep-linked to. **Today it appears in none of them** —
//!    this is a portable build that no installer ever registered — which is why
//!    the changeover cannot currently even be attempted by hand.
//! 2. **Let the operator confirm**, in the OS's own dialog, which is the only
//!    place the confirmation is legal. [`open_settings_page`] opens that page
//!    deep-linked to pdfcer, so it is one click rather than four levels of
//!    navigation.
//!
//! ★★ The surfaces therefore never claim to have changed the default. They say
//! what they did and what is left, and [`current_owner`] reports what Windows
//! actually thinks afterwards — a button that claimed success and had not
//! succeeded would be worse than no button, because the operator would stop
//! looking.
//!
//! ## ★★ Why `reg.exe` rather than the registry API
//!
//! This crate carries `#![forbid(unsafe_code)]` and means it. Writing ten keys
//! through `advapi32` would need a second quarantine crate beside
//! `native-window`, for a job the operating system ships a supported
//! command-line tool for. `reg.exe` is in `System32` on every Windows since
//! 2000, it is the tool Microsoft's own documentation uses for exactly these
//! keys, and it returns a process exit code — which is a better error channel
//! than an `LSTATUS` this module would have to translate anyway.
//!
//! ★ Every invocation carries `CREATE_NO_WINDOW`, from `std`'s own
//! `CommandExt` — safe, no dependency — because a console flashing up ten
//! times is how a considered action looks like a virus.
//!
//! ## Rule 4 — fuzzy, never sneaky
//!
//! Nothing here touches a document, so the canvas rule is not in play. The
//! disclosure half is: this module **reports what it did and what Windows now
//! says**, and the surfaces put that off-canvas in the Settings group and in
//! the dialog. An operator who presses the button and is not asked by Windows
//! must be able to find out why, and [`current_owner`] is how.
//!
//! ## Rule 15
//!
//! No dimension of either kind appears in this module.

/// The ProgID pdfcer registers itself under.
///
/// ★ `pdfcer.pdf`, not `pdfcer-gui.pdf` and not `PDFCERFile`. The convention
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
///
/// `None` when the key is absent, which is the ordinary state on a machine
/// where nobody has ever chosen — Windows then falls back to the machine-wide
/// association, and there is no honest single answer to report.
///
/// ★ Read from `UserChoice` rather than from `HKCR\.pdf`, because `UserChoice`
/// is what Explorer actually consults and the two disagree routinely.
/// Reporting the wrong one would produce a state line saying pdfcer is the
/// default while double-clicking still opened Edge — the exact confusion this
/// line exists to end.
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
///
/// ★★ Three states rather than a bool, and the third is the load-bearing one. A
/// portable build gets unzipped somewhere new; the registration then still
/// names the *old* folder, so Windows opens a build the operator thought they
/// had replaced — or nothing at all, if the old folder is gone. That failure
/// looks exactly like success from the inside: the key exists, the ProgID is
/// pdfcer's, and everything reports fine. Only the **path** tells them apart.
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
///
/// ★★★ Why this is a struct probed on demand rather than three functions the
/// UI calls: a Settings pane redraws on **every frame**, and every one of these
/// answers costs a `reg.exe` process. `dialogs::settings::acrobat`'s header
/// states the same rule for the same reason — *"this module must not resolve,
/// because resolving spawns processes and a Settings pane redraws on every
/// frame"*. So the probe happens when the window opens and when the button is
/// pressed, and never in a paint.
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
    ///
    /// ★ Derived from [`Self::owner`] — a reading of what Windows recorded —
    /// and never from *"we pressed the button"*. Only the operator can make
    /// this true, in a dialog pdfcer does not own, so pdfcer's memory of its
    /// own actions is not evidence.
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
///
/// Returns `Ok(())` when every key was written, or the first failure as a
/// sentence — `reg.exe`'s own message, which names the key it could not write
/// and is more use than any wording this module could invent.
///
/// ## What is written, and why each one is needed
///
/// | Key | Without it |
/// |---|---|
/// | `Classes\pdfcer.pdf` and its `shell\open\command` | there is nothing to associate; the ProgID does not exist |
/// | `Classes\pdfcer.pdf\DefaultIcon` | pdfcer's entry in every chooser is the blank generic icon |
/// | `Classes\.pdf\OpenWithProgids` | pdfcer is missing from the **Open with** submenu |
/// | `Classes\Applications\<exe>` + `SupportedTypes` | *Open with ▸ Choose another app* does not offer it either |
/// | `Software\pdfcer\Capabilities` + `RegisteredApplications` | Windows Settings' *Default apps* list does not contain pdfcer, so the deep link has nothing to land on |
///
/// ★ All under `HKCU`. Nothing here needs administrator rights, nothing affects
/// another user of the machine, and an operator who changes their mind can
/// delete one key. A portable build that wrote to `HKLM` would be a portable
/// build that needed elevation and left something behind — the opposite of what
/// the package promises on its front page.
///
/// # Errors
///
/// The first `reg.exe` invocation that failed, with its own output.
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
///
/// ★★ `registeredAppUser=` and not `registeredAUMID=`. The first names an entry
/// in `HKCU\Software\RegisteredApplications` — which [`register`] has just
/// written — and the second names a packaged (Store) app identity that a
/// portable exe does not have and cannot get. Passing the wrong one lands on
/// the unfiltered list, which is a page with three hundred entries on it.
///
/// ⚠ Older Windows 10 builds ignore the query entirely and open the top of the
/// Default-apps page. That is a degraded outcome rather than a failure, and it
/// is why the wording on both surfaces says *"Windows will ask you to
/// confirm"* rather than describing a specific dialog: the dialog differs
/// between builds, and a sentence describing one of them would be wrong on the
/// others.
///
/// # Errors
///
/// The shell's own refusal when the page could not be opened.
pub fn open_settings_page() -> Result<(), String> {
    let url = format!("ms-settings:defaultapps?registeredAppUser={REGISTERED_APP}");
    open_url(&url)
}

// ---------------------------------------------------------------------------
// The `reg.exe` plumbing
// ---------------------------------------------------------------------------

/// One `reg add` invocation, as owned words.
///
/// `value` is `None` for a key's default (`/ve`) and `Some` for a named one.
///
/// ★ `/f` on every write: these are idempotent declarations, not a
/// conversation. A prompt from a subprocess with no console is a hang with no
/// symptom.
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
///
/// # Errors
///
/// `reg.exe`'s own stderr, or the reason it could not be started.
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
///
/// ★ `None` rather than an error, because every caller is asking a question
/// about the machine that has a legitimate *"cannot tell"* answer — a key that
/// does not exist makes `reg query` exit non-zero, and on a fresh machine that
/// is the normal case rather than a fault worth a sentence.
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
///
/// # Errors
///
/// Whatever the shell said.
#[cfg(windows)]
fn open_url(url: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    /// See [`command`].
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    // ★ `cmd /c start "" <url>` rather than `ShellExecute`: the same
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
    ///
    /// Writing the ProgID as the *data* of a default value is the plausible
    /// mistake, and it produces a key that exists and an *Open with* menu that
    /// does not list pdfcer.
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
