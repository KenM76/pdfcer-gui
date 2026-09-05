//! # `text::trust` — every word this shell says about whether a signature can
//! be trusted
//!
//! The catalog area for [`crate::panels::signatures`] and
//! [`crate::dialogs::settings::signatures`]. Its subject is the one place in
//! the product where **a wrong answer is worse than no answer**, so it carries
//! rules the rest of the catalog does not.
//!
//! ## ★★★ THE FOUR RULES THAT GOVERN EVERY STRING IN THIS FILE
//!
//! ### 1. Nothing here may be invented
//!
//! `crate::text::signature`'s header states this for the *save-time* half of
//! the subject and it binds this file identically: this is claim-bearing copy
//! about a security property of a legal artifact, and the engine that computes
//! the verdict has already written down at length which claims are supportable.
//! Every sentence below is a translation of a distinction `pdfcer-core`'s
//! `signature_verify` and `trust_chain` modules draw, **without softening it and
//! without strengthening it**.
//!
//! Read `D:\Dev\pdfcer\crates\pdfcer-core\src\trust_chain.rs`'s module
//! documentation before changing a word. Its "Security posture" section is what
//! [`trusted`] is a translation of, clause for clause.
//!
//! ### 2. The three facts never collapse
//!
//! `SignatureVerdict` carries `integrity`, `coverage` and `trust`, and the
//! engine's own design note is that they *never collapse into one bool*. So
//! there is no `verdict()` function in this file, no composite sentence, and no
//! badge. Three labelled lines, always all three, in that order — which is also
//! the order of increasing uncertainty: integrity is arithmetic, coverage is
//! arithmetic, trust is a judgement about the world.
//!
//! ### 3. `NotChecked` renders as itself
//!
//! Never as a soft "no", never as a grey tick, never omitted. [`not_checked`]
//! and its three siblings all begin with the words *"Not checked"* before they
//! explain which of the four situations applies. A surface that hid the
//! unchecked case would be indistinguishable, on screen, from one that had
//! checked and found nothing wrong.
//!
//! ★ And the four explanations are four sentences rather than one. *"You have
//! this turned off"*, *"this machine has no Acrobat trust list"*, *"you pointed
//! at a file that is not there"* and *"the list is there and pdfcer could not
//! read it"* are four different calls to action, and only one of them is
//! *"nothing is wrong"*.
//!
//! ### 4. A `Trusted` verdict carries what it did NOT check, in the same sentence
//!
//! Not in a tooltip, not in a footnote at the bottom of the panel. `Pass 10.5`
//! checks chain linkage, RFC 5280 CA/key-usage constraints and — only when a
//! signing-time clock exists — certificate validity dates. It does **not** check
//! revocation, and cannot: CRL and OCSP need the network `pdfcer-core` never
//! touches. The engine's own note on every `Trusted` verdict says so, and
//! [`trusted`] says it where the operator is reading the good news, because a
//! qualification separated from its claim is one nobody reads.
//!
//! ## Why the store's DATE is in the same sentence as its size
//!
//! `ENGINE_BACKLOG.md`'s own argument for the whole feature:
//!
//! > an anchor set that silently went stale is worse than one that was never
//! > imported.
//!
//! So [`store_line`] prints *how many anchors* and *when Adobe last wrote the
//! file* as one sentence, and there is no accessor in this module that produces
//! one without the other. ★ On the machine this was written on, the count is
//! about 1,780 and the date is **2024-05-27** — sixteen months old. A control
//! that had shown "1,780 anchors ✓" would have been true and useless.
//!
//! ## Voice
//!
//! The catalog's standing conventions, plus one borrowed from
//! [`crate::text::signature`]: **no exclamation, no capitals, no "warning"**.
//! The one place this file raises its voice is [`integrity_digest_mismatch`],
//! which describes bytes that were altered after signing — a fact about the
//! operator's document that nothing they do now can undo, which is the same
//! class `text::compact::signature_line` earns its shout in.

use pdfcer_core::trust_store::SourceCounts;

// ---------------------------------------------------------------------------
// The panel's leading disclosure
// ---------------------------------------------------------------------------

/// The sentence above the list, replacing the old *"pdfcer does not check
/// whether these signatures are valid — it cannot yet"*.
///
/// ★★★ That sentence had to go the moment `verify_all_with_trust` was wired,
/// and this project's most expensive recorded failure is exactly a claim like
/// it going stale while the prose around it stayed true. What replaces it is
/// **not** a reassurance: it names the three facts, in the order the rows print
/// them, so a reader knows before they start that there are three answers and
/// that one of them may be *not checked*.
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
///
/// A shared prefix rather than three sentences that each happen to mention
/// integrity: the three facts are read as a column, and a column with a ragged
/// left edge is one an operator scans instead of reads.
#[must_use]
pub const fn integrity_label() -> &'static str {
    "Intact:"
}

/// The digest matched and the CMS signature verified.
///
/// ★ It names the algorithms rather than saying "yes", and that is
/// [`pdfcer_core::signature::Integrity::Verified`]'s own instruction: the two
/// fields are carried *"so a shell can disclose a SHA-1 signature as
/// verified-with-a-weak-digest rather than hide it"*. A shell that printed
/// "verified" alone would be discarding the one field that distinguishes a
/// modern signature from one nobody should rely on.
#[must_use]
pub fn integrity_verified(digest: &str, signature: &str) -> String {
    format!("yes — the signed bytes are exactly what was signed ({digest}, {signature}).")
}

/// A SHA-1 digest, disclosed beside a verdict that is otherwise good news.
///
/// The engine reports SHA-1 in its `notes` and does not downgrade the verdict,
/// which is right: the signature genuinely verifies. This shell repeats the
/// fact where the verdict is read, because *"verified"* and *"verified with a
/// digest that has been collision-broken since 2017"* are different things to
/// act on.
#[must_use]
pub const fn integrity_weak_digest() -> &'static str {
    "This signature uses SHA-1, which is no longer considered strong enough to \
     rule out a forged document. The check passed; what it proves is weaker \
     than the same check with SHA-256."
}

/// The covered bytes were altered after signing.
///
/// The one string in this file that states a loss. Worded as the engine words
/// it — the digest does not match, so the bytes changed — and deliberately not
/// as *"the signature is invalid"*, because that phrase folds integrity, trust
/// and coverage into one word and is the exact collapse this feature refuses.
#[must_use]
pub const fn integrity_digest_mismatch() -> &'static str {
    "NO — the bytes this signature covers have been ALTERED since it was \
     signed. This is not a coverage question and not a trust question: what \
     was signed and what is in the file are different."
}

/// The digest matched but the signature value did not verify.
///
/// A genuinely different fault from a digest mismatch and the engine keeps them
/// apart, so this does too: the document's covered bytes are what was signed,
/// and the signature, the certificate or the signed attributes were tampered
/// with instead.
#[must_use]
pub const fn integrity_signature_invalid() -> &'static str {
    "NO — the covered bytes are what was signed, but the signature itself does \
     not verify against the signer's certificate. The signature, the \
     certificate or the signed attributes have been changed."
}

/// pdfcer could not reach a verdict, and says why in the engine's words.
///
/// ★ `reason` is passed through **unedited**. The engine promises this case is
/// *"never reported as either of the other three"*, and it names each cause
/// precisely — an unimplemented subfilter, `adbe.x509.rsa_sha1`, RFC 3161,
/// P-521, Brainpool, a malformed CMS, a hole that does not fit the range. A
/// shell that paraphrased would produce a second, vaguer vocabulary for faults
/// the engine already names exactly.
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
///
/// ★★★ **The single most dangerous string in this application**, and the reason
/// it is long. It states four things in one sentence because separating any of
/// them would leave the good news standing alone:
///
/// 1. the chain reached a trusted anchor, **by verified signatures**;
/// 2. **which** anchor, by subject — an operator who does not recognise the
///    name has learned something a tick could not tell them;
/// 3. its provenance (`AATL`/`EUTL`/`ADBE`), because those are three different
///    programmes with three different admission bars;
/// 4. **what was not checked** — revocation always, and validity dates when the
///    signature carried no signing-time clock.
///
/// Point 4 is not a hedge. `PathChecks::revocation_checked` is `false` on every
/// verdict this build can produce, and a certificate that was revoked the day
/// after it was issued chains exactly as well as one that was not.
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
///
/// ★ *"Valid but untrusted"* is a real and common state — a self-signed
/// certificate, a corporate CA nobody added to Acrobat — and this sentence says
/// so, because an operator who reads "untrusted" beside an intact signature will
/// otherwise conclude the document was tampered with. The engine's own reason
/// is carried through unedited.
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
///
/// The engine keeps this apart from `Untrusted` and so does this. *"pdfcer could
/// not read the certificate"* and *"pdfcer read the certificate and does not
/// trust it"* are opposite findings, and only the second says anything about the
/// signer.
#[must_use]
pub const fn signer_unknown() -> &'static str {
    "could not be identified — pdfcer could not read the certificate embedded in \
     this signature, so trust could not even be attempted. This is not a \
     judgement about the signer."
}

/// The prefix every unchecked-trust sentence begins with.
///
/// ★★★ Its own function, and every caller of the four `not_checked_*` sentences
/// goes through [`not_checked`], so the words *"Not checked"* cannot be dropped
/// from one branch by a well-meaning edit that shortened it.
#[must_use]
pub const fn not_checked_prefix() -> &'static str {
    "not checked"
}

/// The trust line when no anchors were available, with which of the four
/// situations applies.
///
/// The `why` half is supplied by one of the four functions below. They are kept
/// separate from the prefix so that a test can assert **every** one of them
/// starts with the same three words — see this module's tests.
#[must_use]
pub fn not_checked(why: &str) -> String {
    format!("{} — {why}", not_checked_prefix())
}

/// The setting is off.
///
/// Names the remedy and where it is, because this is the one of the four states
/// the operator can fix in five seconds and will otherwise assume is a missing
/// feature.
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
///
/// ★ Not the same sentence as [`not_checked_no_store`], and the separation is
/// the point: this person did not fail to have a store, they made a typo, and
/// telling them their machine has no trust list would send them looking in
/// entirely the wrong place.
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
///
/// See this module's header for why the date is not separable from the count.
/// `modified` is already formatted by [`crate::trust::modified_date`]; a `None`
/// says the filesystem would not give a date, which is itself worth printing
/// because a store whose age is unknown is not a store known to be current.
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
///
/// Only drawn when non-zero. Surfaced rather than swallowed because an operator
/// whose signer happens to be one of the refused entries would otherwise see an
/// inexplicable *"does not chain"* with nothing to look at.
#[must_use]
pub fn store_undecodable(count: usize) -> String {
    format!(
        "{count} entr(ies) in that list could not be read and were left out of \
         the check, so a signer that relies on one of them will read as untrusted."
    )
}

/// The at-own-risk disclosure, shown wherever the store is turned on or
/// inspected.
///
/// ★★ Translated from `pdfcer_core::settings::AcrobatTrustStore`'s own type
/// documentation and from the CLI's identical warning, deliberately: two front
/// ends wording one legal limitation differently is worse than either wording,
/// and this is the sentence a person would quote back at us.
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
///
/// ★ **This one is not a spec silence and the sentence says so**, exactly as
/// `quad_point_order`'s does. ISO 32000-1 is perfectly clear that validation
/// has a trust leg; what it does not do — and cannot — is tell a program which
/// certificates a particular person trusts. That is a fact about the operator,
/// not about the format, and there is no public machine-readable bundle of the
/// lists that matter.
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
///
/// ★ The label spells *"at your own risk"* because the engine's own persisted
/// token does — `acrobat_trust_store = at_own_risk` — and an operator who opens
/// `settings.txt` must find the same words they clicked.
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
///
/// ★ Reported as of the last time pdfcer looked, which is every frame this
/// group is drawn — a `stat`, not a read. That differs from
/// `crate::text::acrobat::resolved_note`, which cannot update as you type
/// because resolving an Acrobat spawns processes. Locating a file does not, so
/// this line **is** live and the field's mistakes are visible where they are
/// made.
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
///
/// ★★★ **This control is drawn only when a store was actually found**, which is
/// R9: an unavailable capability renders nothing, and greying is reserved for
/// something that is *temporarily* unavailable. A person with no Acrobat store
/// is not one press away from having one. The path field above stays visible in
/// that case, because it is the remedy — and R9's rule cuts both ways: an
/// absent capability whose remedy is also absent is a dead end.
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

    /// ★★★ **`NotChecked` says "not checked", in every one of its four
    /// explanations.**
    ///
    /// The property this whole feature stands on, asserted over the sentence
    /// rather than trusted to the layout. All four branches go through
    /// [`not_checked`], so the words cannot be dropped from one of them — and
    /// this test would catch it if the funnel were bypassed, because it checks
    /// each explanation through that funnel.
    ///
    /// ⚠ **When this fails, the fix is the sentence, not this test.** A softer
    /// wording — "trust unavailable", "no trust information" — is precisely the
    /// drift it exists to refuse: those read as *nothing is wrong*, and the
    /// whole point is that pdfcer has not looked.
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

    /// ★★★ **The four explanations are four different sentences.**
    ///
    /// Not a tautology: the cheap implementation of this feature has one
    /// "trust was not checked" string and four call sites, and it would pass
    /// every other test in this file. The four situations call for four
    /// different actions — turn the setting on, install Acrobat, fix your typo,
    /// your store is corrupt — and collapsing any two of them tells somebody to
    /// do the wrong thing.
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

    /// ★★★ **A `Trusted` verdict discloses that revocation was not checked.**
    ///
    /// The engine attaches that disclosure to every `Trusted` note it produces
    /// and the whole design rests on the shell not dropping it. This is the
    /// assertion that stops a future edit shortening the sentence to *"trusted
    /// — chains to X"*, which is what every other PDF reader says and is the
    /// one thing pdfcer must not say without the qualification.
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
    ///
    /// The single likeliest misreading on this surface: an operator sees
    /// `Signer: does NOT chain…` beside an intact signature and concludes the
    /// document was altered. The sentence has to separate the two claims
    /// itself, because it is read alone.
    #[test]
    fn untrusted_separates_itself_from_integrity() {
        let line = untrusted("the chain is incomplete");
        assert!(
            line.contains("not about whether the bytes are intact"),
            "{line}"
        );
    }

    /// **The store's count and its date are one sentence.**
    ///
    /// `ENGINE_BACKLOG.md`: *"an anchor set that silently went stale is worse
    /// than one that was never imported."* There is deliberately no accessor
    /// that yields the count without the date, and this asserts the one that
    /// exists carries both — including the honest sentence for a store whose
    /// date could not be read.
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
    ///
    /// Cheap, and it is what stops a tidy-up merging two of the columns. The
    /// engine's design note is that the three never collapse into one; a shared
    /// label is the first step of collapsing them.
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
