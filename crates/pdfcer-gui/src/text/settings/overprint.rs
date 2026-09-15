//! # `text::settings::overprint` — the zero-tint rule, in the operator's words
//!
//!
//! ## ★★★ What this module exists to keep straight
//!
//! ISO 32000-1 §8.6.7 says a zero tint in an overprinting colour leaves what is
//! underneath it showing. The argument is over *which* colours count, and pdfcer
//! has changed its mind once, for a good reason:
//!
//!
//! ⇒ Which is why nothing in this module writes down *which* option is the
//! default. It asks. See [`zero_tint_default_suffix`].

// ===========================================================================
// Overprint — which colours get the zero-tint rule
// ===========================================================================

/// Zero-tint scope: what it is.
///
/// ★★★ **The title asks about GREY, not about "OPM 1's scope".** The engine's
/// own account of this setting is four screens on a genuine ambiguity in
/// §8.6.7; the operator's version of the same question is *"does a grey fill
/// wipe out the spot colour underneath it, or not?"* — which is what he would
/// have seen on paper and what would send him looking.
///
/// ⇒ The window's rule is that a heading names the SYMPTOM. A heading reading
/// *"Overprint zero-tint scope"* is the field name, and a field name is
/// findable only by somebody who already knows the answer.
#[must_use]
pub const fn zero_tint_title() -> &'static str {
    "Grey over a spot colour in print-ready files"
}

/// What happens if you never touch it.
#[must_use]
pub const fn zero_tint_silence() -> &'static str {
    "A grey overprinting a spot colour leaves the spot showing through, the way Acrobat draws it."
}

/// What it costs, and what it does not affect.
///
/// ★ It names the same narrow reach the blend-space setting does, because it
/// has the same one: nothing happens on a file that never asked for overprint,
/// which is nearly every file that is not print-ready.
#[must_use]
pub const fn zero_tint_radius() -> &'static str {
    "Changes how overprinted areas are drawn and printed. It never changes the file, and it does nothing at all unless a page actually asks for overprint — which almost none do outside print-ready artwork."
}

/// One scope's name.
#[must_use]
pub const fn zero_tint_label(scope: pdfcer_core::settings::OverprintZeroTintScope) -> &'static str {
    use pdfcer_core::settings::OverprintZeroTintScope as S;
    match scope {
        S::GreyAsKOnly => "Let grey behave like black ink",
        S::DeviceCmykOnly => "What the standard literally says",
        S::AllProcessSpaces => "Let every colour behave like ink",
        // ★ `#[non_exhaustive]`, so a newer engine may add a scope. Named as
        // unknown rather than folded onto a neighbour — see `blend_space_label`
        // for the argument, which is the same one and is made once there.
        _ => "A newer pdfcer added this option; this build cannot describe it",
    }
}

/// **The suffix that marks whichever scope is currently the default.**
///
/// ★★★ DERIVED from `OverprintZeroTintScope::default()`, never written into a
/// label — and that is the whole point of it existing.
///
///
/// The label went on saying "(pdfcer's default)" about the option that was no
/// longer it — a sentence true when written and silently false afterwards,
/// which is the class this project has now met a dozen times. The engine's own
/// note is what caught it, and its advice was exactly this: *"if you read
/// `OverprintZeroTintScope::default()` to decide that, nothing to do."*
#[must_use]
pub fn zero_tint_default_suffix(
    scope: pdfcer_core::settings::OverprintZeroTintScope,
) -> &'static str {
    if scope == pdfcer_core::settings::OverprintZeroTintScope::default() {
        "  (pdfcer's default)"
    } else {
        ""
    }
}

/// One scope's description.
///
/// ★★ Each note says what comes out on paper, with the measured numbers where
/// there are any and an admission where there are none. The third option is
/// unmeasured and its note says so in the operator's terms — *"nobody has
/// checked"* — rather than leaving him to infer it from the absence of a
/// figure.
#[must_use]
pub const fn zero_tint_note(scope: pdfcer_core::settings::OverprintZeroTintScope) -> &'static str {
    use pdfcer_core::settings::OverprintZeroTintScope as S;
    match scope {
        S::GreyAsKOnly => {
            "A grey fill is treated as black ink alone. This is what pdfcer did before September 2026, and what Acrobat does. Choose it if a page changed appearance in this build and you want the old rendering back."
        }
        S::DeviceCmykOnly => {
            "Only a colour written as CMYK gets the rule, which is what the standard's sentence literally says. A grey over a SPOT colour still leaves the spot showing; a grey over process ink knocks it out. This scores best on the print test suite — 43 of 51 cells pass and none fail, against two failures for the option above."
        }
        S::AllProcessSpaces => {
            "Extends the rule to red-green-blue colours as well as grey. The most consistent reading, but nobody has checked it against Acrobat: pdfcer's red-green-blue to ink conversion is a rough one, so a pure red could preserve a cyan backdrop where Acrobat would not."
        }
        _ => "Not described by this build.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::settings::OverprintZeroTintScope as Scope;

    /// **★★★ Exactly one scope is marked as the default, and it is the one the
    /// ENGINE says is the default.**
    ///
    ///
    /// ★ Asserting **exactly one** rather than "the right one carries it"
    /// catches the other half: a suffix added to a second label by hand, which
    /// would leave two options both claiming to be what pdfcer does.
    #[test]
    fn exactly_one_scope_is_marked_as_the_engines_default() {
        let all = [
            Scope::GreyAsKOnly,
            Scope::DeviceCmykOnly,
            Scope::AllProcessSpaces,
        ];
        let marked: Vec<Scope> = all
            .into_iter()
            .filter(|s| !zero_tint_default_suffix(*s).is_empty())
            .collect();
        assert_eq!(
            marked,
            vec![Scope::default()],
            "the default marker must name the engine's own default and nothing else"
        );
    }

    /// **★★ No label may hard-code the word "default".**
    ///
    /// The suffix is derived; a label that spells it out would be a second,
    /// unsynchronised claim about the same fact — which is exactly how the
    /// original defect happened.
    #[test]
    fn no_scope_label_spells_out_the_default_itself() {
        for scope in [
            Scope::GreyAsKOnly,
            Scope::DeviceCmykOnly,
            Scope::AllProcessSpaces,
        ] {
            let label = zero_tint_label(scope);
            assert!(
                !label.to_lowercase().contains("default"),
                "a label states the default instead of deriving it: {label}"
            );
        }
    }
}
