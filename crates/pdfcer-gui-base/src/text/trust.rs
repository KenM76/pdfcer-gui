//! # `text::trust` — every word this shell says about whether a signature can
//! be trusted
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/trust.md`.

use pdfcer_core::trust_store::SourceCounts;

// ---------------------------------------------------------------------------
// The panel's leading disclosure
// ---------------------------------------------------------------------------

/// The sentence above the list, replacing the old *"pdfcer does not check
/// whether these signatures are valid — it cannot yet"*.
#[must_use]
pub const fn panel_intro() -> &'static str {
    "For each signature below, pdfcer reports three separate facts and never \
     merges them: whether the signed bytes are INTACT, what the signature \
     COVERS, and whether the signer can be TRUSTED. A signature can be intact \
     and untrusted, or trusted and cover only part of the file."
}

/// The heading before a signature's three facts.
#[must_use]
pub fn signature_heading(name: &str) -> String {
    format!("Signature: {name}")
}

// ---------------------------------------------------------------------------
// Fact 1 — integrity
// ---------------------------------------------------------------------------

/// The label the integrity line always begins with.
#[must_use]
pub const fn integrity_label() -> &'static str {
    "Intact:"
}

/// The digest matched and the CMS signature verified.
#[must_use]
pub fn integrity_verified(digest: &str, signature: &str) -> String {
    format!("yes — the signed bytes are exactly what was signed ({digest}, {signature}).")
}

/// A SHA-1 digest, disclosed beside a verdict that is otherwise good news.
#[must_use]
pub const fn integrity_weak_digest() -> &'static str {
    "This signature uses SHA-1, which is no longer considered strong enough to \
     rule out a forged document. The check passed; what it proves is weaker \
     than the same check with SHA-256."
}

/// The covered bytes were altered after signing.
#[must_use]
pub const fn integrity_digest_mismatch() -> &'static str {
    "NO — the bytes this signature covers have been ALTERED since it was \
     signed. This is not a coverage question and not a trust question: what \
     was signed and what is in the file are different."
}

/// The digest matched but the signature value did not verify.
#[must_use]
pub const fn integrity_signature_invalid() -> &'static str {
    "NO — the covered bytes are what was signed, but the signature itself does \
     not verify against the signer's certificate. The signature, the \
     certificate or the signed attributes have been changed."
}

/// pdfcer could not reach a verdict, and says why in the engine's words.
#[must_use]
pub fn integrity_unverifiable(reason: &str) -> String {
    format!("pdfcer could not tell — {reason}. This is not a pass and not a failure.")
}

// ---------------------------------------------------------------------------
// Fact 2 — coverage
// ---------------------------------------------------------------------------

/// The label the coverage line always begins with.
#[must_use]
pub const fn coverage_label() -> &'static str {
    "Covers:"
}

// ---------------------------------------------------------------------------
// Fact 3 — trust
// ---------------------------------------------------------------------------

/// The label the trust line always begins with.
#[must_use]
pub const fn trust_label() -> &'static str {
    "Signer:"
}

/// The signer chains to a trusted anchor.
#[must_use]
pub fn trusted(anchor_subject: &str, source: &[String], validity_checked: bool) -> String {
    let provenance = if source.is_empty() {
        String::new()
    } else {
        format!(" ({})", source.join(", "))
    };
    let validity = if validity_checked {
        "The certificates were inside their validity dates at the time of signing."
    } else {
        "This signature carries no signing time, so pdfcer could NOT check whether \
         the certificates had expired."
    };
    format!(
        "chains to a trusted certificate — {anchor_subject}{provenance}. Every link \
         was checked by signature, and the issuing certificates were entitled to \
         issue. {validity} Revocation was NOT checked: pdfcer never goes on the \
         network, so a certificate that has since been revoked would still read as \
         trusted here."
    )
}

/// Trust was evaluated and the signer does not chain to a trusted anchor.
#[must_use]
pub fn untrusted(reason: &str) -> String {
    format!(
        "does NOT chain to any certificate in your trust list — {reason}. That is \
         a statement about who signed it, not about whether the bytes are intact; \
         a signature can be perfectly valid and still be from somebody your trust \
         list has never heard of."
    )
}

/// Trust was requested and the signer's certificate could not be parsed.
#[must_use]
pub const fn signer_unknown() -> &'static str {
    "could not be identified — pdfcer could not read the certificate embedded in \
     this signature, so trust could not even be attempted. This is not a \
     judgement about the signer."
}

/// The prefix every unchecked-trust sentence begins with.
#[must_use]
pub const fn not_checked_prefix() -> &'static str {
    "not checked"
}

/// The trust line when no anchors were available, with which of the four
/// situations applies.
#[must_use]
pub fn not_checked(why: &str) -> String {
    format!("{} — {why}", not_checked_prefix())
}

/// The setting is off.
#[must_use]
pub const fn not_checked_opted_out() -> &'static str {
    "pdfcer did not look at who signed this, because checking signers is turned \
     off. You can let pdfcer use the trust list your Acrobat has already \
     downloaded: Settings > Digital signatures."
}

/// The setting is on and this machine has no Acrobat trust list.
#[must_use]
pub fn not_checked_no_store(looked_in: usize) -> String {
    format!(
        "pdfcer looked for the trust list an installed Acrobat or Reader \
         downloads, in {looked_in} place(s), and found none on this machine. If \
         yours is somewhere else, point pdfcer at it in Settings > Digital \
         signatures."
    )
}

/// The operator configured a path and nothing is there.
#[must_use]
pub fn not_checked_configured_missing(path: &str) -> String {
    format!(
        "pdfcer was told to use the trust list at {path} and there is no file \
         there. Nothing else was tried, because a path you typed is a choice \
         rather than a hint — correct it in Settings > Digital signatures, or \
         clear the field to let pdfcer look in the usual places."
    )
}

/// A store was found and could not be read.
#[must_use]
pub fn not_checked_unreadable(path: &str, reason: &str) -> String {
    format!(
        "pdfcer found a trust list at {path} and could not read it — {reason}. \
         Signers were not checked at all; nothing here is a statement about this \
         document."
    )
}

// ---------------------------------------------------------------------------
// The anchor set's provenance
// ---------------------------------------------------------------------------

/// **How many anchors, from where, and how old** — one sentence, always.
#[must_use]
pub fn store_line(path: &str, modified: Option<&str>, counts: &SourceCounts) -> String {
    let dated = match modified {
        Some(date) => format!("last updated by Acrobat on {date}"),
        None => "with no readable date, so pdfcer cannot tell you how current it is".to_owned(),
    };
    format!(
        "Using {total} trusted certificates from {path} — {dated}. \
         {aatl} from Adobe's approved list (AATL), {eutl} from the EU trusted \
         lists, {adbe} from Adobe itself, {other} from elsewhere.",
        total = counts.total,
        aatl = counts.aatl,
        eutl = counts.eutl,
        adbe = counts.adbe,
        other = counts.other,
    )
}

/// Entries in the store whose certificate could not be decoded.
#[must_use]
pub fn store_undecodable(count: usize) -> String {
    format!(
        "{count} entr(ies) in that list could not be read and were left out of \
         the check, so a signer that relies on one of them will read as untrusted."
    )
}

/// The at-own-risk disclosure, shown wherever the store is turned on or
/// inspected.
#[must_use]
pub const fn at_own_risk() -> &'static str {
    "This reads a file that belongs to Adobe's program, on your own machine, and \
     nothing leaves it. Whether relying on Adobe's downloaded trust list fits \
     your Acrobat or Reader licence is your decision — pdfcer does not make that \
     determination for you, which is why this is off until you turn it on."
}

// ---------------------------------------------------------------------------
// The Settings group
// ---------------------------------------------------------------------------

/// The group heading.
#[must_use]
pub const fn group_signatures() -> &'static str {
    "Digital signatures"
}

/// Setting 1 — the opt-in. Its title.
#[must_use]
pub const fn use_store_title() -> &'static str {
    "Checking who signed a document"
}

/// What the standard leaves open here.
#[must_use]
pub const fn use_store_silence() -> &'static str {
    "The standard says a reader should check who signed a document and cannot \
     say whose certificates you trust. Adobe's approved list and the EU trusted \
     lists are the answer most people mean, and neither is published in a form a \
     program can just download — the only copy on this machine is the one your \
     Acrobat or Reader already fetched."
}

/// Which way costs what.
#[must_use]
pub const fn use_store_radius() -> &'static str {
    "Affects only what pdfcer TELLS you about a signature. It never changes a \
     document, never writes anything, and never uses the network. It also \
     applies to the pdfcer command line, because it is one choice in one file."
}

/// The off option.
#[must_use]
pub const fn use_store_off_label() -> &'static str {
    "Do not check who signed (the default)"
}

/// The off option's note.
#[must_use]
pub const fn use_store_off_note() -> &'static str {
    "Signatures are still checked for whether their bytes are intact and what \
     they cover. Who signed them is reported as not checked."
}

/// The at-own-risk option.
#[must_use]
pub const fn use_store_on_label() -> &'static str {
    "Use the trust list my Acrobat has downloaded, at my own risk"
}

/// The at-own-risk option's note.
#[must_use]
pub const fn use_store_on_note() -> &'static str {
    "pdfcer reads Acrobat's own downloaded list of trusted certificates and uses \
     it to say whether a signer chains to one of them. It checks the certificate \
     chain, whether each issuer was entitled to issue, and the dates at the time \
     of signing. It does NOT check whether a certificate has since been revoked."
}

/// Setting 2 — where the store is. Its title.
#[must_use]
pub const fn store_path_title() -> &'static str {
    "Where the trust list is"
}

/// What is unsettled here.
#[must_use]
pub const fn store_path_silence() -> &'static str {
    "Adobe does not document where this file lives, and it moves between \
     versions. pdfcer looks in the places every Acrobat and Reader release has \
     used, which is a convention rather than a rule — a redirected profile or a \
     version pdfcer has not been told about will not be found."
}

/// Which way costs what.
#[must_use]
pub const fn store_path_radius() -> &'static str {
    "Changes only which file pdfcer reads certificates from. It is read-only and \
     pdfcer never writes to it."
}

/// The field's label.
#[must_use]
pub const fn store_path_label() -> &'static str {
    "Trust list file (leave blank to look in the usual places)"
}

/// The note under the field.
#[must_use]
pub const fn store_path_note() -> &'static str {
    "This is only a location. Whether pdfcer may read it at all is the setting \
     above."
}

/// The picker button.
#[must_use]
pub const fn store_path_browse() -> &'static str {
    "Browse…"
}

/// The picker button's tooltip.
#[must_use]
pub const fn store_path_browse_hover() -> &'static str {
    "Find an addressbook.acrodata file — the list of trusted certificates \
     Acrobat and Reader download."
}

/// The label of the picker's file filter.
#[must_use]
pub const fn store_path_filter() -> &'static str {
    "Acrobat trust list"
}

/// What pdfcer currently resolves, when a usable store was found.
#[must_use]
pub fn resolved_found(path: &str, modified: Option<&str>) -> String {
    match modified {
        Some(date) => format!("pdfcer will read {path}, last updated on {date}."),
        None => format!("pdfcer will read {path}. Its date could not be read."),
    }
}

/// What pdfcer currently resolves, when nothing was found.
#[must_use]
pub fn resolved_none(looked_in: usize) -> String {
    format!(
        "No trust list was found on this machine. pdfcer looked in \
         {looked_in} place(s). If you have Acrobat or Reader, open it once and \
         let it update its trusted certificates, or type the file's location \
         above."
    )
}

/// What pdfcer currently resolves, when the operator's own path is wrong.
#[must_use]
pub fn resolved_configured_missing(path: &str) -> String {
    format!("There is no file at {path}, so no certificates will be read.")
}

/// The button that reads the store and reports what is in it.
#[must_use]
pub const fn inspect_button() -> &'static str {
    "Show what is in it"
}

/// The inspect button's tooltip.
#[must_use]
pub const fn inspect_hover() -> &'static str {
    "Read the file now and report how many trusted certificates it holds and \
     when it was last updated. Nothing is copied and nothing is changed."
}

/// The inspect button's failure.
#[must_use]
pub fn inspect_failed(reason: &str) -> String {
    format!("That file could not be read — {reason}.")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **`NotChecked` says "not checked", in every one of its four
    /// explanations.**
    #[test]
    fn every_unchecked_trust_sentence_says_not_checked() {
        let explanations = [
            not_checked_opted_out().to_owned(),
            not_checked_no_store(4),
            not_checked_configured_missing(r"D:\nope\addressbook.acrodata"),
            not_checked_unreadable(r"D:\a\addressbook.acrodata", "bad header"),
        ];
        for why in explanations {
            let line = not_checked(&why);
            assert!(
                line.starts_with("not checked"),
                "an unchecked-trust line must SAY it was not checked: {line}"
            );
        }
    }

    /// **The four explanations are four different sentences.**
    #[test]
    fn the_four_reasons_trust_was_not_checked_are_four_sentences() {
        let all = [
            not_checked_opted_out().to_owned(),
            not_checked_no_store(4),
            not_checked_configured_missing(r"D:\nope\addressbook.acrodata"),
            not_checked_unreadable(r"D:\a\addressbook.acrodata", "bad header"),
        ];
        for (i, a) in all.iter().enumerate() {
            for b in all.iter().skip(i + 1) {
                assert_ne!(a, b, "two of the four situations share one sentence");
            }
        }
        // And each names the thing that distinguishes it, so the difference is
        // not merely a comma.
        assert!(all[0].contains("turned off"), "{}", all[0]);
        assert!(all[1].contains("found none on this machine"), "{}", all[1]);
        assert!(all[2].contains("no file there"), "{}", all[2]);
        assert!(all[3].contains("could not read it"), "{}", all[3]);
    }

    /// **A `Trusted` verdict discloses that revocation was not checked.**
    #[test]
    fn a_trusted_verdict_never_claims_more_than_the_engine_checked() {
        let with_clock = trusted("CN=Some CA", &["AATL".to_owned()], true);
        assert!(
            with_clock.contains("Revocation was NOT checked"),
            "{with_clock}"
        );
        assert!(with_clock.contains("AATL"), "{with_clock}");
        assert!(with_clock.contains("CN=Some CA"), "{with_clock}");

        // And with no signing-time clock, the sentence must say the dates were
        // NOT checked — the engine's `validity_checked == false`, which is a
        // second thing a `Trusted` does not prove.
        let no_clock = trusted("CN=Some CA", &[], false);
        assert!(no_clock.contains("could NOT check"), "{no_clock}");
        assert!(
            no_clock.contains("Revocation was NOT checked"),
            "{no_clock}"
        );
    }

    /// **"Untrusted" is not allowed to sound like "tampered with".**
    #[test]
    fn untrusted_separates_itself_from_integrity() {
        let line = untrusted("the chain is incomplete");
        assert!(
            line.contains("not about whether the bytes are intact"),
            "{line}"
        );
    }

    /// **The store's count and its date are one sentence.**
    #[test]
    fn the_store_is_never_described_without_its_age() {
        let counts = SourceCounts {
            aatl: 211,
            eutl: 1576,
            adbe: 2,
            other: 0,
            total: 1789,
        };
        let dated = store_line(r"D:\a\addressbook.acrodata", Some("2024-05-27"), &counts);
        assert!(dated.contains("1789"), "{dated}");
        assert!(dated.contains("2024-05-27"), "{dated}");

        let undated = store_line(r"D:\a\addressbook.acrodata", None, &counts);
        assert!(
            undated.contains("cannot tell you how current it is"),
            "a store with no readable date must say its age is unknown: {undated}"
        );
    }

    /// **The three facts have three distinct labels.**
    #[test]
    fn the_three_facts_are_labelled_apart() {
        let labels = [integrity_label(), coverage_label(), trust_label()];
        for (i, a) in labels.iter().enumerate() {
            for b in labels.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
        assert!(
            panel_intro().contains("never merges them"),
            "{}",
            panel_intro()
        );
    }
}
