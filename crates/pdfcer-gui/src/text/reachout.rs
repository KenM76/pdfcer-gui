//! # `text::reachout` — the sentence a document that reaches outside itself earns
//!
//! One function. Its whole difficulty is tone: it has to be **taken
//! seriously without being alarming**, because pdfcer runs none of these and
//! nothing is about to happen.
//!
//! ## ★★★ The two ways to get this wrong, and they are opposite
//!
//! **Too alarming** and it is a warning about a thing that cannot occur here —
//! pdfcer recognises actions and never executes one — so the operator learns
//! that pdfcer cries wolf and stops reading the status row. That costs every
//! future disclosure, not just this one.
//!
//! **Too quiet** and it is the failure the engine named as the urgent one:
//! *"silence and safety are indistinguishable to the reader."* A drawing that
//! posts its title block to a web server when somebody presses a button in
//! Acrobat is a fact its recipient needs before they hand it on.
//!
//! ⇒ So every sentence here is built the same way: **what the document CAN do**,
//! then **who would do it**. *"…if it is opened in a viewer that runs them"* is
//! the clause that carries the whole tone, because it is simultaneously the
//! reassurance and the warning.
//!
//! ## Why one sentence and not a list
//!
//! `app::status` has one slot for consequences. A document with a submit button
//! and a launch action gets both facts in one line rather than two lines of
//! which the operator reads the last.

use crate::app::reachout::ReachOut;

/// **What this document reaches for**, in one sentence.
///
/// Only called when [`ReachOut::worth_saying`] is true, so there is always
/// something in it.
///
/// # ★★ The truncation clause comes FIRST when it applies
///
/// Because it changes what every other clause means. *"pdfcer could not finish
/// checking"* followed by *"and found a submit action"* is honest; the same two
/// facts in the other order reads as a complete finding with a footnote.
///
/// And when the walk was cut short and found **nothing**, the sentence is the
/// truncation alone — never an all-clear, which is the one thing a partial scan
/// must not imply.
#[must_use]
pub fn disclosure(reach: ReachOut) -> String {
    let mut parts: Vec<String> = Vec::new();

    if reach.network > 0 {
        parts
            .push("send data somewhere \u{2014} it carries a submit or web-link action".to_owned());
    }
    if reach.launch > 0 {
        parts.push("start another program".to_owned());
    }
    if reach.script_on_open {
        parts.push("run a script as soon as it is opened".to_owned());
    }

    // ★ The truncation-only case. No counts, so no claim about what is or is
    // not in the file — just the honest report that the check did not finish.
    if parts.is_empty() {
        return "pdfcer could not finish checking this document for submit, launch and script \
                actions, so treat it as unchecked rather than clean."
            .to_owned();
    }

    let what = join_and(&parts);
    // ★★★ The clause that carries the tone. It is the reassurance (pdfcer will
    // not do any of this) and the warning (something else might) in one breath,
    // and removing either half makes the sentence wrong in one of the two
    // directions this module's header describes.
    let body = format!(
        "This document can {what}. pdfcer never does any of that \u{2014} it reads these and \
         leaves them alone \u{2014} but a viewer that runs them would."
    );

    if reach.truncated {
        format!(
            "pdfcer could not finish checking this document, so there may be more than this. {body}"
        )
    } else {
        body
    }
}

/// `a`, `a and b`, `a, b and c`.
fn join_and(parts: &[String]) -> String {
    match parts {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reach(network: usize, launch: usize, on_open: bool, truncated: bool) -> ReachOut {
        ReachOut {
            network,
            launch,
            script_on_open: on_open,
            truncated,
        }
    }

    /// ★★★ A truncated scan that found nothing must NOT read as an all-clear.
    ///
    /// The engine's own words about why this is the urgent case: *"a check that
    /// under-reports reads as a clean bill of health, because silence and
    /// safety are indistinguishable to the reader."*
    #[test]
    fn a_truncated_scan_with_no_findings_says_unchecked_not_clean() {
        let s = disclosure(reach(0, 0, false, true));
        assert!(
            s.contains("unchecked rather than clean"),
            "a partial scan must never imply an all-clear. Got: {s}"
        );
        assert!(
            !s.contains("This document can"),
            "with no findings there is nothing to claim the document CAN do. Got: {s}"
        );
    }

    /// ★★ Truncation leads when there are findings too.
    #[test]
    fn truncation_comes_before_the_findings_it_qualifies() {
        let s = disclosure(reach(1, 0, false, true));
        let cut = s.find("could not finish").expect("the truncation clause");
        let found = s.find("This document can").expect("the findings clause");
        assert!(
            cut < found,
            "the caveat must precede what it qualifies, or the sentence reads as a complete \
             finding with a footnote. Got: {s}"
        );
    }

    /// ★★★ Every sentence says pdfcer does not do it AND that something else might.
    ///
    /// Both halves, always. Drop the first and it is an alarm about a thing
    /// that cannot happen here; drop the second and it is a shrug about a thing
    /// that can happen anywhere else.
    #[test]
    fn every_finding_carries_both_halves_of_the_tone() {
        for r in [
            reach(1, 0, false, false),
            reach(0, 1, false, false),
            reach(0, 0, true, false),
            reach(2, 1, true, false),
        ] {
            let s = disclosure(r);
            assert!(
                s.contains("pdfcer never does any of that"),
                "the reassurance is missing from: {s}"
            );
            assert!(
                s.contains("a viewer that runs them would"),
                "the warning is missing from: {s}"
            );
        }
    }

    /// Three findings read as prose.
    #[test]
    fn three_findings_join_into_one_sentence() {
        let s = disclosure(reach(1, 1, true, false));
        assert!(s.contains("send data somewhere"), "got: {s}");
        assert!(
            s.contains("start another program and run a script as soon as it is opened"),
            "the last two must join with `and`. Got: {s}"
        );
    }
}
