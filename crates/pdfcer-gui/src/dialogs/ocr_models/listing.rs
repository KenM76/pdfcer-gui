//! What the OCR models window lists: every discovered model with its size and
//! whether it is the default, and every supported recogniser with nothing
//! installed.

use std::path::PathBuf;

use crate::app::prefs::Prefs;
use crate::ocr::catalog::{self, Catalog, Choice, Start};
use crate::ocr::manage;

/// One discovery, with what the window shows beside it.
#[derive(Debug)]
pub(super) struct Listing {
    pub(super) catalog: Catalog,
    /// Bytes on disk, by `catalog.choices` index.
    pub(super) sizes: Vec<u64>,
    /// The index Recognise text starts on.
    pub(super) default: Option<usize>,
    /// `<exe dir>/models`: where installs go.
    pub(super) models_dir: Option<PathBuf>,
    /// Whether `models_dir` can be changed from this account.
    pub(super) writable: bool,
    /// `manage::generation` when it was made.
    pub(super) generation: u64,
}

impl Listing {
    /// Discover under the bundled folder and `prefs`' extra folders.
    pub(super) fn load(prefs: &Prefs) -> Self {
        let generation = manage::generation();
        let exe = crate::ocr::exe_dir();
        let roots = catalog::roots(exe.as_deref(), prefs.ocr_models.folders());
        let models_dir = exe.map(|d| d.join(catalog::BUNDLED_DIR));
        let policy = catalog::policy(!prefs.ocr_models.refuse_programs);
        let catalog = catalog::discover(roots, models_dir.as_deref(), policy);
        let sizes = catalog
            .choices
            .iter()
            .map(|c| manage::folder_bytes(&c.folder))
            .collect();
        let default = match catalog::start(
            &catalog,
            prefs.ocr_models.model.as_deref(),
            prefs.ocr_engine,
        ) {
            Start::Chosen(i) => Some(i),
            _ => None,
        };
        let writable = models_dir.as_deref().is_some_and(manage::writable);
        let listing = Self {
            catalog,
            sizes,
            default,
            models_dir,
            writable,
            generation,
        };
        listing.trace();
        listing
    }

    fn trace(&self) {
        let names: Vec<&str> = self
            .catalog
            .choices
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        let missing: Vec<&str> = self.missing().collect();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "ocr-models-listed installed={} names={} default={} bytes={} writable={} missing={}",
                names.len(),
                if names.is_empty() {
                    "none".to_owned()
                } else {
                    names.join(",")
                },
                self.default_name().unwrap_or("none"),
                self.total(),
                self.writable,
                if missing.is_empty() {
                    "none".to_owned()
                } else {
                    missing.join(",")
                },
            )
        });
    }

    /// The default model's name.
    pub(super) fn default_name(&self) -> Option<&str> {
        self.default.map(|i| self.catalog.choices[i].name.as_str())
    }

    /// Every installed model's bytes.
    pub(super) fn total(&self) -> u64 {
        self.sizes.iter().sum()
    }

    /// The models Remove all deletes: every one not a folder the operator
    /// added as a search root.
    pub(super) fn removable(&self) -> impl Iterator<Item = (usize, &Choice)> {
        self.catalog
            .choices
            .iter()
            .enumerate()
            .filter(|(_, c)| c.model.folder != c.model.root)
    }

    /// Supported recognisers with no model installed, in listing order.
    pub(super) fn missing(&self) -> impl Iterator<Item = &'static str> + '_ {
        manage::SUPPORTED
            .into_iter()
            .filter(|t| !self.catalog.choices.iter().any(|c| c.engine_token == *t))
    }

    /// Installed models of `token`, by index.
    pub(super) fn of_engine<'a>(
        &'a self,
        token: &'a str,
    ) -> impl Iterator<Item = (usize, &'a Choice)> {
        self.catalog
            .choices
            .iter()
            .enumerate()
            .filter(move |(_, c)| c.engine_token == token)
    }

    /// Installed models of no supported engine.
    pub(super) fn others(&self) -> impl Iterator<Item = (usize, &Choice)> {
        self.catalog
            .choices
            .iter()
            .enumerate()
            .filter(|(_, c)| !manage::SUPPORTED.contains(&c.engine_token.as_str()))
    }
}
