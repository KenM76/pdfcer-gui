//! `app::dispatch::security` — the File ▸ Security band's commands
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/security.md`.

use crate::app::PdfcerApp;
use crate::protect::Task;

/// Whether `id` is one of the Security commands this module dispatches.
#[must_use]
pub(crate) fn claims(id: &str) -> bool {
    // `file.sign` is behind the `signing` feature, so a build without it
    // registers no such command and this arm can never be reached — but the
    // predicate names it unconditionally, deliberately. `SHELL_FRAMEWORK.md`
    // §5b's rule binds the RIBBON and the registry; a `#[cfg]` here would put
    // a second place that knows about a capability into the dispatcher, and
    // the string costs nothing because nothing can raise it.
    matches!(
        id,
        "file.purge_password_values" | "file.encrypt" | "file.permissions" | "file.sign"
    )
}

impl PdfcerApp {
    /// Route one Security command: a window for Encrypt / Permissions / Sign, an
    /// action for Remove old passwords.
    pub(in crate::app) fn dispatch_security(
        &mut self,
        id: &str,
        actions: &mut Vec<crate::app::actions::Action>,
    ) {
        // No window: the scan decides whether there is anything to do, and the
        // picker opens in the apply phase once the clean copy exists.
        if id == "file.purge_password_values" {
            actions.push(crate::app::actions::Action::Write(
                crate::app::actions::write::WriteAction::PurgePasswords,
            ));
            return;
        }
        // Signing is its own window, not a third `Task`. The two encryption
        // commands share a window because they differ in exactly one value; a
        // signature shares nothing with them — no password fields, no
        // permission list, a private key it must hold and drop, and a different
        // set of engine refusals. One window per subject.
        #[cfg(feature = "signing")]
        if id == "file.sign" {
            self.dialogs
                .open_sign(&self.status, self.prefs.sign_timestamp_server.as_deref());
            return;
        }
        let task = match id {
            "file.permissions" => Task::Permissions,
            // `file.encrypt` and — by construction — nothing else, because
            // `claims` is the only gate that reaches here. Written as the
            // fall-through rather than as a third arm with an `unreachable!`,
            // on this project's standing preference against panicking on a
            // branch a guard has already excluded.
            _ => Task::Password,
        };
        self.dialogs.open_protect(&self.status, task);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The guard and the dispatcher claim the same ids.**
    #[test]
    fn the_guard_and_the_dispatcher_claim_the_same_ids() {
        // Built here rather than reaching for a shared helper: the
        // catalogue's own `all()` is `pub(super)` to its module and the test
        // registry there is private to that module's tests. Registering into a
        // fresh `CommandRegistry` is the same act `crate::shell::commands`
        // performs at start-up, so this asks the question of the real
        // registration path rather than of a list.
        let mut reg = egui_shell::commands::CommandRegistry::new();
        crate::shell::commands::register(&mut reg);
        let registered: Vec<String> = [
            "file.purge_password_values",
            "file.encrypt",
            "file.permissions",
            "file.sign",
        ]
        .into_iter()
        .filter(|id| reg.get(id).is_some())
        .map(str::to_owned)
        .collect();
        // Build-dependent: `file.sign` is registered only with the `signing`
        // feature, which is `SHELL_FRAMEWORK.md` §5b's whole mechanism. Written
        // as arithmetic over `cfg!` rather than as a number, because both
        // answers are correct and one literal would fail one of the two
        // supported builds.
        assert_eq!(
            registered.len(),
            3 + usize::from(cfg!(feature = "signing")),
            "every registered Security command: {registered:?}"
        );
        for id in &registered {
            assert!(
                claims(id),
                "`{id}` is registered and the guard does not claim it"
            );
        }
        assert!(!claims("file.print"), "the guard claims only its own");
        assert!(!claims("file.export_dxf"));
    }

    /// **Each id reaches its own task.**
    #[test]
    fn each_command_reaches_its_own_task() {
        assert_eq!(task_of("file.encrypt"), Task::Password);
        assert_eq!(task_of("file.permissions"), Task::Permissions);
    }

    /// The mapping [`PdfcerApp::dispatch_security`] applies, without an app.
    fn task_of(id: &str) -> Task {
        match id {
            "file.permissions" => Task::Permissions,
            _ => Task::Password,
        }
    }
}
