//! What an edit turned out to do, and why one was refused.
//!
//! The other half of [`super`] carries the words a Style control shows *before*
//! the operator acts. These are the sentences owed *after*: the collateral of a
//! delete, the refusal when a node cannot be reshaped, the disclosure that pdfcer
//! redrew a smoothed stroke as straight segments. Every one of them is
//! off-canvas copy -- a status line, a properties field, a report -- because a
//! disclosure drawn onto the page would be a second rendering path for content
//! that is already applied.
//!
//! Re-exported by [`super`], so a caller names `crate::text::markup::<item>` for
//! both halves and the split is invisible to it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/markup/edits.md`.
/// **What went with it** — the collateral of deleting one annotation.
#[must_use]
pub fn deleted_collateral(
    popup_removed: bool,
    parent_popup_cleared: bool,
    replies_orphaned: usize,
    group_members_promoted: usize,
) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    if popup_removed {
        parts.push("its pop-up note went with it, which the PDF specification requires".to_owned());
    }
    if parent_popup_cleared {
        parts.push("the annotation it belonged to no longer refers to it".to_owned());
    }
    if replies_orphaned == 1 {
        parts.push("1 reply is left without the comment it replied to".to_owned());
    } else if replies_orphaned > 1 {
        parts.push(format!(
            "{replies_orphaned} replies are left without the comment they replied to"
        ));
    }
    if group_members_promoted == 1 {
        parts.push("1 grouped annotation is now on its own".to_owned());
    } else if group_members_promoted > 1 {
        parts.push(format!(
            "{group_members_promoted} grouped annotations are now on their own"
        ));
    }
    if parts.is_empty() {
        return None;
    }
    Some(format!("Deleted — {}.", parts.join("; ")))
}

/// Disclosure: the annotation moved and its pop-up note did not.
#[must_use]
pub fn popup_left_behind() -> String {
    "The note attached to this markup stayed where it was. pdfcer does not show those \
     notes, so you will only see it in a reader that does."
        .to_owned()
}

/// Disclosure: the border width did not scale with the shape.
#[must_use]
pub fn stroke_width_unchanged() -> String {
    "The outline's thickness has not changed. pdfcer treats a line weight as a drawing standard \
     rather than something that scales with the shape."
        .to_owned()
}

/// Disclosure: a foreign appearance was scaled unevenly and its stroke is now
/// anisotropic.
#[must_use]
pub fn appearance_distorted() -> String {
    "This markup was drawn by another program, and scaling it unevenly has made its outline \
     thicker on one side than the other. Hold Shift while dragging a corner to scale it evenly."
        .to_owned()
}

/// **A SECOND COPY OF THE COMMENT WAS REMOVED** — the disclosure for
/// `MarkupNoteChange::rich_text_dropped` (`pdfcer-core`, 2026-09-08).
#[must_use]
pub fn rich_text_dropped(keys: &[String]) -> Option<String> {
    if keys.is_empty() {
        return None;
    }
    Some(
        "This comment also had a formatted copy of its old words, which some readers show \
         instead of the plain one. pdfcer cannot write formatted text, so it removed the old \
         copy rather than leave your document saying two different things. The comment now \
         reads the same everywhere; any styling it had is gone."
            .to_owned(),
    )
}

/// **Disclosure: the words this note used to carry, on the case where a save
/// overwrote them.**
#[must_use]
pub fn note_replaced(previous: &str) -> Option<String> {
    let previous = previous.trim();
    if previous.is_empty() {
        return None;
    }
    let chars = previous.chars().count();
    const KEEP: usize = 120;
    if chars > KEEP {
        let head: String = previous.chars().take(KEEP).collect();
        return Some(format!(
            "The note that was there has been replaced. It began “{head}…” and ran to {chars} \
             characters. Ctrl+Z restores it."
        ));
    }
    Some(format!(
        "The note that was there has been replaced: “{previous}”. Ctrl+Z restores it."
    ))
}

/// **Disclosure: a note was removed, and what it said.**
#[must_use]
pub fn note_removed(previous: &str) -> Option<String> {
    let previous = previous.trim();
    if previous.is_empty() {
        return None;
    }
    let chars = previous.chars().count();
    const KEEP: usize = 120;
    let words = if chars > KEEP {
        let head: String = previous.chars().take(KEEP).collect();
        format!("“{head}…”, {chars} characters")
    } else {
        format!("“{previous}”")
    };
    Some(format!(
        "The note has been removed — it said {words}. The markup itself is still on the page, and \
         Ctrl+Z restores the words."
    ))
}

// ---------------------------------------------------------------------------
// BEFORE the click: why Delete is not offered, and what it would take with it
// ---------------------------------------------------------------------------

/// **Why the Delete control is absent for the selected annotation** —
/// `EditSession::annotation_deletion_refusal` answered `Some` (R83).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotDeleteRefusal {
    /// `EditError::DocumentEncrypted` because the password the document was
    /// opened with does not grant the edit (`EncryptedRefusal::PermissionDenied`).
    Password,
    /// `EditError::DocumentEncrypted` because the document uses RC4 and edits
    /// under it are off (`EncryptedRefusal::Rc4NotAllowed`).
    Rc4,
    /// An enforced certification signature whose `/P` is below 3 (§12.8.2.2
    /// Table 254) — `EditError::CertificationForbidsChange`.
    ///
    /// Table 254 makes `/P` **Optional with default 2**, so a certified
    /// document that states no permission at all lands here — absence is
    /// permissive relative to `P = 1` and not relative to `P = 3`. The
    /// permission number is deliberately **not** carried into the wording:
    /// `1` and `2` refuse for the same reason and leave the operator nothing to
    /// do differently, so printing it would be jargon in place of a fact.
    ///
    /// `P = 3` is the row this query exists for and it does **not** land
    /// here: that is the comment-review certification, where a document was
    /// signed precisely so reviewers could annotate it, and the annotation gate
    /// allows it where the general structural gate would not.
    Certified,
    /// Anything else the query can return.
    ///
    /// Named rather than reached by a `_ =>` carrying a guess, for the reason
    /// `crate::text::unshare::UnshareRefusal::Other` is: an unnamed cause has
    /// exactly one honest operator-facing content — *nothing has changed* — and
    /// inventing a diagnosis is worse than admitting there is none.
    Other,
}

impl AnnotDeleteRefusal {
    /// The sentence this refusal earns.
    #[must_use]
    pub const fn line(self) -> &'static str {
        match self {
            Self::Password => {
                "The password this file was opened with does not allow its comments and markup \
                 to be deleted. Reopen it with the owner password to delete them."
            }
            Self::Rc4 => {
                "This file uses the old RC4 encryption, and edits under it are off. Allow edits \
                 under RC4 to delete its comments and markup."
            }
            Self::Certified => {
                "A certification signature on this document does not allow its comments and \
                 markup to be deleted. Deleting one would invalidate that signature, so pdfcer \
                 leaves it in place rather than breaking the signature quietly."
            }
            Self::Other => {
                "pdfcer cannot delete comments or markup from this document. \
                 Nothing has been changed."
            }
        }
    }
}

/// **Why the Delete control is absent for THIS annotation** — §12.5.3 Table 165
/// bit 8, the `Locked` flag.
#[must_use]
pub const fn annot_delete_locked() -> &'static str {
    "The file marks this comment as one that should not be changed, so pdfcer does not \
     offer to delete it. Other comments on this page may still be deleted."
}

/// **What deleting the selected annotation would take with it**, said
/// *before* the click — `EditSession::annotation_deletion_preview`.
#[must_use]
pub fn deletion_would_take(
    popup_removed: bool,
    parent_popup_cleared: bool,
    replies_orphaned: usize,
    group_members_promoted: usize,
) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    if popup_removed {
        parts.push(
            "its pop-up note will go with it, which the PDF specification requires".to_owned(),
        );
    }
    if parent_popup_cleared {
        parts.push("the annotation it belongs to will stop referring to it".to_owned());
    }
    if replies_orphaned == 1 {
        parts.push("1 reply will be left without the comment it replied to".to_owned());
    } else if replies_orphaned > 1 {
        parts.push(format!(
            "{replies_orphaned} replies will be left without the comment they replied to"
        ));
    }
    if group_members_promoted == 1 {
        parts.push("1 grouped annotation will be on its own".to_owned());
    } else if group_members_promoted > 1 {
        parts.push(format!(
            "{group_members_promoted} grouped annotations will be on their own"
        ));
    }
    if parts.is_empty() {
        return None;
    }
    Some(format!("If you delete this — {}.", parts.join("; ")))
}

//
// > *"I also can't edit or delete nodes of a markup shape once it is drawn."*
//

/// **The operator's word for a shape**, which is not always the PDF name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeWord {
    /// `/Ink` — a freehand mark.
    Ink,
    /// `/Square` — a rectangle.
    Rectangle,
    /// `/Circle` — an ellipse.
    Ellipse,
    /// `/Line` — a straight line, including an arrow.
    Line,
    /// `/Highlight`, `/Underline`, `/StrikeOut`, `/Squiggly`.
    TextMarkup,
    /// Anything else — named generally rather than wrongly.
    Other,
}

/// **Why a node edit did not happen**, in the shell's own vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeEditRefusal {
    /// `EditError::ReshapeWouldBreachVertexFloor` — the shape is at its floor.
    ///
    /// The one an operator meets by doing something perfectly reasonable: a
    /// triangle they want to make into a line. The floors are the engine's own
    /// (`/Polygon` keeps three, `/PolyLine` keeps two) and it refuses by name
    /// rather than silently clamping or turning the shape into something else.
    WouldLeaveTooFew,
    /// `EditError::InkStrokeWouldBreachPointFloor` — one **stroke** of a
    /// freehand mark is at its floor of two points (`Pass 278.0`).
    ///
    /// Its own sentence rather than [`Self::WouldLeaveTooFew`]'s, because that
    /// one says *"the shape has as few corners as it can have"* and that is
    /// false of a freehand mark whose other strokes have dozens. The floor is
    /// per stroke — §12.5.6.13 describes each inner array as *"points along
    /// the path"*, and one point is not a path — so the next act is to add a
    /// point **to that stroke**, and the sentence says so.
    StrokeWouldLeaveTooFew,
    /// `EditError::InkPointIndexOutOfRange` or
    /// `EditError::InkStrokeIndexOutOfRange` — the anchor the operator grabbed
    /// names a point the freehand mark no longer holds (`Pass 278.0`).
    ///
    /// The anchors are drawn from the same annotation walk the engine is
    /// asked about, in the same frame, so this is unreachable while both read
    /// live — and it is the first symptom the day anything caches one of them.
    /// The remedy is the same for both index spaces: select the mark again,
    /// which rebuilds the anchors from the file as it is now. Worded as that
    /// act rather than as a diagnosis.
    PointNotFound,
    /// `EditError::InkWouldBeEmpty` — the edit would leave a freehand mark
    /// with nothing drawable (`Pass 278.0`).
    ///
    /// Only a whole-stroke removal or replacement can raise it, and this shell
    /// calls neither verb yet (`canvas::annotnodes::ink`'s header says why).
    /// Worded anyway: an unreachable refusal that becomes reachable silently is
    /// how a grip comes to do nothing. The next act the engine names is
    /// `delete_annotation` — deleting the whole mark — and so does this.
    WouldLeaveNothing,
    /// `EditError::GeometryNotReshapable` — this kind of mark has no nodes to
    /// edit, and the sentence says which kind it is.
    ///
    /// Reached two ways, which is why it carries the word rather than the
    /// gesture: from a **drag** on a `/Line`'s end that asked to add or remove
    /// one, and from `pdfcer_gui::canvas::annotnodes::explain_unreshapable` when
    /// the operator arms the Points tool over a shape that shows no anchors at
    /// all. The second is the one that answers *"where are the nodes?"*.
    ShapeHasNoNodes {
        /// What to call it. See [`ShapeWord`].
        subtype: ShapeWord,
    },
    /// `EditError::AnnotationVertexNotPlaceable` — the coordinate is not a
    /// usable page value.
    ///
    /// A non-finite number, which on this canvas means the page-space
    /// conversion produced something the format cannot hold. Not an operator
    /// mistake, and the sentence does not imply one.
    Unplaceable,
    /// `EditError::AnnotationLocked` — §12.5.3 Table 165 bit 8.
    ///
    /// The **file** says the user interface may not change this mark's position
    /// or size. Should be unreachable from an anchor, because
    /// `annotnodes::geometry` refuses a locked annotation before one is drawn —
    /// worded anyway, because an unreachable refusal that becomes reachable
    /// silently is how a grip comes to do nothing.
    Locked,
    /// Everything else the engine can say no with: an annotation that is no
    /// longer there, a ce dimension arriving at the wrong verb, an index that
    /// names nothing, an encrypted document, an enforced certification, a
    /// subtype pdfcer does not model.
    ///
    /// One sentence for all of them rather than seven, because they divide into
    /// *cannot happen from an anchor this shell drew* and *is a property of the
    /// file that no wording about nodes would help with*, and neither class
    /// gives the operator a next act about nodes. What they do get is the
    /// knowledge that the press was heard.
    Refused,
}

impl NodeEditRefusal {
    /// The sentence, for `Declined::line`.
    #[must_use]
    pub const fn line(self) -> &'static str {
        match self {
            Self::WouldLeaveTooFew => {
                "That shape has as few corners as it can have. Add one before \
                 taking one away, or delete the whole mark."
            }
            Self::StrokeWouldLeaveTooFew => {
                "That stroke is down to its last two points. Add a point to it \
                 before taking one away, or delete the whole mark."
            }
            Self::PointNotFound => {
                "That point is no longer where the mark has one. Click the mark \
                 again to redraw its points, then try once more."
            }
            Self::WouldLeaveNothing => {
                "Taking that away would leave nothing of the mark to draw. Delete \
                 the whole mark instead."
            }
            Self::ShapeHasNoNodes { subtype } => subtype.no_nodes_line(),
            Self::Unplaceable => {
                "That corner cannot go there — the position is off the page's \
                 usable range."
            }
            Self::Locked => {
                "The file marks this as locked, so its shape cannot be changed \
                 here. Unlock it in the program that made it."
            }
            Self::Refused => "The corner could not be changed. The drawing is unchanged.",
        }
    }
}

impl ShapeWord {
    /// **Why this kind of mark shows no node anchors.**
    #[must_use]
    pub const fn no_nodes_line(self) -> &'static str {
        match self {
            Self::Ink => {
                "This freehand mark carries no points pdfcer can read, so there \
                 is nothing here to drag. You can still move, resize or delete \
                 the whole mark."
            }
            Self::Rectangle => {
                "A rectangle has no corners to drag one at a time. Use its \
                 resize handles to change its shape."
            }
            Self::Ellipse => {
                "An ellipse has no corners to drag one at a time. Use its \
                 resize handles to change its shape."
            }
            Self::Line => {
                "A line has two ends. You can drag either one, but a line \
                 cannot gain or lose a corner — draw a polyline for that."
            }
            Self::TextMarkup => {
                "A text mark follows the words it covers, so it has no corners \
                 of its own to move."
            }
            Self::Other => "This kind of mark has no corners to edit.",
        }
    }
}

/// **The mark's stated measurement may now be wrong** — disclosed after a node
/// edit, never guessed at.
#[must_use]
pub const fn measure_stale() -> &'static str {
    "This mark carries a measurement written by another program. Its shape has \
     changed and that number has not — pdfcer will not overwrite it, because \
     it may have been set by hand."
}

/// **A freehand mark another program drew has been redrawn with straight
/// segments** — disclosed after the first point edit on it, never guessed at.
///
/// `InkForecast::appearance_was_pdfces` (`pdfcer-core` `Pass 278.0`) is `false` (old-name-exempt: the engine's own field name, quoted verbatim)
/// when the `/Ink`'s appearance stream on disk is not one pdfcer would have
/// drawn from its `/InkList` — another producer's artwork, typically a smoothed
/// curve through the recorded points. Moving a point forces a re-bake (the old
/// picture would paint the stroke where it no longer is), and pdfcer bakes an
/// `/InkList` as a **polyline**: straight segments between the points. The
/// engine's own account of why this is a sentence and not a silent fix:
///
/// > On a stroke pdfcer did not draw, re-baking replaces that producer's
/// > artwork with pdfcer's polyline rendering, which **visibly straightens a
/// > smoothed curve** … It is answered by `reshape_ink_preview`, deliberately,
/// > so you can say it *before* the first drag instead of explaining it
/// > afterwards.
///
/// ⇒ The geometry moved as asked; the *look* of the stroke changed more than
/// the drag alone explains. Saying so is R8b rule 4's honest half. It fires
/// once per mark in practice: after the first re-bake the appearance is
/// pdfcer's own and the flag is `true` for every later edit.
///
/// Said at **apply** time, on the same disclosure list as
/// [`measure_stale`], rather than before the first drag as the engine
/// suggests. The before-the-drag moment is `canvas::annotnodes::
/// explain_unreshapable`'s shape — once per (shape, tool) — and is a known
/// follow-up; a sentence on release is the minimum that is not a silence.
#[must_use]
pub const fn ink_redrawn_straight() -> &'static str {
    "This freehand mark was drawn by another program. Moving one of its points \
     redraws the whole mark in pdfcer's style — straight segments between its \
     points — so a smoothed stroke will look sharper than before. Undo puts the \
     original drawing back."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The rich-text disclosure fires when a copy was dropped, and stays
    /// silent when none was.**
    #[test]
    fn a_dropped_rich_copy_is_disclosed_and_an_absent_one_is_not() {
        assert_eq!(
            rich_text_dropped(&[]),
            None,
            "★ nothing was dropped, so nothing may be said. pdfcer's own comments carry no \
             rich copy, so this is the path nearly every edit takes"
        );

        let one = rich_text_dropped(&["RC".to_owned()]).expect("a dropped key must be disclosed");
        let two = rich_text_dropped(&["RC".to_owned(), "DS".to_owned()])
            .expect("two dropped keys must be disclosed");
        assert_eq!(
            one, two,
            "★★ the sentence must not vary with WHICH keys went. `/RC` alone happens on a \
             /Square and `/RC` + `/DS` on a /FreeText, and the operator-visible consequence \
             is identical — a comment that had styling is now plain. Two wordings for one \
             consequence is two things to keep true"
        );
    }

    /// **It says what was lost, and it does not say `/RC`.**
    #[test]
    fn the_rich_text_sentence_is_in_his_words_not_the_specs() {
        let said = rich_text_dropped(&["RC".to_owned(), "DS".to_owned()]).expect("a sentence");
        for jargon in ["/RC", "/DS", "RC", "DS", "rich text", "annotation", "key"] {
            assert!(
                !said.contains(jargon),
                "★ {jargon:?} is spec vocabulary and means nothing to a reviewer. Got: {said}"
            );
        }
        assert!(
            said.contains("formatted"),
            "★★ it must name what was actually lost — the formatting — or the operator cannot \
             tell whether their WORDS survived. Got: {said}"
        );
        // THE REASON AND THE OUTCOME ARE TWO CLAIMS, ASSERTED SEPARATELY.
        //
        // This was one `||` of the two until a falsification run caught it:
        // deleting *"two different things"* from the sentence left the test
        // green on *"same everywhere"* alone. They are not two spellings of
        // one fact — the first says **why pdfcer acted**, the second says
        // **what the document is like now** — and a sentence carrying only the
        // second reads as pdfcer discarding something of theirs on a whim.
        //
        // ⇒ **An `||` between two conditions that are both required is an
        // assertion that neither is.** Written down because the `||` looked
        // like tolerance of a rewording and was actually a hole, and because
        // it was found by planting a defect rather than by reading the code.
        assert!(
            said.contains("two different things"),
            "★★★ it must say WHY — that the document was contradicting itself. Without the \
             reason this reads as pdfcer discarding something of theirs on a whim. Got: {said}"
        );
        assert!(
            said.contains("same everywhere"),
            "★★ …and what the document is like NOW, which is the part that says the problem \
             is closed rather than merely reported. Got: {said}"
        );
    }

    /// **Nothing to say says nothing.**
    #[test]
    fn a_delete_with_no_collateral_produces_no_sentence() {
        assert_eq!(deletion_would_take(false, false, 0, 0), None);
    }

    /// **The preview and the disclosure describe the SAME act in two
    /// tenses**, and they must not drift into describing two.
    #[test]
    fn the_two_tenses_name_the_same_four_consequences() {
        let before = deletion_would_take(true, true, 2, 3).expect("collateral");
        let after = deleted_collateral(true, true, 2, 3).expect("collateral");
        for fragment in ["pop-up", "no longer refers to it", "2 replies", "3 grouped"] {
            let in_after = after.contains(fragment);
            let in_before = before.contains(match fragment {
                "no longer refers to it" => "will stop referring to it",
                other => other,
            });
            assert!(
                in_after && in_before,
                "`{fragment}` must appear in both tenses: a preview that names a \
                 consequence the disclosure does not, or the reverse, has \
                 described a different act"
            );
        }
    }

    /// Singular and plural are separate sentences, in both tenses.
    ///
    /// *"1 replies"* is the kind of thing that gets noticed and remembered as
    /// evidence that nobody read the output.
    #[test]
    fn one_is_not_pluralised() {
        let one = deletion_would_take(false, false, 1, 1).expect("collateral");
        assert!(one.contains("1 reply will be"), "{one}");
        assert!(one.contains("1 grouped annotation will be"), "{one}");
        let two = deletion_would_take(false, false, 2, 2).expect("collateral");
        assert!(two.contains("2 replies will be"), "{two}");
        assert!(two.contains("2 grouped annotations will be"), "{two}");
    }

    /// **Neither tense says "removed", and neither mentions the file.**
    #[test]
    fn neither_tense_promises_redaction() {
        for line in [
            deletion_would_take(true, true, 1, 1).expect("collateral"),
            deleted_collateral(true, true, 1, 1).expect("collateral"),
        ] {
            let lower = line.to_lowercase();
            assert!(!lower.contains("removed from"), "{line}");
            assert!(!lower.contains("erased"), "{line}");
            assert!(!lower.contains("from the file"), "{line}");
        }
    }

    /// The locked sentence blames the FILE, not pdfcer.
    #[test]
    fn the_locked_sentence_names_the_file_as_the_author_of_the_rule() {
        let line = annot_delete_locked();
        assert!(line.contains("The file marks"), "{line}");
        assert!(!line.to_lowercase().contains("pdfcer will not"), "{line}");
    }
}
