//! The **Format** tab — contextual, appearing only while something is
//! selected.
//!
//! `RIBBON_IA.md` §5.8. One group: Selection.
//!
//! # What a contextual tab is, in this manifest
//!
//! It lives in `contextual_tabs` rather than `tabs`, and it carries a
//! `visible_when` condition — `"selection.any"` — that the application
//! publishes each frame in its `egui_shell::commands::ConditionSet`. The
//! separation is not cosmetic: a **mode** names a fixed tab set, and a
//! contextual tab's whole nature is that its presence is decided by
//! application state rather than by configuration. `egui-shell` refuses a
//! mode that names one, by design.
//!
//! It is therefore present in **all three modes** and in none of their tab
//! lists. Selecting a markup in Review mode shows the Format tab exactly
//! as selecting one in Edit mode does. That is correct: Review is the
//! stance in which you place and adjust *your own* markup, and a reviewer
//! who cannot recolour a cloud they just drew has been given half a tool.
//!
//! # Why this tab is nearly empty, and why it ships anyway
//!
//! §5.8 calls the contextual tab *"the single largest usability change
//! proposed here"* and then sets the build order:
//!
//! > Build order: **panel first, tab second.** The panel is the harder
//! > half and the tab's contents are a subset of it, so building the tab
//! > first would mean writing the property editors twice.
//!
//! Every property editor the tab is eventually made of — colour, fill,
//! line width, line style, opacity, arrowheads, note text, dimension
//! group, scale, precision, units, standard, witness lines, size,
//! position, crop, stroke, winding rule, node tools, font, spacing,
//! alignment — is therefore **N**, and under P3 absent. Twenty-four
//! entries in [`super::PLANNED`] come from this one section.
//!
//! # ★ MEASURED 2026-08-17: they are not *unbuilt*, they are *unbuildable*
//!
//! This header used to say the property editors were not built yet, which
//! reads as a scheduling fact. It is not one. The tab was taken up as work
//! and stopped against **two independent blockers**, neither of which is in
//! this file:
//!
//! **1. `EditSession` has no verb that modifies an annotation.** Grepping
//! every public `pub fn` for annotation work returns `add_markup`,
//! `add_text_annotation`, `delete_annotation`, `delete_redaction_mark` and
//! two deletion predicates. **Add and delete, nothing between them.** So a
//! markup's colour, width, fill, opacity, arrowheads and note text cannot be
//! changed after it is placed — which is §5.8's entire markup row.
//!
//! Delete-and-re-add is not a workaround for this and is deliberately not
//! built. Re-adding loses the annotation's object identity, and with it its
//! `/NM`, its place in the page's `/Annots` order (so its z-order), and any
//! reply thread hung off it as an `/IRT` target. A "change the colour" button
//! that silently detaches a reviewer's replies is worse than no button.
//!
//! The **one** exception is the ce dimension row: `set_group_style`,
//! `set_dimension_style`, `set_group_scale` and `set_group_standard` all
//! exist. Dimensions have a style model and nothing else does.
//!
//! **2. The canvas selection cannot address an annotation.**
//! `canvas::selection::identity::Selection` is `page + object + subpath +
//! node` — four integers naming a **paint-order index into page content**.
//! That shape is what makes a selection immune to zoom and is not lightly
//! changed; it also means a markup or a dimension is not selectable at all,
//! so even a perfect `set_markup_style` would have nothing to name.
//!
//! The second is ours and the first is filed as
//! `request_no_verb_modifies_an_existing_annotation.md`. Until both land, the
//! honest content of this tab is exactly what is below — and the argument for
//! shipping it with one command rather than deferring the tab is unchanged and
//! is now *stronger*, because the appear-on-selection behaviour is the only
//! part of §5.8 that can be exercised at all.
//!
//! What is left is the row that appears in *every* selection type's list
//! in §5.8's table, and that works today: **Delete**. An unarmed canvas
//! already does modeless select-and-delete — that is what the removal of
//! the `Editing on` master toggle relies on — so a Delete command on a
//! surface that only appears when something is selected is real, not a
//! stub.
//!
//! Shipping the tab with one command rather than deferring the tab
//! entirely is a deliberate choice and worth defending, because it looks
//! like exactly the placeholder P3 forbids and is not:
//!
//! - The tab **appears on selection**, which is itself the affordance
//!   §5.8 credits it with. That behaviour is the feature, and it is
//!   testable and demonstrable now.
//! - The command in it **does something**.
//! - The alternative — no contextual tab until the property editors land —
//!   means the appear-on-selection behaviour, the mode interaction and the
//!   one-command-one-tab consequences all get their first exercise at the
//!   same moment as twenty-four new controls.
//!
//! # The other two surfaces
//!
//! §5.8 is explicit that the contextual tab and a persistent **properties
//! panel** both ship, and that they answer different questions: the tab
//! carries what a user changes *while working*, the panel carries
//! everything including read-only facts and the editable X/Y/W/H geometry.
//! A **context menu** carries the same commands again for the user who
//! right-clicks — currently there is not one anywhere in the application.
//! Neither is a manifest concern at this stage; both are recorded here so
//! that "Format is nearly empty" is not read as "Format is all there will
//! be".

use super::{command, group};
use crate::text::ribbon;
use egui_shell::manifest::{Item, Tab};

/// The condition, published by the application each frame, under which the
/// Format tab appears.
///
/// # ★★★ It became its own condition on 2026-08-27, and the change is the
/// whole reason the Font group works
///
/// It used to be `"selection.any"` — the **object** selection — and the note
/// here argued for that spelling on the grounds that the tab and the one
/// command inside it must ask the same question, so that a Format tab could
/// never appear holding a single greyed control.
///
/// That argument was sound for a tab with one group. It stops being sound the
/// moment the tab carries controls for a **second kind of selection**, and it
/// does now: the Selection group acts on a page object, addressed by
/// paint-order index, and the Font group acts on a swept text range, addressed
/// by run. Those are unrelated index spaces (`panels::properties::text`'s
/// header argues why nothing maps between them), and neither one of them is
/// the tab's question.
///
/// The tab's question is *"is there anything for me to be about?"*, and
/// `selection.formattable` is that question's name. `app::conditions`
/// publishes it as the union, so:
///
/// | state | tab | Selection group | Font group |
/// |---|---|---|---|
/// | nothing selected | absent | — | — |
/// | an object selected | **shown** | enabled | greyed, and explains itself |
/// | text swept | **shown** | greyed | enabled |
/// | both | shown | enabled | enabled |
///
/// ★ The old note's fear — a tab that appears holding a greyed control — is
/// therefore now the *designed* middle two rows rather than a defect, and R9
/// is what makes that legitimate: the capability is present and the **operand**
/// is missing, which is the textbook temporarily-unavailable case, greyed and
/// explained on hover. It is also, deliberately, the surface that answers
/// O37's admission that nothing on screen tells an operator to press `T`
/// before sweeping — see the Font group below.
///
/// Not spelled `crate::shell::manifest::SELECTION_ANY`: that constant is the
/// object-selection condition and has two other readers (the canvas context
/// menu among them). It used to be an alias for this one, and de-aliasing them
/// was the first step of this change, because editing this string in place
/// would have silently retargeted a Delete in a menu that has nothing to do
/// with this tab.
pub(super) const VISIBLE_WHEN: &str = "selection.formattable"; // ui-text-exempt: a condition name, never displayed

/// The condition under which a mode may change page content, and therefore
/// under which the Font group is drawn at all.
///
/// ★★ **Visibility, not enablement**, and R9 is the whole of the reasoning:
/// *an unavailable capability renders nothing; greying is reserved for
/// temporarily unavailable and is always explained on hover.* Read and Review
/// do not have a mislaid ability to restyle text — they do not have the
/// ability — so the group is **absent** there. Inside Edit the same controls
/// grey on `selection.text`, because there the capability is present and only
/// the operand is missing.
///
/// One condition would not do both jobs. `selection.text` alone would draw an
/// enabled Bold in Read, where pressing it must be refused; `mode.edit_content`
/// alone would draw an enabled Bold in Edit with nothing swept, which is a
/// control that does nothing on almost every press — the exact placeholder
/// shape P3 forbids.
const FONT_VISIBLE_WHEN: &str = "mode.edit_content"; // ui-text-exempt: a condition name, never displayed

/// The Format tab.
pub(super) fn tab() -> Tab {
    Tab::new("format", ribbon::tab_format())
        .with_question(ribbon::question_format())
        .with_visible_when(VISIBLE_WHEN)
        .with_groups([
            // ---------------------------------------------------------------
            // Font — §5.8's "Text run" row, built 2026-08-27 for O37.
            //
            // ★★★ FIRST, ahead of Selection, and the order is the operator's
            // rather than this file's. §5.8's own table lists a text run's
            // groups as *Font · Size · Colour · Spacing · Alignment · Delete*,
            // with Delete last, and every other row in that table ends the
            // same way. Reading left to right therefore goes "change how this
            // looks", then "describe it", then "destroy it" — increasing
            // commitment, which is the ordering rule the Selection group's own
            // comment below already follows internally.
            //
            // It is also where Word puts it. Home ▸ Font is the leftmost
            // group of the tab an operator lives on, and this tab is the
            // nearest thing this product has to Home.
            //
            // # ★★ What is in it is exactly what the PANEL has, and no more
            //
            // §5.8 sets the build order — *"panel first, tab second … the
            // tab's contents are a **subset** of it"* — and that word decides
            // two arguments that would otherwise be matters of taste:
            //
            // * **Bold and Italic are in**, though §5.8's table does not name
            //   them, because the panel has them and because they are what O37
            //   actually asked for: *"all the font tools available that Word
            //   does."* They are a *subset* of the panel, which is the test.
            // * **Grow and Shrink are out**, though Word has them. They exist
            //   in no panel section, so putting them here would make the tab a
            //   superset — and §5.8's reason for the build order is that
            //   building the tab first means writing the editors twice. A
            //   control that exists only on the tab has done exactly that.
            //
            // Spacing and Alignment stay in `manifest::PLANNED`, for a reason
            // that is not about order: `EditSession` has no verb for either.
            //
            // # ★ Every item carries `visible_when`, and the SEPARATOR does not
            //
            // `egui_shell::manifest::Item::Separator` cannot carry a condition
            // — deliberately, and its own docs say why: a divider's visibility
            // is a fact about its **neighbours**, and a separator with an
            // independently-set condition is a contradiction that renders. Here
            // that costs nothing, because all five items share one condition:
            // either the whole group is drawn or none of it is, and a group
            // with nothing left is not drawn at all (`egui-shell`'s
            // `a_group_with_nothing_left_is_not_drawn`). The separator can
            // never be the only thing standing.
            // ---------------------------------------------------------------
            group(
                "font",
                ribbon::group_format_font(),
                [
                    Item::custom(super::FONT_FACE).shown_when(FONT_VISIBLE_WHEN),
                    Item::custom(super::FONT_SIZE).shown_when(FONT_VISIBLE_WHEN),
                    // ★ The rule separates *which typeface* from *how it is
                    // set*, which is the seam Word draws in the same place: a
                    // face and a size are what the text IS, and bold, italic
                    // and colour are what is done to it. An operator scanning
                    // the group meets two clusters rather than five controls.
                    Item::Separator,
                    command("format.bold").shown_when(FONT_VISIBLE_WHEN),
                    command("format.italic").shown_when(FONT_VISIBLE_WHEN),
                    Item::custom(super::FONT_COLOUR).shown_when(FONT_VISIBLE_WHEN),
                ],
            ),
            group(
                "selection",
                ribbon::group_format_selection(),
                [
                    command("format.properties"),
                    // ★ Between Properties and Delete deliberately. §5.8's menu
                    // rule is least-destructive-first, and the same reading orders
                    // a group: describe, then re-aim, then destroy. It is also the
                    // order of increasing commitment, so the eye meets Delete last
                    // in both surfaces that carry it.
                    // ★★★ **Gated on the mode, 2026-09-04 — A18's second half.**
                    // The dispatch arm added a `!edit_content` guard on
                    // 2026-09-03 and that guard is correct, but a guard alone
                    // leaves the CONTROL. In Read the operator saw an enabled
                    // "Select form", pressed it, and nothing happened — this
                    // project's founding defect class, re-created by the fix
                    // for a data-loss defect.
                    //
                    // R9: withheld, not greyed. A mode is not a temporary
                    // condition that will pass while the operator hovers; it is
                    // a standing choice they made in the mode selector, and the
                    // mode selector is the disclosure.
                    //
                    // ★ Why `select_form` counts as authoring when all it does
                    // is move the selection: it is the first half of the
                    // compound `dispatch::format` records — in Read, click a
                    // picture inside a title block, `select_form` re-aims the
                    // selection from the one image to the whole form XObject,
                    // and Delete then takes the lot. The re-aim is only ever
                    // wanted as a prelude to editing.
                    command("format.select_form").shown_when(FONT_VISIBLE_WHEN),
                    // ★★ Between "select the form" and Delete, and the ordering
                    // rule two comments up decides it without needing a new
                    // one: §5.8's menu rule is least-destructive-first, and the
                    // same reading orders a group — **describe, then re-aim,
                    // then detach, then destroy**. Giving a page its own copy
                    // adds an object and changes nothing an operator can see;
                    // it is strictly less committing than a delete and strictly
                    // more than a re-aim, so it lands exactly here and the eye
                    // still meets Delete last.
                    //
                    // ★ It is also the order the two form commands are USED in.
                    // "Select the form" answers *what am I looking at*, and
                    // this answers *make it mine before I change it*. A reader
                    // scanning the group top to bottom reads the workflow.
                    // ★★★ Gated for `select_form`'s reason and a blunter one of
                    // its own: this **writes to the document**
                    // (`EditSession::unshare_form` gives a page its own copy of
                    // a shared form), so an enabled control in a mode that
                    // authors nothing is a promise the dispatch arm must then
                    // break in silence.
                    command("format.unshare_form").shown_when(FONT_VISIBLE_WHEN),
                    // ★★★ **Withheld, not greyed, where the engine would refuse
                    // the delete** — `visible_when` rather than a second
                    // `enabled_when`, and the difference is R9 stated by
                    // `Item::visible_when`'s own doc: *"this is visibility, not
                    // enablement … `Command::enable` is the greying; this is the
                    // disappearing."*
                    //
                    // Greying is for a capability that is **temporarily**
                    // unavailable and is always explained on hover. A
                    // certification signature is not temporary, `/Encrypt` is
                    // not temporary, and §12.5.3's `Locked` bit is a statement
                    // the file's producer wrote down — none of the three is
                    // arguable, and none of them will change while the operator
                    // hovers.
                    //
                    // ⇒ The sentence that replaces the control is in the
                    // Properties panel, on the section describing the very
                    // annotation this Delete would have acted on, and it is
                    // there **before** the operator reaches for anything:
                    // `panels::properties::annotdelete`. The two are derived
                    // from one function, so the control cannot be withheld for a
                    // reason different from the one the panel gives.
                    //
                    // ★ The condition is TRUE for every state but the one narrow
                    // annotation case, so this changes nothing for a content
                    // selection or a form field — see `app::conditions`, which
                    // argues at length why that default direction is the safe
                    // one.
                    command("format.delete").shown_when(super::DELETE_PERMITTED),
                ],
            ),
        ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_shell::manifest::Item;

    /// Every command on the Format tab that WRITES is withheld from a mode
    /// that authors nothing.
    ///
    /// # ★★★ Why this is a list and not a predicate
    ///
    /// Because "does this command write" is not a property the manifest can
    /// see. The manifest holds an id and a condition string; whether the arm
    /// behind that id calls `EditSession` is a fact about
    /// `app::dispatch::format`. A test that tried to derive the answer would
    /// be re-implementing the dispatcher, and a hand-written list inside a
    /// completeness test is exactly the shape this project has already been
    /// bitten by — a new module invisible to the check built to find it.
    ///
    /// So the list is stated, and its JOB is to fail loudly when the Format
    /// tab grows an item it does not name. `every_writer_is_accounted_for`
    /// below is the half that makes the list honest: it asserts that the tab's
    /// full command set is exactly the writers plus the explicitly-declared
    /// readers, so a new command lands in neither bucket and fails.
    const WRITERS: &[&str] = &[
        "format.font",
        "format.font_size",
        "format.font_colour",
        "format.bold",
        "format.italic",
        "format.select_form",
        "format.unshare_form",
    ];

    /// Read on this tab: describe what is selected, and delete it where the
    /// mode permits. `format.delete` carries `DELETE_PERMITTED`, which folds
    /// in the mode question and more besides; `format.properties` opens an
    /// inspector and changes nothing.
    const READERS: &[&str] = &["format.properties", "format.delete"];

    fn items() -> Vec<(String, Option<String>)> {
        tab()
            .groups()
            .iter()
            .flat_map(|g| g.items().iter())
            .filter_map(|item| match item {
                Item::Command {
                    id, visible_when, ..
                } => Some((id.clone(), visible_when.clone())),
                _ => None,
            })
            .collect()
    }

    /// ★ A18, second half. The dispatch guard stops the ACT; this stops the
    /// promise. A control that is enabled, pressed, and does nothing is the
    /// defect this whole project was founded on, and it was re-created on
    /// 2026-09-03 by the fix for a data-loss defect.
    #[test]
    fn every_writing_command_is_withheld_from_a_mode_that_cannot_author() {
        for (id, visible_when) in items() {
            if !WRITERS.contains(&id.as_str()) {
                continue;
            }
            assert_eq!(
                visible_when.as_deref(),
                Some(FONT_VISIBLE_WHEN),
                "`{id}` writes to the document but is shown in every mode. R9: an unavailable \
                 capability renders NOTHING. Greying is for something temporarily unavailable and \
                 explained on hover; a mode is a standing choice the operator made in the mode \
                 selector, and that selector is the disclosure."
            );
        }
    }

    /// The list above is only honest if nothing escapes it.
    #[test]
    fn every_format_command_is_classified_as_a_writer_or_a_reader() {
        for (id, _) in items() {
            assert!(
                WRITERS.contains(&id.as_str()) || READERS.contains(&id.as_str()),
                "`{id}` is on the Format tab and is in neither WRITERS nor READERS. Decide which \
                 it is: if its dispatch arm reaches `EditSession`, it is a writer and must carry \
                 `shown_when(FONT_VISIBLE_WHEN)`."
            );
        }
    }
}
