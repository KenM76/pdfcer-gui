//! The three-layer merge — how a built-in ribbon becomes a customized
//! one without either side being able to break the other.
//!
//! # The three layers
//!
//! `SHELL_FRAMEWORK.md` §4:
//!
//! 1. **Built-in** — compiled into the binary. Always valid, always
//!    available as the reset target.
//! 2. **Application override** — optional file shipped beside the exe.
//! 3. **Operator customization** — `userdata/shell.ron`.
//!
//! > Later layers override earlier ones **per item**, not wholesale, which
//! > is the same per-key fail-soft contract `settings.txt` already uses. A
//! > customization referencing a command that no longer exists loses that
//! > one item and says so in the status surface; it does not discard the
//! > layout.
//!
//! # What "per item, not wholesale" actually means
//!
//! Two things, and both matter:
//!
//! **Structurally**, a layer that mentions one tab changes that tab and
//! nothing else. It does not have to restate the tabs it is happy with,
//! and it cannot delete them by omission. The same holds one level down
//! for groups. This is what makes an operator's customization file a
//! *diff* rather than a fork — and a fork is precisely what makes
//! customization unmaintainable, because the operator stops receiving
//! every improvement to the parts they never touched.
//!
//! **In failure**, the granularity of a problem is the item. A group
//! listing four commands, one of which no longer exists, contributes three
//! commands and one [`Skip`]. It does not contribute nothing, and it does
//! not fail the load.
//!
//! That second half is the one with teeth. The alternative — refusing a
//! customization file that references a stale command — means every
//! application update that renames or retires a command silently resets
//! the layout of every operator who had customized it, at exactly the
//! moment they are least expecting it and least able to attribute it.
//!
//! # Why a skip is disclosed rather than silent
//!
//! The salvage source learned this the expensive way, in a different
//! subsystem. Its diagnostic script parser skipped unparseable steps
//! *silently*, and on 2026-08-07 a misspelled step was dropped — the
//! resulting silence was read as a defect in the feature under test, and
//! was caught only by running a known-good sibling step and noticing the
//! difference. Its own note on the fix applies here word for word:
//!
//! > An absent trace line is indistinguishable from a step that ran and
//! > produced no output, so a typo presented as **a feature failing to
//! > respond** rather than as a step that never executed.
//!
//! A silently dropped ribbon item is the same failure with a slower
//! feedback loop: the operator sees a button missing and concludes the
//! application removed it. So every drop produces a [`Skip`] carrying the
//! layer, the site and the reason, and the application is expected to
//! surface them. [`MergeReport`] is a value the caller must deal with,
//! not a side effect it may forget.
//!
//! # Ordering falls out of the same rule
//!
//! When a layer supplies a list — tabs, groups, modes — the merged order
//! is **the ids that layer mentioned, in the order it mentioned them,
//! followed by everything it did not mention, in the order it already
//! had.**
//!
//! That gives reordering for free, in the same vocabulary, with no extra
//! field: an operator who wants Tools first writes
//!
//! ```ron
//! Shell(tabs: [ Tab(id: "tools") ])
//! ```
//!
//! — a tab reference that overrides nothing. A separate `tab_order` list
//! would have been a second place where tab identity is written down, and
//! therefore a second place for it to go stale.
//!
//! # What this module does not do
//!
//! It does not validate. The merged result still has to pass
//! [`Shell::validate_against`], and the division is deliberate:
//!
//! - **Merge is fail-soft** because its inputs are files from outside the
//!   build — hand-edited, or written by an older version.
//! - **Validation is strict** because its input is the merged whole, and
//!   what it catches are contradictions no fail-soft rule can repair. A
//!   customization that moves a command onto a second tab is not missing
//!   an item; it is asking for something incoherent, and the honest answer
//!   is to say so and fall back to the built-in layer.
//!
//! The built-in layer is deliberately **not** filtered against the
//! catalog. It is compiled in, it is the reset target, and an unknown
//! command in it is a programming error that should surface as a
//! validation failure in the application's own test suite — not be
//! quietly repaired at start-up, which would hide the bug on every machine
//! that runs it.
//!
//! ## ★★ The one exception, added 2026-09-06 — and it is an exception to
//! *what is filtered*, not to the rule above
//!
//! [`prune_absent_capabilities`] drops built-in items that name a
//! **capability** ([`super::Item::Command::capability`]) whose command is not
//! registered. `SHELL_FRAMEWORK.md` §5b calls this the *second, legitimate*
//! case: the built-in manifest names `file.sign`, the build was compiled
//! without signing, and that is not a bug — it is the configuration the
//! operator asked for on 2026-08-13, *"if not needed by someone they could
//! just remove them and they would not show up as options in the GUI."*
//!
//! The paragraph above still holds for every **mandatory** item. A built-in
//! item with no `capability` and no registered command is untouched here and
//! still reaches [`Shell::validate_against`] to fail loudly, which is why the
//! two kinds of absence are two [`SkipReason`] variants rather than one.

use super::validate::Site;
use super::{CommandCatalog, Group, Item, Keymap, Mode, Qat, Shell, Tab, Trailing};
use std::collections::BTreeMap;

/// Which layer an override came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Layer {
    /// Compiled into the binary. The reset target.
    BuiltIn,
    /// A file shipped beside the executable.
    AppOverride,
    /// The operator's own customization.
    Operator,
}

impl std::fmt::Display for Layer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Layer::BuiltIn => "the built-in manifest",
            Layer::AppOverride => "the application override",
            Layer::Operator => "your customization",
        })
    }
}

/// Why one item was dropped during a merge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// The item referenced a command that is not registered.
    UnknownCommand {
        /// The id that did not resolve.
        command: String,
    },
    /// **The item was conditional on a capability this build does not have.**
    ///
    /// ★★★ A DIFFERENT REASON FROM [`Self::UnknownCommand`], AND THE
    /// DIFFERENCE IS THE WHOLE POINT — `SHELL_FRAMEWORK.md` §5b:
    ///
    /// > `CapabilityAbsent` is a *different* reason from `UnknownCommand`
    /// > precisely so the two never get confused in a log: one says "this
    /// > build does not include that", the other says "someone made a
    /// > mistake".
    ///
    /// Raised when an [`Item::Command`] carrying a
    /// [`capability`](super::Item::Command::capability) names a command the
    /// registry does not hold — a build compiled without an optional
    /// capability, behaving exactly as the operator's 2026-08-13 directive
    /// asks. It is the one reason also raised against the **built-in** layer;
    /// [`prune_absent_capabilities`] carries why that is not a weakening of the
    /// rule beside it.
    CapabilityAbsent {
        /// The capability the item named. Carried so the report can say what
        /// is missing; the shell never matches it against anything.
        capability: String,
        /// The command that is not registered.
        command: String,
    },
    /// A mode named a tab that does not exist after merging.
    UnknownTab {
        /// The tab id that did not resolve.
        tab: String,
    },
    /// The whole layer declared a schema this build does not understand,
    /// so none of it was applied.
    UnsupportedSchema {
        /// The schema the layer declared.
        found: u32,
        /// The newest schema this build supports.
        supported: u32,
    },
}

/// One thing the merge could not carry across.
///
/// A structured value rather than a message, deliberately. The shell has
/// no business deciding how another application words a note to its
/// operator, and an application that wants to offer "remove this stale
/// entry from your file" needs the id, not a sentence containing it.
///
/// [`std::fmt::Display`] is provided for diagnostics — a log line, a
/// failing test, `tools/ui-verify` — and is **not** operator-visible copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skip {
    /// Which layer the dropped item came from.
    pub layer: Layer,
    /// Where in the manifest it was.
    pub site: Site,
    /// Why it was dropped.
    pub reason: SkipReason,
}

impl std::fmt::Display for Skip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.reason {
            SkipReason::UnknownCommand { command } => write!(
                f,
                "{}: `{command}` in {} is not a registered command, so that one item \
                 was skipped",
                self.layer, self.site
            ),
            SkipReason::CapabilityAbsent {
                capability,
                command,
            } => write!(
                f,
                "{}: `{command}` in {} is provided by the `{capability}` capability, which this \
                 build does not include, so that one item was left out",
                self.layer, self.site
            ),
            SkipReason::UnknownTab { tab } => write!(
                f,
                "{}: {} names tab `{tab}`, which does not exist, so that one entry \
                 was skipped",
                self.layer, self.site
            ),
            SkipReason::UnsupportedSchema { found, supported } => write!(
                f,
                "{}: schema {found} is newer than this build understands ({supported}), \
                 so that layer was not applied",
                self.layer
            ),
        }
    }
}

/// Everything a merge had to skip.
///
/// Returned by value so the caller must deal with it. The salvage source
/// makes the argument for that shape explicitly: returning the rejects
/// alongside the result makes them *a value the caller must handle*
/// instead of a side effect it may forget.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MergeReport {
    skips: Vec<Skip>,
}

impl MergeReport {
    /// Every skip, in the order it occurred.
    #[must_use]
    pub fn skips(&self) -> &[Skip] {
        &self.skips
    }

    /// Whether the merge carried everything across.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.skips.is_empty()
    }

    /// How many items were skipped.
    #[must_use]
    pub fn len(&self) -> usize {
        self.skips.len()
    }

    fn push(&mut self, layer: Layer, site: Site, reason: SkipReason) {
        self.skips.push(Skip {
            layer,
            site,
            reason,
        });
    }
}

/// The three layers to merge.
///
/// A struct rather than three positional arguments because two of them
/// are `Option<&Shell>` of the same type, and a call site that swapped
/// them would compile and would apply the operator's customization before
/// the application's override — producing a shell that is wrong in a way
/// no test of either file could find.
#[derive(Debug, Clone, Copy)]
pub struct MergeInput<'a> {
    /// Compiled into the binary. Required, and the reset target.
    pub built_in: &'a Shell,
    /// A file shipped beside the executable, if present.
    pub app_override: Option<&'a Shell>,
    /// The operator's customization, if present.
    pub operator: Option<&'a Shell>,
}

impl<'a> MergeInput<'a> {
    /// Just the built-in layer.
    #[must_use]
    pub fn built_in(shell: &'a Shell) -> Self {
        Self {
            built_in: shell,
            app_override: None,
            operator: None,
        }
    }

    /// With an application override.
    #[must_use]
    pub fn with_app_override(mut self, shell: &'a Shell) -> Self {
        self.app_override = Some(shell);
        self
    }

    /// With an operator customization.
    #[must_use]
    pub fn with_operator(mut self, shell: &'a Shell) -> Self {
        self.operator = Some(shell);
        self
    }
}

/// The result of a merge: the shell, and everything that was skipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Merged {
    /// The merged manifest. Still has to pass
    /// [`Shell::validate_against`] — see this module's header.
    pub shell: Shell,
    /// What could not be carried across.
    pub report: MergeReport,
}

/// Merge the three layers.
///
/// Never fails. Anything it cannot carry across becomes a [`Skip`] in
/// [`Merged::report`]; see this module's header for why that is the right
/// posture for inputs that come from outside the build.
///
/// The `catalog` is what makes a stale command id detectable. Pass
/// [`super::AnyCommand`] only in tooling that has no registry — in an
/// application it would disable the check that turns a stale reference
/// into a disclosed skip instead of a control that does nothing.
#[must_use]
pub fn merge(input: MergeInput<'_>, catalog: &dyn CommandCatalog) -> Merged {
    let mut shell = input.built_in.clone();
    shell.schema = Shell::SCHEMA;
    let mut report = MergeReport::default();

    // Which layer last supplied each mode's tab list. Needed because
    // pruning a mode's stale tab references can only happen after every
    // layer has had its chance to ADD the tab, by which point the layer
    // that wrote the reference is no longer on the stack.
    let mut mode_source: BTreeMap<String, Layer> = BTreeMap::new();

    for (layer, overlay) in [
        (Layer::AppOverride, input.app_override),
        (Layer::Operator, input.operator),
    ] {
        let Some(overlay) = overlay else { continue };
        apply(
            &mut shell,
            overlay,
            layer,
            catalog,
            &mut report,
            &mut mode_source,
        );
    }

    // ★ BEFORE `prune_mode_tabs`, and the order is load-bearing. Dropping a
    // capability's items can empty a group and empty a tab; a mode still
    // naming that tab must then lose the reference, and only the pass below
    // does that. Running them the other way round leaves a mode pointing at a
    // tab that has nothing in it.
    prune_absent_capabilities(&mut shell, catalog, &mut report);
    prune_mode_tabs(&mut shell, &mode_source, &mut report);

    Merged { shell, report }
}

/// **Drop the conditional items whose capability this build does not have.**
///
/// `SHELL_FRAMEWORK.md` §5b's *"gap that must be closed"*, closed 2026-09-06
/// by the work that wired document signing — a capability the operator asked
/// for, which `pdfcer-core` gates behind a Cargo feature, and which therefore
/// had to be able to be **absent** without a single `#[cfg]` in a ribbon.
///
/// # ★★★ Why this runs over the whole merged shell, built-in layer included
///
/// It is the one deliberate exception to this module's standing rule that the
/// built-in layer is not filtered. That rule's reason is sound and unchanged:
/// an unknown command compiled into the binary is a programming error, and
/// quietly repairing it at start-up hides the bug on every machine that runs
/// it. But §5b names a **second, legitimate** case — the built-in manifest
/// names `file.sign`, signing is compiled out, and that is not a bug, it is
/// the intended configuration. Under the old rule that was either a validation
/// failure that blocks start-up or a live command with no handler.
///
/// The fix is not to weaken the strict rule but to make the two cases
/// distinguishable, which is exactly what
/// [`Item::Command::capability`](super::Item::Command::capability) does. A
/// mandatory item is still untouched here and still reaches
/// [`super::Shell::validate_against`] to fail loudly.
///
/// # What it reports, and why every skip says `BuiltIn`
///
/// Anything this pass finds came from the built-in layer **by construction**:
/// an overlay's items were already filtered by [`filter_group_items`] and
/// [`filter_items`] as that layer was applied, and those use the same
/// [`absence_reason`], so a conditional overlay item naming an absent command
/// was dropped then — with its own layer on the skip. What survives to here is
/// what `built_in` supplied.
///
/// # Empty groups and empty tabs are left where they are
///
/// §5b's *"a group left with no items does not render"* is a **rendering**
/// rule, already honoured by the renderer. Deleting the group here would also
/// destroy the operator's saved customization of it, which would come back
/// blank the day they installed a build that has the capability again. An empty
/// group is invisible; a deleted one is forgotten.
fn prune_absent_capabilities(
    shell: &mut Shell,
    catalog: &dyn CommandCatalog,
    report: &mut MergeReport,
) {
    // ★★★ CONDITIONAL ITEMS ONLY. `filter_group_items` is deliberately NOT
    // reused here even though it is one line shorter, because it would also
    // drop a MANDATORY item whose command is missing — silently repairing, in
    // the built-in layer, exactly the programming error this module's header
    // says must reach `validate_against` and fail loudly. The two filters
    // share `absence_reason`; only this one narrows what it acts on.
    let mut retain_conditional = |items: &mut Vec<Item>, site: &Site| {
        items.retain(|item| match absence_reason(item, catalog) {
            Some(reason @ SkipReason::CapabilityAbsent { .. }) => {
                report.push(Layer::BuiltIn, site.clone(), reason);
                false
            }
            _ => true,
        });
    };

    for tabs in [shell.tabs.as_mut(), shell.contextual_tabs.as_mut()] {
        for tab in tabs.into_iter().flatten() {
            let tab_id = tab.id.clone();
            for group in tab.groups.iter_mut().flatten() {
                let site = Site::Group {
                    tab: tab_id.clone(),
                    group: group.id.clone(),
                };
                if let Some(items) = group.items.as_mut() {
                    retain_conditional(items, &site);
                }
            }
        }
    }

    if let Some(trailing) = shell.trailing.as_mut() {
        retain_conditional(&mut trailing.0, &Site::Trailing);
    }
}

/// Apply one overlay onto the accumulating shell.
fn apply(
    base: &mut Shell,
    overlay: &Shell,
    layer: Layer,
    catalog: &dyn CommandCatalog,
    report: &mut MergeReport,
    mode_source: &mut BTreeMap<String, Layer>,
) {
    // A layer from a newer build is skipped WHOLE rather than
    // field-by-field. A field this build does not understand may be the
    // one that changes what the fields it does understand mean, and
    // applying half of a document is how a fail-soft loader produces a
    // result nobody wrote.
    if overlay.schema > Shell::SCHEMA {
        report.push(
            layer,
            Site::Document,
            SkipReason::UnsupportedSchema {
                found: overlay.schema,
                supported: Shell::SCHEMA,
            },
        );
        return;
    }

    if let Some(tabs) = &overlay.tabs {
        merge_tabs(&mut base.tabs, tabs, layer, catalog, report);
    }
    if let Some(tabs) = &overlay.contextual_tabs {
        merge_tabs(&mut base.contextual_tabs, tabs, layer, catalog, report);
    }
    if let Some(modes) = &overlay.modes {
        merge_modes(&mut base.modes, modes, layer, mode_source);
    }
    if let Some(qat) = &overlay.qat {
        base.qat = Some(Qat(filter_ids(
            qat.ids(),
            layer,
            &Site::Qat,
            catalog,
            report,
        )));
    }
    if let Some(trailing) = &overlay.trailing {
        // ★ Replaced whole, exactly as the QAT is, and for the same reason:
        // both are short ORDERED lists whose whole content is their order. A
        // per-item merge would have to answer "what does it mean to override
        // item 2?" and every answer to that is worse than "the layer that
        // mentions the region owns it" — which is the rule an operator can
        // hold in their head while editing the file by hand.
        base.trailing = Some(filter_items(
            trailing.items(),
            layer,
            &Site::Trailing,
            catalog,
            report,
        ));
    }
    if let Some(keymap) = &overlay.keymap {
        merge_keymap(&mut base.keymap, keymap, layer, catalog, report);
    }
}

/// Filter a flat list of items (the trailing region), disclosing each drop.
///
/// The item twin of [`filter_ids`]. Kept separate rather than generalised
/// because the two carry different payloads — a bare id and a whole `Item` —
/// and a generic over "things that might contain a command id" would be more
/// machinery than the six lines it replaced.
fn filter_items(
    items: &[Item],
    layer: Layer,
    site: &Site,
    catalog: &dyn CommandCatalog,
    report: &mut MergeReport,
) -> Trailing {
    items
        .iter()
        .filter(|item| match absence_reason(item, catalog) {
            Some(reason) => {
                report.push(layer, site.clone(), reason);
                false
            }
            None => true,
        })
        .cloned()
        .collect()
}

/// **Why this item cannot be carried across, if it cannot.**
///
/// The one place the two absence reasons are told apart, so they cannot drift:
/// every filter in this module routes through it, and adding a third caller
/// means adding a call rather than re-deriving the distinction.
///
/// `None` for anything that stays — a separator, a custom item, and a command
/// the registry holds.
///
/// | the item | the command | reason |
/// |---|---|---|
/// | mandatory (no `capability`) | registered | `None` — it stays |
/// | mandatory | **not** registered | [`SkipReason::UnknownCommand`] — someone made a mistake |
/// | conditional (`capability: Some`) | registered | `None` — it stays; the build has the capability |
/// | conditional | **not** registered | [`SkipReason::CapabilityAbsent`] — this build is smaller, on purpose |
///
/// ★ Note the third row: a conditional item whose command **is** registered is
/// indistinguishable at render time from a mandatory one, and must be. The
/// field says how to read the item's absence, never how to draw its presence.
fn absence_reason(item: &Item, catalog: &dyn CommandCatalog) -> Option<SkipReason> {
    let Item::Command { id, capability, .. } = item else {
        return None;
    };
    if catalog.contains(id) {
        return None;
    }
    Some(match capability {
        Some(capability) => SkipReason::CapabilityAbsent {
            capability: capability.clone(),
            command: id.clone(),
        },
        None => SkipReason::UnknownCommand {
            command: id.clone(),
        },
    })
}

/// Merge a tab list: mentioned ids first in overlay order, then the rest.
///
/// See the module header, "Ordering falls out of the same rule".
fn merge_tabs(
    base: &mut Option<Vec<Tab>>,
    overlay: &[Tab],
    layer: Layer,
    catalog: &dyn CommandCatalog,
    report: &mut MergeReport,
) {
    let mut remaining = base.take().unwrap_or_default();
    let mut out: Vec<Tab> = Vec::with_capacity(remaining.len() + overlay.len());

    for otab in overlay {
        match remaining.iter().position(|t| t.id == otab.id) {
            Some(pos) => {
                let mut tab = remaining.remove(pos);
                merge_tab(&mut tab, otab, layer, catalog, report);
                out.push(tab);
            }
            None => {
                // A tab the base did not have: taken whole, but its items
                // are still filtered, because a NEW tab from an operator's
                // file is exactly as likely to name a stale command as an
                // edit to an existing one.
                let mut tab = otab.clone();
                if let Some(groups) = tab.groups.as_mut() {
                    for group in groups.iter_mut() {
                        filter_group_items(group, &tab.id, layer, catalog, report);
                    }
                }
                out.push(tab);
            }
        }
    }

    out.extend(remaining);
    *base = Some(out);
}

/// Override one tab's stated fields, leaving unstated ones alone.
fn merge_tab(
    base: &mut Tab,
    overlay: &Tab,
    layer: Layer,
    catalog: &dyn CommandCatalog,
    report: &mut MergeReport,
) {
    if overlay.label.is_some() {
        base.label.clone_from(&overlay.label);
    }
    if overlay.question.is_some() {
        base.question.clone_from(&overlay.question);
    }
    if overlay.visible_when.is_some() {
        base.visible_when.clone_from(&overlay.visible_when);
    }
    if overlay.hidden.is_some() {
        base.hidden = overlay.hidden;
    }
    if let Some(groups) = &overlay.groups {
        let tab_id = base.id.clone();
        merge_groups(&mut base.groups, groups, &tab_id, layer, catalog, report);
    }
}

/// Merge a group list, by the same rule as [`merge_tabs`].
fn merge_groups(
    base: &mut Option<Vec<Group>>,
    overlay: &[Group],
    tab_id: &str,
    layer: Layer,
    catalog: &dyn CommandCatalog,
    report: &mut MergeReport,
) {
    let mut remaining = base.take().unwrap_or_default();
    let mut out: Vec<Group> = Vec::with_capacity(remaining.len() + overlay.len());

    for ogroup in overlay {
        match remaining.iter().position(|g| g.id == ogroup.id) {
            Some(pos) => {
                let mut group = remaining.remove(pos);
                if ogroup.caption.is_some() {
                    group.caption.clone_from(&ogroup.caption);
                }
                if ogroup.items.is_some() {
                    // Items are REPLACED, not merged element-wise: an item
                    // has no id, so there is nothing to match on. This is
                    // the level at which "wholesale" is the only coherent
                    // rule, and it is also where the per-item FAILURE
                    // granularity does its work — one stale command costs
                    // one item, not the group.
                    group.items.clone_from(&ogroup.items);
                    filter_group_items(&mut group, tab_id, layer, catalog, report);
                }
                out.push(group);
            }
            None => {
                let mut group = ogroup.clone();
                filter_group_items(&mut group, tab_id, layer, catalog, report);
                out.push(group);
            }
        }
    }

    out.extend(remaining);
    *base = Some(out);
}

/// Drop items naming commands the catalog does not know, disclosing each.
fn filter_group_items(
    group: &mut Group,
    tab_id: &str,
    layer: Layer,
    catalog: &dyn CommandCatalog,
    report: &mut MergeReport,
) {
    let Some(items) = group.items.as_mut() else {
        return;
    };
    let site = Site::Group {
        tab: tab_id.to_owned(),
        group: group.id.clone(),
    };
    items.retain(|item| match absence_reason(item, catalog) {
        Some(reason) => {
            report.push(layer, site.clone(), reason);
            false
        }
        None => true,
    });
}

/// Filter a flat list of command ids (the QAT), disclosing each drop.
fn filter_ids(
    ids: &[String],
    layer: Layer,
    site: &Site,
    catalog: &dyn CommandCatalog,
    report: &mut MergeReport,
) -> Vec<String> {
    ids.iter()
        .filter(|id| {
            if catalog.contains(id) {
                true
            } else {
                report.push(
                    layer,
                    site.clone(),
                    SkipReason::UnknownCommand {
                        command: (*id).clone(),
                    },
                );
                false
            }
        })
        .cloned()
        .collect()
}

/// Merge a mode list, by the same rule as [`merge_tabs`].
fn merge_modes(
    base: &mut Option<Vec<Mode>>,
    overlay: &[Mode],
    layer: Layer,
    mode_source: &mut BTreeMap<String, Layer>,
) {
    let mut remaining = base.take().unwrap_or_default();
    let mut out: Vec<Mode> = Vec::with_capacity(remaining.len() + overlay.len());

    for omode in overlay {
        let mut mode = match remaining.iter().position(|m| m.id == omode.id) {
            Some(pos) => remaining.remove(pos),
            None => Mode {
                id: omode.id.clone(),
                ..Mode::default()
            },
        };
        if omode.label.is_some() {
            mode.label.clone_from(&omode.label);
        }
        if omode.tabs.is_some() {
            mode.tabs.clone_from(&omode.tabs);
            mode_source.insert(mode.id.clone(), layer);
        }
        out.push(mode);
    }

    out.extend(remaining);
    *base = Some(out);
}

/// Merge a keymap per chord.
///
/// An **empty command id unbinds** the chord. That is the only way a
/// later layer can express "remove this binding" in a per-key merge, and
/// without it an operator could rebind every chord but never free one —
/// which matters because a chord an application binds by default may be
/// one the operator's screen reader or window manager wants.
fn merge_keymap(
    base: &mut Option<Keymap>,
    overlay: &Keymap,
    layer: Layer,
    catalog: &dyn CommandCatalog,
    report: &mut MergeReport,
) {
    let map = base.get_or_insert_with(Keymap::default);
    for (chord, command) in overlay.iter() {
        if command.is_empty() {
            map.0.remove(chord);
        } else if catalog.contains(command) {
            map.0.insert(chord.to_owned(), command.to_owned());
        } else {
            report.push(
                layer,
                Site::Keymap {
                    chord: chord.to_owned(),
                },
                SkipReason::UnknownCommand {
                    command: command.to_owned(),
                },
            );
        }
    }
}

/// Drop mode entries naming tabs that do not exist after merging.
///
/// Runs last, once, for the reason recorded on `mode_source` in [`merge`]:
/// a mode may legitimately name a tab that a *later* layer introduces, so
/// checking as each layer is applied would reject correct files.
///
/// A stale reference here is disclosed and dropped rather than rejected —
/// consistent with every other merge failure, and specifically so that an
/// operator's mode survives the removal of a tab minus one entry rather
/// than failing to load. [`Shell::validate`] would reject what survives
/// only if this had left something unresolvable, which by construction it
/// cannot.
fn prune_mode_tabs(
    shell: &mut Shell,
    mode_source: &BTreeMap<String, Layer>,
    report: &mut MergeReport,
) {
    let known: Vec<String> = shell.tabs().iter().map(|t| t.id.clone()).collect();
    let Some(modes) = shell.modes.as_mut() else {
        return;
    };
    for mode in modes.iter_mut() {
        let layer = mode_source.get(&mode.id).copied().unwrap_or(Layer::BuiltIn);
        let mode_id = mode.id.clone();
        if let Some(tabs) = mode.tabs.as_mut() {
            tabs.retain(|tab| {
                if known.iter().any(|k| k == tab) {
                    true
                } else {
                    report.push(
                        layer,
                        Site::Mode {
                            mode: mode_id.clone(),
                        },
                        SkipReason::UnknownTab { tab: tab.clone() },
                    );
                    false
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::AnyCommand;

    /// A catalog holding a fixed set of ids.
    struct Known(&'static [&'static str]);
    impl CommandCatalog for Known {
        fn contains(&self, id: &str) -> bool {
            self.0.contains(&id)
        }
    }

    const CATALOG: Known = Known(&[
        "file.open",
        "file.save_copy",
        "view.single",
        "view.continuous",
        "view.fullscreen",
        "tools.batch",
        "edit.text",
    ]);

    /// A three-tab built-in manifest, complete and valid.
    fn built_in() -> Shell {
        Shell::new()
            .with_tab(
                Tab::new("file", "File").with_groups([
                    Group::new("file", "File").with_items([Item::command("file.open")])
                ]),
            )
            .with_tab(
                Tab::new("view", "View")
                    .with_question("What is on my screen?")
                    .with_groups([
                        Group::new("page_display", "Page display").with_items([
                            Item::command("view.single"),
                            Item::command("view.continuous"),
                        ]),
                        Group::new("window", "Window")
                            .with_items([Item::command("view.fullscreen")]),
                    ]),
            )
            .with_tab(Tab::new("tools", "Tools").with_groups([
                Group::new("batch", "Batch").with_items([Item::command("tools.batch")]),
            ]))
            .with_mode(Mode::new("read", "Read", ["file", "view"]))
            .with_qat(["file.open"])
            .with_binding("F11", "view.fullscreen")
    }

    /// The fixture is valid, so every negative test below means
    /// something.
    #[test]
    fn the_built_in_fixture_is_valid() {
        built_in()
            .validate_against(&CATALOG)
            .expect("the fixture must be valid or every test here is vacuous");
    }

    /// **★ A layer overrides per item: what it does not mention survives
    /// untouched.**
    ///
    /// This is the contract in one test. An operator layer that renames
    /// one tab must not delete the two it did not mention, must not empty
    /// the groups it did not mention on the tab it *did*, and must not
    /// drop the question or the keymap it said nothing about.
    ///
    /// The failure this guards against is not hypothetical: a
    /// `tabs: Vec<Tab>` with no `Option` makes "I did not mention it"
    /// indistinguishable from "I want it gone", and every field on the
    /// tab has the same problem one level down.
    #[test]
    fn a_layer_overrides_per_item_and_leaves_everything_else_alone() {
        let base = built_in();
        let operator = Shell::default().with_tab(Tab {
            id: "view".to_owned(),
            label: Some("Display".to_owned()),
            ..Tab::default()
        });

        let merged = merge(
            MergeInput::built_in(&base).with_operator(&operator),
            &CATALOG,
        );
        assert!(merged.report.is_empty(), "{:?}", merged.report);

        let shell = merged.shell;
        assert_eq!(
            shell
                .tabs()
                .iter()
                .map(|t| t.id.as_str())
                .collect::<Vec<_>>(),
            ["view", "file", "tools"],
            "the mentioned tab moves to the front; the others survive in order"
        );

        let view = &shell.tabs()[0];
        assert_eq!(
            view.label.as_deref(),
            Some("Display"),
            "the label overrides"
        );
        assert_eq!(
            view.question.as_deref(),
            Some("What is on my screen?"),
            "an unmentioned field must survive"
        );
        assert_eq!(
            view.groups().len(),
            2,
            "an unmentioned `groups` must survive whole; got {:?}",
            view.groups()
        );
        assert_eq!(
            shell.keymap.as_ref().and_then(|k| k.get("F11")),
            Some("view.fullscreen"),
            "an unmentioned keymap must survive"
        );
        shell.validate_against(&CATALOG).expect("still valid");
    }

    /// **A bare tab reference reorders, and nothing else.**
    ///
    /// The documented idiom from this module's header. If this ever
    /// stopped working, the alternative would be a `tab_order` field —
    /// a second place tab identity is written down and a second place for
    /// it to go stale.
    #[test]
    fn a_bare_tab_reference_reorders_without_changing_anything() {
        let base = built_in();
        let operator = Shell::default().with_tab(Tab::patch("tools"));
        let merged = merge(
            MergeInput::built_in(&base).with_operator(&operator),
            &CATALOG,
        );
        assert_eq!(
            merged
                .shell
                .tabs()
                .iter()
                .map(|t| t.id.as_str())
                .collect::<Vec<_>>(),
            ["tools", "file", "view"]
        );
        assert_eq!(
            merged.shell.tabs()[0].label.as_deref(),
            Some("Tools"),
            "a reference must not blank the label it did not state"
        );
        assert_eq!(merged.shell.tabs()[0].groups().len(), 1);
    }

    /// **★ An item naming a command that no longer exists is a disclosed
    /// skip, not an error — and the rest of the group survives.**
    ///
    /// `SHELL_FRAMEWORK.md` §4: *"A customization referencing a command
    /// that no longer exists loses that one item and says so in the status
    /// surface; it does not discard the layout."*
    ///
    /// Both halves are asserted, and the second is the one that would
    /// otherwise rot: it is easy to write a filter that drops the group,
    /// and the symptom — an operator's Window group quietly emptying after
    /// an update — is attributed to the update, not to the filter.
    #[test]
    fn an_unknown_command_loses_one_item_and_is_disclosed() {
        let base = built_in();
        let operator = Shell::default().with_tab(Tab::patch("view").with_groups([
            Group::patch("window").with_items([
                Item::command("view.fullscreen"),
                Item::command("view.read_mode"), // retired since they wrote this
                Item::Separator,
                Item::command("edit.text"),
            ]),
        ]));

        let merged = merge(
            MergeInput::built_in(&base).with_operator(&operator),
            &CATALOG,
        );

        let window = merged.shell.tabs()[0]
            .groups()
            .iter()
            .find(|g| g.id == "window")
            .expect("the group survives");
        assert_eq!(
            window.items(),
            [
                Item::command("view.fullscreen"),
                Item::Separator,
                Item::command("edit.text"),
            ],
            "one item is lost, not the group"
        );
        assert_eq!(
            window.caption.as_deref(),
            Some("Window"),
            "a group reference must not blank the caption it did not state"
        );

        assert_eq!(merged.report.len(), 1, "{:?}", merged.report);
        let skip = &merged.report.skips()[0];
        assert_eq!(skip.layer, Layer::Operator);
        assert_eq!(
            skip.reason,
            SkipReason::UnknownCommand {
                command: "view.read_mode".to_owned()
            }
        );
        assert_eq!(
            skip.site,
            Site::Group {
                tab: "view".to_owned(),
                group: "window".to_owned()
            }
        );
        // The disclosure must be able to name the id, the place and the
        // layer — that is what makes it actionable rather than a shrug.
        let text = skip.to_string();
        for needle in ["view.read_mode", "window", "customization"] {
            assert!(text.contains(needle), "{text}");
        }

        merged
            .shell
            .validate_against(&CATALOG)
            .expect("a merged shell with skips is still valid");
    }

    /// A stale id in the QAT and in the keymap is skipped the same way.
    #[test]
    fn stale_qat_and_keymap_entries_are_skipped_and_named() {
        let base = built_in();
        let operator = Shell::default()
            .with_qat(["file.open", "file.bates"])
            .with_binding("Ctrl+B", "file.bates")
            .with_binding("Ctrl+E", "edit.text");

        let merged = merge(
            MergeInput::built_in(&base).with_operator(&operator),
            &CATALOG,
        );

        assert_eq!(
            merged.shell.qat.as_ref().map(Qat::ids),
            Some(&["file.open".to_owned()][..])
        );
        assert_eq!(
            merged.shell.keymap.as_ref().and_then(|k| k.get("Ctrl+E")),
            Some("edit.text")
        );
        assert!(
            merged
                .shell
                .keymap
                .as_ref()
                .is_some_and(|k| k.get("Ctrl+B").is_none()),
            "a chord bound to a stale command must not be bound at all"
        );
        assert_eq!(merged.report.len(), 2, "{:?}", merged.report);
        assert!(merged.report.skips().iter().any(|s| s.site == Site::Qat));
        assert!(merged.report.skips().iter().any(|s| s.site
            == Site::Keymap {
                chord: "Ctrl+B".to_owned()
            }));
    }

    /// An empty command id unbinds a chord — the only per-key way to
    /// express removal.
    #[test]
    fn an_empty_binding_unbinds_the_chord() {
        let base = built_in();
        let operator = Shell::default().with_binding("F11", "");
        let merged = merge(
            MergeInput::built_in(&base).with_operator(&operator),
            &CATALOG,
        );
        assert!(
            merged
                .shell
                .keymap
                .as_ref()
                .is_some_and(|k| k.get("F11").is_none()),
            "the operator must be able to free a chord, not only rebind it"
        );
        assert!(merged.report.is_empty());
    }

    /// **All three layers apply, in order, and each still overrides per
    /// item.**
    ///
    /// The middle layer's contribution must survive the outer one saying
    /// nothing about it — which is what makes an application override
    /// worth shipping at all.
    #[test]
    fn three_layers_apply_in_order_and_each_overrides_per_item() {
        let base = built_in();
        let app = Shell::default()
            .with_tab(
                Tab::patch("view")
                    .with_question("What does the app think?")
                    .with_groups([
                        Group::new("extra", "Extra").with_items([Item::command("file.save_copy")])
                    ]),
            )
            .with_binding("Ctrl+E", "edit.text");
        let operator = Shell::default().with_tab(Tab {
            id: "view".to_owned(),
            label: Some("Display".to_owned()),
            ..Tab::default()
        });

        let merged = merge(
            MergeInput::built_in(&base)
                .with_app_override(&app)
                .with_operator(&operator),
            &CATALOG,
        );
        let view = &merged.shell.tabs()[0];

        assert_eq!(view.label.as_deref(), Some("Display"), "operator wins");
        assert_eq!(
            view.question.as_deref(),
            Some("What does the app think?"),
            "the app override survives an operator layer that says nothing about it"
        );
        assert!(
            view.groups().iter().any(|g| g.id == "extra"),
            "a group the app override ADDED must survive too: {:?}",
            view.groups()
        );
        assert_eq!(
            merged.shell.keymap.as_ref().and_then(|k| k.get("Ctrl+E")),
            Some("edit.text")
        );
        assert_eq!(
            merged.shell.keymap.as_ref().and_then(|k| k.get("F11")),
            Some("view.fullscreen"),
            "the built-in binding survives both layers"
        );
    }

    /// The later layer wins where two layers disagree about one field.
    #[test]
    fn the_operator_layer_beats_the_application_override() {
        let base = built_in();
        let app = Shell::default().with_tab(Tab::patch("view").with_hidden(true));
        let operator = Shell::default().with_tab(Tab::patch("view").with_hidden(false));
        let merged = merge(
            MergeInput::built_in(&base)
                .with_app_override(&app)
                .with_operator(&operator),
            &CATALOG,
        );
        assert!(!merged.shell.tabs()[0].is_hidden());
    }

    /// **A whole layer from a newer build is skipped, not half-applied.**
    ///
    /// Applying the fields a build happens to recognise, from a document
    /// written against a schema it does not, produces a shell nobody
    /// wrote. The fail-soft answer is to fall back to the layer below and
    /// say so.
    #[test]
    fn a_layer_from_a_newer_schema_is_skipped_whole() {
        let base = built_in();
        let operator = Shell {
            schema: Shell::SCHEMA + 1,
            ..Shell::default().with_tab(Tab::patch("tools"))
        };
        let merged = merge(
            MergeInput::built_in(&base).with_operator(&operator),
            &CATALOG,
        );
        assert_eq!(
            merged
                .shell
                .tabs()
                .iter()
                .map(|t| t.id.as_str())
                .collect::<Vec<_>>(),
            ["file", "view", "tools"],
            "nothing from the future layer may be applied"
        );
        assert_eq!(merged.report.len(), 1);
        assert!(matches!(
            merged.report.skips()[0].reason,
            SkipReason::UnsupportedSchema { .. }
        ));
        merged
            .shell
            .validate()
            .expect("the fallback is still valid");
    }

    /// **A mode naming a removed tab loses that entry, disclosed — and a
    /// mode naming a tab a later layer ADDS keeps it.**
    ///
    /// The second half is why pruning happens once at the end rather than
    /// as each layer is applied: an operator may legitimately define a
    /// mode that names a tab their own layer creates further down the
    /// file, or that the application override introduced.
    #[test]
    fn mode_tab_references_are_pruned_once_at_the_end() {
        let base = built_in();
        let operator = Shell::default()
            .with_mode(Mode::new(
                "read",
                "Read",
                ["file", "view", "gone", "custom"],
            ))
            .with_tab(
                Tab::new("custom", "Custom")
                    .with_groups([Group::new("g", "G").with_items([Item::command("edit.text")])]),
            );

        let merged = merge(
            MergeInput::built_in(&base).with_operator(&operator),
            &CATALOG,
        );
        let read = &merged.shell.modes()[0];
        assert_eq!(
            read.tabs(),
            ["file", "view", "custom"],
            "a tab introduced by the same layer must survive the prune"
        );
        assert_eq!(merged.report.len(), 1);
        assert_eq!(
            merged.report.skips()[0].reason,
            SkipReason::UnknownTab {
                tab: "gone".to_owned()
            }
        );
        assert_eq!(
            merged.report.skips()[0].site,
            Site::Mode {
                mode: "read".to_owned()
            }
        );
        merged
            .shell
            .validate_against(&CATALOG)
            .expect("pruning must leave a manifest that validates");
    }

    /// **The built-in layer is not filtered.**
    ///
    /// It is compiled in and is the reset target. A stale id there is a
    /// programming error that must surface as a validation failure in the
    /// application's own tests, not be quietly repaired at start-up on
    /// every machine that runs it.
    #[test]
    fn the_built_in_layer_is_never_filtered() {
        let base = Shell::new().with_tab(
            Tab::new("view", "View")
                .with_groups([Group::new("g", "G").with_items([Item::command("does.not.exist")])]),
        );
        let merged = merge(MergeInput::built_in(&base), &CATALOG);
        assert!(
            merged.report.is_empty(),
            "merge must not silently repair the built-in layer"
        );
        assert_eq!(merged.shell.tabs()[0].groups()[0].items().len(), 1);
        assert!(
            merged.shell.validate_against(&CATALOG).is_err(),
            "…so that validation is what reports it, loudly"
        );
    }

    /// With no overlays the merge is the identity, apart from stamping
    /// the schema.
    #[test]
    fn merging_nothing_onto_the_built_in_changes_nothing() {
        let base = built_in();
        let merged = merge(MergeInput::built_in(&base), &AnyCommand);
        assert_eq!(merged.shell, base);
        assert!(merged.report.is_empty());
    }

    // ------------------------------------------------------------------
    // SHELL_FRAMEWORK.md §5b — a capability that is not in this build
    // ------------------------------------------------------------------
    //
    // ★★★ These four tests are the whole of the §5b contract, and they are
    // written against the MERGE rather than against a ribbon on purpose:
    // the rule they defend is *a capability's presence is expressed by
    // registering its command, and by nothing else*, and a test that drew a
    // band would be a test of the renderer's obedience rather than of the
    // mechanism. If the renderer is the only thing that knows, the exe→DLL
    // move is a rewrite.

    /// **A conditional item whose command is absent is dropped — from the
    /// BUILT-IN layer, which nothing else in this module filters.**
    ///
    /// The case §5b was written for and left open: the manifest compiled
    /// into the binary names a command the build did not register, because
    /// the build was compiled without that capability. Before this, that was
    /// either a hard validation failure at start-up or a live control with
    /// no handler.
    #[test]
    fn a_built_in_item_provided_by_an_absent_capability_is_dropped_by_name() {
        let built_in = Shell::new().with_tab(Tab::new("file", "File").with_groups([
            Group::new("security", "Security").with_items([
                Item::command("file.open"),
                Item::command("file.sign").provided_by("signing"),
            ]),
        ]));

        let merged = merge(MergeInput::built_in(&built_in), &CATALOG);

        let items: Vec<&str> = merged
            .shell
            .tabs()
            .iter()
            .flat_map(Tab::groups)
            .flat_map(Group::items)
            .filter_map(Item::command_id)
            .collect();
        assert_eq!(
            items,
            ["file.open"],
            "the conditional item is gone and its neighbour is untouched"
        );

        let skips = merged.report.skips();
        assert_eq!(skips.len(), 1, "exactly one skip: {skips:?}");
        assert_eq!(
            skips[0].reason,
            SkipReason::CapabilityAbsent {
                capability: "signing".to_owned(),
                command: "file.sign".to_owned(),
            },
            "and it names the capability, not merely the command"
        );
        assert_eq!(skips[0].layer, Layer::BuiltIn);
    }

    /// **A MANDATORY built-in item is still not filtered.**
    ///
    /// ★★★ The other half, and the one a careless implementation loses. The
    /// pass that drops conditional items runs over the same lists, and reusing
    /// the ordinary item filter there would have silently repaired a
    /// programming error in the built-in manifest — the exact behaviour this
    /// module's header says must never happen, because it hides the bug on
    /// every machine that runs it.
    ///
    /// So the item survives the merge, and `validate_against` is left to fail
    /// on it, which is asserted here rather than assumed.
    #[test]
    fn a_mandatory_built_in_item_is_still_left_for_validation_to_reject() {
        let built_in = Shell::new().with_tab(Tab::new("file", "File").with_groups([
            Group::new("security", "Security").with_items([
                // No `provided_by` — this one is a typo, not a lite build.
                Item::command("file.sgin"),
            ]),
        ]));

        let merged = merge(MergeInput::built_in(&built_in), &CATALOG);

        assert!(
            merged.report.skips().is_empty(),
            "the merge repairs nothing: {:?}",
            merged.report.skips()
        );
        assert!(
            merged.shell.validate_against(&CATALOG).is_err(),
            "and validation is what says so, loudly"
        );
    }

    /// **A conditional item whose command IS registered renders like any
    /// other.**
    ///
    /// The negative control, and it is not a formality: an implementation
    /// that dropped every conditional item — or drew one differently — would
    /// pass the first test above and ship a build in which the capability is
    /// compiled in and invisible. The field says how to read an item's
    /// *absence*; it must say nothing about its presence.
    #[test]
    fn a_conditional_item_whose_capability_is_present_is_kept_untouched() {
        let built_in = Shell::new().with_tab(
            Tab::new("tools", "Tools").with_groups([Group::new("batch", "Batch")
                .with_items([Item::command("tools.batch").provided_by("batch-engine")])]),
        );

        let merged = merge(MergeInput::built_in(&built_in), &CATALOG);

        assert!(merged.report.skips().is_empty());
        let kept: Vec<&Item> = merged
            .shell
            .tabs()
            .iter()
            .flat_map(Tab::groups)
            .flat_map(Group::items)
            .collect();
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].command_id(), Some("tools.batch"));
        assert_eq!(kept[0].capability(), Some("batch-engine"));
    }

    /// **An OPERATOR's conditional item is dropped at the operator's layer,
    /// with the same reason.**
    ///
    /// The two paths are different code — an overlay is filtered as it is
    /// applied, the built-in layer in a final pass — and they share
    /// [`absence_reason`] so they cannot disagree about which of the two
    /// reasons applies. This is what pins that sharing: an operator who put
    /// `file.sign` on their own QAT-adjacent group in a build without signing
    /// must be told *"this build does not include that"*, not *"you made a
    /// mistake"*, because they did not.
    #[test]
    fn an_operator_item_provided_by_an_absent_capability_reports_the_capability() {
        let built_in = Shell::new().with_tab(Tab::new("file", "File").with_groups([
            Group::new("security", "Security").with_items([Item::command("file.open")]),
        ]));
        let operator = Shell::new().with_tab(
            Tab::new("file", "File").with_groups([Group::patch("security")
                .with_items([Item::command("file.sign").provided_by("signing")])]),
        );

        let merged = merge(
            MergeInput::built_in(&built_in).with_operator(&operator),
            &CATALOG,
        );

        let skips = merged.report.skips();
        assert_eq!(skips.len(), 1, "{skips:?}");
        assert_eq!(
            skips[0].reason,
            SkipReason::CapabilityAbsent {
                capability: "signing".to_owned(),
                command: "file.sign".to_owned(),
            }
        );
        assert_eq!(
            skips[0].layer,
            Layer::Operator,
            "at THEIR layer, so the disclosure names the file they edited"
        );
    }
}
