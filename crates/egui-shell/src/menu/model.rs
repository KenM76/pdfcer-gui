//! The menu **document** — a context menu in the manifest vocabulary.
//!
//! # A menu is a ribbon band with a different key
//!
//! `RIBBON_IA.md` §6 describes the context menu as *"the other half of
//! making selection mean something"*, and §5 as **a third surface carrying the
//! same commands again** for the user who right-clicks. "The same commands
//! again" is the design constraint that decides this file's entire shape:
//! if a context menu were a second vocabulary, every command an application
//! ships would have to be described twice, and the second description would
//! be the one that goes stale.
//!
//! So a [`Menu`] is deliberately *isomorphic* to
//! [`crate::manifest::Group`]:
//!
//! | | `Group` | `Menu` |
//! |---|---|---|
//! | Identified by | `id`, unique within its tab | [`Menu::context`], unique within the document |
//! | Holds | `Option<Vec<Item>>` | `Option<Vec<Item>>` — **the same [`Item`]** |
//! | Caption | required (a band without one is a mystery) | none (a context menu's caption is the thing you right-clicked) |
//!
//! The `Item` is not a look-alike. It is
//! [`crate::manifest::Item`] itself — `Command(String)`, `Separator`,
//! `Custom { kind, payload }` — so a command id that moves, a separator
//! that is inserted, or a custom control that an application draws works
//! identically on a ribbon band and in a context menu, and the merge,
//! validation and customization machinery that already understands `Item`
//! understands a menu for free.
//!
//! # Why the key is a "context id" and not a panel, a widget or a type
//!
//! The application supplies a string at the right-click site:
//! `"canvas.object"`, `"dock.tab"`, `"pages.thumbnail"`. The shell never
//! interprets it — it is a lookup key and nothing else.
//!
//! That is the purity rule ([`crate`]'s "the hard boundary") applied to
//! menus. The alternative designs all require the shell to know something
//! it must not:
//!
//! - **Keyed by selection type** (`Text`, `Annotation`, `Page`) — the shell
//!   would have to own an enumeration of the application's document model.
//! - **Keyed by panel id** — a panel usually has more than one menu (a
//!   thumbnail and the empty space beside it are different right-clicks)
//!   and some menus belong to no panel at all.
//! - **Keyed by widget id** — an `egui::Id` is not serializable in any form
//!   an operator could type into a customization file, which would retire
//!   the whole point.
//!
//! A string decided by the application is the only key that survives all
//! three, and it is exactly what a command id already is.
//!
//! # Where the menus live: on the `Shell`, beside the ribbon
//!
//! A menu belongs in the same document as the ribbon it mirrors: one file
//! to ship, one file to merge, one file for the operator to edit. So
//! [`Shell::menus`] carries them, and [`menus_of`] is the **single**
//! function that reads that field — nothing else in this crate touches it,
//! so there is one answer to "where did this menu come from".
//!
//! No entry point requires the manifest, though. Every one of them takes a
//! [`MenuLookup`], [`Menus`] implements it, and an application that keeps
//! its menus in a separate `.ron` file loads them with [`Menus::from_ron`]
//! and loses nothing.

use serde::{Deserialize, Serialize};

use crate::manifest::{CommandCatalog, Item, Shell};

use super::shortcut::Shortcuts;

/// One context menu: the ordered items shown when the operator
/// right-clicks a surface the application has labelled [`Self::context`].
///
/// Field-for-field the same shape as [`crate::manifest::Group`], for the
/// reason the module header gives. In particular [`Self::items`] is
/// `Option` rather than `Vec` for the *same* reason a group's is: the
/// `Option` is what distinguishes **"this menu is now empty"** from **"do
/// not mention this menu"**, and a customization layer needs to be able to
/// say both.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Menu {
    /// The application-supplied context id, e.g. `"canvas.object"`.
    ///
    /// Never displayed. Dotted lowercase by convention, matching command
    /// ids; the shell enforces no shape, because a shell that rejected an
    /// application's naming scheme would be dictating something that is
    /// none of its business.
    pub context: String,
    /// The items, in display order.
    ///
    /// Absent means "this is a reference to a menu, not a definition of
    /// one" — see [`Menus::overlay`]. Present-and-empty means a menu that
    /// deliberately offers nothing, which under
    /// [`crate::menu::plan::offers_anything`] is a menu that never opens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<Item>>,
}

impl Menu {
    /// A menu for `context` with no items yet.
    #[must_use]
    pub fn new(context: impl Into<String>) -> Self {
        Self {
            context: context.into(),
            items: Some(Vec::new()),
        }
    }

    /// A menu **reference** for a customization layer: names the context
    /// and overrides nothing.
    ///
    /// Distinguished from [`Self::new`] by `items: None`, which
    /// [`Menus::overlay`] reads as "leave the existing items alone".
    #[must_use]
    pub fn patch(context: impl Into<String>) -> Self {
        Self {
            context: context.into(),
            items: None,
        }
    }

    /// Set the items.
    #[must_use]
    pub fn with_items(mut self, items: impl IntoIterator<Item = Item>) -> Self {
        self.items = Some(items.into_iter().collect());
        self
    }

    /// The items, or an empty slice if unstated.
    #[must_use]
    pub fn items(&self) -> &[Item] {
        self.items.as_deref().unwrap_or(&[])
    }

    /// Every command id this menu names, in order, including repeats.
    pub fn command_ids(&self) -> impl Iterator<Item = &str> {
        self.items().iter().filter_map(Item::command_id)
    }
}

/// Every context menu a shell defines, keyed by context id.
///
/// A `Vec` rather than a `BTreeMap` because the on-disk form must be
/// hand-editable and a list of `Menu(context: "…", items: […])` reads far
/// better in RON than a map whose key is repeated inside its own value.
/// Lookup is linear, over a collection whose realistic size is under
/// twenty, once per right-click.
///
/// Duplicate contexts are refused by [`Self::validate`]; [`Self::get`]
/// returns the **first** match, so an unvalidated document degrades to
/// "the first definition wins" rather than to something order-dependent
/// and invisible.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Menus(pub Vec<Menu>);

/// The empty catalog every "there are no menus" answer points at.
///
/// A `static` so [`menus_of`] can return a reference with no allocation
/// and no lifetime gymnastics. `Vec::new` is `const`, so this costs
/// nothing at run time.
static EMPTY: Menus = Menus(Vec::new());

impl Menus {
    /// An empty catalog.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The menu for a context id, if one is defined.
    #[must_use]
    pub fn get(&self, context_id: &str) -> Option<&Menu> {
        self.0.iter().find(|m| m.context == context_id)
    }

    /// Every menu, in document order.
    pub fn iter(&self) -> impl Iterator<Item = &Menu> {
        self.0.iter()
    }

    /// How many menus are defined.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether no menu is defined.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Add a menu.
    #[must_use]
    pub fn with(mut self, menu: Menu) -> Self {
        self.0.push(menu);
        self
    }

    /// **Apply a customization layer, per menu and per item.**
    ///
    /// This is the menu half of `SHELL_FRAMEWORK.md` §5's customization
    /// contract, and it follows [`crate::manifest::merge`]'s rules rather
    /// than inventing softer ones:
    ///
    /// | The layer says | The result |
    /// |---|---|
    /// | `Menu(context: "canvas.object", items: [...])` for a known context | that menu's items are **replaced** |
    /// | `Menu(context: "canvas.object")` — no `items` key | nothing changes; the entry is a reference |
    /// | `Menu(context: "new.thing", items: [...])` | the menu is **added** |
    ///
    /// Replacement rather than element-wise splicing, because an item list
    /// has no per-item key to merge on — `Separator` is not identified by
    /// anything, and an operator reordering a menu is expressing an order,
    /// not a set of moves. This is the same conclusion the manifest's
    /// group merge reaches for the same reason.
    ///
    /// **It is deliberately fail-soft and validates nothing.** A layer
    /// naming a command that this build does not have is not an error
    /// here: [`crate::menu::plan::resolve`] omits it at render time and
    /// discloses the omission, which is what
    /// `GUI_ROADMAP.md`'s no-placeholders rule requires anyway. Rejecting
    /// the layer instead would throw away an operator's whole
    /// customization over one stale id — the failure mode `merge`'s header
    /// exists to argue against.
    ///
    /// An application that wants the stale id reported at *load* time
    /// rather than at render time calls [`Self::validate_against`].
    pub fn overlay(&mut self, layer: &Menus) {
        for incoming in &layer.0 {
            let Some(items) = incoming.items.as_ref() else {
                // A reference with no `items` key overrides nothing. It is
                // not an error and not a no-op worth disclosing: it is how
                // a layer mentions a menu it does not wish to change.
                continue;
            };
            match self.0.iter_mut().find(|m| m.context == incoming.context) {
                Some(existing) => existing.items = Some(items.clone()),
                None => self.0.push(incoming.clone()),
            }
        }
    }

    /// Check everything checkable without a command registry.
    ///
    /// # What is checked
    ///
    /// 1. Every context id is non-empty. An empty key cannot be
    ///    right-clicked, because no application would pass `""` at a call
    ///    site, so such a menu is unreachable rather than merely odd.
    /// 2. Context ids are unique. Two definitions of one key means
    ///    [`Self::get`] silently ignores one of them, and *which* one is
    ///    a fact about document order that nobody reading the file would
    ///    think to check.
    /// 3. No command appears twice **within one menu**. A menu offering
    ///    "Delete" twice is a defect in every case; unlike the ribbon's
    ///    one-command-one-tab rule this is a within-menu check, because a
    ///    command appearing in *several* menus is the entire point (see
    ///    [`Self::validate`]'s note below).
    ///
    /// # What is deliberately *not* checked
    ///
    /// **A command may appear in as many menus as it likes, and in a menu
    /// as well as on a ribbon tab.** `RIBBON_IA.md` §5 states this
    /// explicitly: the context menu *"carries the same commands again …
    /// that is not duplication in the P1 sense — context menus are not
    /// tabs"*. `Shell::validate`'s one-command-one-tab check therefore walks
    /// `all_tabs()` only; extending it over menus would forbid the design.
    ///
    /// # Errors
    ///
    /// The first failure found, in the order above — same rationale as
    /// [`Shell::validate`]: the second failure is very often a consequence
    /// of the first, and forty errors for one edit teaches a reader to
    /// ignore the list.
    pub fn validate(&self) -> Result<(), MenuError> {
        let mut seen: Vec<&str> = Vec::new();
        for menu in &self.0 {
            if menu.context.trim().is_empty() {
                return Err(MenuError::EmptyContextId);
            }
            if seen.contains(&menu.context.as_str()) {
                return Err(MenuError::DuplicateContext {
                    context: menu.context.clone(),
                });
            }
            seen.push(&menu.context);

            let mut commands: Vec<&str> = Vec::new();
            for id in menu.command_ids() {
                if commands.contains(&id) {
                    return Err(MenuError::DuplicateCommandInMenu {
                        context: menu.context.clone(),
                        command: id.to_owned(),
                    });
                }
                commands.push(id);
            }
        }
        Ok(())
    }

    /// [`Self::validate`], plus: every command named is registered.
    ///
    /// **Opt-in, and never a precondition of rendering.** The renderer's
    /// contract for an unregistered id is *omission with a disclosure*
    /// ([`crate::menu::plan::resolve`]), because
    /// `GUI_ROADMAP.md`'s no-placeholders rule says a command that does not
    /// exist in this build must be absent, not greyed. This function
    /// exists for the application that would rather find its own typo at
    /// start-up than at right-click time, which is a different question
    /// from what the operator should see.
    ///
    /// # Errors
    ///
    /// [`MenuError::UnknownCommand`], naming the context and the id, so
    /// the message points at the line in the file rather than at the file.
    pub fn validate_against(&self, catalog: &dyn CommandCatalog) -> Result<(), MenuError> {
        self.validate()?;
        for menu in &self.0 {
            for id in menu.command_ids() {
                if !catalog.contains(id) {
                    return Err(MenuError::UnknownCommand {
                        context: menu.context.clone(),
                        command: id.to_owned(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Parse a menu catalog from RON.
    ///
    /// The ordinary path is [`Shell::from_ron`], which reads the menus out
    /// of the same document as the ribbon. This is the escape hatch for an
    /// application that would rather keep its menus in a file of their own,
    /// or ship them without using the manifest at all.
    ///
    /// # Errors
    ///
    /// [`MenuError::Parse`], carrying RON's line and column.
    pub fn from_ron(text: &str) -> Result<Self, MenuError> {
        Ok(ron_options().from_str(text)?)
    }

    /// Serialize to compact RON.
    ///
    /// # Errors
    ///
    /// [`MenuError::Serialize`] if RON refuses the value, which for this
    /// type's fields should not be reachable.
    pub fn to_ron(&self) -> Result<String, MenuError> {
        Ok(ron_options().to_string(self)?)
    }

    /// Serialize to indented RON, for a file a human will open.
    ///
    /// # Errors
    ///
    /// As [`Self::to_ron`].
    pub fn to_ron_pretty(&self) -> Result<String, MenuError> {
        // Through the manifest's `tidy` for the reason its own doc gives:
        // RON 0.8 breaks every struct variant across three lines, and
        // `Item::Command` is one. A menu document is edited by hand exactly
        // as a ribbon manifest is, and the two must not disagree about how a
        // command is spelled on disk.
        Ok(crate::manifest::tidy(&ron_options().to_string_pretty(
            self,
            ron::ser::PrettyConfig::default().extensions(IMPLICIT_SOME),
        )?))
    }
}

impl FromIterator<Menu> for Menus {
    fn from_iter<I: IntoIterator<Item = Menu>>(iter: I) -> Self {
        Menus(iter.into_iter().collect())
    }
}

/// The one RON extension this crate's documents are written in.
///
/// Named separately from [`ron_options`] so the two spellings — the
/// parser's and the pretty printer's — cannot drift apart within this
/// file.
const IMPLICIT_SOME: ron::extensions::Extensions = ron::extensions::Extensions::IMPLICIT_SOME;

/// The RON dialect a menu document is read and written in.
///
/// # This must stay identical to [`crate::manifest`]'s
///
/// It is a second copy of one decision, which is a drift hazard, and the
/// alternative was worse: `manifest::ron_options` is private and
/// `manifest/` is not this module's to edit. So the copy is made
/// deliberately, kept to one constant, and pinned by
/// `the_menu_ron_dialect_matches_the_manifests`, which parses the same
/// implicit-`Some` spelling through both types.
///
/// `IMPLICIT_SOME` is not cosmetic. Nearly every field here is an
/// `Option`, because the `Option` is what distinguishes "set this to
/// empty" from "do not mention this". Without the extension a present
/// value must be written `items: Some([…])`, and the obvious spelling —
/// the one every example in this module uses — fails to parse with
/// `ExpectedOption`, a message that means nothing to someone who has never
/// seen a Rust `Option`. The operator's file is precisely the one that
/// must not fail.
fn ron_options() -> ron::Options {
    ron::Options::default().with_default_extension(IMPLICIT_SOME)
}

/// Why a menu document was refused.
///
/// Every variant carries the offending identifiers as fields rather than
/// interpolated into prose, for the reason
/// [`crate::manifest::ManifestError`] gives: the document is hand-edited,
/// so an error must say *which line*, and an application must be able to
/// act on it — offering "reset this one menu" rather than "your file is
/// broken".
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MenuError {
    /// A menu has no context id, so nothing could ever look it up.
    #[error("a menu has an empty context id; no right-click site can name it")]
    EmptyContextId,
    /// Two menus claim the same context id.
    #[error("context id `{context}` defines more than one menu; a context id must resolve to one")]
    DuplicateContext {
        /// The id used twice.
        context: String,
    },
    /// One menu lists the same command twice.
    #[error("menu `{context}` lists `{command}` twice")]
    DuplicateCommandInMenu {
        /// The menu holding both.
        context: String,
        /// The duplicated command id.
        command: String,
    },
    /// A referenced command is not registered.
    ///
    /// Only ever produced by [`Menus::validate_against`]. The **renderer**
    /// omits such an item and discloses it instead; see that function's
    /// documentation for why the two answers differ.
    #[error("`{command}` is not a registered command (referenced by menu `{context}`)")]
    UnknownCommand {
        /// The menu that named it.
        context: String,
        /// The unregistered id.
        command: String,
    },
    /// The document could not be parsed.
    #[error("the menu document could not be parsed: {0}")]
    Parse(#[from] ron::error::SpannedError),
    /// The document could not be written.
    #[error("the menu document could not be written: {0}")]
    Serialize(#[from] ron::Error),
}

/// Anything that can produce the menu for a context id.
///
/// # Why the entry points take this rather than a `&Shell`
///
/// [`Shell`] is where menus belong, and it is not the only place they can
/// come from: an application may build a menu in code, or load one from a
/// file of its own, and neither has a `Shell` to hand. A renderer that
/// demanded one would force such an application to construct an otherwise
/// empty manifest, and a rendering test to do the same — which is how a
/// suite ends up green because there was nothing for it to fail against
/// (the failure mode `ribbon/testfont.rs` exists to prevent).
///
/// A trait removes the choice. [`Menus`], a single [`Menu`] and [`Shell`]
/// all implement it, and every entry point in this module accepts whichever
/// of the three the caller already has.
pub trait MenuLookup {
    /// The menu for this context id, if one is defined.
    fn menu_for(&self, context_id: &str) -> Option<&Menu>;

    /// The chord hints to show beside this document's commands.
    ///
    /// Defaults to none, which is the honest answer for a bare [`Menus`]:
    /// a catalog of menus carries no key bindings, and inventing some
    /// would be the second copy of a keymap that
    /// [`crate::menu::shortcut`] exists to prevent.
    ///
    /// [`Shell`] overrides it with its own keymap, which is the whole
    /// reason a menu wants to be in the same document as the ribbon. An
    /// application whose accelerators live elsewhere supplies them
    /// explicitly with
    /// [`crate::menu::ContextMenu::with_shortcuts`].
    fn shortcuts(&self) -> Shortcuts {
        Shortcuts::none()
    }
}

impl MenuLookup for Menus {
    fn menu_for(&self, context_id: &str) -> Option<&Menu> {
        self.get(context_id)
    }
}

impl MenuLookup for Menu {
    /// A single menu is a catalog of one, which lets an application that
    /// builds a menu in code hand it straight to the renderer.
    fn menu_for(&self, context_id: &str) -> Option<&Menu> {
        (self.context == context_id).then_some(self)
    }
}

impl MenuLookup for Shell {
    fn menu_for(&self, context_id: &str) -> Option<&Menu> {
        menus_of(self).get(context_id)
    }

    /// The manifest's own keymap, inverted. This is the payoff of putting
    /// menus in the same document as the ribbon: an operator who rebinds a
    /// key sees the menu follow, with nothing else to keep in step.
    fn shortcuts(&self) -> Shortcuts {
        Shortcuts::of(self)
    }
}

/// **The one function that reads [`Shell::menus`].**
///
/// Everything in this crate that asks a `Shell` for a menu comes through
/// here, so the field has exactly one reader and the absent case has
/// exactly one answer.
///
/// A manifest that declares no menus resolves to the empty catalog rather
/// than to an error: no menu for a context is a right-click that does nothing,
/// which is the correct behaviour and not a failure.
pub(crate) fn menus_of(shell: &Shell) -> &Menus {
    shell.menus.as_ref().unwrap_or(&EMPTY)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{Command, CommandRegistry, HandlerToken};

    pub(super) fn canvas_menu() -> Menu {
        Menu::new("canvas.object").with_items([
            Item::command("edit.cut"),
            Item::command("edit.copy"),
            Item::Separator,
            Item::command("edit.delete"),
        ])
    }

    fn registry() -> CommandRegistry {
        let mut r = CommandRegistry::new();
        r.register_all([
            Command::new("edit.cut", "Cut", HandlerToken::new(1)),
            Command::new("edit.copy", "Copy", HandlerToken::new(2)),
            Command::new("edit.delete", "Delete", HandlerToken::new(3)),
        ])
        .expect("distinct ids");
        r
    }

    /// **A menu document survives a trip through RON unchanged.**
    ///
    /// The whole value proposition is that a menu is a *file*: an operator
    /// edits it, an application ships one. Every one of those claims fails
    /// if the round trip is lossy, and both forms are checked because a
    /// pretty printer that emits something its own parser rejects is
    /// discovered by the operator rather than by CI.
    #[test]
    fn a_menu_document_round_trips_through_ron() {
        let original =
            Menus::new()
                .with(canvas_menu())
                .with(Menu::new("pages.thumbnail").with_items([
                    Item::command("pages.rotate_left"),
                    Item::custom("page_scale"),
                ]));

        let compact = original.to_ron().expect("serializes");
        assert_eq!(
            Menus::from_ron(&compact).expect("compact parses"),
            original,
            "the compact round trip lost or changed something"
        );

        let pretty = original.to_ron_pretty().expect("serializes");
        assert_eq!(
            Menus::from_ron(&pretty).expect("pretty parses"),
            original,
            "the pretty round trip lost or changed something"
        );

        // The shapes this module documents must actually appear, or the
        // examples are fiction.
        //
        // The one-token spelling is asserted on the COMPACT form because
        // `Item::Command` is a struct variant, and RON's pretty printer
        // breaks a struct variant across three lines. A `contains` for the
        // one-line spelling would fail on the pretty form of a perfectly
        // correct document; the compact form is where the spelling this line
        // pins is a single token.
        let compact = original.to_ron().expect("serializes");
        assert!(compact.contains("Command(id:\"edit.cut\")"), "{compact}");
        assert!(pretty.contains("Separator"), "{pretty}");
        assert!(pretty.contains("canvas.object"), "{pretty}");
    }

    /// **The menu RON dialect is the manifest's dialect.**
    ///
    /// [`ron_options`] is a second copy of one decision, made because
    /// `manifest`'s is private and `manifest/` is not this module's to
    /// edit. A copy is only safe while something checks the two agree, and
    /// the way they would silently disagree is `IMPLICIT_SOME`: with it,
    /// `items: [...]` parses; without it, the same text fails with
    /// `ExpectedOption` and every hand-written menu file in existence
    /// stops loading.
    ///
    /// So the same implicit-`Some` spelling is pushed through both types.
    #[test]
    fn the_menu_ron_dialect_matches_the_manifests() {
        let menus = Menus::from_ron(
            r#"[
                // A hand-written file: comments, trailing commas, no `Some(…)`.
                Menu(context: "canvas.object", items: [ Command(id: "edit.delete") ]),
            ]"#,
        )
        .expect("implicit-some, comments and trailing commas must parse");
        assert_eq!(menus.len(), 1);
        assert_eq!(menus.get("canvas.object").expect("found").items().len(), 1);

        // The same dialect, through the manifest's own parser.
        let shell = Shell::from_ron(r#"Shell(tabs: [ Tab(id: "tools") ])"#)
            .expect("the manifest accepts the same spelling");
        assert_eq!(shell.tabs().len(), 1);
    }

    /// **An unstated `items` stays unstated through a round trip.**
    ///
    /// This is the property [`Menus::overlay`] rests on: `None` means "do
    /// not mention this" and must not come back as `Some(empty)`. If a
    /// layer's omitted `items` round-tripped into `Some(vec![])`, saving
    /// and reloading a customization would silently empty every menu it
    /// mentioned — turning a reference into a deletion.
    #[test]
    fn an_unstated_item_list_stays_unstated_through_a_round_trip() {
        let layer = Menus::new().with(Menu::patch("canvas.object"));
        let text = layer.to_ron().expect("serializes");
        assert!(
            !text.contains("items"),
            "an unstated field must not be written at all; got {text}"
        );
        let back = Menus::from_ron(&text).expect("parses");
        assert_eq!(back, layer);
        assert!(back.0[0].items.is_none(), "`None` must not resurrect");
    }

    /// **Customization: replace, add, and reference.**
    ///
    /// The three rows of [`Menus::overlay`]'s table, asserted together
    /// because the interesting part is that they coexist — a layer that
    /// mentions three menus must be able to change one, extend the set,
    /// and leave the third alone in a single document.
    #[test]
    fn a_layer_replaces_adds_and_leaves_alone() {
        let mut menus = Menus::new()
            .with(canvas_menu())
            .with(Menu::new("dock.tab").with_items([Item::command("panel.close")]));

        menus.overlay(
            &Menus::new()
                // Replace: the operator wants Delete first and no Cut.
                .with(
                    Menu::new("canvas.object")
                        .with_items([Item::command("edit.delete"), Item::command("edit.copy")]),
                )
                // Reference: mentioned, unchanged.
                .with(Menu::patch("dock.tab"))
                // Add: a context the built-in document never defined.
                .with(Menu::new("pages.thumbnail").with_items([Item::command("pages.extract")])),
        );

        assert_eq!(
            menus.get("canvas.object").expect("kept").items(),
            [Item::command("edit.delete"), Item::command("edit.copy")],
            "a layer stating `items` replaces the list wholesale"
        );
        assert_eq!(
            menus.get("dock.tab").expect("kept").items(),
            [Item::command("panel.close")],
            "a layer entry with no `items` key must change nothing — it is a \
             reference to a menu, not an instruction to empty it"
        );
        assert_eq!(
            menus.get("pages.thumbnail").expect("added").items().len(),
            1,
            "a layer may introduce a context the built-in document never had"
        );
        assert_eq!(menus.len(), 3);
    }

    /// A layer naming a command this build does not have is **not** an
    /// error. It survives the overlay and is dealt with at render time by
    /// omission — see [`Menus::overlay`] and the no-placeholders rule.
    #[test]
    fn a_layer_with_a_stale_command_is_not_refused() {
        let mut menus = Menus::new().with(canvas_menu());
        menus.overlay(
            &Menus::new().with(
                Menu::new("canvas.object")
                    .with_items([Item::command("edit.copy"), Item::command("edit.telepathy")]),
            ),
        );
        assert_eq!(menus.get("canvas.object").expect("kept").items().len(), 2);
        menus
            .validate()
            .expect("structure alone is fine; only the id is unknown");
        let err = menus
            .validate_against(&registry())
            .expect_err("an application that asks is told");
        assert_eq!(
            err,
            MenuError::UnknownCommand {
                context: "canvas.object".to_owned(),
                command: "edit.telepathy".to_owned(),
            }
        );
    }

    /// Each structural defect is refused and **named**.
    ///
    /// Naming is the requirement, not a courtesy: this file is hand-edited
    /// and "your menus are invalid" tells the operator to go and bisect it.
    #[test]
    fn structural_defects_are_each_named() {
        assert_eq!(
            Menus::new()
                .with(Menu::new("  "))
                .validate()
                .expect_err("an unreachable menu is refused"),
            MenuError::EmptyContextId
        );

        assert_eq!(
            Menus::new()
                .with(canvas_menu())
                .with(Menu::new("canvas.object"))
                .validate()
                .expect_err("two definitions of one key"),
            MenuError::DuplicateContext {
                context: "canvas.object".to_owned()
            }
        );

        assert_eq!(
            Menus::new()
                .with(Menu::new("canvas.object").with_items([
                    Item::command("edit.delete"),
                    Item::Separator,
                    Item::command("edit.delete"),
                ]))
                .validate()
                .expect_err("one menu offering Delete twice is always a defect"),
            MenuError::DuplicateCommandInMenu {
                context: "canvas.object".to_owned(),
                command: "edit.delete".to_owned(),
            }
        );
    }

    /// **A command may appear in many menus, and on a tab as well.**
    ///
    /// `RIBBON_IA.md` §5: the context menu *"carries the same commands
    /// again … that is not duplication in the P1 sense — context menus are
    /// not tabs"*. If a future edit extends the one-command-one-tab rule
    /// over menus, this is the test that says no.
    #[test]
    fn one_command_may_appear_in_several_menus() {
        Menus::new()
            .with(Menu::new("canvas.object").with_items([Item::command("edit.delete")]))
            .with(Menu::new("pages.thumbnail").with_items([Item::command("edit.delete")]))
            .with(Menu::new("dock.tab").with_items([Item::command("edit.delete")]))
            .validate_against(&registry())
            .expect("a command may be offered by every menu that can act on it");
    }

    /// [`Menus::get`] and the two blanket [`MenuLookup`] impls answer the
    /// same question, so a caller may hand the renderer whichever it has.
    #[test]
    fn a_catalog_a_single_menu_and_a_shell_all_answer_lookups() {
        let menus = Menus::new().with(canvas_menu());
        assert!(menus.menu_for("canvas.object").is_some());
        assert!(menus.menu_for("canvas.nothing").is_none());

        let one = canvas_menu();
        assert!(one.menu_for("canvas.object").is_some());
        assert!(
            one.menu_for("dock.tab").is_none(),
            "a menu is a catalog of exactly one context, not a wildcard"
        );
    }

    /// **A `Shell` carries its menus, and both arms of the `Option` work.**
    ///
    /// The `None` arm matters as much as the `Some` arm: a manifest that
    /// declares no menus is the common case, and it must resolve to "no
    /// menu for this context" — which the renderer turns into a right-click
    /// that does nothing — rather than to a panic or an empty popup.
    #[test]
    fn a_shell_carries_its_menus_and_an_absent_field_offers_none() {
        // The `None` arm — a manifest that never mentions menus.
        let bare = Shell::new();
        assert!(
            menus_of(&bare).is_empty(),
            "a `Shell` with no `menus` field must resolve to no menus"
        );
        assert!(
            bare.menu_for("canvas.object").is_none(),
            "and with no menus, no context resolves"
        );

        // The `Some` arm — the field carrying a real menu.
        let mut carrying = Shell::new();
        carrying.menus = Some(Menus(vec![canvas_menu()]));
        assert!(
            !menus_of(&carrying).is_empty(),
            "`menus_of` must read the field, not the empty stand-in"
        );
        let found = carrying
            .menu_for("canvas.object")
            .expect("the menu the shell carries must resolve by its context id");
        assert_eq!(
            found.context, "canvas.object",
            "and it must be the menu that was put in, not another"
        );
        assert!(
            carrying.menu_for("canvas.nothing-here").is_none(),
            "an unknown context still resolves to nothing"
        );
    }

    /// **One document carries the ribbon and its menus, hand-written.**
    ///
    /// The claim under test is that a `menus:` key in a shell document
    /// survives the parse. It is worth a test of its own because the failure
    /// is silent: `serde` ignores a field it does not know, so a shell that
    /// did not carry menus would load the file, draw the ribbon, and leave
    /// the operator's context menus doing nothing with no error anywhere.
    ///
    /// The input is deliberately **hand-written** rather than produced by
    /// the serializer. A round-trip test cannot detect an ergonomics defect
    /// — writer and reader agree by construction — and this dialect needs
    /// `IMPLICIT_SOME`, without which every one of these fields would
    /// require a `Some(…)` wrapper that no operator would think to type.
    /// See `D:/dev/rag/rust/ron_without_implicit_some_makes_every_optional_field_unwritable_by_hand.md`.
    #[test]
    fn one_shell_document_carries_both_the_ribbon_and_its_menus() {
        let shell = Shell::from_ron(
            r#"Shell(
                tabs: [ Tab(id: "view") ],
                menus: [ Menu(context: "canvas.object", items: [ Command(id: "edit.delete") ]) ],
            )"#,
        )
        .expect("a hand-written document carrying both must parse");

        assert_eq!(shell.tabs().len(), 1, "the ribbon half still loads");

        let menu = shell
            .menu_for("canvas.object")
            .expect("the menus half must now survive the parse");
        assert_eq!(
            menu.command_ids().collect::<Vec<_>>(),
            ["edit.delete"],
            "and carry its items, in order"
        );
    }
}
