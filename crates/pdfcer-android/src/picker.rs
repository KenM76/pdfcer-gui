//! The document picker, answered by `MainActivity.pickDocuments`.
//!
//! The Java method blocks its calling thread until the operator picks or
//! cancels, so this is only ever called from a thread of its own (see
//! `pdfcer_gui::filedialog`), never from the activity's native thread.

use std::path::PathBuf;

use jni::objects::JString;
use jni::{JValue, jni_sig, jni_str};

/// The activity, reached through JNI.
pub struct ActivityPicker {
    activity: crate::Activity,
}

impl ActivityPicker {
    /// Wraps the activity handle.
    pub fn new(activity: crate::Activity) -> Self {
        Self { activity }
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
}
