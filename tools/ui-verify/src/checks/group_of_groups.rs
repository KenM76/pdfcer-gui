//! `a_group_groups_with_a_third_markup_and_comes_apart_one_level`, on
//! `fixtures/annots-nest.pdf`: two markups grouped, that group grouped with a
//! third, one Ungroup brings the first group back, judged on the trace, on
//! clicks, and in two saved copies.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/group_of_groups.md`.

use super::dimdrive::{Fixture, click_on};
use super::multiselect::{Saves, after, run_saving, save, shift_click, step};
use super::savedannots::{SavedAnnot, listed_annots, read};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "annots-nest.pdf",
    method: "Rebuild it with `python fixtures/annots-nest.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 300.0,
    },
};

/// Each markup: the substring that finds it in a saved copy, and a point on
/// its outline, page space.
const SQUARE: (&str, (f64, f64)) = ("/NM (SQ)", (41.0, 210.0)); // ui-text-exempt: a PDF key
const CIRCLE: (&str, (f64, f64)) = ("/NM (CI)", (149.5, 213.4)); // ui-text-exempt: a PDF key
const THIRD: (&str, (f64, f64)) = ("/NM (TR)", (261.0, 210.0)); // ui-text-exempt: a PDF key

const GROUPED: &str = "markup-group-applied"; // ui-text-exempt: a trace event name
const UNGROUPED: &str = "markup-ungroup-applied"; // ui-text-exempt: a trace event name
const TOGETHER: &str = "annot-group-select"; // ui-text-exempt: a trace event name
const TOGGLED: &str = "annot-toggled"; // ui-text-exempt: a trace event name
const ALONE: &str = "annot-select"; // ui-text-exempt: a trace event name
const RECORD: &str = "/pdfcer_Groups"; // ui-text-exempt: a PDF key

/// See the module documentation.
pub struct AGroupGroupsWithAThirdMarkup;

impl Check for AGroupGroupsWithAThirdMarkup {
    fn name(&self) -> &'static str {
        "a_group_groups_with_a_third_markup_and_comes_apart_one_level"
    }

    fn defect(&self) -> &'static str {
        "a group of markups cannot be grouped with another markup, or Ungroup on the result \
         releases every markup instead of bringing the first group back"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_saving(self, ctx, &FIXTURE, "mode.edit", "group-of-groups", drive)
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    saves: &Saves,
) -> Result<Option<String>> {
    // --- the square and the circle become a group --------------------------
    click_on(ctx, session, pointer, &FIXTURE, 0, SQUARE.1)?;
    shift_click(ctx, session, pointer, &FIXTURE, CIRCLE.1)?;
    let first = step!(group(
        session,
        pointer,
        "Ctrl+G on the square and the circle"
    ));
    report.note(format!("first group: `{}`", first.raw));
    if first.get("members") != Some("1") || first.get("recorded") == Some("true") {
        return Ok(Some(format!(
            "Ctrl+G on two loose markups traced `{}`: one member and nothing recorded was \
             wanted.",
            first.raw
        )));
    }

    if let Some(why) = with_the_third(ctx, report, session, pointer)? {
        return Ok(Some(why));
    }
    if let Some(why) = selects(ctx, report, session, pointer, CIRCLE.1, Some(("2", "3")))? {
        return Ok(Some(why));
    }
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    if let Some(why) = judge_merged(&saves.moved)? {
        return Ok(Some(why));
    }

    // --- one Ungroup: the first group is back, the third is loose ----------
    let mark = session.trace()?.mark();
    pointer.key(session, None, "G", Some("ctrl+shift"))?;
    session.settle(40);
    let peeled = step!(after(session, UNGROUPED, mark, "Ctrl+Shift+G was pressed"));
    report.note(format!("`{}`", peeled.raw));
    if peeled.get("restored") != Some("1") {
        return Ok(Some(format!(
            "one Ungroup of the group of groups traced `{}`: one earlier group put back was \
             wanted.",
            peeled.raw
        )));
    }
    if let Some(why) = selects(ctx, report, session, pointer, SQUARE.1, Some(("1", "2")))? {
        return Ok(Some(why));
    }
    if let Some(why) = selects(ctx, report, session, pointer, THIRD.1, None)? {
        return Ok(Some(why));
    }
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    judge_peeled(&saves.undone)
}

/// The group selected by a click, the third markup added, Ctrl+G: the earlier
/// group must be recorded.
fn with_the_third(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    click_on(ctx, session, pointer, &FIXTURE, 0, SQUARE.1)?;
    let mark = session.trace()?.mark();
    shift_click(ctx, session, pointer, &FIXTURE, THIRD.1)?;
    let added = step!(after(
        session,
        TOGGLED,
        mark,
        "the third markup was Shift-clicked"
    ));
    report.note(format!("`{}`", added.raw));
    if added.get("after") != Some("3") {
        return Ok(Some(format!(
            "Shift-clicking the third markup with the group selected traced `{}`: three \
             selected was wanted.",
            added.raw
        )));
    }
    let merged = step!(group(
        session,
        pointer,
        "Ctrl+G on the group and the third markup"
    ));
    report.note(format!("group of groups: `{}`", merged.raw));
    if merged.get("recorded") != Some("true") {
        return Ok(Some(format!(
            "Ctrl+G on a group and a third markup traced `{}`: the earlier group recorded was \
             wanted.",
            merged.raw
        )));
    }
    Ok(None)
}

/// Press Ctrl+G; the `markup-group-applied` line after it.
fn group(
    session: &Session,
    pointer: &ScriptedPointer,
    what: &str,
) -> Result<std::result::Result<crate::trace::TraceLine, String>> {
    let mark = session.trace()?.mark();
    pointer.key(session, None, "G", Some("ctrl"))?;
    session.settle(40);
    after(session, GROUPED, mark, &format!("{what} was pressed"))
}

/// Click `at`; `Some((mates, selected))` wants a group selection of that
/// shape, `None` wants a markup selected alone.
fn selects(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    at: (f64, f64),
    want: Option<(&str, &str)>,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, at)?;
    session.settle(10);
    let together = session.trace()?.last_after(TOGETHER, mark).cloned();
    match (want, together) {
        (Some((mates, selected)), Some(line)) => {
            report.note(format!("click at {at:?}: `{}`", line.raw));
            let right = line.get("mates") == Some(mates) && line.get("selected") == Some(selected);
            Ok((!right).then(|| {
                format!(
                    "a click at {at:?} traced `{}`: mates={mates} selected={selected} was wanted.",
                    line.raw
                )
            }))
        }
        (Some(_), None) => Ok(Some(format!(
            "a click at {at:?} selected no group. Trace: {}.",
            session.trace_path().display()
        ))),
        (None, Some(line)) => Ok(Some(format!(
            "a click at {at:?} on a markup Ungroup should have left loose traced `{}`.",
            line.raw
        ))),
        (None, None) => {
            let alone = step!(after(session, ALONE, mark, &format!("a click at {at:?}")));
            report.note(format!("click at {at:?}: `{}`", alone.raw));
            Ok(None)
        }
    }
}

/// The three saved markups, by `/NM`.
fn three(path: &std::path::Path) -> Result<std::result::Result<[SavedAnnot; 3], String>> {
    let annots = listed_annots(&read(path)?);
    let find = |key: &str| annots.iter().find(|a| a.body.contains(key)).cloned();
    Ok(match (find(SQUARE.0), find(CIRCLE.0), find(THIRD.0)) {
        (Some(s), Some(c), Some(t)) => Ok([s, c, t]),
        _ => Err(format!(
            "{} does not list all three markups.",
            path.display()
        )),
    })
}

/// The object number `/IRT` names in `body`.
fn irt_of(body: &str) -> Option<u32> {
    let rest = body[body.find("/IRT")? + "/IRT".len()..].trim_start();
    if !rest.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    let words: Vec<&str> = rest
        .split(|c: char| c.is_whitespace() || "/[]<>()".contains(c))
        .filter(|w| !w.is_empty())
        .take(3)
        .collect();
    match words[..] {
        [num, _, "R"] => num.parse().ok(),
        _ => None,
    }
}

/// After the group of groups: the square and the circle are under the third
/// markup, and both record the group they came from.
fn judge_merged(path: &std::path::Path) -> Result<Option<String>> {
    let [square, circle, third] = match three(path)? {
        Ok(found) => found,
        Err(why) => return Ok(Some(why)),
    };
    for (name, a) in [("square", &square), ("circle", &circle)] {
        if irt_of(&a.body) != Some(third.num) || !a.body.contains(RECORD) {
            return Ok(Some(format!(
                "after the group of groups the saved {name} reads `{}`: `/IRT {} 0 R` and a \
                 `{RECORD}` record were wanted.",
                a.body, third.num
            )));
        }
    }
    if irt_of(&third.body).is_some() {
        return Ok(Some(format!(
            "after the group of groups the saved third markup, the primary, reads `{}`.",
            third.body
        )));
    }
    Ok(None)
}

/// After one Ungroup: the square is under the circle again and the third
/// markup is in no group.
fn judge_peeled(path: &std::path::Path) -> Result<Option<String>> {
    let [square, circle, third] = match three(path)? {
        Ok(found) => found,
        Err(why) => return Ok(Some(why)),
    };
    if irt_of(&square.body) != Some(circle.num) {
        return Ok(Some(format!(
            "after one Ungroup the saved square reads `{}`: `/IRT {} 0 R`, the first group \
             back, was wanted.",
            square.body, circle.num
        )));
    }
    for (name, a) in [("circle", &circle), ("third markup", &third)] {
        if irt_of(&a.body).is_some() {
            return Ok(Some(format!(
                "after one Ungroup the saved {name} reads `{}`: in no group was wanted.",
                a.body
            )));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::irt_of;

    #[test]
    fn irt_reads_the_reference_and_nothing_else() {
        assert_eq!(irt_of("<< /IRT 9 0 R /RT /Group >>"), Some(9));
        assert_eq!(irt_of("<</IRT 9 0 R/RT /Group>>"), Some(9));
        assert_eq!(irt_of("<< /RT /Group >>"), None);
        assert_eq!(irt_of("<< /IRT [9 0 R] >>"), None);
    }
}
