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
