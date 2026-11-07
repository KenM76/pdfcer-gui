//! The system's open, save and folder pickers: the one type every caller uses.
//!
//! On the desktop this is `rfd::FileDialog`, re-exported unchanged. On Android
//! it is a builder with the same methods, answered by the [`Picker`] the entry
//! point installs (`crates/pdfcer-android`, which calls Android's document
//! picker through the activity).
//!
//! **Android's picker cannot block.** The app's native thread must keep
//! answering activity lifecycle events while the picker is up, or the activity
//! deadlocks on its own pause. So `pick_files` on a PDF filter starts the
//! picker on a thread of its own and returns `None` at once; the chosen files
//! arrive later through [`take_picked`], which the frame turns into opens,
//! exactly like a drop. A document another app opens in pdfcer arrives the
//! same way, through [`deliver`]. A single file asked for a [`Purpose`] goes
//! through [`FileDialog::pick_file_later`] and comes back through
//! [`take_answered`]. A folder pick returns `None` on Android until it has an
//! asynchronous route of its own.
//!
//! **A save is written first and placed afterwards.** `save_file` answers at
//! once with a path in the app's own storage, named as suggested, and the
//! caller writes there as it would anywhere. A thread waits for the file to
//! appear and stop growing, then hands it to Android's create-document picker,
//! which copies it wherever the operator chooses. Cancelling that picker
//! leaves the file only in the app's storage.

#[cfg(not(target_os = "android"))]
pub use rfd::FileDialog;

#[cfg(target_os = "android")]
pub use android::{
    FileDialog, Picker, Purpose, deliver, install, print_later, take_answered, take_picked,
    take_print_failure,
};

#[cfg(target_os = "android")]
mod android {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Mutex, OnceLock};

    /// What the platform supplies: the document picker and the print service.
    pub trait Picker: Send + Sync {
        /// Shows the picker for one or several documents of the given MIME
        /// types and returns readable local paths, empty when cancelled.
        fn open(&self, multiple: bool, mime_types: &[String]) -> Vec<PathBuf>;
        /// The app-private folder a save is written to before it is placed.
        fn save_dir(&self) -> Option<PathBuf>;
        /// Shows the create-document picker suggesting `name` and copies
        /// `path` there; false when cancelled or the copy failed.
        fn place(&self, path: &Path, name: &str) -> bool;
        /// Shows the system print dialog for the PDF at `path`, titled `name`,
        /// and returns once it is up. True hands the file to the dialog, which
        /// deletes it when it closes.
        fn print(&self, path: &Path, name: &str) -> bool;
    }

    static PICKER: OnceLock<Box<dyn Picker>> = OnceLock::new();
    /// Documents the picker returned that the frame has not opened yet.
    static PICKED: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());
    /// Why a print showed no dialog, not yet reported.
    static PRINT_FAILURE: Mutex<Option<String>> = Mutex::new(None);
    /// The context to wake when they arrive; recorded by [`take_picked`].
    static CONTEXT: OnceLock<egui::Context> = OnceLock::new();
    /// Single files picked for a [`Purpose`], not yet acted on.
    static ANSWERED: Mutex<Vec<(Purpose, PathBuf)>> = Mutex::new(Vec::new());

    /// What a single-file pick started by [`FileDialog::pick_file_later`] is for.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Purpose {
        /// Insert ▸ 3D model: the model file to place on the page in view.
        Model,
    }

    /// The single files picked since the last call, with their purpose.
    pub fn take_answered() -> Vec<(Purpose, PathBuf)> {
        std::mem::take(&mut *ANSWERED.lock().unwrap_or_else(|e| e.into_inner()))
    }

    /// The documents picked since the last call. Called once per frame.
    pub fn take_picked(ctx: &egui::Context) -> Vec<PathBuf> {
        let _ = CONTEXT.set(ctx.clone());
        std::mem::take(&mut *PICKED.lock().unwrap_or_else(|e| e.into_inner()))
    }

    /// Queues documents another app handed over (Android's *open with*) to
    /// be opened like picked ones. Callable from any thread, before or after
    /// the first frame.
    pub fn deliver(paths: Vec<PathBuf>) {
        PICKED
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .extend(paths);
        if let Some(ctx) = CONTEXT.get() {
            ctx.request_repaint();
        }
    }

    /// Installs the platform picker; called once, before the app starts.
    /// A second call is ignored.
    pub fn install(picker: Box<dyn Picker>) {
        let _ = PICKER.set(picker);
    }

    /// Writes `pdf` to a private file and shows the system print dialog on a
    /// thread of its own, so the frame never waits. When the dialog cannot be
    /// shown, the reason arrives through [`take_print_failure`].
    pub fn print_later(pdf: Vec<u8>, name: String) {
        static JOBS: AtomicU64 = AtomicU64::new(0);
        std::thread::spawn(move || {
            let Some(picker) = PICKER.get() else { return };
            let Some(dir) = picker.save_dir() else { return };
            let job = JOBS.fetch_add(1, Ordering::Relaxed);
            let path = dir.join(format!("print-{job}.pdf"));
            if let Err(e) = std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(&path, pdf))
            {
                print_failed(e.to_string());
                return;
            }
            if !picker.print(&path, &name) {
                let _ = std::fs::remove_file(&path);
                print_failed(crate::text::print::phone_print_refused().to_owned());
            }
        });
    }

    /// Why the last print showed no dialog, once. Called once per frame.
    pub fn take_print_failure() -> Option<String> {
        PRINT_FAILURE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
    }

    fn print_failed(reason: String) {
        *PRINT_FAILURE.lock().unwrap_or_else(|e| e.into_inner()) = Some(reason);
        if let Some(ctx) = CONTEXT.get() {
            ctx.request_repaint();
        }
    }

    /// The rfd-shaped builder. Titles, folders and file names are ignored:
    /// the Android picker has no title and remembers its own location.
    #[derive(Default)]
    pub struct FileDialog {
        extensions: Vec<String>,
        file_name: Option<String>,
    }

    /// Serialises the placing threads, so two saves never show the
    /// create-document picker at once.
    static PLACING: Mutex<()> = Mutex::new(());

    impl FileDialog {
        pub fn new() -> Self {
            Self::default()
        }
        pub fn set_title(self, _title: impl Into<String>) -> Self {
            self
        }
        pub fn set_directory<P: AsRef<Path>>(self, _path: P) -> Self {
            self
        }
        pub fn set_file_name(mut self, name: impl Into<String>) -> Self {
            self.file_name = Some(name.into());
            self
        }
        pub fn add_filter(
            mut self,
            _name: impl Into<String>,
            extensions: &[impl ToString],
        ) -> Self {
            self.extensions.extend(
                extensions
                    .iter()
                    .map(|e| e.to_string().to_ascii_lowercase()),
            );
            self
        }
        pub fn pick_file(self) -> Option<PathBuf> {
            None
        }
        /// Starts the document picker when the filter names PDF; see the
        /// module header. Always `None` here.
        pub fn pick_files(self) -> Option<Vec<PathBuf>> {
            if self.extensions.iter().any(|e| e == "pdf") && PICKER.get().is_some() {
                let mime = self.mime_types();
                std::thread::spawn(move || {
                    let Some(picker) = PICKER.get() else { return };
                    let paths = picker.open(true, &mime);
                    if paths.is_empty() {
                        return;
                    }
                    deliver(paths);
                });
            }
            None
        }
        /// Starts the picker for one file answering `purpose` and returns at
        /// once; the file arrives through [`take_answered`]. False when no
        /// picker is installed.
        pub fn pick_file_later(self, purpose: Purpose) -> bool {
            if PICKER.get().is_none() {
                return false;
            }
            let mime = self.mime_types();
            std::thread::spawn(move || {
                let Some(picker) = PICKER.get() else { return };
                let Some(path) = picker.open(false, &mime).into_iter().next() else {
                    return;
                };
                ANSWERED
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push((purpose, path));
                if let Some(ctx) = CONTEXT.get() {
                    ctx.request_repaint();
                }
            });
            true
        }
        pub fn pick_folder(self) -> Option<PathBuf> {
            None
        }
        /// A path in the app's storage to write to; see the module header.
        pub fn save_file(self) -> Option<PathBuf> {
            let picker = PICKER.get()?;
            let dir = picker.save_dir()?;
            std::fs::create_dir_all(&dir).ok()?;
            let name = self
                .file_name
                .as_deref()
                .and_then(|n| Path::new(n).file_name())
                .map_or_else(|| "file".to_owned(), |n| n.to_string_lossy().into_owned());
            let path = dir.join(&name);
            // An old file of the same name would be placed before the new one
            // is written.
            if path.exists() && std::fs::remove_file(&path).is_err() {
                return None;
            }
            let waited = path.clone();
            std::thread::spawn(move || {
                if !written(&waited) {
                    return;
                }
                let _one = PLACING.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(picker) = PICKER.get() {
                    picker.place(&waited, &name);
                }
            });
            Some(path)
        }

        /// The filters as MIME types. An extension with no listed type, such
        /// as the "all files" filter, widens the picker to every document.
        fn mime_types(&self) -> Vec<String> {
            let mut out: Vec<String> = Vec::new();
            for ext in &self.extensions {
                let mime = match ext.as_str() {
                    "pdf" => "application/pdf",
                    "png" => "image/png",
                    "jpg" | "jpeg" => "image/jpeg",
                    "txt" => "text/plain",
                    "zip" => "application/zip",
                    "wav" => "audio/wav",
                    _ => return vec!["*/*".to_owned()],
                };
                if !out.iter().any(|m| m == mime) {
                    out.push(mime.to_owned());
                }
            }
            if out.is_empty() {
                out.push("*/*".to_owned());
            }
            out
        }
    }

    /// Waits for the caller to write `path`: true once it exists and its size
    /// has held for a second, false when nothing appears within ten minutes
    /// (the write failed or was abandoned).
    fn written(path: &Path) -> bool {
        const TICK: std::time::Duration = std::time::Duration::from_millis(250);
        let mut last = None;
        let mut steady = 0;
        for _ in 0..2400 {
            std::thread::sleep(TICK);
            let size = std::fs::metadata(path).ok().map(|m| m.len());
            if size.is_some() && size == last {
                steady += 1;
                if steady >= 4 {
                    return true;
                }
            } else {
                steady = 0;
            }
            last = size;
        }
        false
    }
}
