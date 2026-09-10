//! # `dialogs::settings::defaultapp` — the button at the top of Settings
//!
//! `OPERATOR_REQUESTS.md` **O173**: *"Then it should be in the top of our
//! settings as a button to execute the changeover."*
//!
//! ## ★★★ Why this is FIRST, above the presets row
//!
//! [`super`]'s ordering rule runs from what the **program** looks like, through
//! what the **document** is made of, to what pdfcer **does with it** — and the
//! presets row sits above all three because it sets all three. This group is
//! above even that, and it is the only thing in the window that is not a
//! setting at all: it is an **act**, performed once, whose effect is outside
//! pdfcer entirely.
//!
//! The operator asked for it there by name, and the reason it is the right
//! place survives independently of the ask: it is the one control here that
//! somebody arrives at Settings *specifically* to press, having just
//! double-clicked a drawing and watched the wrong program open. A control found
//! by scrolling is a control found by people who already knew it existed.
//!
//! ⇒ It is **not** a collapsible [`super::widgets::group`], for the same
//! reason. A collapsed heading is a heading whose button is not on screen, and
//! `super`'s own comment records what that cost the last time: *"Opening
//! Settings showed nothing but a list of standards."* Four short lines is a
//! price worth paying at the top of a scroll area; a hidden button is not.
//!
//! ## ★★ It is drawn whether or not it would do anything
//!
//! R9's escape hatch, the same shape [`super::acrobat`] argues at length. There
//! is no ribbon command for this and there never will be — it is a machine
//! setting, not a document one — so this group is the **only** evidence in the
//! program that the capability exists. An operator who ticked *Don't ask me
//! again* on the startup offer, and later changed their mind, has exactly one
//! place to look, and it must be here whatever state the machine is in.
//!
//! ## ★ Rule 4 — fuzzy, never sneaky
//!
//! No canvas is involved, so the marking clause is not in play. The disclosure
//! clause is, and it is the whole of [`state_line`]: pdfcer performs half an
//! act and cannot perform the other half, so it says which half it did and
//! reads back what Windows actually thinks rather than reporting its own
//! intention. See [`crate::app::assoc`] for why the second half is impossible
//! and [`crate::text::assoc`] for the wording that carries it.
//!
//! ## Rule 15
//!
//! No dimension of either kind appears in this module.

use egui::Ui;

use crate::app::assoc::{self, Registration, Status};
use crate::text::assoc as t;

/// The state line's region.
///
/// ★ Named for [`super::acrobat::REGION_RESOLVED`]'s reason exactly: the whole
/// value of this line is that it is **on screen and legible**, and a driven
/// check that read the trace instead would learn what pdfcer found and nothing
/// about whether the operator can see it.
pub const REGION_STATE: &str = "settings:defaultapp.state"; // ui-text-exempt: trace region name, never displayed

/// The button's region.
pub const REGION_ACTION: &str = "settings:defaultapp.action"; // ui-text-exempt: trace region name, never displayed

/// What this group knows about the machine, for one opening of the window.
///
/// ★★★ **The `Option` is the whole design.** Every field in [`Status`] costs a
/// `reg.exe` process to obtain, and a Settings pane redraws on **every frame**
/// — so probing from [`group`] unconditionally would spawn two processes sixty
/// times a second. `super::acrobat`'s header states the identical rule for the
/// identical reason. Filling it lazily on the first paint gives exactly one
/// probe per opening of the window, and pressing the button clears it so the
/// next frame reads the machine again.
///
/// ⇒ It is deliberately **not** filled in [`super::Draft::focused_on`]. Draft
/// construction happens in unit tests that never paint, and a constructor that
/// spawned subprocesses would make every one of them slower and one of them
/// machine-dependent.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    /// What the machine said, read once per opening. See the type's own note.
    pub status: Option<Status>,
    /// What happened the last time the button was pressed, if it has been.
    ///
    /// `Some` with the hand-over sentence on success, `Some` with the refusal
    /// on failure. It is **not** a claim that the default changed — nothing in
    /// this module is allowed to make that claim; see [`state_line`].
    pub note: Option<String>,
}

impl State {
    /// Forget what was read, so the next paint reads the machine again.
    ///
    /// ★ Called after the button acts, because the act changes two of the three
    /// things the line reports. It does **not** clear [`Self::note`]: the note
    /// says what pdfcer just did, the status says what Windows now thinks, and
    /// conflating them is how a surface starts claiming an outcome it caused
    /// only half of.
    pub fn forget(&mut self) {
        self.status = None;
    }
}

/// **What Windows currently opens PDFs with, and where pdfcer stands.**
///
/// Two facts, in that order, because they answer two different questions and an
/// operator arrives with one of them: *"why did Edge open?"* is answered by the
/// first, and *"did the button work?"* by the second.
///
/// ★★ The second is suppressed when pdfcer already **is** the default, because
/// then it says nothing the first has not: being the chosen program implies
/// being in the list, and a second sentence restating it would train the reader
/// to skip the pair.
#[must_use]
pub fn state_line(status: &Status) -> String {
    let owner = if status.is_default() {
        return t::state_default();
    } else if let Some(other) = status.owner.as_deref() {
        t::state_other(other)
    } else {
        t::state_unset()
    };
    let standing = match status.registration {
        Registration::Here => t::state_registered(),
        Registration::Absent => t::state_unregistered(),
        Registration::Elsewhere => t::state_registered_elsewhere(),
    };
    // ui-text-exempt: a space between two sentences that are BOTH already in
    // the catalog. There is no word here to translate or to reword; putting the
    // join in the catalog would give a translator a lone separator to maintain.
    format!("{owner} {standing}")
}

/// The group: two lines of explanation, the state line, and the button.
///
/// `state` is borrowed mutably because pressing the button changes what the
/// machine will say next — see [`State::forget`].
pub fn group(ui: &mut Ui, state: &mut State) {
    let status = state.status.get_or_insert_with(assoc::probe).clone();

    ui.label(egui::RichText::new(t::title()));
    ui.label(egui::RichText::new(t::body()).small().weak());
    ui.add_space(4.0);

    // ★★ `notice` rather than the body ink, and a theme role rather than a
    // colour: it is a report about the machine rather than part of a setting,
    // and `tools/gates/check-theme-colors.sh` forbids the literal either way.
    // ★ The note wins when there is one. It is the newer fact — it describes
    // something that happened since the status was read — and showing both
    // would put a sentence about what pdfcer just did beside a sentence about
    // what Windows thought beforehand.
    let line = ui.label(
        egui::RichText::new(state.note.clone().unwrap_or_else(|| state_line(&status)))
            .color(egui_shell::theme::Theme::of(ui.ctx()).palette.notice),
    );
    crate::diag::ui_rect_visible(REGION_STATE, line.rect, ui.clip_rect());

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        // ★ R9: no greying. The button is offered whatever the machine's state
        // is, including when pdfcer already IS the default — because Windows'
        // own page is also where somebody goes to change their mind, and a
        // control that vanished on success would strand them there.
        let act = ui.button(t::action());
        crate::diag::ui_rect_visible(REGION_ACTION, act.rect, ui.clip_rect());
        if act.on_hover_text(t::action_hover()).clicked() {
            state.note = Some(act_now());
            state.forget();
        }
    });
}

/// Register, then hand over to Windows. Returns the sentence to show.
///
/// ★★★ **Two steps, and the second runs only if the first succeeded.** Opening
/// Windows' Default-apps page having failed to register would deep-link the
/// operator to a list pdfcer is not in — a page that proves the feature is
/// broken, with no explanation on it, in an application pdfcer does not own. A
/// refusal shown here is a refusal the operator can act on.
#[must_use]
fn act_now() -> String {
    if let Err(said) = assoc::register() {
        return said;
    }
    match assoc::open_settings_page() {
        Ok(()) => t::handed_over(),
        Err(said) => said,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(owner: Option<&str>, registration: Registration) -> Status {
        Status {
            owner: owner.map(str::to_owned),
            registration,
        }
    }

    /// **When pdfcer is the default, the line says so and stops.**
    ///
    /// The second sentence would restate the first. See [`state_line`].
    #[test]
    fn being_the_default_is_the_whole_line() {
        let line = state_line(&status(Some(assoc::PROGID), Registration::Here));
        assert_eq!(line, crate::text::assoc::state_default());
    }

    /// **Another program holding the association is named**, so the operator
    /// can act on it rather than guess.
    #[test]
    fn another_owner_reaches_the_line() {
        let line = state_line(&status(Some("AppXd4nrz"), Registration::Here));
        assert!(line.contains("AppXd4nrz"), "{line}");
    }

    /// **A registration pointing at a DIFFERENT copy is reported as that**, not
    /// as registered.
    ///
    /// ★ The state that looks like success from the inside — the key exists and
    /// names pdfcer — and opens a build the operator thought they had replaced.
    /// A line that folded it into *"pdfcer is in the list"* would be true and
    /// useless.
    #[test]
    fn a_stale_registration_is_distinguished_from_a_live_one() {
        let stale = state_line(&status(None, Registration::Elsewhere));
        let live = state_line(&status(None, Registration::Here));
        assert_ne!(stale, live);
        assert!(
            stale.contains("different copy"),
            "the stale case must say which way it is wrong: {stale}"
        );
    }

    /// **No choice recorded is reported as no choice**, not as a fault.
    ///
    /// The ordinary state of a fresh Windows account, and a line calling it an
    /// error would send somebody looking for a problem they do not have.
    #[test]
    fn an_unset_association_is_not_dressed_up_as_a_failure() {
        let line = state_line(&status(None, Registration::Absent));
        assert!(
            line.starts_with(&crate::text::assoc::state_unset()),
            "{line}"
        );
    }

    /// **`forget` drops the reading and keeps the note.**
    ///
    /// Two different facts — what pdfcer did, and what Windows now thinks — and
    /// the whole point of the pair is that they can disagree.
    #[test]
    fn forgetting_the_reading_keeps_what_pdfcer_just_did() {
        let mut s = State {
            status: Some(status(None, Registration::Absent)),
            note: Some("did a thing".to_owned()),
        };
        s.forget();
        assert!(s.status.is_none());
        assert_eq!(s.note.as_deref(), Some("did a thing"));
    }
}
