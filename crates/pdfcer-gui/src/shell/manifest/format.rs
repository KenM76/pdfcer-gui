//! The **Format** tab — contextual, appearing only while something is
//! selected.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/shell/manifest/format.md`.

use super::{command, group};
use crate::text::ribbon;
use egui_shell::manifest::{Item, Tab};

/// The condition, published by the application each frame, under which the
/// Format tab appears.
pub(super) const VISIBLE_WHEN: &str = "selection.formattable"; // ui-text-exempt: a condition name, never displayed

/// The condition under which a mode may change page content, and therefore
/// under which the Font group is drawn at all.
const FONT_VISIBLE_WHEN: &str = "mode.edit_content"; // ui-text-exempt: a condition name, never displayed

/// The condition under which the **Markup** group is drawn at all: a markup
/// annotation is selected, and this mode may author markup.
const MARKUP_VISIBLE_WHEN: &str = "selection.markup_restylable"; // ui-text-exempt: a condition name, never displayed

/// The Format tab.
pub(super) fn tab() -> Tab {
    Tab::new("format", ribbon::tab_format())
        .with_question(ribbon::question_format())
        .with_visible_when(VISIBLE_WHEN)
        .with_groups([
            //
            // FIRST, ahead of Selection, and the order is the operator's
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
            // # What is in it is exactly what the PANEL has, and no more
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
            // # Every item carries `visible_when`, and the SEPARATOR does not
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
                    // The rule separates *which typeface* from *how it is
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
            // ---------------------------------------------------------------
            // Markup — §5.8's "Markup annotation" row, built 2026-09-06 on the
            // operator's *"getting full editing working for the Markup tools."*
            //
            // SECOND, between Font and Selection, and the position is
            // decided by the same rule that put Font first. §5.8's tables read
            // *change how this looks · describe it · destroy it*, left to
            // right, in increasing commitment. Font and Markup are both the
            // first of those — one for a swept text range, one for a placed
            // mark — and Selection is the second and third. Putting Markup
            // after Selection would put a Delete between two bands of
            // appearance controls.
            //
            // Font and Markup are never drawn together. Their conditions are
            // disjoint by construction: `mode.edit_content` needs Edit and a
            // swept text range, `selection.markup_restylable` needs an
            // annotation selected — and `SelectionState` cannot hold an
            // annotation and a content selection at once, while a text sweep is
            // a third space again. So the order between them is a decision
            // about the *file*, not about what an operator sees; it is recorded
            // anyway, because the next group added here will not be disjoint
            // from both.
            //
            // # Five items, one condition, and no separator
            //
            // Every item carries `MARKUP_VISIBLE_WHEN`, so the group is drawn
            // whole or not at all and `egui-shell`'s
            // `a_group_with_nothing_left_is_not_drawn` removes the caption with
            // it. There is no separator, unlike the Font group: that one splits
            // *which typeface* from *how it is set*, which is a real seam an
            // operator reads. These five are five properties of one mark with
            // no such seam — and the two that would have been candidates for a
            // divider (the pair of colours) are already adjacent and already
            // read as a pair from their labels.
            //
            // # The arrowhead chooser has NO extra condition, and that is the
            // application's decision rather than the manifest's
            //
            // `/LE` is meaningful for `/Line` alone. That could have been a
            // second published condition, and it is not: the item is drawn for
            // every markup and `app::markupband` **draws nothing** for a
            // subtype with no ends, which costs no space (the shell reserves
            // the budgeted width only when the application supplies no renderer
            // at all — see `egui_shell::ribbon::control`). It is the same shape
            // `panels::properties::markup::width_row` already uses for a
            // highlight, which has no border to widen: a control that decides
            // its own absence from the value it reads, in the one place that
            // has read it. A condition would put the same question in two
            // places and let them disagree.
            // ---------------------------------------------------------------
            group(
                "markup",
                ribbon::group_format_markup(),
                [
                    Item::custom(super::MARKUP_STROKE).shown_when(MARKUP_VISIBLE_WHEN),
                    Item::custom(super::MARKUP_FILL).shown_when(MARKUP_VISIBLE_WHEN),
                    Item::custom(super::MARKUP_WIDTH).shown_when(MARKUP_VISIBLE_WHEN),
                    // Beside the width and not at the end of the row: the two
                    // are one subject — *what the line looks like* — and §5.8's
                    // Markup row lists them adjacent for that reason. An
                    // operator setting a mark's linework should not have to
                    // cross an opacity field to finish the thought.
                    Item::custom(super::MARKUP_DASH).shown_when(MARKUP_VISIBLE_WHEN),
                    Item::custom(super::MARKUP_OPACITY).shown_when(MARKUP_VISIBLE_WHEN),
                    Item::custom(super::MARKUP_ENDINGS).shown_when(MARKUP_VISIBLE_WHEN),
                ],
            ),
            group(
                "selection",
                ribbon::group_format_selection(),
                [
                    command("format.properties"),
                    //
                    // R9: withheld, not greyed. A mode is not a temporary
                    // condition that will pass while the operator hovers; it is
                    // a standing choice they made in the mode selector, and the
                    // mode selector is the disclosure.
                    //
                    // Why `select_form` counts as authoring when all it does
                    // is move the selection: it is the first half of the
                    // compound `dispatch::format` records — in Read, click a
                    // picture inside a title block, `select_form` re-aims the
                    // selection from the one image to the whole form XObject,
                    // and Delete then takes the lot. The re-aim is only ever
                    // wanted as a prelude to editing.
                    command("format.select_form").shown_when(FONT_VISIBLE_WHEN),
                    // Between "select the form" and Delete, and the ordering
                    // rule two comments up decides it without needing a new
                    // one: §5.8's menu rule is least-destructive-first, and the
                    // same reading orders a group — **describe, then re-aim,
                    // then detach, then destroy**. Giving a page its own copy
                    // adds an object and changes nothing an operator can see;
                    // it is strictly less committing than a delete and strictly
                    // more than a re-aim, so it lands exactly here and the eye
                    // still meets Delete last.
                    //
                    // It is also the order the two form commands are USED in.
                    // "Select the form" answers *what am I looking at*, and
                    // this answers *make it mine before I change it*. A reader
                    // scanning the group top to bottom reads the workflow.
                    // Gated for `select_form`'s reason and a blunter one of
                    // its own: this **writes to the document**
                    // (`EditSession::unshare_form` gives a page its own copy of
                    // a shared form), so an enabled control in a mode that
                    // authors nothing is a promise the dispatch arm must then
                    // break in silence.
                    command("format.unshare_form").shown_when(FONT_VISIBLE_WHEN),
                    // Rewrites a content stream, so withheld where content is
                    // not authored; greyed on the engine's preflight otherwise.
                    command("format.merge_text_runs").shown_when(FONT_VISIBLE_WHEN),
                    // **Withheld, not greyed, where the engine would refuse
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
                    // The condition is TRUE for every state but the one narrow
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
    const WRITERS: &[&str] = &[
        "format.font",
        "format.font_size",
        "format.font_colour",
        "format.bold",
        "format.italic",
        "format.select_form",
        "format.unshare_form",
        "format.merge_text_runs",
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

    /// A18, second half. The dispatch guard stops the ACT; this stops the
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

    /// **Every CUSTOM item on this tab is withheld too, and neither test
    /// above can see one.**
    #[test]
    fn every_custom_control_on_this_tab_is_withheld_from_a_mode_that_cannot_author() {
        let mut seen = 0_usize;
        for item in tab().groups().iter().flat_map(|g| g.items().iter()) {
            let Item::Custom {
                kind, visible_when, ..
            } = item
            else {
                continue;
            };
            seen += 1;
            assert!(
                matches!(
                    visible_when.as_deref(),
                    Some(FONT_VISIBLE_WHEN | MARKUP_VISIBLE_WHEN)
                ),
                "the `{kind}` control edits the document and is shown in every mode. R9: an \
                 unavailable capability renders NOTHING. Gate it on `mode.edit_content` if it \
                 changes page content, or on `selection.markup_restylable` if it restyles a \
                 mark."
            );
        }
        assert_eq!(
            seen, 9,
            "three Font controls and six Markup controls. A count, because the loop above \
             passes trivially over an empty tab — which is what a renamed variant or a group \
             lost to an editing accident would leave it"
        );
    }
}
