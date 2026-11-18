//! Copy for Format ▸ Nodes ▸ Continue path and Open path here: why the
//! command did nothing, and what a finished run says.

use pdfcer_core::edit::EditError;
use pdfcer_core::vector::VectorEditError;

/// Why Continue path or Open path here changed nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathEditRefusal {
    /// Not exactly one node is selected.
    NotOneNode,
    /// Continue path: the node is not the first or last node of its subpath.
    NotAnEnd,
    /// Continue path: the subpath is closed, so it has no end to draw on from.
    Closed,
    /// Open path here: the subpath is already open.
    AlreadyOpen,
    /// Open path here: the subpath begins without a move-to, so there is
    /// nowhere to put its new start.
    StartsImplicitly,
    /// Continue path: the run was finished before any click.
    NoClicks,
    /// The selected path no longer exists as selected.
    Stale,
    /// Anything else the engine refused.
    Other,
}

impl PathEditRefusal {
    /// Classify the session verb's answer.
    #[must_use]
    pub fn of_edit(error: &EditError) -> Self {
        match error {
            EditError::VectorEdit(inner) => Self::of_vector(inner),
            EditError::PageOutOfRange { .. } | EditError::FormLeafOutOfRange { .. } => Self::Stale,
            _ => Self::Other,
        }
    }

    /// Classify an engine refusal.
    #[must_use]
    pub fn of_vector(error: &VectorEditError) -> Self {
        match error {
            VectorEditError::NotAnEndNode { .. } => Self::NotAnEnd,
            VectorEditError::SubpathClosed { .. } => Self::Closed,
            VectorEditError::SubpathAlreadyOpen { .. } => Self::AlreadyOpen,
            VectorEditError::SubpathStartsImplicitly { .. } => Self::StartsImplicitly,
            VectorEditError::NoSegmentsGiven => Self::NoClicks,
            VectorEditError::NodeOutOfRange { .. } | VectorEditError::ObjectOutOfRange { .. } => {
                Self::Stale
            }
            _ => Self::Other,
        }
    }

    /// The trace token, stable for `ui-verify`.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::NotOneNode => "not-one-node",
            Self::NotAnEnd => "not-an-end",
            Self::Closed => "closed",
            Self::AlreadyOpen => "already-open",
            Self::StartsImplicitly => "starts-implicitly",
            Self::NoClicks => "no-clicks",
            Self::Stale => "stale",
            Self::Other => "other",
        }
    }

    /// The status-line sentence.
    #[must_use]
    pub const fn line(self) -> &'static str {
        match self {
            Self::NotOneNode => "Select exactly one node of the path first.",
            Self::NotAnEnd => {
                "A path can only be continued from its first or last node. Select an end node. \
                 Nothing was changed."
            }
            Self::Closed => {
                "This path is closed, so it has no end to continue from. Use Open path here \
                 first. Nothing was changed."
            }
            Self::AlreadyOpen => {
                "This path is already open, so there is nothing to break. Nothing was changed."
            }
            Self::StartsImplicitly => {
                "This part of the path has no starting point of its own in the file, so it \
                 cannot be opened. Nothing was changed."
            }
            Self::NoClicks => "Nothing was added: click where the path should go before finishing.",
            Self::Stale => "The selected path changed before the edit ran. Select it again.",
            Self::Other => "pdfcer could not make this change to the path. Nothing was changed.",
        }
    }
}

/// The status-line prompt while a Continue path run is in progress.
#[must_use]
pub const fn continue_prompt() -> &'static str {
    "Continuing the path: click to add segments. Double-click, Enter or Esc finishes; \
     Backspace takes back a click."
}

#[cfg(test)]
mod tests {
    use super::PathEditRefusal as R;

    const ALL: [R; 8] = [
        R::NotOneNode,
        R::NotAnEnd,
        R::Closed,
        R::AlreadyOpen,
        R::StartsImplicitly,
        R::NoClicks,
        R::Stale,
        R::Other,
    ];

    #[test]
    fn every_refusal_has_its_own_sentence_and_token() {
        for (i, a) in ALL.iter().enumerate() {
            assert!(a.line().ends_with('.'), "{a:?}");
            for b in &ALL[i + 1..] {
                assert_ne!(a.line(), b.line(), "{a:?} and {b:?} share a sentence");
                assert_ne!(a.token(), b.token(), "{a:?} and {b:?} share a token");
            }
        }
    }

    #[test]
    fn the_engine_refusals_classify_by_name() {
        use pdfcer_core::vector::VectorEditError as E;
        assert_eq!(R::of_vector(&E::NotAnEndNode { index: 1 }), R::NotAnEnd);
        assert_eq!(R::of_vector(&E::SubpathClosed { index: 0 }), R::Closed);
        assert_eq!(
            R::of_vector(&E::SubpathAlreadyOpen { index: 0 }),
            R::AlreadyOpen
        );
        assert_eq!(R::of_vector(&E::NoSegmentsGiven), R::NoClicks);
    }
}
