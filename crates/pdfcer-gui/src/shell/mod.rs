//! # shell — pdfcer's ribbon, modes, QAT and keymap, as data
//!
//! This module is pdfcer's half of the contract `SHELL_FRAMEWORK.md` §1
//! sets up:
//!
//! > **The shell is data. Tabs, groups, commands, panels, layouts, modes
//! > and key bindings are a serializable document that the application
//! > *supplies* and the operator *edits* — not code that has to be
//! > recompiled to change.**
//!
//! `egui-shell` owns the *types* and the *rules*: what a tab is, what a
//! group is, that a command may appear on exactly one tab, that a mode may
//! only name tabs that exist. It knows nothing about PDF and must never
//! learn. This module owns the *content*: which tabs pdfcer has, what is on
//! them, which verbs exist and what each one is called.
//!
//! | Submodule | Holds |
//! |---|---|
//! | [`manifest`] | [`manifest::built_in`] — the whole ribbon as an `egui_shell::Shell` value: eight tabs, thirty-three groups, three modes, the QAT and the keymap. Plus [`manifest::PLANNED`], the commands deliberately **absent**. |
//! | [`commands`] | [`commands::register`] — every command the manifest names, with its label, tooltip, icon key, enable predicate and opaque handler token. |
//! | [`menus`] | [`menus::built_in`] — the four context menus, carried on the same `Shell`, plus [`menus::MenuHost`], the one seam a right-click site uses. |
//! | [`ron`] | The same manifest as a `.ron` file, with a test that the two agree. |
//!
//! ## The third surface
//!
//! [`menus`] is the context-menu half of `RIBBON_IA.md` §5.8's three
//! surfaces, and it is deliberately **not** a second vocabulary: a menu
//! item is the same `egui_shell::manifest::Item` a ribbon band holds,
//! resolved through the same registry into the same handler token. So P1 —
//! one command, one tab — is *not* extended over menus, and must not be:
//! §5.8 states that a menu carrying a tab's command again *"is not
//! duplication in the P1 sense — context menus are not tabs"*. The tests
//! below reflect that split, and [`menus`]' own tests carry the checks
//! `egui-shell` does not perform.
//!
//! ## The specification this implements
//!
//! `RIBBON_IA.md` §5, tab by tab and group by group, amended by
//! `MODES_AND_PANELS.md` Part 1 (the Read/Review/Edit selector, and the
//! two new View ▸ Window settings). Where this module departs from either
//! document — and there are a handful of places where they contradict each
//! other or contradict what exists — the departure is documented at the
//! site, in the submodule that makes it.
//!
//! ## The rule that shapes this module more than any other
//!
//! `RIBBON_IA.md` P3, **no placeholders**:
//!
//! > An unavailable capability renders nothing, not a disabled stub.
//! > Greying is reserved for *temporarily* unavailable — no document open,
//! > document encrypted, undo stack empty — and is always explained on
//! > hover.
//!
//! `RIBBON_IA.md` §5 marks every command with where it exists today: **G**
//! in the GUI, **C** in `pdfcer-core`/`pdfcer` only, **N** nowhere. P3
//! means a **C** or an **N** must be *absent from this manifest*, not
//! present and disabled — a **C** row is a command whose engine is written
//! and whose shell is not, which is a cheap win and still not a shipped
//! command.
//!
//! Absent is not the same as forgotten, so every one of them is listed in
//! [`manifest::PLANNED`] with the reason it is not here. That list is
//! machine-readable, tested against the manifest in both directions, and
//! is what a later stage reads to find its work.
//!
//! ## Where the strings come from
//!
//! Nowhere in this module. Every operator-visible string — tab labels, tab
//! questions, group captions, command labels, command tooltips, mode
//! labels — is a call into [`crate::text::ribbon`] or
//! [`crate::text::commands`]. `tools/gates/check-ui-strings.sh` scans this
//! tree recursively and fails the build on a literal that carries
//! whitespace, which is the mechanical half of the rule; the reason for
//! the rule is in [`crate::text`]'s own header.
//!
//! ## Where the behaviour comes from
//!
//! Also nowhere in this module. A registered command carries an
//! `egui_shell::HandlerToken`, an opaque `u64` the shell stores and hands
//! back. This module assigns those numbers and says nothing about what
//! they do; the application dispatches on them at one choke point, which
//! is where a confirmation gate, an undo entry or a trace belongs.

pub mod commands;
pub mod manifest;
pub mod menus;
/// The optional capabilities every context menu is built with — an icon
/// painter and a rect sink — kept apart from [`menus`] because both are
/// properties of the build rather than of a frame. See its header.
pub mod menus_wiring;
pub mod ron;

#[cfg(test)]
mod tests {
    use super::*;
    use egui_shell::CommandRegistry;
    use egui_shell::manifest::{Group, Item, Site, Tab};
    use std::collections::BTreeSet;

    /// The manifest and a fully populated registry, built the way the
    /// application builds them. Shared by every cross-cutting test below
    /// so none of them can disagree about what "the shell" is.
    fn shell_and_registry() -> (egui_shell::Shell, CommandRegistry) {
        let mut registry = CommandRegistry::new();
        commands::register(&mut registry);
        (manifest::built_in(), registry)
    }

    /// **★ The built-in manifest is structurally valid.**
    ///
    /// Everything `egui_shell::Shell::validate` checks, which includes the
    /// rule this whole information architecture is built on: **a command
    /// appears on at most one tab**, counting the contextual Format tab.
    ///
    /// That last clause is the one worth having a test for. `RIBBON_IA.md`
    /// §5 is a document written by a person, and it lists `Thin lines`
    /// under both View ▸ Render and View ▸ Display, and `Comments` under
    /// both View ▸ Panels and Markup ▸ Comments. Both would be violations,
    /// both were resolved deliberately (see [`manifest`]'s header), and
    /// this is the check that would have caught them if they had not been.
    #[test]
    fn the_built_in_manifest_is_valid() {
        manifest::built_in()
            .validate()
            .expect("the built-in manifest must satisfy every structural rule");
    }

    /// ★★★ **THE BUILT-IN MANIFEST SURVIVES A BUILD WITH A CAPABILITY
    /// COMPILED OUT — and this is R8's whole point, asserted rather than
    /// hoped for.**
    ///
    /// `SHELL_FRAMEWORK.md` §5b, the operator's directive of 2026-08-13:
    /// *"Everything should be capable of being modular… if not needed by
    /// someone they could just remove them and they would not show up as
    /// options in the GUI."*
    ///
    /// # What this simulates, and why simulating it is the right test
    ///
    /// It builds the real registry, then **withholds** every command that the
    /// manifest marks conditional — which is exactly the state a
    /// `--no-default-features` build is in — and asserts three things:
    ///
    /// 1. The merge drops those items, and drops **only** those items.
    /// 2. Each drop reports [`SkipReason::CapabilityAbsent`], naming the
    ///    capability. Not `UnknownCommand`: one says *"this build does not
    ///    include that"* and the other says *"someone made a mistake"*, and a
    ///    log that confuses them turns modularity and a typo into one event.
    /// 3. **The merged shell still validates.** This is the assertion that
    ///    matters most and the one whose absence would have been catastrophic:
    ///    `validate_against` is strict, an unregistered command fails it, a
    ///    failed validation leaves `PdfcerApp::shell` as `None`, and
    ///    `Capabilities::for_mode` returns **FULL** when the shell is absent.
    ///    So without the merge, a lite build would have lost its whole ribbon
    ///    **and granted every authoring capability to every mode, including
    ///    Read.** A previous session turned eight mode-gating tests red from
    ///    the other direction; this is the same trap facing the other way.
    ///
    /// ⚠ A cross-compilation would be a better test and is not available from
    /// inside one: `cfg!` describes the build that is running. What this does
    /// instead is exercise **the mechanism** — the merge, the skip reason and
    /// the validation — against a registry in the state the other build
    /// produces, which is the part that could be wrong. The other half (that
    /// the command really is absent when the feature is off) is a one-line
    /// `cfg!` assertion below and is checked for real by building with
    /// `--no-default-features --features jpx,ocrs`.
    #[test]
    fn the_built_in_manifest_survives_a_build_without_an_optional_capability() {
        use egui_shell::manifest::{CommandCatalog, MergeInput, SkipReason, merge};

        let built_in = manifest::built_in();
        // Every id the manifest marks conditional, whatever it is today. Read
        // out of the manifest rather than hard-coded, so a second conditional
        // command added tomorrow is covered by this test without editing it —
        // and so the test cannot pass because somebody deleted the field.
        let conditional: Vec<(String, String)> = built_in
            .tabs()
            .iter()
            .flat_map(egui_shell::manifest::Tab::groups)
            .flat_map(egui_shell::manifest::Group::items)
            .filter_map(|item| Some((item.command_id()?.to_owned(), item.capability()?.to_owned())))
            .collect();
        assert!(
            !conditional.is_empty(),
            "the manifest must mark at least one item conditional, or this test asserts nothing — `file.sign` is the first and `capability:` is the field"
        );

        /// The real registry with a named set of commands withheld.
        struct Without {
            real: CommandRegistry,
            withheld: Vec<String>,
        }
        impl CommandCatalog for Without {
            fn contains(&self, id: &str) -> bool {
                !self.withheld.iter().any(|w| w == id) && self.real.get(id).is_some()
            }
        }

        let mut real = CommandRegistry::new();
        commands::register(&mut real);
        let catalog = Without {
            real,
            withheld: conditional.iter().map(|(id, _)| id.clone()).collect(),
        };

        let merged = merge(MergeInput::built_in(&built_in), &catalog);

        // 1 + 2: exactly those items, each with the right reason.
        assert_eq!(
            merged.report.skips().len(),
            conditional.len(),
            "one skip per conditional command and nothing else: {:?}",
            merged.report.skips()
        );
        for (id, capability) in &conditional {
            assert!(
                merged.report.skips().iter().any(|s| s.reason
                    == SkipReason::CapabilityAbsent {
                        capability: capability.clone(),
                        command: id.clone(),
                    }),
                "`{id}` must be dropped as CapabilityAbsent(`{capability}`), not as a mistake: {:?}",
                merged.report.skips()
            );
        }

        // …and the item really is gone from the ribbon, which is the operator-
        // visible half of the same fact.
        let still_there: Vec<String> = merged
            .shell
            .tabs()
            .iter()
            .flat_map(egui_shell::manifest::Tab::groups)
            .flat_map(egui_shell::manifest::Group::items)
            .filter_map(|i| i.command_id())
            .filter(|id| conditional.iter().any(|(c, _)| c == id))
            .map(str::to_owned)
            .collect();
        assert!(
            still_there.is_empty(),
            "a conditional command whose build does not have it must not render: {still_there:?}"
        );

        // 3: THE assertion. The shell must still be valid, or the application
        // falls back to no shell at all and every mode becomes FULL.
        merged
            .shell
            .validate_against(&catalog)
            .expect("a build without an optional capability must still have a ribbon");
    }

    /// **…and with the feature ON, the command really is registered.**
    ///
    /// The negative control for the test above, and it is not a formality: that
    /// one withholds the command itself, so it would pass identically against a
    /// build in which `file.sign` was never registered at all. This is what
    /// makes the pair say *"the feature controls it"* rather than *"it is
    /// absent"*.
    #[test]
    fn the_signing_command_is_registered_exactly_when_the_feature_is_on() {
        let (_, registry) = shell_and_registry();
        assert_eq!(
            registry.get("file.sign").is_some(),
            cfg!(feature = "signing"),
            "`file.sign` is registered if and only if the `signing` feature is compiled in — that is the ONLY way this GUI expresses the capability"
        );
    }

    /// **★ Every command the manifest names is registered.**
    ///
    /// Walks every reference site — tab groups, the quick-access toolbar
    /// and the keymap — not just the groups. A key bound to a command that
    /// does not exist is a chord that does nothing, and it is the
    /// reference easiest to leave behind when a command is renamed,
    /// because unlike a button it is invisible until pressed.
    ///
    /// ★★★ **It validates the MERGED shell, not the raw manifest** — corrected
    /// 2026-09-06, when `file.sign` became the first conditional item.
    ///
    /// `validate_against` is strict by design and rejects any reference to an
    /// unregistered command. That is right for a mandatory item and wrong for a
    /// conditional one, whose absence in a build without its capability is the
    /// intended configuration rather than a bug. The merge is what tells the
    /// two apart, and `PdfcerApp::new` runs it before validating for exactly
    /// this reason.
    ///
    /// ⚠ So this test asserts something slightly stronger than it used to: not
    /// only that everything resolves, but that **the only things the merge had
    /// to drop were capability-absent items.** An `UnknownCommand` skip fails
    /// here rather than being swallowed by the merge's fail-soft posture —
    /// which is the hole a naive "merge then validate" would have opened.
    #[test]
    fn every_command_the_manifest_names_is_registered() {
        use egui_shell::manifest::{MergeInput, SkipReason, merge};

        let (shell, registry) = shell_and_registry();
        let merged = merge(MergeInput::built_in(&shell), &registry);
        for skip in merged.report.skips() {
            assert!(
                matches!(skip.reason, SkipReason::CapabilityAbsent { .. }),
                "the merge may only drop items this BUILD does not have; anything else is a stale reference: {skip}"
            );
        }
        merged
            .shell
            .validate_against(&registry)
            .expect("every referenced command id must be registered");
    }

    /// **★ …and the converse: no registered command is orphaned.**
    ///
    /// A command in the registry that no tab, no QAT slot and no key
    /// binding mentions is unreachable. It is not a crash and it is not
    /// caught by anything in `egui-shell` — the framework's validation
    /// runs manifest → registry, and this is the other direction.
    ///
    /// It is the failure mode a *rename* produces. Change a command's id
    /// in the manifest, forget the registry, and
    /// `every_command_the_manifest_names_is_registered` fires. Change it
    /// in the registry and forget the manifest, and nothing fires at all:
    /// the old id is simply never referenced again, the button disappears,
    /// and the suite stays green.
    ///
    /// # ★ A context menu does not count as reachability
    ///
    /// `command_references()` walks tab groups, the QAT and the keymap —
    /// and deliberately **not** [`menus`]. That is not an omission in
    /// `egui-shell`; it is the right answer to *this* question. A command
    /// reachable only by right-clicking one particular surface is a command
    /// nobody can find: a context menu is discovered by an operator who
    /// already suspects something is there, which is exactly the state a
    /// command with no other home cannot put them in.
    ///
    /// So a menu-only command must fail this test, and
    /// `menus::tests::every_menu_command_is_also_reachable_from_the_ribbon`
    /// states the same rule from the menu side, where the failure message
    /// can name the menu.
    ///
    /// # ★ …and a CUSTOM ITEM does count, because it is a ribbon control
    ///
    /// `command_references()` walks the places a command *id* can appear, and
    /// an `egui_shell::manifest::Item::Custom` carries none — the shell
    /// reserves the space and the application draws whatever it likes. A
    /// command whose ribbon control is such an item is therefore reachable
    /// (it is a control in a band on a tab, as discoverable as any button)
    /// and invisible to the function this test is built on.
    ///
    /// Those are enumerated in [`manifest::CUSTOM_BACKED`], with the item
    /// that draws each and the reason a button could not have. Consulting the
    /// register keeps the rename check the test exists for — a command
    /// reachable by *nothing* still fails — while not forcing a redundant
    /// second button onto the tab to satisfy a check about ids.
    #[test]
    fn no_registered_command_is_orphaned() {
        let (shell, registry) = shell_and_registry();
        let mut referenced: BTreeSet<String> = shell
            .command_references()
            .into_iter()
            .map(|(_, id)| id)
            .collect();
        referenced.extend(
            manifest::CUSTOM_BACKED
                .iter()
                .map(|(id, _, _)| (*id).to_owned()),
        );
        // ★★ …and a command whose operand is THE SURFACE THE OPERATOR
        // GESTURED AT, which no ribbon control can supply. See
        // `manifest::TAB_SCOPED`, whose header holds the bar (the same one
        // `CUSTOM_BACKED` sets), the discoverability answer, and the
        // condition under which these entries come back out.
        referenced.extend(manifest::TAB_SCOPED.iter().map(|(id, _)| (*id).to_owned()));

        let orphans: Vec<&str> = registry
            .ids()
            .filter(|id| !referenced.contains(*id))
            .collect();
        assert!(
            orphans.is_empty(),
            "these commands are registered but unreachable — no tab, no QAT slot, \
             no key binding and no custom item mentions them: {orphans:?}"
        );
    }

    /// **★ Every `CUSTOM_BACKED` entry is real, in both directions.**
    ///
    /// The register buys an exemption from the orphan check above, so it has
    /// to be worth exactly what it claims and no more:
    ///
    /// 1. **The command is registered.** An entry naming an id nothing
    ///    registers would be excusing a command that does not exist.
    /// 2. **The custom item is in the manifest.** This is the one that rots:
    ///    delete the `Item::Custom` from a tab and the command silently
    ///    becomes a genuine orphan while this register goes on excusing it —
    ///    the exemption outliving the thing it was granted for.
    /// 3. **The command is on no tab and no QAT slot.** An entry for a
    ///    command that *is* referenced would be an exemption nobody needs,
    ///    and the next reader would take it as evidence that custom items and
    ///    buttons are interchangeable.
    #[test]
    fn every_custom_backed_command_has_its_item_and_needs_its_exemption() {
        let (shell, registry) = shell_and_registry();
        let referenced: BTreeSet<String> = shell
            .command_references()
            .into_iter()
            .map(|(_, id)| id)
            .collect();
        let kinds: BTreeSet<&str> = shell
            .all_tabs()
            .flat_map(Tab::groups)
            .flat_map(Group::items)
            .filter_map(|item| match item {
                Item::Custom { kind, .. } => Some(kind.as_str()),
                Item::Command { .. } | Item::Separator => None,
            })
            .collect();

        for (id, kind, why) in manifest::CUSTOM_BACKED {
            assert!(
                registry.get(id).is_some(),
                "`{id}` is listed as custom-backed ({why}) but is not registered"
            );
            assert!(
                kinds.contains(kind),
                "`{id}` is listed as custom-backed by the `{kind}` item, and no tab holds \
                 such an item — so the command is a real orphan and this entry is excusing \
                 a control that was deleted"
            );
            assert!(
                !referenced.contains(*id),
                "`{id}` is on a tab, the QAT or the keymap already, so it needs no \
                 exemption from the orphan check"
            );
        }
    }

    /// **★ Read ⊂ Review ⊂ Edit.**
    ///
    /// The premise of the whole mode feature, and the thing nothing else
    /// enforces. `MODES_AND_PANELS.md` Part 1:
    ///
    /// > The three positions are **ordered by capability** — each is a
    /// > superset of the one before. A slider says that; three toggle
    /// > buttons do not… The ordering is the information.
    ///
    /// If Review ever gained a tab that Edit lacks, the control would
    /// still render as a slider and would be lying about what it does.
    /// `egui_shell::Shell::validate` cannot catch this: it checks that a
    /// mode names tabs that exist, which is a different question, and it
    /// must not assume modes are ordered at all — a different application
    /// may ship three unrelated workspaces.
    ///
    /// Checked as a chain over the modes **in manifest order**, not by
    /// naming the three ids, so that a fourth position inserted between
    /// two existing ones is checked too.
    #[test]
    fn each_mode_is_a_subset_of_the_next() {
        let shell = manifest::built_in();
        let modes = shell.modes();
        assert!(
            modes.len() >= 2,
            "the ordering rule is vacuous with fewer than two modes"
        );

        for pair in modes.windows(2) {
            let (narrow, wide) = (&pair[0], &pair[1]);
            let wide_tabs: BTreeSet<&str> = wide.tabs().iter().map(String::as_str).collect();
            let missing: Vec<&str> = narrow
                .tabs()
                .iter()
                .map(String::as_str)
                .filter(|t| !wide_tabs.contains(t))
                .collect();
            assert!(
                missing.is_empty(),
                "mode `{}` is meant to be a subset of `{}`, but names tabs it does not \
                 have: {missing:?}. The selector renders as an ordered slider, so a mode \
                 that is not a subset of the next makes that control a lie.",
                narrow.id,
                wide.id
            );
            assert!(
                narrow.tabs().len() < wide.tabs().len(),
                "mode `{}` and `{}` contain the same tabs; two positions on a \
                 capability slider that differ in nothing are two positions too many",
                narrow.id,
                wide.id
            );
        }
    }

    /// Each mode's tab list references only tabs that exist, and only
    /// **ordinary** tabs.
    ///
    /// `egui_shell::Shell::validate` already enforces this, and it is
    /// asserted again here against the concrete tab set because the
    /// failure it prevents is specific and silent: a mode naming the
    /// contextual `format` tab would be asking for a tab whose presence is
    /// decided by the selection, not by configuration, and the mode would
    /// appear to work until nothing was selected.
    #[test]
    fn every_mode_names_only_ordinary_tabs_that_exist() {
        let shell = manifest::built_in();
        let ordinary: BTreeSet<&str> = shell.tabs().iter().map(|t| t.id.as_str()).collect();
        let contextual: BTreeSet<&str> = shell
            .contextual_tabs()
            .iter()
            .map(|t| t.id.as_str())
            .collect();

        for mode in shell.modes() {
            for tab in mode.tabs() {
                assert!(
                    !contextual.contains(tab.as_str()),
                    "mode `{}` names the contextual tab `{tab}`",
                    mode.id
                );
                assert!(
                    ordinary.contains(tab.as_str()),
                    "mode `{}` names `{tab}`, which is not a tab",
                    mode.id
                );
            }
        }
    }

    /// Every group has at least one item.
    ///
    /// P4 makes group captions mandatory; the corollary is that a group
    /// must have something to caption. An empty band is exactly the
    /// "unfinished program" reading `RIBBON_IA.md` §3 describes, and it is
    /// the shape the no-placeholders rule produces if a group's entire
    /// contents turn out to be **N** — Pages ▸ Stamp, Edit ▸ Arrange,
    /// Measure ▸ Quantity and three more, all of which are consequently
    /// absent as *groups* rather than present and empty.
    #[test]
    fn no_group_is_empty() {
        for tab in manifest::built_in().all_tabs() {
            for group in tab.groups() {
                assert!(
                    !group.items().is_empty(),
                    "group `{}` on tab `{}` has no items; an empty band reads as an \
                     unfinished program. Omit the group instead.",
                    group.id,
                    tab.id
                );
            }
        }
    }

    /// Every tab has at least one group, and a question.
    #[test]
    fn every_tab_is_populated_and_asks_a_question() {
        for tab in manifest::built_in().all_tabs() {
            assert!(!tab.groups().is_empty(), "tab `{}` has no groups", tab.id);
            let question = tab
                .question
                .as_deref()
                .unwrap_or_else(|| panic!("tab `{}` states no question", tab.id));
            assert!(
                question.ends_with('?'),
                "tab `{}`'s question is not one: {question}",
                tab.id
            );
        }
    }

    /// **★ Nothing in `PLANNED` is also in the manifest, and nothing in
    /// `PLANNED` is registered.**
    ///
    /// The two lists are complementary by construction and would drift
    /// silently without this. The dangerous direction is the second one: a
    /// command that is registered *and* listed as planned is a command
    /// someone half-built, and the registry is the half that does not show
    /// up in a screenshot.
    #[test]
    fn planned_commands_are_genuinely_absent() {
        let (shell, registry) = shell_and_registry();
        let referenced: BTreeSet<String> = shell
            .command_references()
            .into_iter()
            .map(|(_, id)| id)
            .collect();

        for (id, why) in manifest::PLANNED {
            assert!(
                !referenced.contains(*id),
                "`{id}` is listed as planned ({why}) but the manifest references it"
            );
            assert!(
                registry.get(id).is_none(),
                "`{id}` is listed as planned ({why}) but it is registered"
            );
        }
    }

    /// `PLANNED` has no duplicate ids and every entry gives a reason.
    ///
    /// The reason is the entry's whole value. `("pages.crop", "")` records
    /// that somebody once thought about cropping and nothing else; the
    /// list exists so a later stage can tell a **C** row — engine written,
    /// shell missing, a day's work — from an **N** row that is a month.
    #[test]
    fn every_planned_entry_is_unique_and_explains_itself() {
        let mut ids: Vec<&str> = manifest::PLANNED.iter().map(|(id, _)| *id).collect();
        let total = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), total, "PLANNED lists an id twice");

        for (id, why) in manifest::PLANNED {
            assert!(
                why.len() > 15,
                "`{id}`'s reason is too short to be one: {why:?}"
            );
        }
    }

    /// Every quick-access toolbar entry is a real command, and the QAT is
    /// the four `RIBBON_IA.md` §6 names.
    ///
    /// Pinned because the QAT is the surface a returning operator's hands
    /// know without looking. Adding to it is a decision; drifting into it
    /// is not.
    ///
    /// ★ **`file.save_copy` became `file.save` on 2026-08-20**, and that is a
    /// decision rather than a drift. `RIBBON_IA.md` §6's list is *open, save,
    /// undo, redo* — the four every application in this class puts there — and
    /// the second slot held Save-a-copy only because in-place save did not
    /// exist. A quick-access Save that opens a file dialog is not the control
    /// those hands are reaching for.
    #[test]
    fn the_qat_is_the_four_documented_commands() {
        let shell = manifest::built_in();
        let qat = shell.qat.as_ref().expect("the QAT is part of the manifest");
        assert_eq!(
            qat.ids(),
            ["file.open", "file.save", "edit.undo", "edit.redo"]
        );
    }

    /// Undo and redo are reachable, and they are reachable from the QAT.
    ///
    /// This is the Pass 47.1 defect, inverted into a test. That defect was
    /// caused by mirroring undo/redo onto *every* tab, which made the
    /// ribbon render only the active band and left undo unreachable from
    /// the Measure tab. The rule that came out of it — one command, one
    /// tab — is enforced by `egui-shell`. The rule's own failure mode is
    /// the opposite one: a command reachable from **no** surface at all.
    ///
    /// Undo and redo sit on no tab in `RIBBON_IA.md`'s layout. That is
    /// deliberate and it is only safe because the QAT is always visible.
    #[test]
    fn undo_and_redo_are_reachable_without_a_tab() {
        let shell = manifest::built_in();
        let on_a_tab: Vec<&str> = shell
            .all_tabs()
            .flat_map(Tab::groups)
            .flat_map(Group::items)
            .filter_map(Item::command_id)
            .collect();
        for id in ["edit.undo", "edit.redo"] {
            assert!(
                !on_a_tab.contains(&id),
                "`{id}` is on a tab; RIBBON_IA.md §7 keeps it on the QAT alone"
            );
            assert!(
                shell
                    .command_references()
                    .iter()
                    .any(|(site, cmd)| matches!(site, Site::Qat) && cmd == id),
                "`{id}` is on no tab AND not on the QAT — it is unreachable"
            );
        }
    }

    /// ★ **The ribbon's overflow chevron exists in the bundled fonts.**
    ///
    /// The third of this family — `crate::app::status` and
    /// `crate::find::bar` each guard their own glyphs, and the reason there
    /// are three rather than one is that none of them can see the others'
    /// strings. This one guards a string neither can see and **neither
    /// crate could have guarded**.
    ///
    /// # What shipped, and why nothing caught it
    ///
    /// `egui_shell::ribbon::plan::overflow_label` built `"⌄ N more"` from
    /// U+2304, which egui's bundled stack cannot draw — so the affordance
    /// rendered as `□ 1 more` in **every build this project has ever
    /// produced**, on both the ribbon band and the dock tab bar. Found by
    /// an agent reading its own screenshots, not by a test.
    ///
    /// It is the same defect that produced `chevron-down.svg` and the same
    /// one `every_glyph_the_status_bar_draws_has_a_glyph` was written after
    /// — that catalog was drafted with `◀ ▶ ▸ ▾`, all four missing. Three
    /// separate sightings of one hazard.
    ///
    /// **The structural reason it survived all three:** `cargo test -p
    /// egui-shell` compiles without egui's `default_fonts`, so a
    /// `has_glyph` assertion *inside* the crate that owns the string would
    /// answer about a font set no real build has, and would pass for the
    /// whole life of the defect. The crate cannot check its own string.
    /// This is the test it delegates.
    ///
    /// # The coupling, stated rather than hidden
    ///
    /// `overflow_label` is `pub(crate)` there, so this spells the character
    /// itself. That is a two-sided pin, the same shape as the
    /// `ribbon.item.*` name contract: `egui-shell`'s own
    /// `the_overflow_label_uses_the_pinned_chevron` asserts the label
    /// *contains* this codepoint, and this asserts the codepoint is
    /// *drawable*. Changing the character fails one; changing it to
    /// something undrawable fails the other.
    #[test]
    fn the_ribbon_overflow_chevron_has_a_glyph() {
        // ui-text-exempt: a codepoint under test, never a rendered string.
        const CHEVRON: char = '⏷';

        let ctx = egui::Context::default();
        let mut has = None;
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            let font = egui::FontId::proportional(14.0);
            ui.ctx()
                .fonts_mut(|f| has = Some(f.has_glyph(&font, CHEVRON)));
        });

        // `Some(false)` rather than `None` — `HANDOFF.md` §10's rule.
        // Under `cargo test -p egui-shell` there are no fonts at all, and a
        // bare `assert!(has_glyph)` written as `unwrap_or(true)` would be
        // vacuous in exactly the command a developer runs most.
        assert_eq!(
            has,
            Some(true),
            "the ribbon and dock overflow affordances draw U+{:04X}, and the bundled \
             fonts cannot; it renders as a tofu box on every one of them. If a \
             measurement did not happen at all this is `None`, which is the other \
             failure and is not a pass.",
            CHEVRON as u32
        );
    }
}
