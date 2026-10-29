//! *Open with*: a document another app hands to pdfcer.
//!
//! `MainActivity` copies the `VIEW` intent's content URI into the cache on a
//! thread of its own and then calls `nativeViewed` with the local path, which
//! queues it to open like a picked document.

use std::path::PathBuf;

use jni::objects::{JObject, JString};

/// `MainActivity.nativeViewed(String path)`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_kenm76_pdfcer_MainActivity_nativeViewed<'caller>(
    mut env: jni::EnvUnowned<'caller>,
    _this: JObject<'caller>,
    path: JString<'caller>,
) {
    env.with_env(|_env| -> jni::errors::Result<()> {
        pdfcer_gui::filedialog::deliver(vec![PathBuf::from(path.to_string())]);
        Ok(())
    })
    .resolve::<jni::errors::LogErrorAndDefault>();
}
