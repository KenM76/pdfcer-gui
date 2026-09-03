//! # `text::signature` — what this shell says about a digital signature before
//! and after it writes
//!
//! The copy for [`crate::dialogs::signature`] and for the two status-bar notes
//! [`crate::app::save`] records once a file has been written.
//!
//! ## ★★★ THE RULE THAT GOVERNS EVERY STRING IN THIS FILE
//!
//! **Nothing here may be invented.** This is claim-bearing copy about a
//! security property of a legal artifact, and the engine that computes the
//! verdict has already written down, at length, exactly which claims are
//! supportable and which are not. Every sentence below is a translation of a
//! distinction `pdfcer-core`'s `signature` module draws, into operator English,
//! **without softening it and without strengthening it**. Where a sentence
//! could be read as saying more than the engine says, it has been cut back
//! until it cannot.
//!
//! Read `D:\Dev\pdfcer\crates\pdfcer-core\src\signature.rs`'s module
//! documentation before changing a word of this file. Its own header opens
//! with the instruction, and the reason is not ceremony: the module records
//! that *"the folk model is wrong in three separate places"*, so recall is a
//! worse source here than in any other area of the catalog.
//!
//! ## The two stages, which is the distinction the whole file is built around
//!
//! ISO 32000-1 §12.8.2.2.2 splits signature validation in two:
//!
//! | Stage | What it proves | Applies to |
//! |---|---|---|
//! | **1 — byte-range digest** | the bytes the signature covers are unchanged since signing | every signature with a `/ByteRange` |
//! | **2 — permitted-changes analysis** | later revisions stayed inside the author's allowance | only signatures carrying a transform method |
//!
//! §12.8.1 NOTE 1 promises that an **incremental** update preserves the signed
//! byte range. That is a **stage-1** fact and it says nothing at all about
//! stage 2. The engine states the consequence in as many words:
//!
//! > Reporting stage-1 success as "the signature is still valid" is the
//! > specific error this section exists to prevent.
//!
//! Hence [`preserved_note`], which is the only string in this file that
//! describes a save whose signatures survived it, spends its second half
//! saying what it does **not** mean. That is not hedging. It is the difference
//! between a disclosure and a reassurance, and the engine's variant
//! documentation forbids the second in terms: *"A front end that renders this
//! as a reassurance is committing precisely the error §12.8.2.2.2's two-stage
//! split exists to prevent. Pair it with the uncertainty, or say nothing."*
//!
//! ## ★★ Why there are TWO wordings for one verdict
//!
//! `SignatureImpact::Invalidated` is reached on two different footings, and
//! `SignatureImpact::documentation_basis` exists — in the engine's own words —
//! *"because the two deserve different operator-facing wording and a front end
//! cannot tell them apart from the variant alone"*:
//!
//! | Footing | When | The strings |
//! |---|---|---|
//! | `ImpactBasis::SpecSourced` | the document carries a **certification** signature, whose `/DocMDP` transform records a **closed** list of permitted changes (Table 254: *"other changes shall invalidate the signature"*) | [`headline_certified`], [`basis_certified`], [`proceed_certified`] |
//! | `ImpactBasis::ConservativeReport` | the document carries only **approval** signatures, for which ISO 32000-1 defines stage 1 and **only** stage 1 — so a conforming validator reading the standard alone would call the document still valid | [`headline_approval`], [`basis_approval`], [`proceed_approval`] |
//!
//! The second row is the engine's headline negative result, and it is the
//! reason the second wording is careful in a way the first is not. pdfcer
//! reports `Invalidated` there anyway — *"a product decision under rule 4
//! (fuzzy-never-sneaky), not a spec citation"* — on the asymmetry that
//! **over-reporting is a reviewable hint an operator can dismiss, and
//! under-reporting is pdfcer making a silent claim about the integrity of a
//! legal artifact.** So the copy must report the verdict *and* say whose
//! verdict it is. [`basis_approval`] does both in one sentence.
//!
//! ## ★★ Three things no string in this file is allowed to say
//!
//! 1. **That any other reader agrees.** The widely-repeated claim that Acrobat
//!    and the PAdES family report such a document as *"signed, but altered
//!    since signing"* is, in the engine's words, *"empirical tool behaviour,
//!    explicitly not sourced"*, and *"must not be cited from here"*. So no
//!    sentence below predicts what will happen in another application. That is
//!    a real temptation — it is the most useful thing an operator could be
//!    told — and it is unsourced, which under the claim-bearing-copy rule
//!    settles it.
//! 2. **That a signature is valid, or verified, or checked.** The engine's
//!    first line is *"This module verifies nothing."* It computes no digest,
//!    parses no PKCS#7 blob and validates no certificate chain. [`verifies_nothing`]
//!    is the footnote that says so on the one surface where an operator might
//!    otherwise read pdfcer's silence as a pass.
//! 3. **"Author signature", or any resolution of it.** Table 234's seed value
//!    `MDP /P 0` defines *"an author signature"* to mean an ordinary approval
//!    signature, while §12.8.2.2.1 uses *"the author of a document"* to mean
//!    the certifier — two incompatible uses of one word in one clause family.
//!    The engine uses neither and says so; neither does this file. The words
//!    used here are **certification signature** (the engine's own term, from
//!    §12.8.1) and, for everything else, plainly *signed*.
//!
//! ## Why the counts are spelled out rather than `(s)`
//!
//! [`crate::text::compact::signature_line`] writes *"{count} digital
//! signature(s)"*, and this file deliberately does not follow it. That form is
//! readable as a template rather than as a sentence, and this copy is read at
//! the moment an operator is deciding whether to accept an irreversible
//! change — which is the worst possible moment for a surface to look
//! unfinished. Each string below branches on the count and reads as English in
//! both directions. The cost is one `if` per sentence; the divergence from the
//! neighbouring module is recorded here so it reads as a choice.
//!
//! ## Voice
//!
//! The catalog's standing conventions apply — sentence case, full sentences
//! with punctuation for prose, name the thing and what the operator can do.
//! One addition specific to this area: **no exclamation, no capitals, no
//! "warning".** The one sentence in this shell that shouts —
//! `text::compact::signature_line`'s *"CANNOT keep them"* — earns it by
//! describing a loss that cannot be repaired by any later act. A save that
//! invalidates a signature is not in that class: the file the operator started
//! from is still on disk (a copy) or still contains its earlier revision (an
//! in-place incremental save), so the situation is recoverable and the copy
//! should not imply otherwise.

/// The title bar of the window that asks before an invalidating save.
///
/// ★ Deliberately **neutral about the verdict**, and it is the one string here
/// that does not branch on the basis. A title is read first and out of
/// context — it is also what the taskbar entry shows — so it states the fact
/// that is true on both footings and leaves the verdict to the body, where the
/// sentence that qualifies it is one line away rather than a window away.
///
/// *"This document is signed"* is also the fact an operator is most likely not
/// to know. A drawing that arrived by email carries no visible mark of it in
/// this shell's canvas, and the Signatures panel is a dock tab they may never
/// have opened.
#[must_use]
pub const fn window_title() -> &'static str {
    "This document is signed"
}

/// The headline when a **certification** signature is present — the
/// `ImpactBasis::SpecSourced` case.
///
/// ★★ It asserts the outcome flatly, because here pdfcer can. §12.8.1 makes a
/// signature a certification signature when its `/Reference` array holds a
/// signature-reference dictionary whose `/TransformMethod` is `/DocMDP`, and
/// Table 254's permitted-change lists are **closed** — *"other changes shall
/// invalidate the signature"* is a `shall`, with no minor-change tolerance.
/// The engine works pdfcer's operations against that table and concludes that
/// none of them is on the permitted list at any `/P` value. So *"will"* is the
/// standard's own modality and not this catalog's confidence.
#[must_use]
pub fn headline_certified(count: usize) -> String {
    if count == 1 {
        "Saving will invalidate this document's certification signature.".to_owned()
    } else {
        format!("Saving will invalidate this document's {count} signatures.")
    }
}

/// The headline when only **approval** signatures are present — the
/// `ImpactBasis::ConservativeReport` case.
///
/// ★★★ It states the **change**, not the verdict, and that asymmetry with
/// [`headline_certified`] is the whole point of having two headlines.
///
/// The verdict here is pdfcer's cautious one rather than the standard's: for an
/// approval signature with no `/Reference`, ISO 32000-1 defines validation in
/// exactly one sentence — *"A signature shall be validated by recomputing the
/// digest and comparing it with the one stored in the signature"* — which is
/// stage 1 and only stage 1, and the engine's RAG confirms the **absence** of
/// any clause saying a post-signing revision invalidates such a signature.
///
/// A headline reading *"Saving will invalidate…"* would therefore put a claim
/// pdfcer cannot source in the largest type in the window, which is exactly
/// where a reader stops. So the headline says the part that is
/// incontrovertible — the document is being changed after it was signed — and
/// [`basis_approval`], one line below, gives pdfcer's verdict together with the
/// fact that it is pdfcer's.
#[must_use]
pub fn headline_approval(count: usize) -> String {
    if count == 1 {
        "This save changes the document after it was signed.".to_owned()
    } else {
        format!("This save changes the document after its {count} signatures were applied.")
    }
}

/// Why the verdict stands, when a certification signature is present.
///
/// The `ImpactBasis::SpecSourced` sentence: a statement of fact, phrased as
/// one. Every clause in it is lifted from the engine's module documentation —
/// the certifier's list, its closedness, and the finding that no pdfcer
/// operation is on it.
///
/// ★ It says *"the person who certified it"* rather than *"the author"*. See
/// the header's third prohibition: the standard uses "author" for both parties
/// in adjacent clauses, and this shell must not silently pick one.
#[must_use]
pub const fn basis_certified() -> &'static str {
    "A certification signature records the changes the person who certified this document \
     allowed. The PDF standard says any other change invalidates it, and none of the changes \
     pdfcer makes are on that list."
}

/// Why the verdict stands, when only approval signatures are present.
///
/// ★★★ **The most carefully worded string in this catalog**, and the one that
/// most repays reading the engine's module documentation before editing.
///
/// It has to do three incompatible-looking things at once:
///
/// 1. **Report pdfcer's verdict**, which is `Invalidated`. Not reporting it
///    would be the under-reporting the engine names as the worse error.
/// 2. **Not attribute that verdict to the standard**, because the standard is
///    silent. The engine is explicit that this arm rests on *"a product
///    decision under rule 4 (fuzzy-never-sneaky), not a spec citation"*.
/// 3. **Not predict another reader's behaviour**, which is the unsourced claim
///    the engine forbids citing.
///
/// The sentence that satisfies all three is the engine's own asymmetry stated
/// plainly: pdfcer would rather over-report a reviewable hint than make a
/// silent claim about a legal artifact. An operator reading it learns both
/// what pdfcer thinks and how much weight to give it, which is more than either
/// half alone would tell them.
#[must_use]
pub const fn basis_approval() -> &'static str {
    "The PDF standard does not settle whether that invalidates a signature of this kind. pdfcer \
     reports it as invalidated rather than tell you nothing has changed, so this is a cautious \
     answer and not a measurement."
}

/// What an in-place save does to the file the signature is in.
///
/// ★ Named because the two save paths differ in the one way an operator cares
/// about at this moment: whether the file they already have survives. This one
/// writes over it.
///
/// It still says the earlier revision is kept, because that is true and it is
/// the whole reason this shell saves incrementally — §7.5.6's update is
/// appended and the original bytes stay verbatim ahead of it. What it must not
/// do is let that sound like the signature is therefore fine: the earlier
/// revision being recoverable *from inside the file* is a different fact from
/// the current revision satisfying a signature, and the sentence keeps them in
/// that order so the second is not read as following from the first.
#[must_use]
pub fn target_in_place(name: &str) -> String {
    format!(
        "This writes over {name}. Your edits are appended, so the signed version is still inside \
         the file — but the document as it now stands is the one a reader checks."
    )
}

/// What a save-a-copy does to the file the signature is in: nothing.
///
/// ★ Lifted deliberately close to [`crate::text::compact::signature_line`]'s
/// closing sentence — *"Your original file keeps its signatures."* — because
/// that sentence already ships, is already true of a command that always
/// writes a new file, and two different phrasings of one guarantee is how two
/// surfaces come to be read as promising two different things.
#[must_use]
pub const fn target_copy() -> &'static str {
    "This writes a new file. Your original is not changed and keeps its signature."
}

/// The button that goes ahead, when a certification signature is present.
///
/// ★★ It **names the destructive act**, which is `crate::dialogs::unsaved`'s
/// standing rule for this crate: *"a destructive button says the destructive
/// thing, so that an operator who reads only the buttons — which is most
/// operators, most of the time — cannot get it wrong."*
///
/// It is safe to name it here and not on the other footing, and that
/// difference is the rule this catalog runs on: **the button may assert
/// exactly as much as the evidence does.** Table 254 supports *"invalidate"*
/// as a statement of fact; nothing supports it for a plain approval signature.
#[must_use]
pub const fn proceed_certified() -> &'static str {
    "Save and invalidate the signature"
}

/// The button that goes ahead, when only approval signatures are present.
///
/// ★ *"Save anyway"* rather than *"Save and invalidate the signature"*,
/// deliberately, and see [`proceed_certified`] for the rule. The word *anyway*
/// carries the whole of what pdfcer can honestly put on a button here: there is
/// something to weigh, the operator has read it, and they are proceeding. It
/// does not assert an outcome pdfcer cannot source.
///
/// It is also the conventional label for exactly this gesture, which matters:
/// the operator's standing instruction is to use the conventional interaction
/// rather than invent one, and a proceed-past-a-warning button reading
/// anything else would be a novelty in a dialog whose whole job is to be
/// instantly legible.
#[must_use]
pub const fn proceed_approval() -> &'static str {
    "Save anyway"
}

/// The button that does not save.
///
/// ★ *"Cancel"*, matching `crate::text::unsaved::cancel_button` and every
/// other confirmation in this crate. The non-destructive answer is the one an
/// operator presses reflexively to make a surprise go away, and it must wear
/// the label that reflex expects.
#[must_use]
pub const fn cancel_button() -> &'static str {
    "Cancel"
}

/// The footnote under the buttons: pdfcer has not checked anything.
///
/// ★★★ The single most important sentence in this file, and it is in the
/// smallest type — because its job is not to be read in this window but to be
/// available when an operator wonders what pdfcer actually knows.
///
/// The engine's module documentation opens with it: **"This module verifies
/// nothing.** It computes no digest, parses no PKCS#7 blob, and validates no
/// certificate chain." Everything this shell says about signatures is
/// arithmetic over where bytes are, plus a reading of a permissions
/// dictionary. Without this sentence, an operator who sees pdfcer speak
/// confidently about a signature here will reasonably conclude that pdfcer's
/// *silence* elsewhere means it looked and found nothing wrong — and it never
/// looked at all.
///
/// It sits below the buttons for `crate::dialogs::unsaved`'s reason: it is
/// what somebody needs *after* they have noticed the window is making a claim
/// and wondered how it knows, and putting it above would stand a sentence
/// between the question and the answer.
#[must_use]
pub const fn verifies_nothing() -> &'static str {
    "pdfcer does not check any signature's certificate or its cryptography. It reports what a \
     save does to the bytes a signature covers, and nothing more."
}

/// The status-bar note after a save whose signatures kept their byte range.
///
/// ⚠️ **This string is the reason `SignatureImpact::ByteRangePreserved` has a
/// surface at all, and it must never become a reassurance.** The engine's
/// variant documentation:
///
/// > ⚠️ **This is not "the signature is still valid."** Stage 2 — whether the
/// > changes are ones the signer permitted — is a separate question this
/// > variant makes no claim about. A front end that renders this as a
/// > reassurance is committing precisely the error §12.8.2.2.2's two-stage
/// > split exists to prevent. **Pair it with the uncertainty, or say nothing.**
///
/// This shell pairs it. The sentence is built in exactly that order — the
/// stage-1 fact, then the word *but*, then the stage-2 question named as
/// unanswered — so that a reader who stops halfway has read the fact and not
/// yet reached a conclusion, rather than the reverse.
///
/// ## ★★ Why *pair it* was chosen over *say nothing*, which was permitted
///
/// Both are allowed and the choice is this shell's. Three reasons, in order of
/// weight:
///
/// 1. **This shell has already made the operator a promise adjacent to it.**
///    `file.save_copy`'s shipped tooltip says the edits *"are appended as an
///    update so the previous version stays intact inside the file"*, and the
///    Save-a-compacted-copy window says in as many words that a rewrite
///    **cannot** keep signatures while the original file does. An operator who
///    has read those two surfaces has been handed exactly the premises from
///    which the folk conclusion — *appended, therefore my signature is fine* —
///    follows. Silence here leaves that inference standing, and it is wrong.
/// 2. **Rule 4.** The operator cannot see this anywhere else: no mark appears
///    on the canvas, and the Signatures panel reports what the document
///    carries rather than what a save did to it.
/// 3. **It is cheap and it is rare.** It costs one line on a row that already
///    exists, only for a document that actually carries a signature — which is
///    the same conditional-rarity argument `crate::text::compact` uses to
///    justify showing its own signature sentence only when it applies.
#[must_use]
pub fn preserved_note(count: usize) -> String {
    let subject = if count == 1 {
        "This document is signed"
    } else {
        "This document carries several signatures"
    };
    format!(
        "{subject}, and this save was appended, so the bytes each signature covers are \
         unchanged. That is not the same as the signature still being valid: whether these \
         changes are ones the signer allowed is a separate question, and pdfcer does not answer \
         it."
    )
}

/// The status-bar note after a save that pdfcer reports as invalidating.
///
/// ★★ It exists for a path where it is the **only** disclosure, and that is
/// why it repeats what the window said rather than assuming the window was
/// seen. `crate::app::lifecycle::resume_after_unsaved` writes a copy from
/// inside an already-answered question, and this shell does not stack a second
/// modal on that gesture — see `crate::dialogs::signature`'s header for the
/// argument. On that route this note is the whole of what the operator is
/// told, so it has to stand alone.
///
/// ★ It says *"pdfcer reports"* rather than *"this invalidated"*, and it says
/// so on **both** footings. A post-hoc receipt is read quickly and out of
/// context; splitting it in two the way the window's copy is split would put
/// the more careful wording on the less careful reading. Attributing the
/// verdict to pdfcer is true when the basis is a spec citation (pdfcer is
/// reporting what the standard says) and necessary when it is not, so one
/// sentence serves both without over-claiming on either.
#[must_use]
pub fn invalidated_note(count: usize) -> String {
    let (subject, object) = if count == 1 {
        ("This document is signed", "the signature")
    } else {
        ("This document carries several signatures", "them")
    };
    format!(
        "{subject}, and the save you just made changes it after signing. pdfcer reports that as \
         invalidating {object}; it has not checked any signature's cryptography."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **The preserved-save note never reads as a reassurance.**
    ///
    /// The one assertion in this file that is about a *prohibition* rather
    /// than about content, and it is the engine's own: a front end that
    /// renders `ByteRangePreserved` as "the signature is still valid" commits
    /// the specific error §12.8.2.2.2's two-stage split exists to prevent.
    ///
    /// Asserted as the presence of the negation rather than as the absence of
    /// the phrase, because the absence is the weaker property: a rewrite that
    /// dropped the qualifying half would still contain no such phrase and
    /// would still be a reassurance by implication. What makes the sentence
    /// safe is that it *names the open question*, so that is what is checked.
    #[test]
    fn the_preserved_note_pairs_the_fact_with_the_uncertainty() {
        for count in [1, 3] {
            let note = preserved_note(count);
            assert!(
                note.contains("not the same as the signature still being valid"),
                "the stage-1 fact is stated without the stage-2 denial beside it: {note}"
            );
            assert!(
                note.contains("separate question"),
                "the note must name stage 2 as unanswered, not merely decline to answer it: \
                 {note}"
            );
            assert!(
                note.contains("pdfcer does not answer it"),
                "the note must say who is not answering: {note}"
            );
        }
    }

    /// ★★ **The two footings do not share a sentence, and the certified one is
    /// the only one that asserts the outcome.**
    ///
    /// `SignatureImpact::documentation_basis` exists *"because the two deserve
    /// different operator-facing wording and a front end cannot tell them
    /// apart from the variant alone"*. A build that called the same function
    /// for both would satisfy every other test in this file, so the divergence
    /// is asserted directly.
    ///
    /// The second half is the substantive one: the word *invalidate* may
    /// appear as an assertion about the future only where Table 254 supports
    /// it. `basis_approval` names the verdict too, but attributes it — hence
    /// the check is on the **headline**, which is the sentence read first and
    /// often alone.
    #[test]
    fn the_two_footings_are_worded_differently() {
        assert_ne!(headline_certified(1), headline_approval(1));
        assert_ne!(basis_certified(), basis_approval());
        assert_ne!(proceed_certified(), proceed_approval());

        assert!(
            headline_certified(1).contains("invalidate"),
            "Table 254's closed list supports the flat assertion and the headline should make it"
        );
        assert!(
            !headline_approval(1).contains("invalidate"),
            "ISO 32000-1 is silent for a plain approval signature; the headline must not put \
             pdfcer's cautious verdict in the largest type as though it were the standard's"
        );
    }

    /// ★★ **The approval footing says whose verdict it is.**
    ///
    /// The engine's headline negative result, guarded. A sentence that
    /// reported `Invalidated` for an approval signature without saying that
    /// the standard is silent would be pdfcer citing a clause that does not
    /// exist — and it would read as more authoritative than the certified
    /// case, which is exactly backwards.
    #[test]
    fn the_approval_footing_attributes_the_verdict_to_pdfcer() {
        let basis = basis_approval();
        assert!(
            basis.contains("does not settle"),
            "the silence of the standard is the load-bearing fact: {basis}"
        );
        assert!(
            basis.contains("pdfcer reports"),
            "the verdict must be attributed, not asserted: {basis}"
        );
    }

    /// ★ **No string in this file predicts another application's behaviour.**
    ///
    /// The engine names the claim and forbids citing it: that Acrobat and the
    /// PAdES family report such a document as *"signed, but altered since
    /// signing"* is empirical tool behaviour, explicitly not sourced in the
    /// spec RAG. It is the single most tempting sentence to add here, because
    /// it is the most useful thing an operator could be told — which is why it
    /// is guarded by a test rather than by a paragraph.
    #[test]
    fn nothing_here_claims_what_another_reader_will_say() {
        let strings = [
            window_title().to_owned(),
            headline_certified(1),
            headline_approval(1),
            basis_certified().to_owned(),
            basis_approval().to_owned(),
            target_in_place("a.pdf"),
            target_copy().to_owned(),
            proceed_certified().to_owned(),
            proceed_approval().to_owned(),
            verifies_nothing().to_owned(),
            preserved_note(1),
            invalidated_note(1),
        ];
        for s in &strings {
            let lower = s.to_lowercase();
            for forbidden in ["acrobat", "pades", "other viewer", "other readers"] {
                assert!(
                    !lower.contains(forbidden),
                    "`{forbidden}` is an unsourced claim about another application: {s}"
                );
            }
        }
    }

    /// ★★ **Nothing here says a signature is valid, or that pdfcer checked
    /// one.**
    ///
    /// `pdfcer-core`'s signature module opens *"This module verifies nothing."*
    /// So the affirmative forms are forbidden outright and the negative ones
    /// are the point — which is why this test hunts **phrases** rather than
    /// the word *valid*. The word itself is unavoidable: *invalidate* and
    /// *invalidated* are the verdict's own vocabulary and contain it, and a
    /// substring check would fail on the two sentences the file most needs.
    ///
    /// The forbidden list is therefore the set of ways a sentence could assert
    /// the thing pdfcer has not established, plus *verified* and *we checked*,
    /// which claim an act that never happened.
    #[test]
    fn no_string_claims_a_signature_is_valid_or_was_verified() {
        assert!(
            preserved_note(1).contains("not the same as the signature still being valid"),
            "the one affirmative-looking phrase in the catalog must sit inside its own negation"
        );
        for s in [
            window_title().to_owned(),
            headline_certified(1),
            headline_approval(1),
            basis_certified().to_owned(),
            basis_approval().to_owned(),
            target_in_place("a.pdf"),
            target_copy().to_owned(),
            proceed_certified().to_owned(),
            proceed_approval().to_owned(),
            verifies_nothing().to_owned(),
            invalidated_note(1),
        ] {
            let lower = s.to_lowercase();
            for forbidden in [
                "still valid",
                "is valid",
                "are valid",
                "remains valid",
                "remain valid",
                "verified",
                "we checked",
                "pdfcer checked",
            ] {
                assert!(
                    !lower.contains(forbidden),
                    "`{forbidden}` asserts something pdfcer has not established: {s}"
                );
            }
        }
    }

    /// The singular and the plural are different sentences, and both read as
    /// English.
    ///
    /// The `(s)` form this file deliberately does not use would pass a test
    /// that only checked the count appeared, which is why the assertion is
    /// that the two forms **differ** and that neither carries the template
    /// marker.
    #[test]
    fn the_counts_read_as_sentences() {
        for pair in [
            (headline_certified(1), headline_certified(4)),
            (headline_approval(1), headline_approval(4)),
            (preserved_note(1), preserved_note(4)),
            (invalidated_note(1), invalidated_note(4)),
        ] {
            assert_ne!(pair.0, pair.1, "the plural must not be the singular");
            assert!(!pair.0.contains("(s)"), "{}", pair.0);
            assert!(!pair.1.contains("(s)"), "{}", pair.1);
        }
    }

    /// ★ **The in-place sentence names the file it is about to write over.**
    ///
    /// The one string here that takes an operand, and the operand is the whole
    /// point: *"this writes over your file"* is a different statement from
    /// *"this writes over `D:\jobs\4471\Sheet 1.pdf`"* to an operator with
    /// four documents open, and the tab strip makes that the ordinary case.
    #[test]
    fn the_in_place_sentence_names_the_file() {
        let line = target_in_place("Sheet 1.pdf");
        assert!(line.contains("Sheet 1.pdf"), "{line}");
        assert_ne!(line, target_copy());
    }
}
