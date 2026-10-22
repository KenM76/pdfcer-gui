//! # `text::compact` — what the Save-a-compacted-copy window says before it
//! throws anything away
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/compact.md`.

use pdfcer_core::optimize::resubset::ResubsetPlan;
use pdfcer_core::optimize::{CleanupPlan, DiscardObject, ObjectDiscard, RebuildReport, SkipReason};
use pdfcer_core::optimize::{ContentChange, ContentSkip, Tolerance, Unrounded};
use pdfcer_core::space_audit::SpaceCategory;

/// The window's title bar.
#[must_use]
pub const fn window_title() -> &'static str {
    "Save a compacted copy"
}

/// The opening sentence.
#[must_use]
pub const fn intro() -> &'static str {
    "pdfcer normally saves by adding your changes to the end of the file, which keeps the earlier \
     version of your drawing inside it. This writes the whole file fresh instead, so anything no \
     longer used is dropped."
}

/// How much smaller the file is expected to be.
#[must_use]
pub fn size_change(before: u64, after: u64) -> String {
    if after >= before {
        // `>=`, not `>`. A rewrite can legitimately come out very slightly
        // LARGER — §7.5.4 requires a single-section cross-reference table with
        // one entry per object number from zero, so a file whose objects are
        // sparsely numbered pays for the gaps. Saying "no smaller" covers both
        // and does not invite the question a byte count would.
        return "This file has nothing unused in it, so a compacted copy would be no smaller. \
                The copy is still written if you want one."
            .to_owned();
    }
    let saved = before - after;
    format!(
        "This file is {}. A compacted copy would be {} — about {} smaller.",
        bytes(before),
        bytes(after),
        bytes(saved)
    )
}

/// A byte count in the units an operator thinks in.
pub(crate) fn bytes(n: u64) -> String {
    let mib = n as f64 / (1024.0 * 1024.0);
    if mib >= 1024.0 {
        format!("{:.1} GB", mib / 1024.0)
    } else if mib >= 0.1 {
        format!("{mib:.1} MB")
    } else {
        format!("{:.0} KB", n as f64 / 1024.0)
    }
}

/// The previous revision is discarded.
#[must_use]
pub const fn revisions_line() -> &'static str {
    "The earlier version of the drawing that pdfcer keeps inside the file is dropped. Your own \
     Undo is not affected, and neither is the original file — this always writes a new one."
}

/// The document is signed and the copy will not be.
#[must_use]
pub fn signature_line(count: usize) -> String {
    format!(
        "This document carries {count} digital signature(s). A compacted copy CANNOT keep them — \
         rewriting the file moves every byte they cover, and nothing later can repair that. Your \
         original file keeps its signatures."
    )
}

/// The button that writes it.
#[must_use]
pub const fn save_button() -> &'static str {
    "Choose where to save…"
}

/// The button that does not.
#[must_use]
pub const fn cancel_button() -> &'static str {
    "Cancel"
}

/// The disclosure after a compacted copy is written.
///
/// It repeats the size because that is the outcome, and names the **file**
/// because a copy an operator cannot find is a copy they will make twice.
#[must_use]
pub fn written(path: &str, before: u64, after: u64) -> String {
    if after >= before {
        return format!("Wrote a compacted copy to {path}. It is no smaller — nothing was unused.");
    }
    format!(
        "Wrote a compacted copy to {path} — {} instead of {}.",
        bytes(after),
        bytes(before)
    )
}

/// The switch that recompresses the copy's streams as well.
#[must_use]
pub const fn recompress_toggle() -> &'static str {
    "Also recompress the drawing data"
}

/// Hover on the switch.
#[must_use]
pub const fn recompress_tooltip() -> &'static str {
    "Re-encodes the data inside the file more tightly, drops parts nothing uses, merges \
     identical ones and packs the file's index. Nothing you see changes, and anything that \
     would not get smaller is kept as it was. A large drawing takes a few seconds."
}

/// The switch that removes links and bookmarks that go nowhere.
#[must_use]
pub const fn broken_toggle() -> &'static str {
    "Remove links and bookmarks that go nowhere"
}

/// Hover on it.
#[must_use]
pub const fn broken_tooltip() -> &'static str {
    "A link or bookmark whose page has been deleted, or whose named place does not exist, does nothing when clicked. A bookmark is removed only when every bookmark under it goes nowhere too, so no working bookmark is lost. Links to web pages and other files are never touched."
}

/// The switch that removes named destinations nothing uses.
#[must_use]
pub const fn destinations_toggle() -> &'static str {
    "Remove named places nothing in the file links to"
}

/// Hover on it.
#[must_use]
pub const fn destinations_tooltip() -> &'static str {
    "Named places (named destinations) that no link or bookmark in this file points to. A link from another file or a web page to one of them stops working, which is why this is off unless you turn it on. A file with scripts is left alone, since a script can name a place."
}

/// The switch that cuts embedded fonts down.
#[must_use]
pub const fn fonts_toggle() -> &'static str {
    "Cut embedded fonts down to the characters used"
}

/// Hover on it.
#[must_use]
pub const fn fonts_tooltip() -> &'static str {
    "Each embedded TrueType or CFF font keeps only the characters this document draws, and copies of one font embedded more than once become one. The pages look the same. Text added later in another program may have no characters to draw with. A font that cannot be cut safely is kept whole and listed with why. In a PDF/A-1 document the list of characters the standard asks for is written with the cut font; a font whose list cannot be written is kept whole."
}

/// The switch that lays the copy out for Fast Web View.
#[must_use]
pub const fn linearize_toggle() -> &'static str {
    "Fast Web View: show the first page before the rest has downloaded"
}

/// Hover on it.
#[must_use]
pub const fn linearize_tooltip() -> &'static str {
    "Writes the file with its first page at the front and a table saying where every other page \
     is, so a web browser or a viewer reading over a network shows a page without waiting for \
     the whole file. The file is always rewritten in full, its index is not packed, and it is \
     usually a little larger; it is kept even then, because you asked for the layout. Saving it \
     again in pdfcer adds your changes at the end, which most viewers then read the slow way."
}

/// The switch that tidies the page drawing instructions.
#[must_use]
pub const fn content_toggle() -> &'static str {
    "Tidy the drawing instructions on each page"
}

/// Hover on it.
#[must_use]
pub const fn content_tooltip() -> &'static str {
    "Rewrites the instructions that draw each page in their shortest form: spacing and comments a \
     PDF reader does not need are dropped, a number like 0.500 is written .5, and instructions \
     that change nothing are removed. Each rewrite is read back and compared with the original \
     before it is kept, so the pages look exactly the same. Text, pictures and fonts are not \
     touched."
}

/// What tidying the drawing instructions did, as one sentence; `None` when
/// nothing was tidied.
#[must_use]
pub fn content<'a>(changes: impl IntoIterator<Item = &'a ContentChange>) -> Option<String> {
    let (mut parts, mut before, mut after, mut removed, mut shortened) = (0usize, 0, 0, 0, 0);
    let (mut rounded, mut farthest) = (0u64, 0.0f64);
    for change in changes {
        parts += 1;
        before += change.decoded_before;
        after += change.decoded_after;
        removed += change.operators_removed;
        shortened += change.numbers_shortened;
        rounded += change.numbers_rounded;
        farthest = farthest.max(change.max_displacement);
    }
    (parts > 0).then(|| {
        let moved = if rounded > 0 {
            format!(
                " {rounded} line position(s) rounded, none moving more than {} pt.",
                distance(farthest)
            )
        } else {
            String::new()
        };
        format!(
            "Drawing instructions in {parts} part(s) tidied from {} to {} before compression: \
             {removed} instruction(s) that changed nothing removed, {shortened} number(s) written \
             shorter.{moved}",
            space_bytes(before),
            space_bytes(after)
        )
    })
}

/// Why tidying a part's drawing instructions left it as stored.
#[must_use]
pub fn content_left_because(why: &ContentSkip) -> String {
    match why {
        ContentSkip::Mismatch => {
            "tidying their drawing instructions did not read back the same, so pdfcer kept them \
             as stored"
                .to_owned()
        }
        other => other.to_string(),
    }
}

/// The rounding row's label.
#[must_use]
pub const fn round_label() -> &'static str {
    "Round line positions"
}

/// Hover on the rounding row.
#[must_use]
pub const fn round_tooltip() -> &'static str {
    "Writes the corners and curve points of drawn lines, and where each line of text starts, \
     with fewer decimals, so no point moves more than the distance chosen. This changes the drawing, by at most that distance; the \
     window says how far the farthest point moved. Only each page's main drawing is rounded."
}

/// Why the rounding row is greyed.
#[must_use]
pub const fn round_needs_content() -> &'static str {
    "Rounding is part of tidying the drawing instructions; turn that on first."
}

/// One rounding choice as the list shows it: off, or the farthest a point may move.
#[must_use]
pub fn round_choice(within: Option<Tolerance>) -> String {
    within.map_or_else(
        || "Off".to_owned(),
        |t| format!("Move no point more than {} pt", distance(t.as_units())),
    )
}

/// A distance in points with no trailing zeros.
fn distance(units: f64) -> String {
    let s = format!("{units:.4}");
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// Why rounding left some drawing instructions as they were, one line per
/// reason with how many parts it covers.
#[must_use]
pub fn unrounded<'a>(changes: impl IntoIterator<Item = &'a ContentChange>) -> Vec<String> {
    let mut counts = std::collections::BTreeMap::new();
    for why in changes.into_iter().filter_map(|c| c.unrounded) {
        *counts.entry(unrounded_because(why)).or_insert(0usize) += 1;
    }
    counts
        .into_iter()
        .map(|(why, n)| format!("Line positions left unrounded in {n} part(s): {why}."))
        .collect()
}

/// Why one part was not rounded, in the operator's terms.
fn unrounded_because(why: Unrounded) -> String {
    match why {
        Unrounded::UnknownStart => {
            "they are drawn inside another part, or after one pdfcer cannot follow, so their scale on the page is not known".to_owned()
        }
        Unrounded::UnknownMatrix => {
            "a change of scale pdfcer cannot follow stopped the rounding part way".to_owned()
        }
        other => other.to_string(),
    }
}

/// What cutting the fonts did, one sentence each: the total, each font cut,
/// each copy merged, and each font kept whole with why.
#[must_use]
pub fn resubset(plan: &ResubsetPlan) -> Vec<String> {
    let mut lines = Vec::new();
    if !plan.targets.is_empty() {
        lines.push(format!(
            "{} embedded font(s) cut down to the characters used, saving {}.",
            plan.targets.len(),
            bytes(plan.saving())
        ));
    }
    for cut in &plan.targets {
        let merged = match cut.merged.len() {
            0 => String::new(),
            n => format!(", with {n} other copy(ies) merged into it"),
        };
        // A PDF/A-1 cut adds the character list the standard requires, which
        // is a change to the file and is said.
        let listed = if cut.listed.is_empty() {
            ""
        } else {
            ", and the character list PDF/A-1 asks for written"
        };
        lines.push(format!(
            "{}: {} of {} characters kept{merged}{listed}.",
            cut.name_before, cut.glyphs_kept, cut.glyphs_total
        ));
    }
    for copies in &plan.unmerged {
        lines.push(format!(
            "Copies of {} kept apart: {}.",
            copies.face, copies.why.reason
        ));
    }
    for kept in &plan.refused {
        lines.push(format!("{} kept whole: {}.", kept.name, kept.why.reason));
    }
    lines
}

/// The switch that removes one class of document object, and its hover;
/// `None` for a class this window does not offer.
#[must_use]
pub const fn discard_toggle(object: DiscardObject) -> Option<(&'static str, &'static str)> {
    Some(match object {
        DiscardObject::Bookmarks => (
            "Remove all bookmarks",
            "Removes the whole bookmark list, working bookmarks included. The pages are not changed, and links on the pages keep working.",
        ),
        DiscardObject::Tags => (
            "Remove tags",
            "Removes the structure that screen readers, reflow and PDF/UA rely on. The pages look the same, but the copy is no longer an accessible (tagged) PDF.",
        ),
        DiscardObject::PrintSettings => (
            "Remove print settings",
            "Removes the page scaling, two-sided printing, paper tray, page range and copy count stored in the file; a reader then prints with its own defaults.",
        ),
        DiscardObject::AlternateImages => (
            "Remove alternate images",
            "Removes the extra versions some pictures carry, such as a high-resolution one for printing, keeping the one shown on screen. A picture on a layer keeps them.",
        ),
        DiscardObject::FormActions => (
            "Remove form submit and reset",
            "Removes the actions that send a form's contents somewhere, import data into it or clear it, from fields, buttons and links. Anything chained after one still runs.",
        ),
        DiscardObject::ExternalReferences => (
            "Remove links to other files",
            "Removes links and actions that open another file or launch a program. Links to web pages stay.",
        ),
        DiscardObject::ObjectData => (
            "Remove object data",
            "Removes the name-and-value properties a CAD or diagram program attaches to each drawn object, such as part numbers and layer details. The drawing looks the same. Has no effect when tags are removed too, since the properties live in the tags.",
        ),
        DiscardObject::FormFields => (
            "Remove form fields",
            "Removes every fill-in field, check box, button and signature field, and what each showed: a filled-in value disappears from the page. To keep what a field shows, flatten the form instead.",
        ),
        DiscardObject::Multimedia => (
            "Remove sound and video",
            "Removes sound, movie and other media players from the pages, and the actions that play them.",
        ),
        DiscardObject::ThreeDModels => (
            "Remove 3D models",
            "Removes 3D models embedded in the pages, and the links that change their view. The area where a model was shown is left empty.",
        ),
        DiscardObject::FlattenLayers => (
            "Merge visible layers into the page",
            "Every layer shown when the file opens stops being a layer: what it draws stays on the page and always shows and prints, and the layer leaves the Layers list. Hidden layers stay layers.",
        ),
        DiscardObject::SearchIndex => (
            "Remove the search index",
            "Removes a ready-made full-text index some programs embed to search large files quickly. It holds a copy of the document's text; searching still works without it.",
        ),
        DiscardObject::MergeImageFragments => (
            "Merge picture strips into one image",
            "Some programs store a picture as many thin strips, each its own image. Each such picture becomes one image that draws the same, and the faint lines some viewers show between the strips go. Pictures that are masked, on a layer or tagged stay as they are.",
        ),
        DiscardObject::SmoothLines => (
            "Smooth short lines into curves",
            "CAD programs often draw an arc or spline as many short straight lines. Each such run is redrawn as a few curves, which is smaller. The linework moves by up to 0.1 pt, so a measurement taken from the drawing can change by that much; the report lists each page's largest move.",
        ),
        _ => return None,
    })
}

/// The heading over the document-object switches, counting those that are on.
#[must_use]
pub fn discard_heading(on: usize) -> String {
    if on == 0 {
        "Remove whole kinds of content".to_owned()
    } else {
        format!("Remove whole kinds of content ({on} on)")
    }
}

/// What the document-object removals took, one sentence per class.
#[must_use]
pub fn discarded(objects: &[ObjectDiscard]) -> Vec<String> {
    DiscardObject::ALL
        .into_iter()
        .filter_map(|class| {
            let of: Vec<&ObjectDiscard> = objects.iter().filter(|o| o.object == class).collect();
            let bytes: u64 = of.iter().map(|o| o.bytes).sum();
            if class == DiscardObject::FlattenLayers {
                return (!of.is_empty()).then(|| merged_layers(&of));
            }
            if class == DiscardObject::MergeImageFragments {
                return (!of.is_empty()).then(|| merged_strips(&of, bytes));
            }
            if class == DiscardObject::SmoothLines {
                return (!of.is_empty()).then(|| smoothed(&of, bytes));
            }
            let what = match class {
                DiscardObject::Bookmarks => "Bookmarks".to_owned(),
                DiscardObject::Tags => "Tags".to_owned(),
                DiscardObject::PrintSettings => "Print settings".to_owned(),
                DiscardObject::AlternateImages => {
                    format!("Alternate images of {} picture(s)", of.len())
                }
                DiscardObject::FormActions => {
                    format!("Form submit and reset actions in {} place(s)", of.len())
                }
                DiscardObject::ExternalReferences => {
                    format!("Links to other files in {} place(s)", of.len())
                }
                DiscardObject::ObjectData => {
                    format!("Object data in {} place(s)", of.len())
                }
                DiscardObject::FormFields => "Form fields".to_owned(),
                DiscardObject::Multimedia => "Sound and video".to_owned(),
                DiscardObject::ThreeDModels => "3D models".to_owned(),
                DiscardObject::SearchIndex => "The search index".to_owned(),
                _ => return None,
            };
            (!of.is_empty()).then(|| format!("{what} removed, about {} KB.", bytes.div_ceil(1024)))
        })
        .collect()
}

/// The layers merged into the page, by name; the engine words a layer's
/// location as `layer 9 0: Notes`.
fn merged_layers(of: &[&ObjectDiscard]) -> String {
    let names: Vec<&str> = of
        .iter()
        .map(|o| {
            o.location
                .split_once(": ")
                .map_or(o.location.as_str(), |(_, n)| n)
        })
        .collect();
    format!(
        "{} layer(s) merged into the page: {}.",
        names.len(),
        names.join(", ")
    )
}

/// The pictures whose strips were merged, page by page; the engine words a
/// page as `page 1: 40 strips into 1 image (800x1100)`.
fn merged_strips(of: &[&ObjectDiscard], bytes: u64) -> String {
    let pages: Vec<String> = of
        .iter()
        .map(|o| o.location.replacen(": ", ", ", 1))
        .collect();
    format!(
        "Picture strips merged: {}. About {} KB smaller.",
        pages.join("; "),
        bytes.div_ceil(1024)
    )
}

/// The pages whose lines were smoothed; the engine words a page as
/// `page 1: 120 line segments into 6 curves (max deviation 0.031 pt)`.
fn smoothed(of: &[&ObjectDiscard], bytes: u64) -> String {
    let pages: Vec<String> = of
        .iter()
        .map(|o| o.location.replacen(": ", ", ", 1))
        .collect();
    format!(
        "Lines smoothed into curves: {}. About {} KB smaller.",
        pages.join("; "),
        bytes.div_ceil(1024)
    )
}

/// Said when the copy is kept although it is no smaller, because it removes
/// what was asked for or is laid out for Fast Web View (`linearized`).
#[must_use]
pub const fn kept_not_smaller(linearized: bool) -> &'static str {
    if linearized {
        "The copy is no smaller, and is kept anyway because it is laid out for Fast Web View."
    } else {
        "The copy is no smaller, and is kept anyway because it removes what you asked for."
    }
}

/// What the clean-ups removed, one sentence per kind; kinds with nothing
/// removed are left out.
#[must_use]
pub fn cleaned(plan: &CleanupPlan) -> Vec<String> {
    let mut lines = Vec::new();
    if !plan.links.is_empty() {
        lines.push(format!(
            "{} link(s) that went nowhere removed.",
            plan.links.len()
        ));
    }
    if !plan.bookmarks.is_empty() {
        let titles: Vec<&str> = plan.bookmarks.iter().map(|b| b.title.as_str()).collect();
        let under: usize = plan.bookmarks.iter().map(|b| b.descendants).sum();
        lines.push(format!(
            "{} bookmark(s) that went nowhere removed{}: {}.",
            plan.bookmarks.len(),
            if under > 0 {
                format!(", with the {under} under them")
            } else {
                String::new()
            },
            titles.join(", ")
        ));
    }
    if !plan.destinations.is_empty() {
        lines.push(format!(
            "{} named place(s) nothing linked to removed.",
            plan.destinations.len()
        ));
    }
    lines
}

/// What the structure rebuild did, one sentence per kind of change; kinds
/// that did not happen are left out.
#[must_use]
pub fn restructured(report: &RebuildReport) -> Vec<String> {
    let mut lines = Vec::new();
    if !report.unreferenced.is_empty() {
        lines.push(format!(
            "{} unused part(s) dropped.",
            report.unreferenced.len()
        ));
    }
    if !report.merged.is_empty() {
        lines.push(format!("{} duplicate part(s) merged.", report.merged.len()));
    }
    if report.dangling > 0 {
        lines.push(format!(
            "{} reference(s) to parts the file does not contain were written as empty, which is \
             what a reader already took them to mean.",
            report.dangling
        ));
    }
    if report.linearized {
        lines.push(
            "The copy is laid out for Fast Web View: its first page shows before the rest has \
             downloaded."
                .to_owned(),
        );
    }
    if let Some(version) = report.version_raised {
        let why = if report.linearized {
            "Fast Web View"
        } else {
            "packing its index"
        };
        lines.push(format!(
            "The copy is marked PDF {version}, which {why} requires. Readers older than that may \
             not open it."
        ));
    }
    lines
}

/// Shown while the copy is being recompressed.
#[must_use]
pub const fn recompress_working() -> &'static str {
    "Working on the copy…"
}

/// How much was recompressed.
#[must_use]
pub fn recompressed(changed: usize) -> String {
    format!("{changed} part(s) of the file were recompressed.")
}

/// Recompressing gained nothing.
#[must_use]
pub const fn recompress_no_gain() -> &'static str {
    "None of this made the copy smaller, so the plain compacted copy is kept."
}

/// Recompressing failed; the plain copy stands.
#[must_use]
pub fn recompress_failed(detail: &str) -> String {
    format!("Could not optimize the copy: {detail}. The plain compacted copy is kept.")
}

/// The worker went away without an answer.
#[must_use]
pub const fn recompress_stopped() -> &'static str {
    "the work on the copy stopped unexpectedly"
}

/// Hover on the save button while recompressing.
#[must_use]
pub const fn save_waiting() -> &'static str {
    "Waiting for the work on the copy to finish."
}

/// Parts the recompression left as stored, for one reason.
#[must_use]
pub fn left_line(reason: &str, count: usize) -> String {
    format!("{count} part(s) left as they were: {reason}.")
}

/// Why a part was left as stored, in the operator's words.
#[must_use]
pub fn left_because(reason: &SkipReason) -> String {
    match reason {
        SkipReason::NotSmaller { .. } => "re-encoding them was not smaller".to_owned(),
        SkipReason::UnsupportedFilter(name) => format!("they use {name}, which is not re-encoded"),
        SkipReason::Undecodable(_) => "they could not be read".to_owned(),
        SkipReason::Protected(what) => format!("{what} is never rewritten"),
        SkipReason::Unplaced => {
            "they are pictures drawn on no page, so they were not scaled".to_owned()
        }
        SkipReason::BelowMinSaving { pct, .. } => {
            format!("re-encoding them saved less than {pct}%")
        }
        SkipReason::ImageUnsupported(why) => super::compact_images::picture_left_because(why),
        SkipReason::Content(why) => content_left_because(why),
        other => other.to_string(),
    }
}

/// The fold that lists where the bytes go.
#[must_use]
pub const fn space_heading() -> &'static str {
    "Where the space goes"
}

/// Hover on the fold.
#[must_use]
pub const fn space_tooltip() -> &'static str {
    "Every byte of the file now and of the compacted copy, sorted into what it is for. The \
     columns each add up to the whole file."
}

/// The column over what is on disk now.
#[must_use]
pub const fn space_column_now() -> &'static str {
    "Now"
}

/// The column over the compacted copy.
#[must_use]
pub const fn space_column_copy() -> &'static str {
    "Compacted"
}

/// The column over the category names.
#[must_use]
pub const fn space_column_what() -> &'static str {
    "What"
}

/// Shown in the Now column when the document has no file yet.
#[must_use]
pub const fn space_no_file() -> &'static str {
    "—"
}

/// What a category of the file holds, in the operator's words.
///
/// A category the engine adds later shows its engine name until it is given
/// one here; its bytes are never hidden.
#[must_use]
pub fn space_category(category: SpaceCategory) -> &'static str {
    match category {
        SpaceCategory::ContentStreams => "Page drawing",
        SpaceCategory::FormXObjects => "Reused drawing blocks",
        SpaceCategory::Images => "Images",
        SpaceCategory::Fonts => "Fonts",
        SpaceCategory::ColorSpaces => "Colour profiles",
        SpaceCategory::PatternsAndShadings => "Patterns and shading",
        SpaceCategory::GraphicsStates => "Drawing settings",
        SpaceCategory::PageStructure => "Pages and the page list",
        SpaceCategory::Annotations => "Comments and markups",
        SpaceCategory::Links => "Links",
        SpaceCategory::Forms => "Form fields",
        SpaceCategory::Bookmarks => "Bookmarks",
        SpaceCategory::NamedDestinations => "Named destinations",
        SpaceCategory::Thumbnails => "Page thumbnails",
        SpaceCategory::StructureTree => "Accessibility tags",
        SpaceCategory::Metadata => "Document information",
        SpaceCategory::EmbeddedFiles => "Attached files",
        SpaceCategory::JavaScript => "Scripts",
        SpaceCategory::Layers => "Layers",
        SpaceCategory::OtherReferenced => "Other content in use",
        SpaceCategory::Unreferenced => "Content nothing uses",
        SpaceCategory::CrossReference => "The file's index",
        SpaceCategory::Superseded => "Earlier versions kept in the file",
        SpaceCategory::FileOverhead => "File overhead",
        other => other.as_str(),
    }
}

/// A byte count in a breakdown cell; small counts stay exact so a 300-byte
/// category does not read as "0 KB".
#[must_use]
pub fn space_bytes(n: u64) -> String {
    if n < 1024 {
        format!("{n} bytes")
    } else {
        bytes(n)
    }
}

/// The engine refused to rewrite this file.
#[must_use]
pub fn refused(detail: &str) -> String {
    format!("pdfcer cannot rewrite this file: {detail}. Save a copy the ordinary way instead.")
}

/// The operating system refused the write.
#[must_use]
pub fn write_failed(detail: &str) -> String {
    format!("pdfcer could not write the compacted copy: {detail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A file with nothing to reclaim is told so, and is not told a
    /// number.**
    #[test]
    fn a_tidy_file_is_told_it_is_tidy() {
        for (before, after) in [(1000_u64, 1000_u64), (1000, 1200)] {
            let line = size_change(before, after);
            assert!(line.contains("no smaller"), "{line}");
            assert!(!line.contains('0'), "it quoted a number: {line}");
        }
        let saving = size_change(4 * 1024 * 1024, 1024 * 1024);
        assert!(saving.contains("smaller"), "{saving}");
        assert!(saving.contains("3.0 MB"), "the saving is named: {saving}");
    }

    /// **The signature sentence says the loss cannot be repaired.**
    #[test]
    fn the_signature_warning_says_it_is_irreversible() {
        let line = signature_line(2);
        assert!(line.contains("CANNOT keep"), "{line}");
        assert!(line.contains("repair"), "{line}");
        assert!(line.contains("original file keeps"), "{line}");
    }

    /// **The written disclosure names the file and the outcome.**
    #[test]
    fn the_result_names_where_it_went() {
        let line = written("C:/out/plan.pdf", 4 * 1024 * 1024, 1024 * 1024);
        assert!(line.contains("C:/out/plan.pdf"), "{line}");
        assert!(line.contains("1.0 MB"), "{line}");
        assert!(written("C:/out/p.pdf", 100, 100).contains("no smaller"));
    }
}
