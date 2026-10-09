//! # icons::assets — the icon set itself, and its provenance record
//!
//! **Generated content.** Every constant below embeds one file of
//! `src/icons/assets/*.svg`, which are byte-for-byte copies of the salvage
//! source's `D:\Dev\pdfce\crates\pdfce-gui\assets\icons\*.svg` — XML
//! rationale comments included. Nothing was retyped, reformatted or
//! "tidied": each asset's own comment is the primary record of what the
//! glyph depicts and which neighbouring glyph it was drawn to stay
//! distinguishable from, and a paraphrase would lose exactly the part that
//! is expensive to re-derive.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/icons/assets.md`.

/// `add-text.svg` — the art for [`super::Icon::AddText`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const ADD_TEXT: &str = include_str!("assets/add-text.svg");

/// `back.svg` — the art for [`super::Icon::Back`].
///
/// Authored for pdfcer in the header §3 style contract — replaces the tofu `←` (U+2190).
pub(super) const BACK: &str = include_str!("assets/back.svg");

/// `bookmarks.svg` — the art for [`super::Icon::Bookmarks`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #4.
pub(super) const BOOKMARKS: &str = include_str!("assets/bookmarks.svg");

/// `chevron-down.svg` — the art for [`super::Icon::ChevronDown`].
///
/// Authored for pdfcer in the header §3 style contract — replaces the tofu `▾` (U+25BE).
pub(super) const CHEVRON_DOWN: &str = include_str!("assets/chevron-down.svg");

/// `chevron-left.svg` — the art for [`super::Icon::ChevronLeft`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const CHEVRON_LEFT: &str = include_str!("assets/chevron-left.svg");

/// `chevron-right.svg` — the art for [`super::Icon::ChevronRight`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const CHEVRON_RIGHT: &str = include_str!("assets/chevron-right.svg");

/// `chevron-up.svg` — the art for [`super::Icon::ChevronUp`].
///
/// Authored for pdfcer in the header §3 style contract — replaces the tofu `▲` (U+25B2).
pub(super) const CHEVRON_UP: &str = include_str!("assets/chevron-up.svg");

/// `close.svg` — the art for [`super::Icon::Close`].
///
/// Authored for pdfcer in the header §3 style contract — replaces the tofu `✕` (U+2715).
pub(super) const CLOSE: &str = include_str!("assets/close.svg");

/// `comment.svg` — the art for [`super::Icon::Comment`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const COMMENT: &str = include_str!("assets/comment.svg");

/// `convert.svg` — the art for [`super::Icon::SetScale`].
///
/// Copied from ScripTree's `icon-convert.svg` (the operator's own art; header §4b) — the reuse ui-spec §8.2 assigns to Set Group Scale.
pub(super) const CONVERT: &str = include_str!("assets/convert.svg");

/// `copy.svg` — the art for [`super::Icon::Copy`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const COPY: &str = include_str!("assets/copy.svg");

/// `delete.svg` — the art for [`super::Icon::Delete`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #7, and shared by both delete verbs.
pub(super) const DELETE: &str = include_str!("assets/delete.svg");

/// `document.svg` — the art for [`super::Icon::Properties`].
///
/// Copied **verbatim** from ScripTree's `icon-document.svg` (the operator's own art; header §4).
pub(super) const DOCUMENT: &str = include_str!("assets/document.svg");

/// `download.svg` — the art for [`super::Icon::Export`].
///
/// Copied from ScripTree's `icon-download.svg` (the operator's own art; header §4b) — the export half of ui-spec §3.1's reserved upload/download pair.
pub(super) const DOWNLOAD: &str = include_str!("assets/download.svg");

/// `edit-objects.svg` — the art for [`super::Icon::EditObjects`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #1.
pub(super) const EDIT_OBJECTS: &str = include_str!("assets/edit-objects.svg");

/// `edit.svg` — the art for [`super::Icon::EditText`].
///
/// Copied **verbatim** from ScripTree's `icon-edit.svg` (the operator's own art; header §4).
pub(super) const EDIT: &str = include_str!("assets/edit.svg");

/// `fit-page.svg` — the art for [`super::Icon::FitPage`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const FIT_PAGE: &str = include_str!("assets/fit-page.svg");

/// `fit-width.svg` — the art for [`super::Icon::FitWidth`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const FIT_WIDTH: &str = include_str!("assets/fit-width.svg");

/// `fit-height.svg` - the art for [`super::Icon::FitHeight`].
pub(super) const FIT_HEIGHT: &str = include_str!("assets/fit-height.svg");

/// `floating-panels.svg` — the art for [`super::Icon::FloatingPanels`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #7.
pub(super) const FLOATING_PANELS: &str = include_str!("assets/floating-panels.svg");

/// `folder.svg` — the art for [`super::Icon::Open`], [`super::Icon::FontFolders`].
///
/// Copied **verbatim** from ScripTree's `icon-folder.svg` (the operator's own art; header §4). One asset, two roles — see [`super::Icon::Open`].
pub(super) const FOLDER: &str = include_str!("assets/folder.svg");

/// `fonts.svg` — the art for [`super::Icon::Fonts`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #5.
pub(super) const FONTS: &str = include_str!("assets/fonts.svg");

/// `form-field.svg` — the art for [`super::Icon::FormField`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #6.
pub(super) const FORM_FIELD: &str = include_str!("assets/form-field.svg");

/// `form-flatten.svg` — the art for [`super::Icon::FormFlatten`].
///
/// Authored for pdfcer in the header §3 style contract — drawn to ui-spec §8.14's own construction for the Flatten action.
pub(super) const FORM_FLATTEN: &str = include_str!("assets/form-flatten.svg");

/// `forms.svg` — the art for [`super::Icon::Forms`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #7.
pub(super) const FORMS: &str = include_str!("assets/forms.svg");

/// `fullscreen.svg` — the art for [`super::Icon::Fullscreen`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #7.
pub(super) const FULLSCREEN: &str = include_str!("assets/fullscreen.svg");

/// `grid.svg` — the art for [`super::Icon::Grid`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #7.
pub(super) const GRID: &str = include_str!("assets/grid.svg");

/// `guides.svg` — the art for [`super::Icon::Guides`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #7.
pub(super) const GUIDES: &str = include_str!("assets/guides.svg");

/// `cut.svg` — the art for [`super::Icon::Cut`].
///
/// Authored for pdfcer in the header §3 style contract. Scissors — the oldest
/// glyph in graphical software and the one nothing has displaced.
pub(super) const CUT: &str = include_str!("assets/cut.svg");

/// `paste.svg` — the art for [`super::Icon::Paste`].
///
/// Authored for pdfcer in the header §3 style contract. A clipboard with its
/// clip; the metaphor the feature is named after.
pub(super) const PASTE: &str = include_str!("assets/paste.svg");

/// `cursor.svg` — the art for [`super::Icon::Cursor`].
pub(super) const CURSOR: &str = include_str!("assets/cursor.svg");

/// `cursor-node.svg` — the art for [`super::Icon::CursorNode`].
///
/// Authored for pdfcer in the header §3 style contract. The hollow half of the
/// pair, plus the three anchor squares the tool reveals.
pub(super) const CURSOR_NODE: &str = include_str!("assets/cursor-node.svg");

/// `hand.svg` — the art for [`super::Icon::Hand`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #7.
pub(super) const HAND: &str = include_str!("assets/hand.svg");

/// `image.svg` — the art for [`super::Icon::InsertImage`].
///
/// Copied from ScripTree's `icon-image.svg` (the operator's own art; header §4b) — ui-spec §8.5 reserved the picture metaphor, and Insert image is its primary claim.
pub(super) const IMAGE: &str = include_str!("assets/image.svg");

/// `info.svg` — the art for [`super::Icon::Info`].
///
/// Authored for pdfcer in the header §3 style contract. A circle enclosing a lower-case "i", drawn as geometry rather than set as text — the most conventional glyph in the whole set, and metaphor-level by construction under header §2 (no single author owns "an i in a circle").
pub(super) const INFO: &str = include_str!("assets/info.svg");

/// `keyboard.svg` — the art for [`super::Icon::Keyboard`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const KEYBOARD: &str = include_str!("assets/keyboard.svg");

/// `layers.svg` — the art for [`super::Icon::Layers`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #4.
pub(super) const LAYERS: &str = include_str!("assets/layers.svg");

/// `link.svg` — the art for [`super::Icon::Combine`].
///
/// Copied **verbatim** from ScripTree's `icon-link.svg` (the operator's own art; header §4). Its `a6 6 0 008 8` packed arc flags are the reason [`super::svg`]'s lexer reads a flag as one character.
pub(super) const LINK: &str = include_str!("assets/link.svg");

/// `list.svg` — the art for [`super::Icon::ManageList`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 deviation #7b (ui-spec §8.2's `icon-ring` reuse, refused with a reason).
pub(super) const LIST: &str = include_str!("assets/list.svg");

/// `markup.svg` — the art for [`super::Icon::Markup`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const MARKUP: &str = include_str!("assets/markup.svg");

/// `page-continuous.svg` — the art for [`super::Icon::PageContinuous`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #7, one of the four-glyph page-display radio.
pub(super) const PAGE_CONTINUOUS: &str = include_str!("assets/page-continuous.svg");

/// `page-extract.svg` — the art for [`super::Icon::PageExtract`].
///
/// Authored for pdfcer in the header §3 style contract — the Extract-pages half of ui-spec §3.1's reserved download direction.
pub(super) const PAGE_EXTRACT: &str = include_str!("assets/page-extract.svg");

/// `page-facing-continuous.svg` — the art for [`super::Icon::PageFacingContinuous`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #7, one of the four-glyph page-display radio.
pub(super) const PAGE_FACING_CONTINUOUS: &str = include_str!("assets/page-facing-continuous.svg");

/// `page-facing.svg` — the art for [`super::Icon::PageFacing`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #7, one of the four-glyph page-display radio.
pub(super) const PAGE_FACING: &str = include_str!("assets/page-facing.svg");

/// `page-single.svg` — the art for [`super::Icon::PageSingle`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #7, one of the four-glyph page-display radio.
pub(super) const PAGE_SINGLE: &str = include_str!("assets/page-single.svg");

/// `pages.svg` — the art for [`super::Icon::Pages`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #7, and the glyph that retires `view.panel_pages`' recorded "no icon" decision.
pub(super) const PAGES: &str = include_str!("assets/pages.svg");

/// `printer.svg` — the art for [`super::Icon::Print`].
///
/// Copied from ScripTree's `icon-printer.svg` (the operator's own art; header §4b) — the reuse ui-spec §8.12 assigns to Print.
pub(super) const PRINTER: &str = include_str!("assets/printer.svg");

/// `read-mode.svg` — the art for [`super::Icon::ReadMode`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #7.
pub(super) const READ_MODE: &str = include_str!("assets/read-mode.svg");

/// `redact.svg` — the art for [`super::Icon::Redact`].
///
/// Authored for pdfcer in the header §3 style contract — the set's ONE filled glyph, an explicit rule-based exception (header §3).
pub(super) const REDACT: &str = include_str!("assets/redact.svg");

/// `redo.svg` — the art for [`super::Icon::Redo`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const REDO: &str = include_str!("assets/redo.svg");

/// `reset-layout.svg` — the art for [`super::Icon::ResetLayout`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #7.
pub(super) const RESET_LAYOUT: &str = include_str!("assets/reset-layout.svg");

/// `rotate-ccw.svg` — the art for [`super::Icon::RotateCcw`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const ROTATE_CCW: &str = include_str!("assets/rotate-ccw.svg");

/// `rotate-cw.svg` — the art for [`super::Icon::RotateCw`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const ROTATE_CW: &str = include_str!("assets/rotate-cw.svg");

/// `ruler.svg` — the art for [`super::Icon::Measure`].
///
/// Copied **verbatim** from ScripTree's `icon-ruler.svg` (the operator's own art; header §4).
pub(super) const RULER: &str = include_str!("assets/ruler.svg");

/// `rulers.svg` — the art for [`super::Icon::Rulers`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #7. Two ruled bands meeting at a corner; deliberately NOT [`RULER`]'s single band.
pub(super) const RULERS: &str = include_str!("assets/rulers.svg");

/// `save.svg` — the art for [`super::Icon::Save`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const SAVE: &str = include_str!("assets/save.svg");

/// `scissors.svg` — the art for [`super::Icon::Split`].
///
/// Copied **verbatim** from ScripTree's `icon-scissors.svg` (the operator's own art; header §4).
pub(super) const SCISSORS: &str = include_str!("assets/scissors.svg");

/// `search.svg` — the art for [`super::Icon::Search`].
///
/// Authored for pdfcer in the header §3 style contract — the unmarked lens of the magnifier family.
pub(super) const SEARCH: &str = include_str!("assets/search.svg");

/// `settings.svg` — the art for [`super::Icon::Settings`].
///
/// Copied from ScripTree's `icon-settings.svg` (the operator's own art; header §4b) — sliders rather than a cogwheel, for the reason the asset records.
pub(super) const SETTINGS: &str = include_str!("assets/settings.svg");

/// `pointer.svg` — the art for [`super::Icon::Pointer`].
///
/// Authored for pdfcer in the header §3 style contract — an arrow cursor with two option rules; the asset records why it is emphatically not `tool.svg`'s wrench.
pub(super) const POINTER: &str = include_str!("assets/pointer.svg");

/// `shape-arrow.svg` — the art for [`super::Icon::ShapeArrow`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const SHAPE_ARROW: &str = include_str!("assets/shape-arrow.svg");

/// `shape-cloud.svg` — the art for [`super::Icon::ShapeCloud`].
///
/// Authored for pdfcer in the header §3 style contract — nine outward arcs on a closed loop; the asset records why nine, why odd, and why the scallop rather than the outline is what carries the meaning.
pub(super) const SHAPE_CLOUD: &str = include_str!("assets/shape-cloud.svg");

/// `shape-ellipse.svg` — the art for [`super::Icon::ShapeEllipse`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const SHAPE_ELLIPSE: &str = include_str!("assets/shape-ellipse.svg");

/// `shape-ink.svg` — the art for [`super::Icon::ShapeInk`].
///
/// Authored for pdfcer in the header §3 style contract — one irregular flowing stroke, and the asset records why it is emphatically not `text-squiggly.svg`'s periodic wave.
pub(super) const SHAPE_INK: &str = include_str!("assets/shape-ink.svg");

/// `shape-polygon.svg` — the art for [`super::Icon::ShapePolygon`].
///
/// Authored for pdfcer in the header §3 style contract — an IRREGULAR closed pentagon; the asset records why regularity and why four corners would both be wrong.
pub(super) const SHAPE_POLYGON: &str = include_str!("assets/shape-polygon.svg");

/// `shape-polyline.svg` — the art for [`super::Icon::ShapePolyline`].
///
/// Authored for pdfcer in the header §3 style contract — `shape-polygon.svg` with its closing segment removed, which is exactly how the two annotations differ.
pub(super) const SHAPE_POLYLINE: &str = include_str!("assets/shape-polyline.svg");

/// `shape-highlight.svg` — the art for [`super::Icon::ShapeHighlight`].
///
/// Authored for pdfcer in the header §3 style contract — the one asset with a 1-unit stroke (its 45° hatch is a texture, not a contour).
pub(super) const SHAPE_HIGHLIGHT: &str = include_str!("assets/shape-highlight.svg");

/// `shape-rect.svg` — the art for [`super::Icon::ShapeRect`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const SHAPE_RECT: &str = include_str!("assets/shape-rect.svg");

/// `show-points.svg` — the art for [`super::Icon::ShowPoints`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #5.
pub(super) const SHOW_POINTS: &str = include_str!("assets/show-points.svg");

/// `sidebar.svg` — the art for [`super::Icon::Sidebar`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const SIDEBAR: &str = include_str!("assets/sidebar.svg");

/// `signatures.svg` — the art for [`super::Icon::Signatures`].
///
/// Authored for pdfcer in the header §3 style contract — header §5 addition #4, and emphatically not a seal, badge, shield or checkmark.
pub(super) const SIGNATURES: &str = include_str!("assets/signatures.svg");

/// `sign.svg` — the art for [`super::Icon::Sign`].
pub(super) const SIGN: &str = include_str!("assets/sign.svg");

/// `stamp.svg` — the art for [`super::Icon::Stamp`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const STAMP: &str = include_str!("assets/stamp.svg");

/// `text-freetext.svg` — the art for [`super::Icon::TextFreeText`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const TEXT_FREETEXT: &str = include_str!("assets/text-freetext.svg");

/// `text-select.svg` — the art for [`super::Icon::TextSelect`].
///
/// Authored for pdfcer in the header §3 style contract — a bare I-beam for the View ▸ Navigate text tool, which is literally the cursor that tool installs. See the asset for why it is not `add-text.svg` without its badge.
pub(super) const TEXT_SELECT: &str = include_str!("assets/text-select.svg");

/// `text-squiggly.svg` — the art for [`super::Icon::TextSquiggly`].
///
/// Authored for pdfcer in the header §3 style contract — the wavy member of the text-markup family, whose four lobes are a legibility decision the asset records.
pub(super) const TEXT_SQUIGGLY: &str = include_str!("assets/text-squiggly.svg");

/// `text-caret.svg` — the art for [`super::Icon::TextCaret`].
///
/// Authored for pdfcer in the header §3 style contract — a parted text line with a caret pointing up into the parting.
pub(super) const TEXT_CARET: &str = include_str!("assets/text-caret.svg");

/// `text-replace.svg` — the art for [`super::Icon::TextReplace`].
///
/// Authored for pdfcer in the header §3 style contract — a struck text line with a caret under its right end.
pub(super) const TEXT_REPLACE: &str = include_str!("assets/text-replace.svg");

/// `text-sticky.svg` — the art for [`super::Icon::TextSticky`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const TEXT_STICKY: &str = include_str!("assets/text-sticky.svg");

/// `text-strikeout.svg` — the art for [`super::Icon::TextStrikeout`].
///
/// Authored for pdfcer in the header §3 style contract — text-underline's sibling with the rule moved between the text lines.
pub(super) const TEXT_STRIKEOUT: &str = include_str!("assets/text-strikeout.svg");

/// `text-underline.svg` — the art for [`super::Icon::TextUnderline`].
///
/// Authored for pdfcer in the header §3 style contract — the first of the three text-markup glyphs, which differ only in where the third stroke goes.
pub(super) const TEXT_UNDERLINE: &str = include_str!("assets/text-underline.svg");

/// `text.svg` — the art for [`super::Icon::Text`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const TEXT: &str = include_str!("assets/text.svg");

/// `tool.svg` — the art for [`super::Icon::Tools`].
///
/// Copied **verbatim** from ScripTree's `icon-tool.svg` (the operator's own art; header §4).
pub(super) const TOOL: &str = include_str!("assets/tool.svg");

/// `undo.svg` — the art for [`super::Icon::Undo`].
///
/// Authored for pdfcer in the header §3 style contract.
pub(super) const UNDO: &str = include_str!("assets/undo.svg");

/// `upload.svg` — the art for [`super::Icon::InsertPages`].
///
/// Copied **verbatim** from ScripTree's `icon-upload.svg` (the operator's own art; header §4).
pub(super) const UPLOAD: &str = include_str!("assets/upload.svg");

/// `zoom-in.svg` — the art for [`super::Icon::ZoomIn`].
///
/// Derived from ScripTree's `icon-search.svg`: the same magnifier, plus a cross in the lens (header §4).
pub(super) const ZOOM_IN: &str = include_str!("assets/zoom-in.svg");

/// `zoom-out.svg` — the art for [`super::Icon::ZoomOut`].
///
/// Derived from ScripTree's `icon-search.svg`: the same magnifier, plus a minus bar in the lens (header §4).
pub(super) const ZOOM_OUT: &str = include_str!("assets/zoom-out.svg");

/// `zoom-region.svg` — the art for [`super::Icon::ZoomRegion`].
///
/// Authored for pdfcer in the header §3 style contract — the fourth member of ui-spec §3.1's magnifier family, carrying a BOX in the lens.
pub(super) const ZOOM_REGION: &str = include_str!("assets/zoom-region.svg");

/// `zoom-selection.svg` — the art for [`super::Icon::ZoomSelection`].
///
/// Authored for pdfcer in the header §3 style contract — ui-spec §3.1's corner-bracket family, reduced to a diagonal PAIR so it cannot be read as [`FIT_PAGE`].
pub(super) const ZOOM_SELECTION: &str = include_str!("assets/zoom-selection.svg");

// ===========================================================================
// The selection filter's rows
// ===========================================================================
//
// Five glyphs for `canvas::pick`'s eleven-row popup. The other six rows reuse
// icons the set already had (`text-select`, `image`, `show-points`, `markup`,
// `ruler`, `form-field`), because reuse of an ICON is free while reuse of an
// ASSET is a decision: `icons::catalog::tests::only_the_documented_assets_are_shared`
// holds every asset to exactly one `Icon` unless the pair is listed in
// `SHARED_PAIRS` with its argument. These five exist because no glyph in the
// set meant what their row means — see each file's embedded comment for which
// neighbour it had to stay distinguishable from, and why.

/// `pick-text.svg` — the art for [`super::Icon::PickText`].
pub(super) const PICK_TEXT: &str = include_str!("assets/pick-text.svg");

/// `pick-path.svg` — the art for [`super::Icon::PickPath`].
pub(super) const PICK_PATH: &str = include_str!("assets/pick-path.svg");

/// `pick-part.svg` — the art for [`super::Icon::PickPart`].
pub(super) const PICK_PART: &str = include_str!("assets/pick-part.svg");

/// `pick-form-xobject.svg` — the art for [`super::Icon::PickFormXObject`].
pub(super) const PICK_FORM_XOBJECT: &str = include_str!("assets/pick-form-xobject.svg");

/// `pick-link.svg` — the art for [`super::Icon::PickLink`].
pub(super) const PICK_LINK: &str = include_str!("assets/pick-link.svg");

// ══════════════════════════════════════════════════════════════════════════
// The glyphs that discharge a written-down refusal, or unshare a borrowed one
// ══════════════════════════════════════════════════════════════════════════
//
// Every one of these fills a gap that was already WRITTEN DOWN. Some close a
// registration carrying a "No icon" refusal in prose — `file.new`, `file.ocr`,
// `markup.finish` and the rest — and those refusals all give the same reason:
// this directory is declared the operator's own art, reusing a neighbour's
// glyph would make two controls say the same thing, and naming a key that does
// not exist draws a slashed placeholder. **A refusal of that kind is
// discharged by the art existing, not by an argument.**
//
// The rest replace a BORROWED glyph. Four form-field tools shared
// `form-field.svg` and four measure tools shared `measure.svg`: eight
// controls rendering as two pictures, which is the failure the set's
// one-asset-per-role rule exists to prevent.
//
// Provenance: these are drawn from primitives for pdfcer in the same style
// contract as the rest of the directory — 48×48, stroke 2.5, round caps and
// joins, no fill except the redaction family. `assets/PROVENANCE.md` covers
// the whole directory and its terms cover these too.

/// `apply-redactions.svg` — the art for [`super::Icon::ApplyRedactions`].
///
/// Edit ▸ Apply redactions (`edit.redact_apply`) — the one irreversible verb
/// in the redaction family.
pub(super) const APPLY_REDACTIONS: &str = include_str!("assets/apply-redactions.svg");

/// `attachment.svg` — the art for [`super::Icon::Attachment`].
///
/// Attachments (`edit.attachments`) — the files this document carries.
pub(super) const ATTACHMENT: &str = include_str!("assets/attachment.svg");

/// `sound.svg` — the art for [`super::Icon::Sound`].
///
/// Authored for pdfcer in the header §3 style contract — a speaker cone with two waves.
pub(super) const SOUND: &str = include_str!("assets/sound.svg");

/// `screen.svg` — the art for [`super::Icon::Screen`].
///
/// Authored for pdfcer in the header §3 style contract — a frame with a play triangle.
pub(super) const SCREEN: &str = include_str!("assets/screen.svg");

/// `check.svg` — the art for [`super::Icon::Accept`].
///
/// Complete the gesture in progress — `markup.finish` and `measure.finish`.
pub(super) const CHECK: &str = include_str!("assets/check.svg");

/// `check-box.svg` — the art for [`super::Icon::CheckBox`].
///
/// Place a **check box** — one independent on/off box.
pub(super) const CHECK_BOX: &str = include_str!("assets/check-box.svg");

/// `close-others.svg` — the art for [`super::Icon::CloseOthers`].
///
/// Close every open document except one — `view.close_other_documents`.
pub(super) const CLOSE_OTHERS: &str = include_str!("assets/close-others.svg");

/// `collapse.svg` — the art for [`super::Icon::Collapse`].
///
/// A tree row whose children are **showing** — press to hide them.
pub(super) const COLLAPSE: &str = include_str!("assets/collapse.svg");

/// `copy-document-text.svg` — the art for [`super::Icon::CopyDocumentText`].
///
/// Copy the whole document's text to the clipboard — `file.copy_document_text`.
pub(super) const COPY_DOCUMENT_TEXT: &str = include_str!("assets/copy-document-text.svg");

/// `copy-page-text.svg` — the art for [`super::Icon::CopyPageText`].
///
/// Copy this page's text to the clipboard — `file.copy_page_text`.
pub(super) const COPY_PAGE_TEXT: &str = include_str!("assets/copy-page-text.svg");

/// `dimension-groups.svg` — the art for [`super::Icon::DimensionGroups`].
///
/// Dimension groups — `measure.manage_groups`, and the caption's dock tab.
pub(super) const DIMENSION_GROUPS: &str = include_str!("assets/dimension-groups.svg");

/// `document-next.svg` — the art for [`super::Icon::NextDocument`].
///
/// Switch to the next open document — `view.next_document` (Ctrl+Tab).
pub(super) const DOCUMENT_NEXT: &str = include_str!("assets/document-next.svg");

/// `document-previous.svg` — the art for [`super::Icon::PreviousDocument`].
///
/// Switch to the previous open document — `view.previous_document`.
pub(super) const DOCUMENT_PREVIOUS: &str = include_str!("assets/document-previous.svg");

/// `drop-down.svg` — the art for [`super::Icon::DropDown`].
///
/// Place a **drop-down** (the `/Ch` choice field).
pub(super) const DROP_DOWN: &str = include_str!("assets/drop-down.svg");

/// `embed-fonts.svg` — the art for [`super::Icon::EmbedFonts`].
///
/// Embed the font programs a document references but does not carry.
pub(super) const EMBED_FONTS: &str = include_str!("assets/embed-fonts.svg");

/// `expand.svg` — the art for [`super::Icon::Expand`].
///
/// A tree row whose children are **hidden** — press to reveal them.
pub(super) const EXPAND: &str = include_str!("assets/expand.svg");

/// `finish-shape.svg` — the art for [`super::Icon::FinishShape`].
///
/// Markup ▸ Finish shape (`markup.finish`) — place the polyline or polygon
/// whose corners have been clicked out.
pub(super) const FINISH_SHAPE: &str = include_str!("assets/finish-shape.svg");

/// `lock.svg` — the art for [`super::Icon::Locked`].
///
/// A row the **document** forbids changing — an optional-content group
/// carrying §12.5.3's `/Locked` bit.
pub(super) const LOCK: &str = include_str!("assets/lock.svg");

/// `measure-angle.svg` — the art for [`super::Icon::MeasureAngle`].
///
/// Two-line measurement — `measure.two_line`.
pub(super) const MEASURE_ANGLE: &str = include_str!("assets/measure-angle.svg");

/// `measure-area.svg` — the art for [`super::Icon::MeasureArea`].
///
/// Area measurement — `measure.area`.
pub(super) const MEASURE_AREA: &str = include_str!("assets/measure-area.svg");

/// `snapshot.svg` — the art for [`super::Icon::Snapshot`].
///
/// The snapshot box — `view.tool_snapshot`.
pub(super) const SNAPSHOT: &str = include_str!("assets/snapshot.svg");

/// `measure-length.svg` — the art for [`super::Icon::MeasureLength`].
///
/// Path-length measurement — `measure.length`.
pub(super) const MEASURE_LENGTH: &str = include_str!("assets/measure-length.svg");

/// `measure-perimeter.svg` — the art for [`super::Icon::MeasurePerimeter`].
///
/// Perimeter measurement — `measure.perimeter`.
pub(super) const MEASURE_PERIMETER: &str = include_str!("assets/measure-perimeter.svg");

/// `measure-radius.svg` — the art for [`super::Icon::MeasureRadius`].
///
/// Radius / diameter measurement — `measure.radius_diameter`.
pub(super) const MEASURE_RADIUS: &str = include_str!("assets/measure-radius.svg");

/// `merge.svg` — the art for [`super::Icon::MergeInto`].
///
/// Merge another file's pages INTO the open document (`pages.merge_into`).
pub(super) const MERGE: &str = include_str!("assets/merge.svg");

/// `new-document.svg` — the art for [`super::Icon::New`].
///
/// New (blank) document — `file.new`.
pub(super) const NEW_DOCUMENT: &str = include_str!("assets/new-document.svg");

/// `new-from-template.svg` — the art for [`super::Icon::NewFromTemplate`].
///
/// New document from a template — `file.new_from_template`.
pub(super) const NEW_FROM_TEMPLATE: &str = include_str!("assets/new-from-template.svg");

/// `push-button.svg` — the art for [`super::Icon::PushButton`].
///
/// Place a **push button** — the `/Btn` field with no on/off state.
pub(super) const PUSH_BUTTON: &str = include_str!("assets/push-button.svg");

/// `put-down.svg` — the art for [`super::Icon::PutDown`].
///
/// Put the armed tool down — the Tool panel's row 4.
pub(super) const PUT_DOWN: &str = include_str!("assets/put-down.svg");

/// `radio-button.svg` — the art for [`super::Icon::RadioButton`].
///
/// Place a **radio button** — one of a mutually exclusive set.
pub(super) const RADIO_BUTTON: &str = include_str!("assets/radio-button.svg");

/// `recent.svg` — the art for [`super::Icon::Recent`].
///
/// Recently-opened documents — `file.recent`, the menu button in File ▸ File.
pub(super) const RECENT: &str = include_str!("assets/recent.svg");

/// `recognise-text.svg` — the art for [`super::Icon::RecogniseText`].
///
/// Recognise text (OCR) — `file.ocr`.
pub(super) const RECOGNISE_TEXT: &str = include_str!("assets/recognise-text.svg");

/// `off-page.svg` — the art for [`super::Icon::OffPage`].
pub(super) const OFF_PAGE: &str = include_str!("assets/off-page.svg");

/// `redact-selection.svg` — the art for [`super::Icon::RedactSelection`].
///
/// Edit ▸ Redact selection (`edit.redact_selection`) — mark whatever is
/// selected — a shape, an image, a piece of text — to be removed.
pub(super) const REDACT_SELECTION: &str = include_str!("assets/redact-selection.svg");

/// `reflow.svg` — the art for [`super::Icon::Reflow`].
///
/// Reflow paragraph (`edit.reflow_block`) — re-wrap the paragraph the caret
/// is in so its lines fill their box again.
pub(super) const REFLOW: &str = include_str!("assets/reflow.svg");

/// `render-diagnostics.svg` — the art for [`super::Icon::RenderDiagnostics`].
///
/// Report how the page was actually drawn (`tools.render_diagnostics`).
pub(super) const RENDER_DIAGNOSTICS: &str = include_str!("assets/render-diagnostics.svg");

/// `save-as.svg` — the art for [`super::Icon::SaveAs`].
///
/// Save As — `file.save_as` (`OPERATOR_REQUESTS.md` O95).
pub(super) const SAVE_AS: &str = include_str!("assets/save-as.svg");

/// `save-compact.svg` — the art for [`super::Icon::SaveCompacted`].
///
/// Save compacted — `file.save_compacted`, the copy written fresh with
/// anything no longer used dropped.
pub(super) const SAVE_COMPACT: &str = include_str!("assets/save-compact.svg");

/// `save-copy.svg` — the art for [`super::Icon::SaveCopy`].
///
/// Save a copy — `file.save_copy`.
pub(super) const SAVE_COPY: &str = include_str!("assets/save-copy.svg");

/// `unembed-fonts.svg` — the art for [`super::Icon::UnembedFonts`].
///
/// Remove embedded font programs, leaving the references behind.
pub(super) const UNEMBED_FONTS: &str = include_str!("assets/unembed-fonts.svg");

/// `wheel-flip.svg` — the art for [`super::Icon::WheelFlip`].
///
/// The wheel-paging toggle on the status bar — `OPERATOR_REQUESTS.md` O30.
pub(super) const WHEEL_FLIP: &str = include_str!("assets/wheel-flip.svg");

// ── the three aliases, broken ─────────────────────────────────────────────
//
// `properties`, `insert-pages` and `set-scale` were live icon KEYS resolving
// to another role's asset: `document.svg`, `upload.svg` and `convert.svg`.
// That is the same defect as four form tools sharing one glyph — a control
// wearing a picture drawn for something else — one level down. Each of the
// three below is the FIRST art drawn for its role.
//
// ⇒ **A proposed name colliding with an existing key does not mean there is
// art to restyle.** Filtering such names out mechanically is what hid these
// three: the key existed, the drawing for that role did not. Check what the
// colliding key actually resolves to before dropping the proposal.

/// `insert-pages.svg` — the art for [`super::Icon::InsertPages`].
///
/// Replaces an alias, not a drawing. See the asset for which glyph it must
/// stay distinguishable from and by what cue.
pub(super) const INSERT_PAGES: &str = include_str!("assets/insert-pages.svg");

/// `properties.svg` — the art for [`super::Icon::Properties`].
///
/// Replaces an alias, not a drawing. See the asset for which glyph it must
/// stay distinguishable from and by what cue.
pub(super) const PROPERTIES: &str = include_str!("assets/properties.svg");

/// `set-scale.svg` — the art for [`super::Icon::SetScale`].
///
/// Replaces an alias, not a drawing. See the asset for which glyph it must
/// stay distinguishable from and by what cue.
pub(super) const SET_SCALE: &str = include_str!("assets/set-scale.svg");

// ══════════════════════════════════════════════════════════════════════════
// Export image, copy as vector, encrypt, permissions, open in Acrobat
// ══════════════════════════════════════════════════════════════════════════
//
// Five glyphs, each named by a registered command:
//
// | glyph | command | where it is drawn |
// |---|---|---|
// | `export-image` | `file.export_image` | File ▸ Export |
// | `copy-as-vector` | `edit.copy_as_vector` (token 408) | Edit ▸ Clipboard, icon-only |
// | `encrypt` | `file.encrypt` (126) | Security ▸ Security, large |
// | `permissions` | `file.permissions` (127) | Security ▸ Security, large |
// | `open-in-acrobat` | `file.open_in_acrobat` | the ribbon's trailing item |
//
// ⇒ **Do not write "this is not built" into a comment as a fact about the
// future.** A claim anchored to a schedule rots; a claim anchored to a
// mechanism does not — which is why the paragraph at the foot of this block,
// about `Icon::ALL` membership and the tests that walk it, stays true in both
// directions while a sentence about what the ribbon does not reach yet does
// not survive the week. Write such a claim so a test can assert it, or expect
// to come back and correct it.
//
// The argument for each PICTURE is unaffected by whether a button exists, and
// is what this block is for.
//
// `file.export_image` first wore `export` (`download.svg`), on a paragraph in
// `shell::commands::catalog::file` defending the share — three export verbs,
// one act, and the FORMAT is "a word only a label can say". That argument is
// right about DXF and form data and wrong about a picture, because this set
// already draws a picture as a subject: `image.svg` exists and `insert-image`
// wears it, so the operator has already learned what a framed tile with a
// horizon means here. The registration is repointed and its comment records
// the reversal rather than being quietly rewritten.
//
// * `open-in-acrobat` — the ribbon's trailing item. The command is
//   `enabled_when("doc.open")`; it is the ribbon ITEM that carries
//   `shown_when("acrobat.available")`, which is where R9 is enforced for it.
// * `copy-as-vector` — the clipboard's missing copy-out, `edit.copy_as_vector`
//   (token 408), drawn icon-only beside Cut / Copy / Paste on Edit ▸
//   Clipboard. Drawn first so the layout mockup could put the proposal in
//   front of the operator as a picture rather than as a sentence, which is
//   what got it built. ⇒ **A mockup that asks the question as a picture is a
//   mechanism, not decoration.**
// * `encrypt` and `permissions` — the engine grew both and nothing in this GUI
//   reached either. `OPERATOR_REQUESTS.md` O119 is the question to him, and it
//   is a question about a SURFACE (a password box, a permission list, a save
//   that rewrites the file), not about a button: the art did not pre-empt his
//   answer, it let the mockup ask. He answered *"yes add encryption and
//   permissions"*, and both ship as `file.encrypt` / `file.permissions` on
//   Security ▸ Security, both large, exactly where the mockup drew them.
//
// ⇒ A variant with no command is the SUPPORTED state, not a loose end.
// [`super::Icon::EditObjects`] is the standing precedent — its command is
// deleted and its variant remains — and the reason is mechanical rather than
// sentimental: `every_icon_parses`,
// `every_icon_rasterizes_to_visible_pixels`,
// `fill_is_semantic_and_the_set_that_uses_it_is_closed`,
// `crlf_line_endings_parse_identically` and
// `no_two_icons_render_as_the_same_picture` all iterate
// [`super::Icon::ALL`]. Art kept outside that list is art no test walks, and
// untested art rots quietly until somebody wires it up months later and finds
// it blank.
//
// Provenance: drawn from primitives for pdfcer in the same style
// contract as the rest of the directory — 48×48, stroke 2.5, round caps and
// joins, no fill except the redaction family. `assets/PROVENANCE.md` covers
// the whole directory. ⚠ `open-in-acrobat` names a vendor in its LABEL and
// carries nothing of that vendor's mark in its ART; see the asset's own
// comment, which states the constraint first because the label is what
// invites the mistake.

/// `copy-as-vector.svg` — the art for [`super::Icon::CopyAsVector`].
pub(super) const COPY_AS_VECTOR: &str = include_str!("assets/copy-as-vector.svg");

/// `encrypt.svg` — the art for [`super::Icon::Encrypt`].
///
/// Put a password on this document — the engine's `set_encryption`. Worn by
/// `file.encrypt` (token 126) on Security ▸ Security.
pub(super) const ENCRYPT: &str = include_str!("assets/encrypt.svg");

/// `export-image.svg` — the art for [`super::Icon::ExportImage`].
pub(super) const EXPORT_IMAGE: &str = include_str!("assets/export-image.svg");

/// `replace-image.svg` — the art for [`super::Icon::ReplaceImage`].
pub(super) const REPLACE_IMAGE: &str = include_str!("assets/replace-image.svg");

/// `open-in-acrobat.svg` — the art for [`super::Icon::OpenInAcrobat`].
pub(super) const OPEN_IN_ACROBAT: &str = include_str!("assets/open-in-acrobat.svg");

/// `permissions.svg` — the art for [`super::Icon::Permissions`].
pub(super) const PERMISSIONS: &str = include_str!("assets/permissions.svg");

/// `select-all.svg` — the art for [`super::Icon::SelectAll`].
pub(super) const SELECT_ALL: &str = include_str!("assets/select-all.svg");

/// `bold.svg` — the art for [`super::Icon::Bold`].
pub(super) const BOLD: &str = include_str!("assets/bold.svg");

/// `italic.svg` — the art for [`super::Icon::Italic`].
pub(super) const ITALIC: &str = include_str!("assets/italic.svg");

/// `line-weights.svg` — the art for [`super::Icon::LineWeights`].
pub(super) const LINE_WEIGHTS: &str = include_str!("assets/line-weights.svg");

/// `skip-tiny.svg` — the art for [`super::Icon::SkipTiny`].
pub(super) const SKIP_TINY: &str = include_str!("assets/skip-tiny.svg");

/// `ink-picker.svg` — the art for [`super::Icon::InkPicker`].
pub(super) const INK_PICKER: &str = include_str!("assets/ink-picker.svg");

/// `align-before.svg` — the art for [`super::Icon::AlignBefore`].
pub(super) const ALIGN_BEFORE: &str = include_str!("assets/align-before.svg");

/// `align-left.svg` — the art for [`super::Icon::AlignLeft`].
pub(super) const ALIGN_LEFT: &str = include_str!("assets/align-left.svg");

/// `align-centre-h.svg` — the art for [`super::Icon::AlignCentreH`].
pub(super) const ALIGN_CENTRE_H: &str = include_str!("assets/align-centre-h.svg");

/// `align-right.svg` — the art for [`super::Icon::AlignRight`].
pub(super) const ALIGN_RIGHT: &str = include_str!("assets/align-right.svg");

/// `align-after.svg` — the art for [`super::Icon::AlignAfter`].
pub(super) const ALIGN_AFTER: &str = include_str!("assets/align-after.svg");

/// `align-text-h.svg` — the art for [`super::Icon::AlignTextH`].
pub(super) const ALIGN_TEXT_H: &str = include_str!("assets/align-text-h.svg");

/// `align-above.svg` — the art for [`super::Icon::AlignAbove`].
pub(super) const ALIGN_ABOVE: &str = include_str!("assets/align-above.svg");

/// `align-top.svg` — the art for [`super::Icon::AlignTop`].
pub(super) const ALIGN_TOP: &str = include_str!("assets/align-top.svg");

/// `align-centre-v.svg` — the art for [`super::Icon::AlignCentreV`].
pub(super) const ALIGN_CENTRE_V: &str = include_str!("assets/align-centre-v.svg");

/// `align-bottom.svg` — the art for [`super::Icon::AlignBottom`].
pub(super) const ALIGN_BOTTOM: &str = include_str!("assets/align-bottom.svg");

/// `align-below.svg` — the art for [`super::Icon::AlignBelow`].
pub(super) const ALIGN_BELOW: &str = include_str!("assets/align-below.svg");

/// `align-text-v.svg` — the art for [`super::Icon::AlignTextV`].
pub(super) const ALIGN_TEXT_V: &str = include_str!("assets/align-text-v.svg");

/// `distribute-left.svg` — the art for [`super::Icon::DistributeLeft`].
pub(super) const DISTRIBUTE_LEFT: &str = include_str!("assets/distribute-left.svg");

/// `distribute-centre-h.svg` — the art for [`super::Icon::DistributeCentreH`].
pub(super) const DISTRIBUTE_CENTRE_H: &str = include_str!("assets/distribute-centre-h.svg");

/// `distribute-right.svg` — the art for [`super::Icon::DistributeRight`].
pub(super) const DISTRIBUTE_RIGHT: &str = include_str!("assets/distribute-right.svg");

/// `distribute-gaps-h.svg` — the art for [`super::Icon::DistributeGapsH`].
pub(super) const DISTRIBUTE_GAPS_H: &str = include_str!("assets/distribute-gaps-h.svg");

/// `distribute-text-h.svg` — the art for [`super::Icon::DistributeTextH`].
pub(super) const DISTRIBUTE_TEXT_H: &str = include_str!("assets/distribute-text-h.svg");

/// `distribute-top.svg` — the art for [`super::Icon::DistributeTop`].
pub(super) const DISTRIBUTE_TOP: &str = include_str!("assets/distribute-top.svg");

/// `distribute-centre-v.svg` — the art for [`super::Icon::DistributeCentreV`].
pub(super) const DISTRIBUTE_CENTRE_V: &str = include_str!("assets/distribute-centre-v.svg");

/// `distribute-bottom.svg` — the art for [`super::Icon::DistributeBottom`].
pub(super) const DISTRIBUTE_BOTTOM: &str = include_str!("assets/distribute-bottom.svg");

/// `distribute-gaps-v.svg` — the art for [`super::Icon::DistributeGapsV`].
pub(super) const DISTRIBUTE_GAPS_V: &str = include_str!("assets/distribute-gaps-v.svg");

/// `distribute-text-v.svg` — the art for [`super::Icon::DistributeTextV`].
pub(super) const DISTRIBUTE_TEXT_V: &str = include_str!("assets/distribute-text-v.svg");

/// `exchange-selection.svg` — the art for [`super::Icon::ExchangeSelection`].
pub(super) const EXCHANGE_SELECTION: &str = include_str!("assets/exchange-selection.svg");

/// `exchange-stacking.svg` — the art for [`super::Icon::ExchangeStacking`].
pub(super) const EXCHANGE_STACKING: &str = include_str!("assets/exchange-stacking.svg");

/// `exchange-clockwise.svg` — the art for [`super::Icon::ExchangeClockwise`].
pub(super) const EXCHANGE_CLOCKWISE: &str = include_str!("assets/exchange-clockwise.svg");

/// `randomize.svg` — the art for [`super::Icon::Randomize`].
pub(super) const RANDOMIZE: &str = include_str!("assets/randomize.svg");

/// `unclump.svg` — the art for [`super::Icon::Unclump`].
pub(super) const UNCLUMP: &str = include_str!("assets/unclump.svg");

/// `remove-overlaps.svg` — the art for [`super::Icon::RemoveOverlaps`].
pub(super) const REMOVE_OVERLAPS: &str = include_str!("assets/remove-overlaps.svg");

/// `arrange-grid.svg` — the art for [`super::Icon::ArrangeGrid`].
pub(super) const ARRANGE_GRID: &str = include_str!("assets/arrange-grid.svg");

/// `arrange-circular.svg` — the art for [`super::Icon::ArrangeCircular`].
pub(super) const ARRANGE_CIRCULAR: &str = include_str!("assets/arrange-circular.svg");

/// `nodes-align-vertical.svg` — the art for [`super::Icon::NodesAlignVertical`].
pub(super) const NODES_ALIGN_VERTICAL: &str = include_str!("assets/nodes-align-vertical.svg");

/// `nodes-align-horizontal.svg` — the art for [`super::Icon::NodesAlignHorizontal`].
pub(super) const NODES_ALIGN_HORIZONTAL: &str = include_str!("assets/nodes-align-horizontal.svg");

/// `nodes-spread-across.svg` — the art for [`super::Icon::NodesSpreadAcross`].
pub(super) const NODES_SPREAD_ACROSS: &str = include_str!("assets/nodes-spread-across.svg");

/// `nodes-spread-down.svg` — the art for [`super::Icon::NodesSpreadDown`].
pub(super) const NODES_SPREAD_DOWN: &str = include_str!("assets/nodes-spread-down.svg");

/// `align-centre.svg` — the art for [`super::Icon::AlignCentre`].
pub(super) const ALIGN_CENTRE: &str = include_str!("assets/align-centre.svg");

/// `align-panel.svg` — the art for [`super::Icon::AlignPanel`].
pub(super) const ALIGN_PANEL: &str = include_str!("assets/align-panel.svg");

/// `model-3d.svg` — the art for [`super::Icon::Model3d`].
pub(super) const MODEL_3D: &str = include_str!("assets/model-3d.svg");

/// `para-align-left.svg` — the art for [`super::Icon::ParaAlignLeft`].
pub(super) const PARA_ALIGN_LEFT: &str = include_str!("assets/para-align-left.svg");

/// `para-align-centre.svg` — the art for [`super::Icon::ParaAlignCentre`].
pub(super) const PARA_ALIGN_CENTRE: &str = include_str!("assets/para-align-centre.svg");

/// `para-align-right.svg` — the art for [`super::Icon::ParaAlignRight`].
pub(super) const PARA_ALIGN_RIGHT: &str = include_str!("assets/para-align-right.svg");

/// `para-justify.svg` — the art for [`super::Icon::ParaJustify`].
pub(super) const PARA_JUSTIFY: &str = include_str!("assets/para-justify.svg");

/// `underline.svg` — the art for [`super::Icon::Underline`].
pub(super) const UNDERLINE: &str = include_str!("assets/underline.svg");

/// `strikethrough.svg` — the art for [`super::Icon::Strikethrough`].
pub(super) const STRIKETHROUGH: &str = include_str!("assets/strikethrough.svg");

/// `window-new.svg` — the art for [`super::Icon::WindowNew`].
pub(super) const WINDOW_NEW: &str = include_str!("assets/window-new.svg");

/// `window-move.svg` — the art for [`super::Icon::WindowMove`].
pub(super) const WINDOW_MOVE: &str = include_str!("assets/window-move.svg");
