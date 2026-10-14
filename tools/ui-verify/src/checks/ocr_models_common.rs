//! What the OCR model manager checks share: a private program folder holding
//! a copy of the binary and the models a check plants, the launch, the clicks,
//! and the waits on the manager's trace lines.
//!
//! Each check runs its own copy because the manager deletes and installs in
//! `<exe dir>/models`, and the copy's `userdata/` keeps its preferences away
//! from every other run.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_models_manage.md`.

use std::path::{Path, PathBuf};

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, declared_names, list};
use crate::checks::{CheckContext, CheckReport};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::trace::TraceLine;

/// Off the desktop, so the check runs while the operator uses the machine.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// Image-only, so Recognise text is enabled.
const FIXTURE: &str = "fixtures/synthetic-image-only.pdf";

// ui-text-exempt: trace region and event names, never displayed.
pub(super) const TAB: &str = "ribbon.tab.file";
pub(super) const ITEM: &str = "ribbon.item.file.ocr_models";
pub(super) const COLLAPSED: &str = "ribbon.group.file.recognise.collapsed";
pub(super) const LISTED: &str = "ocr-models-listed";
/// The manager's whole window; the wheel aimed at its centre scrolls the list.
const BODY: &str = "ocr-models.body";
/// The list's per-model regions, the only ones scrolled to. The question, the
/// result and the buttons acting on the whole list are pinned below it, so a
/// check reaching one of those by scrolling would pass on a window that hid it.
const ROWS: [&str; 6] = [
    "ocr-models.row.",
    "ocr-models.missing.",
    "ocr-models.download.",
    "ocr-models.default.",
    "ocr-models.is-default.",
    "ocr-models.remove.",
];
/// Wheel notches tried, from the top of the list down, before a region is
/// called absent.
const REVEAL_NOTCHES: usize = 12;

/// A program folder: `exe` beside `models/` and `userdata/`.
pub(super) struct Program {
    pub(super) exe: PathBuf,
    pub(super) models: PathBuf,
    pub(super) userdata: PathBuf,
    /// The binary's own `models/`, where planted folders are copied from.
    pub(super) reference: PathBuf,
}

impl Program {
    /// A fresh folder `ctx.out(tag)` with a copy of the binary, an empty
    /// `models/`, and preferences holding `prefs`.
    pub(super) fn new(ctx: &CheckContext, tag: &str, prefs: &str) -> Result<Self> {
        let exe = ctx.resolve_exe().ok_or_else(|| {
            Error::new(format!(
                "no binary to drive. Pass --exe, or build the profile's default at {}.",
                ctx.profile.default_exe
            ))
        })?;
        let reference = exe
            .parent()
            .map(|d| d.join("models"))
            .ok_or_else(|| Error::new("the binary has no folder"))?;
        let dir = std::path::absolute(ctx.out(tag))
            .map_err(|e| Error::new(format!("cannot make the folder absolute: {e}")))?;
        let _ = std::fs::remove_dir_all(&dir);
        let models = dir.join("models");
        std::fs::create_dir_all(&models)
            .map_err(|e| Error::new(format!("making {}: {e}", models.display())))?;
        let copy = dir.join(exe.file_name().unwrap_or_default());
        std::fs::copy(&exe, &copy).map_err(|e| Error::new(format!("copying the binary: {e}")))?;
        let userdata = dir.join("userdata");
        crate::sandbox::write_prefs(&userdata, prefs)
            .map_err(|e| Error::new(format!("could not write preferences: {e}")))?;
        Ok(Self {
            exe: copy,
            models,
            userdata,
            reference,
        })
    }

    /// Copy the binary's own `models/<name>` to `to/<name>`.
    pub(super) fn copy_reference(&self, name: &str, to: &Path) -> Result<PathBuf> {
        let from = self.reference.join(name);
        if !from.is_dir() {
            return Err(Error::new(format!(
                "{} is not there; package or copy the bundled models beside the binary.",
                from.display()
            )));
        }
        let dest = to.join(name);
        copy_tree(&from, &dest)?;
        Ok(dest)
    }

    /// The preferences file as the program last wrote it.
    pub(super) fn prefs(&self) -> String {
        std::fs::read_to_string(self.userdata.join("preferences.txt")).unwrap_or_default()
    }
}

/// Copy a folder of files and folders.
pub(super) fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    let io = |p: &Path, e: std::io::Error| Error::new(format!("copying {}: {e}", p.display()));
    std::fs::create_dir_all(to).map_err(|e| io(to, e))?;
    for entry in std::fs::read_dir(from).map_err(|e| io(from, e))? {
        let entry = entry.map_err(|e| io(from, e))?;
        let path = entry.path();
        let target = to.join(entry.file_name());
        if path.is_dir() {
            copy_tree(&path, &target)?;
        } else {
            std::fs::copy(&path, &target).map_err(|e| io(&path, e))?;
        }
    }
    Ok(())
}

/// Bytes under `dir`, counted independently of the program.
pub(super) fn bytes_under(dir: &Path) -> u64 {
    std::fs::read_dir(dir).map_or(0, |entries| {
        entries
            .flatten()
            .map(|e| {
                let p = e.path();
                if p.is_dir() {
                    bytes_under(&p)
                } else {
                    e.metadata().map_or(0, |m| m.len())
                }
            })
            .sum()
    })
}

/// A running copy and its pointer.
pub(super) struct Drive {
    pub(super) session: Session,
    pub(super) pointer: ScriptedPointer,
    pub(super) ui_rect: &'static str,
    pub(super) path: String,
}

impl Drive {
    /// Launch `program` on the fixture, off-screen, with `env` added.
    pub(super) fn launch(
        ctx: &CheckContext,
        program: &Program,
        tag: &str,
        env: &[(&str, String)],
        report: &mut CheckReport,
    ) -> Result<Self> {
        let viewport_env = ctx
            .profile
            .viewport_env
            .ok_or_else(|| Error::new("the profile has no viewport variable."))?;
        let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");
        let pdf = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(FIXTURE);
        if !pdf.is_file() {
            return Err(Error::new(format!("the fixture {FIXTURE} is not on disk.")));
        }
        let mut spec = LaunchSpec::new(&program.exe, ctx.out(&format!("{tag}.trace.txt")));
        spec.pdf = Some(pdf);
        for (k, v) in [
            (ctx.profile.diag_env.0, ctx.profile.diag_env.1.to_owned()),
            (SHELL_DIAG_ENV.0, SHELL_DIAG_ENV.1.to_owned()),
            (viewport_env, OFFSCREEN.to_owned()),
        ] {
            spec.env.push((k.to_owned(), v));
        }
        for (k, v) in env {
            spec.env.push(((*k).to_owned(), v.clone()));
        }
        spec.place = false;
        spec.allow_stale = ctx.allow_stale;
        spec.source_root = ctx.source_root.clone();
        let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{tag}.pointer.txt")))?;
        let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
        report.artifact(session.trace_path().to_path_buf());
        report.artifact(pointer.path().to_path_buf());
        let path = session.trace_path().display().to_string();
        session.settle(45);
        Ok(Self {
            session,
            pointer,
            ui_rect,
            path,
        })
    }

    /// Whether `region` has been declared, scrolling the manager's list to
    /// find it when it is one of the list's rows.
    pub(super) fn has(&self, region: &str) -> Result<bool> {
        self.reveal(region)?;
        Ok(declared(&self.session.trace()?, self.ui_rect, region).is_some())
    }

    /// Scrolls the manager's list until the row region `region` is in view:
    /// to the top, then a notch at a time down. The list is longer than the
    /// window, so a row can be above or below it. Anything not in [`ROWS`] is
    /// left alone.
    fn reveal(&self, region: &str) -> Result<()> {
        if !ROWS.iter().any(|p| region.starts_with(p)) {
            return Ok(());
        }
        for notch in 0..=REVEAL_NOTCHES {
            let trace = self.session.trace()?;
            if declared(&trace, self.ui_rect, region).is_some() {
                return Ok(());
            }
            let Some((body, viewport)) = declared_in(&trace, self.ui_rect, BODY) else {
                return Ok(());
            };
            let dy = if notch == 0 { 100.0 } else { -2.0 };
            self.pointer.wheel_in(
                &self.session,
                viewport.as_deref(),
                WindowPoint::centre_of(body),
                dy,
            )?;
            self.session.settle(12);
        }
        Ok(())
    }

    /// Click the centre of `region`, in whichever window declared it.
    pub(super) fn click(&self, region: &str) -> Result<()> {
        self.reveal(region)?;
        let trace = self.session.trace()?;
        let (rect, viewport) = declared_in(&trace, self.ui_rect, region).ok_or_else(|| {
            let prefix = region.rsplit_once('.').map_or(region, |(p, _)| p);
            Error::new(format!(
                "no `{region}` region. Declared under `{prefix}`: {}. Trace: {}.",
                list(&declared_names(&trace, self.ui_rect, prefix)),
                self.path
            ))
        })?;
        self.pointer.click_in(
            &self.session,
            viewport.as_deref(),
            WindowPoint::centre_of(rect),
        )?;
        self.session.settle(15);
        Ok(())
    }

    /// The newest trace line number, to wait for lines after it.
    pub(super) fn mark(&self) -> Result<usize> {
        Ok(self.session.trace()?.mark())
    }

    /// The first `event` after `mark`, waiting up to `polls` x 20 frames.
    pub(super) fn wait(&self, event: &str, mark: usize, polls: u32) -> Result<Option<TraceLine>> {
        for _ in 0..polls {
            if let Some(line) = self.session.trace()?.last_after(event, mark) {
                return Ok(Some(line.clone()));
            }
            self.session.settle(20);
        }
        Ok(self.session.trace()?.last_after(event, mark).cloned())
    }

    /// File > Recognise > Manage OCR models…, through the ribbon. `None`
    /// when the item is not there to click.
    pub(super) fn open_manager(&self) -> Result<Option<TraceLine>> {
        self.click(TAB)?;
        if !self.has(ITEM)? && self.has(COLLAPSED)? {
            self.click(COLLAPSED)?;
        }
        if !self.has(ITEM)? {
            return Ok(None);
        }
        let mark = self.mark()?;
        self.click(ITEM)?;
        self.wait(LISTED, mark, 10)
    }

    /// Park the pointer; the first error wins.
    pub(super) fn finish(self, outcome: Result<Option<String>>) -> Result<Option<String>> {
        let parked = self.pointer.gone(&self.session);
        let outcome = outcome?;
        if outcome.is_none() {
            parked?;
        }
        Ok(outcome)
    }
}

/// The comma list a trace field holds; empty for `none`.
pub(super) fn names(line: &TraceLine, key: &str) -> Vec<String> {
    match line.get(key) {
        None | Some("none") => Vec::new(),
        Some(v) => v.split(',').map(str::to_owned).collect(),
    }
}
