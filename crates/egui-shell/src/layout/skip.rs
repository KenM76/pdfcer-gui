//! What a layout load could not carry across, and where.
//!
//! # The shape, and why it is this shape
//!
//! Deliberately, structurally the same as [`crate::manifest::Skip`]: a
//! **site**, a **reason**, both as data, with `Display` for diagnostics
//! and neither of them operator-facing copy. That type's own
//! documentation makes the argument in full, and it applies here without
//! amendment:
//!
//! > A structured value rather than a message, deliberately. The shell
//! > has no business deciding how another application words a note to its
//! > operator, and an application that wants to offer "remove this stale
//! > entry from your file" needs the id, not a sentence containing it.
//!
//! An application that already renders manifest skips in a status
//! surface can render these with the same code path, which is most of the
//! point of matching the shape.
//!
//! # Why a dropped item is disclosed rather than silent
//!
//! [`crate::verify`] states the rule a dropped step has to obey — *"An
//! absent trace line is indistinguishable from a step that ran and
//! produced no output"* — and it is worth repeating here, because the
//! same silence is *quieter* for a layout than for a ribbon.
//!
//! A silently dropped ribbon item is a missing button, which an operator
//! notices. A silently dropped *panel* is a panel that used to be in the
//! dock and now is not, which an operator experiences as *"the
//! application lost my layout"* — the complaint `MODES_AND_PANELS.md`
//! records against the benchmark product, whose remedy for a bad layout
//! file is *"the documented route is to quit and delete that file"*.
//!
//! So every drop produces a [`LayoutSkip`] naming the site and the
//! reason, [`LoadReport`] is returned **by value** so the caller must
//! deal with it, and nothing is ever repaired quietly on a path where
//! there is an operator to tell.

use crate::dock::{DockSide, PanelId};

/// Where in a layout document a problem was.
///
/// Positional, because the model is positional — see
/// [`crate::dock::model`]'s note on why there are no generated handles to
/// name. A site is enough for an application to say *"the second stack of
/// the left dock's first column"* or to offer to open the file at
/// roughly the right place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutSite {
    /// The document as a whole — a missing file, a parse failure, an
    /// unsupported schema.
    Document,
    /// One named workspace as a whole.
    Workspace {
        /// The workspace's name.
        name: String,
    },
    /// One dock side.
    Side {
        /// Which side.
        side: DockSide,
    },
    /// One column of one side.
    Column {
        /// Which side.
        side: DockSide,
        /// The column's index within the side.
        index: usize,
    },
    /// One stack of one column.
    Stack {
        /// Which side.
        side: DockSide,
        /// The column's index.
        column: usize,
        /// The stack's index within the column.
        index: usize,
    },
    /// One tab of one stack.
    Tab {
        /// Which side.
        side: DockSide,
        /// The column's index.
        column: usize,
        /// The stack's index.
        stack: usize,
        /// The panel the tab named.
        panel: PanelId,
    },
}

impl std::fmt::Display for LayoutSite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutSite::Document => f.write_str("the layout file"),
            LayoutSite::Workspace { name } => write!(f, "workspace `{name}`"),
            LayoutSite::Side { side } => write!(f, "the {side} dock"),
            LayoutSite::Column { side, index } => write!(f, "the {side} dock, column {index}"),
            LayoutSite::Stack {
                side,
                column,
                index,
            } => write!(f, "the {side} dock, column {column}, compartment {index}"),
            LayoutSite::Tab {
                side,
                column,
                stack,
                panel,
            } => write!(
                f,
                "the {side} dock, column {column}, compartment {stack}, tab `{panel}`"
            ),
        }
    }
}

/// Why one item of a layout was dropped.
///
/// Every variant carries the offending value, so an application can act
/// on it — offer to remove the entry, name the capability that is not in
/// this build, or simply log it with enough detail to be actionable.
///
/// `PartialEq` but **not** `Eq`, because [`Self::InvalidSize`] carries the
/// `f32` the file actually said. Carrying it is worth losing `Eq` for: an
/// application that wants to tell the operator *which* number was
/// rejected cannot get it from anywhere else, and a reason that said only
/// "a size was wrong" would be one an operator can do nothing with.
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutSkipReason {
    /// There was no layout file. The built-in default was used.
    ///
    /// **A first run is not a failure**, and this is the reason an
    /// application should be prepared to see once on every fresh profile
    /// and never mention to anybody. It is reported at all because
    /// "no file" and "a file I could not read" are different facts, and
    /// an application that cannot tell them apart cannot diagnose a
    /// permissions problem.
    FileMissing,
    /// The file exists and could not be read — permissions, a device
    /// error, a lock.
    Unreadable {
        /// The operating system's account of it.
        detail: String,
    },
    /// The file's syntax is broken, so nothing in it could be recovered.
    ///
    /// The one genuinely wholesale case, and the module header of
    /// [`super`] says exactly how far the per-item promise extends in the
    /// face of it.
    ParseFailed {
        /// The parser's account of it, including a line and column.
        detail: String,
    },
    /// The document declares a schema this build does not understand.
    ///
    /// Not repaired and not guessed at: a field this build does not know
    /// may be the one that changes what the rest of the file means. The
    /// same posture, and the same wording, as
    /// [`crate::manifest::SkipReason::UnsupportedSchema`].
    UnsupportedSchema {
        /// The schema the document declared.
        found: u32,
        /// The newest schema this build supports.
        supported: u32,
    },
    /// The layout mounts a panel the application does not register.
    ///
    /// **This is the expected, healthy case, not an error.**
    /// `SHELL_FRAMEWORK.md` §7: *"A capability's presence is expressed by
    /// registering its command, and by nothing else."* A panel belonging
    /// to a feature that was compiled out is simply absent from the
    /// registry, and its saved mount is dropped here — with no `#[cfg]`
    /// anywhere in this crate, and with the operator's arrangement of
    /// everything else intact.
    ///
    /// It is also what happens to a panel an application *renamed*, which
    /// is why the reason carries the id: an application that wants to
    /// migrate `"props"` to `"properties"` has everything it needs.
    UnknownPanel {
        /// The id that did not resolve.
        panel: PanelId,
    },
    /// The layout mounts the same panel more than once.
    ///
    /// Two live copies of one surface each have their own scroll position
    /// and their own idea of which tab is active; see
    /// [`crate::dock::DockLayout::normalize`] for the full reasoning. The
    /// first mount is kept.
    DuplicatePanel {
        /// The id that appeared twice.
        panel: PanelId,
    },
    /// A compartment ended up holding no tabs, so it was dropped.
    ///
    /// Usually a consequence of another skip rather than a fault of its
    /// own: a stack whose only panel was unregistered has nothing left to
    /// show. Reported separately because *"that panel is not in this
    /// build"* and *"a compartment vanished"* are two facts an operator
    /// may need to connect.
    EmptyContainer,
    /// A stored number was not a usable size, and was replaced.
    InvalidSize {
        /// What the file said.
        value: f32,
    },
    /// A stack's active-tab index pointed past its last tab.
    ActiveOutOfRange {
        /// The index the file gave.
        was: usize,
        /// How many tabs the stack actually has.
        len: usize,
    },
    /// Two workspaces claimed the same name, or a workspace had none.
    ///
    /// The first one wins, because a later duplicate is most likely the
    /// result of a hand-edit that meant to replace it and did not.
    WorkspaceName {
        /// The offending name, empty if there was none.
        name: String,
    },
}

/// One thing a load could not carry across.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutSkip {
    /// Where in the document it was.
    pub site: LayoutSite,
    /// Why it was dropped.
    pub reason: LayoutSkipReason,
}

impl LayoutSkip {
    /// Build a skip.
    #[must_use]
    pub fn new(site: LayoutSite, reason: LayoutSkipReason) -> Self {
        Self { site, reason }
    }
}

impl std::fmt::Display for LayoutSkip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.reason {
            LayoutSkipReason::FileMissing => {
                write!(
                    f,
                    "{}: no saved layout yet, so the default was used",
                    self.site
                )
            }
            LayoutSkipReason::Unreadable { detail } => write!(
                f,
                "{} could not be read ({detail}), so the default was used",
                self.site
            ),
            LayoutSkipReason::ParseFailed { detail } => write!(
                f,
                "{} is not readable as a layout ({detail}), so the default was used",
                self.site
            ),
            LayoutSkipReason::UnsupportedSchema { found, supported } => write!(
                f,
                "{}: schema {found} is newer than this build understands ({supported}), \
                 so it was not applied",
                self.site
            ),
            LayoutSkipReason::UnknownPanel { panel } => write!(
                f,
                "{}: `{panel}` is not a panel this build offers, so that one tab was \
                 dropped",
                self.site
            ),
            LayoutSkipReason::DuplicatePanel { panel } => write!(
                f,
                "{}: `{panel}` was already docked elsewhere, so the second copy was \
                 dropped",
                self.site
            ),
            LayoutSkipReason::EmptyContainer => write!(
                f,
                "{}: nothing was left to show in it, so it was dropped",
                self.site
            ),
            LayoutSkipReason::InvalidSize { value } => write!(
                f,
                "{}: {value} is not a usable size, so the default was used for it",
                self.site
            ),
            LayoutSkipReason::ActiveOutOfRange { was, len } => write!(
                f,
                "{}: tab {was} was selected but there are only {len}, so the last one \
                 was selected",
                self.site
            ),
            LayoutSkipReason::WorkspaceName { name } if name.is_empty() => {
                write!(f, "{}: a workspace with no name was dropped", self.site)
            }
            LayoutSkipReason::WorkspaceName { name } => write!(
                f,
                "{}: a second workspace called `{name}` was dropped",
                self.site
            ),
        }
    }
}

/// Everything a load had to skip.
///
/// Returned **by value** so the caller must deal with it, which is the
/// same shape and the same argument as [`crate::manifest::MergeReport`]:
/// returning the rejects alongside the result makes them a value the
/// caller must handle instead of a side effect it may forget.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LoadReport {
    skips: Vec<LayoutSkip>,
}

impl LoadReport {
    /// Every skip, in the order it occurred.
    #[must_use]
    pub fn skips(&self) -> &[LayoutSkip] {
        &self.skips
    }

    /// Whether the load carried everything across.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.skips.is_empty()
    }

    /// How many items were skipped.
    #[must_use]
    pub fn len(&self) -> usize {
        self.skips.len()
    }

    /// Whether any skip is one an operator would want to hear about.
    ///
    /// [`LayoutSkipReason::FileMissing`] is the first run of a fresh
    /// profile and is not news. Everything else is a difference between
    /// what was saved and what was restored, which is exactly the class
    /// of fact this project's disclosure convention exists to surface.
    #[must_use]
    pub fn is_noteworthy(&self) -> bool {
        self.skips
            .iter()
            .any(|s| !matches!(s.reason, LayoutSkipReason::FileMissing))
    }

    /// Record a skip.
    pub(crate) fn push(&mut self, site: LayoutSite, reason: LayoutSkipReason) {
        self.skips.push(LayoutSkip::new(site, reason));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every reason renders a sentence that names the offending value —
    /// the property that makes a skip actionable rather than merely
    /// present.
    #[test]
    fn every_reason_names_the_thing_it_dropped() {
        let cases = [
            (
                LayoutSkipReason::UnknownPanel {
                    panel: PanelId::new("signatures"),
                },
                "signatures",
            ),
            (
                LayoutSkipReason::DuplicatePanel {
                    panel: PanelId::new("pages"),
                },
                "pages",
            ),
            (
                LayoutSkipReason::UnsupportedSchema {
                    found: 9,
                    supported: 1,
                },
                "9",
            ),
            (LayoutSkipReason::ActiveOutOfRange { was: 7, len: 2 }, "7"),
            (
                LayoutSkipReason::ParseFailed {
                    detail: "unexpected `}` at 3:1".to_owned(),
                },
                "3:1",
            ),
            (
                LayoutSkipReason::WorkspaceName {
                    name: "Review".to_owned(),
                },
                "Review",
            ),
            (LayoutSkipReason::InvalidSize { value: -4.0 }, "-4"),
        ];
        for (reason, needle) in cases {
            let text = LayoutSkip::new(LayoutSite::Document, reason.clone()).to_string();
            assert!(
                text.contains(needle),
                "{reason:?} rendered as {text:?}, which does not name {needle}"
            );
        }
    }

    /// A site names the compartment precisely enough for an operator to
    /// find it.
    #[test]
    fn a_site_names_the_compartment_it_refers_to() {
        let site = LayoutSite::Tab {
            side: DockSide::Right,
            column: 1,
            stack: 2,
            panel: PanelId::new("layers"),
        };
        let text = site.to_string();
        assert!(text.contains("right"), "{text}");
        assert!(text.contains('1') && text.contains('2'), "{text}");
        assert!(text.contains("layers"), "{text}");
    }

    /// ★ **A first run is not news.**
    ///
    /// An application that surfaced every skip would tell every operator,
    /// on the first launch of a fresh profile, that their layout could
    /// not be restored — from a profile that has never had one. That is
    /// how a disclosure surface trains people to ignore it.
    #[test]
    fn a_missing_file_is_not_noteworthy_but_a_dropped_panel_is() {
        let mut report = LoadReport::default();
        report.push(LayoutSite::Document, LayoutSkipReason::FileMissing);
        assert!(!report.is_empty(), "it is still recorded");
        assert!(!report.is_noteworthy(), "but it is not worth saying");

        report.push(
            LayoutSite::Side {
                side: DockSide::Left,
            },
            LayoutSkipReason::UnknownPanel {
                panel: PanelId::new("ocr"),
            },
        );
        assert!(report.is_noteworthy());
        assert_eq!(report.len(), 2);
    }
}
