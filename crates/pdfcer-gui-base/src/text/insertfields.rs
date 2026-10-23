//! # `text::insertfields` — what an insert did with the source's form fields
//!
//! The one sentence an insert owes about form fields that came across with
//! its pages (`InsertOutcome::fields_carried` and its neighbours). Orphaned
//! controls are worded by `text::pages::inserted`; this sentence covers the
//! fields that arrived whole and the ones left behind because only part of
//! them was inserted.

/// What the insert did with form fields; `None` when it carried none and
/// left none behind, so a page without fields says nothing.
#[must_use]
pub fn carried(carried: usize, renamed: usize, split: usize, form_created: bool) -> Option<String> {
    let mut parts = Vec::new();
    if carried > 0 {
        let mut line = match carried {
            1 => "1 form field came across with the pages".to_owned(),
            n => format!("{n} form fields came across with the pages"),
        };
        match renamed {
            0 => {}
            1 => line
                .push_str("; 1 was renamed because this document already has a field by that name"),
            n => line.push_str(&format!(
                "; {n} were renamed because this document already has fields by those names"
            )),
        }
        line.push('.');
        if form_created {
            line.push_str(" This document is now a fillable form.");
        }
        parts.push(line);
    }
    match split {
        0 => {}
        1 => parts.push(
            "1 form field was not brought across because only some of its controls are on the \
             inserted pages."
                .to_owned(),
        ),
        n => parts.push(format!(
            "{n} form fields were not brought across because only some of their controls are \
             on the inserted pages."
        )),
    }
    (!parts.is_empty()).then(|| parts.join(" "))
}

#[cfg(test)]
mod tests {
    use super::carried;

    #[test]
    fn nothing_carried_and_nothing_split_says_nothing() {
        assert_eq!(carried(0, 0, 0, false), None);
    }

    #[test]
    fn a_rename_and_a_new_form_are_both_said() {
        let line = carried(3, 1, 0, true).unwrap_or_default();
        assert!(line.starts_with("3 form fields came across"));
        assert!(line.contains("1 was renamed"));
        assert!(line.ends_with("now a fillable form."));
    }

    #[test]
    fn a_split_field_is_said_without_a_carry() {
        let line = carried(0, 0, 2, false).unwrap_or_default();
        assert!(line.starts_with("2 form fields were not brought across"));
    }
}
