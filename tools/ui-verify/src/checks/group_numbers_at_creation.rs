//! `a_new_groups_numbers_are_set_when_it_is_made` — the Dimension groups
//! tab's *Add a group* fold sets the new ce-dimension group's numbers at
//! creation, in one undo entry, and a unit change keeps them.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/group_numbers_at_creation.md`.

use super::dimdrive::{click_page, press, reveal, run};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const ADD_HEADING: &str = "dimension-groups.heading.add";
/// Mirrors `panels::dimension_groups`'s regions.
const NEW_NAME: &str = "dimension-groups.new_name";
const NEW_UNIT: &str = "dimension-groups.new_unit.combo";
const NEW_UNIT_FT_IN: &str = "dimension-groups.new_unit.option.ft-in";
/// `REGION_NEW_NUMBERS_PREFIX` ahead of `dimensionoverrides::REGION_REDUCE`.
const NEW_REDUCE: &str = "dimension-groups.new_numbers.precision.reduce";
const NEW_SCALE: &str = "dimension-groups.new_scale.combo";
const SCALE_FROM_DEFAULT: &str = "dimension-groups.new_scale.option.0";
const ADD: &str = "dimension-groups.add";
const DRAW_INTO: &str = "dimension-groups.draw_into.";
const ROW: &str = "dimension-groups.row.";
const UNIT: &str = "dimension-groups.unit.combo";
const UNIT_IN: &str = "dimension-groups.unit.option.in";
const UNIT_FT_IN: &str = "dimension-groups.unit.option.ft-in";
const ADDED_GROUP: &str = "dimension-group-added"; // ui-text-exempt: a trace event name, never displayed
const ADDED: &str = "dimension-added"; // ui-text-exempt: a trace event name, never displayed
const SHOWN: &str = "dimension-member-shown"; // ui-text-exempt: a trace event name, never displayed
/// The three picks, in page points: Δx 200, Δy 150, so 250 pt aligned.
const A: (f64, f64) = (60.0, 60.0);
const B: (f64, f64) = (260.0, 210.0);
const PLACE: (f64, f64) = (160.0, 30.0);
/// The fixture's default group reads 5 mm per point.
const MM_PER_PT: f64 = 5.0;
const MM_PER_INCH: f64 = 25.4;
/// Feet and inches start in eighths.
const EIGHTHS: i64 = 8;

/// See the module documentation.
pub struct ANewGroupsNumbersAreSetWhenItIsMade;

impl Check for ANewGroupsNumbersAreSetWhenItIsMade {
    fn name(&self) -> &'static str {
        "a_new_groups_numbers_are_set_when_it_is_made"
    }

    fn defect(&self) -> &'static str {
        "a new feet-and-inches ce-dimension group reads 2/8 instead of 1/4, a group made to \
         keep eighths cannot be made in one step, or a unit change drops the choice"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(
            self,
            ctx,
            "mode.review,measure.linear,measure.manage_groups",
            "group-numbers-at-creation",
            body,
        )
    }
}

/// What the fixture's 250 pt line must read, worked from its geometry.
struct Oracle {
    feet: i64,
    inches: i64,
    total_inches: i64,
    /// The eighths numerator, kept: `2/8`.
    kept: String,
    /// The same fraction in lowest terms: `1/4`.
    reduced: String,
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 {
        a.abs().max(1)
    } else {
        gcd(b, a % b)
    }
}

fn oracle() -> Result<Oracle> {
    let inches = (B.0 - A.0).hypot(B.1 - A.1) * MM_PER_PT / MM_PER_INCH;
    // Truncation is the formatter's: whole inches, then the nearest eighth.
    #[allow(clippy::cast_possible_truncation)]
    let whole = inches.trunc() as i64;
    #[allow(clippy::cast_possible_truncation)]
    let numer = ((inches - inches.trunc()) * 8.0).round() as i64;
    let g = gcd(numer, EIGHTHS);
    if numer <= 0 || numer >= EIGHTHS || g == 1 {
        return Err(Error::new(format!(
            "the picks give {numer}/{EIGHTHS}, which reads the same reduced or kept, so this \
             check could not tell the two apart. Move A or B."
        )));
    }
    Ok(Oracle {
        feet: whole / 12,
        inches: whole % 12,
        total_inches: whole,
        kept: format!("{numer}/{EIGHTHS}"),
        reduced: format!("{}/{}", numer / g, EIGHTHS / g),
    })
}

/// Type `name`, pick feet and inches, optionally untick lowest terms, take the
/// default group's scale and Add. The new group's `dimension-group-added` line.
fn add_group(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    name: &str,
    keep_denominator: bool,
) -> Result<crate::trace::TraceLine> {
    reveal(ctx, session, pointer, NEW_NAME, ADD_HEADING)?;
    press(ctx, session, pointer, NEW_NAME)?;
    pointer.type_text(session, None, name)?;
    session.settle(15);
    press(ctx, session, pointer, NEW_UNIT)?;
    press(ctx, session, pointer, NEW_UNIT_FT_IN)?;
    if keep_denominator {
        press(ctx, session, pointer, NEW_REDUCE)?;
    }
    press(ctx, session, pointer, NEW_SCALE)?;
    press(ctx, session, pointer, SCALE_FROM_DEFAULT)?;
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, ADD)?;
    session.settle(20);
    session
        .trace()?
        .last_after(ADDED_GROUP, mark)
        .cloned()
        .ok_or_else(|| {
            Error::new(format!(
                "Add made no group named {name} (no `{ADDED_GROUP}`). Trace: {}.",
                session.trace_path().display()
            ))
        })
}

/// Draw A → B into `group`; the raw `dimension-member-shown` line for the new
/// ce dimension.
fn draw_into(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    group: &str,
) -> Result<String> {
    press(ctx, session, pointer, &format!("{DRAW_INTO}{group}"))?;
    let mark = session.trace()?.mark();
    for at in [A, B, PLACE] {
        click_page(ctx, session, pointer, at)?;
    }
    session.settle(30);
    let trace = session.trace()?;
    let added = trace.last_after(ADDED, mark).ok_or_else(|| {
        Error::new(format!(
            "three clicks committed no ce dimension into group {group}. Trace: {}.",
            session.trace_path().display()
        ))
    })?;
    if added.get("group") != Some(group) {
        return Err(Error::new(format!(
            "the ce dimension joined the wrong group: `{}`, wanted group {group}. Trace: {}.",
            added.raw,
            session.trace_path().display()
        )));
    }
    let dim = added.get("dim").unwrap_or_default();
    shown_after(session, mark, group, Some(dim))
}

/// The newest `dimension-member-shown` line for `group` (and `dim`) after `mark`.
fn shown_after(session: &Session, mark: usize, group: &str, dim: Option<&str>) -> Result<String> {
    let trace = session.trace()?;
    trace
        .events(SHOWN)
        .filter(|l| l.lineno > mark && l.get("group") == Some(group))
        .filter(|l| dim.is_none_or(|d| l.get("dim") == Some(d)))
        .last()
        .map(|l| l.raw.clone())
        .ok_or_else(|| {
            Error::new(format!(
                "no `{SHOWN}` line for group {group}. Trace: {}.",
                session.trace_path().display()
            ))
        })
}

/// Switch `group` to `option` from its own unit combo; its member's shown line.
fn switch_unit(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    group: &str,
    option: &str,
) -> Result<String> {
    press(ctx, session, pointer, &format!("{ROW}{group}"))?;
    press(ctx, session, pointer, UNIT)?;
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, option)?;
    shown_after(session, mark, group, None)
}

/// The one-undo-entry assertion: `undo_depth` on the group's creation line.
fn depth_is(line: &crate::trace::TraceLine, want: usize, session: &Session) -> Option<String> {
    (line.get_usize("undo_depth") != Some(want)).then(|| {
        format!(
            "making a group with its numbers must be one undo entry; with {} before it the \
             stack must hold {want}, and `{}` says otherwise. Trace: {}.",
            want - 1,
            line.raw,
            session.trace_path().display()
        )
    })
}

fn body(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let want = oracle()?;
    let reduced_group = add_group(ctx, session, pointer, "Reduced", false)?;
    report.note(format!("`{}`", reduced_group.raw));
    // A freshly opened document has an empty undo stack.
    if let Some(failure) = depth_is(&reduced_group, 1, session) {
        return Ok(Some(failure));
    }
    let fixed_group = add_group(ctx, session, pointer, "Fixed", true)?;
    report.note(format!("`{}`", fixed_group.raw));
    // Folded shut so the selected group's unit rises into the window.
    press(ctx, session, pointer, ADD_HEADING)?;
    if let Some(failure) = depth_is(&fixed_group, 2, session) {
        return Ok(Some(failure));
    }
    let reduced_id = reduced_group.get("group").unwrap_or_default().to_owned();
    let fixed_id = fixed_group.get("group").unwrap_or_default().to_owned();

    let ft_in = |frac: &str| format!("{}'-{} {frac}", want.feet, want.inches);
    let readings = [
        (
            "new feet-and-inches group",
            draw_into(ctx, session, pointer, &reduced_id)?,
            ft_in(&want.reduced),
            &want.kept,
        ),
        (
            "group made to keep eighths",
            draw_into(ctx, session, pointer, &fixed_id)?,
            ft_in(&want.kept),
            &want.reduced,
        ),
        (
            "that group switched to inches",
            switch_unit(ctx, session, pointer, &fixed_id, UNIT_IN)?,
            format!("{} {} in", want.total_inches, want.kept),
            &want.reduced,
        ),
        (
            "and back to feet and inches",
            switch_unit(ctx, session, pointer, &fixed_id, UNIT_FT_IN)?,
            ft_in(&want.kept),
            &want.reduced,
        ),
    ];
    for (what, raw, must, must_not) in &readings {
        report.note(format!("{what}: `{raw}`"));
        if !raw.contains(must.as_str()) || raw.contains(must_not.as_str()) {
            return Ok(Some(format!(
                "the {what} must read `{must}` (250 pt at 5 mm per point is 1250 mm), not \
                 `{must_not}`; it read `{raw}`. Trace: {}.",
                session.trace_path().display()
            )));
        }
    }
    Ok(None)
}
