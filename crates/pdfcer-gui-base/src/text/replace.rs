//! # `text::replace` — every string the Find bar's Replace row shows or causes
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/replace.md`.

/// The toggle on the Find bar that shows the Replace row.
#[must_use]
pub fn toggle() -> &'static str {
    "Replace"
}

/// Hover text for the toggle.
#[must_use]
pub fn toggle_tooltip() -> &'static str {
    "Show or hide the Replace row, which rewrites what this search found."
}

/// The word in front of the replacement field.
#[must_use]
pub fn field_label() -> &'static str {
    "With"
}

/// Hover text for the replacement field.
#[must_use]
pub fn field_tooltip() -> &'static str {
    "The text that replaces each hit, exactly as typed. Leave it empty to delete the hits."
}

/// The button that replaces the current hit.
#[must_use]
pub fn one() -> &'static str {
    "Replace"
}

/// Hover text for [`one`].
#[must_use]
pub fn one_tooltip() -> &'static str {
    "Replace the highlighted hit and go to the next one. Undo puts it back."
}

/// The button that replaces every hit.
#[must_use]
pub fn all() -> &'static str {
    "Replace all"
}

/// Hover text for [`all`].
#[must_use]
pub fn all_tooltip() -> &'static str {
    "Replace every hit this search found, on every page. One Undo puts them all back."
}

/// Hover text on either button while there are no current hits to replace.
#[must_use]
pub fn unavailable_tooltip() -> &'static str {
    "Search first: Replace works on the hits the Find bar is showing, and there are none \
     right now."
}

/// Why a Replace press changed nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplaceRefusal {
    /// The search used wildcards, whose hits name a pattern rather than a text.
    Wildcards,
    /// None of the `found` hits lay inside text pdfcer can rewrite in place.
    NothingRewritable {
        /// How many hits the search had.
        found: usize,
    },
}

impl ReplaceRefusal {
    /// The status line's sentence.
    #[must_use]
    pub fn line(self) -> String {
        match self {
            Self::Wildcards => {
                "Replace does not work with wildcards. Turn Wildcards off in the Find options \
                 and search for the exact text."
                    .to_owned()
            }
            Self::NothingRewritable { found } => nothing_rewritable(found),
        }
    }
}

/// The sentence when no hit could be replaced.
#[must_use]
pub fn nothing_rewritable(found: usize) -> String {
    format!(
        "Nothing was replaced: none of the {found} {} lies inside text pdfcer can rewrite in \
         place.",
        hits(found)
    )
}

/// The first disclosure after a replace: how many of how many.
#[must_use]
pub fn summary(replaced: usize, found: usize) -> String {
    if replaced == found {
        format!("Replaced {replaced} {}.", hits(replaced))
    } else {
        format!("Replaced {replaced} of {found} {}.", hits(found))
    }
}

/// Hits left because no single line holds them.
#[must_use]
pub fn left_not_on_one_line(n: usize) -> String {
    format!(
        "Left unchanged: {n} {} that {} across two lines.",
        hits(n),
        runs(n)
    )
}

/// Hits left because they cross a change of font, size or colour.
#[must_use]
pub fn left_crosses_styles(n: usize) -> String {
    format!(
        "Left unchanged: {n} {} that {} across a change of font, size or colour. Edit {} \
         with the Edit text tool.",
        hits(n),
        runs(n),
        them(n)
    )
}

/// Hits left because the text drawing them also draws another line.
#[must_use]
pub fn left_shared_operator(n: usize) -> String {
    format!(
        "Left unchanged: {n} {} drawn by text that also draws another line. Edit {} with \
         the Edit text tool.",
        hits(n),
        them(n)
    )
}

/// Hits the engine declined to rewrite.
#[must_use]
pub fn left_refused(n: usize) -> String {
    format!(
        "Left unchanged: {n} {} in text that cannot be rewritten, often because its font \
         lacks a character of the replacement.",
        hits(n)
    )
}

fn hits(n: usize) -> &'static str {
    if n == 1 { "hit" } else { "hits" }
}

fn runs(n: usize) -> &'static str {
    if n == 1 { "runs" } else { "run" }
}

fn them(n: usize) -> &'static str {
    if n == 1 { "it" } else { "them" }
}
