//! # `text::buttonaction` — every word the *What this button does* chooser says
//!
//! One module for one control, because the control is where this project's
//! rule-4 obligation is heaviest: two of the seven choices write an address
//! into the document that some other program may act on, and **the operator
//! cannot see that by looking at the page**. Everything they can learn about it
//! has to be said here.
//!
//! ## ★★★ The disclosure rule these strings implement
//!
//! `pdfcer-core`'s reply on `set_button_action`, 2026-08-30, measured Acrobat
//! Reader's own submit warning through UI Automation and found it names
//! **scheme and host only** — not the port, not the path — and says **nothing
//! whatever** about the payload: no field count, no whole-file indication, no
//! mention of hidden fields. Its "remember this site" box is ticked by default.
//!
//! ⇒ *"Design it from `SubmitDisclosure`, not from Acrobat"* — a straight copy
//! of Acrobat's dialog would be a **regression** against what pdfcer can now
//! say. So these strings state the whole address, and the six facts about the
//! payload that ISO 32000-1 §12.7.5.2 makes true and nobody can guess.
//!
//! ## ★★ Where the disclosure is NOT
//!
//! **Not on the canvas.** Rule 4's clause that is most often got backwards:
//! applied content renders exactly as saved content will. A button carrying a
//! submit gets no badge, no tint and no dashed outline on the page — it is
//! drawn as the file will draw it. The disclosure lives in this dialog, and
//! afterwards in the status line, which is off-canvas by construction.
//!
//! ## ★ Not a warning, and not a refusal
//!
//! None of these say *"are you sure?"*. No scheme, host or port is refused
//! anywhere — destination policy is open by operator ruling — and `https`
//! appears **zero times** in ISO 32000-1, so blocking `http://` would be pdfcer
//! inventing a conformance requirement. [`submit_unencrypted`] therefore
//! **states** it and lets the operator decide. Nothing here may be phrased as
//! *"the standard requires"*, because none of it is.

use crate::canvas::formfield::action::{
    ActionBlocker, ButtonDoesKind, NamedChoice, PageViewChoice,
};

/// ★★★ **What an EXISTING button currently does**, one sentence per state.
///
/// # The four states, and why there are four
///
/// `EditSession::button_action` answers with `ButtonActionState`, and this
/// shell asked for **three** — `None`, `Known`, `Foreign`. `pdfcer-core` shipped
/// four and explained why, and the explanation is worth carrying because the
/// distinction is entirely about what a control may OFFER:
///
/// | state | what it means | what this row offers |
/// |---|---|---|
/// | `None` | no `/A` at all | "Nothing" — set one |
/// | `Known` | modelled; writable back unchanged | show it, change it |
/// | `Unmodelled` | pdfcer **authors** this subtype and did not decode this instance | name it, offer to **replace**, never claim to show it |
/// | `Foreign` | pdfcer recognises it and **will not author** it | name it, offer nothing |
///
/// ★★ `Unmodelled` and `Foreign` differ in exactly one thing — whether
/// replacing is offered — and that is the decision the operator is actually
/// being asked to make. A three-state enum would have forced a wrong answer in
/// one direction or the other: `Foreign("SubmitForm")` on a submit pdfcer writes
/// happily would grey a row that should have been live.
///
/// ★ Today `GoTo` and `SubmitForm` answer `Unmodelled` — authored, not yet
/// decoded. That will widen, and widening is additive: a state that becomes
/// `Known` gains a value to show and loses nothing.
#[must_use]
pub fn current_none() -> String {
    "Does nothing when pressed.".to_owned()
}

/// A modelled action, named in the operator's terms.
#[must_use]
pub fn current_known(kind: ButtonDoesKind) -> String {
    format!("Pressing it: {}.", does_choice(kind).to_lowercase())
}

/// ★★ A subtype pdfcer writes but did not decode **this instance** of.
///
/// The sentence must do two things at once and neither may be dropped: say the
/// button **does** something, and refuse to say what. Claiming to show it would
/// be the sneaky half of rule 4; claiming it does nothing would be worse.
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

/// ★★★ A subtype pdfcer recognises and will not author.
///
/// The variant the request argued for by name. `None` and `Known` could both be
/// synthesised by a shell that guessed; this one cannot, and it is what lets
/// the row say *"this button runs a script"* instead of silently offering to
/// replace one with "Nothing".
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
/// ★ The refusals match the WRITER's, deliberately: a shell must not learn
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
///
/// ★★ `replaced` is what the engine says was destroyed, **including a script**.
/// `ButtonActionChange::replaced` carries it as a `String` rather than an
/// `Option<ButtonAction>` precisely so a removed script is expressible — a form
/// editor overwriting another tool's work should know it did.
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
///
/// Phrased as a question about the button rather than as *"Action"*, which is
/// the format's word and not the operator's. Acrobat's tab is called *Actions*
/// and this project's standing rule is to use the conventional interaction —
/// but the conventional *interaction* is a chooser of behaviours, and the label
/// on it may be the plainer one.
#[must_use]
pub fn does_label() -> String {
    "What pressing it does".to_owned()
}

/// Each choice, as it appears in the drop-down.
///
/// ★ Verb-first and in the operator's terms, never the `/S` subtype name. A
/// chooser reading *ResetForm / GoToPage / SubmitForm* would be pdfcer showing
/// its own internals to somebody who wants a button that clears the form.
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
///
/// ★★ Every one of the seven says what the action **reaches**, because that is
/// the property none of them shows on screen and the property they differ on.
/// The four that reach nothing say so in as many words, so that the two that do
/// are not the only ones carrying a sentence — a disclosure that appears only
/// on the dangerous choice teaches an operator to skip reading it.
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
///
/// ★★ The terminal-name requirement, said before the engine refuses it. Table
/// 210 states nothing about descendant expansion — the phrase *"all descendants
/// of the specified fields"* occurs twice per edition of ISO 32000 and never on
/// this row — so a grouping name is a button that hides a subtree in one reader
/// and nothing in another. `pdfcer-core` refuses one by name; this says why
/// while the box that holds the mistake is still on screen.
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

/// ★★★ **The submit disclosure** — the six facts §12.7.5.2 makes true and
/// nobody can guess.
///
/// Shown whole, before the button exists, and every clause is sourced:
///
/// 1. **Hidden fields are sent.** `Hidden` is an *annotation* flag; every
///    submit selector addresses *field* dictionaries. The only field-level
///    withhold flag that exists is `NoExport`. Different objects.
/// 2. **Masked fields are sent as plain text.** `Password`'s NOTE constrains
///    storage, not transmission.
/// 3. **A file-select field sends the contents of the local file it names.**
/// 4. **The baseline payload already carries this document's own file path and
///    its trailer `/ID`** — with nothing configured, `/Flags 0`.
/// 5. Not stated here because pdfcer writes the baseline: `IncludeAppendSaves`
///    would turn a submit into a save. Named in the module header so that a
///    later option to set it arrives with its sentence already written.
/// 6. Not stated here for the same reason: `SubmitPDF` ignores field selection
///    entirely.
///
/// ★ It does not say "are you sure". It is a statement of what the file will
/// declare, positioned where the operator is deciding whether to declare it.
#[must_use]
pub fn submit_disclosure() -> String {
    "What that declaration would cover, if a reader program acts on it: every field's value, \
     including fields that are hidden on the page and fields whose characters are masked as \
     they are typed — the masking is only how they are drawn. A field that names a file on \
     the computer sends that file's contents. The message also carries this document's own \
     location on disk and the identifier stored in it."
        .to_owned()
}

/// ★★ Said when the address is not `https:` — a **statement**, never a
/// refusal.
///
/// The standard states no TLS rule; `https` appears zero times in ISO 32000-1.
/// pdfcer does not invent one, so this is the whole of the response: say it, and
/// let the operator decide. Refusing would be pdfcer enforcing a rule nobody
/// wrote, and doing so silently would be worse.
#[must_use]
pub fn submit_unencrypted() -> String {
    "This address is not encrypted, so anything sent to it could be read in transit.".to_owned()
}

/// Why the dialog will not accept the draft yet.
///
/// ★ One sentence per blocker, each naming **the box to fix** rather than the
/// rule that was broken. An operator reading *"the destination is not
/// absolute"* has to work out which of four boxes that refers to.
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
///
/// ★ Names the action in the operator's words, not the `/S` subtype, and is
/// the off-canvas half of rule 4: the button on the page is drawn exactly as
/// the saved file will draw it, and what it now *does* is said here.
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

/// ★★ Said when the button and its action could not be folded into one undo
/// entry.
///
/// `EditSession::coalesce_last` answers `false` when the undo stack was shorter
/// than the count asked for — every change is applied and only the **grouping**
/// failed. So the button exists and does what it was asked to do; the only
/// thing wrong is that taking it back needs two presses.
///
/// ★ Worth a sentence rather than a shrug: an operator who presses Ctrl+Z once,
/// sees a button still sitting there, and is told nothing will conclude that
/// undo is broken — which is a far worse belief than the truth.
#[must_use]
pub fn two_undo_entries(name: &str) -> String {
    format!("Placed {name}. Undoing it takes two presses rather than one.")
}

/// Said when the action could not be written although the button was placed.
///
/// ★★★ **Two commands, and the second one can fail on its own.** `pdfcer-core`
/// authors the button and sets the action as separate verbs, so a refusal on
/// the second leaves a correctly placed button with no behaviour. Silence there
/// would be the exact defect this whole feature exists to remove — a button
/// that looks right and does nothing — arriving by a different door.
///
/// The engine's own words are appended, because they name the specific
/// condition: a page past the end, a field that is not there, a target that is
/// a group.
#[must_use]
pub fn action_refused(name: &str, why: &str) -> String {
    format!("{name} was placed, and could not be given anything to do: {why}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ Every choice has a name and a note, and the note says what it reaches.
    ///
    /// The reach clause is the load-bearing half — see [`does_note`]'s comment
    /// on why the four inert ones carry one too. A new variant added without a
    /// note would fail to compile (the `match` is exhaustive); a new variant
    /// added with an empty one would not, so this asserts non-emptiness.
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

    /// ★★ The two addressed kinds must not claim to reach nothing, and the five
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
    ///
    /// Asserted by keyword rather than by exact text so a rewording does not
    /// break it — but a rewording that DROPS one of the four will, which is the
    /// point. These are the facts an operator cannot learn any other way.
    #[test]
    fn the_submit_disclosure_names_every_fact_it_owes() {
        let d = submit_disclosure();
        for owed in ["hidden", "masked", "file", "location"] {
            assert!(d.contains(owed), "the disclosure dropped `{owed}`: {d}");
        }
    }

    /// ★★★ Nothing here may refuse a scheme. If this test ever needs changing,
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
