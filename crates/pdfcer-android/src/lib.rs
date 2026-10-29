//! The Android entry point for pdfcer.
//!
//! Android starts the program by loading `libpdfcer_android.so` from the APK
//! and calling [`android_main`] on the GameActivity's native thread. It installs
//! the document picker (`MainActivity.pickDocuments`, reached through JNI) and
//! the system-bar insets (`MainActivity.systemBarInsets`), and exports
//! `nativeViewed` for documents opened from another app,
//! hands the `AndroidApp` to eframe through `NativeOptions::android_app`, and
//! then runs the same application the desktop runs, through
//! `pdfcer_gui::launch`. On any other target this crate is empty.

#[cfg(target_os = "android")]
mod activity;
#[cfg(target_os = "android")]
mod picker;
#[cfg(target_os = "android")]
mod viewed;

#[cfg(target_os = "android")]
use activity::Activity;

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

/// Called by GameActivity once the native library is loaded. It returns when
/// the activity is destroyed.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    diag_from_property();
    let activity = Activity::new(&app);
    pdfcer_gui::insets::install(Box::new(activity.clone()));
    pdfcer_gui::filedialog::install(Box::new(picker::ActivityPicker::new(activity)));
    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };
    if let Err(e) = pdfcer_gui::launch(options, None, None) {
        // Logcat shows a native activity's stderr under `RustStdoutStderr`.
        eprintln!("pdfcer: launch failed: {e}");
    }
}

/// The system property that switches on the `PDFCER_DIAG` trace, which an
/// Android app cannot be given through its environment:
/// `adb shell setprop debug.pdfcer.diag 1`, then relaunch. The trace reaches
/// logcat under `RustStdoutStderr`.
#[cfg(target_os = "android")]
const DIAG_PROPERTY: &str = "debug.pdfcer.diag";

/// Copy [`DIAG_PROPERTY`] into `PDFCER_DIAG` before anything reads it.
#[cfg(target_os = "android")]
fn diag_from_property() {
    let Ok(out) = std::process::Command::new("/system/bin/getprop")
        .arg(DIAG_PROPERTY)
        .output()
    else {
        return;
    };
    let value = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    if value.is_empty() || value == "0" {
        return;
    }
    // SAFETY: called first thing in `android_main`, before eframe, the picker
    // or any thread of ours exists, so nothing reads the environment
    // concurrently. The Java side never reads the process environment.
    unsafe { std::env::set_var("PDFCER_DIAG", value) };
}
