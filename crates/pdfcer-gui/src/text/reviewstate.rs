//! # `text::reviewstate` — every word the review-status control says
//!
//! One module for one subject: `/State` and `/StateModel` (§12.5.6.3, Table
//! 171 — 2.0's Table 174), read by `pdfcer-core` `Pass 253.1` and authored by
//! [`pdfcer_core::edit::EditSession::add_review_state`]. It covers the status
//! line on a comment row, the control that records one, the status chooser
//! beside the existing filter, and the two sentences the status row says after
//! the edit lands.
//!
//! It is deliberately **not** in [`crate::text::panels::comments`], even
//! though most of these strings are drawn by that panel. The subject here is a
//! *vocabulary the standard defines and the engine refuses to interpret*, and
//! the wording decisions below are all consequences of that one fact rather
//! than of anything about panels. Keeping them together is what makes the next
//! reader able to check them against §12.5.6.3 in one pass.
//!
//! ## ★★★ THE FACT THAT DECIDES ALMOST EVERY STRING BELOW
//!
//! **A review status is APPENDED, not set.** `add_review_state`'s own doc
//! comment (`edit.rs:26804`) states it twice with the standard's `shall`:
//!
//! > *"THE STATUS IS NOT WRITTEN ONTO THE ANNOTATION IT DESCRIBES … §12.5.6.3
//! > puts it on a **separate** `/Text` annotation that points at the reviewed
//! > one through `/IRT` … That is why this verb returns a new `ObjId` rather
//! > than mutating the target, and why nothing about the target changes."*
//!
//! and
//!
//! > *"AND A SECOND STATUS CHAINS ONTO THE FIRST, PER AUTHOR … 'Additional
//! > state changes shall be made by adding text annotations **in reply to the
//! > previous reply** for a given user.' So this verb walks the `/IRT` graph
//! > rooted at `target` … building a per-author chain rather than a star."*
//!
//! ⇒ The file holds a **log**, not a field. So nothing here may say *set the
//! status*, *change the status* or *the status is now*. [`record_label`] says
//! **Record status**; [`status_recorded`] says what was added and how deep the
//! operator's own history on that comment now is; [`row_status_history`] says
//! how many earlier statuses stand behind the one being shown. A control
//! labelled *Set status* would describe a different document format.
//!
//! ## ★★ THE SECOND FACT: THE ENGINE DOES NOT INTERPRET THE STRINGS
//!
//! `Annotation::state` (`annot.rs:480`) and `::state_model` (`annot.rs:492`)
//! are `Option<String>`, decoded verbatim, and the field's own doc says why:
//!
//! > *"Neither key carries a 'shall be one of' anywhere in either edition, so
//! > a value outside Table 171's vocabulary is **unhandled, not illegal**.
//! > Reporting it verbatim is therefore reading the file rather than tolerating
//! > it. `ReviewState` is the closed set pdfcer AUTHORS; this is the open set
//! > it reads."*
//!
//! ⇒ The vocabulary is **this shell's to present**, and an unknown value must
//! be **shown**, never normalised. That is why there are three families of
//! string for one value — [`state_name`] for the seven pdfcer authors,
//! [`state_unmodelled`] for a value in a model pdfcer authors, and
//! [`state_foreign`] for a value in a model it will not author. The
//! distinction, and the reason collapsing the last two would be wrong, is
//! [`crate::text::buttonaction`]'s, reused rather than re-derived; see
//! [`crate::panels::comments::reviewstate::StateReading`].
//!
//! ## ★ Where these strings are NOT
//!
//! **Never on the canvas.** R8b — *"fuzzy, never sneaky"* — and its clause
//! about the original GUI: *"the nagging and red flagging … made for a lot of
//! extra bugs in the visibility when editing."* A review status is a
//! **disclosure about a comment**, not part of the comment's appearance, so it
//! is drawn in the panel and in the status row and nowhere else. A mark whose
//! status is *Rejected* is drawn exactly as the file will draw it.
//!
//! ## Conventions, restated from [`crate::text`] because they bind here
//!
//! - Sentence case, no trailing period on labels; full sentences for prose.
//! - The **file's own spelling** for any value that came out of the document,
//!   including `Cancelled`, which Table 171 writes in British English and
//!   which is therefore not "corrected" anywhere below.

/// The seven states pdfcer AUTHORS, as the operator reads them.
///
/// # ★ These are the file's own words, and that is the decision
///
/// `Accepted`, `Rejected`, `Cancelled`, `Completed`, `None`, `Marked`,
/// `Unmarked` are Table 171's vocabulary and
/// [`pdfcer_core::edit::ReviewState::as_str`] writes exactly those bytes. A
/// friendlier relabelling — *Approved*, *Done*, *Ticked* — would give the
/// operator a fourth name for a value they will meet again in Acrobat, in the
/// saved file and in `pdfcer list-annotations`, and would make a screenshot of
/// this panel un-checkable against the document it describes.
///
/// The one place a word is added rather than changed is `None`, which alone is
/// ambiguous on screen: Table 171 makes it a **writable value** in the `Review`
/// model and *not* a spelling of "no status recorded" — `annot.rs:472-479`
/// says so in as many words — and a bare *None* in a chooser would read as the
/// empty entry. See [`state_none`].
#[must_use]
pub fn state_name(state: pdfcer_core::edit::ReviewState) -> &'static str {
    match state {
        pdfcer_core::edit::ReviewState::None => state_none(),
        other => other.as_str(),
    }
}

/// `Review`-model `None`, disambiguated from *no status at all*.
///
/// # ★★ The distinction the standard makes and a menu would lose
///
/// `Annotation::state`'s doc: *"`None` is a **writable value**, not a spelling
/// of 'the key is absent'."* Recording `None` is an act — it is how a reviewer
/// withdraws a status they set an hour ago — and it produces a state
/// annotation with a `/State` of `None` in the file. A chooser entry reading
/// just *None* sits beside [`filter_status_any`] and
/// [`filter_status_unrecorded`] and would be read as one of them.
#[must_use]
pub fn state_none() -> &'static str {
    "None (status withdrawn)"
}

/// **A `/State` in a model pdfcer authors, whose value is not in that model's
/// vocabulary** — shown verbatim, never normalised.
///
/// # ★★★ Why this is not folded into [`state_foreign`]
///
/// [`crate::text::buttonaction::button_action_current`]'s table, applied to a
/// different key for the same reason:
///
/// | state | what it means | what the row offers |
/// |---|---|---|
/// | modelled | Table 171's value, in Table 171's model | show it, record another |
/// | **unmodelled** | pdfcer **authors** this model and did not decode this value | name both, **still offer to record** |
/// | foreign | a `/StateModel` pdfcer will not author | name both, offer nothing in it |
///
/// Collapsing the two would grey a row pdfcer writes happily: a `/StateModel`
/// of `Review` carrying a `/State` of `Deferred` is a document pdfcer can add
/// its own `Accepted` to, in the same model, on the same chain rule. Greying it
/// would be this shell claiming an incapacity it does not have.
///
/// `model` is carried because the value alone is not interpretable —
/// `annot.rs:472-479`: *"a caller that wants the effective state must read both
/// fields together."*
#[must_use]
pub fn state_unmodelled(state: &str, model: &str) -> String {
    format!("{state} — a status pdfcer does not recognise, in the {model} model")
}

/// **A `/StateModel` pdfcer will not author** — both values named, and nothing
/// offered in that vocabulary.
///
/// §12.5.6.3 names two models and neither edition says *shall be one of*, so a
/// third is unhandled rather than illegal. pdfcer cannot record a status in a
/// vocabulary it does not know the values of, and
/// [`pdfcer_core::edit::ReviewState`] has no variant that could hold one — so
/// this is the `Foreign` row of the table above, and R9 makes it render the
/// fact rather than a greyed control.
///
/// ★ It does **not** stop the operator recording their own status: their status
/// goes in the `Review` model, on their own `/IRT` chain, and leaves this one
/// untouched. [`record_other_model_note`] is the sentence that says so.
#[must_use]
pub fn state_foreign(state: &str, model: &str) -> String {
    format!("{state} — in {model}, a review vocabulary pdfcer does not author")
}

/// A `/State` with **no** `/StateModel` — the one combination Table 171 forbids.
///
/// Table 171 makes `/StateModel` *"Required if `State` is present"*, and
/// `Annotation::state_model`'s doc calls this out as the asymmetric half:
/// *"the only non-conforming combination is `/State` present with this
/// absent."* So this is a malformed file, surfaced rather than repaired — the
/// same posture `crate::panels::comments::model::CommentRow::subtype` takes
/// for a missing `/Subtype`.
///
/// pdfcer's own authoring cannot produce it: `add_review_state` derives the
/// model from the state.
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
///
/// Worded as a fact about the document rather than as *Anonymous*, for
/// `crate::text::panels::comments::comment_row_byline`'s reason: `/T` is a
/// Table 170 markup key and its absence is not a claim about a person.
///
/// ★ It also has a consequence the operator can act on, and the sentence
/// carries it: `add_review_state` chains on `/T` equality
/// (`edit.rs:26865-26890`), so an unsigned status can never be continued by a
/// signed one — a later status starts a fresh history beside it.
#[must_use]
pub fn row_status_unsigned(status: &str) -> String {
    format!("{status} — recorded without a name, so it stands on its own")
}

/// **How much history stands behind the status being shown**, when there is
/// more than one.
///
/// # ★★★ This is the string that stops the panel lying about the format
///
/// The row shows one status per reviewer — the tip of that reviewer's `/IRT`
/// chain. `depth` is how many states that reviewer has recorded on this
/// comment altogether. Drawn only when it is greater than one, so the line
/// means something when it appears rather than reading *1 status recorded*
/// beside every row.
///
/// Without it the panel shows a single value per person and is indistinguishable
/// from a panel over a format that stores a mutable field — which is precisely
/// what §12.5.6.3 is not. The engine says so plainly: *"the history a
/// reviewer's chain encodes would simply not be there"* if the shape were
/// flattened.
#[must_use]
pub fn row_status_history(depth: usize) -> String {
    format!("{depth} statuses recorded, most recent shown")
}

/// The line a row carries when it **has no status at all**.
///
/// Drawn only under the status chooser's *No status recorded* filter and
/// nowhere else — see [`crate::panels::comments::reviewstate`]. On an ordinary
/// unfiltered list a caption on every unreviewed row would be forty repetitions
/// of "nothing has happened here", which is the noise
/// `crate::panels::comments`' disclosure discipline exists to keep out.
#[must_use]
pub fn row_status_none() -> &'static str {
    "No status recorded"
}

/// ★★★ **The row that IS a status**, named so an empty comment is not a
/// mystery.
///
/// # Why this exists, and why not filtering the row out instead
///
/// A status is a `/Text` annotation with `/IRT`, `/State`, `/StateModel`, a
/// `/T` — and a deliberately **empty** `/Contents`. `add_review_state`:
///
/// > *"`/Contents` is deliberately EMPTY. A status is not a comment, and
/// > inventing 'Accepted' as the body would put a sentence in the operator's
/// > comment list that the operator never wrote."*
///
/// So the moment the operator records one, a **blank row** appears in the
/// Comments panel. Excluding it was considered and rejected: this panel's
/// founding rule is that *nothing is silently omitted*, and its three existing
/// exclusions are each counted and disclosed in numbers. A fourth silent one
/// would be the panel deciding what the file contains.
///
/// So the row stays and says what it is. `status` is the reading of its own
/// `/State`.
#[must_use]
pub fn row_is_a_status(status: &str) -> String {
    format!("This row is a review status — {status} — recorded on another comment")
}

/// The control that records a status. **Record**, never *Set*.
///
/// See this module's header: the file holds a log. *Set status* would name a
/// field that §12.5.6.3 explicitly does not define, and an operator who read
/// it as one would expect their second status to replace their first.
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
///
/// **Where it goes** (a new annotation, not this one), **that it is added**
/// (the earlier one stays), and **that it is signed** with the name from
/// Settings. All three are invisible from a screenshot of the row, and the
/// third writes a person's name into a file that may leave the building —
/// which `crate::app::prefs::Prefs::author_name` treats as a decision the
/// operator makes rather than one pdfcer makes for them.
#[must_use]
pub fn record_tooltip() -> &'static str {
    "Adds a separate status annotation beside this comment, signed with your \
     name from Settings. Earlier statuses are kept — a comment's status is a \
     history, not a field."
}

/// ★ The note on a comment whose only status is in a **foreign** model.
///
/// Says what the control will do rather than leaving the operator to infer it
/// from a value in a vocabulary nobody has explained: their status is recorded
/// in the `Review` model, on its own chain, and the foreign one is untouched.
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
///
/// Worded *Any status* rather than *All*, unlike the author and type choosers'
/// shared [`crate::text::panels::comments::comment_filter_all`], for one
/// reason: this chooser's other entries include [`filter_status_unrecorded`],
/// and *All* beside *No status recorded* reads as a pair of opposites rather
/// than as an entry and its negation.
#[must_use]
pub fn filter_status_any() -> &'static str {
    "Any status"
}

/// The entry that keeps only comments **nobody has reviewed**.
///
/// ★★ This is the entry the whole filter is for. A reviewer's question on a
/// thirty-six-sheet drawing set is *"what have I not dealt with"*, and it is
/// unanswerable from a chooser that can only name statuses that exist. It is
/// distinct from a `/State` of `None` — see [`state_none`] — and the two
/// entries sit next to each other saying so.
#[must_use]
pub fn filter_status_unrecorded() -> &'static str {
    "No status recorded"
}

/// ★★★ **What was written**, on the status row, after the edit lands.
///
/// # Why `depth` is in the sentence
///
/// [`pdfcer_core::edit::ReviewStateAdded::chain_depth`] is *"how deep the chain
/// for this author now is — `1` for their first status on this target"*, and it
/// is the only observable that distinguishes a correct per-author chain from a
/// star of statuses all pointing at the comment. The engine is blunt about why
/// it is exposed: *"the wrong shape is invisible … a star of state annotations
/// all pointing at the target renders the same as a correct chain in every
/// viewer, so nothing would ever report it."*
///
/// So this sentence is the operator-facing half of that, and it does a second
/// job at the same time: on the second press it says **2**, which is the panel
/// telling the truth about a log rather than a field, at the exact moment the
/// operator would otherwise assume they had overwritten something.
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

/// ★★ The second sentence, when the operator has **no name set**.
///
/// `add_review_state` takes an author `&str` and uses it as the chain key
/// (`edit.rs:26865-26890` matches `title == Some(author)`), so an empty one
/// writes an empty `/T` and **cannot be continued** by a later status recorded
/// once a name is set: that one starts a fresh history beside it.
///
/// That consequence is invisible — the row looks identical either way, and it
/// only shows up later as two parallel chains — so it is stated at the moment
/// it is caused, and it names the place to fix it.
#[must_use]
pub fn status_recorded_unsigned() -> &'static str {
    "Recorded without a name. Set one in Settings > Comments — a later status \
     under a name will start a separate history rather than continue this one"
}
