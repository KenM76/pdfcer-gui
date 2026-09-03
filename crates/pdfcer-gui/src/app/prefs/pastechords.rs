//! # `app::prefs::pastechords` — which chord means which paste
//!
//! One preference, two values, and the whole of its subject is that **neither
//! order is obviously right**.

/// **Which chord pastes a form field as a NEW field, and which as a DUPLICATE.**
///
/// `OPERATOR_REQUESTS.md` **O58**. Ken, 2026-08-29: *"let's make it an option to
/// have it swap to match Acrobat or work the way we have it now."*
///
/// # ★★★ Why this is a setting rather than a decision
///
/// Copying a form field has two legitimate meanings: **a new, independent
/// field**, and **another box for the same field** that fills in step with the
/// original. `pdfcer-core` refuses to guess between them, and this shell offers
/// both on two chords rather than asking in a dialog.
///
/// **Acrobat assigns them the other way round.** Its plain Copy/Paste is the
/// *linked* one — paste a field, leave its name alone, and you get a second
/// widget of the same field — and it has **no paste-as-independent chord at
/// all**: independence comes only from its bulk commands (Place Multiple
/// Fields, Create Multiple Copies), which auto-name and so produce separate
/// fields. Sourced, not assumed:
/// `Acrobat_Features/forms__field_copy_paste_and_duplication.md`.
///
/// Both orders have a real argument, which is exactly what makes it a setting:
///
/// - **[`PdfcerOrder`](Self::PdfcerOrder)** puts the *common intent* on the
///   *common chord*. Copying a title-block field down a column almost always
///   wants independent fields — and Acrobat's own linking default is a
///   documented, unresolved point of user friction, with a standing request for
///   an *"option to unlink form fields when copying"* and no remedy but
///   renaming each duplicate by hand.
/// - **[`AcrobatOrder`](Self::AcrobatOrder)** matches muscle memory. An operator
///   who spends the day in Acrobat and reaches for `Ctrl+V` expecting a linked
///   field gets one, and does not have to hold a second rule in their head for
///   one program.
///
/// # ★★ It swaps the CHORDS, never what a command means
///
/// The two commands keep their meanings and their labels for ever: `edit.paste`
/// is always *"paste as a new field"*, `edit.paste_duplicate` is always *"paste
/// as another box for the same field"*. Only the keys move.
///
/// The alternative — swapping what the *commands do* — was rejected because it
/// makes the labels lie. A ribbon button reading **Paste as duplicate** would
/// paste a new field, and no tooltip rescues a control whose name is wrong.
/// Moving the binding instead means the ribbon, the context menu, the shortcuts
/// dialog and the keyboard agree **by construction**, because every one of them
/// reads the same keymap.
///
/// ⇒ Applied by [`crate::shell::manifest::apply_paste_chords`], which rewrites
/// two entries of the shell's keymap. That is the mechanism the framework
/// already has: `SHELL_FRAMEWORK.md`'s central claim is that the keymap is a
/// **manifest**, not code — so an operator preference about keys is a data edit
/// rather than a branch, and this preference is its first real customer.
///
/// # ★ Both chords always exist, whichever way round they are
///
/// This never takes a capability away — it exchanges two keys. Both commands
/// stay on the ribbon and in the context menu under their own names, so an
/// operator who cannot remember which order they chose can always read it off
/// the Edit tab rather than discovering it by pasting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PasteChords {
    /// `Ctrl+V` pastes a **new** field; `Ctrl+Shift+V` pastes a **duplicate**.
    ///
    /// The default, and the operator's original ruling of 2026-08-29.
    #[default]
    PdfcerOrder,
    /// `Ctrl+V` pastes a **duplicate**; `Ctrl+Shift+V` pastes a **new** field.
    ///
    /// Acrobat's own assignment.
    AcrobatOrder,
}

impl PasteChords {
    /// Both, in the order the settings pane offers them.
    ///
    /// pdfcer's own first, because it is the default and because a radio group
    /// whose first entry is not the default reads as though the default were a
    /// fallback.
    pub const ALL: &'static [Self] = &[Self::PdfcerOrder, Self::AcrobatOrder];

    /// **A harness override, read from the environment.**
    ///
    /// `PDFCER_DIAG_PASTE_CHORDS=acrobat` (or `new_field_first`) forces the
    /// order for one run, ahead of whatever is in the preferences file.
    ///
    /// # ★★★ Why a test seam exists here at all
    ///
    /// Because the alternative is worse in two directions, and one of them is
    /// destructive.
    ///
    /// A driven check has to prove that changing this setting changes what
    /// `Ctrl+V` **does** — that is the whole claim, and a unit test cannot
    /// reach it: the keymap, the chord translation and the dispatcher all sit
    /// between the preference and the keystroke. To change the setting a
    /// harness could either **write the preferences file**, which is the
    /// operator's own and would rewrite his choices on his own machine, or
    /// **drive the Settings window**, which makes a check about pasting depend
    /// on a dialog's layout and would fail for reasons that have nothing to do
    /// with its subject.
    ///
    /// ⇒ So the seam is deliberate and is the same shape as
    /// `PDFCER_DIAG_FORM_ACCEPT`: it changes no behaviour a keyless run can
    /// observe, it is read exactly once at start-up, and it never writes
    /// anything. `RAG: a_driven_check_that_mutates_persisted_state_must_normalise_at_the_start`
    /// is the entry this avoids needing.
    ///
    /// ★ An unrecognised value is ignored rather than refused. The variable is
    /// a harness affordance, and a typo in it should degrade to "the operator's
    /// own setting" rather than to a start-up failure on a machine where
    /// somebody exported it once and forgot.
    #[must_use]
    pub fn from_environment() -> Option<Self> {
        // ui-text-exempt: an environment variable name, never displayed.
        std::env::var("PDFCER_DIAG_PASTE_CHORDS")
            .ok()
            .and_then(|v| Self::from_key(v.trim()))
    }

    /// The token this is written under in the preferences file.
    ///
    /// Named for the **behaviour** rather than for the keys, deliberately. A
    /// token of `ctrl_v_is_new` would be a file format that has to change if a
    /// future operator rebinds either chord to something else entirely, and the
    /// preference is about *which paste is the plain one*, not about `V`.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            // ui-text-exempt: a file token, never displayed.
            Self::PdfcerOrder => "new_field_first",
            // ui-text-exempt: a file token, never displayed.
            Self::AcrobatOrder => "acrobat",
        }
    }

    /// Read a token back, or `None` if it names nothing.
    ///
    /// Derived from [`Self::ALL`] and [`Self::key`] so it cannot drift from the
    /// writer — the same shape `WheelPaging::from_key` uses.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|o| o.key() == key)
    }

    /// The chord that reaches `edit.paste` — a **new** field — under this order.
    #[must_use]
    pub const fn new_field_chord(self) -> &'static str {
        match self {
            // ui-text-exempt: keymap chord spellings, matched against the manifest.
            Self::PdfcerOrder => "Ctrl+V",
            Self::AcrobatOrder => "Ctrl+Shift+V",
        }
    }

    /// The chord that reaches `edit.paste_duplicate` under this order.
    #[must_use]
    pub const fn duplicate_chord(self) -> &'static str {
        match self {
            // ui-text-exempt: keymap chord spellings, matched against the manifest.
            Self::PdfcerOrder => "Ctrl+Shift+V",
            Self::AcrobatOrder => "Ctrl+V",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ The two orders must be an EXCHANGE, not two independent choices.
    ///
    /// The failure this forbids is a build where both commands end up on the
    /// same chord — one silently unreachable from the keyboard, with the
    /// ribbon still showing both and the shortcuts dialog still listing a key
    /// that reaches the other one. That is invisible until an operator presses
    /// it, which is the whole class this project keeps meeting.
    #[test]
    fn every_order_binds_the_two_commands_to_two_different_chords() {
        for order in PasteChords::ALL {
            assert_ne!(
                order.new_field_chord(),
                order.duplicate_chord(),
                "{order:?} puts both pastes on one chord, so one of them is unreachable"
            );
        }
    }

    /// The two orders are each other's mirror, and nothing else.
    ///
    /// ★ Asserted as a property rather than by restating the four literals,
    /// because restating them is how a table and its test come to agree with
    /// each other and disagree with the operator.
    #[test]
    fn the_acrobat_order_is_exactly_the_pdfcer_order_reversed() {
        assert_eq!(
            PasteChords::PdfcerOrder.new_field_chord(),
            PasteChords::AcrobatOrder.duplicate_chord()
        );
        assert_eq!(
            PasteChords::PdfcerOrder.duplicate_chord(),
            PasteChords::AcrobatOrder.new_field_chord()
        );
    }

    /// Every value survives the preferences file.
    ///
    /// ★ Over `ALL`, not over two literals: a third order added later is
    /// covered without anybody remembering to extend this.
    #[test]
    fn every_order_round_trips_through_its_file_token() {
        for order in PasteChords::ALL {
            assert_eq!(
                PasteChords::from_key(order.key()),
                Some(*order),
                "{order:?} does not survive a save and reload"
            );
        }
        assert_eq!(PasteChords::from_key("nonsense"), None);
    }

    /// The default is the operator's own ruling, not Acrobat's.
    ///
    /// He chose the split before the divergence was found, was told about it,
    /// and asked for an option rather than a swap — so the default stays his.
    #[test]
    fn the_default_is_the_operators_ruling() {
        assert_eq!(PasteChords::default(), PasteChords::PdfcerOrder);
        assert_eq!(PasteChords::default().new_field_chord(), "Ctrl+V");
    }
}
