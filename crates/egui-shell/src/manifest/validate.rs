//! Manifest validation — what a *complete* manifest must satisfy.
//!
//! # Where this sits
//!
//! [`super::merge`] is fail-soft and runs first: it is handed files an
//! operator edited by hand and an application shipped two versions ago,
//! and its job is to lose one item rather than a layout. This module is
//! strict and runs on the merged result. What it rejects are
//! contradictions no fail-soft rule can repair — two tabs with one id, a
//! command claimed by two tabs, a tab with no label to render.
//!
//! The point that matters about running here rather than at compile time
//! (`SHELL_FRAMEWORK.md` §5): a compile-time uniqueness test can only check
//! the ribbon the developers wrote. This one runs against the **merged**
//! manifest, so a customization that puts one command on two tabs is
//! rejected at load with a message naming the command — and the merged
//! manifest is the ribbon the operator actually gets.
//!
//! # Why every error names the thing that is wrong
//!
//! This validator runs against a file a person edited. An error that says
//! "invalid manifest" tells them to bisect their own customization by
//! hand; an error that says "`view.fit_page` appears on both `view` and
//! `tools`" tells them which two lines to look at. Every variant of
//! [`ManifestError`] therefore carries the identifiers structurally, not
//! interpolated into prose, so an application can also *act* on them —
//! offering "reset this one tab" rather than "reset everything".
//!
//! Design and rationale: `docs/modules/egui-shell/manifest/validate.md`.

use super::{CommandCatalog, Item, Shell};
use std::collections::BTreeMap;

/// Where in a manifest a command reference lives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Site {
    /// The whole document — used for a layer-level problem such as an
    /// unsupported schema.
    Document,
    /// Inside a group on a tab.
    Group {
        /// The tab's id.
        tab: String,
        /// The group's id.
        group: String,
    },
    /// The quick-access toolbar.
    Qat,
    /// The trailing controls at the right of the tab-strip row.
    ///
    /// Its own site rather than sharing [`Self::Qat`], because a message
    /// naming the wrong end of the row sends the reader to the wrong line of
    /// the manifest — and the two regions are three fields apart in a file
    /// that is mostly tabs.
    Trailing,
    /// A group in the left rail.
    ///
    /// Carries the group id rather than only saying "the rail", for
    /// [`Self::Group`]'s reason: the rail is three or four groups deep and a
    /// message that named only the region would send the reader to a strip
    /// rather than to a line.
    Rail {
        /// The rail group's id.
        group: String,
    },
    /// A key binding.
    Keymap {
        /// The chord, e.g. `"Ctrl+E"`.
        chord: String,
    },
    /// A mode's tab list.
    Mode {
        /// The mode's id.
        mode: String,
    },
}

impl std::fmt::Display for Site {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Site::Document => f.write_str("the manifest"),
            Site::Group { tab, group } => write!(f, "tab `{tab}` group `{group}`"),
            Site::Qat => f.write_str("the quick-access toolbar"),
            Site::Trailing => f.write_str("the trailing controls"),
            Site::Rail { group } => write!(f, "rail group `{group}`"),
            Site::Keymap { chord } => write!(f, "key binding `{chord}`"),
            Site::Mode { mode } => write!(f, "mode `{mode}`"),
        }
    }
}

/// Why a manifest was refused.
///
/// Every variant carries the offending identifiers as fields. See this
/// module's header on why that is a requirement rather than a courtesy.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ManifestError {
    /// The document declares a schema this build does not understand.
    #[error(
        "manifest schema {found} is newer than this build understands ({supported}); \
         a field it does not know may be the one that changes what the rest means"
    )]
    UnsupportedSchema {
        /// The schema the document declared.
        found: u32,
        /// The newest schema this build supports.
        supported: u32,
    },
    /// Two tabs share an id.
    #[error("tab id `{id}` is used more than once; a tab id must resolve to one tab")]
    DuplicateTabId {
        /// The id used twice.
        id: String,
    },
    /// Two groups on one tab share an id.
    #[error("tab `{tab}` has two groups with id `{group}`")]
    DuplicateGroupId {
        /// The tab holding both.
        tab: String,
        /// The id used twice.
        group: String,
    },
    /// Two modes share an id.
    #[error("mode id `{id}` is used more than once")]
    DuplicateModeId {
        /// The id used twice.
        id: String,
    },
    /// A tab has no label to render.
    #[error("tab `{tab}` has no label; a complete manifest must be renderable")]
    MissingTabLabel {
        /// The tab with no label.
        tab: String,
    },
    /// A tab has no `groups` key at all, which means "a reference to this
    /// tab", not "an empty tab".
    #[error(
        "tab `{tab}` states no groups; that is a layer's reference to a tab, \
         not a complete tab — write `groups: []` for a deliberately empty one"
    )]
    MissingTabGroups {
        /// The tab with no groups key.
        tab: String,
    },
    /// A group has no caption.
    #[error(
        "group `{group}` on tab `{tab}` has no caption; an uncaptioned band is a row \
         of controls whose relationship the operator has to infer"
    )]
    MissingGroupCaption {
        /// The tab holding the group.
        tab: String,
        /// The uncaptioned group.
        group: String,
    },
    /// A mode has no label.
    #[error("mode `{mode}` has no label")]
    MissingModeLabel {
        /// The mode with no label.
        mode: String,
    },
    /// **The one-command-one-tab rule.**
    #[error(
        "command `{command}` appears on two tabs, `{first_tab}` and `{second_tab}`; \
         a command may appear on exactly one tab (the QAT and status bar may mirror it)"
    )]
    CommandOnTwoTabs {
        /// The command claimed twice.
        command: String,
        /// The tab that claimed it first, in document order.
        first_tab: String,
        /// The tab that claimed it again.
        second_tab: String,
    },
    /// A mode names a tab that does not exist.
    #[error("mode `{mode}` names tab `{tab}`, which does not exist")]
    UnknownTabInMode {
        /// The mode.
        mode: String,
        /// The tab it named.
        tab: String,
    },
    /// The quick-access toolbar lists the same command twice.
    #[error("the quick-access toolbar lists `{command}` twice")]
    DuplicateQatEntry {
        /// The duplicated command id.
        command: String,
    },
    /// A referenced command is not registered.
    #[error("`{command}` is not a registered command (referenced by {site})")]
    UnknownCommand {
        /// The unregistered id.
        command: String,
        /// Where it was referenced.
        site: Site,
    },
    /// The document could not be parsed.
    #[error("the manifest could not be parsed: {0}")]
    Parse(#[from] ron::error::SpannedError),
    /// The document could not be serialized.
    #[error("the manifest could not be written: {0}")]
    Serialize(#[from] ron::Error),
}

impl Shell {
    /// Check everything that can be checked without a command registry.
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.schema > Self::SCHEMA {
            return Err(ManifestError::UnsupportedSchema {
                found: self.schema,
                supported: Self::SCHEMA,
            });
        }

        let mut seen_tabs: BTreeMap<&str, ()> = BTreeMap::new();
        for tab in self.all_tabs() {
            if seen_tabs.insert(tab.id.as_str(), ()).is_some() {
                return Err(ManifestError::DuplicateTabId { id: tab.id.clone() });
            }
            if tab.label.as_ref().is_none_or(|l| l.trim().is_empty()) {
                return Err(ManifestError::MissingTabLabel {
                    tab: tab.id.clone(),
                });
            }
            if tab.groups.is_none() {
                return Err(ManifestError::MissingTabGroups {
                    tab: tab.id.clone(),
                });
            }
            let mut seen_groups: BTreeMap<&str, ()> = BTreeMap::new();
            for group in tab.groups() {
                if seen_groups.insert(group.id.as_str(), ()).is_some() {
                    return Err(ManifestError::DuplicateGroupId {
                        tab: tab.id.clone(),
                        group: group.id.clone(),
                    });
                }
                if group.caption.as_ref().is_none_or(|c| c.trim().is_empty()) {
                    return Err(ManifestError::MissingGroupCaption {
                        tab: tab.id.clone(),
                        group: group.id.clone(),
                    });
                }
            }
        }

        let mut seen_modes: BTreeMap<&str, ()> = BTreeMap::new();
        for mode in self.modes() {
            if seen_modes.insert(mode.id.as_str(), ()).is_some() {
                return Err(ManifestError::DuplicateModeId {
                    id: mode.id.clone(),
                });
            }
            if mode.label.as_ref().is_none_or(|l| l.trim().is_empty()) {
                return Err(ManifestError::MissingModeLabel {
                    mode: mode.id.clone(),
                });
            }
            for tab in mode.tabs() {
                // Ordinary tabs only. A contextual tab's presence is
                // decided by application state, so naming one in a mode's
                // fixed tab set is a category error rather than a typo.
                if !self.tabs().iter().any(|t| &t.id == tab) {
                    return Err(ManifestError::UnknownTabInMode {
                        mode: mode.id.clone(),
                        tab: tab.clone(),
                    });
                }
            }
        }

        if let Some(qat) = &self.qat {
            let mut seen: BTreeMap<&str, ()> = BTreeMap::new();
            for id in qat.ids() {
                if seen.insert(id.as_str(), ()).is_some() {
                    return Err(ManifestError::DuplicateQatEntry {
                        command: id.clone(),
                    });
                }
            }
        }

        self.check_one_command_one_tab()
    }

    /// **A command may appear on exactly one tab.**
    fn check_one_command_one_tab(&self) -> Result<(), ManifestError> {
        let mut owner: BTreeMap<&str, &str> = BTreeMap::new();
        for tab in self.all_tabs() {
            for group in tab.groups() {
                for item in group.items() {
                    let Item::Command { id, .. } = item else {
                        continue;
                    };
                    if let Some(first) = owner.insert(id.as_str(), tab.id.as_str()) {
                        return Err(ManifestError::CommandOnTwoTabs {
                            command: id.clone(),
                            first_tab: first.to_owned(),
                            second_tab: tab.id.clone(),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// [`Shell::validate`], plus: every referenced command is registered.
    pub fn validate_against(&self, catalog: &dyn CommandCatalog) -> Result<(), ManifestError> {
        self.validate()?;
        for (site, command) in self.command_references() {
            if !catalog.contains(&command) {
                return Err(ManifestError::UnknownCommand { command, site });
            }
        }
        Ok(())
    }

    /// Every command id this manifest mentions, with where it was
    /// mentioned.
    #[must_use]
    pub fn command_references(&self) -> Vec<(Site, String)> {
        let mut out = Vec::new();
        for tab in self.all_tabs() {
            for group in tab.groups() {
                for item in group.items() {
                    if let Item::Command { id, .. } = item {
                        out.push((
                            Site::Group {
                                tab: tab.id.clone(),
                                group: group.id.clone(),
                            },
                            id.clone(),
                        ));
                    }
                }
            }
        }
        if let Some(qat) = &self.qat {
            for id in qat.ids() {
                out.push((Site::Qat, id.clone()));
            }
        }
        // The trailing region is walked, so a typo in it is a start-up
        // failure exactly as a typo in the QAT is. It is deliberately NOT
        // treated as a place where an unregistered id means "this build does
        // not have that capability": conditional *presence* is expressed by
        // `Item::Command::visible_when`, which is evaluated every frame, and
        // an id that names nothing is a mistake in either region.
        if let Some(trailing) = &self.trailing {
            for item in trailing.items() {
                if let Item::Command { id, .. } = item {
                    out.push((Site::Trailing, id.clone()));
                }
            }
        }
        // The rail is walked for the trailing region's reason: a
        // typo in it is a start-up failure rather than a control that quietly
        // is not there. It matters more here than anywhere else on the
        // document, because the rail is PERMANENT chrome — a mis-typed id in a
        // ribbon group costs one absent button on one tab, and a mis-typed id
        // here costs a hole in a strip that is on screen for the whole session.
        if let Some(rail) = &self.rail {
            for group in rail.groups() {
                for item in &group.items {
                    if let Item::Command { id, .. } = item {
                        out.push((
                            Site::Rail {
                                group: group.id.clone(),
                            },
                            id.clone(),
                        ));
                    }
                }
            }
        }
        if let Some(keymap) = &self.keymap {
            for (chord, command) in keymap.iter() {
                out.push((
                    Site::Keymap {
                        chord: chord.to_owned(),
                    },
                    command.to_owned(),
                ));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::tests::sketch;
    use crate::manifest::{AnyCommand, Group, Item, Mode, Shell, Tab};

    /// A catalog holding exactly the ids the sketch manifest uses.
    struct Known(&'static [&'static str]);
    impl CommandCatalog for Known {
        fn contains(&self, id: &str) -> bool {
            self.0.contains(&id)
        }
    }

    const SKETCH_COMMANDS: &[&str] = &[
        "file.open",
        "file.save_copy",
        "view.single",
        "view.continuous",
        "view.fullscreen",
        "format.colour",
        "edit.text",
    ];

    /// The shared fixture is itself valid, which is what makes every
    /// negative test below mean something. A fixture that failed for an
    /// unrelated reason would make each of them pass vacuously.
    #[test]
    fn the_sketch_manifest_is_valid() {
        sketch().validate().expect("the documented sketch is valid");
        sketch()
            .validate_against(&Known(SKETCH_COMMANDS))
            .expect("every id in the sketch is registered");
    }

    /// **One command may appear on at most one tab, and the error names it
    /// and both tabs.**
    ///
    /// Running against the merged manifest rather than at compile time is the
    /// point: the merged manifest is the ribbon the operator ends up with.
    #[test]
    fn a_command_on_two_tabs_is_refused_and_all_three_names_appear() {
        let shell = sketch().with_tab(Tab::new("tools", "Tools").with_groups([
            Group::new("misc", "Misc").with_items([Item::command("view.fullscreen")]),
        ]));
        let err = shell
            .validate()
            .expect_err("a command on two tabs is refused");
        assert_eq!(
            err,
            ManifestError::CommandOnTwoTabs {
                command: "view.fullscreen".to_owned(),
                first_tab: "view".to_owned(),
                second_tab: "tools".to_owned(),
            }
        );
        let text = err.to_string();
        for needle in ["view.fullscreen", "`view`", "`tools`"] {
            assert!(text.contains(needle), "message must name {needle}: {text}");
        }
    }

    /// The same command twice **in one group** is the same violation.
    #[test]
    fn a_command_twice_on_the_same_tab_is_also_refused() {
        let shell = Shell::new().with_tab(
            Tab::new("view", "View").with_groups([Group::new("zoom", "Zoom")
                .with_items([Item::command("view.fit"), Item::command("view.fit")])]),
        );
        assert!(matches!(
            shell.validate(),
            Err(ManifestError::CommandOnTwoTabs { .. })
        ));
    }

    /// **The QAT and the keymap may mirror a command that is on a tab.**
    #[test]
    fn the_qat_and_keymap_may_mirror_a_tab_command() {
        let shell = sketch();
        assert!(
            shell
                .qat
                .as_ref()
                .is_some_and(|q| q.ids().iter().any(|id| id == "file.open")),
            "the fixture must actually mirror a tab command, or this test is vacuous"
        );
        assert!(
            shell
                .all_tabs()
                .flat_map(Tab::groups)
                .flat_map(Group::items)
                .any(|i| i.command_id() == Some("file.open")),
            "…and that command must also be on a tab"
        );
        shell.validate().expect("mirroring is explicitly permitted");
    }

    /// **An unregistered command id fails validation and is named, along
    /// with where it was referenced.**
    #[test]
    fn an_unregistered_command_is_refused_with_its_id_and_site() {
        let shell = sketch().with_binding("Ctrl+Q", "app.quit");
        let err = shell
            .validate_against(&Known(SKETCH_COMMANDS))
            .expect_err("`app.quit` is not registered");
        assert_eq!(
            err,
            ManifestError::UnknownCommand {
                command: "app.quit".to_owned(),
                site: Site::Keymap {
                    chord: "Ctrl+Q".to_owned()
                },
            }
        );
        let text = err.to_string();
        assert!(text.contains("app.quit"), "{text}");
        assert!(text.contains("Ctrl+Q"), "{text}");
    }

    /// `AnyCommand` accepts anything, so `validate_against` degrades to
    /// `validate` for tooling with no registry.
    #[test]
    fn the_permissive_catalog_accepts_every_id() {
        sketch()
            .with_binding("Ctrl+Q", "app.quit")
            .validate_against(&AnyCommand)
            .expect("the permissive catalog checks structure only");
    }

    /// Structural rules: duplicate ids, missing labels, missing captions.
    #[test]
    fn structural_defects_are_each_named() {
        let dup_tab = Shell::new()
            .with_tab(Tab::new("view", "View"))
            .with_tab(Tab::new("view", "View again"));
        assert_eq!(
            dup_tab.validate(),
            Err(ManifestError::DuplicateTabId {
                id: "view".to_owned()
            })
        );

        // A contextual tab shares the tab namespace: `format` cannot be
        // both, or a mode naming it would resolve to two things.
        let clash = Shell::new()
            .with_tab(Tab::new("format", "Format"))
            .with_contextual_tab(Tab::new("format", "Format"));
        assert_eq!(
            clash.validate(),
            Err(ManifestError::DuplicateTabId {
                id: "format".to_owned()
            })
        );

        let no_label = Shell::new().with_tab(Tab::patch("view"));
        assert_eq!(
            no_label.validate(),
            Err(ManifestError::MissingTabLabel {
                tab: "view".to_owned()
            })
        );

        // A blank label is a missing label. An operator who typed a space
        // gets told what is wrong rather than an invisible tab.
        let blank_label = Shell::new().with_tab(Tab::new("view", "   "));
        assert_eq!(
            blank_label.validate(),
            Err(ManifestError::MissingTabLabel {
                tab: "view".to_owned()
            })
        );

        let no_groups = Shell::new().with_tab(Tab {
            id: "view".to_owned(),
            label: Some("View".to_owned()),
            ..Tab::default()
        });
        assert_eq!(
            no_groups.validate(),
            Err(ManifestError::MissingTabGroups {
                tab: "view".to_owned()
            })
        );

        let no_caption =
            Shell::new().with_tab(Tab::new("view", "View").with_groups([Group::patch("zoom")]));
        assert_eq!(
            no_caption.validate(),
            Err(ManifestError::MissingGroupCaption {
                tab: "view".to_owned(),
                group: "zoom".to_owned(),
            })
        );

        let dup_group = Shell::new().with_tab(
            Tab::new("view", "View")
                .with_groups([Group::new("zoom", "Zoom"), Group::new("zoom", "Zoom 2")]),
        );
        assert_eq!(
            dup_group.validate(),
            Err(ManifestError::DuplicateGroupId {
                tab: "view".to_owned(),
                group: "zoom".to_owned(),
            })
        );
    }

    /// A mode naming a tab that does not exist is refused — and a mode
    /// naming a *contextual* tab is refused too, because a contextual
    /// tab's presence is decided by application state rather than by the
    /// mode's fixed tab set.
    #[test]
    fn a_mode_may_only_name_ordinary_tabs_that_exist() {
        let missing = Shell::new()
            .with_tab(Tab::new("view", "View"))
            .with_mode(Mode::new("read", "Read", ["view", "pages"]));
        assert_eq!(
            missing.validate(),
            Err(ManifestError::UnknownTabInMode {
                mode: "read".to_owned(),
                tab: "pages".to_owned(),
            })
        );

        let contextual = Shell::new()
            .with_tab(Tab::new("view", "View"))
            .with_contextual_tab(Tab::new("format", "Format").with_visible_when("selection.any"))
            .with_mode(Mode::new("read", "Read", ["view", "format"]));
        assert_eq!(
            contextual.validate(),
            Err(ManifestError::UnknownTabInMode {
                mode: "read".to_owned(),
                tab: "format".to_owned(),
            })
        );
    }

    /// A schema from a newer build is refused rather than half-read.
    #[test]
    fn a_future_schema_is_refused() {
        let future = Shell {
            schema: Shell::SCHEMA + 1,
            ..Shell::new()
        };
        assert_eq!(
            future.validate(),
            Err(ManifestError::UnsupportedSchema {
                found: Shell::SCHEMA + 1,
                supported: Shell::SCHEMA,
            })
        );
    }

    /// The quick-access toolbar may mirror a tab command, but not itself.
    #[test]
    fn the_qat_may_not_list_one_command_twice() {
        let shell = Shell::new().with_qat(["file.open", "file.open"]);
        assert_eq!(
            shell.validate(),
            Err(ManifestError::DuplicateQatEntry {
                command: "file.open".to_owned()
            })
        );
    }

    /// `command_references` finds every reference, in document order, so
    /// nothing escapes `validate_against`.
    #[test]
    fn every_reference_site_is_walked() {
        let refs = sketch().command_references();
        let sites: Vec<&Site> = refs.iter().map(|(s, _)| s).collect();
        assert!(sites.iter().any(|s| matches!(s, Site::Group { .. })));
        assert!(sites.iter().any(|s| matches!(s, Site::Qat)));
        assert!(sites.iter().any(|s| matches!(s, Site::Keymap { .. })));
        assert!(
            refs.iter().any(
                |(s, c)| matches!(s, Site::Group { tab, .. } if tab == "format")
                    && c == "format.colour"
            ),
            "a contextual tab's commands must be walked too"
        );
    }
}
