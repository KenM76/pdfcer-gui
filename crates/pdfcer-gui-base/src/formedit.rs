//! # `formedit` — a form-filling edit the Forms panel raises

/// One thing an operator asked the Forms panel to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormEdit {
    /// Write `value` into the text field named `field`.
    ///
    /// Raised when a text row loses focus with a draft that differs from what
    /// the document holds — see `pdfcer_gui::panels::forms::rows::commit` for why
    /// both halves of that condition are needed.
    FillText {
        /// The field's fully-qualified name (§12.7.3.2).
        field: String,
        /// The plain text to store.
        value: String,
    },
    /// Write `value` into the rich-text field named `field`, **discarding its
    /// `/RV`**.
    ///
    /// A separate variant from [`Self::FillText`] rather than a flag, because
    /// it is a separate act: it destroys formatting, it is offered behind its
    /// own button with its own tooltip, and it calls a different verb
    /// (`fill_text_field_downgrading_rich_text`). A boolean would let a
    /// caller reach the destructive path by passing `true` to something whose
    /// name says "fill".
    ///
    /// `value` is the field's **current plain text**, unchanged — the button
    /// converts the field, it does not also retype it. The operator types
    /// afterwards, into the ordinary text row the conversion produces.
    ConvertRichTextToPlain {
        /// The field's fully-qualified name.
        field: String,
        /// The plain text to keep — the field's existing `/V`.
        value: String,
    },
    /// Select `state` on the check box or radio group named `field`.
    ///
    /// One variant for both, because it is one verb: a check box is a
    /// two-state button and a radio group is an n-state one, and
    /// `EditSession::set_button_state` takes the state name either way.
    ///
    /// `state` is `Off` to clear — the §12.7.4.2.3 name for the cleared state
    /// of every button, whatever its ON state happens to be called. Core
    /// accepts `Off` unconditionally and refuses any other name no widget
    /// defines, which is why the panel only ever offers names it read off the
    /// widgets.
    SetButtonState {
        /// The field's fully-qualified name.
        field: String,
        /// The on-state name, or `Off`.
        state: String,
    },
    /// Select `values` in the choice field named `field`.
    ///
    /// A `Vec` even for a single-select combo, because
    /// `EditSession::set_choice_value` takes a slice and a single-element
    /// slice is the honest way to say "one selection". A separate scalar
    /// variant would be a second spelling of the same command.
    ///
    /// The strings are **export** values where `/Opt` provides them
    /// (§12.7.4.4's `[export display]` pairs), because that is what `/V`
    /// stores.
    SetChoice {
        /// The field's fully-qualified name.
        field: String,
        /// The selections, in the order `/Opt` lists them.
        values: Vec<String>,
    },
    /// Write every value in a recompute plan the operator has just reviewed.
    ///
    /// # The plan travels with the action rather than being recomputed
    ///
    /// `pdfcer_gui::panels::forms::edit::apply` could call `form_script::recompute::plan` itself and get the
    /// same answer — the action is applied in the same frame that raised it,
    /// against the same document. It does not, and the reason is rule 4: what
    /// the operator consented to is **the list of values that was on screen**,
    /// and an action is a complete statement of an operator's intent. Carrying
    /// the list makes "what did they agree to?" answerable from the action
    /// alone; recomputing it makes the answer depend on when the question is
    /// asked.
    ///
    /// **This is N undo entries, not one.** `pdfcer-core` has no batch verb —
    /// applying a plan is a loop the shell writes — so each pair below becomes
    /// its own `fill_text_field` command. Disclosed in
    /// [`crate::text::forms::recompute_apply_tooltip`], whose own doc comment
    /// records that the salvaged wording claimed otherwise.
    Recompute {
        /// `(fully-qualified name, proposed value)`, in evaluation order.
        changes: Vec<(String, String)>,
    },
    /// Return every eligible field to its `/DV`, or empty it (§12.7.5.3).
    ///
    /// No operand list: the panel offers only the whole-form reset, because
    /// the preview it shows above the button is the whole-form preview and a
    /// per-field reset control would need a per-field preview beside it to
    /// mean anything.
    Reset,
    /// Draw every field's current value into the document and clear
    /// `/NeedAppearances`.
    ///
    /// Not authoring, and the distinction is worth stating because this is the
    /// one variant here that does not change a **value**. It changes how the
    /// values already stored are *drawn*, which is the operator-facing answer
    /// to [`crate::text::forms::forms_need_appearances_note`] and the
    /// precondition for [`Self::Flatten`] keeping anything.
    RegenerateAppearances,
    /// Burn every field's appearance into page content and remove the form.
    ///
    /// See this module's header for why it carries a tooltip rather than a
    /// blocking confirmation.
    Flatten,
}
impl FormEdit {
    /// A short, stable name for the diagnostic trace.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::FillText { .. } => "form-fill-text",
            Self::ConvertRichTextToPlain { .. } => "form-convert-rich-text",
            Self::SetButtonState { .. } => "form-set-button-state",
            Self::SetChoice { .. } => "form-set-choice",
            Self::Recompute { .. } => "form-recompute",
            Self::Reset => "form-reset",
            Self::RegenerateAppearances => "form-regenerate-appearances",
            Self::Flatten => "form-flatten",
        }
    }
}
