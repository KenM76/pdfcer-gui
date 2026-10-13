//! `a_copied_radio_pastes_into_its_group_and_stays_exclusive` and
//! `a_radio_button_joins_a_group_from_properties` — a radio button reaches
//! another group's membership by `Ctrl+Shift+V`, by Properties ▸ Radio group
//! and by renaming it to the group's name, judged in the saved file: one
//! field, every member a `/Kids` entry naming it as `/Parent`, every member a
//! distinct on-state, and, re-opened, only one member on at a time.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/radio_group.md`.

use super::dimdrive::{
    Fixture, click_on, launch_path, launch_with, press, press_on_tab, ui_rect_event,
};
use super::savedannots::{last_definitions, numbers_after, references_after};
use super::stroke_style::into_view;
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const FIXTURE: Fixture = Fixture {
    file: "all-field-kinds.pdf",
    method: "Rebuild it with `python fixtures/all-field-kinds.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 420.0,
        height_pt: 620.0,
    },
};
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
const CLIPBOARD_ENV: &str = "PDFCER_DIAG_CLIPBOARD_DIR"; // ui-text-exempt: an environment variable name

/// The fixture's one radio group: two members, on-states `A` and `B`.
const GROUP: &str = "RadioGroup"; // ui-text-exempt: a field name in the fixture
/// Inside member `A`, `[170 452 186 468]`.
const ON_A: (f64, f64) = (178.0, 460.0);
/// Empty page, left of every field; where the paste is aimed.
const PASTE_AT: (f64, f64) = (60.0, 560.0);
/// Two empty sweeps, left of every field, for new radio buttons.
const SWEEP_ONE: ((f64, f64), (f64, f64)) = ((40.0, 480.0), (56.0, 464.0));
const SWEEP_TWO: ((f64, f64), (f64, f64)) = ((40.0, 420.0), (56.0, 404.0));

const FILE_TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name
const SAVE_COPY: &str = "ribbon.item.file.save_copy"; // ui-text-exempt: a trace region name
const EDIT_TAB: &str = "ribbon.tab.edit"; // ui-text-exempt: a trace region name
const RADIO_TOOL: &str = "ribbon.item.edit.form_radio_button"; // ui-text-exempt: a trace region name
const ACCEPT: &str = "dialog.form_field.accept"; // ui-text-exempt: a trace region name
/// Mirror `panels::properties::radiogroup` and `::formfield`.
const GROUP_NAME: &str = "properties.form_field.radio_group.name"; // ui-text-exempt: a trace region name
const GROUP_MOVE: &str = "properties.form_field.radio_group.move"; // ui-text-exempt: a trace region name
const RENAME_NAME: &str = "properties.form_field.rename_name"; // ui-text-exempt: a trace region name
const RENAME: &str = "properties.form_field.rename"; // ui-text-exempt: a trace region name

const SELECTED: &str = "form-field-selected"; // ui-text-exempt: a trace event name
const COPIED: &str = "fieldclip-copy"; // ui-text-exempt: a trace event name
const PASTED: &str = "radio-paste-applied"; // ui-text-exempt: a trace event name
const MOVED: &str = "radio-move-applied"; // ui-text-exempt: a trace event name
const OPENED: &str = "form-field-open"; // ui-text-exempt: a trace event name
const TURNED: &str = "form-button"; // ui-text-exempt: a trace event name
const SAVED: &str = "save-copy"; // ui-text-exempt: a trace event name

/// See the module documentation.
pub struct ACopiedRadioPastesIntoItsGroupAndStaysExclusive;

impl Check for ACopiedRadioPastesIntoItsGroupAndStaysExclusive {
    fn name(&self) -> &'static str {
        "a_copied_radio_pastes_into_its_group_and_stays_exclusive"
    }

    fn defect(&self) -> &'static str {
        "Ctrl+Shift+V over a copied radio button adds nothing (the copy's export value is \
         already the group's), or adds a button outside the group, so radio buttons cannot be \
         made by pasting and the result is not mutually exclusive"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let saved = ctx.out("radio-paste.saved.pdf");
        let reopened = ctx.out("radio-paste.reopened.pdf");
        let filled = ctx.out("radio-paste.filled.pdf");
        for p in [&saved, &reopened, &filled] {
            let _ = std::fs::remove_file(p);
        }
        let env = [
            (SAVE_PATH_ENV, saved.display().to_string()),
            (
                CLIPBOARD_ENV,
                ctx.out("radio-paste-clipboard").display().to_string(),
            ),
        ];
        let first = in_session(
            ctx,
            &mut report,
            None,
            "mode.edit,file.properties",
            &env,
            |r, s, p| paste_into_group(ctx, r, s, p, &saved),
        );
        let outcome = match first {
            Ok(None) => std::fs::copy(&saved, &reopened)
                .map_err(|e| Error::new(format!("copying {}: {e}", saved.display())))
                .and_then(|_| {
                    let env = [(SAVE_PATH_ENV, filled.display().to_string())];
                    in_session(
                        ctx,
                        &mut report,
                        Some(reopened),
                        "mode.read",
                        &env,
                        |r, s, p| fill_one_then_another(ctx, r, s, p, &filled),
                    )
                }),
            other => other,
        };
        finish(report, outcome)
    }
}

/// See the module documentation.
pub struct ARadioButtonJoinsAGroupFromProperties;

impl Check for ARadioButtonJoinsAGroupFromProperties {
    fn name(&self) -> &'static str {
        "a_radio_button_joins_a_group_from_properties"
    }

    fn defect(&self) -> &'static str {
        "a radio button cannot be put into another radio group from Properties, neither by its \
         Radio group row nor by renaming it to the group's name, so separately placed radio \
         buttons can never be made mutually exclusive"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let saved = ctx.out("radio-join.saved.pdf");
        let _ = std::fs::remove_file(&saved);
        let env = [(SAVE_PATH_ENV, saved.display().to_string())];
        let outcome = in_session(
            ctx,
            &mut report,
            None,
            "mode.edit,file.properties,edit.form_radio_button",
            &env,
            |r, s, p| join_from_properties(ctx, r, s, p, &saved),
        );
        finish(report, outcome)
    }
}

/// Launch on the fixture (or on `doc`), run `body`, park the pointer.
fn in_session(
    ctx: &CheckContext,
    report: &mut CheckReport,
    doc: Option<std::path::PathBuf>,
    invoke: &str,
    env: &[(&str, String)],
    body: impl FnOnce(&mut CheckReport, &Session, &ScriptedPointer) -> Result<Option<String>>,
) -> Result<Option<String>> {
    let (session, pointer) = match doc {
        Some(path) => launch_path(ctx, report, path, invoke, "radio-reopened", env)?,
        None => launch_with(ctx, report, &FIXTURE, invoke, "radio-group", env)?,
    };
    let outcome = body(report, &session, &pointer);
    let parked = pointer.gone(&session);
    match outcome? {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

fn finish(report: CheckReport, outcome: Result<Option<String>>) -> CheckReport {
    match outcome {
        Ok(Some(failure)) => report.fail(failure),
        Ok(None) => report.pass(),
        Err(why) => report.from_error(&why),
    }
}

/// The last line named `event` after `mark`, or the failure sentence.
fn after(
    session: &Session,
    event: &str,
    mark: usize,
    what: &str,
) -> Result<std::result::Result<TraceLine, String>> {
    let trace = session.trace()?;
    Ok(trace.last_after(event, mark).cloned().ok_or_else(|| {
        format!(
            "{what} and no `{event}` followed. Trace: {}.",
            session.trace_path().display()
        )
    }))
}

macro_rules! step {
    ($e:expr) => {
        match $e? {
            Ok(line) => line,
            Err(why) => return Ok(Some(why)),
        }
    };
}

fn window_at(
    ctx: &CheckContext,
    session: &Session,
    at: (f64, f64),
) -> Result<crate::coords::WindowPoint> {
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, FIXTURE.page, 0)?;
    mapping.doc_to_window(DocPoint::new(0, at.0, at.1))
}

fn paste_into_group(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    saved: &std::path::Path,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, ON_A)?;
    let selected = step!(after(session, SELECTED, mark, "member A was clicked"));
    if selected.get("field") != Some(GROUP) {
        return Ok(Some(format!(
            "a click on member A selected `{}`, not {GROUP}.",
            selected.raw
        )));
    }
    let mark = session.trace()?.mark();
    pointer.key(session, None, "C", Some("ctrl"))?;
    session.settle(20);
    report.note(step!(after(session, COPIED, mark, "Ctrl+C was pressed")).raw);
    pointer.hover(session, window_at(ctx, session, PASTE_AT)?)?;
    session.settle(10);
    let mark = session.trace()?.mark();
    pointer.key(session, None, "V", Some("ctrl+shift"))?;
    session.settle(30);
    let pasted = step!(after(session, PASTED, mark, "Ctrl+Shift+V was pressed"));
    report.note(pasted.raw.clone());
    if let Some(failure) = save(ctx, session, pointer)? {
        return Ok(Some(failure));
    }
    let members = match group_in(saved, report)? {
        Ok(m) => m,
        Err(why) => return Ok(Some(why)),
    };
    Ok(judge_members(&members, 3))
}

fn fill_one_then_another(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    filled: &std::path::Path,
) -> Result<Option<String>> {
    for (at, what) in [(ON_A, "member A"), (PASTE_AT, "the pasted member")] {
        let mark = session.trace()?.mark();
        click_on(ctx, session, pointer, &FIXTURE, 0, at)?;
        report.note(step!(after(session, TURNED, mark, &format!("{what} was clicked"))).raw);
    }
    if let Some(failure) = save(ctx, session, pointer)? {
        return Ok(Some(failure));
    }
    let members = match group_in(filled, report)? {
        Ok(m) => m,
        Err(why) => return Ok(Some(why)),
    };
    let on: Vec<&Member> = members.iter().filter(|m| m.appearance != "Off").collect();
    let pasted = members.iter().find(|m| m.rect.0 < 100.0);
    let value = group_value(filled)?;
    match (on.as_slice(), pasted) {
        ([only], Some(p)) if only.num == p.num => {
            Ok((value.as_deref() != p.on_state.as_deref()).then(|| {
                format!(
                    "the pasted member is the one on, but the group's /V is {value:?}, not its state {:?}.",
                    p.on_state
                )
            }))
        }
        _ => Ok(Some(format!(
            "after clicking member A and then the pasted member, the re-opened group should \
             have exactly the pasted member on; it has {} on: {members:?}.",
            on.len()
        ))),
    }
}

fn join_from_properties(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    saved: &std::path::Path,
) -> Result<Option<String>> {
    // --- a new radio button, moved into the group by its Radio group row.
    if let Some(failure) = place(ctx, report, session, pointer, SWEEP_ONE)? {
        return Ok(Some(failure));
    }
    let mark = session.trace()?.mark();
    type_without_enter(ctx, session, pointer, GROUP_NAME, GROUP)?;
    into_view(ctx, session, pointer, GROUP_MOVE)?;
    press(ctx, session, pointer, GROUP_MOVE)?;
    session.settle(30);
    let moved = step!(after(session, MOVED, mark, "Move to group was pressed"));
    report.note(format!("by the row: {}", moved.raw));

    // --- another, joined by renaming it to the group's name.
    press_on_tab(ctx, session, pointer, RADIO_TOOL, EDIT_TAB)?;
    if let Some(failure) = place(ctx, report, session, pointer, SWEEP_TWO)? {
        return Ok(Some(failure));
    }
    let mark = session.trace()?.mark();
    type_without_enter(ctx, session, pointer, RENAME_NAME, GROUP)?;
    into_view(ctx, session, pointer, RENAME)?;
    press(ctx, session, pointer, RENAME)?;
    session.settle(30);
    let renamed = step!(after(
        session,
        MOVED,
        mark,
        "Rename to the group's name was pressed"
    ));
    report.note(format!("by rename: {}", renamed.raw));

    if let Some(failure) = save(ctx, session, pointer)? {
        return Ok(Some(failure));
    }
    let members = match group_in(saved, report)? {
        Ok(m) => m,
        Err(why) => return Ok(Some(why)),
    };
    if let Some(failure) = judge_members(&members, 4) {
        return Ok(Some(failure));
    }
    leftover_fields(saved)
}

/// Drag a new radio button out and accept the window's defaults.
fn place(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    sweep: ((f64, f64), (f64, f64)),
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    pointer.drag(
        session,
        window_at(ctx, session, sweep.0)?,
        window_at(ctx, session, sweep.1)?,
        8,
    )?;
    session.settle(30);
    report.note(
        step!(after(
            session,
            OPENED,
            mark,
            "a radio button was dragged out"
        ))
        .raw,
    );
    press(ctx, session, pointer, ACCEPT)?;
    session.settle(40);
    Ok(None)
}

/// Click `region`, replace its text with `text`, and leave focus there.
fn type_without_enter(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
    text: &str,
) -> Result<()> {
    into_view(ctx, session, pointer, region)?;
    let vp = declared_in(&session.trace()?, ui_rect_event(ctx)?, region).and_then(|(_, vp)| vp);
    press(ctx, session, pointer, region)?;
    pointer.key(session, vp.as_deref(), "A", Some("ctrl"))?;
    pointer.type_text(session, vp.as_deref(), text)?;
    session.settle(20);
    Ok(())
}

fn save(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    press_on_tab(ctx, session, pointer, SAVE_COPY, FILE_TAB)?;
    session.settle(40);
    Ok(after(session, SAVED, mark, "Save a copy was pressed")?.err())
}

/// The failure sentence when `/Fields` still lists a field other than the
/// fixture's: a button moved out of its own group leaves no empty field.
fn leftover_fields(path: &std::path::Path) -> Result<Option<String>> {
    let bytes = std::fs::read(path)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", path.display())))?;
    let text = String::from_utf8_lossy(&bytes);
    let defs = last_definitions(&text);
    let listed: Vec<u32> = defs
        .values()
        .find_map(|b| {
            b.contains("/Fields")
                .then(|| references_after(b, "/Fields"))
                .flatten()
        })
        .unwrap_or_default();
    let fixture_fields = fixture_field_names()?;
    let strays: Vec<String> = listed
        .iter()
        .filter_map(|n| defs.get(n).map(|b| (n, title(b))))
        .filter(|(_, t)| !fixture_fields.contains(t))
        .map(|(n, t)| format!("{n} {t}"))
        .collect();
    Ok((!strays.is_empty()).then(|| {
        format!(
            "after both buttons joined {GROUP}, /Fields still lists fields the fixture did not have: {strays:?}."
        )
    }))
}

/// `/T (name)` of a field body, or empty.
fn title(body: &str) -> String {
    body.split("/T (")
        .nth(1)
        .and_then(|r| r.split(')').next())
        .unwrap_or("")
        .to_owned()
}

fn fixture_field_names() -> Result<Vec<String>> {
    let path = crate::fixture::workspace_root()
        .join("fixtures")
        .join(FIXTURE.file);
    let bytes = std::fs::read(&path)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", path.display())))?;
    let text = String::from_utf8_lossy(&bytes);
    Ok(last_definitions(&text).values().map(|b| title(b)).collect())
}

/// The `/V` name of the saved field `/T (RadioGroup)`, if it has one.
fn group_value(path: &std::path::Path) -> Result<Option<String>> {
    let bytes = std::fs::read(path)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", path.display())))?;
    let text = String::from_utf8_lossy(&bytes);
    let title = format!("/T ({GROUP})");
    Ok(last_definitions(&text)
        .values()
        .find(|b| b.contains(&title))
        .and_then(|b| b.split("/V /").nth(1))
        .and_then(|r| {
            r.split(|c: char| c.is_whitespace() || c == '/' || c == '>')
                .next()
        })
        .map(str::to_owned))
}

/// One member of the saved group, as the file finally states it.
#[derive(Debug)]
struct Member {
    num: u32,
    parent: Option<u32>,
    on_state: Option<String>,
    appearance: String,
    rect: (f64, f64),
}

/// The members of the last definition of the field `/T (RadioGroup)`.
fn group_in(
    path: &std::path::Path,
    report: &mut CheckReport,
) -> Result<std::result::Result<Vec<Member>, String>> {
    report.artifact(path.to_path_buf());
    let bytes = std::fs::read(path)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", path.display())))?;
    let text = String::from_utf8_lossy(&bytes);
    let defs = last_definitions(&text);
    let title = format!("/T ({GROUP})");
    let fields: Vec<(&u32, &&str)> = defs.iter().filter(|(_, b)| b.contains(&title)).collect();
    let [(field, body)] = fields.as_slice() else {
        return Ok(Err(format!(
            "the saved file should define one field {title}; it defines {}.",
            fields.len()
        )));
    };
    let Some(kids) = references_after(body, "/Kids") else {
        return Ok(Err(format!("field {field} has no /Kids array: {body}")));
    };
    let members: Vec<Member> = kids
        .iter()
        .map(|k| member(*k, defs.get(k).copied().unwrap_or("")))
        .collect();
    for m in &members {
        report.note(format!("member {m:?} of field {field}"));
    }
    if let Some(stray) = members.iter().find(|m| m.parent != Some(**field)) {
        return Ok(Err(format!(
            "member {} does not name field {field} as its /Parent: {stray:?}.",
            stray.num
        )));
    }
    Ok(Ok(members))
}

fn member(num: u32, body: &str) -> Member {
    let parent = body
        .split("/Parent ")
        .nth(1)
        .and_then(|r| r.split_whitespace().next())
        .and_then(|n| n.parse().ok());
    let appearance = body
        .split("/AS /")
        .nth(1)
        .and_then(|r| {
            r.split(|c: char| c.is_whitespace() || c == '/' || c == '>')
                .next()
        })
        .unwrap_or("")
        .to_owned();
    let on_state = body
        .split("/N <<")
        .nth(1)
        .and_then(|r| r.split(">>").next())
        .and_then(|d| {
            d.split('/')
                .filter_map(|w| w.split_whitespace().next())
                .find(|name| !name.is_empty() && *name != "Off")
                .map(str::to_owned)
        });
    let rect = numbers_after(body, "/Rect")
        .filter(|r| r.len() == 4)
        .map_or((f64::NAN, f64::NAN), |r| (r[0], r[1]));
    Member {
        num,
        parent,
        on_state,
        appearance,
        rect,
    }
}

fn judge_members(members: &[Member], want: usize) -> Option<String> {
    if members.len() != want {
        return Some(format!(
            "the group should have {want} members; the saved file gives it {}: {members:?}.",
            members.len()
        ));
    }
    let mut states: Vec<&str> = members
        .iter()
        .filter_map(|m| m.on_state.as_deref())
        .collect();
    states.sort_unstable();
    states.dedup();
    (states.len() != want).then(|| {
        format!(
            "the {want} members should turn on as {want} distinct states, or two select \
             together; the saved file gives {states:?}: {members:?}."
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(num: u32, state: &str) -> Member {
        Member {
            num,
            parent: Some(18),
            on_state: Some(state.to_owned()),
            appearance: "Off".to_owned(),
            rect: (0.0, 0.0),
        }
    }

    #[test]
    fn two_members_sharing_a_state_are_named() {
        let members = [m(16, "A"), m(17, "B"), m(33, "A")];
        assert!(judge_members(&members, 3).unwrap().contains("distinct"));
        let members = [m(16, "A"), m(17, "B"), m(33, "A1")];
        assert_eq!(judge_members(&members, 3), None);
        assert!(
            judge_members(&members[..2], 3)
                .unwrap()
                .contains("3 members")
        );
    }

    #[test]
    fn a_member_is_read_from_its_saved_body() {
        let body = "<< /Parent 18 0 R /Rect [170 452 186 468] /AS /A \
                    /AP << /N << /A 5 0 R /Off 6 0 R >> >> >>";
        let read = member(16, body);
        assert_eq!(read.parent, Some(18));
        assert_eq!(read.on_state.as_deref(), Some("A"));
        assert_eq!(read.appearance, "A");
        assert_eq!(read.rect, (170.0, 452.0));
    }
}
