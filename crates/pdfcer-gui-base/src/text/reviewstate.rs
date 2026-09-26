//! # `text::reviewstate` — every word the review-status control says
//!
//! One module for one subject: `/State` and `/StateModel` (§12.5.6.3, Table
//! 171 — 2.0's Table 174), read by `pdfcer-core` `Pass 253.1` and authored by
//! [`pdfcer_core::edit::EditSession::add_review_state`]. It covers the status
//! line on a comment row, the control that records one, the status chooser
//! beside the existing filter, and the two sentences the status row says after
//! the edit lands.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/reviewstate.md`.

/// The seven states pdfcer AUTHORS, as the operator reads them.
#[must_use]
pub fn state_name(state: pdfcer_core::edit::ReviewState) -> &'static str {
    match state {
        pdfcer_core::edit::ReviewState::None => state_none(),
        other => other.as_str(),
    }
}

/// `Review`-model `None`, disambiguated from *no status at all*.
#[must_use]
pub fn state_none() -> &'static str {
    "None (status withdrawn)"
}

/// **A `/State` in a model pdfcer authors, whose value is not in that model's
/// vocabulary** — shown verbatim, never normalised.
#[must_use]
pub fn state_unmodelled(state: &str, model: &str) -> String {
    format!("{state} — a status pdfcer does not recognise, in the {model} model")
}

/// **A `/StateModel` pdfcer will not author** — both values named, and nothing
/// offered in that vocabulary.
#[must_use]
pub fn state_foreign(state: &str, model: &str) -> String {
    format!("{state} — in {model}, a review vocabulary pdfcer does not author")
}

/// A `/State` with **no** `/StateModel` — the one combination Table 171 forbids.
#[must_use]
pub fn state_without_model(state: &str) -> String {
    format!("{state} — recorded without saying which review vocabulary it is from")
}

/// One reviewer's current status, on a comment row.
///
/// `who` is `/T`, already trimmed and known non-empty by the caller;
/// `status` is one of the four families above.
#[must_use]
pub fn row_status_by(who: &str, status: &str) -> String {
    format!("{who}: {status}")
}

/// The same line for a status whose state annotation carries **no** `/T`.
#[must_use]
pub fn row_status_unsigned(status: &str) -> String {
    format!("{status} — recorded without a name, so it stands on its own")
}

/// **How much history stands behind the status being shown**, when there is
/// more than one.
#[must_use]
pub fn row_status_history(depth: usize) -> String {
    format!("{depth} statuses recorded, most recent shown")
}

/// The line a row carries when it **has no status at all**.
#[must_use]
pub fn row_status_none() -> &'static str {
    "No status recorded"
}

/// **The row that IS a status**, named so an empty comment is not a
/// mystery.
#[must_use]
pub fn row_is_a_status(status: &str) -> String {
    format!("This row is a review status — {status} — recorded on another comment")
}

/// The control that records a status. **Record**, never *Set*.
#[must_use]
pub fn record_label() -> &'static str {
    "Record status"
}

/// The chooser's resting text before a status is picked.
#[must_use]
pub fn record_placeholder() -> &'static str {
    "Record status…"
}

/// The tooltip on the control, carrying the three things the operator cannot
/// see from the panel.
#[must_use]
pub fn record_tooltip() -> &'static str {
    "Adds a separate status annotation beside this comment, signed with your \
     name from Settings. Earlier statuses are kept — a comment's status is a \
     history, not a field."
}

/// The note on a comment whose only status is in a **foreign** model.
#[must_use]
pub fn record_other_model_note() -> &'static str {
    "Recording a status here adds one in the Review vocabulary, beside the \
     file's own"
}

/// The status chooser's label, beside *Author*, *Type* and *Order*.
#[must_use]
pub fn filter_status_label() -> &'static str {
    "Status"
}

/// The chooser's "no filter" entry.
#[must_use]
pub fn filter_status_any() -> &'static str {
    "Any status"
}

/// The entry that keeps only comments **nobody has reviewed**.
#[must_use]
pub fn filter_status_unrecorded() -> &'static str {
    "No status recorded"
}

/// **What was written**, on the status row, after the edit lands.
#[must_use]
pub fn status_recorded(state: &str, depth: usize) -> String {
    if depth <= 1 {
        format!("Status recorded: {state}. The comment itself is unchanged")
    } else {
        format!(
            "Status recorded: {state}. This is your {depth} status on this \
             comment — the earlier ones are kept"
        )
    }
}

/// The second sentence, when the operator has **no name set**.
#[must_use]
pub fn status_recorded_unsigned() -> &'static str {
    "Recorded without a name. Set one in Settings > Comments — a later status \
     under a name will start a separate history rather than continue this one"
}
