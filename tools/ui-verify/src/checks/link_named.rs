//! A named destination is picked from a list, in Add link and in Properties,
//! instead of typed.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/link_named.md`.

use super::dimdrive::{Fixture, click_on, press, run_on, ui_rect_event};
use super::links::drew_after;
use super::stroke_style::into_view;
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const ADD_INVOKE: &str = "mode.edit,edit.insert_link";
const EDIT_INVOKE: &str = "mode.edit,file.properties";
const FIXTURE: Fixture = Fixture {
    file: "link-named.pdf",
    method: "Rebuild it with `python fixtures/link-named.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 400.0,
    },
};
const ARMED: &str = "place-armed"; // ui-text-exempt: a trace event name, never displayed
const ADDED: &str = "add-link-applied"; // ui-text-exempt: a trace event name, never displayed
const SELECTED: &str = "annot-select"; // ui-text-exempt: a trace event name, never displayed
const RETARGETED: &str = "set-link-target-applied"; // ui-text-exempt: a trace event name, never displayed
const ADD_WINDOW: &str = "dialog:add-link";
const ADD_NAMED: &str = "add-link.named";
const ADD_NAMES: &str = "add-link.names";
const ADD_BUTTON: &str = "add-link.add";
const PROPS_NAMES: &str = "properties.link.names";
const APPLY_TARGET: &str = "properties.link.apply-target";
/// The fixture's two names in byte order: one from each namespace, so a list
/// reading only one of them has no second row.
const NAMES: [&str; 2] = ["appendix", "chapter-two"];
/// The second row's name, as a link's target words it.
const PICKED_WORD: &str = "target=named:chapter-two";
const DRAG_FROM: (f64, f64) = (100.0, 250.0);
const DRAG_TO: (f64, f64) = (260.0, 300.0);
/// Inside the fixture's link, `/Rect [100 100 200 140]`.
const ON_LINK: (f64, f64) = (150.0, 120.0);

/// Add link's Named choice lists the document's names, and a pick writes one.
pub struct ANamedDestinationIsPickedForANewLink;
/// Properties' Named choice lists the names, and a pick re-points the link.
pub struct ANamedDestinationIsPickedForASelectedLink;

impl Check for ANamedDestinationIsPickedForANewLink {
    fn name(&self) -> &'static str {
        "a_named_destination_is_picked_for_a_new_link"
    }

    fn defect(&self) -> &'static str {
        "Add link's Named choice does not list the document's named destinations, or \
         picking one does not write a link to it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, ADD_INVOKE, "link-named-add", add)
    }
}

impl Check for ANamedDestinationIsPickedForASelectedLink {
    fn name(&self) -> &'static str {
        "a_named_destination_is_picked_for_a_selected_link"
    }

    fn defect(&self) -> &'static str {
        "Properties' Named choice for a selected link does not list the document's named \
         destinations, or picking one does not re-point the link"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &FIXTURE,
            EDIT_INVOKE,
            "link-named-props",
            repoint,
        )
    }
}

/// `None` when the list `names` drew exactly the fixture's rows after `mark`;
/// otherwise the failure.
fn listed(session: &Session, names: &str, mark: usize) -> Result<Option<String>> {
    let mut drawn = Vec::new();
    for i in 0..=NAMES.len() {
        drawn.push(drew_after(session, &format!("{names}.{i}"), mark)?);
    }
    let wanted: Vec<bool> = (0..=NAMES.len()).map(|i| i < NAMES.len()).collect();
    Ok((drawn != wanted).then(|| {
        format!(
            "the Named choice must list exactly the fixture's {} names ({}), one in each \
             namespace, as `{names}.0`..; rows drawn: {drawn:?}. Trace: {}.",
            NAMES.len(),
            NAMES.join(", "),
            session.trace_path().display()
        )
    }))
}

fn add(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    if !trace
        .last(ARMED)
        .is_some_and(|l| l.get("kind") == Some("Link"))
    {
        return Ok(Some(format!(
            "Add link was invoked and no `{ARMED} kind=Link` followed. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, FIXTURE.page, 0)?;
    let mark = trace.mark();
    let from = mapping.doc_to_window(DocPoint::new(0, DRAG_FROM.0, DRAG_FROM.1))?;
    let to = mapping.doc_to_window(DocPoint::new(0, DRAG_TO.0, DRAG_TO.1))?;
    pointer.drag(session, from, to, 10)?;
    session.settle(40);
    if !drew_after(session, ADD_WINDOW, mark)? {
        return Ok(Some(format!(
            "a box was dragged with Add link armed and `{ADD_WINDOW}` never drew. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, ADD_NAMED)?;
    session.settle(20);
    if let Some(failure) = listed(session, ADD_NAMES, mark)? {
        return Ok(Some(failure));
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, &format!("{ADD_NAMES}.1"))?;
    session.settle(20);
    // Evidence for a reader, not an oracle: an immediate viewport never
    // answers a screenshot, so a refusal is noted and passed.
    let vp = declared_in(&session.trace()?, ui_rect_event(ctx)?, ADD_WINDOW).and_then(|(_, vp)| vp);
    let shot = ctx.out("link-named-add-list.png");
    match pointer.screenshot_in(session, vp.as_deref(), &shot) {
        Ok(()) => {
            report.artifact(shot);
        }
        Err(why) => {
            report.note(format!("no picture of Add link: {why}"));
        }
    }
    // Add is absent while the name is empty: a pick that sets nothing leaves
    // only the refusal sentence.
    if !drew_after(session, ADD_BUTTON, mark)? {
        return Ok(Some(format!(
            "`{ADD_NAMES}.1` was pressed and `{ADD_BUTTON}` did not draw: the pick set no \
             name. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, ADD_BUTTON)?;
    session.settle(40);
    let shot = ctx.out("link-named-add-after.png");
    pointer.screenshot(session, &shot)?;
    report.artifact(shot);
    let Some(added) = session.trace()?.last_after(ADDED, mark).cloned() else {
        return Ok(Some(format!(
            "a name was picked and Add pressed, and no `{ADDED}` followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("added: `{}`", added.raw));
    Ok((!added.raw.contains(PICKED_WORD)).then(|| {
        format!(
            "`{}` must carry `{PICKED_WORD}`, the second listed name.",
            added.raw
        )
    }))
}

fn repoint(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, ON_LINK)?;
    session.settle(20);
    if !session
        .trace()?
        .last_after(SELECTED, mark)
        .is_some_and(|l| l.get("subtype") == Some("Link"))
    {
        return Ok(Some(format!(
            "a click inside the link traced no `{SELECTED}` with subtype=Link. Trace: {}.",
            session.trace_path().display()
        )));
    }
    // The link goes to a name, so its form opens on the Named choice.
    if let Some(failure) = listed(session, PROPS_NAMES, mark)? {
        return Ok(Some(failure));
    }
    let row = format!("{PROPS_NAMES}.1");
    into_view(ctx, session, pointer, &row)?;
    press(ctx, session, pointer, &row)?;
    session.settle(20);
    into_view(ctx, session, pointer, APPLY_TARGET)?;
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, APPLY_TARGET)?;
    session.settle(30);
    let Some(retargeted) = session.trace()?.last_after(RETARGETED, mark).cloned() else {
        return Ok(Some(format!(
            "a name was picked and Apply destination pressed, and no `{RETARGETED}` \
             followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("re-pointed: `{}`", retargeted.raw));
    Ok((!retargeted.raw.contains(PICKED_WORD)).then(|| {
        format!(
            "`{}` must carry `{PICKED_WORD}`: the link went to `appendix` and the second \
             listed name was picked.",
            retargeted.raw
        )
    }))
}
