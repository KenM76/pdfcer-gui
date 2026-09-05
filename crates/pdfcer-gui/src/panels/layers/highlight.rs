//! `panels::layers::highlight` — which layer the current selection is on.
//!
//! The operator's ask, verbatim: *"selecting an object highlights that
//! layer"*.
//!
//! # ★★★ The finding this whole file is shaped by
//!
//! **For a content object — a path, a text run, an image on the page —
//! `pdfcer-core` cannot answer.** The relation exists in the file: an
//! object's optional-content membership comes from a `BDC /OC /Pn …EMC`
//! marked-content section (§8.11.3.2) or an XObject's own `/OC`
//! (§8.11.3.3). The engine reads both — `pdfcer-render`'s interpreter
//! resolves the OCG's `ObjId` and calls `annot::oc_is_hidden` with it — and
//! then **throws the identity away**, pushing a `bool` onto its
//! marked-content stack. The object model does the same in the other
//! direction: `vector::decompose`'s walk counts `/OC` sections into
//! `DecomposeDiagnostics::oc_sections` and has no `EMC` arm at all, so no
//! `PathObject`, `TextObject`, `ImageObject` or `FormLeaf` carries the group
//! it was painted under.
//!
//! ⇒ **The request is filed**, not invented:
//! `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\request_which_layer_is_this_object_on.md`.
//! `OPERATOR_REQUESTS.md` **O126** names it.
//!
//! ★★ **And the workaround was deliberately not taken.** The shell *could*
//! re-tokenize the page with `ContentStream::from_page`, keep its own
//! `/OC` stack, and index by `VectorObject::tokens()` — about forty lines,
//! all of it public API. It is refused because it would be **a second
//! implementation of `/OC` resolution beside the engine's**, which is the
//! exact "two surfaces drift" failure `pdfcer-core`'s `layers.rs` decision 1
//! exists to prevent; it cannot see OCMD `/VE` visibility-expression policy
//! at all; and `annot::oc_refs` — the OCMD-expansion helper — is
//! `pub(crate)`, so the expansion would have to be forked too. A shell that
//! *nearly* resolves optional content will one day disagree with the
//! renderer about which layer something is on, and the operator's reading
//! of that is "pdfcer got it wrong".
//!
//! # ★★ What IS reachable, and it is not nothing
//!
//! **An annotation carries its own layer.** `pdfcer_core::annot::Annotation`
//! has `oc: Option<ObjId>` — the §8.11.3.3 reference — populated by the
//! same read the Comments panel already uses, and its own doc comment
//! records the asymmetry in as many words: *"(Pass 12.M2 authored-layer
//! `/OC` honouring …; **full content-stream BDC/EMC `/OC` stays
//! deferred**)"*.
//!
//! So this shell can answer the question truthfully for a selected
//! annotation — a stamp, a cloud, a dimension, a note — and cannot for a
//! selected path. That asymmetry is real, it is the engine's, and it is
//! surfaced rather than smoothed.
//!
//! # ★★★ Why the answer is THREE-valued, which is the whole design
//!
//! The obvious type is `Option<ObjId>`, and it is wrong here in a way that
//! matters more than usual.
//!
//! | | `Option` says | the truth |
//! |---|---|---|
//! | a stamp with no `/OC` | `None` | **on no layer** — a fact the engine established |
//! | a selected path | `None` | **not known** — the engine cannot say |
//!
//! Collapsing those two makes the panel unable to distinguish *"this mark
//! is on no layer"* from *"nobody can tell you"*, and the operator reads
//! the second as the first. The operator's own note on this feature is the
//! bar: **highlighting the wrong layer is worse than highlighting none** —
//! and asserting "on no layer" about an object whose layer is simply
//! unknown is the same class of wrong answer, arriving as silence instead
//! of as a highlight.
//!
//! ⇒ [`Membership`] has three variants, and
//! [`Membership::Unknown`] renders **nothing at all** (R9: an unavailable
//! capability renders nothing) while [`Membership::None`] renders a
//! sentence, because it is a positive fact worth stating.
//!
//! # The reverse relation, deliberately not built
//!
//! *Does clicking a layer indicate its objects?* **No, and it cannot be**
//! today. It needs the inverse index — `PageObjects::objects_on_layer(ocg)`
//! or equivalent — which is trivially derivable from the membership this
//! module is waiting for and does not exist without it. It is named in the
//! filed request as a second, lower-priority ask precisely so that it is
//! not built here from a shell-side re-parse. Nothing about the forward
//! direction is shaped around it: when the engine answers, the reverse is a
//! new function and no change to any of this.

use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;

/// Which optional-content group the current selection belongs to.
///
/// Three-valued on purpose — see the module header's table for the two
/// states an `Option` would merge and why merging them produces a false
/// statement rather than a missing one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Membership {
    /// **Nothing is selected**, so there is no question to answer.
    ///
    /// Distinct from [`Self::Unknown`], which means something *is* selected
    /// and the engine cannot say. Both render nothing, and they are still
    /// different: a caller counting "how often can we not answer" must not
    /// count an empty canvas as a failure.
    NothingSelected,
    /// **The selection is on this optional-content group.**
    ///
    /// The `ObjId` is comparable directly against `Layer::id`, against
    /// `OpenDoc::hidden_layers()` and against
    /// `annot::optional_content_default_off` — all four speak the same
    /// vocabulary, which is what makes the highlight a lookup rather than a
    /// translation.
    ///
    /// ★ It may name an **OCMD** rather than an OCG (§8.11.2.2): an
    /// annotation's `/OC` is allowed to be either. An OCMD's id will not
    /// match any row, so the highlight simply finds nothing — which is the
    /// right outcome, because "several layers together, under a visibility
    /// expression" is not one row to emphasise. See
    /// [`resolve`]'s note.
    Group(ObjId),
    /// **The selection is on no layer**, and the engine established that.
    ///
    /// A positive fact, and worth saying out loud: a drawing whose every
    /// mark is on a layer makes an unlayered stamp genuinely surprising, and
    /// an operator who has just switched a layer off and is wondering why
    /// their note is still there deserves to be told why.
    None,
    /// **The engine cannot say**, which is today's answer for every content
    /// object.
    ///
    /// Renders nothing. R9's rule applies exactly: the capability is
    /// unavailable, so no surface claims anything about it — not a greyed
    /// row, not a "layer unknown" line, not a highlight on some default.
    ///
    /// ★ It is a *variant* rather than an absence so that the day the
    /// engine answers, the change is one arm of [`resolve`] and every
    /// caller keeps compiling. It is also what a test can assert about, so
    /// "we stopped being able to answer" and "we never could" are
    /// distinguishable in the suite.
    Unknown,
}

impl Membership {
    /// The row this should emphasise, if any.
    #[must_use]
    pub const fn highlighted(self) -> Option<ObjId> {
        match self {
            Self::Group(id) => Some(id),
            Self::NothingSelected | Self::None | Self::Unknown => None,
        }
    }

    /// Whether the panel owes the operator a sentence about this.
    ///
    /// True only for [`Self::None`]. [`Self::Unknown`] is silent by design
    /// — a sentence saying "we cannot tell" on every path selection would
    /// be a permanent apology in a panel, and R9's answer to an unavailable
    /// capability is to render nothing rather than to explain the gap
    /// forever.
    #[must_use]
    pub const fn owes_a_sentence(self) -> bool {
        matches!(self, Self::None)
    }
}

/// **Which layer is the current selection on?**
///
/// # The order of the arms, which is the whole of the routine
///
/// 1. **An annotation is selected** → ask the engine. `Annotation::oc` is
///    the §8.11.3.3 reference, and `None` there means *on no layer* — the
///    engine has read the annotation and the entry is absent, which is a
///    fact rather than an inability.
/// 2. **Content is selected** → [`Membership::Unknown`]. Not `None`: see
///    the module header.
/// 3. **Nothing is selected** → [`Membership::NothingSelected`].
///
/// `SelectionState` enforces that 1 and 2 are mutually exclusive — its
/// `annot` field exists *"because the two are mutually exclusive and that
/// must be enforced by a type, not remembered"* — so the order between the
/// first two arms cannot decide anything, and it is written annotation-first
/// only because that is the arm that can succeed.
///
/// ★ An annotation whose `/OC` names an **OCMD** returns
/// [`Membership::Group`] carrying the OCMD's id, which will match no row.
/// That is deliberate and it is the honest outcome: an OCMD is a visibility
/// *expression* over several groups, so there is no single row to emphasise,
/// and resolving it to "the first group it mentions" would highlight a layer
/// that does not by itself decide whether the mark is drawn. Nothing
/// highlights, nothing is claimed.
///
/// # Cost
///
/// One `page_annotations` read of the selected annotation's page, once per
/// frame, and only while an annotation is selected. That is the same call
/// the Comments panel makes for every page of the document on every frame,
/// so a single page while one mark is selected is not a new order of cost.
/// It is deliberately **not** cached: `OpenDoc`'s `RefCell`s hold derived
/// caches whose filling nothing can observe, and a cache keyed on a
/// selection plus an edit epoch is a third thing to keep in step for a
/// lookup that costs a vector walk.
#[must_use]
pub fn resolve(doc: &OpenDoc) -> Membership {
    let Some(annot) = doc.selection.annot() else {
        // Content selected, or nothing. `SelectionState::is_empty` is true
        // only when BOTH are empty, which is why it is the right question
        // here rather than `entries().is_empty()`.
        return if doc.selection.is_empty() {
            Membership::NothingSelected
        } else {
            Membership::Unknown
        };
    };
    let view = doc.session.view();
    // `pages_in` and not `EditSession::pages()`: the panel holds a shared
    // `&OpenDoc` and the session's own accessor takes `&mut self`. This is
    // the same call `panels::comments` makes over the same view.
    let Ok(pages) = pdfcer_core::page_tree::pages_in(&view) else {
        // The page tree would not resolve. `Unknown`, emphatically not
        // `None`: a document we cannot read the structure of is the exact
        // case the third variant exists for.
        return Membership::Unknown;
    };
    let Some(page) = pages.get(annot.target.page) else {
        // The selection names a page the current revision does not have —
        // reachable for one frame after a page delete, before the selection
        // is re-resolved. `Unknown` rather than `None`, because "we could
        // not look" is exactly what this type's third variant is for.
        return Membership::Unknown;
    };
    match pdfcer_core::annot::page_annotations(&view, page.id)
        .into_iter()
        .find(|a| a.id == Some(annot.target.id))
    {
        Some(a) => match a.oc {
            Some(oc) => Membership::Group(oc),
            None => Membership::None,
        },
        // The annotation is selected but is not in the page's `/Annots` any
        // more. Same reasoning as the missing page.
        None => Membership::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oc(n: u32) -> ObjId {
        ObjId::new(n, 0)
    }

    /// ★★★ **`Unknown` and `None` are different values**, which is the
    /// whole reason this type exists rather than an `Option<ObjId>`.
    ///
    /// If this ever fails to compile because the two were merged, the panel
    /// has lost the ability to distinguish *"this mark is on no layer"*
    /// from *"nobody can tell you"* — and the operator reads the second as
    /// the first.
    #[test]
    fn not_on_a_layer_is_not_the_same_answer_as_cannot_tell() {
        assert_ne!(Membership::None, Membership::Unknown);
        assert_ne!(Membership::NothingSelected, Membership::Unknown);
        assert_ne!(Membership::None, Membership::NothingSelected);
    }

    /// **Only a known group highlights a row.**
    ///
    /// The operator's bar, made mechanical: highlighting the wrong layer is
    /// worse than highlighting none, so every state that is not a
    /// *positively established* group must highlight nothing.
    #[test]
    fn only_a_known_group_highlights_anything() {
        assert_eq!(Membership::Group(oc(7)).highlighted(), Some(oc(7)));
        assert_eq!(Membership::None.highlighted(), None);
        assert_eq!(Membership::Unknown.highlighted(), None);
        assert_eq!(Membership::NothingSelected.highlighted(), None);
    }

    /// ★★ **Only the established "no layer" says anything.**
    ///
    /// `Unknown` is silent because R9's answer to an unavailable capability
    /// is to render nothing — a line reading "we cannot tell you which
    /// layer this is on" would be a permanent apology in the panel, shown
    /// on every path selection for as long as the engine gap lasts.
    #[test]
    fn only_an_established_absence_owes_a_sentence() {
        assert!(Membership::None.owes_a_sentence());
        assert!(!Membership::Unknown.owes_a_sentence());
        assert!(!Membership::NothingSelected.owes_a_sentence());
        assert!(!Membership::Group(oc(1)).owes_a_sentence());
    }

    /// **A group id round-trips**, so the panel's row lookup is a
    /// comparison rather than a translation.
    #[test]
    fn the_group_id_is_the_one_the_layers_list_speaks() {
        let id = oc(42);
        assert_eq!(Membership::Group(id).highlighted(), Some(id));
    }
}
