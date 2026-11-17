//! `checks::model_pick` — **the 3D viewer selects parts by clicking them,
//! opens one menu on the picture and on the list, and shows the hidden parts
//! alone**
//!
//! Opens `named-tree.prc` in the viewer, off the desktop through the scripted
//! pointer. Every part in it is the same square in the same place, so the
//! centre of the picture is over whichever shown part is nearest: which one
//! is not asserted, only that it is one of the shown three and that hiding it
//! puts a different one under the same point. See
//! `docs/modules/ui-verify/checks/model_pick.md` for the steps.

use std::collections::BTreeSet;

use crate::checks::driving::declared_in;
use crate::checks::model_inferences::open_viewer;
use crate::checks::model_view_window::{launch_with_model_file, ui_rect};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::input::scripted::{ScriptedPointer, xy};
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

/// Assembly (0) > Bolt (1), Bracket (2), Plate (3), Pin (4, stored hidden),
/// Nut (5, stored suppressed).
const NAMED_TREE: &str = "D:/Dev/pdfcer/fixtures/synthetic/prc/named-tree.prc";
const IMAGE: &str = "model3d.image";
const STATE: &str = "model-view-parts-state";
const FOLD: &str = "model-view-parts-fold";
const HIDDEN_ONLY: &str = "model3d.parts.hidden_only";

/// See the module documentation.
pub struct TheViewerSelectsFromThePicture;

impl Check for TheViewerSelectsFromThePicture {
    fn name(&self) -> &'static str {
        "the_3d_viewer_selects_and_hides_from_the_picture"
    }

    fn defect(&self) -> &'static str {
        "a part cannot be selected by clicking it in the 3D picture, a right-click opens no \
         menu to hide it, the list does not follow, or the hidden parts cannot be shown alone"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch_with_model_file(ctx, &mut report, "model-pick", NAMED_TREE).and_then(
            |(session, pointer)| {
                let drive = Drive {
                    ctx,
                    session: &session,
                    pointer: &pointer,
                };
                if let Some(failure) = open_viewer(ctx, &session, &pointer)? {
                    return Ok(Some(failure));
                }
                let failure = drive.picture(&mut report)?;
                if failure.is_some() {
                    return Ok(failure);
                }
                drive.list(&mut report)
            },
        );
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// A `a|b|c` or `-` trace list as a set.
fn set(line: &TraceLine, key: &str) -> BTreeSet<usize> {
    line.get(key)
        .unwrap_or("-")
        .split('|')
        .filter_map(|n| n.parse().ok())
        .collect()
}

struct Drive<'a> {
    ctx: &'a CheckContext,
    session: &'a Session,
    pointer: &'a ScriptedPointer,
}

impl Drive<'_> {
    /// Click the centre of `region` with `how` (`""`, `" btn=r"`,
    /// `" mods=ctrl"`) and return the `event` line it traced, if any.
    fn click(&self, region: &str, how: &str, event: &str) -> Result<Option<TraceLine>> {
        let trace = self.session.trace()?;
        let Some((rect, vp)) = declared_in(&trace, ui_rect(self.ctx)?, region) else {
            return Err(crate::error::Error::new(format!(
                "the viewer drew no `{region}`."
            )));
        };
        let before = trace.events(event).count();
        let vp = vp.map_or(String::new(), |v| format!(" vp={v}"));
        let at = xy(WindowPoint::centre_of(rect));
        self.pointer
            .send(self.session, &format!("click {at}{how}{vp}"))?;
        self.session.settle(30);
        Ok(self.session.trace()?.events(event).nth(before).cloned())
    }

    /// Right-click `region`, then choose the menu item `key`.
    fn menu(&self, region: &str, key: &str) -> Result<Option<TraceLine>> {
        self.click(region, " btn=r", STATE)?;
        self.click(&format!("model3d.menu.{key}"), "", STATE)
    }

    /// Steps 1 to 6: the picture.
    fn picture(&self, report: &mut CheckReport) -> Result<Option<String>> {
        let shown = BTreeSet::from([1, 2, 3]);
        let Some(first) = self.click(IMAGE, "", STATE)? else {
            return Ok(Some(
                "★★★ a click on the model in the picture traced no parts line: it selects nothing."
                    .to_owned(),
            ));
        };
        report.note(format!("click: {}", first.raw));
        let picked = set(&first, "highlight");
        if picked.len() != 1 || !picked.is_subset(&shown) || first.get("why") != Some("pick") {
            return Ok(Some(format!(
                "★★★ the click traced `{}`; it must select exactly one of the shown parts 1, 2, 3.",
                first.raw
            )));
        }
        let Some(hid) = self.menu(IMAGE, "hide")? else {
            return Ok(Some(
                "★★★ Hide on the picture's right-click menu traced no parts line.".to_owned(),
            ));
        };
        report.note(format!("hide: {}", hid.raw));
        if !picked.is_subset(&set(&hid, "hidden")) || set(&hid, "highlight") != picked {
            return Ok(Some(format!(
                "★★★ Hide traced `{}`; the selected part {picked:?} must be hidden and stay selected.",
                hid.raw
            )));
        }
        let Some(second) = self.click(IMAGE, "", STATE)? else {
            return Ok(Some(
                "★★★ a click where the hidden part was selected nothing: the part behind it \
                 cannot be reached."
                    .to_owned(),
            ));
        };
        report.note(format!("click behind: {}", second.raw));
        let behind = set(&second, "highlight");
        if behind.len() != 1 || !behind.is_subset(&shown) || behind == picked {
            return Ok(Some(format!(
                "★★★ with {picked:?} hidden the click traced `{}`; it must select a different \
                 shown part.",
                second.raw
            )));
        }
        let toggled = self.click(IMAGE, " mods=ctrl", STATE)?;
        report.note(format!(
            "ctrl+click: {:?}",
            toggled.as_ref().map(|l| &l.raw)
        ));
        if toggled.as_ref().map(|l| set(l, "highlight")) != Some(BTreeSet::new()) {
            return Ok(Some(
                "★★★ Ctrl+click on the selected part did not take it out of the selection."
                    .to_owned(),
            ));
        }
        self.hidden_view(report, &set(&hid, "hidden"))
    }

    /// Steps 5 and 6: only the hidden parts, and a click there shows one.
    fn hidden_view(
        &self,
        report: &mut CheckReport,
        hidden: &BTreeSet<usize>,
    ) -> Result<Option<String>> {
        let on = self.click(HIDDEN_ONLY, "", STATE)?;
        report.note(format!("hidden only: {:?}", on.as_ref().map(|l| &l.raw)));
        if on.as_ref().and_then(|l| l.get("view")) != Some("hidden") {
            return Ok(Some(
                "★★★ the hidden-parts switch did not change the view to the hidden parts."
                    .to_owned(),
            ));
        }
        let Some(restored) = self.click(IMAGE, "", STATE)? else {
            return Ok(Some(
                "★★★ a click on a hidden part in the hidden-parts view traced nothing.".to_owned(),
            ));
        };
        report.note(format!("restore: {}", restored.raw));
        let left = set(&restored, "hidden");
        let back: Vec<&usize> = hidden.difference(&left).collect();
        if restored.get("why") != Some("restore") || back.len() != 1 || !left.is_subset(hidden) {
            return Ok(Some(format!(
                "★★★ the click traced `{}`; exactly one of the hidden parts {hidden:?} must be \
                 shown again and nothing else change.",
                restored.raw
            )));
        }
        if !set(&restored, "highlight").is_empty() {
            return Ok(Some(
                "★★★ the click that restored a part also selected it.".to_owned(),
            ));
        }
        let off = self.click(HIDDEN_ONLY, "", STATE)?;
        if off.as_ref().and_then(|l| l.get("view")) != Some("shown") {
            return Ok(Some(
                "★★★ the hidden-parts switch did not go back to the shown parts.".to_owned(),
            ));
        }
        Ok(None)
    }

    /// Steps 7 to 9: the list's run selection, its menu, and the list
    /// following a click on the picture.
    fn list(&self, report: &mut CheckReport) -> Result<Option<String>> {
        self.click("model3d.part.name.1", "", STATE)?;
        let run = self.click("model3d.part.name.3", " mods=shift", STATE)?;
        report.note(format!("shift run: {:?}", run.as_ref().map(|l| &l.raw)));
        if run.as_ref().map(|l| set(l, "highlight")) != Some(BTreeSet::from([1, 2, 3])) {
            return Ok(Some(
                "★★★ a click on Bolt then Shift+click on Plate did not select Bolt, Bracket and \
                 Plate."
                    .to_owned(),
            ));
        }
        let fewer = self.click("model3d.part.name.3", " mods=ctrl", STATE)?;
        if fewer.as_ref().map(|l| set(l, "highlight")) != Some(BTreeSet::from([1, 2])) {
            return Ok(Some(
                "★★★ Ctrl+click on Plate did not take it alone out of the selection.".to_owned(),
            ));
        }
        let isolated = self.menu("model3d.part.name.2", "isolate")?;
        report.note(format!("isolate: {:?}", isolated.as_ref().map(|l| &l.raw)));
        let ok = isolated.as_ref().is_some_and(|l| {
            set(l, "hidden") == BTreeSet::from([3, 4, 5])
                && set(l, "highlight") == BTreeSet::from([1, 2])
        });
        if !ok {
            return Ok(Some(
                "★★★ a right-click inside the selection then Show only the selected parts did \
                 not leave Bolt and Bracket shown and selected with the rest hidden."
                    .to_owned(),
            ));
        }
        let folded = self.fold_all()?;
        report.note(format!("fold all: {:?}", folded.as_ref().map(|l| &l.raw)));
        if folded.as_ref().and_then(|l| l.get("folded")) != Some("0") {
            return Ok(Some(
                "Fold all on the list's menu folded nothing.".to_owned(),
            ));
        }
        let opened = self.click(IMAGE, "", FOLD)?;
        report.note(format!(
            "click with the list folded: {:?}",
            opened.as_ref().map(|l| &l.raw)
        ));
        let ok = opened
            .as_ref()
            .is_some_and(|l| l.get("why") == Some("reveal") && l.get("folded") == Some("-"));
        if !ok {
            return Ok(Some(
                "★★★ a click on a part in the picture did not unfold the list down to its row."
                    .to_owned(),
            ));
        }
        Ok(None)
    }

    fn fold_all(&self) -> Result<Option<TraceLine>> {
        self.click("model3d.part.name.2", " btn=r", STATE)?;
        self.click("model3d.menu.fold", "", FOLD)
    }
}
