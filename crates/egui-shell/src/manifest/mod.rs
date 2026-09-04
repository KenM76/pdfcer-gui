//! The manifest — the serializable definition of what the shell *is*.
//!
//! # The decision this module implements
//!
//! `SHELL_FRAMEWORK.md` §1, verbatim, because everything here follows
//! from it:
//!
//! > **The shell is data. Tabs, groups, commands, panels, layouts, modes
//! > and key bindings are a serializable document that the application
//! > *supplies* and the operator *edits* — not code that has to be
//! > recompiled to change.**
//!
//! Two requirements arrived together on this project: be **reusable**
//! across applications, and be **customizable** at runtime by the
//! operator. A ribbon defined in Rust `match` arms can be neither. A
//! ribbon defined as data can be both, and the same serializer that lets
//! an operator save a customized ribbon lets a different application ship
//! a completely different one.
//!
//! It also retires a deferral. The salvage source's ribbon deferred
//! customization on the grounds that *"a customisable ribbon that also
//! forgets itself would be worse than none."* That objection was about
//! persistence, and persistence is the first thing this design builds.
//!
//! # The type shape, and why one type is both document and patch
//!
//! [`Shell`] describes a complete ribbon. It is *also* the type of each
//! **layer** in the three-layer merge — the application-override file and
//! the operator's customization file are `Shell` values too, each
//! typically describing a handful of fields.
//!
//! That is why nearly every field is `Option`: the `Option` is what
//! distinguishes *"set this to empty"* from *"do not mention this"*, and
//! that distinction is the entire difference between a per-item override
//! and a wholesale replacement. A layer that says
//!
//! ```ron
//! Shell(tabs: [ Tab(id: "tools") ])
//! ```
//!
//! is saying "move the Tools tab to the front" — not "delete every other
//! tab", which is what a non-optional `tabs: Vec<Tab>` would have to mean.
//!
//! The cost is that a `Shell` can be *incomplete*, and the answer to that
//! is [`Shell::validate`]: **a complete manifest is one that validates.**
//! A layer is not expected to. The merged result is required to. One type,
//! two roles, and a checked boundary between them — rather than a second
//! `ShellPatch` type that would have to be kept in step with this one
//! field by field, forever.
//!
//! # What is checked, and where
//!
//! | Property | Enforced by |
//! |---|---|
//! | Structure: ids unique, labels present, groups captioned | [`Shell::validate`] |
//! | **One command appears on at most one tab** | [`Shell::validate`] |
//! | Every referenced command exists | [`Shell::validate_against`] |
//! | An operator's stale reference loses one item, not the layout | [`merge`] |
//!
//! The split is deliberate and is the difference between a *rejection*
//! and a *disclosure*:
//!
//! - **[`merge`] is fail-soft.** It is handed files an operator edited by
//!   hand and an application shipped two versions ago. A command that no
//!   longer exists loses that one item and produces a [`Skip`] the
//!   application can show in its status surface. It does not discard the
//!   layout, and it does not fail.
//! - **[`Shell::validate`] is strict.** It runs on the *merged* result,
//!   and what it rejects are contradictions no fail-soft rule can repair —
//!   two tabs with one id, a command on two tabs. `SHELL_FRAMEWORK.md`
//!   §5 makes the point that this is strictly more than the salvage
//!   source's compile-time ownership test could do: that test could only
//!   check the ribbon the developers wrote, and this one checks the ribbon
//!   the operator ends up with.
//!
//! # Commands are referenced, never defined
//!
//! A manifest contains command **ids**. It contains no labels for them, no
//! icons, no handlers, and no way to add any. Those live in
//! [`crate::commands::CommandRegistry`], in code.
//!
//! That is what stops a customized ribbon from inventing a command that
//! does not exist, and it is why an unknown id can be a disclosed skip
//! rather than a crash: there is nothing for the shell to try to run.
//!
//! # On-disk form
//!
//! RON, via [`Shell::from_ron`] and [`Shell::to_ron`]. RON rather than
//! JSON because the format has real enums (`Command("view.single")` beside
//! `Separator`), comments, and trailing commas — all three matter for a
//! file an operator edits by hand, which is the whole point of the
//! customization layer.
//!
//! ```ron
//! Shell(
//!     modes: [
//!         Mode(id: "read",   label: "Read",   tabs: ["file", "view"]),
//!         Mode(id: "review", label: "Review", tabs: ["file", "view", "pages", "markup", "measure"]),
//!     ],
//!     tabs: [
//!         Tab(id: "view", label: "View", question: "What is on my screen?", groups: [
//!             Group(id: "page_display", caption: "Page display", items: [
//!                 Command("view.single"), Command("view.continuous"),
//!             ]),
//!         ]),
//!     ],
//!     contextual_tabs: [
//!         Tab(id: "format", label: "Format", visible_when: "selection.any", groups: []),
//!     ],
//!     qat: ["file.open", "file.save_copy", "edit.undo", "edit.redo"],
//!     keymap: { "Ctrl+E": "edit.text", "Ctrl+1": "mode.read", "F11": "view.fullscreen" },
//! )
//! ```

mod merge;
mod validate;

pub use merge::{Layer, MergeInput, MergeReport, Merged, Skip, SkipReason, merge};
pub use validate::{ManifestError, Site};

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Anything that can say whether a command id is real.
///
/// [`crate::commands::CommandRegistry`] implements it. The trait exists so
/// this module does not depend on that one: a manifest must be parseable,
/// mergeable and round-trippable by a tool that has no registry at all —
/// a schema linter, a diff viewer, `tools/ui-verify` inspecting a `.ron`
/// file without linking the application.
pub trait CommandCatalog {
    /// Whether this id names a real command.
    fn contains(&self, id: &str) -> bool;
}

/// A catalog that accepts every id.
///
/// For tests, for tooling that has no registry, and for the first stage
/// of an application's own bring-up. Using it in production would disable
/// the check that makes an unknown id a disclosed skip, which is why it is
/// a named type at a call site rather than a default.
#[derive(Debug, Clone, Copy, Default)]
pub struct AnyCommand;

impl CommandCatalog for AnyCommand {
    fn contains(&self, _id: &str) -> bool {
        true
    }
}

/// The whole shell definition — or one layer of it. See the module header
/// on why those are the same type.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Shell {
    /// The schema version this document was written against.
    ///
    /// `0` means "unstated", which is what a hand-written operator file
    /// will usually be, and is treated as the current version. A value
    /// **above** [`Shell::SCHEMA`] means the file came from a newer build:
    /// [`merge`] skips that whole layer with a disclosure rather than
    /// guessing, because a field it does not understand may be the one
    /// that makes the rest of the file mean what it says.
    #[serde(skip_serializing_if = "is_zero")]
    pub schema: u32,
    /// Named workspaces. Each names the tabs it contains.
    ///
    /// `MODES_AND_PANELS.md`: *a mode is a named workspace layout*, and
    /// Read/Review/Edit is a **configuration**, not a built-in. Nothing in
    /// this crate knows those three names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modes: Option<Vec<Mode>>,
    /// The ordinary tabs, in display order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tabs: Option<Vec<Tab>>,
    /// Tabs that appear only while their [`Tab::visible_when`] condition
    /// holds — a Format tab that appears on selection, say.
    ///
    /// Separate from [`Self::tabs`] because they are *not* mode members:
    /// a mode names a fixed tab set, and a contextual tab's whole nature
    /// is that its presence is decided by application state rather than by
    /// configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contextual_tabs: Option<Vec<Tab>>,
    /// The quick-access toolbar: command ids, in order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qat: Option<Qat>,
    /// The **trailing controls** — the far right of the tab-strip row, past
    /// the mode selector. See [`Trailing`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing: Option<Trailing>,
    /// Key chord → command id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keymap: Option<Keymap>,
    /// Context menus, keyed by the context id the application supplies at
    /// the right-click site — `"canvas.object"`, `"dock.tab"`, and so on.
    ///
    /// A menu is a [`Group`] in every respect except what keys it, so it
    /// carries the same [`Item`] list and is customized the same way. See
    /// [`crate::menu`].
    ///
    /// **Deliberately not covered by [`Shell::validate`]'s
    /// one-command-one-tab rule.** `RIBBON_IA.md` §6 is explicit that a
    /// context menu carrying the same commands as a tab *"is not
    /// duplication in the P1 sense — context menus are not tabs"*: the rule
    /// exists so a command has one discoverable **home**, and a menu is a
    /// shortcut to that home rather than a rival to it. `validate` walks
    /// `all_tabs()` only, and `one_command_may_appear_in_several_menus`
    /// says so in a test.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub menus: Option<crate::menu::Menus>,
}

/// `skip_serializing_if` predicate for [`Shell::schema`].
///
/// Takes a reference because that is serde's required signature for a
/// `skip_serializing_if` path, not because a `u32` wants one.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(v: &u32) -> bool {
    *v == 0
}

impl Shell {
    /// The schema version this build writes and understands.
    ///
    /// Bump when a change would make an *older* build misread a newer
    /// file — not for an added optional field, which an older build
    /// already ignores safely.
    pub const SCHEMA: u32 = 1;

    /// An empty manifest stamped with the current schema.
    #[must_use]
    pub fn new() -> Self {
        Self {
            schema: Self::SCHEMA,
            ..Self::default()
        }
    }

    /// The ordinary tabs, or an empty slice if unstated.
    #[must_use]
    pub fn tabs(&self) -> &[Tab] {
        self.tabs.as_deref().unwrap_or(&[])
    }

    /// The contextual tabs, or an empty slice if unstated.
    #[must_use]
    pub fn contextual_tabs(&self) -> &[Tab] {
        self.contextual_tabs.as_deref().unwrap_or(&[])
    }

    /// The modes, or an empty slice if unstated.
    #[must_use]
    pub fn modes(&self) -> &[Mode] {
        self.modes.as_deref().unwrap_or(&[])
    }

    /// Every tab, ordinary then contextual.
    ///
    /// The one-command-one-tab rule counts contextual tabs, so most
    /// checks want this rather than [`Self::tabs`].
    pub fn all_tabs(&self) -> impl Iterator<Item = &Tab> {
        self.tabs().iter().chain(self.contextual_tabs())
    }

    /// Add an ordinary tab.
    #[must_use]
    pub fn with_tab(mut self, tab: Tab) -> Self {
        self.tabs.get_or_insert_with(Vec::new).push(tab);
        self
    }

    /// Add a contextual tab.
    #[must_use]
    pub fn with_contextual_tab(mut self, tab: Tab) -> Self {
        self.contextual_tabs.get_or_insert_with(Vec::new).push(tab);
        self
    }

    /// Add a mode.
    #[must_use]
    pub fn with_mode(mut self, mode: Mode) -> Self {
        self.modes.get_or_insert_with(Vec::new).push(mode);
        self
    }

    /// Set the quick-access toolbar.
    #[must_use]
    pub fn with_qat<I, S>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.qat = Some(Qat(ids.into_iter().map(Into::into).collect()));
        self
    }

    /// Set the trailing controls — the far right of the tab-strip row. See
    /// [`Trailing`].
    ///
    /// Takes [`Item`]s rather than ids, unlike [`Self::with_qat`], because the
    /// whole reason this region exists is that its controls carry a
    /// `visible_when` — see [`Trailing`]'s note on R9.
    #[must_use]
    pub fn with_trailing<I>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = Item>,
    {
        self.trailing = Some(items.into_iter().collect());
        self
    }

    /// Bind a key chord to a command.
    #[must_use]
    pub fn with_binding(mut self, chord: impl Into<String>, command: impl Into<String>) -> Self {
        self.keymap
            .get_or_insert_with(Keymap::default)
            .0
            .insert(chord.into(), command.into());
        self
    }

    /// Parse a manifest from RON.
    ///
    /// # Errors
    ///
    /// [`ManifestError::Parse`], carrying RON's own line and column. The
    /// span is the useful part: this file is hand-edited, and "expected
    /// `)` at 14:3" is the difference between a fixable typo and a file
    /// the operator reverts wholesale.
    ///
    /// Parsing does **not** validate. A layer is not expected to be a
    /// complete manifest, so refusing to parse one that is incomplete
    /// would make the layered design unrepresentable. Call
    /// [`Self::validate`] on the merged result.
    pub fn from_ron(text: &str) -> Result<Self, ManifestError> {
        Ok(ron_options().from_str(text)?)
    }

    /// Serialize to compact RON.
    ///
    /// # Errors
    ///
    /// [`ManifestError::Serialize`] if RON refuses the value, which for
    /// this type's fields should not be reachable.
    pub fn to_ron(&self) -> Result<String, ManifestError> {
        Ok(ron_options().to_string(self)?)
    }

    /// Serialize to indented RON, for a file a human will open.
    ///
    /// # Errors
    ///
    /// As [`Self::to_ron`].
    pub fn to_ron_pretty(&self) -> Result<String, ManifestError> {
        Ok(tidy(
            &ron_options().to_string_pretty(self, pretty_config())?,
        ))
    }
}

/// The RON dialect this manifest is read and written in.
///
/// # ★ Why `IMPLICIT_SOME`, and why it is not a cosmetic preference
///
/// Nearly every field of [`Shell`], [`Tab`], [`Group`] and [`Mode`] is an
/// `Option`, because the `Option` is what distinguishes *"set this to
/// empty"* from *"do not mention this"* — see the module header. That is
/// the right model in Rust and, in stock RON, a disaster on disk: a
/// present value has to be written `tabs: Some([…])`, and the operator's
/// customization file — the whole point of the format being editable —
/// fills up with a wrapper that carries no information at all.
///
/// Worse, it is a wrapper that is *easy to forget*. Writing
///
/// ```ron
/// Shell(tabs: [ Tab(id: "tools") ])
/// ```
///
/// is the obvious thing, it is what every example in this crate's
/// documentation shows, and under stock RON it fails to parse with
/// `ExpectedOption` — a message that means nothing to someone who has
/// never seen a Rust `Option`.
///
/// `IMPLICIT_SOME` makes the obvious spelling the correct one. It is set
/// on the [`ron::Options`] used for **both** directions rather than only
/// emitted as an `#![enable(implicit_some)]` header, because a header
/// only helps files this crate wrote. A file the operator wrote from
/// scratch, or pasted from documentation, has no header — and that file
/// is exactly the one that must not fail.
///
/// The header is emitted as well, by [`pretty_config`], so that a file
/// this crate writes is also readable by RON tooling that honours it.
fn ron_options() -> ron::Options {
    ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
}

/// Formatting for a manifest a person will open.
///
/// Carries the same extension as [`ron_options`] so the written file
/// declares the dialect it is in.
fn pretty_config() -> ron::ser::PrettyConfig {
    ron::ser::PrettyConfig::default().extensions(ron::extensions::Extensions::IMPLICIT_SOME)
}

/// The longest line [`tidy`] will produce by joining a block.
///
/// Generous, because these lines are nested four levels deep and the whole
/// point is that a one-field item reads as one thing. Past this, three short
/// lines really are easier to read than one long one.
const TIDY_MAX: usize = 100;

/// **Collapse a short multi-line block onto one line.**
///
/// # Why a manifest needs this at all
///
/// RON 0.8's pretty printer breaks **every** struct and struct variant across
/// lines, with no option to keep a short one inline — it has
/// `compact_arrays` and nothing for structs. That was invisible while
/// [`Item::Command`] was a tuple variant printed as `Command("file.open")`.
/// The moment it gained [`ItemSize`] and became a struct variant, every one of
/// pdfcer's hundred-odd ribbon items became
///
/// ```ron
/// Command(
///     id: "file.open",
/// ),
/// ```
///
/// — three lines and a two-thirds-empty column where there had been one line,
/// and the file grew by half.
///
/// ★★★ That is not a cosmetic complaint. This file's **entire purpose** is to
/// be read and edited by an operator: it is the customization surface
/// `SHELL_FRAMEWORK.md` §1 is about. A format that triples the length of its
/// most common construct has made itself worse at the one job it has.
///
/// # What it does, and the two guards on it
///
/// A block is joined when its body is one or two `key: value` lines, neither
/// containing a bracket of its own, and the joined result is under
/// [`TIDY_MAX`]. Anything nested, anything long, and anything it does not
/// recognise is left exactly as RON printed it.
///
/// ★ It is **safe by construction and by test**: the transform only ever
/// removes newlines and indentation between tokens RON itself emitted, which
/// RON's own parser is insensitive to — and
/// [`tests::a_manifest_round_trips_through_ron`] parses the tidied output
/// back and compares the whole value, so a transform that broke the document
/// would fail rather than ship.
pub(crate) fn tidy(pretty: &str) -> String {
    let lines: Vec<&str> = pretty.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        // An opener is a line ending in `(` — `Command(`, `Group(`, `Tab(`.
        let joined = line.trim_end().ends_with('(').then(|| {
            let mut body: Vec<&str> = Vec::new();
            let mut j = i + 1;
            while j < lines.len() && body.len() <= 2 {
                let inner = lines[j].trim();
                if inner.starts_with(')') {
                    // The closer. Join if we collected something usable.
                    let text = body.join(" ");
                    let candidate = format!("{}{}{}", line, text.trim_end_matches(','), inner);
                    return (!body.is_empty() && candidate.len() <= TIDY_MAX)
                        .then_some((candidate, j));
                }
                // Refuse anything that opens a block of its own; joining it
                // would swallow lines this pass has not looked at.
                if inner.contains('(') || inner.contains('[') || inner.contains('{') {
                    return None;
                }
                body.push(inner);
                j += 1;
            }
            None
        });
        match joined.flatten() {
            Some((text, closed_at)) => {
                out.push(text);
                i = closed_at + 1;
            }
            None => {
                out.push(line.to_owned());
                i += 1;
            }
        }
    }
    let mut text = out.join("\n");
    if pretty.ends_with('\n') {
        text.push('\n');
    }
    text
}

/// A named workspace: a label and the tabs it contains.
///
/// `MODES_AND_PANELS.md` Part 1 describes what a mode is for, and one
/// rule from it binds anything rendering this type:
///
/// > **A mode changes what is *visible*. It never makes a visible control
/// > silently inert.**
///
/// That is the difference between a mode and the master toggle it
/// replaced: the toggle left the editing tools on screen and made
/// gestures quietly do nothing. A mode *removes* the tools it disables, so
/// there is no click that mysteriously fails.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Mode {
    /// Stable id, e.g. `"review"`. Never displayed.
    pub id: String,
    /// The operator-visible label. Required in a complete manifest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The ids of the ordinary tabs this mode contains, in order.
    ///
    /// A reference to a tab that does not exist is dropped by [`merge`]
    /// with a disclosure — an operator's mode surviving a tab's removal
    /// minus one entry is better than the mode failing to load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tabs: Option<Vec<String>>,
}

impl Mode {
    /// A complete mode.
    #[must_use]
    pub fn new<I, S>(id: impl Into<String>, label: impl Into<String>, tabs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            id: id.into(),
            label: Some(label.into()),
            tabs: Some(tabs.into_iter().map(Into::into).collect()),
        }
    }

    /// A mode reference for a layer: names the id and overrides nothing.
    #[must_use]
    pub fn patch(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ..Self::default()
        }
    }

    /// The tabs this mode names, or an empty slice if unstated.
    #[must_use]
    pub fn tabs(&self) -> &[String] {
        self.tabs.as_deref().unwrap_or(&[])
    }
}

/// One ribbon tab.
///
/// `RIBBON_IA.md` §4 keeps an idiom worth preserving: every tab carries a
/// one-line **question** it exists to answer — *"What is on my screen, and
/// how is the page laid out?"* That is what [`Self::question`] is, and it
/// is not decoration: a tab whose question cannot be written in one line
/// is a tab carrying two unrelated jobs, which is the defect that split
/// six tabs into seven in that document.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Tab {
    /// Stable id, e.g. `"view"`. Never displayed.
    pub id: String,
    /// The operator-visible label. Required in a complete manifest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The one-line question this tab exists to answer. Optional; a
    /// renderer may show it as a hint, and a reviewer should read it as a
    /// test of whether the tab is coherent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub question: Option<String>,
    /// For a contextual tab: the condition under which it appears, in the
    /// language of [`crate::commands::ConditionSet`], e.g.
    /// `"selection.any"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible_when: Option<String>,
    /// Hidden tabs keep their definition and are not rendered.
    ///
    /// Hiding rather than deleting is what makes "unhide it again" a
    /// possible operation. An operator who deletes a tab from their
    /// customization file gets the built-in one back at the next merge,
    /// which is surprising; an operator who hides it gets what they asked
    /// for and can undo it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    /// The groups on this tab, in display order. Required in a complete
    /// manifest — a tab with no `groups` key at all is a layer's
    /// reference to a tab, not an empty tab.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<Group>>,
}

impl Tab {
    /// A tab with a label and no groups yet.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: Some(label.into()),
            groups: Some(Vec::new()),
            ..Self::default()
        }
    }

    /// A tab reference for a layer: names the id and overrides nothing.
    ///
    /// This is the whole vocabulary needed to reorder tabs — see
    /// [`merge`]'s ordering rule.
    #[must_use]
    pub fn patch(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ..Self::default()
        }
    }

    /// Set the groups.
    #[must_use]
    pub fn with_groups(mut self, groups: impl IntoIterator<Item = Group>) -> Self {
        self.groups = Some(groups.into_iter().collect());
        self
    }

    /// Set the one-line question.
    #[must_use]
    pub fn with_question(mut self, question: impl Into<String>) -> Self {
        self.question = Some(question.into());
        self
    }

    /// Set the visibility condition, making this a contextual tab.
    #[must_use]
    pub fn with_visible_when(mut self, condition: impl Into<String>) -> Self {
        self.visible_when = Some(condition.into());
        self
    }

    /// Hide or show.
    #[must_use]
    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = Some(hidden);
        self
    }

    /// Whether this tab is hidden. Unstated means visible.
    #[must_use]
    pub fn is_hidden(&self) -> bool {
        self.hidden.unwrap_or(false)
    }

    /// The groups, or an empty slice if unstated.
    #[must_use]
    pub fn groups(&self) -> &[Group] {
        self.groups.as_deref().unwrap_or(&[])
    }
}

/// A captioned band of items within a tab.
///
/// The caption is required in a complete manifest, and that is a rule
/// carried across from the salvage source, which enforced it with a
/// single closure through which every group had to be rendered. An
/// uncaptioned group is a row of controls whose relationship the operator
/// has to infer.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Group {
    /// Stable id, unique within its tab. Never displayed.
    pub id: String,
    /// The operator-visible caption. Required in a complete manifest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// The items in this group, in display order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<Item>>,
    /// **When this group gives up its rows**, as the band runs out of width.
    ///
    /// `None` — the default, and the meaning of an absent key — is *"this
    /// group never collapses"*. That is deliberately the safe answer for a
    /// manifest written before this field existed: such a manifest keeps
    /// exactly the behaviour it had, and the ladder simply has nothing to
    /// spend.
    ///
    /// `Some(n)` enters the group in the ladder at priority `n`, and **lower
    /// collapses first**. Ties break on manifest order, so two groups at the
    /// same priority collapse left to right, which is the order a reader would
    /// predict.
    ///
    /// # ★★ Why a manifest field and not a heuristic
    ///
    /// Because the right answer is editorial and cannot be measured. Word
    /// keeps **Clipboard** expanded at every width down to 460 pt while Font,
    /// Paragraph, Styles and Editing all collapse to single buttons — not
    /// because Clipboard is narrow (it is not) but because it carries the verb
    /// the operator came to the tab for. A width-based or item-count-based
    /// rule gets that exactly backwards, and no amount of tuning fixes a rule
    /// that is measuring the wrong property.
    ///
    /// ★ It is also the field that keeps `egui-shell` domain-free (R7). The
    /// shell cannot know which group matters on a PDF editor's Markup tab; the
    /// application says so in its manifest, in the same place it says
    /// everything else about its ribbon, and the shell just reads a number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collapse: Option<u32>,
    /// **Lay this group out on several rows even when one row would fit.**
    ///
    /// `None` — the default and the meaning of an absent key — is *"one row
    /// until the band runs out of width"*, which is what every group did before
    /// this field existed and what most groups should keep doing.
    ///
    /// # ★★★ Why a group would ask for rows it does not need
    ///
    /// Because wrapping under pressure and wrapping by design are different
    /// things, and the planner only ever did the first. `wrap_group` searches
    /// for the narrowest packing **once the group no longer fits**; a group that
    /// fits stays on one row however wide that row is.
    ///
    /// That is right for a Font group and wrong for a **radio**. Four square
    /// icon buttons in a row is a strip; the same four as a 2 x 2 block is half
    /// the width, reads as one control, and is what Acrobat, Word and every
    /// other ribbon does with a four-position choice. Operator, 2026-09-02:
    /// *"our display buttons should be on two rows to save space."*
    ///
    /// # ★★ It is a HINT, not a height
    ///
    /// The value is a ceiling the planner is asked to prefer, and the band's own
    /// row limit still wins: a group asking for four rows in a two-row band gets
    /// two. Nor does it force that many rows — the planner still returns the
    /// **narrowest** packing it can find, so a group of two items asking for two
    /// rows gets whichever of 1 x 2 and 2 x 1 is narrower.
    ///
    /// ★ And it does not stop the group re-wrapping further under pressure. A
    /// group that prefers two rows still goes to three on the collapse ladder,
    /// exactly as a group that reached two under pressure would.
    ///
    /// ★ R7: the shell reads a number. It has no idea which of an application's
    /// groups is a radio, and the manifest is where the application already says
    /// everything else about its ribbon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefer_rows: Option<u32>,
}

impl Group {
    /// A group with a caption and no items yet.
    #[must_use]
    pub fn new(id: impl Into<String>, caption: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            caption: Some(caption.into()),
            items: Some(Vec::new()),
            collapse: None,
            prefer_rows: None,
        }
    }

    /// The same group, entered into the collapse ladder at `priority`.
    ///
    /// Lower collapses first. A group that never calls this never collapses,
    /// which is the Clipboard case — see [`Group::collapse`].
    #[must_use]
    pub fn collapses_at(mut self, priority: u32) -> Self {
        self.collapse = Some(priority);
        self
    }

    /// A group reference for a layer: names the id and overrides nothing.
    #[must_use]
    pub fn patch(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ..Self::default()
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

    /// **Ask for this group to be laid out on `rows` rows even when one fits.**
    ///
    /// See [`Self::prefer_rows`] for what it does and does not promise. `rows`
    /// below 2 is stored as given and ignored by the planner, which is the
    /// honest handling: `1` means *"one row"*, which is already the default, and
    /// silently rewriting it to `None` would make a manifest that round-trips
    /// differently from the one that was written.
    #[must_use]
    pub fn with_prefer_rows(mut self, rows: u32) -> Self {
        self.prefer_rows = Some(rows);
        self
    }

    /// How many rows this group asks for, or `None` for the default.
    #[must_use]
    pub const fn preferred_rows(&self) -> Option<u32> {
        self.prefer_rows
    }
}

/// **How much room a control asks for, and how much of itself it shows.**
///
/// `RIBBON_SCALING.md` §5.1, learned by photographing Word at twelve widths.
/// Word has exactly three sizes and a group mixes them freely — one Large
/// button beside a column of three Small ones is its Clipboard group — and
/// that mixing is where its density comes from. Measured: at 884 client points
/// Word puts **ten** groups on the band and this shell put **three**, because
/// every control here was Medium and nothing could be narrower.
///
/// ★ [`Self::Medium`] is the default **and is exactly the presentation this
/// shell had before sizes existed**, so a manifest that says nothing renders
/// identically. That is what makes the vocabulary safe to introduce in one
/// change rather than behind a flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ItemSize {
    /// Icon, gap, label, on one row. The default, and every control's
    /// presentation before `RIBBON_SCALING.md`.
    #[default]
    Medium,
    /// **Icon only.**
    ///
    /// ★★ Earned, not asserted. A control renders icon-only only when it names
    /// an icon, carries a tooltip **and** a painter is installed — the rule
    /// [`crate::ribbon::qat`] already applies to the quick-access toolbar,
    /// applied here unchanged. The tooltip is the icon's accessible name;
    /// without one an icon-only button is an unlabelled rectangle to a screen
    /// reader and a guess to everybody else. A `Small` that has not earned it
    /// **falls back to `Medium`** rather than rendering a mystery.
    Small,
    /// **Icon above the label**, spanning the band's rows.
    ///
    /// The group's headline verb — Word's Paste, Dictate, Editor. Its width is
    /// the wider of its icon and its label, so a long label makes a wide
    /// button.
    Large,
}

impl ItemSize {
    /// Whether this is the default, for `skip_serializing_if`.
    ///
    /// Keeps `Command(id: "file.open")` in the manifest file rather than
    /// `Command(id: "file.open", size: Medium)` on every one of a hundred
    /// lines. The on-disk manifest is meant to be read and edited by an
    /// operator, and a field that is the default everywhere is noise that
    /// hides the two places it is not.
    #[must_use]
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Medium)
    }
}

/// One entry in a group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Item {
    /// A command, by id. The id is resolved against the registry; the manifest
    /// carries nothing else **about the command** — no label, no icon, no
    /// tooltip. What it does carry is how the command is *presented here*,
    /// which is a property of this position on this tab rather than of the
    /// command, and is therefore the manifest's to state.
    Command {
        /// The registered command id.
        id: String,
        /// How much room it asks for. See [`ItemSize`].
        ///
        /// ★ Ignored by menus, which have one row shape and no use for a
        /// size. `Item` is the shared vocabulary for ribbon groups and menus
        /// both; the alternative — a second item type for menus — would
        /// duplicate `visible_when`, which they genuinely do share.
        #[serde(default, skip_serializing_if = "ItemSize::is_default")]
        size: ItemSize,
        /// A condition name. When set, the item is drawn **only** while the
        /// condition holds — and when it is not, its space is reclaimed
        /// **before measurement**, so the group re-flows and a group with
        /// nothing left is not drawn at all.
        ///
        /// ★★★ This is visibility, not enablement, and the difference is R9:
        /// *an unavailable capability renders nothing; greying is reserved for
        /// **temporarily** unavailable and is always explained on hover.*
        /// [`crate::commands::Command::enable`] is the greying; this is the
        /// disappearing.
        ///
        /// It is what lets one tab definition serve Read, Review and Edit with
        /// different contents rather than three near-identical tabs —
        /// `RIBBON_SCALING.md` §5.3.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        visible_when: Option<String>,
    },
    /// A vertical rule between neighbours. Presentation only.
    Separator,
    /// Something the application draws itself.
    ///
    /// The extension point for controls that are not a button: a colour
    /// swatch, a zoom slider, a scale picker, a split button with a
    /// gallery. The shell reserves the space and hands `kind` and
    /// `payload` back; it draws nothing and interprets neither.
    ///
    /// This is what keeps the item vocabulary from growing a variant per
    /// widget an application happens to want — which is the road by which
    /// a reusable shell acquires a `ColourSwatch` variant and stops being
    /// reusable.
    Custom {
        /// An application-defined kind, e.g. `"colour_swatch"`.
        kind: String,
        /// Optional application-defined payload.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        payload: Option<String>,
        /// A condition name, with exactly the meaning it has on
        /// [`Self::Command`] — the item is drawn only while the condition
        /// holds, and its space is reclaimed **before measurement** when it
        /// does not.
        ///
        /// # ★★ Why this is a second copy of the field rather than a wrapper
        ///
        /// [`Self::visible_condition`] used to close with a standing
        /// instruction, and it is quoted rather than deleted because the
        /// reasoning is sound and the decision to depart from it has to be
        /// argued rather than assumed:
        ///
        /// > A separator and a custom item cannot carry one yet; when one
        /// > needs to, the field moves onto a **wrapper** rather than being
        /// > copied into three variants, because three copies of a rule is
        /// > three chances for it to drift.
        ///
        /// The need arrived on 2026-08-27: pdfcer's Format tab carries a Font
        /// group whose face chooser, size field and colour swatch are all
        /// custom items, and the whole group must be **absent** in a mode that
        /// cannot edit page content — R9's rule that an unavailable
        /// *capability* renders nothing while a temporarily unavailable one
        /// greys. Without this field, three of that group's seven controls
        /// would draw in Read mode and the application would have to fake
        /// their absence by drawing nothing into a slot the band had already
        /// reserved, which leaves a hole rather than reflowing the group.
        ///
        /// **What makes the copy safe is that the rule was never in the
        /// field.** It is in [`Self::visible_condition`] — one accessor, one
        /// match, read by exactly one predicate
        /// (`crate::ribbon::sizing::visible`). Two variants declaring a
        /// `visible_when` produce two serde attributes and two arms of that
        /// one match; they do not produce two statements of *when an item is
        /// drawn*. The drift the old note feared is drift in the **rule**, and
        /// the rule stayed single.
        ///
        /// **What would still justify the wrapper**, and this is the trigger
        /// to watch for: a *second* per-position property — an `enabled_when`,
        /// a `label_override`, an `order` — or a `Separator` that needs to
        /// disappear with its neighbours. At that point the fields stop being
        /// one field on two variants and become a *record*, and a record
        /// belongs beside the item rather than inside it.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        visible_when: Option<String>,
    },
}

impl Item {
    /// A command item, at the default size, always visible.
    ///
    /// The terse constructor, and the one nearly every call site wants: the
    /// manifest names a command and says nothing else about it.
    #[must_use]
    pub fn command(id: impl Into<String>) -> Self {
        Item::Command {
            id: id.into(),
            size: ItemSize::default(),
            visible_when: None,
        }
    }

    /// The same item, at a different size.
    ///
    /// A builder rather than a second constructor, so the common form above
    /// stays the short one and a sized item reads as *"this command, but
    /// large"* — which is what it is.
    ///
    /// ★ A separator and a custom item have no size to set, and this returns
    /// them untouched rather than panicking. A manifest is **data**, and the
    /// honest response to nonsense in data is that it does nothing, not that
    /// the application stops.
    #[must_use]
    pub fn sized(self, size: ItemSize) -> Self {
        match self {
            Item::Command {
                id, visible_when, ..
            } => Item::Command {
                id,
                size,
                visible_when,
            },
            other => other,
        }
    }

    /// The same item, drawn only while `condition` holds.
    #[must_use]
    pub fn shown_when(self, condition: impl Into<String>) -> Self {
        match self {
            Item::Command { id, size, .. } => Item::Command {
                id,
                size,
                visible_when: Some(condition.into()),
            },
            // ★ A custom item takes one too, since 2026-08-27. A separator
            // still does not and returns untouched, for the reason
            // [`Self::visible_condition`] gives: a divider's visibility is a
            // fact about its neighbours, not about itself.
            Item::Custom { kind, payload, .. } => Item::Custom {
                kind,
                payload,
                visible_when: Some(condition.into()),
            },
            other => other,
        }
    }

    /// A custom item with no payload.
    #[must_use]
    pub fn custom(kind: impl Into<String>) -> Self {
        Item::Custom {
            kind: kind.into(),
            payload: None,
            visible_when: None,
        }
    }

    /// The command id, if this item is a command.
    #[must_use]
    pub fn command_id(&self) -> Option<&str> {
        match self {
            Item::Command { id, .. } => Some(id),
            Item::Separator | Item::Custom { .. } => None,
        }
    }

    /// How much room this item asks for. A separator and a custom item have
    /// one presentation each and report the default.
    #[must_use]
    pub fn size(&self) -> ItemSize {
        match self {
            Item::Command { size, .. } => *size,
            Item::Separator | Item::Custom { .. } => ItemSize::default(),
        }
    }

    /// The condition this item is shown under, if any.
    ///
    /// ★ `None` means *always*, which is what the overwhelming majority of
    /// items are.
    ///
    /// ★★ **This function is where the rule lives, and that is what let the
    /// field be copied onto a second variant** on 2026-08-27. The note that
    /// used to sit here forbade the copy and named a wrapper as the remedy;
    /// [`Item::Custom`]'s `visible_when` carries the argument for departing
    /// from it, and the trigger that would still bring the wrapper back.
    ///
    /// A **separator** still cannot carry one, and deliberately: a rule for
    /// when a divider disappears is a rule about its *neighbours*, which is
    /// the record-shaped problem the wrapper exists for. A separator between
    /// two hidden items is a cosmetic defect; a separator with its own
    /// condition, set independently of the items it divides, is a
    /// contradiction that renders.
    #[must_use]
    pub fn visible_condition(&self) -> Option<&str> {
        match self {
            Item::Command { visible_when, .. } | Item::Custom { visible_when, .. } => {
                visible_when.as_deref()
            }
            Item::Separator => None,
        }
    }
}

/// The quick-access toolbar: command ids, in order.
///
/// `SHELL_FRAMEWORK.md` §5 amends the salvage source's one-command-one-tab
/// rule specifically to allow this: *"a command may appear on exactly one
/// **tab**; the QAT and status bar may mirror it."* A QAT that could not
/// mirror would be a second place to hunt for a command rather than a
/// shortcut to a known one.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Qat(pub Vec<String>);

impl Qat {
    /// The command ids, in order.
    #[must_use]
    pub fn ids(&self) -> &[String] {
        &self.0
    }
}

impl<S: Into<String>> FromIterator<S> for Qat {
    fn from_iter<I: IntoIterator<Item = S>>(iter: I) -> Self {
        Qat(iter.into_iter().map(Into::into).collect())
    }
}

/// The **trailing controls**: the far right of the tab-strip row, past the
/// mode selector.
///
/// ```text
/// [QAT…] │ File View Pages ⏷ 2 more   ( Read │ Review │ Edit )  [ … ]
///                                                                 ↑ here
/// ```
///
/// # ★★★ Why this exists at all, and why it is a manifest field rather than
/// a callback
///
/// It is the extension point for a control an application wants **beside the
/// mode selector** — the one region of the tab-strip row that has, until now,
/// had no way to hold anything. The immediate consumer is a button whose
/// *existence* is a property of the machine the program is running on rather
/// than of the program, and `SHELL_FRAMEWORK.md` §2's diagnostic applies with
/// full force: this crate must not learn what that button opens, and the
/// abstraction would be wrong if it had to.
///
/// The obvious cheaper spelling is a closure — *"hand the application a
/// rectangle beside the selector and let it draw"*. That is what
/// [`crate::ribbon::ctx::CustomItemRenderer`] does for [`Item::Custom`], and
/// it would work. It is rejected here because it would put a control on the
/// ribbon that the **command registry does not know about**, and
/// `SHELL_FRAMEWORK.md` §5b's one rule is that
///
/// > a capability's presence is expressed by registering its command, and by
/// > nothing else.
///
/// A raw callback is precisely the hole in that rule: an application could
/// draw a button for a capability that has no command, no enable predicate,
/// no keyboard binding and no id in any trace. Making the trailing region
/// carry [`Item`]s instead means every control in it is a registered command
/// with all of that machinery, reached through the same
/// [`crate::ribbon::ctx::Ctx::command`] lookup as every other control on the
/// ribbon.
///
/// # Why [`Item`] and not [`Qat`]'s bare id list
///
/// Because [`Item::Command::visible_when`] is exactly the mechanism R9 asks
/// for — *"an unavailable capability renders nothing; greying is reserved for
/// **temporarily** unavailable"* — and a bare id has nowhere to carry it. An
/// application whose trailing control depends on something being installed
/// needs the control to be **absent**, not disabled, and needs that decision
/// re-made every frame because the operator can install it, or point a
/// setting at it, without restarting the program.
///
/// A conditional *registration* cannot do that: the registry is built once,
/// at start-up, and [`Shell::validate_against`] hard-fails on a manifest
/// naming a command that is not in it. So the command is always registered —
/// which keeps validation strict and typos fatal — and the *item* carries the
/// condition.
///
/// # What it is not
///
/// Not a second QAT. The QAT is *"the handful of controls that must never sit
/// behind a tab switch"* — continuous-use verbs, on the left, where the eye
/// starts. This is the opposite end of the row and holds controls that leave
/// the program: it is the last thing read, and nothing in it should be
/// something the operator reaches for all day.
///
/// One command may appear here **and** on a tab, for the same reason
/// `SHELL_FRAMEWORK.md` §5 permits it for the QAT: a shortcut to a known home
/// is not a second place to hunt. [`Shell::validate`]'s one-command-one-tab
/// rule walks [`Shell::all_tabs`] only, so nothing here has to enforce it and
/// nothing here relaxes it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Trailing(pub Vec<Item>);

impl Trailing {
    /// The items, in display order — left to right, ending at the row's
    /// right edge.
    #[must_use]
    pub fn items(&self) -> &[Item] {
        &self.0
    }

    /// Whether there is nothing to draw.
    ///
    /// A present-but-empty `Trailing` is treated exactly as an absent one by
    /// the renderer, so that an operator customization that removed the last
    /// item reclaims the space instead of leaving a gap the width of nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl FromIterator<Item> for Trailing {
    fn from_iter<I: IntoIterator<Item = Item>>(iter: I) -> Self {
        Trailing(iter.into_iter().collect())
    }
}

/// Key chord → command id.
///
/// The chord is an opaque string here — `"Ctrl+E"`, `"F11"`. Parsing it
/// into modifiers and a key is the renderer's job, and doing it in this
/// type would mean a manifest could not be read by a tool that does not
/// link `egui`.
///
/// Ordered (`BTreeMap`) so a serialized manifest is byte-stable: an
/// operator's customization file that reordered itself on every save
/// would produce a diff on every run and make version control useless for
/// exactly the file most worth versioning.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Keymap(pub BTreeMap<String, String>);

impl Keymap {
    /// The command bound to a chord, if any.
    #[must_use]
    pub fn get(&self, chord: &str) -> Option<&str> {
        self.0.get(chord).map(String::as_str)
    }

    /// Every binding, in chord order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// How many chords are bound.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether nothing is bound.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A manifest in the shape of `SHELL_FRAMEWORK.md` §4's sketch,
    /// reduced to two tabs. Shared by the round-trip and validation
    /// tests so they cannot disagree about what a well-formed manifest
    /// looks like.
    pub(super) fn sketch() -> Shell {
        Shell::new()
            .with_mode(Mode::new("read", "Read", ["file", "view"]))
            .with_mode(Mode::new("edit", "Edit", ["file", "view"]))
            .with_tab(
                Tab::new("file", "File")
                    .with_question("What do I do with the file as a whole?")
                    .with_groups([Group::new("file", "File").with_items([
                        Item::command("file.open"),
                        Item::Separator,
                        Item::command("file.save_copy"),
                    ])]),
            )
            .with_tab(
                Tab::new("view", "View")
                    .with_question("What is on my screen?")
                    .with_groups([
                        Group::new("page_display", "Page display").with_items([
                            Item::command("view.single"),
                            Item::command("view.continuous"),
                        ]),
                        Group::new("window", "Window").with_items([
                            Item::command("view.fullscreen"),
                            Item::custom("zoom_slider"),
                        ]),
                    ]),
            )
            .with_contextual_tab(
                Tab::new("format", "Format")
                    .with_visible_when("selection.any")
                    .with_groups([
                        Group::new("style", "Style").with_items([Item::command("format.colour")])
                    ]),
            )
            .with_qat(["file.open", "file.save_copy"])
            .with_binding("Ctrl+E", "edit.text")
            .with_binding("F11", "view.fullscreen")
    }

    /// **A manifest survives a trip through RON unchanged.**
    ///
    /// The manifest's whole value proposition is that it is a file: an
    /// operator edits it, an application ships one, a workspace is saved
    /// as one, and `SHELL_FRAMEWORK.md` §6 lists "inspectable, diffable,
    /// testable without a GUI, and serializable" as what the design buys.
    /// Every one of those claims fails if the round trip is lossy.
    ///
    /// Both forms are checked. The compact form is what a save writes;
    /// the pretty form is what an operator opens, and a pretty printer
    /// that emits something its own parser rejects would be discovered by
    /// the operator rather than by CI.
    #[test]
    fn a_manifest_round_trips_through_ron() {
        let original = sketch();

        let compact = original.to_ron().expect("serializes");
        assert_eq!(
            Shell::from_ron(&compact).expect("compact form parses"),
            original,
            "the compact round trip lost or changed something"
        );

        let pretty = original.to_ron_pretty().expect("serializes");
        assert_eq!(
            Shell::from_ron(&pretty).expect("pretty form parses"),
            original,
            "the pretty round trip lost or changed something"
        );

        // The shapes the module header advertises must actually appear, or the
        // documented example is fiction.
        //
        // ★ `Command(id: "…")` since `ItemSize` landed. The spelling changed
        // once, deliberately, rather than growing a second variant so that the
        // old spelling could survive beside a new one: two ways to write one
        // item is two shapes for `merge` to reconcile and two for an operator
        // editing the file by hand to choose between. `built_in.ron` is
        // regenerated from the Rust manifest by a test, so the churn cost
        // nothing.
        assert!(compact.contains("Command(id:\"file.open\")"), "{compact}");
        assert!(pretty.contains("Separator"), "{pretty}");
        assert!(pretty.contains("\"Ctrl+E\""), "{pretty}");
        // ★★ And the default size is NOT written. The manifest file is meant
        // to be read and edited by an operator; `size: Medium` on every one of
        // a hundred lines is noise that hides the two lines where it is not.
        assert!(
            !compact.contains("size:"),
            "the default size must not be serialised: {compact}"
        );
    }

    /// **An unstated field stays unstated through a round trip.**
    ///
    /// This is the property the whole layered design rests on: `None`
    /// means "do not mention this" and must not come back as
    /// `Some(empty)`. If a layer's omitted `groups` round-tripped into
    /// `Some(vec![])`, saving and reloading an operator's customization
    /// would silently empty every tab it mentioned.
    #[test]
    fn an_unstated_field_stays_unstated_through_a_round_trip() {
        let layer = Shell::default().with_tab(Tab::patch("tools"));
        let text = layer.to_ron().expect("serializes");
        assert!(
            !text.contains("groups"),
            "an unstated field must not be written at all; got {text}"
        );
        let back = Shell::from_ron(&text).expect("parses");
        assert_eq!(back, layer);
        assert!(
            back.tabs()[0].groups.is_none(),
            "`None` must not resurrect as `Some(empty)` — that turns a \
             reference to a tab into an instruction to empty it"
        );
    }

    /// A hand-written operator file, with comments and a trailing comma,
    /// parses. This is the ergonomics claim RON was chosen for.
    #[test]
    fn a_hand_written_file_with_comments_parses() {
        let text = r#"
            Shell(
                // Move Tools to the front; leave everything else alone.
                tabs: [
                    Tab(id: "tools"),
                ],
                keymap: { "Ctrl+K": "tools.batch" },
            )
        "#;
        let shell = Shell::from_ron(text).expect("comments and trailing commas are allowed");
        assert_eq!(shell.tabs().len(), 1);
        assert_eq!(shell.tabs()[0].id, "tools");
        assert_eq!(
            shell.keymap.as_ref().and_then(|k| k.get("Ctrl+K")),
            Some("tools.batch")
        );
    }

    /// Accessors treat "unstated" as empty rather than panicking, so a
    /// layer can be read by the same code that reads a complete manifest.
    #[test]
    fn accessors_read_an_incomplete_layer_as_empty() {
        let layer = Shell::default();
        assert!(layer.tabs().is_empty());
        assert!(layer.modes().is_empty());
        assert!(layer.contextual_tabs().is_empty());
        assert_eq!(layer.all_tabs().count(), 0);
    }

    /// `Item::command_id` distinguishes the three variants, which is what
    /// every reference walk in `validate` and `merge` relies on.
    #[test]
    fn only_command_items_carry_a_command_id() {
        assert_eq!(Item::command("a.b").command_id(), Some("a.b"));
        assert_eq!(Item::Separator.command_id(), None);
        assert_eq!(Item::custom("swatch").command_id(), None);
    }
}
