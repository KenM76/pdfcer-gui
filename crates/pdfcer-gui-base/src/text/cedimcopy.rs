//! # `text::cedimcopy` — what a page copy did with the ce dimensions on its pages
//!
//! Every page-copy route (extract, split, merge files, insert, merge document)
//! reports how many ce dimensions came across still editable, how many came
//! across as plain line annotations, and what happened to their ce dimension
//! groups: arrived as groups of their own, joined a group of the same name and
//! settings already there, or arrived renamed because the name was taken by a
//! group of other settings. [`CeDimCopy`] is that report in one shape and
//! [`copied`] is the receipt's sentence for it; nothing is said for a copy
//! with no ce dimensions. `AssembleReport::ce_dimensions_left` is not said: a
//! ce dimension whose page was not taken is the copy doing what was asked.

use pdfcer_core::edit::{InsertOutcome, MergeOutcome};
use pdfcer_core::pageops::AssembleReport;

/// What one page copy did with ce dimensions and their groups.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CeDimCopy {
    /// ce dimensions that arrived still editable.
    pub carried: usize,
    /// ce dimensions that arrived as plain line annotations.
    pub as_lines: usize,
    /// ce dimension groups that arrived as groups of their own.
    pub groups_carried: usize,
    /// ce dimension groups that joined a group of the same name and settings.
    pub groups_matched: usize,
    /// Of `groups_carried`, those given a numbered name.
    pub groups_renamed: usize,
    /// Whether `as_lines` is because a newer pdfcer wrote the record.
    pub sidecar_newer: bool,
}

impl CeDimCopy {
    /// Sum two copies' reports, for a gesture that runs several.
    pub fn add(&mut self, more: Self) {
        self.carried += more.carried;
        self.as_lines += more.as_lines;
        self.groups_carried += more.groups_carried;
        self.groups_matched += more.groups_matched;
        self.groups_renamed += more.groups_renamed;
        self.sidecar_newer |= more.sidecar_newer;
    }
}

impl From<&InsertOutcome> for CeDimCopy {
    fn from(o: &InsertOutcome) -> Self {
        Self {
            carried: o.ce_dimensions_carried,
            as_lines: o.ce_dimensions_as_lines,
            groups_carried: o.ce_dimension_groups_carried,
            groups_matched: o.ce_dimension_groups_matched,
            groups_renamed: o.ce_dimension_groups_renamed,
            sidecar_newer: o.ce_dimension_sidecar_newer,
        }
    }
}

impl From<&MergeOutcome> for CeDimCopy {
    fn from(o: &MergeOutcome) -> Self {
        Self {
            carried: o.ce_dimensions_carried,
            as_lines: o.ce_dimensions_as_lines,
            groups_carried: o.ce_dimension_groups_carried,
            groups_matched: o.ce_dimension_groups_matched,
            groups_renamed: o.ce_dimension_groups_renamed,
            sidecar_newer: o.ce_dimension_sidecar_newer,
        }
    }
}

impl From<&AssembleReport> for CeDimCopy {
    fn from(r: &AssembleReport) -> Self {
        Self {
            carried: r.ce_dimensions_carried,
            as_lines: r.ce_dimensions_as_lines.iter().sum(),
            groups_carried: r.ce_dimension_groups_carried,
            groups_matched: r.ce_dimension_groups_matched,
            groups_renamed: r.ce_dimension_groups_renamed,
            sidecar_newer: r.ce_dimension_sidecar_newer,
        }
    }
}

/// `n` and the noun, singular or plural.
fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// The editable half: how many arrived, and where their groups went.
fn carried_sentence(c: &CeDimCopy) -> Option<String> {
    if c.carried == 0 {
        return None;
    }
    let mut groups = Vec::new();
    if c.groups_carried > 0 {
        let own = count(
            c.groups_carried,
            "ce dimension group came as a group of its own",
            "ce dimension groups came as groups of their own",
        );
        groups.push(match c.groups_renamed {
            0 => own,
            1 => format!(
                "{own} (one given a number after its name, because a group of other settings \
                 already had it)"
            ),
            r => format!(
                "{own} ({r} of them given a number after their names, because groups of other \
                 settings already had them)"
            ),
        });
    }
    if c.groups_matched > 0 {
        groups.push(count(
            c.groups_matched,
            "joined the group of the same name and settings already there",
            "joined groups of the same name and settings already there",
        ));
    }
    let head = format!(
        "{} came across still editable",
        count(c.carried, "ce dimension", "ce dimensions")
    );
    Some(if groups.is_empty() {
        format!("{head}.")
    } else {
        format!("{head}: {}.", groups.join("; "))
    })
}

/// The lost half: how many arrived as plain lines, and why when known.
fn as_lines_sentence(c: &CeDimCopy) -> Option<String> {
    if c.as_lines == 0 {
        return None;
    }
    let why = if c.sidecar_newer {
        ", because they were saved by a newer version of pdfcer"
    } else {
        ""
    };
    Some(format!(
        "{} came across as plain lines{why}: they look the same, but they cannot be edited as \
         ce dimensions and belong to no group.",
        count(c.as_lines, "ce dimension", "ce dimensions")
    ))
}

/// The receipt's sentence for one copy, or `None` when it held no ce
/// dimensions.
#[must_use]
pub fn copied(c: &CeDimCopy) -> Option<String> {
    let parts: Vec<String> = [carried_sentence(c), as_lines_sentence(c)]
        .into_iter()
        .flatten()
        .collect();
    (!parts.is_empty()).then(|| parts.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_said_without_ce_dimensions() {
        assert_eq!(copied(&CeDimCopy::default()), None);
    }

    #[test]
    fn every_count_reaches_the_sentence() {
        let c = CeDimCopy {
            carried: 5,
            as_lines: 2,
            groups_carried: 3,
            groups_matched: 1,
            groups_renamed: 1,
            sidecar_newer: true,
        };
        let said = copied(&c).unwrap();
        assert!(
            said.starts_with("5 ce dimensions came across still editable"),
            "{said}"
        );
        assert!(
            said.contains("3 ce dimension groups came as groups of their own"),
            "{said}"
        );
        assert!(said.contains("one given a number"), "{said}");
        assert!(said.contains("1 joined the group"), "{said}");
        assert!(
            said.contains("2 ce dimensions came across as plain lines, because"),
            "{said}"
        );
    }

    #[test]
    fn one_editable_ce_dimension_with_no_group_news() {
        let c = CeDimCopy {
            carried: 1,
            ..CeDimCopy::default()
        };
        assert_eq!(
            copied(&c).as_deref(),
            Some("1 ce dimension came across still editable.")
        );
    }

    #[test]
    fn every_sentence_names_ce_dimensions_never_bare() {
        let c = CeDimCopy {
            carried: 2,
            as_lines: 1,
            groups_carried: 2,
            groups_matched: 2,
            groups_renamed: 2,
            sidecar_newer: false,
        };
        let said = copied(&c).unwrap();
        let bare = said.replace("ce dimension", "");
        assert!(!bare.contains("dimension"), "{said}");
    }
}
