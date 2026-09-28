//! Pointer steps delivered through the application's `PDFCER_DIAG_POINTER`
//! seam instead of the operating system, so a window placed off the desktop
//! can be clicked, dragged and wheeled while the operator uses the machine.
//!
//! Each step is appended to a file the application follows, and waits for
//! the application's `diag-pointer seq=N` acknowledgement. The acknowledgement
//! means the events were delivered, not what they did; read the effect from
//! the application's own trace. Points are [`WindowPoint`]s: egui logical
//! points, the space `ui-rect` lines are written in, with no conversion to the
//! desktop.
//!
//! What this cannot test: anything decided below egui — OS focus and
//! activation, capture, OS drag-and-drop, the focus-chain regressions of
//! `docs/modules/ui-verify/input.md` D1. Those checks keep [`super::Driver`].
//!
//! Design and rationale: `docs/modules/ui-verify/input.md` § Scripted pointer.

use std::cell::Cell;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::trace::TraceLine;

/// The variable naming the step file.
pub const POINTER_ENV: &str = "PDFCER_DIAG_POINTER";
/// The acknowledgement of a delivered step.
const ACK: &str = "diag-pointer"; // ui-text-exempt: a trace event name, never displayed
/// The answer to a line the application could not read.
const REFUSED: &str = "diag-pointer-refused"; // ui-text-exempt: a trace event name
/// How long a step may take to be delivered. The application polls every
/// 50 ms and a step is at most a dozen frames, so this is only reached when
/// it has stopped drawing.
const ACK_TIMEOUT: Duration = Duration::from_secs(10);

/// A step file and the sequence numbers issued into it.
pub struct ScriptedPointer {
    path: PathBuf,
    seq: Cell<u64>,
}

impl ScriptedPointer {
    /// Create (or empty) the step file at `path` and name it in `spec`.
    pub fn attach(spec: &mut LaunchSpec, path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        std::fs::write(&path, b"").map_err(|e| {
            Error::new(format!(
                "could not create the pointer file {}: {e}",
                path.display()
            ))
        })?;
        spec.env
            .push((POINTER_ENV.to_owned(), path.display().to_string()));
        Ok(Self {
            path,
            seq: Cell::new(0),
        })
    }

    /// The step file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// A primary click at `at` in the root viewport.
    pub fn click(&self, session: &Session, at: WindowPoint) -> Result<TraceLine> {
        self.send(session, &format!("click {}", xy(at)))
    }

    /// A primary click at `at` in the viewport a `ui-rect` line named with
    /// `viewport=`, or the root one when it named none. A dialog is its own
    /// OS window and its rects are relative to it.
    pub fn click_in(
        &self,
        session: &Session,
        viewport: Option<&str>,
        at: WindowPoint,
    ) -> Result<TraceLine> {
        match viewport {
            Some(vp) => self.send(session, &format!("click {} vp={vp}", xy(at))),
            None => self.click(session, at),
        }
    }

    /// A secondary (right) click.
    pub fn right_click(&self, session: &Session, at: WindowPoint) -> Result<TraceLine> {
        self.send(session, &format!("click {} btn=r", xy(at)))
    }

    /// Two primary clicks, one frame apart.
    pub fn double_click(&self, session: &Session, at: WindowPoint) -> Result<TraceLine> {
        self.send(session, &format!("dclick {}", xy(at)))
    }

    /// Press at `from`, move in `steps` frames, release at `to`.
    pub fn drag(
        &self,
        session: &Session,
        from: WindowPoint,
        to: WindowPoint,
        steps: u32,
    ) -> Result<TraceLine> {
        self.send(
            session,
            &format!("drag {} {} steps={steps}", xy(from), xy(to)),
        )
    }

    /// Wheel lines at `at`; positive `dy` moves the content down. `ctrl`
    /// makes it a zoom.
    pub fn wheel(
        &self,
        session: &Session,
        at: WindowPoint,
        dy: f32,
        ctrl: bool,
    ) -> Result<TraceLine> {
        let mods = if ctrl { " mods=ctrl" } else { "" };
        self.send(session, &format!("wheel {} {dy}{mods}", xy(at)))
    }

    /// Take the pointer off the window, so nothing stays hovered.
    pub fn gone(&self, session: &Session) -> Result<TraceLine> {
        self.send(session, "gone")
    }

    /// Append one step in the seam's grammar (without the sequence number)
    /// and wait for its acknowledgement.
    pub fn send(&self, session: &Session, body: &str) -> Result<TraceLine> {
        let seq = self.seq.get() + 1;
        self.seq.set(seq);
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&self.path)
            .map_err(|e| Error::new(format!("could not open {}: {e}", self.path.display())))?;
        file.write_all(format!("{seq} {body}\n").as_bytes())
            .map_err(|e| Error::new(format!("could not append a pointer step: {e}")))?;
        drop(file);

        let want = seq.to_string();
        let deadline = Instant::now() + ACK_TIMEOUT;
        loop {
            let trace = session.trace()?;
            if let Some(line) = trace.events(REFUSED).find(|l| l.get("seq") == Some(&want)) {
                return Err(Error::new(format!(
                    "the application could not read pointer step `{seq} {body}`: `{}`",
                    line.raw
                )));
            }
            if let Some(line) = trace.events(ACK).find(|l| l.get("seq") == Some(&want)) {
                return Ok(line.clone());
            }
            if Instant::now() >= deadline {
                return Err(Error::new(format!(
                    "pointer step `{seq} {body}` was not acknowledged within {}s. Either the \
                     binary predates the `{POINTER_ENV}` seam (no `{ACK}` line has ever \
                     appeared: {}), or it stopped drawing frames.",
                    ACK_TIMEOUT.as_secs(),
                    if trace.first(ACK).is_some() {
                        "one has"
                    } else {
                        "none has"
                    }
                )));
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

fn xy(p: WindowPoint) -> String {
    format!("{:.1} {:.1}", p.x(), p.y())
}
