//! # `text::forms::authoring` — the words for **making** a form field
//!
//!
//! Its own file rather than more of `super`, for the reason `super`'s header
//! gives about itself: *"the reviewer of a disclosure sentence is reading a file
//! that contains nothing but disclosure sentences"*. It also keeps both files
//! comfortably inside R2 — `super` is already at 1,265 lines.
//!
//! ## The four disclosures, and why they are status lines
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
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/forms/authoring.md`.

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
#[must_use]
pub fn form_field_added(noun: &str) -> String {
    format!("{noun} added.")
}

/// **The name matched an existing field, so this widget joined it.**
#[must_use]
pub fn form_field_merged() -> String {
    "That name already existed, so this control shows the same value as the \
     other one — typing in either changes both. Give it a different name if you \
     wanted two separate fields."
        .to_owned()
}

/// **No tooltip was given**, and what that costs.
#[must_use]
pub fn form_field_no_tooltip() -> String {
    "It has no tooltip, so a screen reader will announce only what kind of \
     control it is."
        .to_owned()
}

/// **A drop-down with no options in it.**
#[must_use]
pub fn form_field_no_options() -> String {
    "It has no options yet, so it will open empty.".to_owned()
}

/// **The document is tagged, and this control is not in the tag tree.**
#[must_use]
pub fn form_field_tagged_document() -> String {
    "This document is tagged for accessibility, and the new control is not in \
     its structure tree — its reading order will not include this field."
        .to_owned()
}

/// **The field was renamed.**
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

/// Rule-4 disclosure: **pdfcer rewrote other people's buttons** so they keep
/// pointing at the field the operator just renamed.
#[must_use]
pub fn form_field_actions_retargeted(count: usize) -> String {
    format!(
        "pdfcer also updated {count} place(s) where a button referred to this field by its old \
         name, so those buttons still work. Any JavaScript in the document was not changed."
    )
}

/// Rule-4 disclosure: **buttons elsewhere now name a field that is gone**, and
/// pdfcer did not repair them.
#[must_use]
pub fn form_field_actions_orphaned(count: usize) -> String {
    format!(
        "⚠ {count} button reference(s) elsewhere in this document still name this field, and \
         pdfcer cannot repair them — those buttons will now do less than they say when pressed. \
         Any JavaScript naming it was not counted."
    )
}

/// **The field was deleted**, and how many boxes went with it.
///
/// The count is the part that cannot be seen. A field drawn in three places
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

/// **The last box went, so the field went with it** — which is not what the
/// operator pressed.
#[must_use]
pub fn form_widget_deleted_last() -> String {
    "That was the field's last box, so the field was removed from the form too.".to_owned()
}

/// **The `Sort` flag was set over a list nobody has sorted.**
#[must_use]
pub const fn field_sort_claim_unmet() -> &'static str {
    "The Sort flag now says this list was sorted by whoever wrote the file, and it is not in \
     order. pdfcer has not reordered it — the order options appear in is what a reader shows."
}

/// **pdfcer put the list in a different order from the one it was sent.**
#[must_use]
pub const fn field_options_reordered() -> &'static str {
    "pdfcer put the options in a different order from the one shown when you pressed. The list in the file is sorted; what you saw was not."
}

/// **One field's flag changed and several boxes on the page followed.**
#[must_use]
pub fn field_widgets_affected(widgets: usize) -> String {
    format!(
        "This field is drawn in {widgets} places, and all {widgets} changed — including any \
         on other pages."
    )
}

/// **The engine recorded the edit and could not repaint the box.**
#[must_use]
pub fn field_appearance_not_repainted(resized: bool, why: &str) -> String {
    if resized {
        format!(
            "This box was resized and its artwork could not be redrawn, so it will look \
             stretched: {why}"
        )
    } else {
        format!(
            "This was stored in the file, but the box could not be redrawn, so what you see has \
             not changed: {why}"
        )
    }
}

/// **Another program's check box or radio artwork was replaced by pdfcer's.**
#[must_use]
pub const fn field_foreign_appearance_replaced() -> &'static str {
    "This box was drawn by another program, so pdfcer redrew it in its own style. The look it \
     had while being pressed or hovered over is gone."
}

/// **A widget was moved or resized.**
#[must_use]
pub fn field_widget_moved(resized: bool, regenerated: bool) -> &'static str {
    match (resized, regenerated) {
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
        // That is precisely the case `resize_annotation` REFUSES by name
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
        // A move that regenerated is not distinguished from one that did
        // not, and that is correct rather than an omission: a translation
        // changes no length, so an appearance carried across is exact and
        // there is nothing to disclose. Only a RESIZE can be unsatisfiable.
        (false, _) => "The box was moved.",
    }
}

/// **A widget property other than its geometry changed.**
#[must_use]
pub fn field_widget_property_changed(touched: &str, regenerated: bool) -> String {
    if regenerated {
        format!("Changed {touched}. The box was redrawn to match.")
    } else {
        format!("Changed {touched}.")
    }
}

/// **The other placements of this field were left where they are.**
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

    /// **A resize that redrew nothing does not claim it redrew something**
    /// — `OPERATOR_REQUESTS.md` O76.
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

    /// **A non-geometry edit does not report a move it did not make.**
    #[test]
    fn a_property_change_names_what_was_touched_and_not_a_move() {
        let line = field_widget_property_changed("the background colour", true);
        assert!(
            line.contains("the background colour"),
            "the receipt must name the control that was pressed: {line}"
        );
        assert!(
            !line.contains("moved"),
            "★ the defect, stated: a colour edit claimed a move: {line}"
        );
        assert_ne!(
            field_widget_property_changed("the border", true),
            field_widget_property_changed("the border", false),
            "whether the box was redrawn is the one fact this line adds"
        );
    }

    /// **The stretched-artwork sentence belongs to a RESIZE only.**
    #[test]
    fn only_a_resize_claims_the_artwork_is_stretched() {
        let why = "the caption font is not embedded";
        let resized = field_appearance_not_repainted(true, why);
        let colour_only = field_appearance_not_repainted(false, why);

        assert!(resized.contains("stretched"), "{resized}");
        assert!(
            !colour_only.contains("stretched"),
            "★ the defect, stated: a colour edit was told its artwork is stretched: {colour_only}"
        );
        for line in [&resized, &colour_only] {
            assert!(
                line.contains(why),
                "the engine's own reason is carried verbatim: {line}"
            );
        }
    }

    /// A move says the same thing whether or not the appearance regenerated.
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

    /// **The merge disclosure explains the consequence, not the mechanism.**
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

    /// ⚠ **THESE THREE ARE STRING TESTS. What they cover, and what covers the
    /// rest — stated rather than implied.**
    #[test]
    fn a_rename_and_a_delete_say_different_things_about_other_peoples_buttons() {
        let repaired = form_field_actions_retargeted(3);
        let broken = form_field_actions_orphaned(3);

        // The load-bearing distinction, and it is not cosmetic. A rename REPAIRS
        // the actions; a delete cannot. A build that worded them alike would tell
        // the operator his form still works when it does not.
        assert!(
            repaired.contains("still work"),
            "a rename repoints the actions, so the sentence must say the buttons survive: {repaired}"
        );
        assert!(
            broken.contains("do less than they say"),
            "a delete leaves them naming nothing, and the operator's document is now degraded — the \
             sentence must name that consequence rather than count internals: {broken}"
        );
        assert!(
            !repaired.contains("do less than"),
            "the rename case must not borrow the delete case's alarm: {repaired}"
        );
        assert!(
            !broken.contains("still work"),
            "the delete case must not borrow the rename case's reassurance: {broken}"
        );
    }

    /// **Both name JavaScript as un-handled**, because `R55` forbids rewriting
    /// a script carrier and the count is therefore a floor on any scripted form.
    #[test]
    fn both_sentences_admit_that_javascript_was_not_handled() {
        for line in [
            form_field_actions_retargeted(1),
            form_field_actions_orphaned(1),
        ] {
            assert!(
                line.contains("JavaScript"),
                "R55 means pdfcer never rewrites a script, so a sentence about repairing references \
                 that does not say so overstates what happened: {line}"
            );
        }
    }

    /// **The rename count is ACTIONS, not buttons**, and the wording must not
    /// promise the distinction pdfcer does not draw.
    #[test]
    fn the_retarget_count_is_worded_as_places_not_buttons() {
        let line = form_field_actions_retargeted(3);
        assert!(
            line.contains("place(s)"),
            "the count is of ACTION REFERENCES; wording it as a button count claims a distinction \
             the engine explicitly does not draw: {line}"
        );
        assert!(
            !line.contains("3 button"),
            "…and specifically must not read as a count of buttons: {line}"
        );
    }
}
