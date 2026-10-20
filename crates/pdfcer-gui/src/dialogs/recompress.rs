//! # `dialogs::recompress` — the compacted copy, recompressed
//!
//! Contract: [`recompress`] runs the engine's lossless optimize preset on a
//! scratch session parsed from the compacted copy, so the open document and
//! its undo history are untouched. The preset re-encodes streams and rebuilds
//! the structure: unused parts dropped, duplicates merged, the index packed.
//! It answers the smaller file with what the plan left alone and what the
//! rebuild did. A signed copy proceeds: the compact window already states
//! that the copy keeps no signatures. [`Job`] runs it off the UI thread.

use std::collections::BTreeMap;
use std::sync::mpsc::{Receiver, TryRecvError, channel};

use pdfcer_core::document::Document;
use pdfcer_core::optimize::{OptimizedFile, Preset};
use pdfcer_core::settings::Settings;

use crate::app::settings::SettingsExt;
use crate::text::compact as t;

/// What recompressing the copy did.
#[derive(Debug)]
pub struct Recompressed {
    /// The smaller file; `None` when the result was no smaller than the copy,
    /// which is then kept.
    pub bytes: Option<Vec<u8>>,
    /// Streams re-encoded.
    pub changed: usize,
    /// Why selected streams were left as stored, with how many each.
    pub left: BTreeMap<String, usize>,
    /// The engine's own disclosures.
    pub notes: Vec<String>,
    /// What the structure rebuild did, one sentence each.
    pub restructured: Vec<String>,
}

/// Recompress `copy` in a session and with save options built from the
/// operator's `settings`.
///
/// # Errors
///
/// The engine's sentence when the copy does not parse, the plan is refused
/// (an encrypted copy) or the rewrite fails.
pub fn recompress(copy: &[u8], settings: &Settings) -> Result<Recompressed, String> {
    let doc = Document::from_bytes(copy.to_vec()).map_err(|e| e.to_string())?;
    let mut session = settings.open_session(doc);
    let options = settings.save_options();
    let mut preset = Preset::Lossless.settings();
    preset.allow_signed = true;
    let plan = session.plan_optimize(&preset).map_err(|e| e.to_string())?;
    session.apply_optimize(&plan).map_err(|e| e.to_string())?;
    let (file, report) = session
        .save_optimized(&options, &preset.structure)
        .map_err(|e| e.to_string())?;
    let mut left = BTreeMap::new();
    for skipped in &plan.skipped {
        *left.entry(t::left_because(&skipped.reason)).or_insert(0) += 1;
    }
    Ok(Recompressed {
        bytes: match file {
            OptimizedFile::Smaller(bytes) => Some(bytes),
            _ => None,
        },
        changed: plan.changes.len(),
        left,
        notes: plan.notes,
        restructured: report.as_ref().map(t::restructured).unwrap_or_default(),
    })
}

/// [`recompress`] on its own thread.
pub struct Job {
    rx: Receiver<Result<Recompressed, String>>,
}

impl Job {
    /// Start recompressing `copy`; the window repaints when it is done.
    pub fn start(ctx: &egui::Context, copy: Vec<u8>, settings: Settings) -> Self {
        let (tx, rx) = channel();
        let ctx = ctx.clone();
        let spawned = std::thread::Builder::new()
            .name("compact-recompress".to_owned())
            .spawn(move || {
                let started = std::time::Instant::now();
                let result = recompress(&copy, &settings);
                crate::diag::trace(|| {
                    format!(
                        "compact-recompressed ok={} smaller={} changed={} ms={}", // ui-text-exempt: diagnostic trace
                        result.is_ok(),
                        result.as_ref().is_ok_and(|r| r.bytes.is_some()),
                        result.as_ref().map_or(0, |r| r.changed),
                        started.elapsed().as_millis()
                    )
                });
                let _ = tx.send(result);
                ctx.request_repaint();
            });
        if let Err(error) = spawned {
            let (tx, rx) = channel();
            let _ = tx.send(Err(error.to_string()));
            return Self { rx };
        }
        Self { rx }
    }

    /// The result, once there is one.
    pub fn poll(&self) -> Option<Result<Recompressed, String>> {
        match self.rx.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Err(t::recompress_stopped().to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A one-page file whose content stream is stored unfiltered, with one
    /// object nothing refers to.
    fn unfiltered() -> Vec<u8> {
        let page = "0 g 10 10 20 20 re f\n".repeat(200);
        let bodies = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 40] /Contents 4 0 R >>".to_string(),
            format!("<< /Length {} >>\nstream\n{page}\nendstream", page.len()),
            "<< /Unused true >>".to_string(),
        ];
        let mut buf = b"%PDF-1.7\n".to_vec();
        let mut offsets = Vec::new();
        for (i, body) in bodies.iter().enumerate() {
            offsets.push(buf.len());
            buf.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
        }
        let xref = buf.len();
        buf.extend_from_slice(
            format!("xref\n0 {}\n0000000000 65535 f \n", bodies.len() + 1).as_bytes(),
        );
        for o in offsets {
            buf.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
        }
        buf.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
                bodies.len() + 1
            )
            .as_bytes(),
        );
        buf
    }

    /// The unfiltered stream is re-encoded, the unused object is dropped, the
    /// file comes out smaller, and it opens with the same page content.
    #[test]
    fn an_unfiltered_stream_is_recompressed_into_a_smaller_copy() {
        let copy = unfiltered();
        let done = recompress(&copy, &Settings::default()).expect("recompresses");
        assert_eq!(done.changed, 1, "{done:?}");
        assert!(
            done.restructured.iter().any(|l| l.starts_with("1 unused")),
            "the unused object is dropped and said so: {:?}",
            done.restructured
        );
        let bytes = done.bytes.expect("smaller");
        assert!(bytes.len() < copy.len());
        let after = Document::from_bytes(bytes).expect("parses back");
        let view = after.view();
        let page = "0 g 10 10 20 20 re f\n".repeat(200).into_bytes();
        let stored = after.objects().find_map(|o| match &o.value {
            pdfcer_core::object::Object::Stream(s) => {
                let raw = view.slice(s.data_span)?;
                let decoded = pdfcer_core::filters::decode_stream(&s.dict, raw).ok()?;
                (decoded == page).then(|| (raw.len(), s.dict.get(b"Filter").is_some()))
            }
            _ => None,
        });
        let (raw, filtered) = stored.expect("the page content decodes unchanged");
        assert!(
            filtered && raw < page.len(),
            "stored {raw} B, filtered {filtered}"
        );
    }
}
