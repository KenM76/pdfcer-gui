//! `checks::signing::archive` — **File ▸ Add archive time-stamp writes a copy
//! sealed by a document time-stamp, and leaves the open file alone**
//!
//! Drives the command off the desktop through the scripted pointer (no OS
//! mouse or keyboard), on a document already signed by another producer,
//! against a time-stamping server this check runs on the loopback interface:
//! `openssl ts -reply` over the engine corpus's own TSA key. No packet leaves
//! the machine. A build without the `timestamp` feature declares no ribbon
//! item and the check reports SKIPPED; a machine without `openssl` on `PATH`
//! likewise.
//!
//! Oracles: the `archive-written` trace line (what the engine reported), and
//! the written bytes (a second `/ByteRange` and a `/DocTimeStamp`), and the
//! source file's bytes unchanged.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;

use super::reaching::engine_fixture;
use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const TAB: &str = "ribbon.tab.security";
const ITEM: &str = "ribbon.item.file.add_archive_timestamp";
const COLLAPSED: &str = "ribbon.group.security.security.collapsed";
/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const REGION_BODY: &str = "archive.body";
const REGION_SERVER: &str = "archive.server";
const REGION_SAVE: &str = "archive.commit";
/// Signed once by pyHanko, no validation evidence: the stamp is expected to
/// report one prior signature and no DSS.
const SIGNED: &str = "signing/foreign-pyhanko-first.pdf";

/// The time-stamping server's configuration for `openssl ts -reply`.
const TSA_CNF: &str = "[ tsa ]\ndefault_tsa = cfg\n[ cfg ]\nserial = ./serial\n\
crypto_device = builtin\nsigner_digest = sha256\ndefault_policy = 1.2.3.4.1\n\
digests = sha256, sha384, sha512\naccuracy = secs:1\nordering = no\ntsa_name = no\n\
ess_cert_id_chain = no\ness_cert_id_alg = sha256\n";

/// See the module documentation.
pub struct AnArchiveTimestampSealsACopy;

impl Check for AnArchiveTimestampSealsACopy {
    fn name(&self) -> &'static str {
        "an_archive_timestamp_seals_a_copy"
    }

    fn defect(&self) -> &'static str {
        "Add archive time-stamp cannot be reached, writes nothing, writes a copy without a \
         document time-stamp, or changes the open document"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let signed_source = engine_fixture(SIGNED, "the pyHanko-signed document")?;
    // Driven on a copy: the source fixture belongs to the engine repository.
    let signed = ctx.out("archive-source.pdf");
    std::fs::copy(&signed_source, &signed)
        .map_err(|e| Error::new(format!("copying {SIGNED}: {e}")))?;
    let before = std::fs::read(&signed).map_err(|e| Error::new(e.to_string()))?;
    let port = serve(ctx)?;
    let out = ctx.out("archived.pdf");
    let _ = std::fs::remove_file(&out);

    let mut spec = LaunchSpec::new(&exe, ctx.out("archive.trace.txt"));
    spec.pdf = Some(signed.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env.push((
        "PDFCER_DIAG_SAVE_PATH".to_owned(),
        out.display().to_string(),
    ));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("archive.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);

    // Click a declared region; returns the viewport it lives in.
    let click = |region: &str| -> Result<Option<String>> {
        let trace = session.trace()?;
        let (rect, viewport) = declared_in(&trace, ui_rect, region).ok_or_else(|| {
            let prefix = region.rsplit_once('.').map_or(region, |(head, _)| head);
            Error::new(format!(
                "no `{region}` region. Declared under `{prefix}`: {}.",
                list(&declared_names(&trace, ui_rect, prefix))
            ))
        })?;
        pointer.click_in(&session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
        session.settle(15);
        Ok(viewport)
    };

    click(TAB)?;
    let trace = session.trace()?;
    if declared(&trace, ui_rect, ITEM).is_none() {
        if declared(&trace, ui_rect, COLLAPSED).is_some() {
            click(COLLAPSED)?;
        } else {
            return Err(Error::new(format!(
                "no `{ITEM}` on the File tab: this build excludes the `timestamp` feature, which \
                 reads as SKIPPED. Items declared: {}.",
                list(&declared_names(&trace, ui_rect, "ribbon.item.file."))
            )));
        }
    }
    click(ITEM)?;
    session.settle(20);
    if declared(&session.trace()?, ui_rect, REGION_BODY).is_none() {
        return Ok(Some(format!(
            "`{ITEM}` was clicked and no `{REGION_BODY}` window was drawn."
        )));
    }
    let vp = click(REGION_SERVER)?;
    pointer.key(&session, vp.as_deref(), "A", Some("ctrl"))?;
    pointer.type_text(
        &session,
        vp.as_deref(),
        &format!("http://127.0.0.1:{port}/"),
    )?;
    session.settle(10);
    click(REGION_SAVE)?;
    // Wait for the stamp, bounded: the local server answers in well under a second.
    for _ in 0..40 {
        let t = session.trace()?;
        if t.events("archive-written").last().is_some()
            || t.events("archive-refused").last().is_some()
        {
            break;
        }
        session.settle(10);
    }
    let shot = ctx.out("archive-after.png");
    if pointer.screenshot(&session, &shot).is_ok() {
        report.artifact(shot);
    }
    pointer.gone(&session)?;

    let trace = session.trace()?;
    let requested = trace
        .events("archive-requested")
        .last()
        .and_then(|l| l.get("signatures").map(str::to_owned));
    let written = trace.events("archive-written").last().map(|l| {
        (
            l.get("prior").map(str::to_owned),
            l.get("dss").map(str::to_owned),
        )
    });
    let refused = trace
        .events("archive-refused")
        .last()
        .and_then(|l| l.get("reason").map(str::to_owned));
    drop(session);
    let bytes = std::fs::read(&out).unwrap_or_default();
    let ranges = count(&bytes, b"/ByteRange");
    let doc_ts = count(&bytes, b"/DocTimeStamp");
    let untouched = std::fs::read(&signed).is_ok_and(|now| now == before);
    report.note(format!(
        "requested signatures={requested:?}; written={written:?}; refused={refused:?}; copy \
         bytes={} ByteRange={ranges} DocTimeStamp={doc_ts}; source unchanged={untouched}",
        bytes.len()
    ));

    let mut findings = Vec::new();
    if requested.as_deref() != Some("1") {
        findings.push(format!(
            "the window counted {requested:?} signatures in a document signed once."
        ));
    }
    match written {
        Some((prior, dss)) => {
            if prior.as_deref() != Some("1") || dss.as_deref() != Some("0") {
                findings.push(format!(
                    "the engine reported prior={prior:?} dss={dss:?}; the fixture has one \
                     signature and no validation evidence."
                ));
            }
        }
        None => findings.push(format!(
            "no `archive-written` line (refused reason={refused:?}): the stamp was not written."
        )),
    }
    if ranges != 2 || doc_ts < 1 {
        findings.push(format!(
            "the copy holds {ranges} /ByteRange and {doc_ts} /DocTimeStamp; a sealed copy of a \
             once-signed file holds 2 and at least 1."
        ));
    }
    if !untouched {
        findings.push("the open document's file changed on disk.".to_owned());
    }
    Ok((!findings.is_empty()).then(|| findings.join("\n")))
}

fn count(hay: &[u8], needle: &[u8]) -> usize {
    hay.windows(needle.len()).filter(|w| *w == needle).count()
}

/// Start a loopback RFC 3161 server and return its port. It answers until the
/// process exits.
fn serve(ctx: &CheckContext) -> Result<u16> {
    let dir = ctx.out("tsa");
    std::fs::create_dir_all(&dir).map_err(|e| Error::new(e.to_string()))?;
    let key = engine_fixture("signing/tsa-rsa2048.key.der", "the TSA key")?;
    let cert = engine_fixture("signing/tsa-rsa2048.cer", "the TSA certificate")?;
    openssl(
        &dir,
        &[
            "pkey",
            "-inform",
            "DER",
            "-in",
            &path(&key),
            "-out",
            "k.pem",
        ],
    )?;
    openssl(
        &dir,
        &[
            "x509",
            "-inform",
            "DER",
            "-in",
            &path(&cert),
            "-out",
            "c.pem",
        ],
    )?;
    std::fs::write(dir.join("serial"), "01\n").map_err(|e| Error::new(e.to_string()))?;
    std::fs::write(dir.join("tsa.cnf"), TSA_CNF).map_err(|e| Error::new(e.to_string()))?;
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| Error::new(e.to_string()))?;
    let port = listener
        .local_addr()
        .map_err(|e| Error::new(e.to_string()))?
        .port();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let _ = answer(&dir, stream);
        }
    });
    Ok(port)
}

fn answer(dir: &Path, stream: std::net::TcpStream) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || line.trim().is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            length = value.trim().parse().unwrap_or(0);
        }
    }
    let mut query = vec![0; length];
    reader.read_exact(&mut query)?;
    std::fs::write(dir.join("q.tsq"), &query)?;
    let _ = std::fs::remove_file(dir.join("r.tsr"));
    let args = [
        "ts",
        "-reply",
        "-config",
        "tsa.cnf",
        "-queryfile",
        "q.tsq",
        "-inkey",
        "k.pem",
    ];
    let status = Command::new("openssl")
        .current_dir(dir)
        .args(args)
        .args(["-signer", "c.pem", "-out", "r.tsr"])
        .output()?;
    let body = if status.status.success() {
        std::fs::read(dir.join("r.tsr"))?
    } else {
        Vec::new()
    };
    let (code, kind) = if body.is_empty() {
        ("500 Error", "text/plain")
    } else {
        ("200 OK", "application/timestamp-reply")
    };
    let mut stream = stream;
    write!(
        stream,
        "HTTP/1.1 {code}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(&body)?;
    stream.flush()
}

fn openssl(dir: &Path, args: &[&str]) -> Result<()> {
    let run = Command::new("openssl").current_dir(dir).args(args).output().map_err(|e| {
        Error::new(format!(
            "`openssl` is not on PATH ({e}); this check runs its own time-stamping server with it \
             and reports SKIPPED without it."
        ))
    })?;
    if run.status.success() {
        Ok(())
    } else {
        Err(Error::new(format!(
            "`openssl {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&run.stderr)
        )))
    }
}

fn path(p: &Path) -> String {
    p.display().to_string()
}
