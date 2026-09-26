//! # `canvas::annotclip` — **the annotation half of the canvas clipboard**
//!
//! The seam against [`crate::canvas::clipboard`] is **annotation versus
//! content**, not copy versus paste:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/annotclip.md`.

use pdfcer_core::object::ObjId;
use pdfcer_core::vector::{ClipAnnotation, ObjectClip};

use super::clipboard::Refusal;
use crate::app::actions::Action;
use crate::app::state::OpenDoc;

/// **One selected annotation, resolved into the address space
/// `copy_selection` takes.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selected {
    /// The 0-based page the annotation is on.
    pub page: usize,
    /// Its position in the page's `/Annots`, in document order.
    pub index: usize,
    /// Its object id, for the cut's delete.
    pub id: ObjId,
}

/// **The annotations the current selection names**, resolved onto the page
/// they live on.
pub fn selected(doc: &OpenDoc) -> Result<Vec<Selected>, Refusal> {
    let Some(selection) = doc.selection.annot() else {
        return Ok(Vec::new());
    };
    let page = selection.target.page;
    let Some(page_ref) = doc.pages.get(page) else {
        return Err(Refusal::Unreadable);
    };
    let all = pdfcer_core::annot::page_annotations(&doc.session.graph(), page_ref.id);
    let Some(index) = all.iter().position(|a| a.id == Some(selection.target.id)) else {
        return Err(Refusal::Unreadable);
    };
    Ok(vec![Selected {
        page,
        index,
        id: selection.target.id,
    }])
}

/// **What the engine decided to do with each annotation on a clip.**
///
/// Derived by reading the clip back, never by classifying subtypes here. See
/// the module header for why that distinction is the whole point.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Plan {
    /// Annotations the engine carried **whole** — a raw dictionary with its
    /// baked appearance, or a ce dimension with its group.
    pub whole: usize,
    /// **Annotations the engine carried with a known loss. Always zero against
    /// the current engine, and kept wired rather than deleted.**
    ///
    /// It counts annotations carried as a bare `MarkupSpec`, which describes
    /// the SHAPE alone and so cannot express `/CA`, `/T`, `/M` or `/Contents`.
    /// No current carrier is in that state: `ClipAnnotation::Markup` carries a
    /// `MarkupCarry` beside the spec holding the border dash, `/CA`,
    /// `/Contents` and `/T`, so [`Plan::of`] has no arm that sets this.
    /// `FEATURES.md`: *"A pasted markup is the mark that was copied."*
    ///
    /// ⚠ `/M` is not carried and is not a loss: a paste **authors a fresh
    /// mark**, so a new modification date is the correct answer rather than a
    /// dropped one. This shell stamps `/M` itself (`app::clock::pdf_date_utc`)
    /// everywhere it authors.
    ///
    /// # Why the field survives its own subject
    ///
    /// `ClipAnnotation` is `#[non_exhaustive]`, so a future carrier that *does*
    /// lose something is a live possibility, and on the day it arrives the
    /// count, the disclosure and the status wording must already exist and
    /// already agree. Deleting the field would delete the route as well as the
    /// count, and the route is the expensive half.
    ///
    /// ⇒ If a lossy markup carrier reappears, this is where it is wired back in
    /// — deliberately, with the sentence re-checked against what is actually
    /// lost rather than against what this comment says was lost once.
    pub thin: usize,
    /// The `/Subtype`s the engine refuses to put on a clipboard at all,
    /// verbatim from `ClipAnnotation::Unsupported`.
    ///
    /// Carried as owned `String`s taken off the clip rather than mapped onto
    /// a `&'static str` table here. `canvas::cutgate::Blocker` does keep such a
    /// table, and it has a test asserting the complement, which is what makes
    /// it survivable there — but that table is about *greying a control before
    /// the gesture*, where nothing but a compile-time string will do. Here the
    /// engine has already told us the answer in the payload, and re-deriving it
    /// from a list would be the third copy of a fact the engine owns.
    pub refused: Vec<String>,
}

impl Plan {
    /// Read a clip back and say what it will and will not carry.
    #[must_use]
    pub fn of(clip: &ObjectClip) -> Self {
        let mut plan = Self::default();
        for annotation in &clip.annotations {
            match annotation {
                ClipAnnotation::Unsupported { subtype } => plan.refused.push(subtype.clone()),
                _ => plan.whole += 1,
            }
        }
        plan
    }

    /// How many annotations the clip will actually place.
    #[must_use]
    pub const fn carried(&self) -> usize {
        self.whole + self.thin
    }

    /// **Whether this copy has nothing to offer**, so it must refuse by name
    /// rather than park an empty clip.
    #[must_use]
    pub fn nothing_to_carry(&self, content: usize) -> bool {
        content == 0 && self.carried() == 0
    }
}

/// **An annotation's `/Rect` centre**, in PDF user space — the point a paste
/// places under the cursor.
pub fn rect_centre_of(dict: &pdfcer_core::object::Dict) -> Option<(f64, f64)> {
    use pdfcer_core::object::Object;
    let Object::Array(values) = dict.get(b"Rect")? else {
        return None;
    };
    if values.len() != 4 {
        return None;
    }
    let n = |i: usize| match values.get(i)? {
        Object::Integer(v) => Some(*v as f64),
        Object::Real(v) => Some(*v),
        _ => None,
    };
    // Normalised (§7.9.5): a `/Rect` is not required to be written with its
    // lower-left first, and averaging the pair gives the same centre either
    // way — so no `min`/`max` pass is needed to get this right.
    Some(((n(0)? + n(2)?) / 2.0, (n(1)? + n(3)?) / 2.0))
}

// ===========================================================================
// DUPLICATE — `edit.duplicate`, Ctrl+D
// ===========================================================================

/// **Place a second copy of the selected annotation on the same page**, offset
/// so it is visible, as one undoable command — **without touching the
/// clipboard**.
pub fn duplicate(doc: &OpenDoc, actions: &mut Vec<Action>) -> Result<(), Refusal> {
    let annots = selected(doc)?;
    let Some(target) = annots.first() else {
        return Err(Refusal::NothingSelected);
    };
    let page = target.page;
    // No content indices. A duplicate's subject is the selected annotation,
    // and `SelectionState` cannot hold both — passing `object_indices_on(page)`
    // here would be asking a question whose answer is always the empty list,
    // and would read as though a mixed duplicate were supported.
    let clip = doc
        .session
        .copy_selection(page, &[], &[target.index])
        .map_err(|_| Refusal::EngineRefused)?;
    let plan = Plan::of(&clip);

    if plan.nothing_to_carry(0) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "annot-duplicate-refused reason=cannot-carry what={:?}",
                plan.refused
            )
        });
        return Err(Refusal::CannotCarry(plan.refused));
    }

    let (dx, dy) = (
        crate::canvas::clipboard::PASTE_OFFSET_PT,
        -crate::canvas::clipboard::PASTE_OFFSET_PT,
    );

    // The whole-carrier route: a raw dictionary with its baked `/AP`, or a ce
    // dimension with its group. `paste_objects` takes a page-space MATRIX
    // rather than a displacement, which is why the offset cannot simply be
    // shared with the branch above even though the rule that decides it is.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "annot-duplicate id={} page={page} carrier=clip whole={} thin={} dx={dx:.1} dy={dy:.1}",
            target.id.num, plan.whole, plan.thin
        )
    });
    actions.push(
        crate::app::actions::VectorAction::PasteObjects {
            page,
            clip: clip.to_bytes(),
            at: pdfcer_core::vector::Matrix::translate(dx, dy),
        }
        .into(),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fixture every assertion in this module is aimed at.
    const FIXTURE: &str = "annots-with-everything.pdf";

    /// Open the fixture and select the annotation at `index` in `/Annots`.
    fn with_annot_selected(index: usize) -> crate::app::state::OpenDoc {
        use crate::canvas::selection::annot::{AnnotKind, AnnotSelection, AnnotTarget};

        let mut doc = crate::app::state::open_local_fixture(FIXTURE);
        let page = doc.pages.first().expect("the fixture has a page");
        let annots = pdfcer_core::annot::page_annotations(&doc.session.graph(), page.id);
        let annot = annots.get(index).expect("the fixture has this annotation");
        let subtype = String::from_utf8_lossy(&annot.subtype).into_owned();
        let id = annot.id.expect("an indirect annotation");
        doc.selection.select_annot(AnnotSelection {
            target: AnnotTarget {
                page: 0,
                id,
                kind: AnnotKind::Markup,
                subtype,
                locked: false,
            },
            outline: egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(10.0, 10.0)),
            oriented: None,
        });
        doc
    }

    /// **The engine's own carrier choice, asserted rather than assumed.**
    #[test]
    fn the_engine_models_a_square_and_carries_a_sticky_note_whole() {
        let doc = crate::app::state::open_local_fixture(FIXTURE);

        let square = Plan::of(
            &doc.session
                .copy_annotations(0, &[0])
                .expect("the square copies"),
        );
        // A `/Square` is the kind this shell authors most and the kind the
        // model carrier claims, so it is the one whose carrier choice decides
        // whether an operator's revision cloud copies faithfully. It must be
        // counted CARRIED WHOLE; `(1, 0)` means a lossy carrier is back.
        assert_eq!(
            (square.thin, square.whole),
            (0, 1),
            "★ a /Square must be carried without loss. (1, 0) means Plan::of has a markup arm \
             again, or the engine reverted to a bare MarkupSpec — either way the operator is \
             being told a copy loses four keys, and one of those two claims is now false."
        );

        // The count alone is not evidence: `Plan` is a census, and a
        // carrier that dropped these keys would still be counted whole. So the
        // properties are asserted on the PAYLOAD as well.
        let clip = doc
            .session
            .copy_annotations(0, &[0])
            .expect("the square copies");
        let carried = clip.annotations.iter().any(|a| {
            matches!(a, pdfcer_core::vector::ClipAnnotation::Markup(_, carry)
                if carry.opacity.is_some() || carry.contents.is_some() || carry.author.is_some())
        });
        assert!(
            carried,
            "★ the fixture's /Square carries a note and an opacity, and the clip must hold them \
             — without this the count above could read (0, 1) on a carrier that lost them anyway"
        );

        for (index, what) in [(1usize, "/Text sticky note"), (2, "/FreeText box")] {
            let plan = Plan::of(
                &doc.session
                    .copy_annotations(0, &[index])
                    .expect("it copies"),
            );
            assert_eq!(
                (plan.thin, plan.whole),
                (0, 1),
                "★ a {what} is not modelled, so the engine must carry the whole dictionary and \
                 its baked /AP. If this is now (1, 0) the engine learned to model it on a LOSSY \
                 carrier, and Plan::thin's disclosure has a subject again — re-verify what is lost."
            );
        }
    }

    /// **The lossless route is lossless — asserted key by key against the
    /// SOURCE dictionary, not against a list written here.**
    #[test]
    fn a_sticky_note_survives_the_clipboard_key_by_key() {
        use pdfcer_core::graph::ObjectGraph;
        use pdfcer_core::object::Object;

        let mut doc = crate::app::state::open_local_fixture(FIXTURE);
        let page = doc.pages.first().expect("a page").id;

        let before = pdfcer_core::annot::page_annotations(&doc.session.graph(), page);
        let source_id = before[1].id.expect("indirect");
        let Some(Object::Dict(source)) = doc.session.value(source_id).cloned() else {
            panic!("the sticky note is a dictionary");
        };
        assert_eq!(
            source.get(b"CA"),
            Some(&Object::Real(0.4)),
            "★ the fixture must carry a /CA a MarkupSpec cannot express, or this test passes \
             against a build that re-authors from the spec"
        );

        let clip = doc
            .session
            .copy_annotations(0, &[1])
            .expect("the sticky note copies");
        // Through the bytes, not the live struct: the bytes are what the
        // clipboard parks and what a cross-process paste would carry, so a
        // codec that dropped a key would otherwise pass here and fail in the
        // running program.
        let bytes = clip.to_bytes();
        let clip = pdfcer_core::vector::ObjectClip::from_bytes(&bytes).expect("it round-trips");

        let session =
            std::sync::Arc::get_mut(&mut doc.session).expect("the test holds the only handle");
        session
            .paste_objects(0, &clip, pdfcer_core::vector::Matrix::IDENTITY)
            .expect("it pastes");

        let after = pdfcer_core::annot::page_annotations(&session.graph(), page);
        assert_eq!(
            after.len(),
            before.len() + 1,
            "the paste must add exactly one annotation"
        );
        let pasted_id = after
            .last()
            .and_then(|a| a.id)
            .expect("the pasted annotation is indirect");
        let Some(Object::Dict(pasted)) = session.value(pasted_id).cloned() else {
            panic!("the pasted annotation is a dictionary");
        };

        for (key, value) in source.iter() {
            let name = String::from_utf8_lossy(key.as_bytes()).into_owned();
            // `/P` names the source page and the engine strips it by design;
            // `/AP` and `/Rect` are asserted separately below because both are
            // legitimately rewritten.
            if matches!(name.as_str(), "P" | "AP" | "Rect") {
                continue;
            }
            assert_eq!(
                pasted.get(key.as_bytes()),
                Some(value),
                "★ /{name} did not survive the clipboard. A copy implemented as a re-author is \
                 only as faithful as the authoring type, and this key is one no MarkupSpec can \
                 express — its loss is invisible on the page and nobody would report it."
            );
        }
        let graph = session.graph();
        assert!(
            matches!(
                pasted.get(b"AP").map(|o| graph.resolve(o)),
                Some(Object::Dict(_))
            ),
            "★ the baked /AP must arrive: a sticky note without one renders as NOTHING, which \
             errors nowhere and looks like a paste that did not happen"
        );
    }

    /// **An annotation the engine carries WHOLE duplicates through the clip
    /// route**, with its baked appearance.
    #[test]
    fn duplicating_an_unmodelled_annotation_takes_the_whole_carrier() {
        let doc = with_annot_selected(1);
        let mut actions = Vec::new();
        duplicate(&doc, &mut actions).expect("a sticky note duplicates");
        let [action] = actions.as_slice() else {
            panic!("★ exactly one action: {actions:?}");
        };
        assert!(
            matches!(
                action,
                Action::Vector(crate::app::actions::VectorAction::PasteObjects { .. })
            ),
            "★ an annotation the engine carries whole must travel as a CLIP — any route that \
             re-authors it from a spec drops its baked /AP and renders a sticky note as \
             nothing at all: {action:?}"
        );
    }

    /// Nothing selected refuses by name and raises nothing.
    #[test]
    fn duplicating_nothing_refuses_and_raises_nothing() {
        let doc = crate::app::state::open_local_fixture(FIXTURE);
        let mut actions = Vec::new();
        assert_eq!(duplicate(&doc, &mut actions), Err(Refusal::NothingSelected));
        assert!(actions.is_empty(), "a refusal raises nothing: {actions:?}");
    }

    /// **A selection that has outlived its annotation refuses**, through the
    /// same [`selected`] guard the copy uses, rather than duplicating whichever
    /// annotation now sits at that `/Annots` position.
    #[test]
    fn duplicating_a_stale_selection_refuses() {
        use crate::canvas::selection::annot::{AnnotKind, AnnotSelection, AnnotTarget};

        let mut doc = crate::app::state::open_local_fixture(FIXTURE);
        doc.selection.select_annot(AnnotSelection {
            target: AnnotTarget {
                page: 0,
                id: pdfcer_core::object::ObjId::new(999, 0),
                kind: AnnotKind::Markup,
                subtype: "Square".to_owned(),
                locked: false,
            },
            outline: egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(10.0, 10.0)),
            oriented: None,
        });
        let mut actions = Vec::new();
        assert_eq!(duplicate(&doc, &mut actions), Err(Refusal::Unreadable));
        assert!(actions.is_empty());
    }

    /// **The duplicate does not touch the clipboard**, which is the whole
    /// reason the command exists.
    #[test]
    fn the_duplicate_cannot_reach_the_clipboard() {
        let _: fn(&crate::app::state::OpenDoc, &mut Vec<Action>) -> Result<(), Refusal> = duplicate;
    }
}
