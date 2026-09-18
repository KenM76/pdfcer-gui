//! # `text::settings::redaction` — the words on the one setting that destroys
//!
//! One group, one setting: how far applying a redaction may reach beyond the
//! regions the operator marked.
//!
//! ## Why the copy here is written harder than the rest of the window
//!
//! Every other setting in this window changes how a document is *read*, *drawn*
//! or *written*. A wrong answer is a document that looks wrong, and reopening
//! it with the other answer undoes the mistake. This one decides what is
//! **removed and cannot be recovered**, in both directions: the narrow end can
//! leave a phrase in a file the operator believes is clean, and the wide end
//! can delete text off a page they never looked at.
//!
//! So each option's note says what it *leaves behind* as well as what it takes,
//! because a scale where every step reads as an improvement is a scale with the
//! cost hidden at one end.

/// Group 9 — the one setting that decides what a redaction destroys.
#[must_use]
pub const fn group_redaction() -> &'static str {
    "Redacting"
}

/// Residual sweep: what it is.
#[must_use]
pub const fn reach_title() -> &'static str {
    "How far a redaction reaches beyond what you marked"
}

/// Residual sweep: what the standard leaves open.
///
/// It leaves it genuinely open, and this is one of the few settings in the
/// window where naming the clause earns its place: §12.5.6.23 states the
/// obligation as an *outcome* over all the content a document can hold, and
/// says nothing about where that content may be. Two readings both satisfy it
/// and they disagree about the operator's unmarked pages.
#[must_use]
pub const fn reach_silence() -> &'static str {
    "The standard says redacted text must not survive anywhere in the file, and \
     does not say whether that licenses editing parts of the document you did \
     not mark. Both readings comply."
}

/// Residual sweep: what changing it costs.
#[must_use]
pub const fn reach_radius() -> &'static str {
    "Changes what applying a redaction PERMANENTLY REMOVES from the file it \
     writes. It does not change what you see on screen, and it never changes \
     what is reported: every setting tells you everything it found."
}

/// One sweep setting's name.
#[must_use]
pub const fn reach_label(reach: crate::app::prefs::RedactionReach) -> &'static str {
    use crate::app::prefs::RedactionReach as S;
    match reach {
        S::MarkedOnly => "Only what I marked",
        S::HiddenCarriers => "What I marked, and the parts I cannot see (pdfcer's default)",
        S::WholeDocument => "Everywhere the same text appears",
    }
}

/// One sweep setting's description.
///
/// Each says what it leaves behind, not only what it takes — see the module
/// header. The middle one is also the only one carrying the reasoning behind
/// the default, which is obligation 1: a default rests on an argument and the
/// argument belongs where the default is.
#[must_use]
pub const fn reach_note(reach: crate::app::prefs::RedactionReach) -> &'static str {
    use crate::app::prefs::RedactionReach as S;
    match reach {
        S::MarkedOnly => {
            "The literal reading: the content under your marks goes, and nothing \
             else is touched. Leaves the most behind — the same words can \
             survive in the document title, the keywords, or an XMP packet, \
             none of which show on the page. pdfcer still finds them and still \
             tells you."
        }
        S::HiddenCarriers => {
            "Also clears the places a copy of the text can hide where you would \
             never see it: the document information, XMP packets, and text \
             entries in the file's own structures. Leaves every page you did \
             not mark exactly as it was. The default because nobody chooses to \
             keep a keyword, and nobody notices when it goes — whereas a page \
             is the document, and deleting an unmarked word off one is \
             destruction wearing diligence as a disguise."
        }
        S::WholeDocument => {
            "Also blanks the same text wherever it is drawn, on pages you never \
             marked. The strongest promise that the phrase is gone from the \
             artifact, and the only setting here that edits a page you did not \
             look at. Redacting a common word under this setting removes it \
             from the whole drawing set."
        }
    }
}
