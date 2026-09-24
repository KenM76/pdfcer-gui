//! Which recognisers this build carries, where each one's model lives, and the
//! loaded model a run holds.
//!
//! [`EngineId`] names every engine this shell knows how to drive, compiled in
//! or not, so a preference naming one survives a build that lacks it.
//! [`available`] is the only answer to *which can run here*: the dialog offers
//! exactly that list and asks nothing else (R8 — presence is what this build
//! linked, never a `cfg` in a surface). The first entry is the default choice.
//!
//! Every engine returns words in the input image's pixel space, y-down;
//! `words_to_page_space_on` is the one place they are flipped, for all of them.

use std::path::Path;

use pdfcer_core::ocr::{RecognizedWord, models};

use super::Refusal;

/// A recogniser this shell can drive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EngineId {
    /// `ocrs`: two neural networks; scores nothing.
    Ocrs,
    /// OCRcer: prototype matching with a calibrated per-word confidence.
    Ocrcer,
}

impl EngineId {
    /// Every engine, in preference order. [`available`] filters this.
    pub const ALL: [Self; 2] = [Self::Ocrs, Self::Ocrcer];

    /// The stable key: the preferences-file value and the trace token.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Ocrs => "ocrs",
            Self::Ocrcer => "ocrcer",
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
        }
    }

    /// The directory under a model root that holds this engine's files.
    ///
    /// The engine's own constant where it publishes one, so a rename there
    /// cannot leave this resolving a directory the engine then refuses.
    #[must_use]
    pub const fn model_dir(self) -> models::EngineDirName {
        match self {
            #[cfg(feature = "ocrs")]
            Self::Ocrs => pdfcer_core::ocr::engine_ocrs::MODEL_DIR,
            #[cfg(not(feature = "ocrs"))]
            Self::Ocrs => "ocrs",
            Self::Ocrcer => OCRCER_MODEL_DIR,
        }
    }

    /// The files a model directory must hold to count as found.
    ///
    /// Empty for an engine this build did not link: resolution then only has
    /// to name where the files would have gone.
    #[must_use]
    pub const fn model_files(self) -> &'static [&'static str] {
        match self {
            #[cfg(feature = "ocrs")]
            Self::Ocrs => &[
                pdfcer_core::ocr::engine_ocrs::DETECTION_MODEL,
                pdfcer_core::ocr::engine_ocrs::RECOGNITION_MODEL,
            ],
            #[cfg(feature = "ocrcer")]
            Self::Ocrcer => &[OCRCER_MODEL_FILE],
            #[allow(unreachable_patterns)]
            _ => &[],
        }
    }

    /// Whether the engine scores its words.
    ///
    /// The value its `OcrEngine::reports_confidence` returns, stated per
    /// engine so the dialog can word its disclosure before a model is loaded.
    /// Each page is stamped from the LOADED engine instead, and `recognise`
    /// debug-asserts the two agree.
    #[must_use]
    pub const fn reports_confidence(self) -> bool {
        match self {
            Self::Ocrs => false,
            Self::Ocrcer => true,
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

/// OCRcer's directory under a model root. Equal to the engine's
/// `engine_ocrcer::MODEL_DIR`, which exists only when the feature is on; a
/// test holds the two together.
pub const OCRCER_MODEL_DIR: models::EngineDirName = "ocrcer";

/// OCRcer's single model file, inside [`OCRCER_MODEL_DIR`]; the engine's
/// `engine_ocrcer::MODEL_FILE`.
pub const OCRCER_MODEL_FILE: &str = "ocrcer.ocrw";

/// A loaded model, held for a whole run so a hundred pages load it once.
/// Boxed: the two engines differ in size by kilobytes.
pub(super) enum Recogniser {
    #[cfg(feature = "ocrs")]
    Ocrs(Box<pdfcer_core::ocr::engine_ocrs::OcrsEngine>),
    #[cfg(feature = "ocrcer")]
    Ocrcer(Box<pdfcer_core::ocr::engine_ocrcer::OcrcerEngine>),
}

impl Recogniser {
    /// Load `engine`'s model from `model_dir`. Refuses before any page is
    /// rasterised, so a build without the engine spends no time rendering.
    pub(super) fn load(engine: EngineId, model_dir: &Path) -> Result<Self, Refusal> {
        let _ = model_dir;
        match engine {
            #[cfg(feature = "ocrs")]
            EngineId::Ocrs => pdfcer_core::ocr::engine_ocrs::OcrsEngine::from_model_dir(model_dir)
                .map(|e| Self::Ocrs(Box::new(e)))
                .map_err(|e| Refusal::Engine(e.to_string())),
            #[cfg(feature = "ocrcer")]
            EngineId::Ocrcer => {
                let bytes = std::fs::read(model_dir.join(OCRCER_MODEL_FILE))
                    .map_err(|e| Refusal::Engine(e.to_string()))?;
                pdfcer_core::ocr::engine_ocrcer::OcrcerEngine::from_bytes(&bytes)
                    .map(|e| Self::Ocrcer(Box::new(e)))
                    .map_err(|e| Refusal::Engine(e.to_string()))
            }
            #[allow(unreachable_patterns)]
            _ => Err(Refusal::EngineAbsent),
        }
    }

    /// Recognise one greyscale image: row-major, top-down, `width * height`
    /// bytes.
    pub(super) fn recognise(
        &self,
        width: u32,
        height: u32,
        grey: &[u8],
    ) -> Result<Vec<RecognizedWord>, Refusal> {
        #[cfg(any(feature = "ocrs", feature = "ocrcer"))]
        use pdfcer_core::ocr::OcrEngine as _;
        let _ = (width, height, grey);
        match self {
            #[cfg(feature = "ocrs")]
            Self::Ocrs(e) => e
                .recognize(width, height, grey)
                .map_err(|e| Refusal::Engine(e.to_string())),
            #[cfg(feature = "ocrcer")]
            Self::Ocrcer(e) => e
                .recognize(width, height, grey)
                .map_err(|e| Refusal::Engine(e.to_string())),
            #[allow(unreachable_patterns)]
            _ => Err(Refusal::EngineAbsent),
        }
    }

    /// The loaded engine's own `OcrEngine::reports_confidence`.
    pub(super) fn reports_confidence(&self) -> bool {
        #[cfg(any(feature = "ocrs", feature = "ocrcer"))]
        use pdfcer_core::ocr::OcrEngine as _;
        match self {
            #[cfg(feature = "ocrs")]
            Self::Ocrs(e) => e.reports_confidence(),
            #[cfg(feature = "ocrcer")]
            Self::Ocrcer(e) => e.reports_confidence(),
            #[allow(unreachable_patterns)]
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn the_model_directories_are_distinct() {
        assert_ne!(EngineId::Ocrs.model_dir(), EngineId::Ocrcer.model_dir());
        assert_eq!(EngineId::Ocrs.model_dir(), "ocrs");
    }

    #[test]
    #[cfg(feature = "ocrcer")]
    fn the_ocrcer_names_are_the_engines() {
        use pdfcer_core::ocr::engine_ocrcer::{MODEL_DIR, MODEL_FILE};
        assert_eq!(OCRCER_MODEL_DIR, MODEL_DIR);
        assert_eq!(OCRCER_MODEL_FILE, MODEL_FILE);
    }

    /// A model file that is not an `.ocrw` container is a named engine
    /// refusal, never a panic and never an empty page.
    #[test]
    #[cfg(feature = "ocrcer")]
    fn a_corrupt_ocrcer_model_is_refused_by_the_engine() {
        let dir = std::env::temp_dir().join(format!("pdfcer-ocrcer-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        std::fs::write(dir.join(OCRCER_MODEL_FILE), b"not a model").expect("write");
        let err = Recogniser::load(EngineId::Ocrcer, &dir)
            .err()
            .expect("refused");
        assert!(matches!(err, Refusal::Engine(_)), "{err:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
