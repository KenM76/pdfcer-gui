//! What pdfcer's drawing of a field showed in place of, or ignored from, its
//! stored state ([`DisplayNotes`]), as the Forms panel's per-field notes.

use pdfcer_core::field_display::DisplayNotes;

/// One note's kind, as the trace names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldNoteKind {
    Formatted,
    Unformatted,
    IgnoredTopIndex,
    IgnoredSelected,
    RichText,
    RichIgnored,
    RichFonts,
    RichUnreadable,
    SignatureText,
    SignatureOmitted,
}

impl FieldNoteKind {
    /// The trace's spelling.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        // ui-text-exempt: trace values, never displayed
        match self {
            Self::Formatted => "formatted",
            Self::Unformatted => "unformatted",
            Self::IgnoredTopIndex => "ignored-top-index",
            Self::IgnoredSelected => "ignored-selected",
            Self::RichText => "rich-text",
            Self::RichIgnored => "rich-ignored",
            Self::RichFonts => "rich-fonts",
            Self::RichUnreadable => "rich-unreadable",
            Self::SignatureText => "signature-text",
            Self::SignatureOmitted => "signature-omitted",
        }
    }
}

fn joined(items: &[impl ToString]) -> String {
    items
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Every note `notes` carries, in a fixed order, each with its sentence.
#[must_use]
pub fn field_display_notes(notes: &DisplayNotes) -> Vec<(FieldNoteKind, String)> {
    use FieldNoteKind as K;
    let mut out = Vec::new();
    if notes.formatted {
        out.push((
            K::Formatted,
            "Shown as its format script displays it; the stored value is unchanged.".to_owned(),
        ));
    }
    if let Some(why) = &notes.unformatted {
        out.push((K::Unformatted, format!("Shown as stored: {why}.")));
    }
    if let Some(ti) = notes.ignored_top_index {
        out.push((
            K::IgnoredTopIndex,
            format!("The list's first shown row is set to {ti}, which names no option, so it is ignored."),
        ));
    }
    if !notes.ignored_selected.is_empty() {
        out.push((
            K::IgnoredSelected,
            format!(
                "Selected rows {} name no option, so they are ignored.",
                joined(&notes.ignored_selected)
            ),
        ));
    }
    if notes.rich_text {
        out.push((K::RichText, "Drawn from its rich text.".to_owned()));
    }
    if !notes.rich_ignored.is_empty() {
        out.push((
            K::RichIgnored,
            format!(
                "Rich-text styling not drawn: {}.",
                joined(&notes.rich_ignored)
            ),
        ));
    }
    if !notes.rich_fonts.is_empty() {
        out.push((
            K::RichFonts,
            format!("Drawn with other fonts: {}.", joined(&notes.rich_fonts)),
        ));
    }
    if let Some(why) = &notes.rich_unreadable {
        out.push((
            K::RichUnreadable,
            format!("Its rich text could not be read, so the plain value is shown: {why}."),
        ));
    }
    signature_notes(notes, &mut out);
    out
}

/// What a rich-text fill of `field` owes the status line: the formatting
/// its earlier value held that the typed text replaced (`before`, parsed
/// without `/DS`, which the fill keeps), then what drawing the new value
/// reported.
#[must_use]
pub fn rich_fill_notes(
    field: &str,
    before: &[pdfcer_core::richtext::Run],
    notes: &DisplayNotes,
) -> Vec<String> {
    let mut out = Vec::new();
    let replaced = crate::text::richtext::formatting(before);
    if !replaced.is_empty() {
        out.push(format!(
            "{field}: the typed text replaced the formatting it held ({}); it now takes the \
             field's default style.",
            replaced.join(", ")
        ));
    }
    for (kind, sentence) in field_display_notes(notes) {
        if matches!(
            kind,
            FieldNoteKind::RichIgnored | FieldNoteKind::RichFonts | FieldNoteKind::RichUnreadable
        ) {
            out.push(format!("{field}: {sentence}"));
        }
    }
    out
}

fn signature_notes(notes: &DisplayNotes, out: &mut Vec<(FieldNoteKind, String)>) {
    if !notes.signature_text.is_empty() {
        out.push((
            FieldNoteKind::SignatureText,
            format!(
                "The signature has no picture of its own; pdfcer drew its signer and date, \
                 which say nothing about whether it is valid: {}.",
                notes.signature_text.join(" / ")
            ),
        ));
    }
    if !notes.signature_omitted.is_empty() {
        out.push((
            FieldNoteKind::SignatureOmitted,
            format!(
                "Left out of the signature's drawing: {}.",
                joined(&notes.signature_omitted)
            ),
        ));
    }
}

/// The heading over a field's notes, shown on hover.
#[must_use]
pub fn field_display_notes_tooltip() -> &'static str {
    "What pdfcer's drawing of this field shows differently from what is stored in it. The \
     stored value is never changed."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_draw_has_no_notes() {
        assert!(field_display_notes(&DisplayNotes::default()).is_empty());
    }

    #[test]
    fn the_engines_reason_is_carried_whole() {
        let mut n = DisplayNotes::default();
        n.unformatted = Some("the format action is not recognised".to_owned());
        n.ignored_top_index = Some(7);
        let got = field_display_notes(&n);
        assert_eq!(got[0].0, FieldNoteKind::Unformatted);
        assert!(got[0].1.contains("the format action is not recognised"));
        assert_eq!(got[1].0, FieldNoteKind::IgnoredTopIndex);
        assert!(got[1].1.contains('7'));
    }
}
