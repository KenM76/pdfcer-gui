//! # shell — pdfcer's ribbon, modes, QAT and keymap, as data
//!
//! This module is pdfcer's half of the contract `SHELL_FRAMEWORK.md` §1
//! sets up:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/shell/mod.md`.

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

    /// **The built-in manifest is structurally valid.**
    #[test]
    fn the_built_in_manifest_is_valid() {
        manifest::built_in()
            .validate()
            .expect("the built-in manifest must satisfy every structural rule");
    }

    /// **THE BUILT-IN MANIFEST SURVIVES A BUILD WITH A CAPABILITY COMPILED
    /// OUT — and this is R8's whole point, asserted rather than hoped for.**
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
    #[test]
    fn the_signing_command_is_registered_exactly_when_the_feature_is_on() {
        let (_, registry) = shell_and_registry();
        assert_eq!(
            registry.get("file.sign").is_some(),
            cfg!(feature = "signing"),
            "`file.sign` is registered if and only if the `signing` feature is compiled in — that is the ONLY way this GUI expresses the capability"
        );
    }

    /// **Every command the manifest names is registered.**
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

    /// **…and the converse: no registered command is orphaned.**
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
        // …and a command whose operand is THE SURFACE THE OPERATOR
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

    /// **Every `CUSTOM_BACKED` entry is real, in both directions.**
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

    /// **Read ⊂ Review ⊂ Edit.**
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

    /// **Nothing in `PLANNED` is also in the manifest, and nothing in
    /// `PLANNED` is registered.**
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

    /// **The ribbon's overflow chevron exists in the bundled fonts.**
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

        // **A measurement that did not happen must not read as a pass.**
        // Under `cargo test -p egui-shell` there are no fonts at all, so a
        // bare `assert!(has_glyph)` written as `unwrap_or(true)` would be
        // vacuous in exactly the command a developer runs most. `Some(false)`
        // and `None` are therefore distinguished rather than collapsed.
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
