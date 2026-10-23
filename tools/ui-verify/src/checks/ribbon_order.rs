//! `ribbon_puts_clipboard_first_and_save_beside_file` — Clipboard is the
//! Edit tab's first group, Save is the File tab's second, after File, and the
//! highlight glyph's band is drawn filled yellow.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ribbon_order.md`.

use super::dimdrive::{Fixture, press, run_on, ui_rect_event};
use crate::checks::driving::{declared, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::{Error, Result};
use crate::image::Image;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "paragraph.pdf",
    method: "Regenerate it: python tools/gen-reflow-fixture.py.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};

/// The highlight command's ribbon button.
const HIGHLIGHT: &str = "ribbon.item.markup.highlight"; // ui-text-exempt: a trace region name, never displayed
/// Fewest yellow pixels that count as the band filled. The band at the
/// smallest icon size covers several times this.
const MIN_YELLOW: usize = 12;

/// See the module documentation.
pub struct RibbonPutsClipboardFirstAndSaveBesideFile;

impl Check for RibbonPutsClipboardFirstAndSaveBesideFile {
    fn name(&self) -> &'static str {
        "ribbon_puts_clipboard_first_and_save_beside_file"
    }

    fn defect(&self) -> &'static str {
        "Clipboard is not the first group on Edit, Save is not beside File, or the highlight \
         icon's band is hollow, so the ribbon does not read in a word processor's order"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, "mode.edit", "ribbon-order", drive)
    }
}

/// The group ids drawn on `tab` now, left to right. A collapsed group counts
/// under its own id.
fn groups_in_order(ctx: &CheckContext, session: &Session, tab: &str) -> Result<Vec<String>> {
    let trace = session.trace()?;
    let ui_rect = ui_rect_event(ctx)?;
    let prefix = format!("ribbon.group.{tab}.");
    let mut drawn: Vec<(f32, String)> = Vec::new();
    for name in declared_names(&trace, ui_rect, &prefix) {
        if name.ends_with(".caption") {
            continue;
        }
        let Some(rect) = declared(&trace, ui_rect, &name) else {
            continue;
        };
        let id = name[prefix.len()..]
            .trim_end_matches(".collapsed")
            .to_owned();
        if !drawn.iter().any(|(_, d)| *d == id) {
            drawn.push((rect.min.x, id));
        }
    }
    drawn.sort_by(|a, b| a.0.total_cmp(&b.0));
    Ok(drawn.into_iter().map(|(_, id)| id).collect())
}

/// Click `tab` and answer its groups left to right.
fn open_tab(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    tab: &str,
) -> Result<Vec<String>> {
    press(ctx, session, pointer, &format!("ribbon.tab.{tab}"))?;
    session.settle(20);
    groups_in_order(ctx, session, tab)
}

/// Pixels of the highlighter's wash, in either preset's yellow.
fn yellow(image: &Image, region: crate::geom::PixRect) -> usize {
    image
        .pixels_in(region)
        .filter(|p| p.r > 200 && p.g > 180 && p.b < 110)
        .count()
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    session.settle(30);
    let path = session.trace_path().display().to_string();
    let edit = open_tab(ctx, session, pointer, "edit")?;
    report.note(format!("Edit groups: {}", list(&edit)));
    if edit.first().map(String::as_str) != Some("clipboard") {
        return Ok(Some(format!(
            "★ Edit's groups read left to right as {}; Clipboard must lead. Trace: {path}.",
            list(&edit)
        )));
    }
    let file = open_tab(ctx, session, pointer, "file")?;
    report.note(format!("File groups: {}", list(&file)));
    if file.get(..2) != Some(&["file".to_owned(), "save".to_owned()][..]) {
        return Ok(Some(format!(
            "★ File's groups read left to right as {}; File then Save must lead. \
             Trace: {path}.",
            list(&file)
        )));
    }
    press(ctx, session, pointer, "ribbon.tab.markup")?;
    session.settle(20);
    let Some((rect, _)) = declared_in(&session.trace()?, ui_rect_event(ctx)?, HIGHLIGHT) else {
        return Err(Error::new(format!(
            "no `{HIGHLIGHT}` on the Markup tab, so the glyph cannot be read. Trace: {path}."
        )));
    };
    let png = ctx.out("ribbon-order.markup.png");
    pointer.screenshot(session, &png)?;
    report.artifact(png.clone());
    let image = Image::load_png(&png)?;
    let lit = yellow(&image, session.frame()?.logical_to_capture_pixels(rect));
    report.note(format!("highlight button: {lit} yellow pixels"));
    Ok((lit < MIN_YELLOW).then(|| {
        format!(
            "★ the highlight button draws {lit} yellow pixels, fewer than {MIN_YELLOW}: its \
             band is hollow with coloured icons on. Screenshot: {}.",
            png.display()
        )
    }))
}
