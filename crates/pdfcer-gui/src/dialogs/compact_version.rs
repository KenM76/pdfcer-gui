//! # `dialogs::compact_version` — *Make compatible with* in Save a compacted copy
//!
//! Contract: [`row`] draws the version list; [`Lowering`] runs
//! `pdfcer_core::optimize::lower_version` on the bytes the window would
//! otherwise save, off the UI thread, and [`lines`] says what it did. The
//! list starts at *Keep*, because ISO 32000-2 Annex I.2 says a writer should
//! not change a document to an older version: lowering happens only when
//! asked. The engine refuses when the content needs more than the version
//! chosen, and [`lower`] carries the copy's floor and the constructs that
//! set it so the refusal says why.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/compact_version.md`.

use std::sync::mpsc::{Receiver, TryRecvError, channel};

use egui::Ui;
use pdfcer_core::PdfVersion;
use pdfcer_core::document::Document;
use pdfcer_core::optimize::{LowerError, LowerOptions, LowerRoute, lower_version, version_floor};
use pdfcer_core::writer::SaveOptions;

use crate::text::compact_version as t;

/// The version list; each entry is `<this>.<index>`, `0` being *Keep*.
// ui-text-exempt: trace region name, never displayed
pub const REGION_VERSION: &str = "compact.version";
/// Each line saying what lowering did.
// ui-text-exempt: trace region name, never displayed
pub const REGION_VERSION_SAID: &str = "compact.version.said";
/// The versions offered after *Keep*. 1.3 is not offered: the engine's
/// floor starts there, so nothing below it can be proven.
pub const TARGETS: [PdfVersion; 4] = [
    PdfVersion { major: 1, minor: 4 },
    PdfVersion { major: 1, minor: 5 },
    PdfVersion { major: 1, minor: 6 },
    PdfVersion { major: 1, minor: 7 },
];
/// How many constructs are named under a floor.
const REASONS: usize = 3;

/// What lowering the copy did.
#[derive(Debug)]
pub struct Lowered {
    /// The file to save; `None` when the engine refused, so the copy is
    /// saved at its own version.
    pub bytes: Option<Vec<u8>>,
    /// One sentence each: the change, or the refusal, then why.
    pub lines: Vec<String>,
    /// The trace's word for the outcome.
    pub word: &'static str,
}

/// Lower `copy` to `target`. Never fails: a refusal is an answer, said in
/// [`Lowered::lines`], and the copy is kept as it was.
#[must_use]
pub fn lower(copy: Vec<u8>, target: PdfVersion, save: &SaveOptions) -> Lowered {
    let floor = Document::from_bytes(copy.clone())
        .ok()
        .map(|d| version_floor(&d));
    // A signed copy proceeds: the window already says the copy keeps no
    // signatures, and changing the header is one more reason they stop
    // verifying.
    let options = LowerOptions::default().with_allow_signed(true);
    match lower_version(copy, target, &options, save) {
        Ok(done) => {
            let mut lines = vec![t::lowered(&done.from, &done.to)];
            if done.catalog_version_removed {
                lines.push(t::catalog_removed().to_owned());
            }
            if matches!(done.route, LowerRoute::Rebuilt(_)) {
                lines.push(t::rebuilt().to_owned());
            }
            lines.extend(reasons(&done.floor));
            Lowered {
                bytes: Some(done.bytes),
                lines,
                word: "lowered", // ui-text-exempt: diagnostic trace
            }
        }
        Err(why) => {
            let mut lines = vec![refusal(&why)];
            if matches!(why, LowerError::BelowFloor { .. }) {
                lines.extend(floor.as_ref().map(reasons).unwrap_or_default());
            }
            Lowered {
                bytes: None,
                lines,
                word: refusal_word(&why),
            }
        }
    }
}

/// The constructs that set the floor, highest first.
fn reasons(floor: &pdfcer_core::optimize::VersionFloor) -> Vec<String> {
    floor
        .requirements
        .iter()
        .take(REASONS)
        .map(|r| t::needs(&r.version, &r.construct, r.objects))
        .collect()
}

/// The refusal in the window's words.
fn refusal(why: &LowerError) -> String {
    match why {
        LowerError::BelowFloor { target, floor, .. } => t::below_floor(target, floor),
        LowerError::NotLower { target, declared } => t::not_lower(target, declared),
        LowerError::Unproven { floor, unreadable } => t::unproven(floor, *unreadable),
        LowerError::Encrypted => t::encrypted().to_owned(),
        other => t::failed(&other.to_string()),
    }
}

/// The trace's word for a refusal.
const fn refusal_word(why: &LowerError) -> &'static str {
    match why {
        LowerError::BelowFloor { .. } => "below-floor", // ui-text-exempt: diagnostic trace
        LowerError::NotLower { .. } => "not-lower",     // ui-text-exempt: diagnostic trace
        LowerError::Unproven { .. } => "unproven",      // ui-text-exempt: diagnostic trace
        LowerError::Encrypted => "encrypted",           // ui-text-exempt: diagnostic trace
        _ => "failed",                                  // ui-text-exempt: diagnostic trace
    }
}

/// Where lowering the copy has got to.
#[derive(Default)]
pub enum Lowering {
    /// Not asked for, or waiting for the bytes it works on.
    #[default]
    Off,
    /// Running on its own thread.
    Working(Receiver<Lowered>),
    /// Finished.
    Done(Box<Lowered>),
}

impl Lowering {
    /// Start lowering `copy` to `target`; the window repaints when it is done.
    pub fn start(
        ctx: &egui::Context,
        copy: Vec<u8>,
        target: PdfVersion,
        save: SaveOptions,
    ) -> Self {
        let (tx, rx) = channel();
        let ctx = ctx.clone();
        let spawned = std::thread::Builder::new()
            .name("compact-lower".to_owned())
            .spawn(move || {
                let started = std::time::Instant::now();
                let done = lower(copy, target, &save);
                crate::diag::trace(|| {
                    format!(
                        "compact-lowered to={target} outcome={} bytes={} ms={}", // ui-text-exempt: diagnostic trace
                        done.word,
                        done.bytes.as_ref().map_or(0, Vec::len),
                        started.elapsed().as_millis()
                    )
                });
                let _ = tx.send(done);
                ctx.request_repaint();
            });
        match spawned {
            Ok(_) => Self::Working(rx),
            Err(error) => Self::Done(Box::new(Lowered {
                bytes: None,
                lines: vec![t::failed(&error.to_string())],
                word: "failed", // ui-text-exempt: diagnostic trace
            })),
        }
    }

    /// Take the result when it has arrived.
    pub fn step(&mut self) {
        let Self::Working(rx) = self else {
            return;
        };
        let landed = match rx.try_recv() {
            Ok(done) => done,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Lowered {
                bytes: None,
                lines: vec![t::stopped().to_owned()],
                word: "failed", // ui-text-exempt: diagnostic trace
            },
        };
        *self = Self::Done(Box::new(landed));
    }

    /// The lowered bytes, when there are some.
    #[must_use]
    pub fn bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Done(done) => done.bytes.as_deref(),
            _ => None,
        }
    }

    /// Move the lowered bytes out, when there are some.
    pub fn take_bytes(&mut self) -> Option<Vec<u8>> {
        match std::mem::take(self) {
            Self::Done(done) => done.bytes,
            _ => None,
        }
    }
}

/// The *Make compatible with* list; whether the choice changed.
pub fn row(ui: &mut Ui, target: &mut Option<PdfVersion>) -> bool {
    let before = *target;
    ui.horizontal(|ui| {
        ui.label(t::label()).on_hover_text(t::tooltip());
        let choices = std::iter::once(None).chain(TARGETS.into_iter().map(Some));
        let list = egui::ComboBox::from_id_salt("compact-version") // ui-text-exempt: an id, never displayed
            .selected_text(t::choice(*target))
            .show_ui(ui, |ui| {
                for (i, choice) in choices.enumerate() {
                    let item = ui.selectable_value(target, choice, t::choice(choice));
                    let region = format!("{REGION_VERSION}.{i}");
                    crate::diag::ui_rect_visible(&region, item.rect, ui.clip_rect());
                }
            });
        crate::diag::ui_rect_visible(REGION_VERSION, list.response.rect, ui.clip_rect());
        list.response.on_hover_text(t::tooltip());
    });
    *target != before
}

/// What lowering did, or that it is running.
pub fn lines(ui: &mut Ui, lowering: &Lowering) {
    match lowering {
        Lowering::Off => {}
        Lowering::Working(_) => {
            ui.small(t::working());
        }
        Lowering::Done(done) => {
            for line in &done.lines {
                let said = ui.small(line.as_str());
                crate::diag::ui_rect_visible(REGION_VERSION_SAID, said.rect, ui.clip_rect());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const V14: PdfVersion = PdfVersion { major: 1, minor: 4 };

    /// A one-page file declaring 1.7, with `page_extra` added to its page.
    fn file(page_extra: &str) -> Vec<u8> {
        let bodies = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
            format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200]{page_extra} >>"),
        ];
        let mut buf = b"%PDF-1.7\n".to_vec();
        let mut offsets = Vec::new();
        for (i, body) in bodies.iter().enumerate() {
            offsets.push(buf.len());
            buf.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
        }
        let xref = buf.len();
        buf.extend_from_slice(b"xref\n0 4\n0000000000 65535 f \n");
        for o in offsets {
            buf.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
        }
        buf.extend_from_slice(
            format!("trailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n").as_bytes(),
        );
        buf
    }

    #[test]
    fn a_plain_file_is_lowered_and_says_from_and_to() {
        let done = lower(file(""), V14, &SaveOptions::default());
        assert_eq!(done.word, "lowered", "{:?}", done.lines);
        let bytes = done.bytes.expect("lowered");
        assert!(bytes.starts_with(b"%PDF-1.4"), "{:?}", &bytes[..8]);
        assert!(done.lines[0].contains("1.7") && done.lines[0].contains("1.4"));
    }

    #[test]
    fn a_target_not_below_the_declared_version_is_refused_and_keeps_the_copy() {
        let done = lower(
            file(""),
            PdfVersion { major: 1, minor: 7 },
            &SaveOptions::default(),
        );
        assert!(done.bytes.is_none());
        assert_eq!(done.word, "not-lower");
    }

    /// `/UserUnit` arrived in 1.6: 1.4 is refused, the construct is named,
    /// and 1.6 is allowed.
    #[test]
    fn content_needing_more_is_refused_with_its_reason() {
        let done = lower(file(" /UserUnit 2"), V14, &SaveOptions::default());
        assert_eq!(done.word, "below-floor", "{:?}", done.lines);
        assert!(done.bytes.is_none());
        assert!(
            done.lines.iter().any(|l| l.contains("UserUnit")),
            "{:?}",
            done.lines
        );
        let allowed = lower(
            file(" /UserUnit 2"),
            PdfVersion { major: 1, minor: 6 },
            &SaveOptions::default(),
        );
        assert_eq!(allowed.word, "lowered", "{:?}", allowed.lines);
    }
}
