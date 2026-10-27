//! # `text::snapshot` — what the snapshot box says on the status row

use crate::clipboard::snapshot::{KeptText, SnapshotCopy, Vectors};
use pdfcer_core::pageops::RegionReport;

/// The save dialog's title.
#[must_use]
pub const fn save_title() -> &'static str {
    "Save the snapshot box as a PDF"
}

/// Why nothing was saved: there is no box, or it lies off its page.
#[must_use]
pub const fn save_no_box() -> &'static str {
    "There is no snapshot box to save. Draw one with the Snapshot tool on the View tab \
     first."
}

/// Why nothing was saved: the file picked is the document on screen.
#[must_use]
pub const fn save_over_document() -> &'static str {
    "The snapshot box was not saved: that file is the document on screen. Pick another \
     name."
}

/// Why nothing was saved: the engine refused the region, in its own words.
#[must_use]
pub fn save_refused(why: &str) -> String {
    format!("The snapshot box could not be cut out of its page: {why}. Nothing was saved.")
}

/// Why nothing was saved: the file could not be written.
#[must_use]
pub fn save_failed(path: &str, why: &str) -> String {
    format!("The snapshot box could not be written to {path}: {why}.")
}

/// **What saving the snapshot box wrote**: the file, its page size, and
/// anything the engine left out or could not cut away.
#[must_use]
pub fn saved_pdf(path: &str, report: &RegionReport) -> String {
    let (w, h) = (
        report.rect.urx - report.rect.llx,
        report.rect.ury - report.rect.lly,
    );
    let edge = match report.glyphs_removed {
        0 => String::new(),
        1 => " One character crossing its edge was left out.".to_owned(),
        n => format!(" {n} characters crossing its edge were left out."),
    };
    let cut = if report.has_residuals() {
        " Some drawing outside the box could not be cut away; it is in the file, beyond the \
         page's edge."
    } else {
        " Everything outside the box was cut away."
    };
    let said = format!(
        "Saved the snapshot box as a one-page PDF, {w:.0} \u{d7} {h:.0} pt: {path}.{cut}{edge}"
    );
    report
        .notes
        .iter()
        .fold(said, |text, note| format!("{text} {note}"))
}

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
            blends_flattened,
            text,
        } => {
            let edge = match glyphs_removed {
                0 => String::new(),
                1 => " One character crossing its edge was left out.".to_owned(),
                n => format!(" {n} characters crossing its edge were left out."),
            };
            let blends = match blends_flattened {
                0 => String::new(),
                1 => " One see-through area, such as a highlight, was copied as its \
                      blended pixels so it stays see-through when pasted."
                    .to_owned(),
                n => format!(
                    " {n} see-through areas, such as highlights, were copied as their \
                     blended pixels so they stay see-through when pasted."
                ),
            };
            let kept = text.map(kept_text).unwrap_or_default();
            let said = format!(
                "Copied the snapshot box as vectors and as {picture}. Everything outside the \
                 box was cut away.{edge}{blends}{kept}"
            );
            (said, notes.as_slice())
        }
        Vectors::PictureOnly => {
            let said = format!(
                "Copied the snapshot box as {picture} only, as the Snapshot copies as \
                 setting asks."
            );
            (said, &[][..])
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

/// What became of the text when it was to be kept as text.
fn kept_text(kept: KeptText) -> String {
    let runs = |n: usize| super::plural(n, "run", "runs");
    if kept.svg_outlines == 0 && kept.emf_outlines == 0 {
        if kept.svg_text == 0 && kept.emf_text == 0 {
            return String::new();
        }
        return format!(" The text was kept as text ({}).", runs(kept.svg_text.max(kept.emf_text)));
    }
    format!(
        " Some text could not be kept as text and stays as outlines: {} of {} in the SVG form, \
         {} of {} in the EMF form.",
        runs(kept.svg_outlines),
        kept.svg_outlines + kept.svg_text,
        runs(kept.emf_outlines),
        kept.emf_outlines + kept.emf_text,
    )
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
            blends_flattened: 0,
            text: None,
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
        assert!(copied(&copy(300, 300, Vectors::PictureOnly)).contains("only"));
    }

    #[test]
    fn flattened_blends_and_kept_text_are_said() {
        let vectors = Vectors::Cut {
            glyphs_removed: 0,
            notes: Vec::new(),
            blends_flattened: 2,
            text: Some(KeptText {
                svg_text: 3,
                svg_outlines: 1,
                emf_text: 4,
                emf_outlines: 0,
            }),
        };
        let said = copied(&copy(300, 300, vectors));
        assert!(said.contains("2 see-through areas"), "{said}");
        assert!(said.contains("1 run of 4 in the SVG"), "{said}");
    }
}
