//! `network_print_reaches_the_printer` — File ▸ Print ▸ Send to network
//! printer… on `fixtures/four-pages.pdf`, against a printer this check runs on
//! 127.0.0.1. Connect must read the printer's answer (staple offered, so the
//! staple control appears); Send must deliver a Print-Job carrying the PDF.
//! The window is placed off the desktop and driven through the scripted
//! pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/netprint_scripted.md`.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use crate::checks::driving::{SHELL_DIAG_ENV, declared};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const FILE: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.file.network_print"; // ui-text-exempt: a trace region name, never displayed
const PRINT_COLLAPSED: &str = "ribbon.group.file.print.collapsed"; // ui-text-exempt: a trace region name, never displayed
const CONNECT: &str = "netprint.connect"; // ui-text-exempt: a trace region name, never displayed
const SEND: &str = "netprint.send"; // ui-text-exempt: a trace region name, never displayed
const STAPLE: &str = "netprint.staple"; // ui-text-exempt: a trace region name, never displayed
const CONNECTED: &str = "netprint-connected"; // ui-text-exempt: a trace event name, never displayed
const SENT: &str = "netprint-sent"; // ui-text-exempt: a trace event name, never displayed
/// The variable that prefills the window's address field.
const URI_ENV: &str = "PDFCER_DIAG_NETPRINT_URI"; // ui-text-exempt: an environment variable name
/// Off the desktop, so no OS input can reach it and none of his is taken.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// The job id the fake printer assigns.
const JOB_ID: i32 = 7;

/// See the module documentation.
pub struct NetworkPrintReachesThePrinter;

impl Check for NetworkPrintReachesThePrinter {
    fn name(&self) -> &'static str {
        "network_print_reaches_the_printer"
    }

    fn defect(&self) -> &'static str {
        "Send to network printer cannot be reached, does not read the printer's offer, or \
         does not deliver the PDF as a Print-Job"
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

/// The fake printer's received requests.
type Jobs = Arc<Mutex<Vec<Vec<u8>>>>;

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let pdf = crate::fixture::workspace_root()
        .join("fixtures")
        .join("four-pages.pdf");
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the four-page fixture is not at {}; it is committed, so this is a broken checkout.",
            pdf.display()
        )));
    }
    let jobs: Jobs = Arc::new(Mutex::new(Vec::new()));
    let port = serve(Arc::clone(&jobs))?;
    let (session, pointer) = launch(ctx, report, pdf, port)?;
    let failure = connect(&session, &pointer, ui_rect, &jobs)?;
    pointer.gone(&session)?;
    if failure.is_some() {
        return Ok(failure);
    }
    delivered(&session, &jobs)
}

/// Launch off the desktop with the address field aimed at the fake printer.
fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    pdf: std::path::PathBuf,
    port: u16,
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
    let mut spec = LaunchSpec::new(&exe, ctx.out("netprint-scripted.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env.push((
        URI_ENV.to_owned(),
        format!("ipp://127.0.0.1:{port}/ipp/print"),
    ));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("netprint-scripted.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    Ok((session, pointer))
}

/// Open the window, Connect, read the offer, and press Send.
fn connect(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    jobs: &Jobs,
) -> Result<Option<String>> {
    let drawn = |region: &str| -> Result<bool> {
        Ok(declared(&session.trace()?, ui_rect, region).is_some())
    };
    pointer.click_region(session, ui_rect, FILE)?;
    if !drawn(ITEM)? && drawn(PRINT_COLLAPSED)? {
        pointer.click_region(session, ui_rect, PRINT_COLLAPSED)?;
    }
    pointer.click_region(session, ui_rect, ITEM)?;
    session.settle(15);
    pointer.click_region(session, ui_rect, CONNECT)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(connected) = trace.last(CONNECTED) else {
        let refused = trace
            .last("netprint-refused")
            .map_or("none".to_owned(), |l| l.raw.clone());
        return Ok(Some(format!(
            "Connect was clicked and no `{CONNECTED}` line followed. Refusal: {refused}. \
             Requests the printer saw: {}.",
            jobs.lock().map_or(0, |j| j.len())
        )));
    };
    // The fake printer lists staple-top-left (20), one tray, and PDF.
    if connected.get("pdf") != Some("1") || connected.get("staples") != Some("1") {
        return Ok(Some(format!(
            "the printer's answer was misread: `{}`; expected pdf=1 staples=1.",
            connected.raw
        )));
    }
    if !drawn(STAPLE)? {
        return Ok(Some(
            "the printer offers a staple and no staple control was drawn.".to_owned(),
        ));
    }
    pointer.click_region(session, ui_rect, SEND)?;
    session.settle(30);
    Ok(None)
}

/// The receipt names the printer's job id, and the printer got the PDF.
fn delivered(session: &Session, jobs: &Jobs) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(sent) = trace.last(SENT) else {
        let refused = trace
            .last("netprint-refused")
            .map_or("none".to_owned(), |l| l.raw.clone());
        return Ok(Some(format!(
            "Send was clicked and no `{SENT}` line followed. Refusal: {refused}."
        )));
    };
    if sent.get("job_id") != Some(JOB_ID.to_string().as_str()) {
        return Ok(Some(format!(
            "the receipt does not carry the printer's job id {JOB_ID}: `{}`.",
            sent.raw
        )));
    }
    let jobs = jobs.lock().map_err(|e| Error::new(e.to_string()))?;
    let Some(job) = jobs.iter().find(|b| operation(b) == Some(0x0002)) else {
        return Ok(Some(format!(
            "the receipt was traced but the printer received no Print-Job ({} requests).",
            jobs.len()
        )));
    };
    if !job.windows(5).any(|w| w == b"%PDF-") {
        return Ok(Some(
            "the Print-Job the printer received carries no PDF after its attributes.".to_owned(),
        ));
    }
    Ok(None)
}

/// The IPP operation id of a request body.
fn operation(body: &[u8]) -> Option<u16> {
    Some(u16::from_be_bytes([*body.get(2)?, *body.get(3)?]))
}

/// Run the fake printer on a free local port; every request body lands in `jobs`.
fn serve(jobs: Jobs) -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| Error::new(e.to_string()))?;
    let port = listener
        .local_addr()
        .map_err(|e| Error::new(e.to_string()))?
        .port();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let _ = answer(&jobs, stream);
        }
    });
    Ok(port)
}

fn answer(jobs: &Mutex<Vec<Vec<u8>>>, stream: TcpStream) -> std::io::Result<()> {
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
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    let reply = response(&body);
    if let Ok(mut j) = jobs.lock() {
        j.push(body);
    }
    let mut out = stream;
    write!(
        out,
        "HTTP/1.1 200 OK\r\nContent-Type: application/ipp\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        reply.len()
    )?;
    out.write_all(&reply)?;
    out.flush()
}

/// The IPP answer to `request`: the printer's attributes, or a job id.
fn response(request: &[u8]) -> Vec<u8> {
    let id = request.get(4..8).unwrap_or(&[0, 0, 0, 1]);
    let mut r = vec![2, 0, 0, 0];
    r.extend_from_slice(id);
    r.push(0x01);
    attr(&mut r, 0x47, "attributes-charset", b"utf-8");
    attr(&mut r, 0x48, "attributes-natural-language", b"en");
    if operation(request) == Some(0x0002) {
        r.push(0x02);
        attr(&mut r, 0x21, "job-id", &JOB_ID.to_be_bytes());
    } else {
        r.push(0x04);
        attr(&mut r, 0x41, "printer-make-and-model", b"ui-verify printer");
        attr(
            &mut r,
            0x49,
            "document-format-supported",
            b"application/pdf",
        );
        attr(&mut r, 0x23, "finishings-supported", &3i32.to_be_bytes());
        attr(&mut r, 0x23, "", &20i32.to_be_bytes());
        attr(&mut r, 0x44, "output-bin-supported", b"top");
    }
    r.push(0x03);
    r
}

/// One attribute value: tag, name, value. An empty name continues the
/// previous attribute's values.
fn attr(out: &mut Vec<u8>, tag: u8, name: &str, value: &[u8]) {
    out.push(tag);
    out.extend_from_slice(&u16::try_from(name.len()).unwrap_or(0).to_be_bytes());
    out.extend_from_slice(name.as_bytes());
    out.extend_from_slice(&u16::try_from(value.len()).unwrap_or(0).to_be_bytes());
    out.extend_from_slice(value);
}
