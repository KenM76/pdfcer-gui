//! `the_properties_delete_buttons_take_every_selected_field` — a text field
//! and one box of a two-box radio group selected, then the Properties panel's
//! *Delete selected boxes* and *Delete fields* buttons pressed in turn: the
//! first takes the two boxes and leaves the radio's other box, the second
//! takes both fields whole, each as one undo step, judged in saved copies.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/field_delete_panel.md`.

use super::dimdrive::{click_on, launch_with};
use super::field_delete_many::{COMBO, FIXTURE, LISTED, RADIO_A, RADIO_B, Spot, TEXT, listed};
use super::multiselect::{after, save, shift_click, step};
use crate::checks::{Check, CheckContext, driving};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
const TOGGLED: &str = "form-field-toggled"; // ui-text-exempt: a trace event name
const BOXES_APPLIED: &str = "delete-widgets-applied"; // ui-text-exempt: a trace event name
const FIELDS_APPLIED: &str = "delete-fields-applied"; // ui-text-exempt: a trace event name
/// Mirrors `panels::properties::fielddelete::REGION_DELETE_BOXES`.
const BOXES: &str = "properties.form_field.delete_boxes"; // ui-text-exempt: a trace region name
/// Mirrors `panels::properties::fielddelete::REGION_DELETE_FIELDS`.
const FIELDS: &str = "properties.form_field.delete_fields"; // ui-text-exempt: a trace region name
/// The Properties panel's body, which scrolls.
const PANEL_BODY: &str = "dock.body.file.properties"; // ui-text-exempt: a trace region name
/// Wheel turns tried before a button below the fold counts as unreachable.
const MAX_SCROLL: usize = 12;

/// See the module documentation.
pub struct ThePropertiesDeleteButtonsTakeEverySelectedField;

impl Check for ThePropertiesDeleteButtonsTakeEverySelectedField {
    fn name(&self) -> &'static str {
        "the_properties_delete_buttons_take_every_selected_field"
    }

    fn defect(&self) -> &'static str {
        "with several form fields selected, the Properties panel's delete buttons delete only \
         the primary one, or take one Undo per field to bring them back"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let saves = ["boxes", "boxes-undone", "fields", "fields-undone"]
            .map(|s| ctx.out(&format!("field-delete-panel.{s}.pdf")));
        for p in &saves {
            let _ = std::fs::remove_file(p);
        }
        let joined: Vec<String> = saves.iter().map(|p| p.display().to_string()).collect();
        let env = [(SAVE_PATH_ENV, joined.join(";"))];
        let outcome = launch_with(
            ctx,
            &mut report,
            &FIXTURE,
            "mode.edit,file.properties",
            "field-delete-panel",
            &env,
        )
        .and_then(|(session, pointer)| {
            let outcome = drive(ctx, &mut report, &session, &pointer, &saves);
            let parked = pointer.gone(&session);
            match outcome? {
                Some(failure) => Ok(Some(failure)),
                None => parked.map(|_| None),
            }
        });
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    saves: &[std::path::PathBuf; 4],
) -> Result<Option<String>> {
    // The boxes button: TextOne and RadioGroup #0 go, #1 stays.
    if let Some(why) = select(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let line = step!(press(ctx, session, pointer, BOXES, BOXES_APPLIED));
    report.note(format!("boxes: `{}`", line.raw));
    if let Some(why) = two_in_one_step(&line, BOXES) {
        return Ok(Some(why));
    }
    let kept = [
        (TEXT, false),
        (RADIO_A, false),
        (RADIO_B, true),
        (COMBO, true),
    ];
    if let Some(why) = save_and_list(ctx, session, pointer, &saves[0], &kept, LISTED - 2)? {
        return Ok(Some(format!("after `{BOXES}`, {why}")));
    }
    if let Some(why) = undo_restores(ctx, session, pointer, &saves[1])? {
        return Ok(Some(why));
    }

    // The fields button: both fields go whole, both radio boxes included.
    if let Some(why) = select(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let line = step!(press(ctx, session, pointer, FIELDS, FIELDS_APPLIED));
    report.note(format!("fields: `{}`", line.raw));
    if let Some(why) = two_in_one_step(&line, FIELDS) {
        return Ok(Some(why));
    }
    let gone = [
        (TEXT, false),
        (RADIO_A, false),
        (RADIO_B, false),
        (COMBO, true),
    ];
    if let Some(why) = save_and_list(ctx, session, pointer, &saves[2], &gone, LISTED - 3)? {
        return Ok(Some(format!("after `{FIELDS}`, {why}")));
    }
    undo_restores(ctx, session, pointer, &saves[3])
}

/// TextOne by a click, then RadioGroup's first box by Shift-click.
fn select(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    click_on(ctx, session, pointer, &FIXTURE, 0, TEXT.2)?;
    session.settle(20);
    let mark = session.trace()?.mark();
    shift_click(ctx, session, pointer, &FIXTURE, RADIO_A.2)?;
    let toggled = step!(after(
        session,
        TOGGLED,
        mark,
        "RadioGroup #0 was Shift-clicked"
    ));
    session.settle(20);
    Ok((toggled.get("before") != Some("1")).then(|| {
        format!(
            "Shift-clicking RadioGroup #0 with TextOne selected traced `{}`.",
            toggled.raw
        )
    }))
}

/// Click the panel button `region` and return the `event` line it traces.
fn press(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
    event: &str,
) -> Result<std::result::Result<TraceLine, String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("no ui-rect event in this profile"))?;
    let find = |what: &str| -> Result<LRect> {
        driving::declared(&session.trace()?, ui_rect, what).ok_or_else(|| {
            Error::new(format!(
                "no `{what}` region with two fields selected and Properties open. Trace: {}.",
                session.trace_path().display()
            ))
        })
    };
    // The buttons sit at the foot of a long panel; a region below the fold is
    // published at its content position, outside the panel.
    let panel = find(PANEL_BODY)?;
    let mut rect = find(region)?;
    for _ in 0..MAX_SCROLL {
        if panel.contains_rect(rect) {
            break;
        }
        pointer.wheel(session, WindowPoint::centre_of(panel), -5.0, false)?;
        session.settle(10);
        rect = find(region)?;
    }
    if !panel.contains_rect(rect) {
        return Ok(Err(format!(
            "`{region}` is still outside the Properties panel ({rect:?} against {panel:?}) after {MAX_SCROLL} wheel turns."
        )));
    }
    let mark = session.trace()?.mark();
    pointer.click(session, WindowPoint::centre_of(rect))?;
    session.settle(40);
    after(session, event, mark, &format!("`{region}` was clicked"))
}

fn two_in_one_step(line: &TraceLine, region: &str) -> Option<String> {
    (line.get("deleted") != Some("2/2") || line.get("one_step") != Some("true")).then(|| {
        format!(
            "`{region}` over two selected fields traced `{}`: deleted=2/2 one_step=true was \
             wanted.",
            line.raw
        )
    })
}

fn save_and_list(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    path: &std::path::Path,
    want: &[(Spot, bool)],
    count: usize,
) -> Result<Option<String>> {
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    listed(path, want, count)
}

/// One Undo, saved to `path`, lists every box again.
fn undo_restores(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    path: &std::path::Path,
) -> Result<Option<String>> {
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(40);
    let all = [TEXT, RADIO_A, RADIO_B, COMBO].map(|b| (b, true));
    Ok(save_and_list(ctx, session, pointer, path, &all, LISTED)?
        .map(|why| format!("after ONE Undo, {why} The deletes were not one undo step.")))
}
