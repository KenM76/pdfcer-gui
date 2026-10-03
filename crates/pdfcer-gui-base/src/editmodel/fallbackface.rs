//! # `editmodel::fallbackface` — fallback faces with the `'static` lifetime
//! `EditOptions::with_fallback` asks for
//!
//! [`named`] answers one leaked `FallbackFace::Named` per distinct selector
//! and the same reference on every later call, so the leak is bounded by the
//! number of faces the operator ever falls back to in a session.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/editmodel/fallbackface.md`.

use std::sync::{Mutex, PoisonError};

use pdfcer_core::text_edit::{FallbackFace, FallbackSource};

static FACES: Mutex<Vec<&'static FallbackFace>> = Mutex::new(Vec::new());

/// The interned `FallbackFace::Named(selector)`.
#[must_use]
pub fn named(selector: &str) -> &'static FallbackFace {
    let mut faces = FACES.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(face) = faces
        .iter()
        .find(|f| matches!(f, FallbackFace::Named(n) if n == selector))
    {
        return face;
    }
    let face: &'static FallbackFace = Box::leak(Box::new(FallbackFace::Named(selector.to_owned())));
    faces.push(face);
    face
}

/// The trace token for where a fallback face's resource came from.
#[must_use]
pub const fn source_token(source: FallbackSource) -> &'static str {
    match source {
        FallbackSource::PageResource => "page",
        FallbackSource::AddedStandard14 => "standard14",
        FallbackSource::EmbeddedSubset => "embedded",
        FallbackSource::SameProgram => "same-program",
        _ => "other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_selector_is_one_face_and_two_are_two() {
        let a = named("Helvetica");
        assert!(std::ptr::eq(a, named("Helvetica")));
        assert!(!std::ptr::eq(a, named("Times-Roman")));
        assert_eq!(a, &FallbackFace::Named("Helvetica".to_owned()));
    }
}
