//! # `text::forms::authoring` — the words for **making** a form field
//!
//! [`super`] covers **filling** an `/AcroForm` that already exists. This file
//! covers the opposite direction, added 2026-08-26 on the operator's request to
//! place form controls from the ribbon: the five kinds' nouns, the placement
//! dialog's labels, and the disclosures the engine's `FieldAuthorOutcome`
//! obliges.
//!
//! Its own file rather than more of `super`, for the reason `super`'s header
//! gives about itself: *"the reviewer of a disclosure sentence is reading a file
//! that contains nothing but disclosure sentences"*. It also keeps both files
//! comfortably inside R2 — `super` is already at 1,265 lines.
//!
//! ## ★★★ The four disclosures, and why they are status lines
//!
//! `FieldAuthorOutcome` reports four things about a field that has just been
//! authored, and **not one of them is visible on the rendered page**. That is
//! the exact condition rule 4's surviving half describes: an inference the
//! operator cannot see still owes an off-canvas report. Decision 059 settled
//! *where* — the status line, never a mark on the canvas — so a screenshot of
//! the page with a merged field and a screenshot with an independent one are
//! identical, which is correct, and the status bar is what tells them apart.
//!
//! The sharpest is [`form_field_merged`]. A name that matches an existing field
//! does not make a second field; it makes a second **view** of the first, so
//! typing in one changes the other. An operator who meant to place two
//! independent boxes has placed one, and nothing about the page says so.
//!
//! ## What is deliberately NOT here
//!
//! The field-name stems (`Text`, `Check Box`, `Group`, …). Those are `/T`
//! strings written into the file, are what a form-filling script and an FDF
//! import key on, and translating them would rename every field for an operator
//! running a different language — invisibly, until the import failed. They live
//! on `FormFieldKind::name_prefix` as literals with that reasoning attached.

/// The noun for a text field, in a sentence.
#[must_use]
pub fn form_noun_text() -> String {
    "Text field".to_owned()
}

/// The noun for a check box, in a sentence.
#[must_use]
pub fn form_noun_check_box() -> String {
    "Check box".to_owned()
}

/// The noun for a radio button, in a sentence.
#[must_use]
pub fn form_noun_radio() -> String {
    "Radio button".to_owned()
}

/// The noun for a drop-down or list box, in a sentence.
///
/// ★ "Drop-down list" rather than "choice field", which is the PDF spec's word
/// (`/Ch`) and means nothing to anyone who has not read it. The operator's
/// standing tie-breaker — *make it work the way other programs do* — applies to
/// vocabulary as much as to behaviour, and every program calls this a drop-down.
#[must_use]
pub fn form_noun_choice() -> String {
    "Drop-down list".to_owned()
}

/// The noun for a push button, in a sentence.
#[must_use]
pub fn form_noun_push_button() -> String {
    "Button".to_owned()
}

/// **The field was authored.** The one line every placement produces.
///
/// It names the kind rather than saying "field added", because five commands
/// place five different things and a generic confirmation cannot tell an
/// operator that the button they pressed was not the one they meant.
#[must_use]
pub fn form_field_added(noun: &str) -> String {
    format!("{noun} added.")
}

/// ★★★ **The name matched an existing field, so this widget joined it.**
///
/// The single most important sentence in this file, and the one with the least
/// visible cause. In PDF a fully-qualified name *is* the field's identity: two
/// widgets carrying the same one are one field with two appearances on the
/// page, and typing into either changes both.
///
/// The page looks exactly as it would if they were independent. So this is
/// stated plainly, with what it means rather than with the word "merged" —
/// which is the engine's word and describes the mechanism, not the consequence.
#[must_use]
pub fn form_field_merged() -> String {
    "That name already existed, so this control shows the same value as the \
     other one — typing in either changes both. Give it a different name if you \
     wanted two separate fields."
        .to_owned()
}

/// **No tooltip was given**, and what that costs.
///
/// Not a scolding and not a warning: leaving it blank is a legitimate decision
/// and the engine accepts it as one. What the operator may not know is the
/// consequence, which is entirely invisible on screen — a screen reader has
/// nothing to announce for this control but its type.
#[must_use]
pub fn form_field_no_tooltip() -> String {
    "It has no tooltip, so a screen reader will announce only what kind of \
     control it is."
        .to_owned()
}

/// **A drop-down with no options in it.**
///
/// Authorable, and empty. Worth saying because an empty list renders as a
/// control that opens and shows nothing, which reads as a broken field rather
/// than an unfinished one.
#[must_use]
pub fn form_field_no_options() -> String {
    "It has no options yet, so it will open empty.".to_owned()
}

/// **The document is tagged, and this control is not in the tag tree.**
///
/// Covers both `tagged_document` and `structure_tab_order` in one sentence,
/// deliberately: they are two symptoms of one situation, and an operator who
/// gets two lines about the same thing reads the second as a separate problem.
///
/// ★ It says what is true rather than what to do, because pdfcer cannot yet fix
/// it and a line that recommended an action it does not offer would be worse
/// than one that reports a fact.
#[must_use]
pub fn form_field_tagged_document() -> String {
    "This document is tagged for accessibility, and the new control is not in \
     its structure tree — its reading order will not include this field."
        .to_owned()
}

/// **The field was renamed.**
///
/// ★★ It names `descendants_renamed` when there are any, and that is the whole
/// reason this takes two arguments. Renaming a field that has children renames
/// their fully-qualified names too — `Address` becoming `Postal` turns
/// `Address.Line1` into `Postal.Line1` — because a qualified name is built from
/// the parent chain. The operator renamed one thing and several changed, and
/// every one of those is a name an FDF import or a filling script keys on.
/// Nothing on the page says so.
#[must_use]
pub fn form_field_renamed(to: &str, descendants: usize) -> String {
    if descendants == 0 {
        format!("Renamed to \u{201c}{to}\u{201d}.")
    } else {
        format!(
            "Renamed to \u{201c}{to}\u{201d}. {descendants} field(s) inside it were renamed \
             with it, because their names are built from this one."
        )
    }
}

/// **The field was deleted**, and how many boxes went with it.
///
/// ★ The count is the part that cannot be seen. A field drawn in three places
/// disappears from three pages and the operator is looking at one of them.
#[must_use]
pub fn form_field_deleted(widgets: usize) -> String {
    if widgets <= 1 {
        "Field deleted.".to_owned()
    } else {
        format!("Field deleted, including {widgets} boxes across the document.")
    }
}

/// **One box was deleted and the field remains.**
#[must_use]
pub fn form_widget_deleted() -> String {
    "Box deleted. The field is still in the form, drawn elsewhere.".to_owned()
}

/// ★★ **The last box went, so the field went with it** — which is not what the
/// operator pressed.
///
/// `delete_widget` removes the field when its last widget goes, and that is
/// right: a named field nothing draws is a field nothing can fill. It is still
/// a larger outcome than the button promised, so it is said.
#[must_use]
pub fn form_widget_deleted_last() -> String {
    "That was the field's last box, so the field was removed from the form too.".to_owned()
}

// ===========================================================================
// EDITING A FIELD THAT IS ALREADY PLACED — `Pass 134.0`, consumed 2026-08-27
// ===========================================================================

/// **The `Sort` flag was set over a list nobody has sorted.**
///
/// ★★ pdfcer will not reorder `/Opt` on the operator's behalf and this sentence
/// is why that is the right refusal rather than an omission. Table 230 makes
/// `Sort` *"intended for use by writers, not by readers"* and requires a
/// conforming reader to display the options *"in the order in which they occur
/// in the Opt array"* — so `Sort` is a **claim about provenance**, not an
/// instruction. Setting it over an unsorted list makes the file say something
/// untrue; silently sorting would change what the operator sees in a
/// drop-down without being asked.
///
/// So the operator gets the flag they asked for and the sentence that says what
/// it now claims. Both, which is Rule 4's *render normally, report separately*.
#[must_use]
pub const fn field_sort_claim_unmet() -> &'static str {
    "The Sort flag now says this list was sorted by whoever wrote the file, and it is not in \
     order. pdfcer has not reordered it — the order options appear in is what a reader shows."
}

/// **One field's flag changed and several boxes on the page followed.**
///
/// ★★ The engine's scope table, taken verbatim from Acrobat's own scripting
/// model: some properties *"apply to all widgets that are children of that
/// field"* and others *"are specific to individual widgets"*. Required,
/// read-only, the tooltip and the type flags are all in the first group — one
/// write, every placement.
///
/// The operator is looking at **one** box. A field drawn in three places has
/// just changed in three places, two of which may be on other pages, and
/// nothing on screen would otherwise say so. `widgets_affected` is reported by
/// the engine *"to be shown"*, in its own words.
///
/// ★ Said only when the count is above one. On the overwhelming majority of
/// fields it is exactly one, and a bar that narrated that would stop being
/// read.
#[must_use]
pub fn field_widgets_affected(widgets: usize) -> String {
    format!(
        "This field is drawn in {widgets} places, and all {widgets} changed — including any \
         on other pages."
    )
}

/// **The widget was resized and its artwork could not be rebuilt.**
///
/// ★★★ The one disclosure here that is about something the operator can SEE
/// and will misread. §12.5.5 derives the appearance matrix from the appearance
/// box's corners and the `/Rect` corners, so a pure translation moves baked
/// artwork exactly and a changed **extent** makes the same algorithm *scale*
/// it. `edit_widget` rebuilds the appearance when the extent changed — except
/// where it cannot: a push button's baked caption, or a signature.
///
/// The widget then renders **distorted**, and `appearance_stale` is the
/// engine's own string saying which one and why. It is passed through verbatim
/// and this sentence prefixes it, because "stale appearance" is a phrase about
/// the file and "it will look stretched" is a fact about the screen.
#[must_use]
pub fn field_appearance_stale(why: &str) -> String {
    format!(
        "This box was resized and its artwork could not be redrawn, so it will look stretched: {why}"
    )
}

/// **A widget was moved or resized.**
///
/// ★ It names which of the two happened, because the engine distinguishes them
/// and the consequences differ: a move keeps the baked artwork exact and free,
/// a resize rebuilds it. An operator who dragged a corner and one who dragged
/// the middle have done different things to the file.
#[must_use]
pub fn field_widget_moved(resized: bool, regenerated: bool) -> &'static str {
    match (resized, regenerated) {
        // ★★★ **The third case, added 2026-08-31 — `OPERATOR_REQUESTS.md` O76.**
        //
        // The operator: *"Form shape outlines of checkboxes and such scale
        // when I drag them larger."*
        //
        // They do, and until today pdfcer told him the opposite. This sentence
        // was chosen on `outcome.resized` alone, so a resize that regenerated
        // nothing still said *"its contents were redrawn to fit"* — a claim
        // the very outcome it was reading denied on the next field.
        //
        // `regen_after_property_change` returns `Ok(false)` for every field
        // type except Text and Choice, and a check box is `/FT /Btn`. So the
        // appearance pdfcer itself drew — `BBox` = the ORIGINAL box, stroke
        // authored at a hard-coded 1.0 — is kept, and §12.5.5 stretches it
        // into the new `/Rect`. Drag a 12 pt check box to 40 pt and its 1 pt
        // border draws at about 3.3 pt, and the tick thickens with it.
        //
        // ★★ That is precisely the case `resize_annotation` REFUSES by name
        // for a foreign appearance — *"a foreign appearance cannot be rebuilt
        // without replacing somebody else's artwork with pdfcer's rendering of
        // it"* — and the widget path takes it silently, on artwork pdfcer drew
        // and could therefore rebuild exactly.
        //
        // ⇒ The engine half is filed
        // (`request_resizing_a_check_box_stretches_its_appearance.md`). This
        // sentence is what pdfcer owes in the meantime, and it is **not** a
        // lesser version of the fix: it is the difference between a program
        // that is wrong and one that is honest about being limited. It is
        // deleted, not reworded, on the day the engine redraws a `/Btn`.
        //
        // It names the appearance rather than "the artwork" because the
        // operator is looking at a tick and a border, and "stretched" is what
        // he can see happening.
        (true, false) => {
            "The box was resized. Its contents could not be redrawn at the new size, so they \
             are stretched to fit it."
        }
        (true, true) => "The box was resized and its contents were redrawn to fit.",
        // ★ A move that regenerated is not distinguished from one that did
        // not, and that is correct rather than an omission: a translation
        // changes no length, so an appearance carried across is exact and
        // there is nothing to disclose. Only a RESIZE can be unsatisfiable.
        (false, _) => "The box was moved.",
    }
}

/// **The other placements of this field were left where they are.**
///
/// The mirror of [`field_widgets_affected`], and the reason the two exist as a
/// pair: a *field* edit changes every box and a *widget* edit changes one, so
/// an operator working on a field drawn in three places needs to know which
/// kind of control they just used. `siblings_untouched` is the engine's own
/// count, reported *"to be shown"*.
#[must_use]
pub fn field_siblings_untouched(siblings: usize) -> String {
    format!(
        "The other {siblings} box(es) of this field are unchanged — a box's size and border \
         belong to that placement alone."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **A resize that redrew nothing does not claim it redrew something**
    /// — `OPERATOR_REQUESTS.md` O76.
    ///
    /// The three cases are asserted as a set, because the defect was that two
    /// of them shared one sentence: `field_widget_moved` keyed on `resized`
    /// alone, so a check box whose appearance was stretched rather than
    /// rebuilt was told *"its contents were redrawn to fit"* — the opposite of
    /// what the outcome it was reading said.
    ///
    /// The assertions are on the CLAIM rather than on the wording, because the
    /// wording will change and the claim must not: the unsatisfiable case must
    /// say the contents are stretched, and must not say they were redrawn.
    #[test]
    fn a_resize_that_could_not_redraw_says_so() {
        let stretched = field_widget_moved(true, false);
        let redrawn = field_widget_moved(true, true);
        let moved = field_widget_moved(false, false);

        assert!(
            stretched.contains("stretched"),
            "an appearance that could not be rebuilt must say it is stretched: {stretched}"
        );
        assert!(
            !stretched.contains("redrawn to fit"),
            "★ the defect, stated: this case claimed a redraw that did not happen: {stretched}"
        );
        assert!(
            redrawn.contains("redrawn"),
            "a rebuilt appearance must say so: {redrawn}"
        );
        assert_ne!(
            stretched, redrawn,
            "the two resize outcomes must not share a sentence — sharing one is what shipped"
        );
        assert!(
            !moved.contains("resized") && !moved.contains("stretched"),
            "a move changes no length and owes no disclosure about one: {moved}"
        );
    }

    /// A move says the same thing whether or not the appearance regenerated.
    ///
    /// Deliberate, not an omission: a translation changes no length, so an
    /// appearance carried across a move is exact and there is nothing to
    /// disclose. Only a resize can be unsatisfiable.
    #[test]
    fn a_move_owes_no_disclosure_either_way() {
        assert_eq!(
            field_widget_moved(false, true),
            field_widget_moved(false, false)
        );
    }

    /// **Every noun is distinct**, because the confirmation line is the only
    /// place the operator learns which of five buttons they actually pressed.
    #[test]
    fn the_five_nouns_are_distinct() {
        let nouns = [
            form_noun_text(),
            form_noun_check_box(),
            form_noun_radio(),
            form_noun_choice(),
            form_noun_push_button(),
        ];
        for (i, a) in nouns.iter().enumerate() {
            for b in nouns.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
    }

    /// ★★ **The merge disclosure explains the consequence, not the mechanism.**
    ///
    /// Asserted rather than left to review because the tempting rewrite — "the
    /// field was merged with an existing one" — is shorter, is what the engine
    /// calls it, and tells an operator nothing about what will happen when they
    /// type. This test fails if the sentence stops saying that both change.
    #[test]
    fn the_merge_disclosure_says_what_it_means_for_the_operator() {
        let line = form_field_merged();
        assert!(
            line.contains("changes both"),
            "the consequence must be stated, not just the fact of merging: {line}"
        );
        assert!(
            !line.contains("merged"),
            "\u{201c}merged\u{201d} is the engine's word for the mechanism: {line}"
        );
    }

    /// **Every disclosure is one sentence an operator could act on or ignore**
    /// — none of them is empty, and none runs past a status line's width.
    #[test]
    fn the_disclosures_are_stated_and_bounded() {
        for line in [
            form_field_merged(),
            form_field_no_tooltip(),
            form_field_no_options(),
            form_field_tagged_document(),
        ] {
            assert!(!line.trim().is_empty());
            assert!(line.len() < 240, "too long for a status line: {line}");
        }
    }
}
