//! The command registry — the set of verbs an application can perform.
//!
//! # Why this module exists
//!
//! `SHELL_FRAMEWORK.md` §5 states the rule this module enforces:
//!
//! > **Commands are referenced by string id, never defined [in the
//! > manifest].** The application registers `measure.linear` with its
//! > label, icon, tooltip, enable predicate and handler; the manifest only
//! > says where it appears. That is what stops a customized ribbon from
//! > being able to invent a command that does not exist, and what makes an
//! > unknown id a *disclosed skip* rather than a crash.
//!
//! So there are two halves to a command and they live in different
//! places, for different reasons:
//!
//! | Half | Lives in | Owned by | Changeable by the operator |
//! |---|---|---|---|
//! | **What it is** — id, label, tooltip, icon, when it is enabled, what runs | this registry | the application, in code | no |
//! | **Where it appears** — which tab, which group, which key, whether on the QAT | [`crate::manifest`] | a data file | yes |
//!
//! `SHELL_FRAMEWORK.md` §9 turns that split into the customization
//! contract: an operator may reorder, rename, hide, move between groups,
//! create tabs, and rebind keys. An operator may **not** invent a command,
//! change what a command does, or bypass a command's enable predicate —
//! predicates are safety, not decoration. Every one of those prohibitions
//! is a consequence of this half of the split being code.
//!
//! # The handler is not here
//!
//! A [`Command`] carries a [`HandlerToken`], which is an opaque number the
//! shell never interprets. The application maps it back to whatever it
//! dispatches on — an `Action` enum variant, a function pointer, an index.
//!
//! This is one indirection between a button and its handler, and
//! `SHELL_FRAMEWORK.md` §5 accepts it explicitly as the cost of the
//! design. It buys three things:
//!
//! 1. **The shell stays domain-free.** A registry holding
//!    `Box<dyn FnMut(&mut AppState)>` would need to name `AppState`, and a
//!    shell that names the application's state is not reusable.
//! 2. **The registry stays inspectable.** It is `Debug`, it can be
//!    enumerated, and a test can assert every command in the manifest is
//!    registered — with no application, no window and no state.
//! 3. **Dispatch stays at one choke point.** The application receives a
//!    token and dispatches on it in one place, which is where a
//!    confirmation gate, an undo entry or a trace belongs. A registry of
//!    closures scatters that across as many sites as there are commands.
//!
//! # The enable predicate is data first, and a closure only if it must be
//!
//! [`Enable`] defaults to a **condition name** evaluated against a
//! [`ConditionSet`] the application publishes each frame — `"doc.open"`,
//! `"selection.any"`, `"undo.available"`. The manifest already speaks that
//! language (`visible_when: "selection.any"` in the §4 sketch), and it has
//! properties a closure does not:
//!
//! - it is **serializable**, so a future layer can express a condition;
//! - it is **testable headlessly** — assert the command set with
//!   `selection.any` set and again with it clear, with no application;
//! - it **cannot capture**, so it cannot accidentally hold state that
//!   makes a command's availability depend on when it was registered.
//!
//! [`Enable::Custom`] exists for the case that genuinely needs code, and
//! it is deliberately the awkward option.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// An opaque token the application dispatches on.
///
/// The shell stores it, hands it back when the command is invoked, and
/// never looks inside. A `u64` because every plausible application-side
/// representation — an enum discriminant, a slot index, a hash of a
/// function name — fits in one, and because a type parameter here would
/// propagate to every signature in the shell that mentions a command.
///
/// The value has no meaning to the shell, so two commands may share a
/// token if the application wants two ids to run the same handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HandlerToken(u64);

impl HandlerToken {
    /// Wrap an application-defined value.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// The application-defined value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for HandlerToken {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

/// The set of condition names currently true, published by the
/// application once per frame.
///
/// Ordered rather than hashed so that enumerating it — in a trace, in a
/// failing test message — is deterministic.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConditionSet {
    on: BTreeSet<String>,
}

impl ConditionSet {
    /// An empty set: every [`Enable::When`] command is disabled.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark a condition true.
    pub fn set(&mut self, condition: impl Into<String>) {
        self.on.insert(condition.into());
    }

    /// Builder form of [`Self::set`].
    #[must_use]
    pub fn with(mut self, condition: impl Into<String>) -> Self {
        self.set(condition);
        self
    }

    /// Mark a condition false.
    pub fn clear(&mut self, condition: &str) {
        self.on.remove(condition);
    }

    /// Whether a condition is currently true.
    #[must_use]
    pub fn is_set(&self, condition: &str) -> bool {
        self.on.contains(condition)
    }

    /// Every true condition, in name order.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.on.iter().map(String::as_str)
    }
}

/// When a command is available.
///
/// See the module header on why the data form is the default and the
/// closure form is the escape hatch.
#[derive(Clone)]
pub enum Enable {
    /// Always available. The right answer for commands with no
    /// precondition at all — "Settings", "About", "Open".
    Always,
    /// Available while the named condition is true.
    ///
    /// A leading `!` negates: `"!doc.readonly"` is available while
    /// `doc.readonly` is *not* set. That is the whole expression language,
    /// deliberately. Anything richer belongs in [`Self::Custom`], because
    /// a grammar in a string is a parser and a parser is a thing that has
    /// its own bugs and its own error messages.
    When(String),
    /// Available when this returns `true`.
    ///
    /// The escape hatch. `Arc<dyn Fn>` rather than `Box` so [`Command`]
    /// stays `Clone`, and `Send + Sync` so a registry can be shared across
    /// threads by an application that wants to.
    ///
    /// A predicate here must be **pure and cheap**: it is called for every
    /// rendered control, every frame. It must also be *total* — it cannot
    /// see the application's state except through the [`ConditionSet`] it
    /// is handed, which is deliberate. A predicate that reached into
    /// global state would make a command's availability depend on when it
    /// was asked rather than on what is true.
    Custom(Arc<dyn Fn(&ConditionSet) -> bool + Send + Sync>),
}

impl Enable {
    /// Evaluate against the conditions currently true.
    #[must_use]
    pub fn evaluate(&self, conditions: &ConditionSet) -> bool {
        match self {
            Enable::Always => true,
            Enable::When(name) => match name.strip_prefix('!') {
                Some(negated) => !conditions.is_set(negated),
                None => conditions.is_set(name),
            },
            Enable::Custom(f) => f(conditions),
        }
    }
}

impl std::fmt::Debug for Enable {
    /// `Custom` prints as an opaque marker. The alternative is not
    /// deriving `Debug` at all, and a registry that cannot be printed is a
    /// registry whose failing test says nothing.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Enable::Always => f.write_str("Always"),
            Enable::When(name) => write!(f, "When({name:?})"),
            Enable::Custom(_) => f.write_str("Custom(<fn>)"),
        }
    }
}

/// One verb the application can perform.
///
/// Built with [`Command::new`] and the `with_*` methods: a handful of
/// fields, most of them optional, is exactly the shape that makes a
/// positional constructor unreadable at the call site — and this
/// constructor is called once per command, which for a real application
/// fills a file.
#[derive(Debug, Clone)]
pub struct Command {
    /// The stable id a manifest refers to, e.g. `"view.fit_page"`.
    ///
    /// Never displayed. Dotted lowercase by convention — the shell does
    /// not enforce a shape, because a shell that rejected an
    /// application's naming scheme would be dictating something that is
    /// none of its business.
    pub id: String,
    /// The operator-visible label.
    ///
    /// Supplied by the application, which owns its own string catalogue.
    /// The shell renders it and never invents one.
    pub label: String,
    /// The operator-visible tooltip, if any.
    pub tooltip: Option<String>,
    /// A key naming the icon to draw, resolved by the application.
    ///
    /// A `String` key rather than a texture, an SVG or an
    /// `egui::ImageSource` because icon *rendering* is the application's: an
    /// icon set is a licensing decision, and an application may want to
    /// rasterize vector path data at the physical pixel size in force rather
    /// than ship pre-baked bitmaps. The shell only needs to know that a
    /// control has an icon and which one.
    pub icon: Option<String>,
    /// When this command is available.
    pub enable: Enable,
    /// The opaque token the application dispatches on.
    pub handler: HandlerToken,
}

impl Command {
    /// A command with no tooltip, no icon, and no precondition.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>, handler: HandlerToken) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            tooltip: None,
            icon: None,
            enable: Enable::Always,
            handler,
        }
    }

    /// Attach a tooltip.
    #[must_use]
    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Attach an icon key.
    #[must_use]
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Attach an enable predicate.
    #[must_use]
    pub fn with_enable(mut self, enable: Enable) -> Self {
        self.enable = enable;
        self
    }

    /// Convenience for [`Enable::When`].
    #[must_use]
    pub fn enabled_when(self, condition: impl Into<String>) -> Self {
        self.with_enable(Enable::When(condition.into()))
    }

    /// Whether this command is available under the given conditions.
    #[must_use]
    pub fn is_enabled(&self, conditions: &ConditionSet) -> bool {
        self.enable.evaluate(conditions)
    }
}

/// Every command an application can perform, by id.
///
/// Ordered (`BTreeMap`) so [`Self::ids`] is stable: a failing validation
/// that lists the registered ids must list them the same way twice, or the
/// diff between two runs is noise.
#[derive(Debug, Clone, Default)]
pub struct CommandRegistry {
    by_id: BTreeMap<String, Command>,
}

impl CommandRegistry {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a command.
    ///
    /// # Errors
    ///
    /// [`RegistryError::DuplicateId`] if the id is already registered.
    ///
    /// A duplicate is an error rather than a replacement because the two
    /// registrations disagree about something — a label, a predicate, a
    /// handler — and silently keeping the last one makes the application's
    /// behaviour depend on the order of its own start-up code. That is a
    /// defect that reproduces only after an unrelated reordering, which is
    /// the worst kind to be handed.
    pub fn register(&mut self, command: Command) -> Result<(), RegistryError> {
        if self.by_id.contains_key(&command.id) {
            return Err(RegistryError::DuplicateId { id: command.id });
        }
        self.by_id.insert(command.id.clone(), command);
        Ok(())
    }

    /// Register several commands, stopping at the first duplicate.
    ///
    /// # Errors
    ///
    /// As [`Self::register`].
    pub fn register_all(
        &mut self,
        commands: impl IntoIterator<Item = Command>,
    ) -> Result<(), RegistryError> {
        for command in commands {
            self.register(command)?;
        }
        Ok(())
    }

    /// The command with this id, if it is registered.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Command> {
        self.by_id.get(id)
    }

    /// Every registered id, in order.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.by_id.keys().map(String::as_str)
    }

    /// Every registered command, in id order.
    pub fn iter(&self) -> impl Iterator<Item = &Command> {
        self.by_id.values()
    }

    /// How many commands are registered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Whether nothing is registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}

/// This is the bridge that lets [`crate::manifest`] validate against a
/// registry without depending on this module's concrete type.
///
/// The dependency direction matters: `manifest` must be usable — parsed,
/// merged, round-tripped — by a tool that has no registry at all, which
/// is how `tools/ui-verify` and a schema linter can work on a `.ron` file
/// without linking the application.
impl crate::manifest::CommandCatalog for CommandRegistry {
    fn contains(&self, id: &str) -> bool {
        self.by_id.contains_key(id)
    }
}

/// Why a registration was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RegistryError {
    /// Two commands claim the same id.
    #[error(
        "command id `{id}` is registered twice; the second registration would \
         silently win and make behaviour depend on start-up order"
    )]
    DuplicateId {
        /// The id registered twice.
        id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Group, Item, Shell, Tab};

    fn registry() -> CommandRegistry {
        let mut r = CommandRegistry::new();
        r.register_all([
            Command::new("view.fit_page", "Fit page", HandlerToken::new(1))
                .with_tooltip("Scale the page to fit the window")
                .with_icon("fit-page"),
            Command::new("edit.undo", "Undo", HandlerToken::new(2)).enabled_when("undo.available"),
        ])
        .expect("distinct ids");
        r
    }

    /// Lookup by id, and a registry is inspectable without an
    /// application.
    #[test]
    fn commands_round_trip_by_id() {
        let r = registry();
        let fit = r.get("view.fit_page").expect("registered");
        assert_eq!(fit.label, "Fit page");
        assert_eq!(fit.icon.as_deref(), Some("fit-page"));
        assert_eq!(fit.handler, HandlerToken::new(1));
        assert!(r.get("view.fit_width").is_none());
        assert_eq!(r.ids().collect::<Vec<_>>(), ["edit.undo", "view.fit_page"]);
    }

    /// **A duplicate id is refused, not silently replaced.**
    ///
    /// Two registrations of one id disagree about something. Keeping the
    /// last one makes behaviour depend on the order of start-up code,
    /// which produces a defect that appears after an unrelated
    /// reordering and is close to unattributable.
    #[test]
    fn a_duplicate_id_is_refused_and_named() {
        let mut r = registry();
        let err = r
            .register(Command::new(
                "edit.undo",
                "Undo again",
                HandlerToken::new(9),
            ))
            .expect_err("duplicate must be refused");
        assert_eq!(
            err,
            RegistryError::DuplicateId {
                id: "edit.undo".to_owned()
            }
        );
        assert_eq!(
            r.get("edit.undo").expect("still there").label,
            "Undo",
            "the FIRST registration must survive, so the error is not also a mutation"
        );
    }

    /// Enable predicates evaluate against the published conditions, and
    /// `!` negates.
    #[test]
    fn enable_predicates_evaluate_against_the_condition_set() {
        let undo = registry().get("edit.undo").expect("registered").clone();
        assert!(!undo.is_enabled(&ConditionSet::new()), "empty set disables");
        assert!(undo.is_enabled(&ConditionSet::new().with("undo.available")));

        let not_readonly = Command::new("edit.text", "Edit text", HandlerToken::new(3))
            .enabled_when("!doc.readonly");
        assert!(not_readonly.is_enabled(&ConditionSet::new()));
        assert!(!not_readonly.is_enabled(&ConditionSet::new().with("doc.readonly")));
    }

    /// The closure escape hatch works and cannot see anything but the
    /// condition set.
    #[test]
    fn a_custom_predicate_sees_only_the_condition_set() {
        let cmd =
            Command::new("tools.ocr", "OCR", HandlerToken::new(4)).with_enable(Enable::Custom(
                Arc::new(|c| c.is_set("doc.open") && !c.is_set("doc.scanned")),
            ));
        assert!(!cmd.is_enabled(&ConditionSet::new()));
        assert!(cmd.is_enabled(&ConditionSet::new().with("doc.open")));
        assert!(!cmd.is_enabled(&ConditionSet::new().with("doc.open").with("doc.scanned")));
    }

    /// **★ A manifest referencing an unregistered command id fails
    /// validation, and the failure names that id.**
    ///
    /// This is the invariant the whole registry exists to make
    /// enforceable, and it is what turns `SHELL_FRAMEWORK.md` §9's refusal
    /// of "invent a command" from a policy into a check.
    ///
    /// **Naming the id is the point, not a nicety.** The manifest is a
    /// file an operator edits. "Your shell.ron is invalid" tells them to
    /// go and bisect it; "`view.fit_pge` is not a command" tells them
    /// where the typo is. The same message is what a merge turns into a
    /// disclosed skip — see [`crate::manifest::Skip`] — so the id must be
    /// carried structurally, not embedded in prose.
    #[test]
    fn a_manifest_naming_an_unregistered_command_fails_validation_with_that_id() {
        let shell = Shell::new().with_tab(Tab::new("view", "View").with_groups([
            Group::new("zoom", "Zoom").with_items([
                Item::command("view.fit_page"),
                Item::command("view.fit_pge"),
            ]),
        ]));

        // Structure alone is fine: nothing about the manifest is
        // malformed, which is exactly why this check has to exist
        // separately.
        shell
            .validate()
            .expect("the manifest is structurally valid; only the id is wrong");

        let err = shell
            .validate_against(&registry())
            .expect_err("an unregistered id must fail");
        assert!(
            err.to_string().contains("view.fit_pge"),
            "the error must name the offending id so an operator can find the \
             typo in their own file; got: {err}"
        );
    }

    /// The registry satisfies the manifest's catalog trait, so a manifest
    /// can be validated against it without `manifest` depending on this
    /// module's concrete type.
    #[test]
    fn the_registry_is_a_command_catalog() {
        use crate::manifest::CommandCatalog as _;
        let r = registry();
        assert!(r.contains("edit.undo"));
        assert!(!r.contains("edit.redo"));
    }
}
