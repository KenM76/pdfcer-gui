//! The activity handle the picker and the insets source share.

use std::sync::Arc;

use jni::objects::{Global, JObject, JString};
use jni::{JavaVM, jni_sig, jni_str};

/// The VM and a global reference to `MainActivity`. Cheap to clone.
#[derive(Clone)]
pub struct Activity {
    pub vm: JavaVM,
    pub object: Arc<Global<JObject<'static>>>,
}

impl Activity {
    /// Captures the VM and activity from the `AndroidApp` handle.
    pub fn new(app: &winit::platform::android::activity::AndroidApp) -> Self {
        // SAFETY: android-activity guarantees both pointers stay valid for the
        // life of the process (the VM) and of the activity (the activity
        // object, which the global reference below then keeps alive).
        let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) };
        let object = vm
            .attach_current_thread(|env| -> jni::errors::Result<_> {
                let local = unsafe { JObject::from_raw(env, app.activity_as_ptr().cast()) };
                env.new_global_ref(local)
            })
            .expect("a global reference to the activity");
        Self {
            vm,
            object: Arc::new(object),
        }
    }
}

impl pdfcer_gui::insets::Insets for Activity {
    /// Asks `MainActivity.systemBarInsets`; zero insets when the call fails,
    /// which leaves the layout edge-to-edge rather than breaking the frame.
    fn system_bars_px(&self) -> [f32; 4] {
        let text = self
            .vm
            .attach_current_thread(|env| -> jni::errors::Result<String> {
                let result = env
                    .call_method(
                        &*self.object,
                        jni_str!("systemBarInsets"),
                        jni_sig!(() -> JString),
                        &[],
                    )?
                    .l()?;
                Ok(env.cast_local::<JString>(result)?.to_string())
            })
            .unwrap_or_default();
        let mut px = [0.0; 4];
        for (slot, value) in px.iter_mut().zip(text.split_whitespace()) {
            *slot = value.parse().unwrap_or(0.0);
        }
        px
    }
}
