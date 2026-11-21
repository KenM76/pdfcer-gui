//! Font files offered to Bold and Italic as donor faces: rung 3 of the
//! engine's style ladder (`StyleRung::SuppliedFaceEmbedded`).
//!
//! [`add`] keeps the operator's file on `OpenDoc::style_donors`. [`plans`]
//! subsets each file to the characters being restyled. The forecast passes
//! the plans to `EditSession::preview_style_ladder_with_donors`, and the
//! restyle passes them to `FormatRequest::style_donor`, so the preview and the
//! commit take the same rung. The engine binds a donor only after the page's
//! faces and the standard-14 sibling, and only when its name (the file stem)
//! has the run's family and claims exactly the style asked for.

use pdfcer_core::font_embed::FontEmbedPlan;

use crate::app::state::OpenDoc;

/// Ask for a font file and offer it to Bold and Italic.
pub(crate) fn add(doc: &mut OpenDoc) {
    let crate::app::files::Picked::Path(path) = crate::app::files::pick_font_file() else {
        return;
    };
    if !doc.style_donors.contains(&path) {
        doc.style_donors.push(path.clone());
    }
    let donors = doc.style_donors.len();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("text-style-donor-added donors={donors} file={path:?}")
    });
}

/// One plan per donor file that covers `wanted`; a file that cannot is traced
/// and left out.
pub(crate) fn plans(doc: &OpenDoc, wanted: &[char]) -> Vec<FontEmbedPlan> {
    doc.style_donors
        .iter()
        .filter_map(|path| match plan(path, wanted) {
            Ok(plan) => Some(plan),
            Err(why) => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!("text-style-donor-unusable file={path:?} detail={why}")
                });
                None
            }
        })
        .collect()
}

/// The characters `text` holds, sorted and deduplicated, as a plan wants them.
pub(crate) fn chars_of<'a>(text: impl IntoIterator<Item = &'a str>) -> Vec<char> {
    let mut wanted: Vec<char> = text.into_iter().flat_map(str::chars).collect();
    wanted.sort_unstable();
    wanted.dedup();
    wanted
}

/// Subset the font file at `path` to `wanted`, named after its file stem.
///
/// # Errors
///
/// The file could not be read, or `plan_subset` refused it.
pub(super) fn plan(path: &std::path::Path, wanted: &[char]) -> Result<FontEmbedPlan, String> {
    use pdfcer_render::font::subset::{plan_subset, subset_tag_for};
    let donor = std::fs::read(path).map_err(|e| e.to_string())?;
    let stem = path.file_stem().map_or_else(
        || "EmbeddedFont".to_owned(),
        |s| s.to_string_lossy().into_owned(),
    );
    // The tag names the character set too, so two gestures with different
    // letters never share a tag for two different subsets (§9.6.4).
    let keyed: String = wanted.iter().collect();
    let tag = subset_tag_for(&format!("{stem}\u{0}{keyed}"));
    plan_subset(&donor, 0, wanted, &stem, &tag).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chars_are_sorted_and_unique() {
        assert_eq!(chars_of(["CAB", "BA"]), vec!['A', 'B', 'C']);
    }

    #[test]
    fn an_unreadable_file_has_no_plan() {
        assert!(plan(std::path::Path::new("no-such-face.ttf"), &['A']).is_err());
    }
}
