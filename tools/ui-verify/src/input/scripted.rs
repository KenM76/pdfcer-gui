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

    /// Move the pointer to `at` in the root viewport, pressing nothing.
    pub fn hover(&self, session: &Session, at: WindowPoint) -> Result<TraceLine> {
        self.send(session, &format!("move {}", xy(at)))
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

    /// Three primary clicks, one frame apart.
    pub fn triple_click(&self, session: &Session, at: WindowPoint) -> Result<TraceLine> {
        self.send(session, &format!("tclick {}", xy(at)))
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

    /// [`Self::drag`] in `viewport` (root when `None`), with button `btn`
    /// (`l`, `r` or `m`).
    pub fn drag_in(
        &self,
        session: &Session,
        viewport: Option<&str>,
        from: WindowPoint,
        to: WindowPoint,
        steps: u32,
        btn: &str,
    ) -> Result<TraceLine> {
        let vp = viewport.map_or(String::new(), |v| format!(" vp={v}"));
        self.send(
            session,
            &format!("drag {} {} steps={steps} btn={btn}{vp}", xy(from), xy(to)),
        )
    }

    /// [`Self::wheel`] in `viewport` (root when `None`), without modifiers.
    pub fn wheel_in(
        &self,
        session: &Session,
        viewport: Option<&str>,
        at: WindowPoint,
        dy: f32,
    ) -> Result<TraceLine> {
        let vp = viewport.map_or(String::new(), |v| format!(" vp={v}"));
        self.send(session, &format!("wheel {} {dy}{vp}", xy(at)))
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

    /// Press and release one key in `viewport` (root when `None`), spelled as
    /// `egui::Key::from_name` spells it, with `mods` (`ctrl`, `shift`, `alt`,
    /// joined by `+`) or none.
    pub fn key(
        &self,
        session: &Session,
        viewport: Option<&str>,
        name: &str,
        mods: Option<&str>,
    ) -> Result<TraceLine> {
        let mods = mods.map_or(String::new(), |m| format!(" mods={m}"));
        let vp = viewport.map_or(String::new(), |v| format!(" vp={v}"));
        self.send(session, &format!("key {name}{mods}{vp}"))
    }

    /// Deliver `text` as typed in `viewport` (root when `None`), to whatever
    /// holds keyboard focus there.
    pub fn type_text(
        &self,
        session: &Session,
        viewport: Option<&str>,
        text: &str,
    ) -> Result<TraceLine> {
        match viewport {
            Some(vp) => self.send(session, &format!("type vp={vp} {text}")),
            None => self.send(session, &format!("type {text}")),
        }
    }

    /// Deliver `text` as pasted from the clipboard in `viewport` (root when
    /// `None`); line breaks and tabs survive the one-line step.
    pub fn paste(
        &self,
        session: &Session,
        viewport: Option<&str>,
        text: &str,
    ) -> Result<TraceLine> {
        let text = text
            .replace('\\', "\\\\")
            .replace('\n', "\\n")
            .replace('\t', "\\t");
        match viewport {
            Some(vp) => self.send(session, &format!("paste vp={vp} {text}")),
            None => self.send(session, &format!("paste {text}")),
        }
    }

    /// Deliver an input method's composition in progress in the root viewport.
    pub fn ime_preedit(&self, session: &Session, text: &str) -> Result<TraceLine> {
        self.send(session, &format!("preedit {text}"))
    }

    /// Deliver an input method's committed text in the root viewport.
    pub fn ime_commit(&self, session: &Session, text: &str) -> Result<TraceLine> {
        self.send(session, &format!("commit {text}"))
    }

    /// Deliver the platform's Copy command in `viewport` (root when `None`),
    /// as Ctrl+C reaches the app through the windowing layer.
    pub fn copy(&self, session: &Session, viewport: Option<&str>) -> Result<TraceLine> {
        match viewport {
            Some(vp) => self.send(session, &format!("copy vp={vp}")),
            None => self.send(session, "copy"),
        }
    }

    /// Deliver the platform's Cut command, as [`Self::copy`] does Copy.
    pub fn cut(&self, session: &Session, viewport: Option<&str>) -> Result<TraceLine> {
        match viewport {
            Some(vp) => self.send(session, &format!("cut vp={vp}")),
            None => self.send(session, "cut"),
        }
    }

    /// Take the pointer off the window, so nothing stays hovered.
    pub fn gone(&self, session: &Session) -> Result<TraceLine> {
        self.send(session, "gone")
    }

    /// Write the window's own rendered frame to `png`: egui's screenshot,
    /// which sees a window placed off the desktop where an OS capture sees
    /// whatever is on screen there.
    pub fn screenshot(&self, session: &Session, png: &Path) -> Result<()> {
        self.screenshot_in(session, None, png)
    }

    /// [`Self::screenshot`] of `viewport` (root when `None`), such as a
    /// dialog's own window.
    pub fn screenshot_in(
        &self,
        session: &Session,
        viewport: Option<&str>,
        png: &Path,
    ) -> Result<()> {
        let ack = match viewport {
            Some(vp) => self.send(session, &format!("shot vp={vp}"))?,
            None => self.send(session, "shot")?,
        };
        let Some(ppm) = ack.get("path") else {
            return Err(Error::new(format!(
                "the screenshot was not written: `{}`",
                ack.raw
            )));
        };
        let bytes = std::fs::read(ppm)
            .map_err(|e| Error::new(format!("could not read the screenshot {ppm}: {e}")))?;
        let (w, h, rgb) = parse_ppm(&bytes)
            .ok_or_else(|| Error::new(format!("{ppm} is not a binary PPM the seam writes")))?;
        let encoded = crate::png::encode_rgb(w, h, rgb)
            .ok_or_else(|| Error::new(format!("could not encode {ppm} as PNG")))?;
        std::fs::write(png, encoded)
            .map_err(|e| Error::new(format!("could not write {}: {e}", png.display())))?;
        let _ = std::fs::remove_file(ppm);
        Ok(())
    }

    /// Drop `files` at `at` in the root viewport with `mods` held, as a file
    /// dragged from another program lands.
    pub fn drop_files(
        &self,
        session: &Session,
        at: WindowPoint,
        mods: Option<&str>,
        files: &[&Path],
    ) -> Result<TraceLine> {
        let mods = mods.map_or(String::new(), |m| format!(" mods={m}"));
        let list: Vec<String> = files.iter().map(|p| p.display().to_string()).collect();
        self.send(
            session,
            &format!("drop {}{mods} {}", xy(at), list.join("|")),
        )
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

/// Width, height and pixels of a `P6` file with a maxval of 255 and no
/// comments — the only form the seam writes.
fn parse_ppm(bytes: &[u8]) -> Option<(u32, u32, &[u8])> {
    let mut fields = Vec::new();
    let mut at = 0;
    while fields.len() < 4 {
        while bytes.get(at)?.is_ascii_whitespace() {
            at += 1;
        }
        let start = at;
        while !bytes.get(at)?.is_ascii_whitespace() {
            at += 1;
        }
        fields.push(std::str::from_utf8(&bytes[start..at]).ok()?);
    }
    let pixels = bytes.get(at + 1..)?;
    let (w, h): (u32, u32) = (fields[1].parse().ok()?, fields[2].parse().ok()?);
    let ok =
        fields[0] == "P6" && fields[3] == "255" && pixels.len() == (w as usize) * (h as usize) * 3;
    ok.then_some((w, h, pixels))
}

fn xy(p: WindowPoint) -> String {
    format!("{:.1} {:.1}", p.x(), p.y())
}

#[cfg(test)]
mod tests {
    use super::parse_ppm;

    #[test]
    fn a_ppm_the_seam_writes_reads_back_and_a_short_one_is_refused() {
        let mut bytes = b"P6\n2 1\n255\n".to_vec();
        bytes.extend_from_slice(&[1, 2, 3, 4, 5, 6]);
        assert_eq!(parse_ppm(&bytes), Some((2, 1, &[1u8, 2, 3, 4, 5, 6][..])));
        bytes.pop();
        assert_eq!(parse_ppm(&bytes), None);
        assert_eq!(parse_ppm(b"P5\n1 1\n255\n\0"), None);
    }
}
