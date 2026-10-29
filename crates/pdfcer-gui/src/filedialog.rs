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
//! same way, through [`deliver`]. Every other pick (single file, folder, save) returns
//! `None` on Android until it has an asynchronous route of its own.

#[cfg(not(target_os = "android"))]
pub use rfd::FileDialog;

#[cfg(target_os = "android")]
pub use android::{FileDialog, Picker, deliver, install, take_picked};

#[cfg(target_os = "android")]
mod android {
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, OnceLock};

    /// What the platform supplies: the document picker.
    pub trait Picker: Send + Sync {
        /// Shows the picker for one or several documents of the given MIME
        /// types and returns readable local paths, empty when cancelled.
        fn open(&self, multiple: bool, mime_types: &[String]) -> Vec<PathBuf>;
    }

    static PICKER: OnceLock<Box<dyn Picker>> = OnceLock::new();
    /// Documents the picker returned that the frame has not opened yet.
    static PICKED: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());
    /// The context to wake when they arrive; recorded by [`take_picked`].
    static CONTEXT: OnceLock<egui::Context> = OnceLock::new();

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

    /// The rfd-shaped builder. Titles, folders and file names are ignored:
    /// the Android picker has no title and remembers its own location.
    #[derive(Default)]
    pub struct FileDialog {
        extensions: Vec<String>,
    }

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
        pub fn set_file_name(self, _name: impl Into<String>) -> Self {
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
        pub fn pick_folder(self) -> Option<PathBuf> {
            None
        }
        pub fn save_file(self) -> Option<PathBuf> {
            None
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
}
