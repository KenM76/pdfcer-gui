//! # `app::fontlook` — the face and size the Font controls show for the
//! letters a press would act on
//!
//! A word processor's font and size boxes describe the selection, or the
//! letter before the caret, and go blank when the selection mixes values.
//! The shared [`crate::panels::properties::text::TextStyleDraft`] reads one
//! whole run, so this module answers for the draft's caret or selection, or a
//! live sweep, from the page's cached provenance: each glyph carries its `Tf`
//! resource and size, so the answer costs no extraction.

use egui_shell::commands::ConditionSet;
use pdfcer_core::text_edit::TextPosition;

use crate::app::state::OpenDoc;
use crate::canvas::textedit::{self, Anchor};

const CACHE_KEY: &str = "pdfcer-fontlook-cache"; // ui-text-exempt: internal memory id, never displayed

/// The letters a Font press acts on: one position, or `[at, to)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Subject {
    /// The 0-based page.
    pub page: usize,
    /// The first letter, or the one the caret reads.
    pub at: TextPosition,
    /// The end of a selection, exclusive; `None` for a caret.
    pub to: Option<TextPosition>,
    /// The draft's caret, when a draft answered.
    pub caret: Option<usize>,
}

/// One control's value: every letter agrees, or they differ.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Shown<T> {
    /// Every letter has this value.
    One(T),
    /// The letters differ; the control shows blank.
    Mixed,
}

/// What the face box and the size field show; `None` in a field means no
/// letter carried it.
#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) struct Look {
    /// The `/BaseFont`, subset tag and all.
    pub face: Option<Shown<String>>,
    /// The `Tf` size in points.
    pub size: Option<Shown<f64>>,
}

/// The draft's caret or selection, else a live sweep. A draft also publishes
/// the text-subject conditions into `set`.
pub(crate) fn subject(
    ctx: &egui::Context,
    doc: &OpenDoc,
    set: Option<&mut ConditionSet>,
) -> Option<Subject> {
    let Some(d) = textedit::read(ctx) else {
        let s = doc
            .text_selection
            .as_ref()
            .filter(|s| s.live(doc.edit_epoch) && !s.is_empty())?;
        let (from, to) = pdfcer_gui_base::textselection::ordered(s.anchor(), s.focus());
        return Some(Subject {
            page: s.page,
            at: from,
            to: Some(to),
            caret: None,
        });
    };
    let chosen = textedit::caret::range(d.mark, d.caret);
    let first = chosen.map_or(d.caret.saturating_sub(1), |(a, _)| a);
    let byte_of = |n: usize| {
        d.text
            .char_indices()
            .nth(n)
            .map_or(d.text.len(), |(b, _)| b)
    };
    let byte = byte_of(first);
    let mut to = None;
    let at = match &d.anchor {
        Anchor::Run { run, original } => {
            // An unedited draft is the run's text, so a selection in it is a
            // byte range in the run.
            if let Some((_, end)) = chosen.filter(|_| d.text == *original) {
                to = Some(TextPosition::new(*run, byte_of(end)));
            }
            TextPosition::new(*run, byte)
        }
        // An edited paragraph reads the look it was opened in.
        Anchor::Block {
            places, original, ..
        } => {
            let byte = if d.text == *original { byte } else { 0 };
            textedit::promote::position(places, byte, false)?
        }
        _ => return None,
    };
    if let Some(set) = set {
        // ui-text-exempt: condition ids, never displayed.
        set.set("selection.text_runs");
        set.set("selection.formattable");
    }
    Some(Subject {
        page: d.page,
        at,
        to,
        caret: Some(d.caret),
    })
}

/// [`look`] for the current subject, read once per subject and edit.
pub(crate) fn current(ctx: &egui::Context, doc: &OpenDoc) -> Option<Look> {
    let subject = subject(ctx, doc, None)?;
    let id = egui::Id::new(CACHE_KEY);
    let key = (subject, doc.edit_epoch);
    if let Some((_, hit)) = ctx
        .data(|d| d.get_temp::<((Subject, u64), Option<Look>)>(id))
        .filter(|(k, _)| *k == key)
    {
        return hit;
    }
    let found = look(doc, subject);
    ctx.data_mut(|d| d.insert_temp(id, (key, found.clone())));
    found
}

/// The face and size of the subject's letters; `None` when no letter in it
/// has provenance.
pub(crate) fn look(doc: &OpenDoc, subject: Subject) -> Option<Look> {
    let text = doc.provenance_page_text(subject.page)?;
    let model = pdfcer_core::text_edit::EditableTextModel::recognize(
        &text,
        &pdfcer_core::text_edit::BlockRecognitionOptions::default(),
    );
    let (from, to) = (subject.at, subject.to);
    let mut glyphs = Vec::new();
    for run in from.run..=to.map_or(from.run, |t| t.run) {
        let Some(list) = text.runs.get(run).map(|r| &r.glyphs) else {
            continue;
        };
        let wanted: Vec<usize> = match to {
            Some(to) => (0..list.len())
                .filter(|&i| {
                    let start = list[i].text_start as usize;
                    !((run == from.run && start < from.byte_offset)
                        || (run == to.run && start >= to.byte_offset))
                })
                .collect(),
            None => list
                .iter()
                .position(|g| g.text_start as usize >= from.byte_offset)
                .or_else(|| list.len().checked_sub(1))
                .into_iter()
                .collect(),
        };
        glyphs.extend(wanted.into_iter().map(|i| (run, i)));
    }
    let provenance: Vec<_> = glyphs
        .into_iter()
        .filter_map(|(run, i)| model.provenance(pdfcer_core::text_edit::GlyphRef::new(run, i)))
        .collect();
    if provenance.is_empty() {
        return None;
    }
    let inventory = doc.font_inventory();
    let face_of = |key: &[u8]| {
        let key = String::from_utf8_lossy(key);
        inventory
            .fonts
            .iter()
            .find(|r| r.resource_names.iter().any(|n| *n == key))
            .and_then(|r| r.base_font.clone())
    };
    Some(Look {
        face: common(
            provenance
                .iter()
                .filter_map(|p| p.font_resource.as_deref().and_then(face_of)),
        ),
        size: common(provenance.iter().map(|p| f64::from(p.tf_size))),
    })
}

/// What every item agrees on; `None` for no items.
pub(crate) fn common<T: PartialEq>(mut items: impl Iterator<Item = T>) -> Option<Shown<T>> {
    let first = items.next()?;
    Some(if items.all(|x| x == first) {
        Shown::One(first)
    } else {
        Shown::Mixed
    })
}

#[cfg(test)]
mod tests;
