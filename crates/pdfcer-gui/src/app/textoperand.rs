//! # `app::textoperand` — **which runs a Format command acts on**, from either
//! gesture
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/textoperand.md`.

use crate::app::state::OpenDoc;

/// **Which gesture produced the runs**, kept beside them because callers need
/// to tell the two sources apart and cannot re-derive it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Source {
    /// A range swept with the Text tool — [`OpenDoc::text_selection`], live
    /// against the current edit epoch.
    Swept,
    /// The single selected text object, by its index in the page's paint
    /// order.
    Object(usize),
}

/// The page, the runs, and where they came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Operand {
    /// The 0-based page the runs are ordinals into.
    pub page: usize,
    /// Extraction run ordinals, ascending — the operand
    /// [`crate::app::actions::Action::TextStyle`] takes and the engine's
    /// `format_text` acts on.
    pub runs: Vec<usize>,
    /// Which of the two rungs answered.
    pub source: Source,
}

/// **The cheap question: is there an object-shaped operand?**
#[must_use]
pub(crate) fn selected_text_object(doc: &OpenDoc) -> Option<(usize, usize)> {
    if doc.selection.annot().is_some() {
        return None;
    }
    let page = doc.view.page_index;
    let objects = doc.selection.object_indices_on(page);
    let [object] = objects.as_slice() else {
        return None;
    };
    let object = *object;
    is_text(doc, page, object).then_some((page, object))
}

/// Is the object at this index page text?
fn is_text(doc: &OpenDoc, page: usize, object: usize) -> bool {
    use crate::canvas::target::CanvasTargetProvider as _;
    doc.page_objects().is_some_and(|provider| {
        provider
            .page_objects_model(page)
            .and_then(|model| model.objects.get(object))
            .is_some_and(|o| {
                crate::panels::objects::summary::object_kind(o)
                    == crate::panels::objects::summary::ObjectKind::Text
            })
    })
}

/// **The live swept range**, or `None`.
#[must_use]
pub(crate) fn swept(doc: &OpenDoc) -> Option<Operand> {
    let selection = doc.text_selection.as_ref()?;
    let runs = selection.runs(doc.edit_epoch);
    (!runs.is_empty()).then_some(Operand {
        page: selection.page,
        runs,
        source: Source::Swept,
    })
}

/// **The whole answer**, paying the extraction when the operand is an object.
#[must_use]
pub(crate) fn resolve(doc: &OpenDoc) -> Option<Operand> {
    if let Some(swept) = swept(doc) {
        return Some(swept);
    }
    let (page, object) = selected_text_object(doc)?;
    let found = crate::canvas::textedit::pin::object_text(doc, page, object)?;
    Some(Operand {
        page,
        runs: (found.first_run..=found.last_run).collect(),
        source: Source::Object(object),
    })
}

/// **A stamped memory of the object rung**, so a surface that asks every frame
/// pays the extraction once per selection instead of once per frame.
#[derive(Default)]
pub(crate) struct Cache {
    /// `(page, object, edit epoch)` the runs below were resolved at.
    stamp: Option<(usize, usize, u64)>,
    /// The resolved run ordinals, or `None` when the resolution found nothing.
    runs: Option<Vec<usize>>,
}

impl Cache {
    /// [`resolve`], behind the stamp.
    pub(crate) fn resolve(&mut self, doc: &OpenDoc) -> Option<Operand> {
        if let Some(swept) = swept(doc) {
            return Some(swept);
        }
        let (page, object) = selected_text_object(doc)?;
        let stamp = (page, object, doc.edit_epoch);
        if self.stamp != Some(stamp) {
            self.stamp = Some(stamp);
            let started = std::time::Instant::now();
            self.runs = crate::canvas::textedit::pin::object_text(doc, page, object)
                .map(|found| (found.first_run..=found.last_run).collect::<Vec<_>>());
            let ms = started.elapsed().as_millis();
            let runs = self.runs.as_ref().map_or(0, Vec::len);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("text-operand-resolved page={page} object={object} runs={runs} ms={ms}")
            });
        }
        let runs = self.runs.clone()?;
        (!runs.is_empty()).then_some(Operand {
            page,
            runs,
            source: Source::Object(object),
        })
    }
}
