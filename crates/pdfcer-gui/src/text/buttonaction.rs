//! # `text::buttonaction` — every word the *What this button does* chooser says
//!
//! One module for one control, because the control is where this project's
//! rule-4 obligation is heaviest: two of the seven choices write an address
//! into the document that some other program may act on, and **the operator
//! cannot see that by looking at the page**. Everything they can learn about it
//! has to be said here.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/buttonaction.md`.

use crate::canvas::formfield::action::{
    ActionBlocker, ButtonDoesKind, NamedChoice, PageViewChoice,
};

/// **What an EXISTING button currently does**, one sentence per state.
#[must_use]
pub fn current_none() -> String {
    "Does nothing when pressed.".to_owned()
}

/// A modelled action, named in the operator's terms.
#[must_use]
pub fn current_known(kind: ButtonDoesKind) -> String {
    format!("Pressing it: {}.", does_choice(kind).to_lowercase())
}

/// A subtype pdfcer writes but did not decode **this instance** of.
#[must_use]
pub fn current_unmodelled(subtype: &str) -> String {
    let named = if subtype.is_empty() {
        "an action".to_owned()
    } else {
        format!("a {subtype} action")
    };
    format!(
        "This button carries {named}. pdfcer cannot show you its settings yet, so you can replace it — which discards what is there now — but not edit it."
    )
}

/// A subtype pdfcer recognises and will not author.
#[must_use]
pub fn current_foreign(subtype: &str) -> String {
    let named = if subtype.is_empty() {
        "an action".to_owned()
    } else {
        format!("a {subtype} action")
    };
    format!(
        "This button carries {named}, which pdfcer will not write. It is left exactly as it is, and saving the document keeps it."
    )
}

/// Said when the reader itself refused — the field is not a push button, or is
/// not there.
///
/// The refusals match the WRITER's, deliberately: a shell must not learn
/// through the reader about a field it would be refused permission to change.
#[must_use]
pub fn current_unreadable(why: &str) -> String {
    format!("pdfcer could not read what this button does: {why}")
}

/// The control that opens the chooser on an existing button.
#[must_use]
pub fn change_button() -> String {
    "Change…".to_owned()
}

/// The control that applies a change to an existing button.
#[must_use]
pub fn apply_button() -> String {
    "Apply".to_owned()
}

/// The status line after an existing button's action is changed.
#[must_use]
pub fn changed(name: &str, replaced: Option<&str>) -> String {
    match replaced {
        Some(was) if !was.is_empty() && was != "none" => {
            format!("{name} changed. It previously carried a {was} action, which is gone.")
        }
        _ => format!("{name} changed."),
    }
}

/// The label above the chooser.
#[must_use]
pub fn does_label() -> String {
    "What pressing it does".to_owned()
}

/// Each choice, as it appears in the drop-down.
#[must_use]
pub fn does_choice(kind: ButtonDoesKind) -> String {
    match kind {
        ButtonDoesKind::Nothing => "Nothing",
        ButtonDoesKind::ResetForm => "Clear the form",
        ButtonDoesKind::GoToPage => "Go to a page",
        ButtonDoesKind::Named => "Move through the pages",
        ButtonDoesKind::ShowHide => "Show or hide fields",
        ButtonDoesKind::Uri => "Open a web address",
        ButtonDoesKind::SubmitForm => "Send the form's data",
    }
    .to_owned()
}

/// The one-line explanation under the chooser, per choice.
#[must_use]
pub fn does_note(kind: ButtonDoesKind) -> String {
    match kind {
        ButtonDoesKind::Nothing => {
            "The button is placed and does nothing when pressed. You can give it something to \
             do later."
        }
        ButtonDoesKind::ResetForm => {
            "Every field in this document goes back to the value it was created with. Nothing \
             leaves the document."
        }
        ButtonDoesKind::GoToPage => {
            "Jumps to a page of this document. The page is referred to by identity, so \
             reordering the pages does not break it. Nothing leaves the document."
        }
        ButtonDoesKind::Named => {
            "Asks the reader program to turn the page. Nothing leaves the document."
        }
        ButtonDoesKind::ShowHide => {
            "Hides or shows the fields you name. This is a setting rather than a switch — \
             pressing the button twice does not put them back. Nothing leaves the document."
        }
        ButtonDoesKind::Uri => {
            "Writes a web address into the document. pdfcer never opens it; a reader program \
             will, if someone presses the button."
        }
        ButtonDoesKind::SubmitForm => {
            "Writes an address into the document, and a declaration that the form's data \
             should be sent there. pdfcer sends nothing and has no way to — but a reader \
             program will, if someone presses the button."
        }
    }
    .to_owned()
}

/// The label above the page-number box.
#[must_use]
pub fn page_number_label() -> String {
    "Page".to_owned()
}

/// Each landing position, as it appears in its chooser.
#[must_use]
pub fn page_view_choice(view: PageViewChoice) -> String {
    match view {
        PageViewChoice::WholePage => "Fit the whole page",
        PageViewChoice::FullWidth => "Fit the page's width",
        PageViewChoice::TopLeft => "Top-left corner, same zoom",
    }
    .to_owned()
}

/// Each navigation action, as it appears in its chooser.
#[must_use]
pub fn named_choice(named: NamedChoice) -> String {
    match named {
        NamedChoice::NextPage => "Next page",
        NamedChoice::PrevPage => "Previous page",
        NamedChoice::FirstPage => "First page",
        NamedChoice::LastPage => "Last page",
    }
    .to_owned()
}

/// The label above the show/hide field list.
#[must_use]
pub fn targets_label() -> String {
    "Fields, one per line".to_owned()
}

/// What a show/hide target may be.
#[must_use]
pub fn targets_note() -> String {
    "Name the fields themselves, not a group they belong to. The standard does not say what a \
     group name means here, so different readers would do different things with it."
        .to_owned()
}

/// The hide / show pair.
#[must_use]
pub fn hide_them() -> String {
    "Hide them".to_owned()
}

/// The show half of the pair.
#[must_use]
pub fn show_them() -> String {
    "Show them".to_owned()
}

/// The label above the address box, for both addressed kinds.
#[must_use]
pub fn url_label() -> String {
    "Web address".to_owned()
}

/// **The submit disclosure** — the six facts §12.7.5.2 makes true and
/// nobody can guess.
#[must_use]
pub fn submit_disclosure() -> String {
    "What that declaration would cover, if a reader program acts on it: every field's value, \
     including fields that are hidden on the page and fields whose characters are masked as \
     they are typed — the masking is only how they are drawn. A field that names a file on \
     the computer sends that file's contents. The message also carries this document's own \
     location on disk and the identifier stored in it."
        .to_owned()
}

/// Said when the address is not `https:` — a **statement**, never a
/// refusal.
#[must_use]
pub fn submit_unencrypted() -> String {
    "This address is not encrypted, so anything sent to it could be read in transit.".to_owned()
}

/// Why the dialog will not accept the draft yet.
#[must_use]
pub fn blocker(reason: ActionBlocker) -> String {
    match reason {
        ActionBlocker::PageNumberMissing => {
            "Type the page number the button should jump to — 1 for the first page."
        }
        ActionBlocker::NoTargets => "Name at least one field for the button to act on.",
        ActionBlocker::UrlMissing => "Type the address.",
        ActionBlocker::UrlNotStatable => {
            "Type a complete address, beginning with https:// or http://, using ordinary \
             keyboard characters. An address without one, or with accented letters in it, \
             means different things to different reader programs."
        }
    }
    .to_owned()
}

/// The status line after a button is placed with an action.
#[must_use]
pub fn placed_with_action(name: &str, kind: ButtonDoesKind) -> String {
    if matches!(kind, ButtonDoesKind::Nothing) {
        format!("Placed {name}. It does nothing when pressed.")
    } else {
        format!(
            "Placed {name}. Pressing it: {}.",
            does_choice(kind).to_lowercase()
        )
    }
}

/// Said when the button and its action could not be folded into one undo
/// entry.
#[must_use]
pub fn two_undo_entries(name: &str) -> String {
    format!("Placed {name}. Undoing it takes two presses rather than one.")
}

/// Said when the action could not be written although the button was placed.
#[must_use]
pub fn action_refused(name: &str, why: &str) -> String {
    format!("{name} was placed, and could not be given anything to do: {why}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every choice has a name and a note, and the note says what it reaches.
    #[test]
    fn every_choice_is_named_and_explained() {
        for kind in ButtonDoesKind::ALL {
            assert!(!does_choice(kind).is_empty(), "{kind:?}");
            let note = does_note(kind);
            assert!(!note.is_empty(), "{kind:?}");
            assert!(
                note.contains("document") || note.contains("does nothing"),
                "{kind:?}: the note must say what the action reaches"
            );
        }
    }

    /// The two addressed kinds must not claim to reach nothing, and the five
    /// others must not omit that they do.
    #[test]
    fn only_the_inert_choices_say_nothing_leaves_the_document() {
        for kind in ButtonDoesKind::ALL {
            let says_inert = does_note(kind).contains("Nothing leaves the document")
                || matches!(kind, ButtonDoesKind::Nothing);
            assert_eq!(says_inert, !kind.reaches_outside(), "{kind:?}");
        }
    }

    /// The submit disclosure must carry all four facts it claims to.
    #[test]
    fn the_submit_disclosure_names_every_fact_it_owes() {
        let d = submit_disclosure();
        for owed in ["hidden", "masked", "file", "location"] {
            assert!(d.contains(owed), "the disclosure dropped `{owed}`: {d}");
        }
    }

    /// Nothing here may refuse a scheme. If this test ever needs changing,
    /// someone has made pdfcer enforce a rule ISO 32000-1 does not state.
    #[test]
    fn the_unencrypted_line_states_rather_than_refuses() {
        let s = submit_unencrypted();
        assert!(s.contains("not encrypted"));
        assert!(
            !s.to_lowercase().contains("cannot") && !s.to_lowercase().contains("not allowed"),
            "this is a statement, not a refusal: {s}"
        );
    }

    #[test]
    fn every_blocker_names_a_box_to_fix() {
        for reason in [
            ActionBlocker::PageNumberMissing,
            ActionBlocker::NoTargets,
            ActionBlocker::UrlMissing,
            ActionBlocker::UrlNotStatable,
        ] {
            let s = blocker(reason);
            assert!(!s.is_empty());
            assert!(
                s.starts_with("Type") || s.starts_with("Name"),
                "{reason:?} must tell the operator what to do: {s}"
            );
        }
    }
}
