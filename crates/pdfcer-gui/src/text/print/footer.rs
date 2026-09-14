//! **The print window's three ways out** — `OPERATOR_REQUESTS.md` **O185**.
//!
//! Split out of [`super`] under R2 on 2026-09-14, at the banner that was
//! already there. The seam is not only the line count: every other section of
//! that catalogue names a control describing *what will be printed*, and this
//! one names the controls for *leaving*, which after O166 is a decision about
//! state the window owns rather than about the job.
//!
//! # ★★ What the three words have to do between them
//!
//! | string | promise |
//! |---|---|
//! | [`commit`] (in [`super`]) | print, and keep these settings |
//! | [`keep_and_close`] | keep these settings, print nothing |
//! | [`cancel`] | put the settings back to what they were |
//!
//! Two of those three are new sentences rather than new behaviour: printing
//! has kept the settings since O166. Nothing had to *say so* until there was a
//! button beside it that only keeps, at which point an operator choosing
//! between them needs each to disclose the half the other does not.
//!
//! # ★★★ Why the hovers exist at all, given R9 and this project's dislike of chrome
//!
//! Because the difference between these three buttons is entirely in what
//! happens to state the operator cannot see. A label can say *Cancel*; it
//! cannot say *and the copy count goes back to one*. Rule 4 is **fuzzy, never
//! sneaky**, and its surviving half is the one that binds here: an inference or
//! an effect the operator cannot observe still owes a disclosure, off-canvas.
//! A tooltip on the control that causes it is as close to the act as
//! off-canvas gets.
//!
//! Re-exported wholesale by [`super`], so every caller keeps writing
//! `t::cancel()` and this split is invisible at the call sites.

/// Leave without printing, **and put the settings back**.
///
/// # ★★★ It said "Close" until O185, and the old reasoning was right about the old window
///
/// The argument it carried: *nothing has started, so there is nothing to
/// cancel, and a Cancel button next to a Print button invites the reading that
/// a job is in flight and this stops it.* That was sound while the window held
/// nothing but a job description. It stopped being sound the day the window
/// started owning persistent state, because from that day there IS something
/// to cancel and it is not the job -- it is every setting changed since the
/// window opened.
///
/// The misreading the old comment feared is still worth avoiding and still is
/// avoided, by [`cancel_hover`], which says in words what the button puts back
/// and does not mention jobs at all.
///
/// # ★★ Why this word and not "Discard" or "Revert"
///
/// Because it sits on the route the window chrome means. `dialogs.md` G4 makes
/// the OS close button, Escape and this button deliberately indistinguishable,
/// and the operator learned what the X does from every other window on the
/// machine. A button labelled with a verb the chrome cannot be labelled with
/// would be claiming a meaning the other two routes then quietly share without
/// saying so.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

/// What [`cancel`] does to the settings, in the operator's own terms.
///
/// Names all three routes, because they are one route with three doorways and
/// an operator who learns it from the button should not have to re-learn it
/// from the X. ★ It says *"opened with"* rather than *"saved"*: the window may
/// have written settings already -- a Print that reached a printer which then
/// refused the job saves before it spools -- and "unsaved changes" would be a
/// false description of what is being put back.
#[must_use]
pub const fn cancel_hover() -> &'static str {
    "Puts the print settings back to what they were when this window opened. Escape and the window's close button do the same."
}

/// Keep the settings for next time, without printing.
///
/// # ★★★ The fourth route, and why it needs a label rather than a chord
///
/// Operator request O185: *"I set the printer up, close the window to go check
/// something, and it's all gone."* The three routes that existed -- Print, and
/// the two spellings of Close -- could express "print and keep" and "leave and
/// lose", and had no way at all to say *"keep what I set, but do not print
/// yet"*.
///
/// It cannot be folded into the chrome. G4 reserves the chrome for one
/// meaning, and putting the KEEPING one there would mean an operator who shuts
/// the window on a mess of three hundred copies and the wrong tray inherits
/// that mess on their next print, having done nothing to ask for it. The safe
/// meaning goes on the chrome; the positively-chosen one gets a button with a
/// verb on it.
///
/// ★ **Two words, both of them promises.** *Keep* says what happens to the
/// settings and *close* says what happens to the window, and an operator who
/// reads only the first word still gets the half that distinguishes this
/// button from the one beside it.
#[must_use]
pub const fn keep_and_close() -> &'static str {
    "Keep and close"
}

/// What [`keep_and_close`] does, in the operator's own terms.
///
/// ★ *"without printing"* is the clause that earns the tooltip. The label
/// already says the settings are kept; what an operator hesitating over an
/// unfamiliar button needs to know is that pressing it does **not** put paper
/// through the machine.
#[must_use]
pub const fn keep_and_close_hover() -> &'static str {
    "Keeps these settings for next time without printing."
}

/// What pressing Print does to the settings, beyond printing.
///
/// The behaviour is O166's and predates O185 by four days; the sentence is
/// new, because until there was a button beside it that ONLY keeps, nothing
/// had to distinguish the two.
#[must_use]
pub const fn commit_hover() -> &'static str {
    "Prints, and keeps these settings for next time."
}

/// Send the job. The plain label, when nothing will be clipped.
#[must_use]
pub const fn commit() -> &'static str {
    "Print"
}

/// Send the job, **with the clip count in the button's own label**.
///
/// The dialog is the confirmation and there is no second gate, so the
/// uncertainty is stated in the disclosure rather than implied by a confirm
/// step existing. Putting it *in the label* rather than beside the button is
/// the difference between a warning the operator has to have read and one
/// they can have looked past — it is on the control their hand is already on.
#[must_use]
pub fn commit_with_clipping(clipped: usize) -> String {
    if clipped == 1 {
        "Print — 1 sheet will be clipped".to_owned()
    } else {
        format!("Print — {clipped} sheets will be clipped")
    }
}

/// Send the job, **naming a count that has actually been measured** —
/// operator request O113, 2026-09-04.
///
/// # Why a second sentence rather than a reworded [`commit_with_clipping`]
///
/// The two say different things and both are needed.
///
/// `commit_with_clipping` says *"N sheets will be **clipped**"* — a geometric
/// fact about page boxes and the printable rectangle, taken at planning time
/// with no raster in hand. It is exactly true and it is what the button says
/// when nothing better is known.
///
/// This one says *"N sheets will **lose content**"*, and it may only be shown
/// when every clipped sheet in the job has been rendered by the preview and
/// its overhang tested for ink. `N` is then the number that really will lose
/// something, which is smaller than the geometric count whenever the operator
/// prints a 1:1 CAD sheet whose border is empty paper — *"the area that isn't
/// printed is just empty border."*
///
/// ★★ **Reusing the old sentence for the corrected number would have been the
/// defect.** With two of five clipped sheets known blank, *"Print — 3 sheets
/// will be clipped"* is plainly false: five are clipped. The count changed
/// what it counts, so the sentence has to say what it now counts. That is a
/// correction, and it is the opposite of softening a true statement to match a
/// better one.
#[must_use]
pub fn commit_losing_content(losing: usize) -> String {
    if losing == 1 {
        "Print — 1 sheet will lose content".to_owned()
    } else {
        format!("Print — {losing} sheets will lose content")
    }
}

/// Send the job, **naming a ceiling** — operator request O113, 2026-09-04.
///
/// Shown when some clipped sheets have been examined and found blank and
/// others have not been looked at. The number is `known_inked + unexamined`:
/// the most sheets that could possibly lose something, with every sheet nobody
/// has looked at still counted, because a claim about an unexamined sheet
/// would be invented.
///
/// # ★ The two words carrying the whole difference
///
/// *"up to"* and *"may"*. They are here because the number is a bound rather
/// than a count, and they are **absent** from [`commit_losing_content`] and
/// from [`commit_with_clipping`] because those two report numbers that were
/// measured — one by the ink test, one by the geometry. A hedge that appeared
/// on all three would say nothing at all; appearing on exactly the one bounded
/// number is what makes it informative.
///
/// # The singular has no "up to", and that is not an inconsistency
///
/// *"Up to 1 sheet"* reads as a quantity discount. *"1 sheet **may** lose
/// content"* carries the same uncertainty in the word that is doing the work,
/// which is `may` in both forms.
#[must_use]
pub fn commit_may_lose_content(at_most: usize) -> String {
    if at_most == 1 {
        "Print — 1 sheet may lose content".to_owned()
    } else {
        format!("Print — up to {at_most} sheets may lose content")
    }
}

/// Confirmation that the job went out.
#[must_use]
pub fn sent(pages: usize) -> String {
    if pages == 1 {
        "Sent 1 page to the printer.".to_owned()
    } else {
        format!("Sent {pages} pages to the printer.")
    }
}

/// The job did not go out, and why.
///
/// `detail` is `pdfcer-print`'s own error `Display`, passed through rather than
/// rewritten — for the same reason [`crate::text::canvas_render_failed`] does
/// it: those errors are structured, specific diagnostics, and replacing one
/// with "an error occurred" throws away the only part of the sentence that
/// helps.
///
/// **Says nothing came out.** A failed spool can leave an operator wondering
/// whether half a job reached the tray, and the first line of the answer
/// belongs in the message.
#[must_use]
pub fn failed(detail: &str) -> String {
    format!("Nothing was sent to the printer. {detail}")
}

/// ★ **The driver would not report its settings, so the job carried only what
/// pdfcer sets itself.**
///
/// Shown beside [`sent`] after a job whose `SettingsSource` came back
/// `Synthesised`, and after no other.
///
/// # Why this is a disclosure and not an error
///
/// The job printed. Paper came out. Nothing failed, and if the operator was
/// changing nothing but orientation they may not be able to tell the
/// difference.
///
/// What was lost is everything the driver holds that pdfcer does not model:
/// media type, print quality, colour handling, output bin, stapling, and the
/// whole vendor-private half of a `DEVMODE` — which on the printers measured
/// while this was built was between 920 and 7,972 bytes, up to 97 % of the
/// structure. A synthesised `DEVMODE` has no private tail to carry any of it.
///
/// So the failure mode is a print that is *subtly* wrong — plain where glossy
/// was configured, draft where best was — and it looks like a printer problem
/// rather than a pdfcer one. That is precisely the class of thing rule 4
/// exists for: pdfcer chose something the operator did not ask for, and it
/// says so.
///
/// # Why it names the remedy
///
/// Because there is one, and it is one button away: opening the driver's own
/// properties dialog produces a real `DEVMODE`, which the next job carries.
#[must_use]
pub const fn settings_synthesised() -> &'static str {
    "This printer would not report its current settings, so the job was sent with only the settings shown here — media type, quality and finishing fell back to the driver's own. Open Properties… before printing again to send them."
}

/// ★★★ Why the Custom percentage field is greyed —
/// `OPERATOR_REQUESTS.md` O77's sweep.
///
/// `dialogs::print::tabs` has always argued that greying is the correct side
/// of R9 here *because* the field is only **temporarily** unavailable: one
/// click on the radio beside it makes it live. R9's other half is that the
/// argument has to reach the operator, and it never did — the control was
/// greyed with no hover explanation of any kind, so the reasoning existed only
/// in a source comment.
///
/// ★ It names the remedy and where the remedy is. *"Choose Custom"* alone
/// would be true and would still leave him looking for what to choose it on;
/// the radio is immediately to the left and saying so costs three words.
#[must_use]
pub fn scale_custom_disabled() -> &'static str {
    "Choose Custom on the radio beside this field to set your own percentage."
}
