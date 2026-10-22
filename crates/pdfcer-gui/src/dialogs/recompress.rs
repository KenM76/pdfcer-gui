//! # `dialogs::recompress` — the compacted copy, optimized
//!
//! Contract: [`recompress`] runs the engine's lossless optimize preset, cut
//! down to the operator's [`Choices`], on a scratch session parsed from the
//! compacted copy, so the open document and its undo history are untouched.
//! Recompressing re-encodes streams and rebuilds the structure (unused parts
//! dropped, duplicates merged, the index packed); the clean-ups remove links,
//! bookmarks and named destinations that go nowhere or that nothing uses; the
//! discards remove whole classes of document object. It answers the smaller
//! file, or a file no smaller that removes what was asked for, with what the
//! plan left alone and what each step did. Pictures are changed only when
//! [`Choices::pictures`] is on, through `OptimizeSettings::images`; the
//! preset's own stream action stays, so everything else is still recompressed.
//! [`Choices::fonts`] cuts embedded TrueType fonts to the glyphs drawn
//! through `OptimizeSettings::resubset`. [`Choices::content`] tidies page
//! drawing instructions losslessly through `OptimizeSettings::content`, and
//! [`Choices::round`] adds `ContentPatch::round`, which moves line positions
//! by at most the distance chosen.
//! A signed copy proceeds: the compact window already states
//! that the copy keeps no signatures. [`Job`] runs it off the UI thread.

use std::collections::BTreeMap;
use std::sync::mpsc::{Receiver, TryRecvError, channel};

use pdfcer_core::document::Document;
use pdfcer_core::optimize::resubset::ResubsetFonts;
use pdfcer_core::optimize::{
    CleanupOptions, DiscardOptions, OptimizedFile, PlannedChange, Preset, RebuildOptions,
    StreamAction,
};
use pdfcer_core::optimize::{ContentPatch, Rounding, Tolerance};
use pdfcer_core::settings::Settings;
use pdfcer_render::font::resubset::GLYPH_ZERO_RESUBSETTER;

use crate::app::settings::SettingsExt;
use crate::dialogs::compact_images::{Pictures, Tally};
use crate::text::compact as t;

/// What the operator asked the copy to have done.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Choices {
    /// Re-encode streams and rebuild the structure.
    pub recompress: bool,
    /// Remove links and bookmarks whose destination is broken.
    pub broken: bool,
    /// Remove named destinations nothing in the file names.
    pub destinations: bool,
    /// The classes of document object removed; metadata kinds stay with
    /// File ▸ Remove metadata.
    pub discard: DiscardOptions,
    /// What is done to pictures; the only choice that changes how the page
    /// looks.
    pub pictures: Pictures,
    /// Cut each embedded TrueType font to the glyphs drawn, merging copies
    /// of one face.
    pub fonts: bool,
    /// Rewrite page drawing instructions in their shortest equivalent form.
    pub content: bool,
    /// Round path coordinates within this distance; acted on only with
    /// [`Self::content`], because rounding is part of the same rewrite.
    pub round: Option<Tolerance>,
}

impl Choices {
    /// Whether anything is asked for.
    #[must_use]
    pub const fn any(self) -> bool {
        self.recompress
            || self.broken
            || self.destinations
            || !self.discard.is_none()
            || self.pictures.on
            || self.fonts
            || self.content
    }
}

/// What optimizing the copy did.
#[derive(Debug)]
pub struct Recompressed {
    /// The file to save; `None` when the result was no smaller than the copy
    /// and removed nothing, so the copy is kept.
    pub bytes: Option<Vec<u8>>,
    /// The file is no smaller than the copy and is kept for what it removes.
    pub not_smaller: bool,
    /// Streams re-encoded.
    pub changed: usize,
    /// Why selected streams were left as stored, with how many each.
    pub left: BTreeMap<String, usize>,
    /// The engine's own disclosures.
    pub notes: Vec<String>,
    /// What the structure rebuild did, one sentence each.
    pub restructured: Vec<String>,
    /// What the clean-ups and discards removed, one sentence each.
    pub cleaned: Vec<String>,
    /// What was done to pictures.
    pub pictures: Tally,
    /// What cutting the fonts did, one sentence each.
    pub fonts: Vec<String>,
    /// Embedded fonts cut.
    pub fonts_cut: usize,
    /// What tidying the drawing instructions did, when it did anything.
    pub content: Option<String>,
    /// Parts whose drawing instructions were tidied.
    pub content_tidied: usize,
    /// Line positions rounded, summed over every part.
    pub content_rounded: u64,
    /// Why rounding left parts as they were, one line per reason.
    pub unrounded: Vec<String>,
}

/// Optimize `copy` as `choices` ask, in a session and with save options
/// built from the operator's `settings`.
///
/// # Errors
///
/// The engine's sentence when the copy does not parse, the plan is refused
/// (an encrypted copy) or the rewrite fails.
pub fn recompress(
    copy: &[u8],
    settings: &Settings,
    choices: Choices,
) -> Result<Recompressed, String> {
    let doc = Document::from_bytes(copy.to_vec()).map_err(|e| e.to_string())?;
    let mut session = settings.open_session(doc);
    let options = settings.save_options();
    let mut preset = Preset::Lossless.settings();
    preset.allow_signed = true;
    if !choices.recompress {
        preset.streams = StreamAction::Leave;
        preset.structure = RebuildOptions::none();
    }
    preset.cleanup = CleanupOptions {
        invalid_links: choices.broken,
        invalid_bookmarks: choices.broken,
        unreferenced_destinations: choices.destinations,
    };
    preset.discard = choices.discard;
    if let Some(patch) = choices.pictures.patch() {
        patch.validate()?;
        preset.images = patch;
    }
    if choices.fonts {
        preset.resubset = Some(ResubsetFonts::new(&GLYPH_ZERO_RESUBSETTER));
    }
    if choices.content {
        let mut patch = ContentPatch::new();
        patch.round = choices.round.map(Rounding::Within);
        preset.content = Some(patch);
    }
    let plan = session.plan_optimize(&preset).map_err(|e| e.to_string())?;
    session.apply_optimize(&plan).map_err(|e| e.to_string())?;
    let (file, report) = session
        .save_optimized(&plan, &options, &preset.structure)
        .map_err(|e| e.to_string())?;
    let pictures = Tally::of(&plan);
    let mut left = BTreeMap::new();
    for skipped in &plan.skipped {
        *left.entry(t::left_because(&skipped.reason)).or_insert(0) += 1;
    }
    let (bytes, not_smaller) = match file {
        OptimizedFile::Smaller(bytes) => (Some(bytes), false),
        OptimizedFile::NotSmaller { bytes, .. } => (Some(bytes), true),
        _ => (None, false),
    };
    let fonts = plan.resubset.as_ref().map(t::resubset).unwrap_or_default();
    let fonts_cut = plan.resubset.as_ref().map_or(0, |r| r.targets.len());
    let tidied: Vec<_> = plan
        .changes
        .iter()
        .filter_map(|c: &PlannedChange| c.content.as_ref())
        .collect();
    let mut cleaned = t::cleaned(&plan.cleanup);
    cleaned.extend(t::discarded(&plan.discard_objects));
    Ok(Recompressed {
        bytes,
        not_smaller,
        changed: plan.changes.len(),
        left,
        notes: plan.notes,
        restructured: report.as_ref().map(t::restructured).unwrap_or_default(),
        cleaned,
        pictures,
        fonts,
        fonts_cut,
        content: t::content(tidied.iter().copied()),
        content_tidied: tidied.len(),
        content_rounded: tidied.iter().map(|c| c.numbers_rounded).sum(),
        unrounded: t::unrounded(tidied.iter().copied()),
    })
}

/// [`recompress`] on its own thread.
pub struct Job {
    rx: Receiver<Result<Recompressed, String>>,
}

impl Job {
    /// Start recompressing `copy`; the window repaints when it is done.
    pub fn start(ctx: &egui::Context, copy: Vec<u8>, settings: Settings, choices: Choices) -> Self {
        let (tx, rx) = channel();
        let ctx = ctx.clone();
        let spawned = std::thread::Builder::new()
            .name("compact-recompress".to_owned())
            .spawn(move || {
                let started = std::time::Instant::now();
                let result = recompress(&copy, &settings, choices);
                crate::diag::trace(|| {
                    format!(
                        "compact-recompressed ok={} smaller={} changed={} resized={} jpeg={} converted={} lossy={} fonts_cut={} content={} rounded={} bytes={} ms={}", // ui-text-exempt: diagnostic trace
                        result.is_ok(),
                        result.as_ref().is_ok_and(|r| r.bytes.is_some()),
                        result.as_ref().map_or(0, |r| r.changed),
                        result.as_ref().map_or(0, |r| r.pictures.resized),
                        result.as_ref().map_or(0, |r| r.pictures.jpeg),
                        result.as_ref().map_or(0, |r| r.pictures.converted),
                        result.as_ref().is_ok_and(|r| r.pictures.lossy),
                        result.as_ref().map_or(0, |r| r.fonts_cut),
                        result.as_ref().map_or(0, |r| r.content_tidied),
                        result.as_ref().map_or(0, |r| r.content_rounded),
                        result
                            .as_ref()
                            .ok()
                            .and_then(|r| r.bytes.as_ref())
                            .map_or(0, Vec::len),
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
    use pdfcer_core::optimize::DiscardObject;

    const ALL: Choices = Choices {
        recompress: true,
        broken: true,
        destinations: true,
        discard: DiscardOptions::none(),
        pictures: Pictures::OFF,
        fonts: false,
        content: false,
        round: None,
    };

    /// A one-page file whose content stream is stored unfiltered, with one
    /// object nothing refers to.
    fn unfiltered() -> Vec<u8> {
        let page = "0 g 10 10 20 20 re f\n".repeat(200);
        file(&[
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 40] /Contents 4 0 R >>".to_string(),
            format!("<< /Length {} >>\nstream\n{page}\nendstream", page.len()),
            "<< /Unused true >>".to_string(),
        ])
    }

    /// A one-page file with a link and a bookmark to a page that is not in
    /// it.
    fn broken() -> Vec<u8> {
        file(&[
            "<< /Type /Catalog /Pages 2 0 R /Outlines 6 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 40] /Contents 4 0 R \
             /Annots [5 0 R] >>"
                .to_string(),
            "<< /Length 0 >>\nstream\n\nendstream".to_string(),
            "<< /Type /Annot /Subtype /Link /Rect [0 0 10 10] /Dest [9 0 R /Fit] >>".to_string(),
            "<< /Type /Outlines /First 7 0 R /Last 7 0 R /Count 1 >>".to_string(),
            "<< /Title (Gone) /Parent 6 0 R /Dest [9 0 R /Fit] >>".to_string(),
        ])
    }

    /// The links and bookmarks that go nowhere are removed and named when
    /// asked for, and left when not.
    #[test]
    fn links_and_bookmarks_that_go_nowhere_are_removed_when_asked() {
        let copy = broken();
        let done = recompress(&copy, &Settings::default(), ALL).expect("optimizes");
        assert_eq!(
            done.cleaned,
            [
                "1 link(s) that went nowhere removed.",
                "1 bookmark(s) that went nowhere removed: Gone."
            ],
            "{done:?}"
        );
        let kept = Choices {
            broken: false,
            ..ALL
        };
        let done = recompress(&copy, &Settings::default(), kept).expect("optimizes");
        assert!(done.cleaned.is_empty(), "{done:?}");
    }

    /// A working bookmark goes only when every bookmark is asked to, and the
    /// copy saved then has no bookmark list, even if it is no smaller.
    #[test]
    fn all_bookmarks_are_removed_only_when_asked() {
        let copy = file(&[
            "<< /Type /Catalog /Pages 2 0 R /Outlines 5 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 40] /Contents 4 0 R >>".to_string(),
            "<< /Length 0 >>\nstream\n\nendstream".to_string(),
            "<< /Type /Outlines /First 6 0 R /Last 6 0 R /Count 1 >>".to_string(),
            "<< /Title (Here) /Parent 5 0 R /Dest [3 0 R /Fit] >>".to_string(),
        ]);
        let none = Choices {
            recompress: false,
            ..ALL
        };
        let kept = recompress(&copy, &Settings::default(), none).expect("optimizes");
        assert!(kept.cleaned.is_empty(), "{kept:?}");
        let asked = Choices {
            discard: DiscardOptions::none().with_object(DiscardObject::Bookmarks),
            ..none
        };
        let done = recompress(&copy, &Settings::default(), asked).expect("optimizes");
        assert!(
            done.cleaned
                .iter()
                .any(|l| l.starts_with("Bookmarks removed")),
            "{done:?}"
        );
        let bytes = done.bytes.expect("the removal is written");
        let again = recompress(&bytes, &Settings::default(), asked).expect("optimizes");
        assert!(
            again.cleaned.is_empty(),
            "the bookmarks are gone: {again:?}"
        );
    }

    /// Asking for links to other files removes the one to `other.pdf` and
    /// leaves the web link, and asking for form actions alone touches neither.
    #[test]
    fn a_link_to_another_file_goes_and_a_web_link_stays() {
        let copy = file(&[
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 40] /Contents 4 0 R \
             /Annots [5 0 R 6 0 R] >>"
                .to_string(),
            "<< /Length 0 >>\nstream\n\nendstream".to_string(),
            "<< /Type /Annot /Subtype /Link /Rect [0 0 10 10] \
             /A << /S /GoToR /F (other.pdf) /D [0 /Fit] >> >>"
                .to_string(),
            "<< /Type /Annot /Subtype /Link /Rect [20 0 30 10] \
             /A << /S /URI /URI (https://example.com) >> >>"
                .to_string(),
        ]);
        let base = Choices {
            recompress: false,
            ..ALL
        };
        let forms = Choices {
            discard: DiscardOptions::none().with_object(DiscardObject::FormActions),
            ..base
        };
        let untouched = recompress(&copy, &Settings::default(), forms).expect("optimizes");
        assert!(untouched.cleaned.is_empty(), "{untouched:?}");
        let asked = Choices {
            discard: DiscardOptions::none().with_object(DiscardObject::ExternalReferences),
            ..base
        };
        let done = recompress(&copy, &Settings::default(), asked).expect("optimizes");
        assert!(
            done.cleaned
                .iter()
                .any(|l| l.starts_with("Links to other files in 1 place(s) removed")),
            "{done:?}"
        );
        let bytes = done.bytes.expect("the removal is written");
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("example.com"), "the web link stays");
        assert!(!text.contains("other.pdf"), "the file link goes");
    }

    /// Asking for object data removes a drawn object's part number and keeps
    /// the tags that held it.
    #[test]
    fn object_data_goes_and_the_tags_stay() {
        let copy = file(&[
            "<< /Type /Catalog /Pages 2 0 R /StructTreeRoot 5 0 R /MarkInfo << /Marked true /UserProperties true >> >>"
                .to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 40] /Contents 4 0 R >>".to_string(),
            "<< /Length 0 >>\nstream\n\nendstream".to_string(),
            "<< /Type /StructTreeRoot /K 6 0 R >>".to_string(),
            "<< /Type /StructElem /S /Figure /P 5 0 R /A << /O /UserProperties /P [<< /N (Part) /V (A-100) >>] >> >>"
                .to_string(),
        ]);
        let asked = Choices {
            recompress: false,
            discard: DiscardOptions::none().with_object(DiscardObject::ObjectData),
            ..ALL
        };
        let done = recompress(&copy, &Settings::default(), asked).expect("optimizes");
        assert!(
            done.cleaned
                .iter()
                .any(|l| l.starts_with("Object data in ")),
            "{done:?}"
        );
        let text =
            String::from_utf8_lossy(&done.bytes.expect("the removal is written")).into_owned();
        assert!(!text.contains("A-100"), "the part number goes");
        assert!(text.contains("/StructTreeRoot"), "the tags stay");
    }

    /// Asking for form fields removes the field, its widget and the form.
    #[test]
    fn form_fields_go_with_what_they_showed() {
        let copy = file(&[
            "<< /Type /Catalog /Pages 2 0 R /AcroForm << /Fields [5 0 R] >> >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 40] /Contents 4 0 R /Annots [5 0 R] >>"
                .to_string(),
            "<< /Length 0 >>\nstream\n\nendstream".to_string(),
            "<< /Type /Annot /Subtype /Widget /FT /Tx /T (Name) /V (Filled-In) /Rect [0 0 50 10] /P 3 0 R >>"
                .to_string(),
        ]);
        let asked = Choices {
            recompress: false,
            discard: DiscardOptions::none().with_object(DiscardObject::FormFields),
            ..ALL
        };
        let done = recompress(&copy, &Settings::default(), asked).expect("optimizes");
        assert!(
            done.cleaned
                .iter()
                .any(|l| l.starts_with("Form fields removed")),
            "{done:?}"
        );
        let text =
            String::from_utf8_lossy(&done.bytes.expect("the removal is written")).into_owned();
        assert!(!text.contains("Filled-In"), "the field goes");
    }

    /// Merging visible layers names the shown layer and leaves the hidden one,
    /// with recompression on as the window opens: a recompression planned for
    /// the same content must not bring the merged layer back.
    #[test]
    fn a_visible_layer_is_merged_and_a_hidden_one_stays() {
        let content = "/OC /L1 BDC 0 0 m 10 10 l S EMC /OC /L2 BDC 1 1 m 9 9 l S EMC\n".repeat(50);
        let copy = file(&[
            "<< /Type /Catalog /Pages 2 0 R /OCProperties << /OCGs [5 0 R 6 0 R] /D << /OFF [6 0 R] >> >> >>"
                .to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 40] /Contents 4 0 R /Resources << /Properties << /L1 5 0 R /L2 6 0 R >> >> >>"
                .to_string(),
            format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()),
            "<< /Type /OCG /Name (Notes) >>".to_string(),
            "<< /Type /OCG /Name (Draft) >>".to_string(),
        ]);
        let asked = Choices {
            discard: DiscardOptions::none().with_object(DiscardObject::FlattenLayers),
            ..ALL
        };
        let done = recompress(&copy, &Settings::default(), asked).expect("optimizes");
        assert!(
            done.cleaned
                .iter()
                .any(|l| l == "1 layer(s) merged into the page: Notes."),
            "{done:?}"
        );
        let bytes = done.bytes.expect("the merge is written");
        let saved = Document::from_bytes(bytes).expect("parses");
        let pages = pdfcer_core::page_tree::pages(&saved).expect("a page tree");
        let session = pdfcer_core::edit::EditSession::new(saved);
        let drawn = pdfcer_core::content::ContentStream::from_page(&session.view(), &pages[0])
            .expect("content decodes");
        let drawn = String::from_utf8_lossy(&drawn.buf);
        assert!(!drawn.contains("/L1"), "the merged layer came back");
        assert_eq!(drawn.matches("/L2").count(), 50, "the hidden layer stays");
        assert_eq!(drawn.matches("10 10 l").count(), 50, "what it drew stays");
    }

    /// Twenty grey strips stacked edge to edge become one image drawn once,
    /// and the report names the page, the strips and the size.
    #[test]
    fn picture_strips_become_one_image() {
        let strips = 20;
        let names: String = (0..strips)
            .map(|i| format!("/S{i} {} 0 R ", i + 5))
            .collect();
        let content: String = (0..strips)
            .map(|i| format!("q 40 0 0 10 0 {} cm /S{i} Do Q\n", 190 - 10 * i))
            .collect();
        let mut bodies = vec![
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Contents 4 0 R /Resources << /XObject << {names}>> >> >>"
            ),
            format!(
                "<< /Length {} >>\nstream\n{content}endstream",
                content.len()
            ),
        ];
        for i in 0..strips {
            let row = char::from(b'A' + i as u8).to_string().repeat(32);
            bodies.push(format!("<< /Type /XObject /Subtype /Image /Width 16 /Height 2 /ColorSpace /DeviceGray /BitsPerComponent 8 /Length 32 >>\nstream\n{row}\nendstream"));
        }
        let asked = Choices {
            discard: DiscardOptions::none().with_object(DiscardObject::MergeImageFragments),
            ..ALL
        };
        let done = recompress(&file(&bodies), &Settings::default(), asked).expect("optimizes");
        assert!(
            done.cleaned.iter().any(|l| l.starts_with(
                "Picture strips merged: page 1, 20 strips into 1 image (16x40). About "
            )),
            "{done:?}"
        );
        let saved =
            Document::from_bytes(done.bytes.expect("the merge is written")).expect("parses");
        let pages = pdfcer_core::page_tree::pages(&saved).expect("a page tree");
        let session = pdfcer_core::edit::EditSession::new(saved);
        let drawn = pdfcer_core::content::ContentStream::from_page(&session.view(), &pages[0])
            .expect("content decodes");
        let drawn = String::from_utf8_lossy(&drawn.buf);
        assert_eq!(drawn.matches(" Do").count(), 1, "one image drawn: {drawn}");
    }

    /// A quarter circle drawn as 90 short lines, as CAD writes an arc,
    /// comes back as a few curves, and the report names the page's move.
    #[test]
    fn short_lines_become_curves() {
        let mut path = String::new();
        for i in 0..=90 {
            let a = f64::from(i).to_radians();
            let op = if i == 0 { "m" } else { "l" };
            path.push_str(&format!(
                "{:.3} {:.3} {op}\n",
                50.0 + 100.0 * a.cos(),
                50.0 + 100.0 * a.sin()
            ));
        }
        path.push_str("S\n");
        let bodies = vec![
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 300] /Contents 4 0 R /Resources << >> >>".to_string(),
            format!("<< /Length {} >>\nstream\n{path}endstream", path.len()),
        ];
        let asked = Choices {
            discard: DiscardOptions::none().with_object(DiscardObject::SmoothLines),
            ..ALL
        };
        let done = recompress(&file(&bodies), &Settings::default(), asked).expect("optimizes");
        assert!(
            done.cleaned.iter().any(|l| l
                .starts_with("Lines smoothed into curves: page 1, 90 line segments into ")
                && l.contains("(max deviation ")),
            "{done:?}"
        );
        let saved =
            Document::from_bytes(done.bytes.expect("the curves are written")).expect("parses");
        let pages = pdfcer_core::page_tree::pages(&saved).expect("a page tree");
        let session = pdfcer_core::edit::EditSession::new(saved);
        let drawn = pdfcer_core::content::ContentStream::from_page(&session.view(), &pages[0])
            .expect("content decodes");
        let drawn = String::from_utf8_lossy(&drawn.buf);
        assert!(drawn.matches(" c").count() > 0, "curves drawn: {drawn}");
        assert!(drawn.matches(" l").count() < 10, "the lines went: {drawn}");
    }

    /// A 300-pixel noise picture drawn 100 pt wide (216 ppi), asked down to
    /// 72 ppi as JPEG, is scaled, stored as JPEG and disclosed as lossy, and
    /// the copy is much smaller; with pictures off none of it happens.
    #[test]
    fn pictures_change_only_when_asked() {
        let side = 300_usize;
        let mut seed = 12_345_u32;
        let pixels: String = (0..side * side)
            .map(|_| {
                seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                char::from(b'0' + ((seed >> 16) % 64) as u8)
            })
            .collect();
        let content = "q 100 0 0 100 0 0 cm /Im1 Do Q\n";
        let copy = file(&[
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Contents 4 0 R \
             /Resources << /XObject << /Im1 5 0 R >> >> >>"
                .to_string(),
            format!(
                "<< /Length {} >>\nstream\n{content}endstream",
                content.len()
            ),
            format!(
                "<< /Type /XObject /Subtype /Image /Width {side} /Height {side} \
                 /ColorSpace /DeviceGray /BitsPerComponent 8 /Length {} >>\nstream\n{pixels}\nendstream",
                pixels.len()
            ),
        ]);
        let off = recompress(&copy, &Settings::default(), ALL).expect("optimizes");
        assert_eq!(off.pictures, Tally::default(), "{:?}", off.pictures);
        let asked = Choices {
            pictures: Pictures {
                on: true,
                ppi: Some(72),
                format: crate::dialogs::compact_images::Format::Jpeg,
                ..Pictures::OFF
            },
            ..ALL
        };
        let done = recompress(&copy, &Settings::default(), asked).expect("optimizes");
        let tally = done.pictures;
        assert!(
            tally.resized == 1 && tally.jpeg == 1 && tally.lossy,
            "{tally:?} left {:?} notes {:?}",
            done.left,
            done.notes
        );
        let (small, plain) = (
            done.bytes.expect("smaller").len(),
            off.bytes.map_or(copy.len(), |b| b.len()),
        );
        assert!(small * 2 < plain, "{small} B against {plain} B");
    }

    /// One face embedded twice: the switch off leaves both copies; on, one
    /// program is cut, the other copy is merged into it, and the window's
    /// sentences say both, with the bytes saved.
    #[test]
    fn fonts_are_cut_and_merged_only_when_asked() {
        let copy = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/resubset-pair.pdf"),
        )
        .expect("fixture");
        let off = recompress(&copy, &Settings::default(), ALL).expect("optimizes");
        assert!(off.fonts.is_empty() && off.fonts_cut == 0, "{off:?}");
        let on = Choices { fonts: true, ..ALL };
        let done = recompress(&copy, &Settings::default(), on).expect("optimizes");
        assert_eq!(done.fonts_cut, 1, "{done:?}");
        assert!(
            done.fonts[0].starts_with("1 embedded font(s) cut down to the characters used, saving"),
            "{done:?}"
        );
        assert!(
            done.fonts
                .iter()
                .any(|l| l.contains("3 of 8 characters kept, with 1 other copy(ies) merged")),
            "{done:?}"
        );
        let cut = done.bytes.as_ref().expect("smaller");
        assert!(
            cut.len() < off.bytes.map_or(copy.len(), |b| b.len()),
            "{done:?}"
        );
    }

    /// Padded drawing instructions are tidied only when asked: the sentence
    /// counts the 199 setters repeating the grey already set (one in the
    /// first line, both in each later one), and the
    /// copy's page no longer spells `0.500`.
    #[test]
    fn drawing_instructions_are_tidied_only_when_asked() {
        // string-gap-exempt: the gap is the redundant spacing being tidied.
        let page = "0.500 g   0.500 g
10 10 20 20 re f % a comment
"
        .repeat(100);
        let copy = file(&[
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 40] /Contents 4 0 R >>".to_string(),
            format!(
                "<< /Length {} >>
stream
{page}
endstream",
                page.len()
            ),
        ]);
        let off = recompress(&copy, &Settings::default(), ALL).expect("optimizes");
        assert!(off.content.is_none() && off.content_tidied == 0, "{off:?}");
        let on = Choices {
            content: true,
            ..ALL
        };
        let done = recompress(&copy, &Settings::default(), on).expect("optimizes");
        assert_eq!(done.content_tidied, 1, "{done:?}");
        let said = done.content.as_deref().expect("a sentence");
        assert!(
            said.starts_with("Drawing instructions in 1 part(s) tidied")
                && said.contains(": 199 instruction(s) that changed nothing removed, ")
                && !said.contains("  "),
            "{said}"
        );
        let after = Document::from_bytes(done.bytes.expect("written")).expect("parses back");
        let view = after.view();
        let decoded = after
            .objects()
            .filter_map(|o| match &o.value {
                pdfcer_core::object::Object::Stream(s) => {
                    pdfcer_core::filters::decode_stream(&s.dict, view.slice(s.data_span)?).ok()
                }
                _ => None,
            })
            // The writer may pack objects into an object stream; the page's is
            // the one that draws a rectangle.
            .find(|d| d.windows(3).any(|w| w == b" re"))
            .expect("the page content");
        let text = String::from_utf8_lossy(&decoded);
        assert!(!text.contains("0.500") && text.contains(".5"), "{text}");
    }

    /// Line positions round only when a distance is chosen, and the
    /// sentence says how many moved and how far at most.
    #[test]
    fn line_positions_round_only_when_asked() {
        let page = "10.00371 10.00412 m 20.00618 20.00133 l S
"
        .repeat(50);
        let copy = file(&[
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 40] /Contents 4 0 R >>".to_string(),
            format!(
                "<< /Length {} >>
stream
{page}
endstream",
                page.len()
            ),
        ]);
        let tidy = Choices {
            content: true,
            ..ALL
        };
        let plain = recompress(&copy, &Settings::default(), tidy).expect("optimizes");
        assert_eq!(plain.content_rounded, 0, "{plain:?}");
        let rounded = Choices {
            round: Tolerance::from_units(0.01),
            ..tidy
        };
        let done = recompress(&copy, &Settings::default(), rounded).expect("optimizes");
        assert!(done.content_rounded > 0, "{done:?}");
        let said = done.content.as_deref().expect("a sentence");
        assert!(
            said.contains(" line position(s) rounded, none moving more than ")
                && !said.contains("  "),
            "{said}"
        );
        assert!(done.unrounded.is_empty(), "{done:?}");
    }

    fn file(bodies: &[String]) -> Vec<u8> {
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
        let done = recompress(&copy, &Settings::default(), ALL).expect("recompresses");
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
