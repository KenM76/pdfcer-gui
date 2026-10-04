//! `digital_id_created_and_chosen` — the Sign window's *Create a digital ID…*
//! refuses two different passwords, then creates a `.pfx` that Windows'
//! own `certutil` opens with the password typed, makes it the chosen
//! certificate with its identity open, and saves a `.cer` whose SHA-256 is
//! the fingerprint the window shows. Driven off the desktop through the
//! scripted pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/digital_id.md`.

use std::path::Path;

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const FIXTURE: &str = "four-pages.pdf";
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.file.sign"; // ui-text-exempt: a trace region name, never displayed
const COLLAPSED: &str = "ribbon.group.file.security.collapsed"; // ui-text-exempt: a trace region name, never displayed
const BODY: &str = "sign-body";
const OPEN_FORM: &str = "sign-create-id";
const NAME: &str = "sign-create-name";
const PASSWORD: &str = "sign-create-password";
const CONFIRM: &str = "sign-create-confirm";
const GO: &str = "sign-create-go";
const SHARE: &str = "sign-create-share";
const SIGN_CONFIRM: &str = "sign-confirm";
const PFX_ENV: &str = "PDFCER_DIAG_DIGITAL_ID_PATH"; // ui-text-exempt: an environment variable name
const CER_ENV: &str = "PDFCER_DIAG_SHARE_CERTIFICATE_PATH"; // ui-text-exempt: an environment variable name
const HOLDER: &str = "Jane Example";
const PASS: &str = "pw-one";
const WRONG: &str = "pw-two";

/// See the module documentation.
pub struct DigitalIdCreatedAndChosen;

impl Check for DigitalIdCreatedAndChosen {
    fn name(&self) -> &'static str {
        "digital_id_created_and_chosen"
    }

    fn defect(&self) -> &'static str {
        "the Sign window cannot create a digital ID: no Create a digital ID… button, two \
         different passwords are not refused, the .pfx is not written or does not open with its \
         password, it does not become the chosen certificate, or the shared .cer does not match \
         the fingerprint shown"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

struct Drive<'a> {
    session: &'a Session,
    pointer: &'a ScriptedPointer,
    ui_rect: &'static str,
}

impl Drive<'_> {
    /// Wheel the Sign window's body until `name` lies inside it.
    fn reveal(&self, name: &str) -> Result<()> {
        for _ in 0..16 {
            let trace = self.session.trace()?;
            let (Some((rect, viewport)), Some(body)) = (
                declared_in(&trace, self.ui_rect, name),
                declared(&trace, self.ui_rect, BODY),
            ) else {
                return Ok(());
            };
            let dy = if rect.max.y > body.max.y {
                -3.0
            } else if rect.min.y < body.min.y {
                3.0
            } else {
                return Ok(());
            };
            self.pointer.wheel_in(
                self.session,
                viewport.as_deref(),
                WindowPoint::centre_of(body),
                dy,
            )?;
            self.session.settle(8);
        }
        Err(Error::new(format!(
            "`{name}` stayed outside the Sign window's body after sixteen wheel steps."
        )))
    }

    /// Click `name` in whichever viewport declared it; returns that viewport.
    fn click(&self, name: &str) -> Result<Option<String>> {
        if name.starts_with("sign-") {
            self.reveal(name)?;
        }
        let trace = self.session.trace()?;
        let (rect, viewport) = declared_in(&trace, self.ui_rect, name).ok_or_else(|| {
            let prefix = name.split(['.', '-']).next().unwrap_or(name);
            Error::new(format!(
                "no `{name}` region. Declared under `{prefix}`: {}. Trace: {}.",
                list(&declared_names(&trace, self.ui_rect, prefix)),
                self.session.trace_path().display()
            ))
        })?;
        self.pointer.click_in(
            self.session,
            viewport.as_deref(),
            WindowPoint::centre_of(rect),
        )?;
        self.session.settle(12);
        Ok(viewport)
    }

    /// Click the field `name`, clear it, and type `text`.
    fn fill(&self, name: &str, text: &str) -> Result<()> {
        let viewport = self.click(name)?;
        self.pointer
            .key(self.session, viewport.as_deref(), "A", Some("ctrl"))?;
        self.pointer
            .type_text(self.session, viewport.as_deref(), text)?;
        self.session.settle(8);
        Ok(())
    }

    /// The first `event` line after `mark`, waiting up to `rounds` settles.
    fn wait_for(&self, event: &str, mark: usize, rounds: usize) -> Result<Option<String>> {
        for _ in 0..rounds {
            if let Some(line) = self.session.trace()?.last_after(event, mark) {
                return Ok(Some(line.raw.clone()));
            }
            self.session.settle(10);
        }
        Ok(None)
    }
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    pfx: &Path,
    cer: &Path,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let pdf = crate::fixture::workspace_root()
        .join("fixtures")
        .join(FIXTURE);
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "{} is committed and absent: a broken checkout.",
            pdf.display()
        )));
    }
    let mut spec = LaunchSpec::new(&exe, ctx.out("digital_id.trace.txt"));
    spec.pdf = Some(pdf);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1.to_owned()),
        (SHELL_DIAG_ENV.0, SHELL_DIAG_ENV.1.to_owned()),
        (viewport_env, OFFSCREEN.to_owned()),
        (PFX_ENV, pfx.display().to_string()),
        (CER_ENV, cer.display().to_string()),
    ] {
        spec.env.push((k.to_owned(), v));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("digital_id.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(40);
    Ok((session, pointer))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let pfx = ctx.out("digital_id.pfx");
    let cer = ctx.out("digital_id.cer");
    let _ = std::fs::remove_file(&pfx);
    let _ = std::fs::remove_file(&cer);
    let (session, pointer) = launch(ctx, report, &pfx, &cer)?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let d = Drive {
        session: &session,
        pointer: &pointer,
        ui_rect,
    };
    d.click(TAB)?;
    let trace = session.trace()?;
    if declared(&trace, ui_rect, ITEM).is_none() && declared(&trace, ui_rect, COLLAPSED).is_some() {
        d.click(COLLAPSED)?;
    }
    d.click(ITEM)?;
    session.settle(20);
    d.click(OPEN_FORM)?;
    d.fill(NAME, HOLDER)?;
    if let Some(failure) = mismatch(&d, &pfx, report)? {
        return Ok(Some(failure));
    }
    if let Some(failure) = create(&d, &pfx, report)? {
        return Ok(Some(failure));
    }
    let verdict = share(&d, &cer, report)?;
    pointer.gone(&session)?;
    Ok(verdict)
}

/// Two different passwords are refused before anything is generated.
fn mismatch(d: &Drive<'_>, pfx: &Path, report: &mut CheckReport) -> Result<Option<String>> {
    d.fill(PASSWORD, PASS)?;
    d.fill(CONFIRM, WRONG)?;
    let mark = d.session.trace()?.mark();
    d.click(GO)?;
    let refused = d.wait_for("digital-id-refused", mark, 10)?;
    if !refused
        .as_deref()
        .is_some_and(|l| l.ends_with("kind=mismatch"))
        || pfx.exists()
    {
        return Ok(Some(format!(
            "two different passwords were not refused by name: refusal {refused:?}, file \
             written {}.",
            pfx.exists()
        )));
    }
    report.note("two different passwords: refused with kind=mismatch, nothing written");
    Ok(None)
}

/// The matching password creates the file, Windows opens it, and the window
/// takes it as the chosen certificate.
fn create(d: &Drive<'_>, pfx: &Path, report: &mut CheckReport) -> Result<Option<String>> {
    d.fill(CONFIRM, PASS)?;
    let mark = d.session.trace()?.mark();
    d.click(GO)?;
    let Some(created) = d.wait_for("digital-id-created", mark, 120)? else {
        return Ok(Some(format!(
            "Create and save… with matching passwords traced no `digital-id-created`. Last \
             refusal: {:?}. Trace: {}.",
            d.session
                .trace()?
                .last_after("digital-id-refused", mark)
                .map(|l| l.raw.clone()),
            d.session.trace_path().display()
        )));
    };
    report.note(format!("`{created}`"));
    // Opening the container re-derives its key from the password, which takes
    // frames of its own after `digital-id-created`.
    let opened = d.wait_for("sign-identity", mark, 60)?;
    let declined = d
        .session
        .trace()?
        .last_after("sign-identity-declined", mark)
        .map(|l| l.raw.clone());
    if let Some(declined) = declined {
        return Ok(Some(format!(
            "the new ID did not open with its own password: `{declined}`."
        )));
    }
    d.session.settle(10);
    let trace = d.session.trace()?;
    if opened.is_none() || declared(&trace, d.ui_rect, SIGN_CONFIRM).is_none() {
        return Ok(Some(
            "the new ID was created but the window did not open it as the chosen certificate: \
             no `sign-identity` line, or no live Sign button."
                .to_owned(),
        ));
    }
    let dump = std::process::Command::new("certutil")
        .args(["-p", PASS, "-dump"])
        .arg(pfx)
        .output()
        .map_err(|e| Error::new(format!("certutil did not run: {e}")))?;
    let text = String::from_utf8_lossy(&dump.stdout);
    if !dump.status.success() || !text.contains(HOLDER) {
        return Ok(Some(format!(
            "Windows' certutil could not open {} with the password typed, or found no \
             `{HOLDER}` in it (exit {:?}).",
            pfx.display(),
            dump.status.code()
        )));
    }
    report.note("certutil opened the .pfx with the typed password and names the holder");
    Ok(None)
}

/// The shared `.cer` hashes to the fingerprint the window traced.
fn share(d: &Drive<'_>, cer: &Path, report: &mut CheckReport) -> Result<Option<String>> {
    let traced = d
        .session
        .trace()?
        .events("digital-id-created")
        .last()
        .and_then(|l| l.get("fingerprint").map(str::to_lowercase))
        .unwrap_or_default();
    let mark = d.session.trace()?.mark();
    d.click(SHARE)?;
    if d.wait_for("digital-id-shared", mark, 20)?.is_none() || !cer.is_file() {
        return Ok(Some(format!(
            "Save the certificate to share… wrote no {}.",
            cer.display()
        )));
    }
    let hash = std::process::Command::new("certutil")
        .args(["-hashfile"])
        .arg(cer)
        .arg("SHA256")
        .output()
        .map_err(|e| Error::new(format!("certutil did not run: {e}")))?;
    let digest: String = String::from_utf8_lossy(&hash.stdout)
        .lines()
        .nth(1)
        .unwrap_or_default()
        .split_whitespace()
        .collect::<String>()
        .to_lowercase();
    if traced.len() != 64 || digest != traced {
        return Ok(Some(format!(
            "the shared certificate's SHA-256 is `{digest}` and the window showed `{traced}`."
        )));
    }
    report.note(format!(
        "the .cer hashes to the fingerprint shown: {digest}"
    ));
    Ok(None)
}
