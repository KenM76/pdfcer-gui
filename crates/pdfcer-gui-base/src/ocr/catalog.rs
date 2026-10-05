//! The OCR models Recognise text can offer: every model the engine's add-on
//! discovery finds under the bundled `models` folder and the operator's extra
//! folders, each paired with whether this build can run it and, if not, why.
//!
//! Discovery is `pdfcer_core::ocr::addons::discover_ocr_models`; runnability is
//! `pdfcer_ocr_host::check_runnable`, whose refusal is shown in its own words.
//! This module only adds the starting choice. A remembered model that is gone
//! is reported, never silently replaced.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/ocr/catalog.md`.

use std::path::{Path, PathBuf};

use pdfcer_core::ocr::addons::{DiscoveryNote, OcrModel, discover_ocr_models};
pub use pdfcer_ocr_host::ProgramPolicy;
use pdfcer_ocr_host::{ProgramError, ProgramRefusal, RunnerError};

use super::EngineId;

/// The engine token of a PaddleOCR-VL add-on.
// ui-text-exempt: an engine token matched literally, never displayed.
pub const PADDLE_VL: &str = "paddle-vl";

/// The folder under the executable's directory that holds the bundled models.
// ui-text-exempt: a directory name, never displayed.
pub const BUNDLED_DIR: &str = "models";

/// Why a discovered model cannot run here: the engine's refusal, verbatim,
/// and a stable token for the trace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unrunnable {
    token: &'static str,
    reason: String,
}

impl Unrunnable {
    /// The engine's refusal.
    #[must_use]
    pub fn from_error(error: &RunnerError) -> Self {
        Self {
            token: token_of(error),
            reason: error.to_string(),
        }
    }

    /// The stable trace token.
    #[must_use]
    pub const fn token(&self) -> &'static str {
        self.token
    }

    /// The engine's sentence.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

// ui-text-exempt: trace tokens, never displayed.
const fn token_of(error: &RunnerError) -> &'static str {
    match error {
        RunnerError::EngineNotInBuild { .. } => "not-in-build",
        RunnerError::MissingFile { .. } => "missing-files",
        RunnerError::NeedsProgramKind { .. } => "needs-program-kind",
        RunnerError::Verify { .. } => "verify",
        RunnerError::Program(ProgramError::Refused(refusal)) => match refusal {
            ProgramRefusal::RefusedByPolicy { .. } => "refused-by-policy",
            ProgramRefusal::NoProgramHash { .. } => "no-program-hash",
            ProgramRefusal::NoProtocol { .. } => "no-protocol",
            ProgramRefusal::ProgramMissing { .. } => "program-missing",
            ProgramRefusal::NotAProgram { .. } => "not-a-program",
            _ => "program-refused",
        },
        RunnerError::Program(_) => "program",
        RunnerError::Engine(_) => "engine",
        _ => "other",
    }
}

/// The program policy the `ocr_program_addons` preference names.
#[must_use]
pub const fn policy(allow_programs: bool) -> ProgramPolicy {
    if allow_programs {
        ProgramPolicy::Allow
    } else {
        ProgramPolicy::Refuse
    }
}

/// One model the dialog lists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    /// The discovery's unique name; the `ocr_model` preference value.
    pub name: String,
    /// The manifest's label, if it has one.
    pub label: Option<String>,
    /// The engine token, as the manifest or folder states it.
    pub engine_token: String,
    /// The engine this shell drives it with, when the token names one.
    pub engine: Option<EngineId>,
    /// The model folder.
    pub folder: PathBuf,
    /// `None` when this build can run it.
    pub unrunnable: Option<Unrunnable>,
    /// What discovery found; what a run loads.
    pub model: OcrModel,
}

impl Choice {
    /// Whether this build can run the model.
    #[must_use]
    pub const fn runnable(&self) -> bool {
        self.unrunnable.is_none()
    }
}

/// Everything discovery found, in priority order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Catalog {
    /// The folders searched, in order.
    pub roots: Vec<PathBuf>,
    /// Every model found, earlier roots first.
    pub choices: Vec<Choice>,
    /// The engine's own sentences for what it skipped or shadowed.
    pub notes: Vec<String>,
}

/// What a new dialog starts on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Start {
    /// The choice at this index.
    Chosen(usize),
    /// The remembered model, which is not found or cannot run; nothing chosen.
    Remembered {
        /// The `ocr_model` preference value.
        name: String,
        /// Why it cannot run; `None` when it was not found at all.
        why: Option<Unrunnable>,
    },
    /// No model this build can run.
    Nothing,
}

/// The folders searched: the bundled one first, then `extra` in order.
#[must_use]
pub fn roots(exe_dir: Option<&Path>, extra: &[PathBuf]) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = exe_dir.map(|d| d.join(BUNDLED_DIR)).into_iter().collect();
    for folder in extra {
        if !out.contains(folder) {
            out.push(folder.clone());
        }
    }
    out
}

/// Discover every model under `roots`, judged under `policy`. A bundled
/// folder that does not exist is not reported: a build with no bundled models
/// is a valid build.
#[must_use]
pub fn discover(roots: Vec<PathBuf>, bundled: Option<&Path>, policy: ProgramPolicy) -> Catalog {
    let found = discover_ocr_models(&roots);
    let notes = found
        .notes
        .iter()
        .filter(|n| !matches!(n, DiscoveryNote::RootMissing(p) if Some(p.as_path()) == bundled))
        .map(ToString::to_string)
        .collect();
    let choices = found.models.iter().map(|m| choice(m, policy)).collect();
    Catalog {
        roots,
        choices,
        notes,
    }
}

fn choice(model: &OcrModel, policy: ProgramPolicy) -> Choice {
    Choice {
        name: model.name.clone(),
        label: model.label().map(str::to_owned),
        engine_token: model.engine.clone(),
        engine: EngineId::from_key(&model.engine),
        folder: model.folder.clone(),
        unrunnable: pdfcer_ocr_host::check_runnable(model, policy)
            .err()
            .map(|e| Unrunnable::from_error(&e)),
        model: model.clone(),
    }
}

/// The starting choice: the remembered model when it can run, else nothing
/// with the reason; with no remembered model, the first runnable model of the
/// remembered engine, else of the first engine in [`EngineId::ALL`] order,
/// else the first runnable model of any engine.
#[must_use]
pub fn start(catalog: &Catalog, model: Option<&str>, engine: Option<EngineId>) -> Start {
    if let Some(name) = model {
        return match catalog.choices.iter().position(|c| c.name == name) {
            Some(i) if catalog.choices[i].runnable() => Start::Chosen(i),
            found => Start::Remembered {
                name: name.to_owned(),
                why: found.and_then(|i| catalog.choices[i].unrunnable.clone()),
            },
        };
    }
    let first_of = |e: EngineId| {
        catalog
            .choices
            .iter()
            .position(|c| c.engine == Some(e) && c.runnable())
    };
    engine
        .and_then(first_of)
        .or_else(|| EngineId::ALL.into_iter().find_map(first_of))
        .or_else(|| catalog.choices.iter().position(Choice::runnable))
        .map_or(Start::Nothing, Start::Chosen)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("pdfcer-ocr-catalog-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    fn add_on(root: &Path, folder: &str, manifest: &str) {
        let dir = root.join(folder);
        std::fs::create_dir_all(&dir).expect("add-on dir");
        std::fs::write(dir.join(pdfcer_core::ocr::addons::MANIFEST_FILE), manifest)
            .expect("manifest");
    }

    #[test]
    fn the_bundled_folder_is_first_and_an_extra_is_listed_once() {
        let extra = vec![
            PathBuf::from("E:/a"),
            PathBuf::from("E:/a"),
            PathBuf::from("E:/b"),
        ];
        let got = roots(Some(Path::new("C:/app")), &extra);
        assert_eq!(
            got,
            vec![
                Path::new("C:/app").join(BUNDLED_DIR),
                PathBuf::from("E:/a"),
                PathBuf::from("E:/b")
            ]
        );
    }

    fn token(cat: &Catalog, name: &str) -> Option<&'static str> {
        let c = cat.choices.iter().find(|c| c.name == name).expect(name);
        c.unrunnable.as_ref().map(Unrunnable::token)
    }

    /// A VL add-on without its weights is refused by the engine's own check.
    #[test]
    fn a_vl_add_on_without_its_files_lists_with_the_engines_reason() {
        let root = scratch("vl");
        add_on(
            &root,
            "vl",
            "name = vl-test\nengine = paddle-vl\nlabel = VL test\n",
        );
        let cat = discover(vec![root.clone()], None, ProgramPolicy::Allow);
        let c = &cat.choices[0];
        assert_eq!(c.label.as_deref(), Some("VL test"));
        let want = if cfg!(feature = "ocr-vl") {
            "missing-files"
        } else {
            "not-in-build"
        };
        assert_eq!(token(&cat, "vl-test"), Some(want));
        let why = c.unrunnable.as_ref().expect("refused");
        assert!(why.reason().contains("vl-test"), "{why:?}");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn program_and_unknown_models_carry_the_engines_refusal() {
        let root = scratch("odd");
        add_on(&root, "x", "name = x\nengine = tesseract\n");
        add_on(&root, "u", "name = u\nengine = nonesuch\n");
        add_on(
            &root,
            "p",
            "name = p\nengine = ocrs\nkind = program\nprogram = run.exe\n",
        );
        let cat = discover(vec![root.clone()], None, ProgramPolicy::Allow);
        assert_eq!(token(&cat, "x"), Some("needs-program-kind"));
        assert_eq!(token(&cat, "u"), Some("not-in-build"));
        assert_eq!(token(&cat, "p"), Some("no-protocol"));
        let cat = discover(vec![root.clone()], None, ProgramPolicy::Refuse);
        assert_eq!(token(&cat, "p"), Some("refused-by-policy"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    #[cfg(feature = "ocrcer")]
    fn a_model_without_its_files_names_them() {
        let root = scratch("files");
        add_on(&root, "o", "name = o\nengine = ocrcer\n");
        let cat = discover(vec![root.clone()], None, ProgramPolicy::Allow);
        assert_eq!(token(&cat, "o"), Some("missing-files"));
        let why = cat.choices[0].unrunnable.as_ref().expect("refused");
        assert!(
            why.reason().contains(super::super::OCRCER_MODEL_FILE),
            "{why:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_missing_bundled_folder_is_quiet_and_a_missing_extra_is_not() {
        let gone = std::env::temp_dir().join(format!(
            "pdfcer-ocr-catalog-never-there-{}",
            std::process::id()
        ));
        let cat = discover(vec![gone.clone()], Some(&gone), ProgramPolicy::Allow);
        assert!(cat.notes.is_empty(), "{:?}", cat.notes);
        let cat = discover(vec![gone], None, ProgramPolicy::Allow);
        assert_eq!(cat.notes.len(), 1);
    }

    fn refused() -> Unrunnable {
        Unrunnable {
            token: "not-in-build",
            reason: String::new(),
        }
    }

    fn listed(name: &str, engine: Option<EngineId>, runnable: bool) -> Choice {
        let token = engine.map_or(PADDLE_VL, EngineId::key).to_owned();
        Choice {
            name: name.to_owned(),
            label: None,
            engine_token: token.clone(),
            engine,
            folder: PathBuf::from(name),
            unrunnable: (!runnable).then(refused),
            model: OcrModel {
                name: name.to_owned(),
                engine: token,
                folder: PathBuf::from(name),
                root: PathBuf::new(),
                manifest: None,
            },
        }
    }

    #[test]
    fn a_remembered_model_that_is_gone_is_reported_not_replaced() {
        let cat = Catalog {
            choices: vec![listed("ocrs", Some(EngineId::Ocrs), true)],
            ..Catalog::default()
        };
        assert_eq!(
            start(&cat, Some("gone"), Some(EngineId::Ocrs)),
            Start::Remembered {
                name: "gone".into(),
                why: None
            }
        );
        assert_eq!(start(&cat, Some("ocrs"), None), Start::Chosen(0));
    }

    #[test]
    fn a_remembered_model_that_cannot_run_says_why() {
        let cat = Catalog {
            choices: vec![
                listed("ocrs", Some(EngineId::Ocrs), true),
                listed("vl", None, false),
            ],
            ..Catalog::default()
        };
        assert_eq!(
            start(&cat, Some("vl"), None),
            Start::Remembered {
                name: "vl".into(),
                why: Some(refused())
            }
        );
    }

    #[test]
    fn with_no_remembered_model_the_engine_order_decides() {
        let cat = Catalog {
            choices: vec![
                listed("vl", None, false),
                listed("ocrcer", Some(EngineId::Ocrcer), true),
                listed("ocrs", Some(EngineId::Ocrs), true),
            ],
            ..Catalog::default()
        };
        assert_eq!(start(&cat, None, None), Start::Chosen(2));
        assert_eq!(start(&cat, None, Some(EngineId::Ocrcer)), Start::Chosen(1));
        assert_eq!(start(&Catalog::default(), None, None), Start::Nothing);
        let only_vl = Catalog {
            choices: vec![listed("vl", None, true)],
            ..Catalog::default()
        };
        assert_eq!(start(&only_vl, None, None), Start::Chosen(0));
    }
}
