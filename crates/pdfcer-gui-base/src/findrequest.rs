//! # `findrequest` — what a Find asks for and which way it steps

/// Which way [`FindRequest::Step`] moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Toward the end of the document, wrapping to the first hit.
    Next,
    /// Toward the start, wrapping to the last hit.
    Previous,
}
/// One thing the operator asked Find to do, carried by
/// `pdfcer_gui::app::actions::Action::Find`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindRequest {
    /// Run the search now, for whatever is in the bar.
    Search,
    /// Move to the adjacent hit.
    Step(Step),
    /// Rewrite the current hit, or every hit when `all`, with the Replace
    /// row's text; carried out by the app, which owns document edits.
    Replace {
        /// Every hit rather than the current one.
        all: bool,
    },
}
