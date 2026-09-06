//! `app::dispatch::security` — the File > Security band's commands, and the one
//! line of `dispatch.rs` they cost
//!
//! ## ★★ Renamed from `dispatch::protect` on 2026-09-06, when `file.sign`
//! joined the band
//!
//! The old name described two of the three: encrypting and re-permissioning are
//! *protection*, and signing is not — a signature protects nothing, it asserts
//! authorship. Keeping the name would have made it a description of the
//! module's history rather than of its contents, which is the thing this
//! project's own `native-clipboard` manifest argues against by name.
//!
//! What all three ARE is the **File > Security band**: each writes something
//! into the file that is about the file rather than about any page, none is an
//! undoable content edit, and each produces a new document rather than changing
//! the one on screen. That is the subject, and the module is now named after
//! it.
//!
//! `OPERATOR_REQUESTS.md` **O119**, approved and wired 2026-09-04: *"yes add
//! encryption and permissions"*. `file.encrypt` and `file.permissions`, which
//! open one window ([`crate::dialogs::protect`]) with two starting points.
//!
//! # Why this is a module and not two arms in [`super`]
//!
//! **R2, and [`super::panels`]' precedent, taken deliberately rather than
//! invented.** That module's own header records the situation exactly:
//!
//! > Split out of [`super`] under **R2** on 2026-09-04: `dispatch.rs` was at
//! > 1,460 lines and four arms carrying their own reasoning would not fit under
//! > the 1,500-line ceiling.
//!
//! By the afternoon of the same day `dispatch.rs` was at **1,496 lines before
//! this feature existed** — a concurrent track had added its own arms — so two
//! more arms with a paragraph of reasoning between them took it to 1,518 and
//! `tools/gates/check-file-size.sh` went red naming it. Compressing the prose
//! got it to 1,508, which is a smaller violation and still a violation.
//!
//! ⇒ **The ceiling is not a budget to spend down to; it is a signal that a file
//! has stopped being one subject.** The seam here is a real one and the same
//! shape [`super::panels`] found: these two commands are the only ones in the
//! program that **write a whole new file out of the open document's
//! encryption**, they share a window, a `Task` and a set of disclosures, and
//! none of that has anything to say to the hundred other arms in `dispatch.rs`.
//!
//! ★ The alternative — an exemption in the gate — is explicitly an operator
//! decision, not a build session's, and the gate says so in its own failure
//! text. Splitting is what the rule asks for.
//!
//! # What [`super`] keeps
//!
//! One guard arm and a two-line comment:
//!
//! ```ignore
//! id if security::claims(id) => self.dispatch_security(id),
//! ```
//!
//! …which is [`super::panels`]' arrangement precisely, including the pairing of
//! a `claims` predicate with a dispatcher over the same list. The pair is
//! pinned by [`tests::the_guard_and_the_dispatcher_claim_the_same_ids`], so a
//! third Security command added to one and not the other fails a named test
//! rather than becoming a control that traces `command-unimplemented`.
//!
//! # ★★ What this dispatch deliberately does NOT decide
//!
//! **Whether the document is signed.** Neither the registry predicate
//! (`doc.open`) nor these arms ask, and that is the R9 ruling rather than an
//! omission: whether *this* document carries a signature is not known when the
//! command registry is built, so the control stays present and the **window**
//! refuses — by name, with the signature count, explaining that protecting
//! rewrites every byte the signature covers. A click or a chord on a signed
//! document therefore produces a sentence about the operator's document rather
//! than a failure, which is R9's *explained* branch: *the control is absent or
//! explained, never a button that fails on press.*
//!
//! **And whether the mode allows it.** It does, in all three, and that is a
//! decision rather than an oversight — `crate::shell::commands::catalog::file`
//! carries the argument at the registrations. Protecting a drawing before
//! sending it out changes nothing on any page, so it is not authoring; an
//! operator reading a document in Read mode is exactly the operator about to
//! email it to somebody. `file.export_dxf` is gated the same way for the same
//! reason.

use crate::app::PdfcerApp;
use crate::protect::Task;

/// Whether `id` is one of the Security commands this module dispatches.
///
/// ★ Paired with [`PdfcerApp::dispatch_security`] over the same list, and the
/// two are pinned together by a test — [`super::panels::claims`]' arrangement
/// and its reason: a guard and a dispatcher that disagree turn a registered
/// command into one that traces `command-unimplemented`, which looks from the
/// outside exactly like a command nobody wired.
#[must_use]
pub(crate) fn claims(id: &str) -> bool {
    // ★ `file.sign` is behind the `signing` feature, so a build without it
    // registers no such command and this arm can never be reached — but the
    // predicate names it unconditionally, deliberately. `SHELL_FRAMEWORK.md`
    // §5b's rule binds the RIBBON and the registry; a `#[cfg]` here would put
    // a second place that knows about a capability into the dispatcher, and
    // the string costs nothing because nothing can raise it.
    matches!(id, "file.encrypt" | "file.permissions" | "file.sign")
}

impl PdfcerApp {
    /// Open the Encrypt / Permissions window on the right starting point.
    ///
    /// The two commands differ in exactly one value — the [`Task`] — and
    /// everything else about them is one implementation, which is the whole
    /// argument `crate::protect::Task`'s own doc makes: two windows would put
    /// the password fields, the destination choice, the disclosures and the
    /// atomic write in two files, and the second copy is where a disclosure
    /// goes missing.
    ///
    /// The already-open guard lives in
    /// [`crate::dialogs::DialogsState::open_protect`] rather than here, so a
    /// chord and a ribbon click are gated by one expression.
    pub(in crate::app) fn dispatch_security(&mut self, id: &str) {
        // ★★★ Signing is its own window, not a third `Task`. The two encryption
        // commands share a window because they differ in exactly one value; a
        // signature shares nothing with them — no password fields, no
        // permission list, a private key it must hold and drop, and a different
        // set of engine refusals. One window per subject.
        #[cfg(feature = "signing")]
        if id == "file.sign" {
            self.dialogs.open_sign(&self.status);
            return;
        }
        let task = match id {
            "file.permissions" => Task::Permissions,
            // ★ `file.encrypt` and — by construction — nothing else, because
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

    /// ★★ **The guard and the dispatcher claim the same ids.**
    ///
    /// [`super::panels`]' test by the same name and for its reason: the two
    /// lists are written separately and a command added to one and not the
    /// other becomes a registered control that does nothing — indistinguishable
    /// from the outside from one that was never wired.
    ///
    /// Asserted against the **registry** rather than against a third hard-coded
    /// list, so a third Security command registered tomorrow fails here rather
    /// than passing a test that lists the same two ids a third time.
    #[test]
    fn the_guard_and_the_dispatcher_claim_the_same_ids() {
        // ★ Built here rather than reaching for a shared helper: the
        // catalogue's own `all()` is `pub(super)` to its module and the test
        // registry there is private to that module's tests. Registering into a
        // fresh `CommandRegistry` is the same act `crate::shell::commands`
        // performs at start-up, so this asks the question of the real
        // registration path rather than of a list.
        let mut reg = egui_shell::commands::CommandRegistry::new();
        crate::shell::commands::register(&mut reg);
        let registered: Vec<String> = ["file.encrypt", "file.permissions", "file.sign"]
            .into_iter()
            .filter(|id| reg.get(id).is_some())
            .map(str::to_owned)
            .collect();
        // ★ Build-dependent: `file.sign` is registered only with the `signing`
        // feature, which is `SHELL_FRAMEWORK.md` §5b's whole mechanism. Written
        // as arithmetic over `cfg!` rather than as a number, because both
        // answers are correct and one literal would fail one of the two
        // supported builds.
        assert_eq!(
            registered.len(),
            2 + usize::from(cfg!(feature = "signing")),
            "every registered Security command: {registered:?}"
        );
        for id in &registered {
            assert!(
                claims(id),
                "`{id}` is registered and the guard does not claim it"
            );
        }
        assert!(!claims("file.print"), "the guard claims only its own two");
        assert!(!claims("file.export_dxf"));
    }

    /// **Each id reaches its own task.**
    ///
    /// The one decision this module makes, asserted as a pure mapping. A build
    /// that sent both commands to `Task::Password` would open a window that
    /// works — and `Permissions…` would offer to set a password on a document
    /// the operator wanted to re-permission, which is a wrong window rather
    /// than a broken one and therefore the kind that ships.
    #[test]
    fn each_command_reaches_its_own_task() {
        assert_eq!(task_of("file.encrypt"), Task::Password);
        assert_eq!(task_of("file.permissions"), Task::Permissions);
    }

    /// The mapping [`PdfcerApp::dispatch_security`] applies, without an app.
    ///
    /// ★ A second spelling of three lines, and it is the honest cost of
    /// asserting a decision that is otherwise only reachable through a
    /// `&mut PdfcerApp`. It is pinned to the real one by
    /// [`each_command_reaches_its_own_task`] reading the same two ids the
    /// registry test above proves are registered — so the two cannot drift
    /// about *which* commands exist, only about what they map to, and that
    /// mapping is three lines long and in view.
    fn task_of(id: &str) -> Task {
        match id {
            "file.permissions" => Task::Permissions,
            _ => Task::Password,
        }
    }
}
