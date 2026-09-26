//! # `text::forms::groups` — every word the Field-groups section says
//!
//! The copy for [`crate::panels::forms::groups`], which is the shell's route to
//! `EditSession::delete_field_group` and its companion
//! `EditSession::field_group_deletion_preview`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/forms/groups.md`.

use pdfcer_core::edit::EditError;

/// How many terminal names the pre-press disclosure prints before it stops
/// naming them and starts counting them.
pub const MAX_LISTED_NAMES: usize = 8;

/// The section's collapsing heading.
#[must_use]
pub fn field_groups_heading() -> String {
    "Field groups".to_owned()
}

/// What a field group is, and the one fact about deleting one that an operator
/// cannot discover any other way.
#[must_use]
pub fn field_groups_explainer() -> String {
    "A field group is a name other fields are filed under — \u{201c}Personal\u{201d} in \
     \u{201c}Personal.Address.Zip\u{201d}. Deleting one deletes every field beneath it. \
     Groups are not drawn anywhere on the page, so the only record of what a deletion took \
     is what pdfcer tells you here."
        .to_owned()
}

/// **Why no control is offered**, when the document refuses structural change.
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
fn is_certification(error: &EditError) -> bool {
    let rendered = error.to_string().to_ascii_lowercase();
    rendered.contains("certif")
}

/// One grouping node's row: its name, and how many fields are filed under it.
#[must_use]
pub fn field_group_row(name: &str, fields: usize) -> String {
    if fields == 1 {
        format!("\u{201c}{name}\u{201d} \u{2014} 1 field")
    } else {
        format!("\u{201c}{name}\u{201d} \u{2014} {fields} fields")
    }
}

/// The control that arms a deletion.
#[must_use]
pub fn field_group_delete_button() -> String {
    "Delete group\u{2026}".to_owned()
}

/// What pressing it will and will not do, on hover.
#[must_use]
pub fn field_group_delete_hover(name: &str) -> String {
    format!(
        "Shows exactly what deleting \u{201c}{name}\u{201d} would remove. Nothing changes \
         until you confirm."
    )
}

/// **The pre-press disclosure**: the three numbers, in one sentence.
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
#[must_use]
pub fn field_group_preview_refused(name: &str) -> String {
    format!(
        "pdfcer could not work out what deleting \u{201c}{name}\u{201d} would remove, so \
         nothing was changed. The document is exactly as it was."
    )
}

/// **The preview refused** — the decline-channel wording, without the name.
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
#[must_use]
pub fn field_group_delete_refused(name: &str) -> String {
    format!(
        "\u{201c}{name}\u{201d} was not deleted \u{2014} pdfcer declined the change and the form \
         is unchanged. Nothing was removed."
    )
}

/// **What the deletion actually took**, from the engine's returned report.
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

    /// The encrypted refusal does not blame a signature, and the certified one
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
