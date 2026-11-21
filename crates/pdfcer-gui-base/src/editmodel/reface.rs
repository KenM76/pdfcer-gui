//! # `editmodel::reface` — the characters of a line edit set in another face

/// Characters of a line edit that the run's font lacks, to be set in `face`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reface {
    /// The selector the engine's fallback names (`FallbackFace::Named`).
    pub face: String,
    /// The face as the operator reads its name.
    pub label: String,
    /// The characters to set in `face`.
    pub chars: Vec<char>,
}
