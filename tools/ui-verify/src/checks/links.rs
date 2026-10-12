//! A link is drawn on the canvas and given a web address, and a selected
//! link is re-pointed and loses its border from Properties.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/links.md`.

use super::dimdrive::{Fixture, click_on, press, run_on};
use super::stroke_style::{into_view, type_into};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const ADD_INVOKE: &str = "mode.edit,edit.insert_link";
const EDIT_INVOKE: &str = "mode.edit,file.properties";
const REVIEW_INVOKE: &str = "mode.review,file.properties";
const FOLLOWED: &str = "link-click"; // ui-text-exempt: a trace event name, never displayed
const FIXTURE: Fixture = Fixture {
    file: "link-edit.pdf",
    method: "Rebuild it with `python fixtures/link-edit.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 400.0,
    },
};
const ADDED: &str = "add-link-applied"; // ui-text-exempt: a trace event name, never displayed
const SELECTED: &str = "annot-select"; // ui-text-exempt: a trace event name, never displayed
const RETARGETED: &str = "set-link-target-applied"; // ui-text-exempt: a trace event name, never displayed
const REBORDERED: &str = "set-link-border-applied"; // ui-text-exempt: a trace event name, never displayed
/// Mirror `dialogs::link`.
const ADD_WEB: &str = "add-link.web";
const ADD_FIELD: &str = "add-link.field";
const ADD_BORDER: &str = "add-link.border";
const ADD_BUTTON: &str = "add-link.add";
/// Mirror `panels::properties::link`.
const PROPS_WEB: &str = "properties.link.web";
const PROPS_FIELD: &str = "properties.link.field";
const PROPS_BORDER: &str = "properties.link.border";
const APPLY_TARGET: &str = "properties.link.apply-target";
const APPLY_BORDER: &str = "properties.link.apply-border";
/// The address typed, and the trace's spelling of it.
const ADDRESS: &str = "https://example.com/drawings";
const ADDRESS_WORD: &str = "target=uri:https://example.com/drawings";
/// The box dragged for the new link, page space: clear of the fixture's
/// link at `[100 180 300 220]`.
const DRAG_FROM: (f64, f64) = (100.0, 300.0);
const DRAG_TO: (f64, f64) = (260.0, 340.0);
/// Inside the fixture's link.
const ON_LINK: (f64, f64) = (200.0, 200.0);
/// The fixture link's top-right corner, where its grip is centred, and where
/// the drag lands.
const LINK_CORNER: (f64, f64) = (300.0, 220.0);
const CORNER_TO: (f64, f64) = (340.0, 250.0);
const RESIZED: &str = "resize-annotation-applied"; // ui-text-exempt: a trace event name, never displayed
const ARMED: &str = "place-armed"; // ui-text-exempt: a trace event name, never displayed
const RECT: &str = "ui-rect"; // ui-text-exempt: a trace event name, never displayed
/// Mirror `dialogs::link::REGION_BODY` and `panels::properties::link::REGION`.
const ADD_WINDOW: &str = "dialog:add-link";
const PROPS_SECTION: &str = "properties.link";

/// Whether a `ui-rect` named `region` was traced after `mark`. The feature's
/// own surface missing is a FAIL; [`press`] alone would report it as a
/// harness error, which the runner files as a SKIP.
fn drew_after(session: &Session, region: &str, mark: usize) -> Result<bool> {
    let trace = session.trace()?;
    Ok(trace
        .events(RECT)
        .any(|l| l.lineno > mark && l.get("name") == Some(region)))
}

/// Drawing a box with Add link and giving it a web address writes a link.
pub struct ALinkIsDrawnAndGivenAnAddress;
/// A selected link is re-pointed, and its border removed, from Properties.
pub struct ASelectedLinkIsRepointedFromProperties;

/// In Review a click on a link follows it and does not select it.
pub struct AReviewClickFollowsALinkAndDoesNotSelectIt;

impl Check for ALinkIsDrawnAndGivenAnAddress {
    fn name(&self) -> &'static str {
        "a_link_is_drawn_and_given_an_address"
    }

    fn defect(&self) -> &'static str {
        "Add link does not arm a box on the canvas, or the window that follows offers no web \
         address or border, or Add link writes nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, ADD_INVOKE, "link-add", add)
    }
}

impl Check for ASelectedLinkIsRepointedFromProperties {
    fn name(&self) -> &'static str {
        "a_selected_link_is_repointed_from_properties"
    }

    fn defect(&self) -> &'static str {
        "a click on a link in Edit does not select it, or Properties offers no destination or \
         border for it, or Apply writes nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, EDIT_INVOKE, "link-edit", repoint)
    }
}

impl Check for AReviewClickFollowsALinkAndDoesNotSelectIt {
    fn name(&self) -> &'static str {
        "a_review_click_follows_a_link_and_does_not_select_it"
    }

    fn defect(&self) -> &'static str {
        "in Review a click on a link selects it as an annotation instead of following it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, REVIEW_INVOKE, "link-review", follow)
    }
}

fn follow(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, ON_LINK)?;
    let trace = session.trace()?;
    let followed = trace.last_after(FOLLOWED, mark);
    let selected = trace
        .last_after(SELECTED, mark)
        .is_some_and(|l| l.get("subtype") == Some("Link"));
    if let Some(line) = followed {
        report.note(format!("followed: `{}`", line.raw));
    }
    Ok((followed.is_none() || selected).then(|| {
        format!(
            "a Review click inside the link must trace `{FOLLOWED}` and no `{SELECTED} \
             subtype=Link`; followed={}, selected={selected}. Trace: {}.",
            followed.is_some(),
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
            "Add link was invoked and no `{ARMED} kind=Link` followed: nothing arms the \
             canvas for the box. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, FIXTURE.page, 0)?;
    let drag_mark = trace.mark();
    let from = mapping.doc_to_window(DocPoint::new(0, DRAG_FROM.0, DRAG_FROM.1))?;
    let to = mapping.doc_to_window(DocPoint::new(0, DRAG_TO.0, DRAG_TO.1))?;
    pointer.drag(session, from, to, 10)?;
    session.settle(40);
    if !drew_after(session, ADD_WINDOW, drag_mark)? {
        return Ok(Some(format!(
            "a box was dragged with Add link armed and the `{ADD_WINDOW}` window never \
             drew. Trace: {}.",
            session.trace_path().display()
        )));
    }

    press(ctx, session, pointer, ADD_WEB)?;
    type_into(ctx, session, pointer, ADD_FIELD, ADDRESS)?;
    press(ctx, session, pointer, ADD_BORDER)?;
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, ADD_BUTTON)?;
    session.settle(40);
    let shot = ctx.out("link-add-after.png");
    pointer.screenshot(session, &shot)?;
    report.artifact(shot);

    let trace = session.trace()?;
    let Some(added) = trace.last_after(ADDED, mark) else {
        return Ok(Some(format!(
            "a box was dragged, an address typed and Add link pressed, and no `{ADDED}` \
             followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("added: `{}`", added.raw));
    // The wrong arms each miss one term: a page link (the form's default)
    // lacks the address, a borderless one says border=false, a link on the
    // wrong page says another page.
    Ok((!(added.raw.contains(ADDRESS_WORD)
        && added.get("border") == Some("true")
        && added.get("page") == Some("0")))
    .then(|| {
        format!(
            "`{}` must put a bordered link to `{ADDRESS}` on page 0.",
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
    let trace = session.trace()?;
    if !trace
        .last_after(SELECTED, mark)
        .is_some_and(|l| l.get("subtype") == Some("Link"))
    {
        return Ok(Some(format!(
            "a click inside the link traced no `{SELECTED}` with subtype=Link. Trace: {}.",
            session.trace_path().display()
        )));
    }
    session.settle(20);
    if !drew_after(session, PROPS_SECTION, mark)? {
        return Ok(Some(format!(
            "a link was selected and Properties drew no `{PROPS_SECTION}` section. Trace: {}.",
            session.trace_path().display()
        )));
    }

    into_view(ctx, session, pointer, PROPS_WEB)?;
    press(ctx, session, pointer, PROPS_WEB)?;
    type_into(ctx, session, pointer, PROPS_FIELD, ADDRESS)?;
    into_view(ctx, session, pointer, APPLY_TARGET)?;
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, APPLY_TARGET)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(retargeted) = trace.last_after(RETARGETED, mark) else {
        return Ok(Some(format!(
            "an address was typed and Apply destination pressed, and no `{RETARGETED}` \
             followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("re-pointed: `{}`", retargeted.raw));
    // The fixture's link runs JavaScript: re-pointing must report removing it.
    if !(retargeted.raw.contains(ADDRESS_WORD) && retargeted.get("replaced") == Some("JavaScript"))
    {
        return Ok(Some(format!(
            "`{}` must point the link at `{ADDRESS}` and report the JavaScript action replaced.",
            retargeted.raw
        )));
    }

    // The fixture's link has a visible 1 pt border, so the box starts ticked.
    into_view(ctx, session, pointer, PROPS_BORDER)?;
    press(ctx, session, pointer, PROPS_BORDER)?;
    into_view(ctx, session, pointer, APPLY_BORDER)?;
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, APPLY_BORDER)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(rebordered) = trace.last_after(REBORDERED, mark) else {
        return Ok(Some(format!(
            "the border was unticked and Apply border pressed, and no `{REBORDERED}` \
             followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("re-bordered: `{}`", rebordered.raw));
    if rebordered.get("visible") != Some("false") {
        return Ok(Some(format!(
            "`{}` must remove the border (visible=false).",
            rebordered.raw
        )));
    }

    // The top-right grip, dragged outward: a link takes a new size.
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, FIXTURE.page, 0)?;
    let grip = mapping.doc_to_window(DocPoint::new(0, LINK_CORNER.0, LINK_CORNER.1))?;
    let landing = mapping.doc_to_window(DocPoint::new(0, CORNER_TO.0, CORNER_TO.1))?;
    let mark = session.trace()?.mark();
    pointer.drag(session, grip, landing, 10)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(resized) = trace.last_after(RESIZED, mark) else {
        return Ok(Some(format!(
            "the selected link's top-right grip was dragged outward and no `{RESIZED}` \
             followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("resized: `{}`", resized.raw));
    Ok(None)
}
