//! # `text::markup` — the words the Markup ▸ Style group shows
//!
//! Four tooltips, two suffixes and **ten colour names**, which is the whole
//! operator-visible surface of `canvas::markup::swatch`. The controls themselves
//! are colour chips and two numbers: none of them can carry a label without
//! doubling the width of a ribbon group, so **the tooltip is the only place they
//! say what they are** — which makes these strings load-bearing rather than
//! supplementary.
//!
//! ⚠ The count in that first sentence has been wrong before. It read *"Three
//! tooltips and one suffix"* while the opacity tooltip and the percent suffix
//! sat forty lines below it, added on 2026-08-28 without the header being told.
//! A count in prose is a claim nothing checks;
//! [`tests::the_header_counts_what_this_module_actually_holds`] now does.
//!
//! ## Each one answers "what will this change, and when?"
//!
//! Because that is the question a swatch in a ribbon cannot answer by looking
//! like a swatch. Every tooltip here says two things: which markup the setting
//! applies to, and — the half an operator is most likely to get wrong — that it
//! applies to the **next** one rather than to anything already on the page.
//!
//! `RIBBON_IA.md` §5.5 is explicit that these are two different surfaces:
//!
//! > The `Style` group sets defaults for the next markup. Changing an
//! > *existing* markup's style happens on the contextual **Format** tab.
//!
//! The Format tab's property editors are not built yet, so an operator who
//! recolours the swatch expecting the rectangle they just drew to change will
//! be disappointed — and the tooltip is the only thing standing between them
//! and concluding the control is broken. Saying "the next one" is therefore a
//! disclosure and not a nicety.

/// Hover text for the ink swatch.
#[must_use]
pub const fn pen_colour_tooltip() -> &'static str {
    "The colour of the next shape, arrow, line or freehand mark you draw. \
     Marks already on the page keep the colour they were drawn in."
}

/// Hover text for the highlighter swatch.
///
/// A separate control and a separate sentence, because they are separate pens
/// — see `canvas::markup::pen`'s header. An operator who sets the ink to green
/// does not thereby want a green highlight, and a tooltip that said "the
/// markup colour" for both would suggest they had.
#[must_use]
pub const fn highlighter_colour_tooltip() -> &'static str {
    "The colour of the next highlight band. Kept separate from the pen above, \
     so choosing a pen colour does not change your highlighter."
}

/// Hover text for the width control.
///
/// Names the **unit** as well as the effect, because "2" on a ribbon is a
/// number without a scale — and points are what the PDF stores, so it is also
/// the number the operator would see if they opened the file in another
/// program.
#[must_use]
pub const fn pen_width_tooltip() -> &'static str {
    "How thick the next mark's line is, in points — the same unit the document \
     itself uses. A drawing's own linework is often a quarter point, so 2 sits \
     clearly above it without covering it."
}

/// Hover text for the opacity control.
///
/// # ★★★ Why this sentence names the CAD case rather than describing the slider
///
/// Because the reason to reach for it is specific and is not obvious from a
/// percentage: a comment sits on top of the thing it is about, and on a dense
/// drawing an opaque cloud hides the dimension it is drawing attention to. An
/// operator who has never used annotation transparency has no reason to guess
/// that, and a tooltip reading *"the opacity of the next mark"* would restate
/// the label.
///
/// # ★ It says the mark stays selectable, because faint is not gone
///
/// The bottom of the range is a tenth, deliberately (`canvas::markup::pen`'s
/// `MIN_OPACITY` carries the argument), and at a tenth over dark linework a
/// mark can be hard to find with the eye. Saying it is still there and still
/// listed is the disclosure that stops a faint mark reading as a failed one.
#[must_use]
pub const fn pen_opacity_tooltip() -> &'static str {
    "How much of the drawing shows through the next mark. Below 100% the mark \
     is see-through, which is what lets a cloud or a box sit over a dimension \
     without hiding it. Even the faintest mark is still selectable and still \
     listed in the Comments panel."
}

/// The opacity control's suffix.
///
/// A percent sign, because opacity is the one property in this group an
/// operator already thinks about as a percentage — every other program that
/// offers it says 40%, not 0.4. The value written into `/CA` is the fraction;
/// the conversion happens at the control and nowhere else.
#[must_use]
pub const fn opacity_suffix() -> &'static str {
    "%"
}

// ---------------------------------------------------------------------------
// The palette grid — the name of each colour Acrobat marks up in
// ---------------------------------------------------------------------------
//
// ★★★ THESE WORDS ARE THE ONLY LABEL A COLOUR CELL HAS.
//
// A cell in `canvas::markup::palette::ACROBAT` is a filled square about twelve
// points on a side. It cannot carry text, so the tooltip is the whole of its
// accessible name — the same argument this module's header makes about the two
// swatches, one size down and one step more acute, because there are ten of
// them and they differ only by hue.
//
// ★★ THEY ARE PLAIN COLOUR WORDS, NOT ACROBAT ROLES, AND THAT IS A DECISION.
//
// The tempting alternative was "Underline blue", "Sticky-note violet" — naming
// each cell after the Acrobat tool whose default it is. Rejected: one grid is
// offered from every swatch, so a cell reading "Underline blue" under the
// HIGHLIGHTER swatch would be describing a tool the operator is not using and
// a setting they are not making. The Acrobat role is recorded at each palette
// constant's own doc comment, where the reader who wants it is; the operator
// gets the word they would say out loud.
//
// ★ NO HEX, NO RGB TRIPLE. A tooltip reading "Blue (#1373E8)" tells an operator
// choosing a pen colour nothing they can act on, and pushes the useful word off
// the front of a narrow tip. The numbers are in the code and in the palette
// module's table, which is where a number is useful.

/// The palette cell at [`crate::canvas::markup::palette::MARKUP_RED`].
#[must_use]
pub const fn colour_red() -> &'static str {
    "Red"
}

/// The palette cell at [`crate::canvas::markup::palette::HIGHLIGHTER_ORANGE`].
#[must_use]
pub const fn colour_orange() -> &'static str {
    "Orange"
}

/// The palette cell at [`crate::canvas::markup::palette::CLASSIC_YELLOW`].
#[must_use]
pub const fn colour_yellow() -> &'static str {
    "Yellow"
}

/// The palette cell at [`crate::canvas::markup::palette::FREETEXT_GREEN`].
#[must_use]
pub const fn colour_green() -> &'static str {
    "Green"
}

/// The palette cell at [`crate::canvas::markup::palette::UNDERLINE_BLUE`].
#[must_use]
pub const fn colour_blue() -> &'static str {
    "Blue"
}

/// The palette cell at [`crate::canvas::markup::palette::NOTE_PURPLE`].
///
/// **Violet, not purple**, and the difference is worth the thought it took.
/// `#9643FC` sits on the blue side of purple, and the two neighbouring cells are
/// Blue and Magenta — so an operator scanning for "the purple one" between a
/// blue and a magenta gets no help from a word that could mean either. Violet
/// names the position in the spectrum, which is how the cell is found.
#[must_use]
pub const fn colour_violet() -> &'static str {
    "Violet"
}

/// The palette cell at [`crate::canvas::markup::palette::CARET_MAGENTA`].
#[must_use]
pub const fn colour_magenta() -> &'static str {
    "Magenta"
}

/// The palette cell at [`crate::canvas::markup::palette::STRIKEOUT_PINK`].
///
/// Acrobat's strikeout colour, which is a light desaturated red. "Light red"
/// would be the accurate description and is the wrong label: it puts two cells
/// called Red and Light red side by side in a grid, which is a distinction the
/// eye has to make twice. Pink is the word for it.
#[must_use]
pub const fn colour_pink() -> &'static str {
    "Pink"
}

/// The palette cell at [`crate::canvas::markup::palette::BLACK`].
#[must_use]
pub const fn colour_black() -> &'static str {
    "Black"
}

/// The palette cell at [`crate::canvas::markup::palette::WHITE`].
///
/// ★ The one cell whose tooltip earns a second clause. A white mark on a
/// black-on-white CAD sheet is invisible everywhere except over the drawing's
/// own linework, so an operator who picks it by accident sees a tool that has
/// stopped working. Saying so at the moment of choosing is cheaper than the
/// support question.
#[must_use]
pub const fn colour_white() -> &'static str {
    "White — invisible on a white page"
}

/// The heading over the palette grid.
///
/// It names **Adobe**, deliberately and once. The operator's ask was for
/// Acrobat's colours specifically, and a grid captioned "Colours" would look
/// like ten colours somebody liked. This is the one place the provenance of the
/// values is visible from inside the program.
#[must_use]
pub const fn palette_heading() -> &'static str {
    "Acrobat's markup colours"
}

/// The route out of the grid to the full colour picker.
///
/// ★ The trailing ellipsis is the platform convention for *"this opens
/// something"* and is load-bearing here: every other cell in the popup applies
/// immediately, and this one does not.
#[must_use]
pub const fn more_colours() -> &'static str {
    "More colours…"
}

/// Hover text for the More-colours button.
#[must_use]
pub const fn more_colours_tooltip() -> &'static str {
    "Open the full colour picker to choose a colour that is not in the grid. \
     Anything you pick there is used exactly as chosen."
}

/// The width control's suffix.
///
/// A separate entry rather than a literal in the widget call, for the reason
/// the settings window's degree sign is: `check-ui-strings.sh` looks for
/// exactly this, and a translator localising the ribbon must be able to see
/// that a unit abbreviation exists.
#[must_use]
pub const fn width_suffix() -> &'static str {
    " pt"
}

// ---------------------------------------------------------------------------
// Deleting a selected annotation
// ---------------------------------------------------------------------------

/// ★ **What went with it** — the collateral of deleting one annotation.
///
/// # Why a deletion needs to say anything at all
///
/// Because the operator named **one** annotation and the engine may legitimately
/// remove or alter more. `AnnotationDeletion` reports three such cases and each
/// is a fact about the file rather than about pdfcer:
///
/// * a `/Popup` companion goes with its parent — §12.5.6.14 is a `shall`, a
///   pop-up *"shall not appear alone but is associated with a markup
///   annotation"*, so leaving it would be a clause violation. The spec
///   requiring it is a reason, **not a licence to stay quiet**;
/// * replies hanging off it as `/IRT` targets are **orphaned**, not deleted —
///   the thread survives and its root does not;
/// * group members are **promoted** when the group's primary goes.
///
/// Rule 4, in its second clause: pdfcer did something the operator did not ask
/// for, so pdfcer says so, off-canvas, in words.
///
/// # ★ What this deliberately does NOT say
///
/// **That the content is gone from the file.** It is not: deleting an
/// annotation removes an entry from `/Annots` and does not touch page content,
/// and the previous revision is still in the file after an incremental save.
/// `docs/core-api/03-capabilities.md` §3.4 states the rule this observes —
/// *"delete is not redaction"* — and the redaction surface is where that
/// distinction is made loudly. Saying "removed" here would be the exact wording
/// `crate::text::redact`'s header forbids.
///
/// Returns `None` when nothing but the named annotation was affected, which is
/// the ordinary case: a disclosure that fires on every delete is one nobody
/// reads by the third time.
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
///
/// ★★★ **The one consequence of a move that this program cannot show.** §12.5.6.14
/// makes a pop-up a separate annotation with its own placement and leaves to the
/// reader whether it follows; `pdfcer-core` reports the object it left behind and
/// says the decision is the shell's.
///
/// This shell does not draw pop-ups, so a stranded one is invisible here and
/// perfectly visible in Acrobat — which is Rule 4's surviving half in its
/// purest form: render normally, report separately, both.
///
/// ★ It says pdfcer did **not** move it, rather than offering to. Moving it
/// would be a second undo entry for something the operator cannot see, and a
/// gesture that produces two entries is one `Ctrl+Z` away from a state nobody
/// can explain.
#[must_use]
pub fn popup_left_behind() -> String {
    "The note attached to this markup stayed where it was. pdfcer does not show those \
     notes, so you will only see it in a reader that does."
        .to_owned()
}

/// Disclosure: the border width did not scale with the shape.
///
/// ★★★ The engine asks for this sentence by name — *"an operator who scaled a
/// square 3× and expected a heavier border needs telling it stayed"* — and it is
/// Rule 4's surviving half in its purest form: the shape grew around the border
/// and **nothing on the canvas says the border did not grow with it**.
///
/// ★★ It states the default as a **choice**, not as a limitation, because it is
/// one: on a CAD drawing a line weight is a drafting standard rather than
/// decoration, which is this project's own argument and the one the engine
/// promoted into the rule that decides every future case — *is the property a
/// length in the space being transformed?* An inset is; a line weight is not.
#[must_use]
pub fn stroke_width_unchanged() -> String {
    "The outline's thickness has not changed. pdfcer treats a line weight as a drawing standard \
     rather than something that scales with the shape."
        .to_owned()
}

/// Disclosure: a foreign appearance was scaled unevenly and its stroke is now
/// anisotropic.
///
/// ★★★ Not a defect and not pdfcer's choice — an arithmetic limit. **Neither PDF
/// nor SVG has a per-axis stroke width**: both are scalars, so a stroke drawn
/// through a matrix applied *after* stroking cannot keep an even thickness under
/// a non-uniform scale. Inkscape closed the identical report **Invalid** and
/// silently produces the distorted stroke.
///
/// ★★ pdfcer says so instead, which is the whole difference. The operator can see
/// the result — a border thicker on one axis — and cannot see *why*, so the
/// sentence names the cause and the remedy: drag a corner with Shift held, or
/// accept it.
#[must_use]
pub fn appearance_distorted() -> String {
    "This markup was drawn by another program, and scaling it unevenly has made its outline \
     thicker on one side than the other. Hold Shift while dragging a corner to scale it evenly."
        .to_owned()
}

/// **Disclosure: the words this note used to carry, on the case where a save
/// overwrote them.**
///
/// ★★★ Rule 4's surviving half, and `pdfcer-core` commissioned this sentence
/// itself: *"those words are gone from the document and nothing on the page
/// shows that they were ever there"*. A shape does not change when its note
/// does. A sticky's words live in a pop-up window this shell does not draw. So
/// on every subtype an operator can comment on, overwriting a note is an edit
/// with **no visible consequence at all** — which is precisely the class this
/// project's disclosure rule exists for.
///
/// ★★ It carries **the text, not a count**, because the engine chose to return
/// the text and said why: a count lets a shell *mention* the loss, and the text
/// lets it *offer the words back*. They are on the status line for as long as
/// the edit epoch holds, so an operator who overwrote the wrong comment can
/// read what was there and retype it — `Ctrl+Z` restores it outright, and this
/// is the surface that tells them there is something to undo.
///
/// ★ `None` when the annotation had no note, which is the ordinary case for
/// every shape this shell draws: a disclosure that fires on every save is one
/// nobody reads by the third time. Same rule as [`deleted_collateral`].
///
/// # The truncation, and why it is not a formatting decision
///
/// A `/Contents` may legitimately be a paragraph. The status line is one
/// bounded row that elides rather than wraps (`DEFECTS.md` R128), so a long
/// previous note would be cut by the *layout* with no indication that it had
/// been. Cutting it here, with an ellipsis and a stated character count, is the
/// difference between an operator seeing all of a short note and believing they
/// have seen all of a long one.
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
///
/// The same argument as [`note_replaced`] at its strongest — a removal leaves
/// the markup on the page looking exactly as it did — so this fires even for a
/// short note and never returns `None` for a note that had words.
///
/// ★ It says the markup itself stayed, because that is the thing an operator
/// pressing a button labelled *Remove note* most reasonably fears they have
/// just done, and the canvas cannot answer it: a shape with a note and the same
/// shape without one are the same picture.
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

/// ★★★ **Why the Delete control is absent for the selected annotation** —
/// `EditSession::annotation_deletion_refusal` answered `Some` (R83).
///
/// # The defect this closes, stated as it was found
///
/// `annotation_deletion_refusal` is a **pure query**. Its own doc comment names
/// this call site in as many words — *"safe to call every frame from a UI (R83:
/// ask before offering the control)"* — and until this landed **nothing in this
/// shell called it**. On a certified drawing the Format tab's Delete, the
/// canvas right-click's Delete and the Delete key were all live, and every one
/// of them ended in `crate::app::actions::apply::vector_edit`'s `Err` arm,
/// which wrote one line to the trace and **said nothing at all to the
/// operator** — it words one un-categorised sentence since O116 (2026-09-04),
/// which names no cause and so replaces none of these.
/// That is the identical shape the forms panel's `deletion_refusal`
/// audit found the day before (`crate::panels::properties::formfield`), one
/// annotation kind along, and it was found the same way: by asking what the
/// engine offers rather than by re-reading this shell.
///
/// # ★★ Why an enum rather than one sentence
///
/// Because the two reachable causes are **different facts about the operator's
/// file** and only one of them is about a signature. An encrypted drawing and a
/// certified one look identical on the canvas; telling an operator that a
/// signature forbids the delete when in fact the file is encrypted sends them
/// hunting for a signature that is not there. The mapping is
/// [`crate::panels::properties::annotdelete::refusal_for`], a total match
/// written in the engine's own guard order — exactly the shape
/// `crate::app::actions::xobject::refusal_for` has for `unshare_form`, and for
/// the same reason: a `_ =>` that silently swallows a mistyped variant name
/// turns an instruction back into a dead end and the compiler stays happy.
///
/// # ★★★ What none of these sentences does is offer a remedy it cannot back
///
/// The forms twin ends *"the values in it can still be filled in and changed"*,
/// which is true and checkable: `fill_refusal` allows at `/P 2` where
/// `deletion_refusal` refuses, so there is a second verb to point at. **There is
/// no such second verb here.** §12.8.2.2 Table 254 puts annotation *creation,
/// deletion and modification* on one line, so at `/P 2` an operator cannot add a
/// comment either, and a sentence suggesting they could would be an invented
/// remedy of exactly the kind this project's copy rule forbids.
///
/// ⇒ Each of these therefore explains **why** instead. That is what keeps a
/// refusal from reading as a dead end without claiming something the engine
/// would refuse the moment the operator tried it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotDeleteRefusal {
    /// The document carries `/Encrypt` (§7.6) — `EditError::DocumentEncrypted`,
    /// the engine's first guard.
    ///
    /// ★ Reachable on an entirely ordinary file: plenty of drawing sets ship
    /// with an owner password set for printing, and nothing on the canvas says
    /// so.
    Encrypted,
    /// An enforced certification signature whose `/P` is below 3 (§12.8.2.2
    /// Table 254) — `EditError::CertificationForbidsChange`.
    ///
    /// ★ Table 254 makes `/P` **Optional with default 2**, so a certified
    /// document that states no permission at all lands here — absence is
    /// permissive relative to `P = 1` and not relative to `P = 3`. The
    /// permission number is deliberately **not** carried into the wording:
    /// `1` and `2` refuse for the same reason and leave the operator nothing to
    /// do differently, so printing it would be jargon in place of a fact.
    ///
    /// ★★ `P = 3` is the row this query exists for and it does **not** land
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
            Self::Encrypted => {
                "This document is encrypted, so pdfcer cannot change anything inside it. \
                 Its comments and markup can be read here but not deleted."
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
///
/// # ★★ Why this is a free function and not a fourth [`AnnotDeleteRefusal`]
/// variant
///
/// Because it comes from a different place and has a different scope. Every
/// member of that enum is derived from an `EditError` and describes the **whole
/// document**; this is a bit in the selected annotation's own flags word, and
/// two annotations on one page can disagree about it. Folding it in would make
/// `crate::panels::properties::annotdelete::refusal_for` — a total match over
/// `EditError` — answerable for a variant no `EditError` produces.
///
/// # ★★★ It is the more actionable of the two facts, and that is why it wins
///
/// A certified document and a locked annotation can both be true at once, and
/// the gate checks this one **first**. An operator told *"this comment is marked
/// as one that should not be changed"* has somewhere to go: they can look at
/// that comment, ask whoever placed it, or select a different one. An operator
/// told *"the document is certified"* can do nothing about one annotation. When
/// both are true, the sentence that leaves the operator with a next step is the
/// one worth saying.
///
/// ★ It says *"the file marks"*, not *"pdfcer will not"*. §12.5.3's `Locked` is a
/// statement the **producer** wrote into the annotation, and this shell honours
/// it rather than imposing it. Wording it as pdfcer's decision would send an
/// operator looking for a pdfcer setting to turn it off.
#[must_use]
pub const fn annot_delete_locked() -> &'static str {
    "The file marks this comment as one that should not be changed, so pdfcer does not \
     offer to delete it. Other comments on this page may still be deleted."
}

/// ★★★ **What deleting the selected annotation would take with it**, said
/// *before* the click — `EditSession::annotation_deletion_preview`.
///
/// # The future-tense twin of [`deleted_collateral`], and why they live together
///
/// They are one vocabulary in two tenses and they must not drift. A preview that
/// says *"1 reply will be left without the comment it replied to"* followed by a
/// disclosure that says *"1 grouped annotation is now on its own"* has described
/// two different acts, and the operator has no way to tell which of the two
/// lied. Keeping them adjacent in one file is the cheapest guard available — a
/// reader editing either one sees the other — and the counts they render come
/// from **one engine body**: `plan_annotation_deletion` is shared between
/// `annotation_deletion_preview` and `delete_annotation` precisely so that
/// *"the warning cannot disagree with the act."*
///
/// # ★★ Why this is worth showing at all, in the engine's own words
///
/// > Two of the counts describe consequences on **other** annotations — replies
/// > that stop being replies, and group subordinates whose previously-suppressed
/// > text becomes visible (§12.5.6.2). Those are exactly the facts rule 4 says an
/// > operator must be able to see *before* they act.
///
/// A reply three rows down a scrolled Comments list is not visible, and this
/// shell's delete carries no confirmation dialog by deliberate decision
/// (decision 024 §4.4's no-confirm carve-out, which is **conditioned on the
/// result being visible**). So the only moment this fact can reach the operator
/// is while the annotation is selected and before the key goes down, which is
/// where the panel puts it.
///
/// # ★ It returns `None` far more often than not, and that is the design
///
/// The overwhelmingly common annotation has no pop-up, no replies and no group,
/// and there is nothing whatever to say about deleting it. A sentence that
/// appeared on every selection would be read the first three times and skipped
/// for ever after — which is the failure mode that makes the *interesting* case
/// invisible. R9: nothing to say renders nothing.
///
/// ★★ **It does not say "removed" and it does not mention the file.** Deleting
/// an annotation takes an entry out of `/Annots`; it does not touch page
/// content, and an incremental save leaves the previous revision in the file.
/// `docs/core-api/03-capabilities.md` §3.4 — *"delete is not redaction"* — and
/// [`deleted_collateral`] observes the same rule in its own wording, which is
/// the other half of why these two functions sit together.
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

// ===========================================================================
// Editing the NODES of a markup shape — the operator's report of 2026-09-05
// ===========================================================================
//
// > *"I also can't edit or delete nodes of a markup shape once it is drawn."*
//
// `canvas::annotnodes` draws an anchor on every node of a selected `/Polygon`,
// `/PolyLine` or `/Line` and drags them. These are the sentences for the cases
// where it cannot, and every one of them exists for the reason this project was
// founded on: **a refusal must be a sentence, never a silence.** A drag that is
// released and does nothing is the exact defect the operator reported, and a
// shape that shows no anchors at all is the same defect one step earlier.

/// **The operator's word for a shape**, which is not always the PDF name.
///
/// ★★ A mapping and not a passthrough, and each row is a place the file's
/// vocabulary and the operator's disagree:
///
/// | `/Subtype` | what pdfcer's own ribbon calls it |
/// |---|---|
/// | `Square` | **rectangle** — the Rectangle tool draws it |
/// | `Circle` | **ellipse** — it is an ellipse inscribed in `/Rect`, not a circle |
/// | `Ink` | **freehand mark** — the Freehand tool draws it |
/// | `Highlight`/`Underline`/`StrikeOut`/`Squiggly` | **text mark** |
///
/// Showing "Square" to an operator who drew a rectangle is the surface
/// disagreeing with itself about what it just did, and it sends them looking
/// for a Square tool that does not exist.
///
/// [`ShapeWord::Other`] is the honest arm for a subtype this shell has never
/// heard of: the sentence it produces says *this kind of mark* rather than
/// inventing a name, because a wrong name is worse than a general one.
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
///
/// ★ `Copy` and fieldless-payloaded, because [`crate::app::status`]'s decline
/// store is `Copy` and its `line()` returns `&'static str`. That constraint is
/// the reason no sentence here quotes a *number* — a floor of 3 for a polygon
/// and 2 for a polyline would each need their own static string, and the
/// operator does not need the number to know what to do next. `VertexEditRefusal`
/// makes the identical choice for ce dimensions, and this enum is deliberately
/// its sibling rather than a reuse of it: the ce-dimension sentences say
/// *"measurement"*, which is the wrong word for a comment shape (R8b rule 15),
/// and one enum serving both would have to say something vague enough to be
/// true of either.
///
/// ★★ The engine offers a `reason: &'static str` on
/// `EditError::GeometryNotReshapable` and says a shell may show it verbatim.
/// It is deliberately not shown: those sentences name PDF keys and engine verbs
/// (*"author a PolyLine instead"*, *"use resize_annotation"*, *"/QuadPoints are
/// text-anchored quadrilaterals"*) and are written for a developer at a CLI.
/// They go to the trace, where that reader is. `canvas::annotnodes::refusal_for`
/// carries the argument at the mapping site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeEditRefusal {
    /// `EditError::ReshapeWouldBreachVertexFloor` — the shape is at its floor.
    ///
    /// The one an operator meets by doing something perfectly reasonable: a
    /// triangle they want to make into a line. The floors are the engine's own
    /// (`/Polygon` keeps three, `/PolyLine` keeps two) and it refuses by name
    /// rather than silently clamping or turning the shape into something else.
    WouldLeaveTooFew,
    /// `EditError::GeometryNotReshapable` — this kind of mark has no nodes to
    /// edit, and the sentence says which kind it is.
    ///
    /// Reached two ways, which is why it carries the word rather than the
    /// gesture: from a **drag** on a `/Line`'s end that asked to add or remove
    /// one, and from [`crate::canvas::annotnodes::explain_unreshapable`] when
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
    ///
    /// Each names what is true rather than what the engine called it, and each
    /// names a **next act** where there is one — which is the rule
    /// `resize_not_rebuildable` states and the reason the ce-dimension twin
    /// gives: at the moment it is read the operator has just released a drag
    /// and seen nothing happen, and what they need is what to do, not a
    /// diagnosis.
    #[must_use]
    pub const fn line(self) -> &'static str {
        match self {
            Self::WouldLeaveTooFew => {
                "That shape has as few corners as it can have. Add one before \
                 taking one away, or delete the whole mark."
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
    ///
    /// ★★ Every one of these says what the operator *can* do instead, because
    /// each of them can do something: a rectangle and an ellipse resize, a
    /// freehand mark moves and resizes as a whole, a line's two ends drag. A
    /// sentence that only said "no" would leave them looking for a control that
    /// does not exist.
    ///
    /// ★ The freehand sentence deliberately does **not** apologise or promise.
    /// pdfcer refuses per-point ink editing on purpose — the engine's words:
    /// *"an `/InkList` stroke is a recorded pen trace, and Acrobat has never
    /// offered per-point ink editing at any version"* — so wording it as a
    /// missing feature would be this surface predicting work nobody has agreed
    /// to.
    #[must_use]
    pub const fn no_nodes_line(self) -> &'static str {
        match self {
            Self::Ink => {
                "A freehand mark has no corners to edit — it is a recorded pen \
                 stroke. You can move, resize or delete the whole mark."
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
///
/// `ReshapeForecast::measure_not_recomputed` is `true` when the annotation
/// carries a `/Measure` dictionary (§12.9) whose number pdfcer did **not**
/// recompute. The engine's account of why it does not is the reason this
/// sentence exists rather than a silent fix:
///
/// > Acrobat recomputes the number and — a sourced user complaint — silently
/// > clobbers any manual override in doing so. pdfcer's markup bake draws no
/// > caption and reads no `/Measure`, so it neither recomputes nor clobbers.
///
/// ⇒ The geometry moved and the text did not. Saying so is the whole of R8b
/// rule 4's honest half: an operator who is not told will read a number that
/// describes the shape before their drag.
#[must_use]
pub const fn measure_stale() -> &'static str {
    "This mark carries a measurement written by another program. Its shape has \
     changed and that number has not — pdfcer will not overwrite it, because \
     it may have been set by hand."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ Every tooltip says the setting applies to the NEXT mark.
    ///
    /// The disclosure this module exists for. `RIBBON_IA.md` §5.5 puts
    /// "restyle what is already there" on the contextual Format tab, whose
    /// property editors are not built — so an operator who recolours the swatch
    /// expecting the rectangle they just drew to change has no other way to
    /// learn otherwise, and would reasonably report the control as broken.
    ///
    /// A test rather than a convention, because the natural edit when a tooltip
    /// reads long is to cut its second sentence.
    #[test]
    fn every_style_tooltip_says_it_applies_to_the_next_mark() {
        for tip in [
            pen_colour_tooltip(),
            highlighter_colour_tooltip(),
            pen_width_tooltip(),
        ] {
            assert!(
                tip.contains("next"),
                "a Style tooltip no longer says it applies to the next mark: {tip:?}"
            );
        }
    }

    /// ★★ **The ten palette names are ten different words.**
    ///
    /// A cell's name is its whole accessible label — see this module's palette
    /// section — so two cells reading "Purple" would be two controls an operator
    /// cannot tell apart by any means the program offers, hover included.
    ///
    /// It also asserts each is non-empty, which is the failure a `const fn`
    /// returning `""` produces: a cell with no tooltip at all, silently, on a
    /// control that has nothing else to say what it is.
    #[test]
    fn every_palette_cell_has_its_own_word() {
        let names = [
            colour_red(),
            colour_orange(),
            colour_yellow(),
            colour_green(),
            colour_blue(),
            colour_violet(),
            colour_magenta(),
            colour_pink(),
            colour_black(),
            colour_white(),
        ];
        for (i, name) in names.iter().enumerate() {
            assert!(!name.trim().is_empty(), "cell {i} has no name at all");
            for (j, other) in names.iter().enumerate().skip(i + 1) {
                assert_ne!(name, other, "cells {i} and {j} are both named {name:?}");
            }
        }
    }

    /// ★ **The palette heading names Adobe, and the white cell warns.**
    ///
    /// Two disclosures that a shortening edit would take out first, and both are
    /// the kind this project does not leave to convention:
    ///
    /// * the heading is the only place in the running program where the
    ///   provenance of these ten values is visible — the operator asked for
    ///   *Adobe's* colours and is entitled to see the claim being made;
    /// * white is invisible on a white page, and an operator who picks it sees a
    ///   tool that has stopped working rather than a colour they chose.
    #[test]
    fn the_palette_says_where_its_colours_came_from() {
        assert!(
            palette_heading().contains("Acrobat"),
            "the heading must name the program these values were measured from: {:?}",
            palette_heading()
        );
        assert!(
            colour_white().to_lowercase().contains("invisible"),
            "the white cell must warn that it disappears on a white page: {:?}",
            colour_white()
        );
        assert!(
            more_colours().ends_with('…'),
            "the ellipsis is the convention for 'this opens something', and it is \
             the only cell in the popup that does: {:?}",
            more_colours()
        );
    }

    /// ★★★ **The header's count of what this module holds is checked.**
    ///
    /// It read *"Three tooltips and one suffix"* for four months after a fourth
    /// tooltip and a second suffix were added. Nothing was broken by it and
    /// nobody could have noticed, which is exactly the class of statement that
    /// rots — a count in prose is a claim with no reader that verifies it.
    ///
    /// Falsified by changing the header to say "five tooltips": the assertion
    /// fired. Restored.
    #[test]
    fn the_header_counts_what_this_module_actually_holds() {
        let header = include_str!("markup.rs");
        let first_line = header
            .lines()
            .find(|l| l.contains("tooltips"))
            .expect("the header's opening sentence names a count of tooltips");
        // Four: pen colour, highlighter colour, width, opacity. Counted here
        // rather than derived, because the point is to compare the prose against
        // a number a human had to think about.
        let tooltips = [
            pen_colour_tooltip(),
            highlighter_colour_tooltip(),
            pen_width_tooltip(),
            pen_opacity_tooltip(),
        ];
        assert_eq!(tooltips.len(), 4);
        assert!(
            first_line.contains("Four tooltips"),
            "this module holds {} tooltips and its header says: {first_line:?}",
            tooltips.len()
        );
        let suffixes = [opacity_suffix(), width_suffix()];
        assert_eq!(suffixes.len(), 2);
        assert!(first_line.contains("two suffixes"), "{first_line:?}");
    }

    /// The two colour tooltips are different sentences about different pens.
    ///
    /// They are two controls sitting side by side with no labels, so identical
    /// or near-identical hover text would make them indistinguishable — which
    /// is the state the operator is already in before they hover.
    #[test]
    fn the_two_swatches_are_told_apart_by_their_words() {
        assert_ne!(pen_colour_tooltip(), highlighter_colour_tooltip());
        assert!(highlighter_colour_tooltip().contains("highlight"));
    }

    /// ★★★ **Nothing to say says nothing.**
    ///
    /// The overwhelmingly common annotation has no pop-up, no replies and no
    /// group. A sentence that appeared on every selection would be read the
    /// first three times and skipped for ever after, which is exactly what makes
    /// the interesting case invisible when it finally arrives.
    #[test]
    fn a_delete_with_no_collateral_produces_no_sentence() {
        assert_eq!(deletion_would_take(false, false, 0, 0), None);
    }

    /// ★★ **The preview and the disclosure describe the SAME act in two
    /// tenses**, and they must not drift into describing two.
    ///
    /// Pinned by counting: for one set of counts each function names every
    /// consequence the other names. What is deliberately NOT asserted is that
    /// the strings are equal or mechanically derived — they are not, because
    /// *"1 reply is left"* and *"1 reply will be left"* are different English —
    /// and a test that demanded a shared template would forbid the difference
    /// that makes both of them readable.
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

    /// ★ Singular and plural are separate sentences, in both tenses.
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

    /// ★★★ **Neither tense says "removed", and neither mentions the file.**
    ///
    /// Deleting an annotation takes an entry out of `/Annots`; it does not touch
    /// page content, and an incremental save leaves the previous revision in the
    /// file. `docs/core-api/03-capabilities.md` §3.4 — *"delete is not
    /// redaction"* — and a preview that promised removal would be the exact
    /// wording `crate::text::redact`'s header forbids, stated one gesture
    /// earlier than the disclosure that already observes the rule.
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

    /// ★ The locked sentence blames the FILE, not pdfcer.
    ///
    /// §12.5.3's `Locked` is a statement the producer wrote into the annotation.
    /// Wording it as pdfcer's own decision would send an operator looking for a
    /// pdfcer setting to turn off, and there is not one.
    #[test]
    fn the_locked_sentence_names_the_file_as_the_author_of_the_rule() {
        let line = annot_delete_locked();
        assert!(line.contains("The file marks"), "{line}");
        assert!(!line.to_lowercase().contains("pdfcer will not"), "{line}");
    }

    /// The width suffix names a unit and is not empty.
    #[test]
    fn the_width_carries_its_unit() {
        assert!(width_suffix().contains("pt"));
        assert!(pen_width_tooltip().contains("points"));
    }
}
