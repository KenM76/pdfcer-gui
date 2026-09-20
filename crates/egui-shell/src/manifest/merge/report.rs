//! What a merge could not carry across, and why it is said out loud.
//!
//! An absence is indistinguishable from a thing that ran and produced
//! nothing, so a silently dropped item presents as **the application
//! having removed a control** rather than as a stale reference in a file
//! the operator can fix. The operator then attributes it to the update
//! they just installed, and there is nothing anywhere that says otherwise.
//!
//! So every drop produces a [`Skip`] carrying the layer, the site and the
//! reason, and the application is expected to surface them.
//! [`MergeReport`] is returned by value — something the caller must deal
//! with, not a side effect it may forget.
//!
//! The vocabulary lives beside the algorithm rather than inside it because
//! it crosses the crate boundary: an application matches on [`SkipReason`]
//! to decide what to tell its operator, where [`super::merge`] itself is an
//! implementation the application never names.

use crate::manifest::validate::Site;

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
    /// A deliberately different reason from [`Self::UnknownCommand`], so that
    /// the two can never be confused in a log: this one says *this build does
    /// not include that*, the other says *someone made a mistake*. Collapsing
    /// them would make a lite build indistinguishable from a typo.
    ///
    /// Raised when an [`Item::Command`](crate::manifest::Item::Command) carrying a
    /// [`capability`](crate::manifest::Item::Command::capability) names a command the
    /// registry does not hold. It is the one reason also raised against the
    /// **built-in** layer; the pruning pass in the parent module carries why
    /// that is not a weakening of the rule beside it.
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
/// Returned by value so the caller must deal with it: returning the rejects
/// alongside the result makes them a value that has to be handled rather
/// than a side effect that can be forgotten.
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

    pub(super) fn push(&mut self, layer: Layer, site: Site, reason: SkipReason) {
        self.skips.push(Skip {
            layer,
            site,
            reason,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A site with both ids filled, so a sentence that drops one is visible.
    fn site() -> Site {
        Site::Group {
            tab: "review".to_owned(),
            group: "markup".to_owned(),
        }
    }

    fn unknown_command(layer: Layer) -> Skip {
        Skip {
            layer,
            site: site(),
            reason: SkipReason::UnknownCommand {
                command: "markup.chisel".to_owned(),
            },
        }
    }

    /// The same command id and the same site as [`unknown_command`], so the
    /// only thing that can make the two sentences differ is the reason. Given
    /// distinct ids they would differ whatever the wording said, and the test
    /// that compares them would pass on a pair that had collapsed.
    fn capability_absent(layer: Layer) -> Skip {
        Skip {
            layer,
            site: site(),
            reason: SkipReason::CapabilityAbsent {
                capability: "ocr".to_owned(),
                command: "markup.chisel".to_owned(),
            },
        }
    }

    fn unknown_tab(layer: Layer) -> Skip {
        Skip {
            layer,
            site: Site::Mode {
                mode: "read".to_owned(),
            },
            reason: SkipReason::UnknownTab {
                tab: "drafting".to_owned(),
            },
        }
    }

    fn unsupported_schema(layer: Layer) -> Skip {
        Skip {
            layer,
            site: Site::Document,
            reason: SkipReason::UnsupportedSchema {
                found: 4,
                supported: 2,
            },
        }
    }

    fn one_of_each(layer: Layer) -> [Skip; 4] {
        [
            unknown_command(layer),
            capability_absent(layer),
            unknown_tab(layer),
            unsupported_schema(layer),
        ]
    }

    const LAYERS: [Layer; 3] = [Layer::BuiltIn, Layer::AppOverride, Layer::Operator];

    #[test]
    fn every_sentence_names_the_layer_the_item_came_from() {
        for layer in LAYERS {
            for skip in one_of_each(layer) {
                let said = skip.to_string();
                assert!(
                    said.contains(&layer.to_string()),
                    "a skip that does not say WHICH file to edit sends the operator to three \
                     of them. layer {layer:?}, said: {said}"
                );
            }
        }
    }

    #[test]
    fn a_dropped_item_says_where_it_was() {
        for skip in [
            unknown_command(Layer::Operator),
            capability_absent(Layer::Operator),
            unknown_tab(Layer::Operator),
        ] {
            let where_it_was = skip.site.to_string();
            let said = skip.to_string();
            assert!(
                said.contains(&where_it_was),
                "the layer names a file and the site names the line in it; without the site the \
                 operator has a sentence they cannot act on. Wanted {where_it_was}, said: {said}"
            );
        }
    }

    #[test]
    fn a_capability_this_build_lacks_does_not_read_as_a_mistake() {
        let absent = capability_absent(Layer::Operator).to_string();
        let unknown = unknown_command(Layer::Operator).to_string();
        assert_ne!(absent, unknown);
        assert!(
            absent.contains("does not include"),
            "the operator's file is correct and this build is the narrow one; a sentence that \
             does not say so reads as a typo they must go and find. Got: {absent}"
        );
        assert!(
            absent.contains("ocr"),
            "naming the capability is what turns the sentence into an action — install it, or \
             delete the entry. Got: {absent}"
        );
        assert!(
            !unknown.contains("does not include"),
            "the opposite failure: a genuine typo worded as a missing capability sends the \
             operator looking for an installer. Got: {unknown}"
        );
    }

    #[test]
    fn only_the_schema_skip_says_a_whole_layer_was_dropped() {
        let schema = unsupported_schema(Layer::AppOverride).to_string();
        assert!(
            schema.contains("that layer was not applied"),
            "an unsupported schema drops EVERY customization in the file, not one item, and a \
             sentence that says otherwise understates it by however many the file held. \
             Got: {schema}"
        );
        for skip in [
            unknown_command(Layer::AppOverride),
            capability_absent(Layer::AppOverride),
            unknown_tab(Layer::AppOverride),
        ] {
            let said = skip.to_string();
            assert!(
                !said.contains("that layer was not applied"),
                "the other three drop one item; saying the layer went would send the operator \
                 looking for customizations that are still there. Got: {said}"
            );
        }
    }

    #[test]
    fn the_schema_sentence_carries_both_version_numbers() {
        let said = unsupported_schema(Layer::Operator).to_string();
        assert!(
            said.contains("schema 4"),
            "without the number the file declared, the operator cannot tell which of several \
             files is the new one. Got: {said}"
        );
        assert!(
            said.contains("(2)"),
            "without the number this build supports, the operator cannot tell whether to update \
             the application or the file — and the two remedies are opposite. Got: {said}"
        );
    }

    #[test]
    fn no_two_reasons_share_a_sentence() {
        let said: Vec<String> = one_of_each(Layer::Operator)
            .iter()
            .map(ToString::to_string)
            .collect();
        for (i, a) in said.iter().enumerate() {
            for b in said.iter().skip(i + 1) {
                assert_ne!(
                    a, b,
                    "two reasons reading alike is one the operator cannot act on"
                );
            }
        }
    }

    #[test]
    fn the_three_layers_are_told_apart_in_the_operators_words() {
        let said: Vec<String> = LAYERS.iter().map(ToString::to_string).collect();
        assert_ne!(said[0], said[1]);
        assert_ne!(said[1], said[2]);
        assert_ne!(said[0], said[2]);
        assert!(
            said[2].contains("your"),
            "the operator's own file is the one he can edit, and it is named as his. Got: {}",
            said[2]
        );
        for one in &said {
            assert!(
                !one.contains("Layer") && !one.contains("::"),
                "a debug spelling reaching a sentence is the vocabulary leaking. Got: {one}"
            );
        }
    }

    #[test]
    fn a_report_counts_what_it_holds_and_keeps_the_order() {
        let mut report = MergeReport::default();
        assert!(report.is_empty());
        assert_eq!(report.len(), 0);
        assert!(report.skips().is_empty());

        report.push(
            Layer::Operator,
            site(),
            SkipReason::UnknownCommand {
                command: "markup.chisel".to_owned(),
            },
        );
        report.push(
            Layer::AppOverride,
            Site::Document,
            SkipReason::UnsupportedSchema {
                found: 4,
                supported: 2,
            },
        );

        assert!(!report.is_empty());
        assert_eq!(report.len(), 2);
        assert_eq!(report.skips().len(), report.len());
        assert_eq!(
            report.skips()[0].layer,
            Layer::Operator,
            "the skips are in the order they occurred, so a caller can read them as a story of \
             the merge rather than as a set"
        );
        assert_eq!(report.skips()[1].layer, Layer::AppOverride);
    }
}
