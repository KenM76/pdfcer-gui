//! # `text::snapshot` — what the snapshot box says on the status row

use crate::clipboard::snapshot::{SnapshotCopy, Vectors};

/// **What a snapshot copy put on the clipboard**, said on the status row:
/// the picture's size and resolution, whether the vectors went with it, and
/// anything the engine removed or kept that the operator cannot see.
#[must_use]
pub fn copied(copy: &SnapshotCopy) -> String {
    let (dpi, asked) = (copy.dpi, copy.asked_dpi);
    let resolution = if asked > dpi {
        format!("{dpi} dpi (lowered from {asked} dpi, the most a picture this size can hold)")
    } else {
        format!("{dpi} dpi")
    };
    let picture = format!(
        "a {} \u{d7} {} pixel picture at {resolution}",
        copy.width_px, copy.height_px
    );
    let (said, notes) = match &copy.vectors {
        Vectors::Cut {
            glyphs_removed,
            notes,
        } => {
            let edge = match glyphs_removed {
                0 => String::new(),
                1 => " One character crossing its edge was left out.".to_owned(),
                n => format!(" {n} characters crossing its edge were left out."),
            };
            let said = format!(
                "Copied the snapshot box as vectors and as {picture}. Everything outside the \
                 box was cut away.{edge}"
            );
            (said, notes.as_slice())
        }
        Vectors::Withheld { notes } => {
            let said = format!(
                "Copied the snapshot box as {picture} only: some drawing outside the box could \
                 not be cut away, so it was not copied as vectors."
            );
            (said, notes.as_slice())
        }
        Vectors::Refused(why) => {
            let said = format!(
                "Copied the snapshot box as {picture} only: it could not be copied as vectors \
                 ({why})."
            );
            (said, &[][..])
        }
    };
    notes
        .iter()
        .fold(said, |text, note| format!("{text} {note}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn copy(dpi: u32, asked: u32, vectors: Vectors) -> SnapshotCopy {
        SnapshotCopy {
            formats: Vec::new(),
            dpi,
            asked_dpi: asked,
            width_px: 10,
            height_px: 20,
            vectors,
        }
    }

    fn cut(glyphs_removed: u64) -> Vectors {
        Vectors::Cut {
            glyphs_removed,
            notes: Vec::new(),
        }
    }

    #[test]
    fn a_lowered_resolution_names_both_figures() {
        let said = copied(&copy(150, 300, cut(0)));
        assert!(said.contains("150 dpi") && said.contains("300 dpi"));
        assert!(!copied(&copy(300, 300, cut(0))).contains("lowered"));
    }

    #[test]
    fn each_outcome_says_whether_the_vectors_went() {
        assert!(copied(&copy(300, 300, cut(2))).contains("2 characters"));
        assert!(!copied(&copy(300, 300, cut(0))).contains("character"));
        let withheld = Vectors::Withheld {
            notes: vec!["A path could not be cut.".to_owned()],
        };
        let said = copied(&copy(300, 300, withheld));
        assert!(said.contains("not copied as vectors") && said.ends_with("could not be cut."));
        let refused = copied(&copy(300, 300, Vectors::Refused("encrypted".to_owned())));
        assert!(refused.contains("only") && refused.contains("encrypted"));
    }
}
