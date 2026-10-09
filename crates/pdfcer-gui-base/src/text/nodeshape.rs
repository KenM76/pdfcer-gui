//! Status-line copy for Format ▸ Nodes: what a press could not do at some of
//! the selected nodes.

/// Why some selected nodes were left alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skipped {
    /// Smooth or symmetric at an end of an open line, which has one side.
    EndNode,
    /// The last node of an open line, which has no segment after it.
    NoSegment,
    /// Any other refusal.
    Other,
}

/// `count` selected nodes were skipped, and why. Said only when the press
/// changed the others; a press that changed nothing shows the engine's
/// refusal instead.
#[must_use]
pub fn nodes_skipped(count: usize, why: Skipped) -> String {
    let which = if count == 1 {
        "One selected node was left as it was".to_owned()
    } else {
        format!("{count} selected nodes were left as they were")
    };
    let because = match why {
        Skipped::EndNode => {
            "an end of an open line has only one side, so it cannot be smooth or symmetric"
        }
        Skipped::NoSegment => "the last node of an open line has no segment after it",
        Skipped::Other => "the change could not be made there",
    };
    format!("{which}: {because}.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_and_many_read_as_sentences() {
        assert_eq!(
            nodes_skipped(1, Skipped::NoSegment),
            "One selected node was left as it was: the last node of an open line has no \
             segment after it."
        );
        assert!(nodes_skipped(3, Skipped::EndNode).starts_with("3 selected nodes were left"));
    }
}
