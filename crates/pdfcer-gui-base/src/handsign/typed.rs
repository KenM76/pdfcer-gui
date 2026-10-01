//! # `handsign::typed` — a signature typed in a handwriting face: the faces this computer offers, the embed plan, and its fit into a signature box
//!
//! Contract: [`faces`] lists the usable handwriting faces in the operating
//! system's font folders, in a fixed preference order, each already proven
//! embeddable; [`plan`] subsets one for a name; [`fit_typed`] sizes and
//! places that name in a y-down box by the rule [`super::fit`] applies to a
//! drawn mark.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/handsign/typed.md`.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use egui::{Pos2, Rect, pos2};
use pdfcer_core::font_embed::FontEmbedPlan;

use super::{CENTRE_BELOW, FILL, INSET, RISE};

/// The file a remembered typed signature is kept in, beside `settings.txt`.
pub const TYPED_FILE: &str = "typed-signature.txt"; // ui-text-exempt: a file name, never displayed as copy

/// The handwriting faces looked for, in the order they are offered: the
/// family name shown to the operator, and the file name in the font folder.
const CANDIDATES: [(&str, &str); 3] = [
    ("Segoe Script", "segoesc.ttf"), // ui-text-exempt: a font's own family name
    ("Ink Free", "Inkfree.ttf"),     // ui-text-exempt: a font's own family name
    ("Segoe Print", "segoepr.ttf"),  // ui-text-exempt: a font's own family name
];

/// One handwriting face this computer has and pdfcer may embed.
#[derive(Debug)]
pub struct Face {
    /// The family name, as the operator knows it.
    pub label: &'static str,
    /// The font file it was read from.
    pub path: PathBuf,
    /// The whole font program, shared with the preview.
    pub bytes: Arc<Vec<u8>>,
}

/// The usable faces, found once per run. Empty when none is installed or
/// none permits embedding; typing a signature is then not offered.
#[must_use]
pub fn faces() -> &'static [Face] {
    static FACES: OnceLock<Vec<Face>> = OnceLock::new();
    FACES.get_or_init(|| {
        let dirs = crate::fontsearch::os_font_dirs();
        CANDIDATES
            .iter()
            .filter_map(|(label, file)| {
                let path = dirs.iter().map(|d| d.join(file)).find(|p| p.is_file())?;
                let bytes = Arc::new(std::fs::read(&path).ok()?);
                let face = Face { label, path, bytes };
                // A probe subset proves the face is TrueType and that its
                // licence bits allow embedding before it is ever offered.
                plan(&face, "Ab").ok().map(|_| face)
            })
            .collect()
    })
}

/// The subset of `face` covering every character of `name`.
///
/// # Errors
///
/// The subsetter's reason, in its words: the face cannot be subset, or lacks
/// a character `name` uses.
pub fn plan(face: &Face, name: &str) -> Result<FontEmbedPlan, String> {
    use pdfcer_render::font::subset::{plan_subset, subset_tag_for};
    let mut wanted: Vec<char> = name.chars().collect();
    wanted.sort_unstable();
    wanted.dedup();
    let stem = face.path.file_stem().map_or_else(
        || face.label.replace(' ', ""),
        |s| s.to_string_lossy().into_owned(),
    );
    // The tag names the character set, so two names never share a tag for
    // two different subsets (ISO 32000-1 §9.6.4).
    let keyed: String = wanted.iter().collect();
    let tag = subset_tag_for(&format!("{stem}\u{0}{keyed}"));
    plan_subset(&face.bytes, 0, &wanted, &stem, &tag).map_err(|e| e.to_string())
}

/// `name`'s advance width per point of font size, from `plan`'s glyph
/// widths; `None` when the plan lacks a character `name` uses.
#[must_use]
pub fn advance(plan: &FontEmbedPlan, name: &str) -> Option<f32> {
    name.chars()
        .map(|c| {
            plan.glyphs
                .iter()
                .find(|g| g.unicode == c)
                .map(|g| g.width as f32 / 1000.0)
        })
        .sum()
}

/// A typed name placed in a box: the font size, and the left end of the
/// baseline, in the target's units and y-down space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypedFit {
    /// Font size.
    pub size: f32,
    /// Where the first glyph's origin sits.
    pub origin: Pos2,
}

/// Place a name `advance` wide per point of size, in a face whose `ascent`
/// and `descent` (negative) are per point of size, into `target` (y-down).
/// The face's full height stands in for a drawn mark's ink height in
/// [`super::fit`]'s rule. `None` for an empty name or a degenerate target.
#[must_use]
pub fn fit_typed(advance: f32, ascent: f32, descent: f32, target: Rect) -> Option<TypedFit> {
    let height = ascent - descent;
    if !(advance > 0.0 && height > 0.0 && target.width() > 0.0 && target.height() > 0.0) {
        return None;
    }
    let (w, h) = (target.width(), target.height());
    let size = (FILL * w / advance).min(FILL * RISE * h / height);
    let placed_h = height * size;
    let top = if placed_h <= CENTRE_BELOW * h {
        target.min.y + (h - placed_h) / 2.0
    } else {
        target.max.y - (1.0 - FILL) * h - placed_h
    };
    Some(TypedFit {
        size,
        origin: pos2(target.min.x + INSET * w, top + ascent * size),
    })
}

/// A remembered typed signature: the face and the name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Typed {
    /// The face's family name. An index would name a different face on a
    /// computer with a different set installed.
    pub face: String,
    /// The name as typed.
    pub name: String,
}

impl Typed {
    /// The on-disk form: the face on the first line, the name on the second.
    #[must_use]
    pub fn to_text(&self) -> String {
        format!("{}\n{}\n", self.face, self.name)
    }

    /// Read [`Self::to_text`]'s form; `None` when either line is missing or
    /// blank.
    #[must_use]
    pub fn from_text(text: &str) -> Option<Self> {
        let mut lines = text.lines();
        let face = lines.next()?.trim();
        let name = lines.next()?.trim();
        (!face.is_empty() && !name.is_empty()).then(|| Self {
            face: face.to_owned(),
            name: name.to_owned(),
        })
    }
}

fn typed_path() -> Option<PathBuf> {
    pdfcer_core::settings::resolve_store()
        .directory()
        .map(|dir| dir.join(TYPED_FILE))
}

/// The remembered typed signature, if one is kept and readable.
#[must_use]
pub fn load() -> Option<Typed> {
    Typed::from_text(&std::fs::read_to_string(typed_path()?).ok()?)
}

/// Keep `typed` on this computer. Returns whether it was written.
pub fn save(typed: &Typed) -> bool {
    let Some(path) = typed_path() else {
        return false;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&path, typed.to_text()).is_ok()
}

/// Delete the remembered typed signature. Absent already is success.
pub fn forget() -> bool {
    match typed_path() {
        Some(path) => match std::fs::remove_file(path) {
            Ok(()) => true,
            Err(e) => e.kind() == std::io::ErrorKind::NotFound,
        },
        None => true,
    }
}

/// Whether text written along `sheet`'s own x axis reads left to right on
/// screen. The engine's text verb has no rotation, so on a page shown turned
/// a typed name would run up or down its box.
#[must_use]
pub fn writes_along(sheet: &pdfcer_core::page_tree::Page) -> bool {
    use crate::viewer::space::canvas_to_pdf_space;
    let a = canvas_to_pdf_space(pos2(0.0, 0.0), sheet);
    let b = canvas_to_pdf_space(pos2(1.0, 0.0), sheet);
    a.zip(b)
        .is_some_and(|(a, b)| (b.x - a.x - 1.0).abs() < 1e-3 && (b.y - a.y).abs() < 1e-3)
}

#[cfg(test)]
mod tests;
