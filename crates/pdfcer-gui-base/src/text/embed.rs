//! # `text::embed` — what the Embed-fonts window says before it changes
//! anything
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/embed.md`.

use pdfcer_core::font_embed_missing::EmbedBlocker;
use pdfcer_core::font_unembed::PdfaClaim;

/// The window's title bar.
#[must_use]
pub const fn window_title() -> &'static str {
    "Embed fonts"
}

/// The opening sentence.
#[must_use]
pub const fn intro() -> &'static str {
    "Embedding puts each font's outlines inside this document, so it looks the same on a machine \
     that does not have them. It makes the file bigger, and it is a change to the document \
     rather than to how it is shown."
}

/// The document carries no fonts that are missing their program.
#[must_use]
pub const fn nothing_missing() -> &'static str {
    "Every font this document uses is already embedded, so there is nothing to do."
}

/// How many fonts will be embedded, and from where.
#[must_use]
pub fn will_embed(count: usize) -> String {
    format!("{count} font(s) will be embedded:")
}

/// One font that will be embedded, and where it comes from.
#[must_use]
pub fn embed_row(face: &str, source: &str, matched: crate::fontlibrary::Match) -> String {
    use crate::fontlibrary::Match;
    match matched {
        Match::Exact => format!("{face} — from {source}"),
        // The weaker file match, said plainly. A stem match is this shell
        // deciding that a file called `Helv.ttf` is the face the document calls
        // `Helvetica` — an inference, and Rule 4's surviving half says an
        // inference the operator cannot see owes them a report.
        Match::Stem => {
            format!("{face} — from {source}, matched on the file's name rather than the font's")
        }
        // A documented family equivalence, and the sentence says the
        // letterforms differ. `Helvetica` → `Arial` is metric-compatible by
        // design and the advances come from `/Widths` regardless, so the page
        // does not reflow — what changes is the shape of every letter, which is
        // exactly the part a screenshot would not tell them either.
        Match::Alias => format!(
            "{face} — from {source}, a different face of the same metrics. The letters \
             will look different."
        ),
        // The loud one, and O47's answer was "always, DISCLOSED LOUDLY".
        // It says three things in order: that nothing of theirs answered, that
        // pdfcer supplied one of its own, and that the result is a stand-in.
        // Dropping any of the three leaves a row that reads like the others.
        Match::Bundled => format!(
            "{face} — none of your fonts matched, so pdfcer used {source}. It is a \
             stand-in, not the font the document asks for."
        ),
    }
}

/// The heading over the fonts that cannot be embedded.
#[must_use]
pub fn cannot_embed(count: usize) -> String {
    format!("{count} font(s) cannot be embedded:")
}

/// One blocked font, with the engine's reason put into the operator's words.
#[must_use]
pub fn blocked_row(face: &str, blocker: &EmbedBlocker, pdfcer_has_a_copy: bool) -> String {
    let why = match blocker {
        EmbedBlocker::AlreadyEmbedded => "it is already embedded".to_owned(),
        EmbedBlocker::ProgramDeclaredButUnreadable => {
            "the document says it carries this font, and those bytes cannot be read".to_owned()
        }
        //
        // It used to say only *"add a folder that does"*. Since O47 and O50,
        // there are **two** remedies and the cheap one is a checkbox — so this
        // names that first, because a row that sends an operator to a folder
        // picker when one click would do is a row that costs them the
        // difference.
        //
        // ⇒ A refusal's wording is a claim about what would fix it, and the
        // things that fix it change under it. This one had been true for
        // exactly one day.
        //
        //
        // Until today pdfcer's own fourteen faces answered unconditionally, so
        // a font pdfcer carries could never reach this row and the clause *"and
        // it is not one of the fourteen pdfcer carries itself"* was true of
        // everything that did. Now that the operator can decline them (O47,
        // built as the disclosed opt-in), a declined standard-14 face lands
        // here — and that clause would tell him pdfcer has no copy of a font
        // pdfcer is holding in its hand.
        //
        // So the row is told, and the two sentences are genuinely different
        // claims about the same document rather than one sentence with a
        // detail swapped.
        EmbedBlocker::NoSourceFont if pdfcer_has_a_copy => {
            "none of your fonts matched it, and pdfcer has its own copy of this one. Tick the \
             box below to use it — the letters will look different — or add the folder that \
             has the real font"
                .to_owned()
        }
        EmbedBlocker::NoSourceFont => {
            "pdfcer has nowhere to take it from, and it is not one of the fourteen pdfcer \
             carries itself. Under Settings, switch on the fonts installed on this computer, \
             or add the folder that has it"
                .to_owned()
        }
        EmbedBlocker::Composite { .. } => {
            "it is a composite font, which pdfcer does not embed into".to_owned()
        }
        EmbedBlocker::Type3 => {
            "it is a Type 3 font, whose glyphs are drawings in the file rather than a font \
             program — there is nothing to embed"
                .to_owned()
        }
        EmbedBlocker::NoMetricSource => {
            "the document records no widths for it, and embedding without them would move every \
             letter"
                .to_owned()
        }
        EmbedBlocker::ProgramUnrecognised => {
            "the file found for it is not a font pdfcer can read".to_owned()
        }
        EmbedBlocker::ProgramIsCollection => {
            "the file found for it holds several faces in one, which pdfcer cannot embed from"
                .to_owned()
        }
        _ => "pdfcer would not embed it, and this build cannot say why".to_owned(),
    };
    format!("{face} — {why}")
}

/// The heading over names that matched nothing at all.
#[must_use]
pub fn own_fonts_offer(faces: &[String]) -> String {
    format!(
        "pdfcer carries its own copy of {}: {}.",
        if faces.len() == 1 {
            "one font this document is missing".to_owned()
        } else {
            format!("{} of the fonts this document is missing", faces.len())
        },
        faces.join(", ")
    )
}

/// **What using them costs, in his terms** — the two things the list does not
/// say.
#[must_use]
pub const fn own_fonts_consequence() -> &'static str {
    "Those letters will look different on the screen of whoever you send this to. pdfcer's \
     copies also come with a licence that asks to be credited wherever the file goes, which \
     is why this is your choice and not something pdfcer does on its own."
}

/// The checkbox itself.
#[must_use]
pub const fn own_fonts_checkbox() -> &'static str {
    "Use pdfcer's own copies where none of yours match"
}

/// Distinct from a blocked font: an unmatched name is one the *request* named
/// and the document does not have, which is an operator's typo or a stale list
/// rather than anything about the file.
#[must_use]
pub fn unmatched(names: &[String]) -> String {
    format!(
        "These names are not fonts in this document: {}",
        names.join(", ")
    )
}

/// What the document claims about PDF/A, and what an embed does to it.
#[must_use]
pub fn pdfa_line(claim: &PdfaClaim) -> Option<String> {
    match claim {
        PdfaClaim::None => None,
        PdfaClaim::Identified { part, conformance } => {
            let named = match (part.as_deref(), conformance.as_deref()) {
                (Some(p), Some(c)) => format!("PDF/A-{p}{c}"),
                (Some(p), None) => format!("PDF/A-{p}"),
                _ => "PDF/A".to_owned(),
            };
            Some(format!(
                "This document says it is {named}. Embedding is usually what a PDF/A needs — an \
                 unembedded font is one of the things that breaks the claim — but the claim is \
                 the document's, and pdfcer does not re-check it."
            ))
        }
        // Its own sentence, and the engine's own reason for the distinction:
        // an output intent alone is NOT a PDF/A claim, because a plain PDF may
        // legitimately carry one for colour management. Reporting it as a claim
        // would tell an operator their drawing is something it is not.
        PdfaClaim::OutputIntentOnly => Some(
            "This document carries a PDF/A output intent but does not identify as PDF/A. That is \
             normal for colour management and is not a claim."
                .to_owned(),
        ),
        // "We looked and there is none" and "we could not look" lead to
        // different decisions — `PdfaClaim`'s own docs make that distinction and
        // this keeps it.
        _ => Some(
            "This document's metadata could not be read, so pdfcer cannot say whether it claims \
             to be PDF/A."
                .to_owned(),
        ),
    }
}

/// The button that performs the embed.
#[must_use]
pub const fn embed_button() -> &'static str {
    "Embed"
}

/// The button that does not.
#[must_use]
pub const fn cancel_button() -> &'static str {
    "Cancel"
}

/// What the embed did, for the status bar.
#[must_use]
pub fn embedded(count: usize) -> String {
    format!("Embedded {count} font(s) into this document.")
}

/// The embed was refused outright by the document.
#[must_use]
pub fn refused(detail: &str) -> String {
    format!("pdfcer would not embed into this document: {detail}")
}

/// The most the file can grow by, in the operator's units.
#[must_use]
pub fn size_ceiling(bytes: u64) -> String {
    let mib = bytes as f64 / (1024.0 * 1024.0);
    if mib >= 0.1 {
        format!("This will add at most {mib:.1} MB to the file.")
    } else {
        let kib = bytes as f64 / 1024.0;
        format!("This will add at most {kib:.0} KB to the file.")
    }
}

/// How many fonts are still without a program once this plan has run.
#[must_use]
pub fn still_missing(count: usize) -> Option<String> {
    (count > 0).then(|| {
        format!(
            "{count} font(s) will still have no program afterwards. Each one is listed below with \
             its reason."
        )
    })
}

/// The same sentence when some of those fonts are **not** listed below.
#[must_use]
pub fn still_missing_partly_unexplained(count: usize, unexplained: usize) -> String {
    format!(
        "{count} font(s) will still have no program afterwards, and {unexplained} of them are not \
         listed below — they were not part of this request."
    )
}

/// Why the Embed button is dead.
#[must_use]
pub const fn nothing_to_embed() -> &'static str {
    "None of these fonts can be embedded. Each one below says what stopped it."
}

/// The disclosure after an embed, one sentence per fact worth stating.
#[must_use]
pub fn embedded_disclosure(
    embedded: usize,
    still_missing: usize,
    substituted: bool,
) -> Vec<String> {
    let mut out = vec![format!("Embedded {embedded} font(s) into this document.")];
    if still_missing > 0 {
        out.push(format!(
            "{still_missing} font(s) still have no program. Tools > Embed fonts lists what each \
             one needs."
        ));
    }
    if substituted {
        out.push(
            "At least one font was matched to a different face than the document names.".to_owned(),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every blocker an operator can meet names a remedy or says there is
    /// none.**
    #[test]
    fn every_blocker_reads_as_a_sentence_a_person_can_act_on() {
        for blocker in [
            EmbedBlocker::AlreadyEmbedded,
            EmbedBlocker::ProgramDeclaredButUnreadable,
            EmbedBlocker::NoSourceFont,
            EmbedBlocker::Type3,
            EmbedBlocker::NoMetricSource,
            EmbedBlocker::ProgramUnrecognised,
            EmbedBlocker::ProgramIsCollection,
        ] {
            // BOTH positions of the new flag, in the same loop. A sweep run
            // only at `false` would have left the `pdfcer_has_a_copy` arm —
            // which is the arm a declined standard-14 font actually lands on —
            // untested by the completeness check written to make sure no
            // blocker reads as a dead end.
            for pdfcer_has_a_copy in [false, true] {
                let line = blocked_row("ArialMT", &blocker, pdfcer_has_a_copy);
                assert!(line.starts_with("ArialMT — "), "names the face: {line}");
                assert!(line.len() > 20, "says something: {line}");
                assert!(
                    !line.contains("Blocker") && !line.contains("::"),
                    "leaks a type name at the operator: {line}"
                );
            }
        }
    }

    /// **A standard-14 font pdfcer carries is told so, and one it does not is
    /// not** — the two halves of the same row, asserted together.
    #[test]
    fn a_font_pdfcer_carries_is_offered_its_own_copy_and_one_it_does_not_is_not() {
        let ours = blocked_row("Helvetica", &EmbedBlocker::NoSourceFont, true);
        let theirs = blocked_row("Helvetica", &EmbedBlocker::NoSourceFont, false);
        assert_ne!(
            ours, theirs,
            "a font pdfcer carries a copy of and one it does not must not get the same remedy; \
             the whole point of the flag is that the cheap fix exists for one and not the other"
        );
        assert!(
            ours.contains("box"),
            "the row for a font pdfcer carries must name the checkbox as the remedy, because it \
             is one click against a folder picker: {ours}"
        );
        assert!(
            !theirs.contains("box"),
            "the row for a font pdfcer does NOT carry must not offer a box that cannot help it — \
             that is a press that always fails: {theirs}"
        );
    }

    /// **The offer names the fonts**, and does not merely count them.
    #[test]
    fn the_offer_names_every_font_it_would_stand_in_for() {
        let faces = vec![
            "Helvetica".to_owned(),
            "Helvetica-Bold".to_owned(),
            "Times-Roman".to_owned(),
        ];
        let line = own_fonts_offer(&faces);
        for face in &faces {
            assert!(
                line.contains(face.as_str()),
                "the offer must name {face}, and it said: {line}"
            );
        }
    }

    /// **The consequence states BOTH costs**, and the licence is the one that
    /// gets forgotten.
    #[test]
    fn the_consequence_states_the_look_and_the_licence() {
        let line = own_fonts_consequence();
        assert!(
            line.contains("look different"),
            "the letters change on the recipient's screen, and it must say so: {line}"
        );
        assert!(
            line.contains("licence"),
            "embedding pdfcer's own face carries its licence into a file the operator sends \
             out, and that is the half that binds him rather than his reader: {line}"
        );
    }

    /// **A document with no PDF/A claim gets no sentence about PDF/A.**
    #[test]
    fn no_claim_means_no_line() {
        assert!(pdfa_line(&PdfaClaim::None).is_none());
        assert!(pdfa_line(&PdfaClaim::OutputIntentOnly).is_some());
        assert!(
            pdfa_line(&PdfaClaim::Identified {
                part: Some("2".to_owned()),
                conformance: Some("B".to_owned()),
            })
            .expect("a claim gets a line")
            .contains("PDF/A-2B")
        );
    }

    /// **An output intent is not reported as a claim.**
    ///
    /// The engine's own distinction, and losing it would tell an operator their
    /// colour-managed drawing claims a conformance it does not.
    #[test]
    fn an_output_intent_is_not_a_claim() {
        let line = pdfa_line(&PdfaClaim::OutputIntentOnly).expect("has a line");
        assert!(line.contains("does not identify"), "{line}");
        assert!(line.contains("not a claim"), "{line}");
    }

    /// **The disclosure after an embed drops the clauses that would be
    /// zero.**
    #[test]
    fn the_disclosure_says_only_what_is_true() {
        let clean = embedded_disclosure(3, 0, false);
        assert_eq!(clean.len(), 1, "{clean:?}");
        assert!(clean[0].contains("3"), "{clean:?}");

        let full = embedded_disclosure(3, 2, true);
        assert_eq!(full.len(), 3, "{full:?}");
        assert!(full[1].contains("2"), "{full:?}");
        assert!(full[2].contains("different face"), "{full:?}");
    }

    /// **A number that has arrived gets no sentence.**
    #[test]
    fn nothing_still_missing_means_nothing_said() {
        assert!(still_missing(0).is_none());
        assert!(still_missing(1).is_some());
    }

    /// **The size is stated as a CEILING, in both branches.**
    #[test]
    fn the_size_is_a_bound_and_never_a_prediction() {
        for bytes in [512_u64, 40_000, 3_000_000] {
            let line = size_ceiling(bytes);
            assert!(line.contains("at most"), "{line}");
        }
    }

    /// **Each of the four rungs reads as a different sentence, and the
    /// bundled one is the loudest.**
    #[test]
    fn every_rung_says_something_different_and_bundled_says_the_most() {
        use crate::fontlibrary::Match;
        let exact = embed_row("ArialMT", "C:/f/Arial.ttf", Match::Exact);
        let stem = embed_row("Helvetica", "C:/f/Helv.ttf", Match::Stem);
        let alias = embed_row("Helvetica", "C:/f/Arial.ttf", Match::Alias);
        let bundled = embed_row(
            "Helvetica",
            "pdfcer's own copy of FoxitSans",
            Match::Bundled,
        );

        assert!(!exact.contains("matched on"), "{exact}");
        assert!(!exact.contains("look different"), "{exact}");
        assert!(stem.contains("matched on the file's name"), "{stem}");
        assert!(alias.contains("look different"), "{alias}");

        // The three clauses the loud row must carry, asserted one at a time
        // so a rewrite that drops one fails naming which.
        assert!(
            bundled.contains("none of your fonts matched"),
            "it does not say nothing of theirs answered: {bundled}"
        );
        assert!(
            bundled.contains("pdfcer used"),
            "it does not say pdfcer supplied one: {bundled}"
        );
        assert!(
            bundled.contains("stand-in"),
            "it does not say the result is a stand-in: {bundled}"
        );

        // Four rungs, four distinct sentences.
        let all = [&exact, &stem, &alias, &bundled];
        for (i, a) in all.iter().enumerate() {
            for b in all.iter().skip(i + 1) {
                assert_ne!(a, b, "two rungs render identically");
            }
        }
    }
}
