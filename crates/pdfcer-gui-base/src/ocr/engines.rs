//! Which recognisers this build carries.
//! Loading and running a model is `pdfcer_ocr_host::OcrRunner`.
//!
//! [`EngineId`] names every engine this shell knows how to drive, compiled in
//! or not, so a preference naming one survives a build that lacks it.
//! [`available`] is the only answer to *which can run here*: the dialog offers
//! exactly that list and asks nothing else (R8 — presence is what this build
//! linked, never a `cfg` in a surface). The first entry is the default choice.
//!
//! Every engine returns words in the input image's pixel space, y-down;
//! `words_to_page_space_on` is the one place they are flipped, for all of them.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/ocr/engines.md`.

/// A recogniser this shell can drive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EngineId {
    /// `ocrs`: two neural networks; scores nothing.
    Ocrs,
    /// OCRcer: prototype matching with a calibrated per-word confidence.
    Ocrcer,
    /// PaddleOCR (PP-OCR): operator-supplied ONNX exports; scores every word.
    Paddle,
}

impl EngineId {
    /// Every engine, in preference order. [`available`] filters this.
    pub const ALL: [Self; 3] = [Self::Ocrs, Self::Ocrcer, Self::Paddle];

    /// The stable key: the preferences-file value and the trace token.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Ocrs => "ocrs",
            Self::Ocrcer => "ocrcer",
            Self::Paddle => "paddle",
        }
    }

    /// The engine a [`Self::key`] names, if any.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|e| e.key() == key)
    }

    /// Whether this build linked the engine.
    #[must_use]
    pub const fn compiled_in(self) -> bool {
        match self {
            Self::Ocrs => cfg!(feature = "ocrs"),
            Self::Ocrcer => cfg!(feature = "ocrcer"),
            Self::Paddle => cfg!(feature = "paddle"),
        }
    }
}

/// The engines this build can run, default first. Empty in a build with none.
#[must_use]
pub fn available() -> Vec<EngineId> {
    EngineId::ALL
        .into_iter()
        .filter(|e| e.compiled_in())
        .collect()
}

/// OCRcer's single model file; the engine's `engine_ocrcer::MODEL_FILE`,
/// which exists only when the feature is on. A test holds the two together.
pub const OCRCER_MODEL_FILE: &str = "ocrcer.ocrw";

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(any(feature = "paddle", feature = "ocrcer"))]
    use pdfcer_ocr_host::RunnerError;

    #[test]
    fn every_key_round_trips_and_no_two_engines_share_one() {
        for e in EngineId::ALL {
            assert_eq!(EngineId::from_key(e.key()), Some(e));
        }
        assert_ne!(EngineId::Ocrs.key(), EngineId::Ocrcer.key());
        assert_eq!(EngineId::from_key("tesseract"), None);
    }

    /// `ocrs` stays the default choice: the engine's ruling is that the
    /// head-to-head decides the default, and it has not.
    #[test]
    #[cfg(feature = "ocrs")]
    fn ocrs_is_offered_first() {
        assert_eq!(available().first(), Some(&EngineId::Ocrs));
    }

    #[test]
    fn only_linked_engines_are_offered() {
        for e in available() {
            assert!(e.compiled_in(), "{e:?} offered but not linked");
        }
        assert_eq!(
            available().contains(&EngineId::Ocrcer),
            cfg!(feature = "ocrcer")
        );
    }

    #[test]
    #[cfg(feature = "ocrcer")]
    fn the_ocrcer_names_are_the_engines() {
        assert_eq!(
            OCRCER_MODEL_FILE,
            pdfcer_core::ocr::engine_ocrcer::MODEL_FILE
        );
    }

    /// Load a bare model folder of `engine` the way a run does.
    #[cfg(any(feature = "paddle", feature = "ocrcer"))]
    fn load_bare(engine: EngineId, dir: &std::path::Path) -> RunnerError {
        let model = pdfcer_core::ocr::addons::OcrModel {
            name: engine.key().to_owned(),
            engine: engine.key().to_owned(),
            folder: dir.to_path_buf(),
            root: dir.to_path_buf(),
            manifest: None,
        };
        match pdfcer_ocr_host::OcrRunner::load(
            &model,
            &pdfcer_ocr_host::RunOptions::new("eng", 300.0),
        ) {
            Ok(_) => panic!("a bare {} folder loaded", engine.key()),
            Err(e) => e,
        }
    }

    /// PaddleOCR files that are not ONNX models are a named engine refusal.
    #[test]
    #[cfg(feature = "paddle")]
    fn a_corrupt_paddle_model_is_refused_by_the_engine() {
        let dir = std::env::temp_dir().join(format!("pdfcer-paddle-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        use pdfcer_core::ocr::engine_paddle::{DETECTION_MODEL, RECOGNITION_MODEL};
        for f in [DETECTION_MODEL, RECOGNITION_MODEL] {
            std::fs::write(dir.join(f), b"not a model").expect("write");
        }
        let err = load_bare(EngineId::Paddle, &dir);
        assert!(matches!(err, RunnerError::Engine(_)), "{err:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A model file that is not an `.ocrw` container is a named engine
    /// refusal, never a panic and never an empty page.
    #[test]
    #[cfg(feature = "ocrcer")]
    fn a_corrupt_ocrcer_model_is_refused_by_the_engine() {
        let dir = std::env::temp_dir().join(format!("pdfcer-ocrcer-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        std::fs::write(dir.join(OCRCER_MODEL_FILE), b"not a model").expect("write");
        let err = load_bare(EngineId::Ocrcer, &dir);
        assert!(matches!(err, RunnerError::Engine(_)), "{err:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
