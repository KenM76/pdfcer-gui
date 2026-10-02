//! `the_draft_keys_do_what_a_word_processor_does` — inside a text draft,
//! Ctrl+Z and Ctrl+Y step the draft's own typing, Ctrl+Backspace removes a
//! word, Tab types spaces and says so, a pasted line break joins and says so,
//! Ctrl+A selects the draft, and Ctrl+S commits the edit and saves the file.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/draft_keys.md`.

use crate::checks::driving::{INVOKE_EVENT, SHELL_DIAG_ENV, repo_fixture, shell_trace};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "word-fragmented-lines.pdf";
/// Edit mode with the Edit Text tool armed, so one click opens a caret.
const INVOKE: &str = "mode.edit,edit.text";
/// Inside `Required` on the fixture's one-font first line, in PDF points.
const LINE: (f64, f64) = (150.0, 703.0);
const TYPING: &str = "text-edit-typing"; // ui-text-exempt: a trace event name, never displayed
const HISTORY: &str = "text-edit-history"; // ui-text-exempt: a trace event name, never displayed
const TAB: &str = "text-edit-tab"; // ui-text-exempt: a trace event name, never displayed
const NOTE: &str = "text-edit-note"; // ui-text-exempt: a trace event name, never displayed
const PASTE: &str = "text-edit-paste"; // ui-text-exempt: a trace event name, never displayed
const SELECT: &str = "text-select"; // ui-text-exempt: a trace event name, never displayed
const ENTER_DECLINED: &str = "text-edit-enter-declined"; // ui-text-exempt: a trace event name, never displayed
const SAVE: &str = "text-edit-save"; // ui-text-exempt: a trace event name, never displayed
const SAVED: &str = "save-in-place"; // ui-text-exempt: a trace event name, never displayed
const LEFT_EDGE: &str = "edit-text-left-edge"; // ui-text-exempt: a trace event name, never displayed
const REGION: &str = "status-group:draft-note"; // ui-text-exempt: a trace region name, never displayed

/// See the module documentation.
pub struct TheDraftKeysDoWhatAWordProcessorDoes;

impl Check for TheDraftKeysDoWhatAWordProcessorDoes {
    fn name(&self) -> &'static str {
        "the_draft_keys_do_what_a_word_processor_does"
    }

    fn defect(&self) -> &'static str {
        "inside a text draft Ctrl+Z, Ctrl+Y and Ctrl+S do nothing, Ctrl+Backspace removes one \
         character, Tab types nothing, and a pasted line break vanishes without a word"
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

fn launch(ctx: &CheckContext) -> Result<(Session, ScriptedPointer, std::path::PathBuf)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let source = repo_fixture(FIXTURE, "Run fixtures/word-fragmented-lines.PROVENANCE.py.")?;
    let doc = ctx.out("draft-keys-source.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("draft-keys.trace.txt"));
    spec.pdf = Some(doc.clone());
    for (k, v) in [
        ctx.profile.diag_env,
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", INVOKE),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("draft-keys.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer, doc))
}

/// The session, its pointer and the trace path, for the steps below.
struct Drive<'a> {
    session: &'a Session,
    pointer: &'a ScriptedPointer,
    path: String,
    /// The profile's ui-rect trace event.
    ui_rect: &'static str,
}

impl Drive<'_> {
    fn key(&self, name: &str, mods: Option<&str>) -> Result<()> {
        self.pointer.key(self.session, None, name, mods)?;
        self.session.settle(15);
        Ok(())
    }

    fn typed(&self, text: &str) -> Result<()> {
        self.pointer.type_text(self.session, None, text)?;
        self.session.settle(15);
        Ok(())
    }

    /// The draft's length from the last `text-edit-typing` line.
    fn len(&self) -> Result<usize> {
        self.session
            .trace()?
            .events(TYPING)
            .last()
            .and_then(|l| l.get("len").and_then(|v| v.parse().ok()))
            .ok_or_else(|| Error::new(format!("no `{TYPING}` line carried a length.")))
    }

    /// The last raw line of `event`, if any.
    fn last(&self, event: &str) -> Result<Option<String>> {
        Ok(self
            .session
            .trace()?
            .events(event)
            .last()
            .map(|l| l.raw.clone()))
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer, doc) = launch(ctx)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let page = crate::fixture::page_geometry(&doc)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let d = Drive {
        session: &session,
        pointer: &pointer,
        path: session.trace_path().display().to_string(),
        ui_rect: ctx
            .profile
            .vocab
            .ui_rect_event
            .ok_or_else(|| Error::new("the profile names no ui-rect trace event."))?,
    };
    pointer.click(
        &session,
        mapping.doc_to_window(DocPoint::new(0, LINE.0, LINE.1))?,
    )?;
    session.settle(20);
    d.key("End", None)?;
    for step in [
        history,
        word_delete,
        tab_and_paste,
        arrows_and_enter,
        select_and_save,
    ] {
        if let Some(failure) = step(&d, report)? {
            return Ok(Some(failure));
        }
    }
    pointer.gone(&session)?;
    Ok(None)
}

/// ★ Two typed runs are one undo entry; Ctrl+Z takes them back inside the
/// draft and Ctrl+Y puts them back.
fn history(d: &Drive<'_>, report: &mut CheckReport) -> Result<Option<String>> {
    let start = d.len()?;
    d.typed("ad")?;
    d.typed("e")?;
    let typed = d.len()?;
    d.key("Z", Some("ctrl"))?;
    let undone = d.last(HISTORY)?;
    let after_undo = d.len()?;
    d.key("Y", Some("ctrl"))?;
    let after_redo = d.len()?;
    let ok = typed == start + 3
        && after_undo == start
        && after_redo == typed
        && undone.as_deref().is_some_and(|l| l.contains("owner=draft"));
    if !ok {
        return Ok(Some(format!(
            "★ typing then Ctrl+Z, Ctrl+Y did not step the draft: lengths {start} → {typed} \
             → {after_undo} → {after_redo} (want +3, back, forward); history: {undone:?}. \
             Trace: {}.",
            d.path
        )));
    }
    report.note(format!(
        "★ Ctrl+Z took back both typed runs ({typed} → {start}), Ctrl+Y restored them"
    ));
    Ok(None)
}

/// ★★ Ctrl+Backspace removes the whole word before the caret.
fn word_delete(d: &Drive<'_>, report: &mut CheckReport) -> Result<Option<String>> {
    let before = d.len()?;
    d.key("Backspace", Some("ctrl"))?;
    let after = d.len()?;
    if after + 1 >= before {
        return Ok(Some(format!(
            "★★ Ctrl+Backspace removed {} character(s), not a word ({before} → {after}). \
             Trace: {}.",
            before.saturating_sub(after),
            d.path
        )));
    }
    report.note(format!(
        "★★ Ctrl+Backspace removed a word ({before} → {after})"
    ));
    Ok(None)
}

/// ★★★ Tab types spaces and the status bar says so; a pasted line break
/// becomes a space and the status bar says so; typing still lands after both.
fn tab_and_paste(d: &Drive<'_>, report: &mut CheckReport) -> Result<Option<String>> {
    let before = d.len()?;
    d.key("Tab", None)?;
    let tab = d.last(TAB)?;
    let spaces: usize = tab
        .as_deref()
        .and_then(|l| l.split("spaces=").nth(1))
        .and_then(|v| v.split_whitespace().next())
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let tabbed = d.len()?;
    let tab_note = d.last(NOTE)?;
    d.pointer.paste(d.session, None, "a\nd")?;
    d.session.settle(20);
    let pasted = d.len()?;
    let paste = d.last(PASTE)?;
    let paste_note = d.last(NOTE)?;
    let trace = d.session.trace()?;
    let disclosed = trace
        .events(d.ui_rect)
        .any(|l| l.get("name") == Some(REGION));
    let ok = spaces >= 1
        && tabbed == before + spaces
        && tab_note
            .as_deref()
            .is_some_and(|l| l.contains("note=tab-as-spaces"))
        && pasted == tabbed + 3
        && paste.as_deref().is_some_and(|l| l.contains("joined=1"))
        && paste_note
            .as_deref()
            .is_some_and(|l| l.contains("note=lines-joined"))
        && disclosed;
    if !ok {
        return Ok(Some(format!(
            "★★★ Tab or a two-line paste went wrong: lengths {before} → {tabbed} → {pasted} \
             (want +{spaces} then +3); tab: {tab:?}; paste: {paste:?}; notes: {tab_note:?}, \
             {paste_note:?}; `{REGION}` drawn: {disclosed}. Trace: {}.",
            d.path
        )));
    }
    report.note(format!(
        "★★★ Tab typed {spaces} space(s), the paste joined its lines, and `{REGION}` said so"
    ));
    Ok(None)
}

/// ★★★★ Shift+Left extends a selection and Left drops it; Enter on a line
/// already on the page declines and keeps the text.
fn arrows_and_enter(d: &Drive<'_>, report: &mut CheckReport) -> Result<Option<String>> {
    let len = d.len()?;
    d.key("ArrowLeft", Some("shift"))?;
    d.key("ArrowLeft", Some("shift"))?;
    let grown = d.last(SELECT)?;
    d.key("ArrowLeft", None)?;
    let dropped = d.last(SELECT)?;
    let declines = d.session.trace()?.events(ENTER_DECLINED).count();
    let invoked = shell_trace(d.session)?.events(INVOKE_EVENT).count();
    if let Err(e) = d.key("Enter", None) {
        let shell = shell_trace(d.session)?;
        let Some(pressed) = shell.events(INVOKE_EVENT).nth(invoked) else {
            return Err(e);
        };
        return Ok(Some(format!(
            "★★★★ Enter inside the draft pressed a ribbon button: `{}`. An earlier Tab \
             walked egui's focus out of the draft; `canvas::textedit::claim_tab`, run from \
             the `app::keyclaim` plugin, is what keeps it in. Trace: {}.",
            pressed.raw, d.path
        )));
    }
    let declined = d.session.trace()?.events(ENTER_DECLINED).count() > declines;
    let after = d.len()?;
    let ok = grown.as_deref().is_some_and(|l| l.contains(" n=2"))
        && dropped
            .as_deref()
            .is_some_and(|l| l.contains("text-select none"))
        && declined
        && after == len;
    if !ok {
        return Ok(Some(format!(
            "★★★★ Shift+Left, Left or Enter misbehaved: after two Shift+Left: {grown:?}; \
             after Left: {dropped:?}; Enter declined: {declined}; length {len} → {after}. \
             Trace: {}.",
            d.path
        )));
    }
    report.note("★★★★ Shift+Left selected two characters, Left dropped them, Enter declined");
    Ok(None)
}

/// ★★★★★ Ctrl+A selects the whole draft; Ctrl+S commits it and saves the file
/// in place.
fn select_and_save(d: &Drive<'_>, report: &mut CheckReport) -> Result<Option<String>> {
    let len = d.len()?;
    d.key("A", Some("ctrl"))?;
    let selected = d.last(SELECT)?;
    let edits_before = d.session.trace()?.events(LEFT_EDGE).count();
    d.key("S", Some("ctrl"))?;
    d.session.settle(40);
    let save = d.last(SAVE)?;
    let saved = d.last(SAVED)?;
    let committed = d
        .session
        .trace()?
        .events(LEFT_EDGE)
        .skip(edits_before)
        .last()
        .map(|l| l.raw.clone());
    let want = format!("from=0 to={len} ");
    let ok = selected.as_deref().is_some_and(|l| l.contains(&want))
        && save.as_deref().is_some_and(|l| l.contains("in_place=1"))
        && committed
            .as_deref()
            .is_some_and(|l| l.contains("committed=yes"))
        && saved.as_deref().is_some_and(|l| l.contains("outcome=ok"));
    if !ok {
        return Ok(Some(format!(
            "★★★★★ Ctrl+A then Ctrl+S did not select, commit and save: select: {selected:?}; \
             save: {save:?}; commit: {committed:?}; saved: {saved:?}. Trace: {}.",
            d.path
        )));
    }
    report.note("★★★★★ Ctrl+A selected the draft; Ctrl+S committed it and saved the file");
    Ok(None)
}
