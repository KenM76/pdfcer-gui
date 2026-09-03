//! # `canvas::formfield::action` — what a push button does when it is pressed
//!
//! The shell's model of `pdfcer_core::edit::ButtonAction`, and the one place it
//! is translated into the engine's type.
//!
//! ## ★★★ Why a second enum instead of using the engine's directly
//!
//! Three reasons, and only the third is about types.
//!
//! 1. **A draft is edited, and an engine action is complete.**
//!    `ButtonAction::GoToPage { page_index, view }` cannot represent *"the
//!    operator has chosen Go to a page and has not said which one yet"*, and a
//!    dialog spends most of its life in exactly that state. The variants here
//!    carry every parameter for every kind at once — the way a dialog holds a
//!    text box's contents whether or not that box is showing — so switching
//!    the chooser to *Reset* and back does not lose the page number that was
//!    typed. That is [`ButtonDoes`]'s whole shape and it is why it is a struct
//!    with a `kind` rather than an enum with payloads.
//!
//! 2. **The engine's type is `#[non_exhaustive]` and its constructors are
//!    private-by-omission.** `SubmitSpec::new(url)` then assign fields; a
//!    struct literal from outside the crate does not compile. A dialog that
//!    held one would have to build it fresh on every frame, which is the same
//!    work as building it once at commit — so building it once at commit is
//!    what this does.
//!
//! 3. **Not every draft is authorable, and the refusal has to be shown before
//!    the press.** [`ButtonDoes::blocker`] is the predicate the dialog greys
//!    its Add button on, and it exists so the operator is not told *"that URL
//!    is relative"* by a dialog that has already closed.
//!
//! ## ★★ What is deliberately NOT here
//!
//! **Reading an existing button's action** — because that is not a draft.
//! `pdfcer-core` `28b982c` could write one and not read one back, which is why
//! this module served only the placement path; the reader landed the same day
//! (`request_a_buttons_action_can_be_written_and_not_read.md`, answered by
//! `Pass 212.0`) and lives in `panels::forms::button`, which converts INTO this
//! type rather than the other way round.
//!
//! ⇒ **That was true for four hours.** `Pass 212.0` shipped
//! `EditSession::button_action` on 2026-09-01, so this model now serves two
//! surfaces: the placement dialog, where the current action is known to be
//! *none* because the button does not exist yet, and `panels::forms::button`,
//! where it is read from the document.
//!
//! ★ The engine shipped **four** states where three were asked for, and the
//! fourth is the one that makes the row honest. `panels::forms::button`'s
//! header carries that argument; this module is unchanged by it, because a
//! DRAFT has no fourth state — an operator is always editing something this
//! shell can express, or it would not have offered to edit it.
//!
//! ## `/JavaScript` and `/Launch`
//!
//! Absent, permanently, and not as an omission: `pdfcer-core` refuses both by
//! name and this shell agrees with the refusal. There is no *Run a script* row
//! to grey — R9's rule is that an unavailable capability renders **nothing**,
//! and a greyed *Run a script* would advertise a capability pdfcer has decided
//! not to have.

use pdfcer_core::edit::{ButtonAction, NamedAction, PageView, ResetScope};

/// Which of the six things a button may be set to do.
///
/// ★ `Nothing` is first and is the default, because that is what
/// `add_push_button` authors and this shell does not change a document's
/// meaning by having a dialog open. Choosing anything else is a deliberate act.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonDoesKind {
    /// No `/A` is written at all.
    #[default]
    Nothing,
    /// §12.7.5.3 — return the form's fields to their defaults.
    ResetForm,
    /// §12.6.4.2 — jump to a page of **this** document.
    GoToPage,
    /// §12.6.4.11 — next / previous / first / last page.
    Named,
    /// §12.6.4.10 — hide or show named fields.
    ShowHide,
    /// §7.11 — open a web address. pdfcer never follows one.
    Uri,
    /// §12.7.5.2 — send the form's data to an address the author chose.
    SubmitForm,
}

impl ButtonDoesKind {
    /// Every kind, in the order the chooser offers them.
    ///
    /// ★★ Ordered by **reach**, not by the standard's section numbers: the four
    /// that cannot leave the document come first, then the two that write an
    /// address into the file. An operator scanning the list meets the safe ones
    /// first and the two that need a sentence of disclosure last, which is the
    /// order a chooser should be read in.
    pub const ALL: [Self; 7] = [
        Self::Nothing,
        Self::ResetForm,
        Self::GoToPage,
        Self::Named,
        Self::ShowHide,
        Self::Uri,
        Self::SubmitForm,
    ];

    /// Whether choosing this writes an address that some other program may act
    /// on.
    ///
    /// The predicate the dialog uses to decide whether a disclosure block is
    /// drawn. **Not** a predicate about danger — a `Uri` is inert until a human
    /// clicks it in a viewer — but about whether the file gains a statement
    /// pointing off the machine, which is the thing an operator cannot see by
    /// looking at the page.
    #[must_use]
    pub const fn reaches_outside(self) -> bool {
        matches!(self, Self::Uri | Self::SubmitForm)
    }
}

/// A push button's action **as the operator is editing it**.
///
/// Every parameter for every kind, held at once. See the module header for why
/// this is a struct with a discriminant rather than an enum with payloads: a
/// dialog that lost the page number when the chooser moved to *Reset* and back
/// would be punishing the operator for looking.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ButtonDoes {
    /// Which kind of action.
    pub kind: ButtonDoesKind,
    /// **Go to a page** — the 1-based page number as typed, so an empty box and
    /// a `0` are distinguishable and both are refusals rather than silent
    /// clamps.
    pub page_number: String,
    /// **Go to a page** — where on the page to land.
    pub view: PageViewChoice,
    /// **Next/previous/first/last** — which one.
    pub named: NamedChoice,
    /// **Show or hide** — the field names, one per line.
    ///
    /// One string rather than a `Vec`, for the same reason a choice field's
    /// options are: it is what the operator edits.
    ///
    /// ★★ These must be **terminal** field names. Table 210 states nothing
    /// about descendant expansion — the phrase *"all descendants of the
    /// specified fields"* appears twice per edition of ISO 32000 and never on
    /// this row — so a grouping name is a button that either hides a subtree or
    /// hides nothing, depending on the reader. `pdfcer-core` refuses one by name
    /// (`ButtonActionHideTargetNotTerminal`) and this shell does not try to
    /// guess around it.
    pub targets: String,
    /// **Show or hide** — `true` hides the named fields, `false` shows them.
    ///
    /// ★ Not a toggle, and the standard chose that: Table 210's action works
    /// *"by setting or clearing their `Hidden` flags"*, so a second press does
    /// not reverse it. A genuinely toggling button needs JavaScript and is
    /// therefore out of pdfcer's scope rather than merely unbuilt.
    pub hide: bool,
    /// **Open a web link** / **Submit** — the address, exactly as typed.
    pub url: String,
}

/// Where a *Go to a page* action lands on the page it reaches.
///
/// A shell mirror of `pdfcer_core::edit::PageView`, for the same reason
/// [`ButtonDoes`] mirrors `ButtonAction`: this one is `Default` and `Copy` and
/// sits in a draft that is cloned every frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PageViewChoice {
    /// `[page /Fit]` — the whole page in the window.
    ///
    /// The default because it is the only one of the three whose result does
    /// not depend on the window's shape or on the zoom the reader happened to
    /// be at.
    #[default]
    WholePage,
    /// `[page /FitH top]` — the page's full width.
    FullWidth,
    /// `[page /XYZ left top null]` — the top-left corner at the current zoom.
    TopLeft,
}

impl PageViewChoice {
    /// Every choice, in the order the chooser offers them.
    pub const ALL: [Self; 3] = [Self::WholePage, Self::FullWidth, Self::TopLeft];

    /// The engine's value.
    #[must_use]
    pub const fn to_core(self) -> PageView {
        match self {
            Self::WholePage => PageView::WholePage,
            Self::FullWidth => PageView::FullWidth,
            Self::TopLeft => PageView::TopLeft,
        }
    }
}

/// Which of the four reader-predefined navigation actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NamedChoice {
    /// `/NextPage`.
    #[default]
    NextPage,
    /// `/PrevPage`.
    PrevPage,
    /// `/FirstPage`.
    FirstPage,
    /// `/LastPage`.
    LastPage,
}

impl NamedChoice {
    /// Every choice, in reading order.
    pub const ALL: [Self; 4] = [
        Self::NextPage,
        Self::PrevPage,
        Self::FirstPage,
        Self::LastPage,
    ];

    /// The engine's value.
    #[must_use]
    pub const fn to_core(self) -> NamedAction {
        match self {
            Self::NextPage => NamedAction::NextPage,
            Self::PrevPage => NamedAction::PrevPage,
            Self::FirstPage => NamedAction::FirstPage,
            Self::LastPage => NamedAction::LastPage,
        }
    }
}

/// Why a draft action cannot be authored yet.
///
/// ★ Returned rather than rendered, so the caller decides where the sentence
/// goes — the dialog puts it under the chooser and greys Add; a driven check
/// reads the discriminant. A function that drew the message itself would make
/// the condition untestable except by screenshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionBlocker {
    /// *Go to a page* with an empty, non-numeric or zero page box.
    ///
    /// Zero is included deliberately: the engine takes a 0-based index and the
    /// operator types a 1-based number, so `0` is the one value that would be
    /// silently reinterpreted rather than refused.
    PageNumberMissing,
    /// *Show or hide* with no field named.
    NoTargets,
    /// *Open a web link* or *Submit* with an empty address.
    UrlMissing,
    /// An address `pdfcer-core` will refuse because it cannot state it
    /// unambiguously: relative, or carrying a non-ASCII character.
    ///
    /// ★★ Checked here as well as in the engine, and that is not duplication
    /// for its own sake: the engine's refusal arrives when the action drains,
    /// which is **after the dialog has closed**. An operator who typed a
    /// relative URL would see the dialog accept it and a status line contradict
    /// it a frame later, with the box that held the mistake already gone.
    ///
    /// The engine remains the authority. This is the same question asked early
    /// enough to be answerable.
    UrlNotStatable,
}

impl ButtonDoes {
    /// Why this draft cannot be authored, or `None`.
    ///
    /// Checks only what can be checked without the document. Everything that
    /// needs one — does that page exist, is that field name terminal, is it
    /// even a push button — is the engine's, and its refusals are reported when
    /// they arrive.
    #[must_use]
    pub fn blocker(&self) -> Option<ActionBlocker> {
        match self.kind {
            ButtonDoesKind::Nothing | ButtonDoesKind::ResetForm | ButtonDoesKind::Named => None,
            ButtonDoesKind::GoToPage => {
                if self.page_number.trim().parse::<usize>().unwrap_or(0) == 0 {
                    Some(ActionBlocker::PageNumberMissing)
                } else {
                    None
                }
            }
            ButtonDoesKind::ShowHide => {
                if self.target_names().is_empty() {
                    Some(ActionBlocker::NoTargets)
                } else {
                    None
                }
            }
            ButtonDoesKind::Uri | ButtonDoesKind::SubmitForm => url_blocker(&self.url),
        }
    }

    /// The field names for a *show or hide*, blanks discarded and ends trimmed.
    ///
    /// The same treatment a choice field's options get, for the same reason: a
    /// text box has a trailing newline after the last line typed, and without
    /// discarding empties every button would carry a final target that is the
    /// empty string — which the engine would refuse as a field that does not
    /// exist, naming a name the operator never typed.
    #[must_use]
    pub fn target_names(&self) -> Vec<String> {
        self.targets
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    }

    /// The engine's action, or `None` for *Nothing* — which is what
    /// `set_button_action` takes to clear one.
    ///
    /// # Returns `None` for two different reasons, and the caller must not care
    ///
    /// *Nothing* and *a draft that [`Self::blocker`] refuses* both answer
    /// `None`. That is safe **only** because the one caller checks `blocker`
    /// first and does not reach here otherwise — which the dialog enforces by
    /// greying Add. Stated here because a second caller written later would not
    /// know, and the failure would be a button silently authored inert.
    #[must_use]
    pub fn to_core(&self) -> Option<ButtonAction> {
        if self.blocker().is_some() {
            return None;
        }
        match self.kind {
            ButtonDoesKind::Nothing => None,
            ButtonDoesKind::ResetForm => Some(ButtonAction::ResetForm {
                // ★ `All`, and only `All`. The panel offers no field picker for
                // a reset because the reset it can preview is the whole-form
                // one — the same reasoning `FormEdit::Reset` carries. `Only`
                // and `Except` exist in the engine and are reachable from the
                // CLI; offering them here without a preview would be a control
                // whose effect the operator cannot see before pressing.
                scope: ResetScope::All,
            }),
            ButtonDoesKind::GoToPage => {
                let n = self.page_number.trim().parse::<usize>().ok()?;
                Some(ButtonAction::GoToPage {
                    // ★ 1-based in the box, 0-based in the file. `blocker`
                    // refuses `0` precisely so this subtraction cannot wrap.
                    page_index: n - 1,
                    view: self.view.to_core(),
                })
            }
            ButtonDoesKind::Named => Some(ButtonAction::Named(self.named.to_core())),
            ButtonDoesKind::ShowHide => Some(ButtonAction::SetHidden {
                targets: self.target_names(),
                hidden: self.hide,
            }),
            ButtonDoesKind::Uri => Some(ButtonAction::Uri {
                uri: self.url.trim().to_owned(),
            }),
            ButtonDoesKind::SubmitForm => {
                // `SubmitSpec` is `#[non_exhaustive]`: built by `new`, then
                // assigned. The default format is FDF with `/Flags 0`, which is
                // the baseline the engine's disclosure describes.
                let spec = pdfcer_core::edit::SubmitSpec::new(self.url.trim());
                Some(ButtonAction::SubmitForm(spec))
            }
        }
    }
}

/// Whether an address is one pdfcer can state unambiguously.
///
/// # ★★ The two refusals are the engine's, restated, and each has a reason
/// that is about READERS rather than about safety
///
/// - **Relative** — §7.11.2.2 resolves it against the document's own location,
///   and ISO issue #256 records readers disagreeing about §12.6.4.8's `/Base`
///   concatenation badly enough that *"only the host portion gets used"* in
///   some of them. Two readers, two destinations, one file.
/// - **Non-ASCII** — §7.11.5 requires RFC 1738 encoding; ISO 32000-2 then types
///   `/URI` as an `ASCII string` in one column and *"encoded in UTF-8"* in the
///   next.
///
/// ★ **`http://` is allowed and is not a refusal.** Destination policy is open
/// by operator ruling — *"we'll allow a submit to send filled data wherever the
/// document's author said"* — and `https` appears **zero times** in ISO 32000-1.
/// Blocking it would be pdfcer inventing a conformance requirement. It is
/// disclosed instead, which is the honest half.
fn url_blocker(url: &str) -> Option<ActionBlocker> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Some(ActionBlocker::UrlMissing);
    }
    if !trimmed.is_ascii() {
        return Some(ActionBlocker::UrlNotStatable);
    }
    // "Absolute" here means the §7.11.2.2 sense: it carries a scheme. Checked
    // by the shape `scheme:` rather than against a list of schemes, because no
    // scheme is refused and a list would become one.
    let has_scheme = trimmed.split_once(':').is_some_and(|(scheme, _)| {
        !scheme.is_empty()
            && scheme.starts_with(|c: char| c.is_ascii_alphabetic())
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    });
    if has_scheme {
        None
    } else {
        Some(ActionBlocker::UrlNotStatable)
    }
}

/// Whether the address is unencrypted, for the disclosure line.
///
/// ★ A **statement**, never a refusal. See [`url_blocker`] for why: the standard
/// states no TLS rule, and pdfcer does not invent one. What it does is say so.
#[must_use]
pub fn url_is_unencrypted(url: &str) -> bool {
    let t = url.trim();
    !t.is_empty() && !t.to_ascii_lowercase().starts_with("https:")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **THE TRIPWIRE FIRED, AND THIS IS WHAT IT LEFT BEHIND.**
    ///
    /// It read *"a tripwire that names its own deletion"* and asserted that
    /// `pdfcer-core` could write a button's action and not read one — so this
    /// module served the PLACEMENT path only, and the Forms panel had no row
    /// for a button already in the document.
    ///
    /// `Pass 212.0` shipped `EditSession::button_action` on 2026-09-01, hours
    /// after the request, and the four steps that test named were carried out:
    /// the test deleted, the row added (`panels::forms::button`),
    /// `ButtonActionState::Foreign` consumed so a script is named rather than
    /// silently offered for replacement, and the request closed.
    ///
    /// ★★ Kept as a headstone rather than deleted outright, because the shape
    /// paid out for the fifth time in three days and the count is the
    /// argument: a test that ASSERTS a limitation goes red on the first build
    /// after `cargo update` that lifts it, and names the code to change while
    /// somebody is still looking.
    ///
    /// What survives as an assertion is the half that is still true: `Nothing`
    /// clears, and every other kind writes.
    #[test]
    fn the_reader_landed_and_this_is_what_was_owed() {
        let does = ButtonDoes {
            kind: ButtonDoesKind::ResetForm,
            ..ButtonDoes::default()
        };
        assert!(does.to_core().is_some());
        assert!(
            ButtonDoes::default().to_core().is_none(),
            "Nothing must clear rather than write"
        );
    }

    #[test]
    fn nothing_is_the_default_and_clears() {
        let does = ButtonDoes::default();
        assert_eq!(does.kind, ButtonDoesKind::Nothing);
        assert!(does.blocker().is_none());
        assert!(does.to_core().is_none(), "Nothing must clear, not write");
    }

    #[test]
    fn a_page_number_is_one_based_in_the_box_and_zero_based_in_the_file() {
        let does = ButtonDoes {
            kind: ButtonDoesKind::GoToPage,
            page_number: "3".to_owned(),
            ..ButtonDoes::default()
        };
        assert!(matches!(
            does.to_core(),
            Some(ButtonAction::GoToPage { page_index: 2, .. })
        ));
    }

    /// ★ Zero is refused rather than clamped, and this is the test that says
    /// why: `page_index: n - 1` would wrap, and a clamp to page 1 would author
    /// a destination the operator did not type.
    #[test]
    fn page_zero_is_refused_rather_than_clamped() {
        for typed in ["", "0", "  ", "x", "-1"] {
            let does = ButtonDoes {
                kind: ButtonDoesKind::GoToPage,
                page_number: typed.to_owned(),
                ..ButtonDoes::default()
            };
            assert_eq!(
                does.blocker(),
                Some(ActionBlocker::PageNumberMissing),
                "{typed:?} must be refused"
            );
            assert!(does.to_core().is_none());
        }
    }

    #[test]
    fn a_relative_or_non_ascii_url_is_refused_before_the_dialog_closes() {
        for typed in ["forms/collect", "/collect", "example.com"] {
            let does = ButtonDoes {
                kind: ButtonDoesKind::Uri,
                url: typed.to_owned(),
                ..ButtonDoes::default()
            };
            assert_eq!(
                does.blocker(),
                Some(ActionBlocker::UrlNotStatable),
                "{typed}"
            );
        }
        let does = ButtonDoes {
            kind: ButtonDoesKind::Uri,
            url: "https://exämple.com/help".to_owned(),
            ..ButtonDoes::default()
        };
        assert_eq!(does.blocker(), Some(ActionBlocker::UrlNotStatable));
    }

    /// ★★ `http://` is **allowed**. If this test ever inverts, someone has made
    /// pdfcer enforce a rule the standard does not state — see [`url_blocker`].
    #[test]
    fn plain_http_is_allowed_and_disclosed_rather_than_blocked() {
        let does = ButtonDoes {
            kind: ButtonDoesKind::SubmitForm,
            url: "http://forms.example.com:8080/collect".to_owned(),
            ..ButtonDoes::default()
        };
        assert!(does.blocker().is_none(), "no scheme is refused");
        assert!(url_is_unencrypted(&does.url), "and it is said");
        assert!(!url_is_unencrypted("https://forms.example.com/collect"));
    }

    #[test]
    fn blank_target_lines_never_become_a_field_named_nothing() {
        let does = ButtonDoes {
            kind: ButtonDoesKind::ShowHide,
            targets: "Section2\n\n  Section3  \n".to_owned(),
            hide: true,
            ..ButtonDoes::default()
        };
        assert_eq!(does.target_names(), vec!["Section2", "Section3"]);
        assert!(does.blocker().is_none());
        let empty = ButtonDoes {
            kind: ButtonDoesKind::ShowHide,
            targets: "\n  \n".to_owned(),
            ..ButtonDoes::default()
        };
        assert_eq!(empty.blocker(), Some(ActionBlocker::NoTargets));
    }

    /// ★ Every kind that reaches outside the document must say so, and no kind
    /// that cannot may claim to. This is the predicate a disclosure block is
    /// drawn on, so getting it wrong in either direction is a rule-4 defect:
    /// too narrow hides a statement the operator cannot otherwise see, too wide
    /// trains them to ignore it.
    #[test]
    fn only_the_two_addressed_kinds_reach_outside() {
        use ButtonDoesKind as K;
        for kind in K::ALL {
            assert_eq!(
                kind.reaches_outside(),
                matches!(kind, K::Uri | K::SubmitForm),
                "{kind:?}"
            );
        }
    }
}
