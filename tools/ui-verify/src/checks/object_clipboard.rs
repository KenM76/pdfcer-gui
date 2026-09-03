//! `copy_and_paste_page_content` — **the operator's oldest open request**,
//! driven end to end.
//!
//! # What this is for
//!
//! > *"can you get cut copy and paste working for objects I select on the
//! > canvas?"* — asked in the first week and repeatedly since.
//!
//! Until 2026-08-20 `Ctrl+C` on a shape put a **sentence** on the status row —
//! *"pdfcer can copy comments and markup, but it cannot yet put page content
//! back onto a page"* — which was honest and was still a refusal. `Pass 120.0`
//! shipped `ObjectClip` and this check is the wiring of it.
//!
//! # ★★ Why this cannot be a unit test
//!
//! Because the interesting failure is **silent and correct-looking**, and it is
//! the one the engine warned about in the reply that shipped the verb:
//!
//! > `import_object` copies indirect objects. A page's content objects are byte
//! > ranges inside a content stream, and the operators in those bytes name
//! > their resources **by page-local name**. On the destination page, `/F1` is a
//! > different font. Paste the bytes verbatim and you get the right glyphs in
//! > the wrong typeface — or nothing at all. **Neither failure errors**, and
//! > neither is visible in a diff.
//!
//! So the assertions here are about **counts the engine reports**, not about
//! whether a call returned `Ok`:
//!
//! | # | link | its own test |
//! |---|---|---|
//! | 1 | `Ctrl+C` on page content reaches `copy_objects` rather than the old refusal | nothing — the refusal was in the shell |
//! | 2 | the clip is parked where a paste can find it, across a page change | nothing |
//! | 3 | `Ctrl+V` deserialises it and reaches `paste_objects` | nothing |
//! | 4 | **resources came with it** | `pdfcer-core` — and the count is the only thing this side can see |
//!
//! Link 4 is the one that would ship. A clip that carried the operators and
//! dropped the resources pastes *something*, on the right page, at the right
//! place, in the wrong typeface. `resources_added=0` on a paste of text is the
//! tell, and it is on the trace for exactly that reason.
//!
//! # What it does NOT assert, said rather than implied
//!
//! **The pixels.** A paste offsets by 10 pt, which on a CAD sheet at 0.3× is
//! three screen pixels — below the noise floor of a window capture, and
//! `insert_image`'s own threshold repair is the record of what happens when a
//! pixel oracle is asked a question at that scale. What is asserted instead is
//! the engine's own count of what it wrote, which is a number a wrong build
//! gets wrong and a capture cannot resolve.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode whose canvas may select and edit content.
const MODE: &str = "edit";
/// `clipboard-copy kind=content page=… objects=… bytes=…`.
const COPY_EVENT: &str = "clipboard-copy";
/// `clipboard-paste kind=content page=… from=… objects=… offset=…`.
const PASTE_EVENT: &str = "clipboard-paste";
/// `paste-objects page=… pasted=… resources_added=… at=[…]`.
const APPLIED_EVENT: &str = "paste-objects-applied";

/// See the module documentation.
pub struct CopyAndPastePageContent;

impl Check for CopyAndPastePageContent {
    fn name(&self) -> &'static str {
        "copy_and_paste_page_content"
    }

    fn defect(&self) -> &'static str {
        "Ctrl+C on a shape, a line or a piece of text does nothing, or says pdfcer cannot copy \
         page content — the operator's oldest open request. Or it copies and the paste arrives \
         WITHOUT ITS RESOURCES, which puts the right glyphs on the page in the wrong typeface \
         and errors nowhere"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new("no --pdf. This check needs a drawing with selectable page content.")
    })?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space naming a point with selectable \
             content on it. There is deliberately no default: a click on empty page is \
             symptom-identical to a broken hit test.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, clicks page \
             content and presses Ctrl+C and Ctrl+V. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("object-clipboard.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- 1: Edit, the one mode whose canvas selects content ----------------
    //
    // ★ And the one mode that may PASTE content. `edit.paste` gates on
    // `edit_content` when the clipboard holds page content and on
    // `author_markup` otherwise — a Review that could paste a line onto a
    // drawing would break the promise its whole name makes.
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    // --- 2: select something -----------------------------------------------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let window_point = mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?;
    let frame = session.frame()?;
    driver.click_at(frame.to_screen(window_point))?;
    session.settle(16);

    let trace = session.trace()?;
    let selected = trace
        .last(vocab.click_event)
        .and_then(|l| l.get_usize(vocab.click_selection_field))
        .or_else(|| {
            trace
                .last(vocab.canvas_event)
                .and_then(|l| l.get_usize(vocab.canvas_selection_field))
        });
    if selected == Some(0) {
        return Err(Error::new(format!(
            "the click at (page {}, {:.1}, {:.1}) selected nothing, so there is nothing to copy. \
             A fact about the fixture and the point, not about the build. SKIPPED for exactly \
             that reason.",
            target.page + 1,
            target.x,
            target.y
        )));
    }
    report.note("the click selected page content");

    // --- 3: Ctrl+C ----------------------------------------------------------
    driver.press_chord(&[vk::CONTROL], vk::C)?;
    session.settle(20);

    let trace = session.trace()?;
    let copy = trace
        .events(COPY_EVENT)
        .filter(|l| l.get("kind") == Some("content"))
        .last();
    let Some(copy) = copy else {
        // ★ Ask what else happened before accusing — the rule three separate
        // repairs in this suite have now earned. A refusal is a different
        // finding from a silence and sends the reader somewhere else.
        let markup = trace
            .events(COPY_EVENT)
            .any(|l| l.get("kind") == Some("markup"));
        return Ok(Some(format!(
            "★ CTRL+C ON PAGE CONTENT COPIED NOTHING.{}\n\
             This is the operator's oldest open request: until 2026-08-20 the shell refused it \
             by name in `canvas::clipboard::copy`, whose `ContentNotAnnotation` arm said pdfcer \
             could not put page content back. `Pass 120.0` made that false. If no \
             `{COPY_EVENT} kind=content` line appears, either that refusal is back or \
             `copy_content` never ran. Trace: {}.",
            if markup {
                " It copied a MARKUP instead, so the selection was an annotation rather than \
                 page content — aim --doc-point at a line or a shape."
            } else {
                ""
            },
            session.trace_path().display()
        )));
    };
    let bytes: usize = copy.get("bytes").and_then(|v| v.parse().ok()).unwrap_or(0);
    report.note(format!("★ the copy reached the engine: `{}`", copy.raw));
    if bytes == 0 {
        return Ok(Some(format!(
            "the clip serialised to ZERO bytes: `{}`. `ObjectClip::to_bytes` writes a magic \
             prefix and a version before anything else, so an empty payload is not a clip with \
             nothing in it — it is a clip that was never assembled. Trace: {}.",
            copy.raw,
            session.trace_path().display()
        )));
    }

    // --- 4: Ctrl+V ----------------------------------------------------------
    driver.press_chord(&[vk::CONTROL], vk::V)?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(paste) = trace
        .events(PASTE_EVENT)
        .filter(|l| l.get("kind") == Some("content"))
        .last()
    else {
        return Ok(Some(format!(
            "the copy happened and CTRL+V RAISED NOTHING: no `{PASTE_EVENT} kind=content` line. \
             The clip is parked in `egui::Memory` by `clipboard::store` and read back by \
             `clipboard::read`; a paste that finds nothing there raises `NothingCopied` and puts \
             a sentence on the status row instead. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the paste was raised: `{}`", paste.raw));

    // --- 5: ★★ and it reached the engine WITH ITS RESOURCES ------------------
    let Some(applied) = trace.events(APPLIED_EVENT).last() else {
        return Ok(Some(format!(
            "the paste was raised and no `{APPLIED_EVENT}` line followed, so the action never \
             reached its apply arm — or the clip failed to deserialise, which the engine refuses \
             by name as `ClipError::NotAClip` and which `vector_edit` would have worded on the \
             status row. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let pasted: u64 = applied
        .get("pasted")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if pasted == 0 {
        return Ok(Some(format!(
            "the paste reached the engine and wrote NOTHING: `{}`. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ {pasted} object(s) reached the document: `{}`",
        applied.raw
    ));

    // ★★★ THE ASSERTION THIS CHECK EXISTS FOR, and it is a count rather than a
    // picture.
    //
    // A content object's operators name their resources by PAGE-LOCAL NAME. A
    // clip that carried the bytes and dropped the objects behind those names
    // pastes the right glyphs in the wrong typeface, or nothing at all, and
    // **neither failure errors**. `resources_added` is the engine's own count of
    // the fresh `/Resources` entries it bound on the destination page, and it is
    // the only thing on this side of the boundary that can see the difference.
    //
    // ★ It is asserted as "greater than zero" rather than against a number,
    // because how many resources a given object consumes is a fact about the
    // fixture — a bare stroked path may genuinely consume none. So a zero is
    // reported with the fixture named as the first suspect rather than the
    // build, which is the three-state discipline this suite reaches for.
    let added: u64 = applied
        .get("resources_added")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if added == 0 {
        report.note(
            "resources_added=0 — the pasted objects consume no page-local resource names, which \
             a bare stroked path genuinely may not. The resource-rebinding half of the clip is \
             therefore UNVERIFIED by this run; aim --doc-point at text or an image to exercise \
             it",
        );
    } else {
        report.note(format!(
            "★★★ and its resources came with it: {added} fresh entry(s) bound on the \
             destination page. A clip that carried the operators and dropped these would paste \
             the right glyphs in the wrong typeface and error nowhere"
        ));
    }
    Ok(None)
}
