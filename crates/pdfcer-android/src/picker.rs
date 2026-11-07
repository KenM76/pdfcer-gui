//! The document picker, answered by `MainActivity.pickDocuments`.
//!
//! The Java method blocks its calling thread until the operator picks or
//! cancels, so this is only ever called from a thread of its own (see
//! `pdfcer_gui::filedialog`), never from the activity's native thread.

use std::path::{Path, PathBuf};

use jni::objects::JString;
use jni::{JValue, jni_sig, jni_str};

/// The activity, reached through JNI.
pub struct ActivityPicker {
    activity: crate::Activity,
    save_dir: Option<PathBuf>,
}

impl ActivityPicker {
    /// Wraps the activity handle; saves are written under `save_dir`.
    pub fn new(activity: crate::Activity, save_dir: Option<PathBuf>) -> Self {
        Self { activity, save_dir }
    }
}

impl pdfcer_gui::filedialog::Picker for ActivityPicker {
    fn open(&self, multiple: bool, mime_types: &[String]) -> Vec<PathBuf> {
        let answer = self
            .activity
            .vm
            .attach_current_thread(|env| -> jni::errors::Result<String> {
                let mimes = env.new_string(mime_types.join("\n"))?;
                let result = env
                    .call_method(
                        &*self.activity.object,
                        jni_str!("pickDocuments"),
                        jni_sig!((multiple: jboolean, mimes: JString) -> JString),
                        &[JValue::Bool(multiple), JValue::Object(&mimes)],
                    )?
                    .l()?;
                let text = env.cast_local::<JString>(result)?;
                Ok(text.to_string())
            });
        match answer {
            Ok(text) => text
                .lines()
                .filter(|l| !l.is_empty())
                .map(PathBuf::from)
                .collect(),
            Err(e) => {
                eprintln!("pdfcer: document picker failed: {e}");
                Vec::new()
            }
        }
    }

    fn save_dir(&self) -> Option<PathBuf> {
        self.save_dir.clone()
    }

    /// `MainActivity.placeDocument`, which blocks until the copy is done.
    fn place(&self, path: &Path, name: &str) -> bool {
        let answer = self
            .activity
            .vm
            .attach_current_thread(|env| -> jni::errors::Result<bool> {
                let path = env.new_string(path.to_string_lossy())?;
                let name = env.new_string(name)?;
                env.call_method(
                    &*self.activity.object,
                    jni_str!("placeDocument"),
                    jni_sig!((path: JString, name: JString) -> jboolean),
                    &[JValue::Object(&path), JValue::Object(&name)],
                )?
                .z()
            });
        answer.unwrap_or_else(|e| {
            eprintln!("pdfcer: placing a saved file failed: {e}");
            false
        })
    }

    /// `MainActivity.printDocument`, which returns once the dialog is up.
    fn print(&self, path: &Path, name: &str) -> bool {
        let answer = self
            .activity
            .vm
            .attach_current_thread(|env| -> jni::errors::Result<bool> {
                let path = env.new_string(path.to_string_lossy())?;
                let name = env.new_string(name)?;
                env.call_method(
                    &*self.activity.object,
                    jni_str!("printDocument"),
                    jni_sig!((path: JString, name: JString) -> jboolean),
                    &[JValue::Object(&path), JValue::Object(&name)],
                )?
                .z()
            });
        answer.unwrap_or_else(|e| {
            eprintln!("pdfcer: opening the print dialog failed: {e}");
            false
        })
    }
}
