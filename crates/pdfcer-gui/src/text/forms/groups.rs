//! # `text::forms::groups` — every word the Field-groups section says
//!
//! The copy for [`crate::panels::forms::groups`], which is the shell's route to
//! `EditSession::delete_field_group` and its companion
//! `EditSession::field_group_deletion_preview`.
//!
//! ## Why this is its own file rather than more lines in [`super`]
//!
//! **R2.** `text/forms/mod.rs` stood at 1,294 of the 1,500-line budget, and the
//! seam here is the same one [`super::authoring`] and [`super::tab_order`] were
//! cut along: a *surface*, not a size. Every sentence in this file is about one
//! question — *what is a field group, and what goes when you delete one* — and
//! a reviewer of that wording should be reading a file that contains nothing
//! else. It is re-exported from [`super`], so no call site knows the split
//! exists.
//!
//! ## ★★★ The one sentence-writing problem this surface has, which no other
//! panel in the shell has
//!
//! **A form field is invisible on a printed page, and a grouping node is
//! invisible even in the Forms panel's own field list.**
//!
//! `AcroForm::groups` is the field-name tree's *interior*: `Personal` in
//! `Personal.Address.Zip`. It has no type, no value, no widget and no
//! rectangle — §12.7.3 gives it existence only as a link in a `/Parent` chain.
//! It is drawn nowhere, on any page, in any viewer. So an operator who deletes
//! one sees:
//!
//! - the same page, pixel for pixel;
//! - a field list four rows shorter, if they happen to be looking at it;
//! - nothing else.
//!
//! ⇒ Every consequence of this verb is off-canvas, which makes rule 4's
//! *"disclosure lives off-canvas"* not a constraint here but the **entire
//! delivery mechanism**. If the sentence does not say what went, nothing does.
//!
//! That is why the numbers in this file are three and not one:
//!
//! | number | why it is not derivable from the others |
//! |---|---|
//! | **fields** | what the operator thinks of as "the things in the form" |
//! | **boxes** | one field may draw on three pages (§12.7.3.1's split field/widget shape), so this is not `fields` |
//! | **groups** | deleting `Personal` may also empty `Personal.Address`, a node the operator never named |
//!
//! and why the terminal fields are named rather than merely counted: *"4
//! fields"* is a quantity, *"`Personal.Name`, `Personal.Address.Zip`, …"* is a
//! decision. `FieldGroupDeletion::terminals`' own doc comment makes the same
//! ruling on core's side — *"by name rather than by count"*.
//!
//! ## Where each sentence is read
//!
//! | function | where | when |
//! |---|---|---|
//! | [`field_groups_heading`], [`field_groups_explainer`] | the section, above everything | always, while the form has a group |
//! | [`field_groups_refusal`] | in place of every control | the document refuses structural change (R83) |
//! | [`field_group_row`], [`field_group_delete_button`] | one row per grouping node | always |
//! | [`field_group_preview_summary`], [`field_group_preview_names`] | under the armed row | **before** the destructive press |
//! | [`field_group_deleted`] | the status bar's disclosure row | **after** it, from the engine's report |
//!
//! ## Conventions, which bind here as everywhere in [`crate::text`]
//!
//! - Sentence case; full sentences with punctuation for prose, no trailing
//!   period on a button label.
//! - **Never state a capability the build does not have.** Nothing here offers
//!   to *rename* a group or to *move* a field out of one; the shell has neither
//!   route today, and a sentence implying otherwise is a promise the program
//!   breaks.
//! - **A refusal is a sentence, never a silence.** [`field_groups_refusal`]
//!   exists because the alternative — drawing no controls and saying nothing —
//!   is indistinguishable from a feature nobody built.

use pdfcer_core::edit::EditError;

/// How many terminal names the pre-press disclosure prints before it stops
/// naming them and starts counting them.
///
/// ★ A cap rather than a scroll area, because this block sits *inside* the
/// Forms panel's own layout and above the fill list. An uncapped list on a
/// form with two hundred fields under one node would push every control below
/// it out of a container that does not scroll — the defect
/// `crate::panels::forms::tab_order` records at length and caps its own list
/// against.
///
/// Eight rather than three: the number has to be large enough that the common
/// case (a two- or three-field group, which is what `pdfcer`'s own dotted-path
/// authoring produces) is listed **completely**, because a truncated list of
/// three is a worse disclosure than an honest count.
pub const MAX_LISTED_NAMES: usize = 8;

/// The section's collapsing heading.
///
/// "Field groups", not "Field hierarchy" or "Field tree": the operator's word
/// for a name that other names hang under is *group*, and the section's own
/// explainer defines it in the next line rather than relying on the heading to.
#[must_use]
pub fn field_groups_heading() -> String {
    "Field groups".to_owned()
}

/// What a field group is, and the one fact about deleting one that an operator
/// cannot discover any other way.
///
/// ★★ Two clauses, and the second is the load-bearing one. The first defines
/// the noun; the second says the deletion **cascades**, which is the whole
/// reason this section carries a preview rather than a plain button. An
/// operator who reads "delete group" and expects one row to disappear has been
/// told, by the control's own name, something false.
///
/// ★ It also says the groups are **not drawn**, because the panel is beside a
/// canvas and everything else in the shell that can be deleted is visible on
/// it. Without that clause an operator looks at the page for the thing they are
/// about to remove, does not find it, and concludes the list is stale.
#[must_use]
pub fn field_groups_explainer() -> String {
    "A field group is a name other fields are filed under — \u{201c}Personal\u{201d} in \
     \u{201c}Personal.Address.Zip\u{201d}. Deleting one deletes every field beneath it. \
     Groups are not drawn anywhere on the page, so the only record of what a deletion took \
     is what pdfcer tells you here."
        .to_owned()
}

/// **Why no control is offered**, when the document refuses structural change.
///
/// # ★★★ R83, and why this returns a sentence rather than a `bool`
///
/// `EditSession::deletion_refusal` is a *pure query* that answers **before**
/// anything is drawn, so this surface knows the answer while it still has the
/// choice of what to render. R9 then decides what to do with it: a capability
/// that is **permanently** refused for this document renders **nothing** — no
/// greyed button, no disabled row — and says why in prose.
///
/// A greyed button would be wrong twice over. It would imply the state is
/// temporary and could be argued out of, which a certification signature
/// cannot; and R9 requires a greyed control to explain itself on hover, which
/// means the sentence has to exist anyway and is then hidden behind a gesture
/// the operator has no reason to make.
///
/// # ★★ Three arms, because `deletion_refusal` really can answer three ways
///
/// The shell's existing structural-refusal string names *certification* and
/// nothing else, which is correct for the case it was written for and silently
/// wrong for an encrypted document — a real shape, and the first of the two
/// guards `structural_form_refusal` runs. Naming the wrong cause is worse than
/// naming none: the operator goes looking for a signature that is not there.
///
/// The wildcard exists because `EditError` is `#[non_exhaustive]`. It says the
/// document refuses and does not guess at a reason, which is the honest answer
/// for a variant this build has never seen — and the engine's own words reach
/// the trace regardless.
#[must_use]
pub fn field_groups_refusal(error: &EditError) -> String {
    match error {
        EditError::DocumentEncrypted => "This document is encrypted, so its form structure \
             cannot be changed. Field groups are listed below and cannot be deleted."
            .to_owned(),
        _ if is_certification(error) => "A certification signature on this document forbids \
             changing the form's structure. Field groups are listed below and cannot be \
             deleted; values can still be filled in."
            .to_owned(),
        _ => "This document refuses structural changes to its form, so field groups are \
             listed below and cannot be deleted."
            .to_owned(),
    }
}

/// Whether a refusal came from the certification gate.
///
/// ★ A helper rather than a second match arm, because the certification refusal
/// is not one variant. `check_certification` reports the document as certified
/// by name, and which variant carries that has changed once already on the
/// engine's side; matching on the *family* through the error's own rendering
/// keeps this sentence correct across that. The rendering is never shown to the
/// operator — see [`crate::text::status::save_copy_failed`] for why a `Display`
/// impl's prose is not operator copy — it is only asked a yes/no question here.
fn is_certification(error: &EditError) -> bool {
    let rendered = error.to_string().to_ascii_lowercase();
    rendered.contains("certif")
}

/// One grouping node's row: its name, and how many fields are filed under it.
///
/// # ★★ The count comes from `AcroForm::descendants_of`, which is CORE'S walk
///
/// Not a prefix match written here. `FieldGroupDeletion::nodes`' doc comment
/// forbids a shell re-deriving core's notion of descendant, and it is right —
/// but `descendants_of` *is* core's notion, exposed for exactly this. Calling
/// it is the opposite of re-deriving it.
///
/// It is nonetheless only an **indication**, and the wording keeps it to one:
/// the row says how many fields are under the group, and says nothing about
/// boxes or about the other grouping nodes the removal would empty. Those two
/// numbers are the preview's to give, because they are the ones a walk of the
/// field list cannot answer.
#[must_use]
pub fn field_group_row(name: &str, fields: usize) -> String {
    if fields == 1 {
        format!("\u{201c}{name}\u{201d} \u{2014} 1 field")
    } else {
        format!("\u{201c}{name}\u{201d} \u{2014} {fields} fields")
    }
}

/// The control that arms a deletion.
///
/// ★ The ellipsis is load-bearing and follows the oldest convention in desktop
/// software: a verb that ends in `\u{2026}` **asks before it acts**. Pressing
/// this changes no document — it asks the engine what would go and draws the
/// answer. The confirm control below it has no ellipsis, for the same reason.
#[must_use]
pub fn field_group_delete_button() -> String {
    "Delete group\u{2026}".to_owned()
}

/// What pressing it will and will not do, on hover.
///
/// ★ It promises the *preview*, not the deletion, because that is what the
/// press does. A hover that described the deletion would make the first press
/// feel like the destructive one, and an operator who then pressed it
/// tentatively and read nothing would have learned that the control is broken.
#[must_use]
pub fn field_group_delete_hover(name: &str) -> String {
    format!(
        "Shows exactly what deleting \u{201c}{name}\u{201d} would remove. Nothing changes \
         until you confirm."
    )
}

/// **The pre-press disclosure**: the three numbers, in one sentence.
///
/// # ★★★ Three numbers, and every one of them is invisible
///
/// This is the sentence the whole preview exists to produce, and the engine's
/// own words for why it must exist are worth keeping at the call site:
/// *"an operator looking at a collapsed tree row cannot see how many that is or
/// what they are called."*
///
/// - **fields** — the terminals. What the operator means by "the form".
/// - **boxes** — widget annotations. Not `fields`, because one field can draw
///   on three pages, and those three pages are the ones the operator is *not*
///   looking at.
/// - **groups** — grouping nodes, *including the one that was named*. Greater
///   than one means the removal also empties an ancestor or an intermediate,
///   which is a node the operator did not choose and cannot see.
///
/// ★ The `groups` clause is omitted when the count is 1, because "1 group" is
/// the ordinary case and carrying it every time trains the eye to skip the
/// clause that matters on the day it says 3.
#[must_use]
pub fn field_group_preview_summary(
    name: &str,
    fields: usize,
    boxes: usize,
    groups: usize,
) -> String {
    let mut line = format!(
        "Deleting \u{201c}{name}\u{201d} removes {} and {}",
        plural(fields, "field", "fields"),
        plural(boxes, "box on the page", "boxes on the page"),
    );
    if groups > 1 {
        line.push_str(&format!(
            ", and empties {groups} field groups in total \u{2014} not only this one"
        ));
    }
    line.push('.');
    line
}

/// **The terminal fields, by name.**
///
/// ★★ Named rather than counted, and the argument is core's: a count answers
/// *how much*, a list answers *which* — and *which* is what decides whether the
/// operator presses. A form whose group holds `Personal.Name` and
/// `Personal.Address.Zip` is a different decision from one holding
/// `Personal.Signature`, and the count is identical.
///
/// Capped at [`MAX_LISTED_NAMES`]; the overflow is counted rather than dropped,
/// because a list that silently stops is a list the operator believes is
/// complete.
///
/// Returns `None` when there is nothing to list. That is not reachable from a
/// resolvable group — core rules that a node with no terminals beneath it *is
/// not a grouping node* — and is handled rather than asserted, because an
/// empty list under a sentence promising names is a worse failure than a
/// missing line.
#[must_use]
pub fn field_group_preview_names(names: &[String]) -> Option<String> {
    if names.is_empty() {
        return None;
    }
    let shown: Vec<&str> = names
        .iter()
        .take(MAX_LISTED_NAMES)
        .map(String::as_str)
        .collect();
    let listed = shown.join(", ");
    let hidden = names.len().saturating_sub(shown.len());
    Some(if hidden == 0 {
        format!("Fields removed: {listed}.")
    } else {
        format!("Fields removed: {listed}, and {hidden} more.")
    })
}

/// The control that commits.
///
/// ★ It states the **count**, not just the verb. A confirm button reading
/// "Delete" beside a sentence reading "4 fields" makes the operator carry the
/// number across; one reading "Delete 4 fields" is the same decision with
/// nothing to remember. No ellipsis: this one acts.
#[must_use]
pub fn field_group_preview_confirm(fields: usize) -> String {
    format!("Delete {}", plural(fields, "field", "fields"))
}

/// The control that disarms, changing nothing.
#[must_use]
pub fn field_group_preview_cancel() -> String {
    "Cancel".to_owned()
}

/// **The preview itself refused.**
///
/// ★★ Close to unreachable from this surface, because the section asks
/// `deletion_refusal` before it draws a single control and renders
/// [`field_groups_refusal`] instead when the answer is `Some`. It is worded
/// anyway, for the case the two disagree — which would mean the query and the
/// preview have come apart, and is a fault to see rather than a silence to
/// wonder about.
///
/// The engine's own reason is **not** printed. `EditError`'s `Display` is
/// written for a log; it reaches the trace, and the operator gets a sentence
/// that says what happened and what state the document is in.
#[must_use]
pub fn field_group_preview_refused(name: &str) -> String {
    format!(
        "pdfcer could not work out what deleting \u{201c}{name}\u{201d} would remove, so \
         nothing was changed. The document is exactly as it was."
    )
}

/// **The preview refused** — the decline-channel wording, without the name.
///
/// # ★★★ Why there are two of these and why this one drops the group's name
///
/// The pair above went to `record_note`, which renders under **`⚑ About your
/// last edit:`** — and `crate::text::status`' own rule forbids exactly that for
/// a decline, in as many words: *"an operator who reads 'About your last edit'
/// after a gesture that did nothing has been told a small lie confidently."*
/// A decline gets `⊗` and its own lead-in, because nothing happened.
///
/// The correct channel is `app::status::decline`, whose `Declined` is **`Copy`**
/// — a deliberate property, and one a `String` variant would take away. So the
/// name goes, and the loss is small: the operator pressed a button on a named
/// row, with the confirmation block naming that group still on screen. The
/// sentence has to say what happened, not re-identify what they were looking at.
///
/// ⇒ The two above are kept rather than deleted, because they are still the
/// right wording for a *disclosure* if this verb ever gains one, and because
/// deleting them would erase the record of which channel was wrong.
#[must_use]
pub const fn field_group_preview_declined() -> &'static str {
    "pdfcer could not work out what deleting that field group would remove, so nothing was \
     changed. The document is exactly as it was."
}

/// **The deletion refused, after the operator confirmed** — the decline-channel
/// wording. See [`field_group_preview_declined`] for why the name is absent.
#[must_use]
pub const fn field_group_delete_declined() -> &'static str {
    "That field group was not deleted \u{2014} pdfcer declined the change and the form is \
     unchanged. Nothing was removed."
}

/// **The deletion refused, after the operator confirmed.**
///
/// ★★★ The one sentence on this surface that must never be a silence. The
/// operator has read a preview, decided, and pressed a button whose label named
/// a number of fields. If the call then refuses and nothing is said, the
/// program has just shown them a list of what it was about to destroy and then
/// behaved exactly as if it had destroyed it.
#[must_use]
pub fn field_group_delete_refused(name: &str) -> String {
    format!(
        "\u{201c}{name}\u{201d} was not deleted \u{2014} pdfcer declined the change and the form \
         is unchanged. Nothing was removed."
    )
}

/// **What the deletion actually took**, from the engine's returned report.
///
/// # ★★★ The engine's numbers, not the preview's
///
/// `delete_field_group` deliberately overwrites `nodes_removed` with *"what the
/// cascade ACTUALLY emptied, not the preview's prediction"*, and carries a
/// `debug_assert` for the day the two disagree. This sentence is built from the
/// returned report for the same reason: reporting the prediction would make the
/// two agree even on the day they stop.
///
/// # Why it repeats numbers the operator already read
///
/// Because they read them about a *hypothetical*. The preview said what would
/// happen; this says what did. On a form field — invisible on the page,
/// invisible in the canvas, invisible in any raster — there is no other
/// evidence that the press did anything at all, and "the row disappeared from a
/// list" is evidence only for an operator who was looking at the list.
#[must_use]
pub fn field_group_deleted(name: &str, fields: usize, boxes: usize, groups: usize) -> String {
    let mut line = format!(
        "Deleted \u{201c}{name}\u{201d}: {} and {} removed",
        plural(fields, "field", "fields"),
        plural(boxes, "box", "boxes"),
    );
    if groups > 1 {
        line.push_str(&format!(", along with {groups} field groups in total"));
    }
    line.push('.');
    line
}

/// `1 field` / `4 fields`, without a `{n} field(s)` in operator copy.
///
/// ★ `(s)` is a form-filling convention that leaked into prose across this
/// catalog, and it reads as a machine talking. One helper here rather than a
/// conditional at each of the four call sites.
fn plural(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The three numbers are all present, and the groups clause appears only
    /// when it says something.
    #[test]
    fn the_summary_names_what_cannot_be_seen() {
        let one = field_group_preview_summary("Personal", 4, 6, 1);
        assert!(one.contains("4 fields"), "{one}");
        assert!(one.contains("6 boxes"), "{one}");
        assert!(
            !one.contains("in total"),
            "a lone group must not carry the cascade clause: {one}"
        );

        let cascade = field_group_preview_summary("Personal", 4, 6, 3);
        assert!(
            cascade.contains("3 field groups in total"),
            "a cascade that empties an ancestor the operator never named must say so: {cascade}"
        );
    }

    /// Singular reads as English rather than as `1 field(s)`.
    #[test]
    fn one_of_a_thing_is_not_written_with_a_bracketed_s() {
        let line = field_group_preview_summary("Personal", 1, 1, 1);
        assert!(line.contains("1 field "), "{line}");
        assert!(!line.contains("(s)"), "{line}");
        assert_eq!(field_group_preview_confirm(1), "Delete 1 field");
    }

    /// The overflow is counted, never dropped.
    #[test]
    fn a_capped_list_says_how_many_it_did_not_name() {
        let names: Vec<String> = (0..MAX_LISTED_NAMES + 3)
            .map(|i| format!("Personal.F{i}"))
            .collect();
        let line = field_group_preview_names(&names).expect("non-empty");
        assert!(line.contains("and 3 more"), "{line}");
        assert!(line.contains("Personal.F0"), "{line}");
        assert!(
            !line.contains(&format!("Personal.F{}", MAX_LISTED_NAMES)),
            "the cap must actually cap: {line}"
        );
    }

    /// A short list is printed whole, with no "and 0 more".
    #[test]
    fn a_short_list_is_named_completely() {
        let names = vec!["Personal.Name".to_owned(), "Personal.Zip".to_owned()];
        let line = field_group_preview_names(&names).expect("non-empty");
        assert_eq!(line, "Fields removed: Personal.Name, Personal.Zip.");
    }

    /// ★ The encrypted refusal does not blame a signature, and the certified one
    /// does not blame encryption. Naming the wrong cause sends the operator
    /// looking for something that is not in their file.
    #[test]
    fn each_refusal_names_its_own_cause() {
        let encrypted = field_groups_refusal(&EditError::DocumentEncrypted);
        assert!(encrypted.contains("encrypted"), "{encrypted}");
        assert!(
            !encrypted.to_ascii_lowercase().contains("signature"),
            "{encrypted}"
        );
    }
}
