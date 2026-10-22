//! `a_marks_outline_from_the_format_tab_is_stored_in_cmyk` — a mark's outline
//! colour, set from Format ▸ Markup on the picker's CMYK tab, is written into
//! the annotation's `/C` as four components. Scripted pointer, window off the
//! desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/band_cmyk.md`.

use super::dimdrive::{Fixture, click_on, press, reveal, run_on, ui_rect_event};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.review";
/// One red RGB `/Square` at `[200 400 340 460]` with no appearance.
const FIXTURE: Fixture = Fixture {
    file: "square-no-appearance.pdf",
    method: "Committed with the repository; restore it from git.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const STEM: &str = "band-cmyk";
/// Inside the square's `/Rect`, page space.
const ON_THE_SQUARE: (f64, f64) = (270.0, 430.0);
const FORMAT_TAB: &str = "ribbon.tab.format";
/// Mirror `app::markupband::REGION_STROKE` and the picker's tab region.
const SWATCH: &str = "markup_band.stroke";
const CMYK_TAB: &str = "markup_band.stroke.space.cmyk";
const SELECT: &str = "annot-select"; // ui-text-exempt: a trace event name, never displayed
const SAVED: &str = "save-in-place"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct AMarksOutlineFromTheFormatTabIsStoredInCmyk;

impl Check for AMarksOutlineFromTheFormatTabIsStoredInCmyk {
    fn name(&self) -> &'static str {
        "a_marks_outline_from_the_format_tab_is_stored_in_cmyk"
    }

    fn defect(&self) -> &'static str {
        "the Format tab's outline colour offers no CMYK, or a colour chosen on its CMYK tab is \
         not written to the mark's /C as CMYK"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, INVOKE, STEM, steps)
    }
}

/// The component count of every `/C` array in a file, as written.
fn colour_arity(bytes: &[u8]) -> Vec<usize> {
    let text = String::from_utf8_lossy(bytes);
    text.match_indices("/C")
        .filter_map(|(i, _)| {
            let rest = text[i + 2..].trim_start().strip_prefix('[')?;
            Some(rest.split(']').next()?.split_whitespace().count())
        })
        .collect()
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let copy = ctx.out(&format!("{STEM}.pdf"));
    let read = |p: &std::path::Path| {
        std::fs::read(p).map_err(|e| Error::new(format!("reading {}: {e}", p.display())))
    };
    // The fixture's only `/C` is RGB, so a four-component one can only be the edit's.
    if colour_arity(&read(&copy)?).contains(&4) {
        return Ok(Some(format!(
            "the fixture already holds a CMYK `/C`, so this check cannot tell the edit's. {}",
            FIXTURE.method
        )));
    }
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, ON_THE_SQUARE)?;
    if session.trace()?.last_after(SELECT, mark).is_none() {
        return Ok(Some(format!(
            "a click inside the square traced no `{SELECT}`. Trace: {path}."
        )));
    }
    reveal(ctx, session, pointer, SWATCH, FORMAT_TAB)?;
    press(ctx, session, pointer, SWATCH)?;
    let Some((_, vp)) = declared_in(&session.trace()?, ui_rect_event(ctx)?, CMYK_TAB) else {
        return Ok(Some(format!(
            "★★★ the Format tab's outline picker opened with no CMYK tab (`{CMYK_TAB}`). \
             Trace: {path}."
        )));
    };
    press(ctx, session, pointer, CMYK_TAB)?;
    // The close is the commit.
    pointer.key(session, vp.as_deref(), "Escape", None)?;
    session.settle(30);
    pointer.key(session, None, "S", Some("ctrl"))?;
    session.settle(40);
    if !session
        .trace()?
        .events(SAVED)
        .any(|l| l.get("outcome") == Some("ok"))
    {
        return Ok(Some(format!(
            "Ctrl+S traced no `{SAVED} outcome=ok`. Trace: {path}."
        )));
    }
    let arities = colour_arity(&read(&copy)?);
    report.note(format!(
        "/C component counts in the saved file: {arities:?}"
    ));
    Ok((!arities.contains(&4)).then(|| {
        format!(
            "★★★ after the CMYK tab was chosen and the picker closed, the saved file holds no \
             four-component `/C` (component counts {arities:?}). Trace: {path}."
        )
    }))
}

#[cfg(test)]
mod tests {
    use super::colour_arity;

    #[test]
    fn colour_arity_counts_each_c_arrays_components() {
        let got = colour_arity(b"<< /C [1 0 0] /CA 1 /Contents (x) >> << /C [0 1 1 0] >>");
        assert_eq!(got, vec![3, 4]);
    }
}
