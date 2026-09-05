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
//! ## ★ Why the art lives in `src/icons/assets/` rather than a top-level
//! ## `crates/pdfcer-gui/assets/`
//!
//! The salvage source read its art with
//! `include_str!("../assets/icons/<name>.svg")` — a sibling directory of
//! `src/`. Here the same `include_str!` reaches a directory *inside* the
//! icon module. Two reasons:
//!
//! 1. **This module's write territory is `src/icons/`.** The rebuild runs
//!    several agents in parallel over one tree, and the boundaries between
//!    them are directories. Creating `crates/pdfcer-gui/assets/` would put
//!    art outside the territory the icon work owns.
//! 2. **Co-location.** The art, the catalogue that names it, the parser that
//!    reads it and the painter that draws it are one subject. A reader who
//!    opens `src/icons/` sees all of it.
//!
//! `include_str!` — rather than a runtime file read — is carried across
//! unchanged, and its reason is unchanged: pdfcer ships single-folder
//! portable, so the executable must not depend on an `assets/` directory
//! travelling beside it. The whole set is **79 files, 82,336 bytes** of text
//! (measured 2026-08-14) and an icon that fails to load at startup is not a
//! failure mode worth having. (It was 46 files and 34 KB when this module
//! landed; the 2026-08-14 pass that filled the ribbon's remaining text
//! buttons added 25. Roughly half of the growth is the embedded rationale
//! comments, which are the point of copying assets verbatim rather than
//! re-emitting them — see §4.)
//!
//! ★ **That figure read "75 files, ~76 KB" until 2026-08-14 and was wrong by
//! four files**, having been written before the three Phase 6 markup glyphs
//! and `text-select` landed. It is the same defect `HANDOFF.md` §10 names —
//! *prose that quotes a number drifts from the number* — and it is corrected
//! here rather than silently updated, because the correction is the fifth
//! instance and the pattern is the useful part. Unlike `Icon::ALL`'s size,
//! which `catalog.rs` pins with an assertion, **nothing tests this one**: it
//! counts files on disk, and no test walks the directory.
//!
//! ## Licensing
//!
//! `assets/PROVENANCE.md` is the licensing record for this directory, and
//! `tools/gates/check-shipped-assets.py` requires it to exist and to name
//! terms. In one line: the art is the operator's own, under the project's own
//! MIT licence, which is why it needs **no** entry in `about.hbs` — the
//! shipped `LICENSE` already covers it, and there is no third-party grant to
//! reproduce. §1 below is the primary record of how that was established.
//!
//! ## ★ Why the SVG text is NOT inlined into Rust source
//!
//! The obvious alternative — a `const FOLDER: &str = r##"<svg …>"##;` with
//! the markup inline — was implemented first and then **withdrawn**,
//! because it fails `tools/gates/check-ui-strings.sh` on 138 lines and
//! cannot be exempted.
//!
//! That gate is a line-oriented scanner over `.rs` files looking for string
//! literals containing whitespace, which is its proxy for "prose that
//! belongs in the ui-text catalog". Almost every line of an SVG asset trips
//! it, because SVG *attribute values* are quoted strings full of spaces:
//! `viewBox="0 0 48 48"`, `d="M14 10h10M19 10v28"`. Its escape hatch is a
//! `// ui-text-exempt:` marker on the offending line or in the comment block
//! immediately above it — and neither can reach these lines, because they
//! are **inside a raw string**: a marker written there would become part of
//! the asset and be handed to the SVG parser.
//!
//! Keeping the art in `.svg` files is not a workaround for that gate; it is
//! the arrangement that makes the gate's question meaningful. XML markup is
//! not Rust source and should not be scanned as though it were. The finding
//! is recorded here because the inline form *looks* simpler and someone will
//! propose it again.
//!
//! ## ★ Regenerating
//!
//! The `.svg` files are copied from the salvage source; the constants below
//! are produced mechanically from the directory listing. If the set gains an
//! asset, drop the `.svg` in and add a constant here — **and add the
//! matching [`super::Icon`] variant to [`super::Icon::ALL`]**, or it ships
//! unverified (see `super::tests::every_icon_parses`).
//!
//! ---
//!
//! # Provenance
//!
//! Carried across from the salvage source's `assets/icons/PROVENANCE.md`,
//! which `docs/ui_specs/icon-set-and-toolbar.md` §7.2 required before any
//! art was bundled: the provenance of this set had to be **confirmed, not
//! assumed**. That record is a licensing artefact, so it travels with the
//! art it describes.
//!
//! ## §1 — Operator confirmation (the licensing question, closed)
//!
//! The ui-spec flagged an open question: were `D:\Dev\ScripTree\icons\*.svg`
//! drawn from scratch for ScripTree, or adapted from a third-party icon pack
//! (Feather, Lucide, Font Awesome, …) whose own licence would then travel
//! with them into pdfcer's asset tree?
//!
//! Answered directly by the operator (Ken), 2026-08-02, verbatim:
//!
//! > "Scriptree icons are mine, use from it what makes sense and create new
//! > ones in its style when necessary, try to make them close to what
//! > inkscape and Adobe use for similar commands without running into
//! > copyright issues."
//!
//! Consequences, all of them binding on this module:
//!
//! * The ScripTree source art is the operator's own work. He owns both
//!   projects, so no third-party licence travels with the copied files and
//!   nothing here needs an upstream attribution entry.
//! * pdfcer's own licence is MIT; these assets ship under it like the rest of
//!   the tree.
//! * `THIRD_PARTY_LICENSES.md` is unaffected — it is generated by
//!   `cargo-about` from the **dependency** set, and this set adds zero
//!   dependencies (see §6).
//!
//! ## §2 — The metaphor-not-artwork rule (binding on every future icon)
//!
//! The operator's instruction — *"try to make them close to what inkscape
//! and Adobe use for similar commands without running into copyright
//! issues"* — draws the line this module is held to:
//!
//! * **Allowed: metaphor-level resemblance.** A magnifier means zoom. A
//!   curved arrow means undo. Scissors mean split. Corner brackets mean fit
//!   to frame. These are industry conventions with no single author, and
//!   matching them is what makes a toolbar legible to someone arriving from
//!   Acrobat or Inkscape.
//! * **Forbidden: asset-level copying.** No tracing, no importing, no
//!   "adapting" of any Adobe or Inkscape SVG, icon font, or screenshot.
//!   Every glyph here was constructed from primitives (rectangles, circles,
//!   line segments, arcs), and every asset's embedded comment says which
//!   concept it depicts.
//! * This mirrors the standing rule that Acrobat and Inkscape are
//!   **behavioural** references only, never sources of GUI structure or art.
//!
//! This rule is *stricter* than copyright law strictly requires for simple
//! geometric glyphs. That is deliberate: it removes the question entirely
//! rather than leaving a judgement call in a file nobody will re-examine.
//!
//! ## §3 — Style contract (every asset here obeys it)
//!
//! ```xml
//! <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48" fill="none" aria-hidden="true">
//!   <!-- Generic <concept> shape — placeholder, not a vendor trademark logo -->
//!   <path|rect|circle … stroke="currentColor" stroke-width="2.5"
//!         stroke-linecap="round" stroke-linejoin="round"/>
//! </svg>
//! ```
//!
//! * **48×48 viewBox**, content inset ~6–8 units from every edge. That edge
//!   length is [`super::svg::VIEWBOX`], and every coordinate is in it.
//! * **`fill="none"`, `stroke="currentColor"`** — pure outline, no baked
//!   colour. This is the property the whole pipeline rests on: one raster per
//!   icon, tinted per theme and per widget state at draw time. An asset with
//!   a hardcoded colour would break light/dark theming *structurally*, not
//!   just cosmetically.
//! * **`stroke-width="2.5"`**, round caps and joins throughout.
//! * A comment naming the concept and disclaiming trademark risk.
//!
//! ### The two deliberate exceptions
//!
//! `redact.svg` is the only asset with a **filled** shape
//! (`fill="currentColor"` on the redaction bar). ui-spec §8.1 makes this an
//! explicit, rule-based exception, not style drift: a solid bar is what
//! redaction actually leaves behind, and an outline-only glyph would
//! visually understate a feature that irreversibly removes content. **A
//! future icon audit must not "fix" it back to an outline.**
//! `super::tests::fill_is_semantic_and_the_set_that_uses_it_is_closed`
//! asserts both halves — that redaction is filled and that nothing outside a
//! named set is.
//!
//! ★ That test was called `redaction_is_the_only_filled_icon` until
//! 2026-08-19, when the arrow pair joined the set and the name stopped
//! being true. **Four doc comments went on citing the old name for sixteen
//! days**, and on 2026-09-04 an audit grepped for it, found it in zero test
//! bodies, and correctly reported that the cited gate did not exist. It did
//! exist — under a name none of its citations had been told about. A rename
//! that leaves its citations behind blinds a reader exactly as thoroughly as
//! a deletion, and costs the same to fix.
//!
//! `shape-highlight.svg` uses `stroke-width="1"` for its 45° hatch. Also
//! deliberate (ui-spec §3.3): the hatch is a *texture* standing in for
//! translucent highlighter colour, not a contour, and at 2.5 it would fill
//! the band solid and read as redaction.
//!
//! ## §4 — File-by-file origin
//!
//! **Copied verbatim from `D:\Dev\ScripTree\icons\`** (the operator's own
//! art), byte for byte apart from the filename. Renamed to pdfcer's **role**
//! rather than ScripTree's shape name so a future re-draw changes one asset
//! and no call site:
//!
//! | Here | ScripTree source | Role |
//! |---|---|---|
//! | `folder.svg` | `icon-folder.svg` | Open **and** Font folders — one asset, two roles |
//! | `document.svg` | `icon-document.svg` | Properties |
//! | `edit.svg` | `icon-edit.svg` | Edit Text |
//! | `tool.svg` | `icon-tool.svg` | Tools |
//! | `ruler.svg` | `icon-ruler.svg` | Measure |
//! | `link.svg` | `icon-link.svg` | Combine files… |
//! | `scissors.svg` | `icon-scissors.svg` | Split this document… |
//! | `upload.svg` | `icon-upload.svg` | Insert pages from a file… |
//!
//! **Derived from a ScripTree file** (geometry reused, elements added):
//!
//! | Here | Derived from | What changed |
//! |---|---|---|
//! | `zoom-out.svg` | `icon-search.svg` | Magnifier circle and handle reused verbatim; a minus bar added inside |
//! | `zoom-in.svg` | `icon-search.svg` | Same base, plus a cross instead of a bar |
//!
//! ### §4b — Copied from ScripTree with ONE comment added
//!
//! Five more of the operator's own files were brought across on 2026-08-14.
//! Their **geometry is unmodified** — every `path`/`rect`/`circle` element is
//! byte-identical to the ScripTree original, and so is the original's own
//! `<!-- Generic … -->` comment. What was added to each is a **second XML
//! comment naming the ui-spec clause that assigns it**, because §3's style
//! contract as this module now states it wants both: the trademark
//! disclaimer *and* the citation. The §4 files above predate that and keep
//! their single comment; a mass edit to "harmonise" them would break the
//! byte-for-byte claim those rows make, which is the more valuable property.
//!
//! | Here | ScripTree source | Role | Assigned by |
//! |---|---|---|---|
//! | `printer.svg` | `icon-printer.svg` | Print | ui-spec §8.12 |
//! | `settings.svg` | `icon-settings.svg` | Settings | no spec row — see the asset |
//! | `download.svg` | `icon-download.svg` | Export DXF **and** Export form data | ui-spec §3.1 "save", closing paragraph |
//! | `image.svg` | `icon-image.svg` | Insert image | ui-spec §8.5 (reserved for OCR; see the asset for why Insert image is the primary claim) |
//! | `convert.svg` | `icon-convert.svg` | Set scale | ui-spec §8.2 |
//!
//! **Authored new for pdfcer, in the §3 contract** — everything else. Each
//! asset's embedded XML comment carries that glyph's own construction note
//! and, where it has one, the distinction it is drawn to preserve against a
//! neighbouring glyph. Those notes are the reason the files were copied
//! verbatim rather than re-emitted.
//!
//! ## §5 — Deviations from the ui-spec, recorded
//!
//! The standing convention is that the engineer implements a UI spec and
//! deviates only with a recorded reason. Eight:
//!
//! 1. **`edit-objects.svg` is an addition the spec does not cover.** The
//!    ui-spec was written 2026-08-01 and audited the toolbar as it then
//!    stood; the "Obj" vector-edit toggle shipped afterwards. Its metaphor
//!    (a path with draggable nodes) is chosen to collide with nothing: not
//!    `edit.svg`'s pencil (page **text**), not `markup.svg`'s shapes
//!    (annotation **authoring**).
//! 2. **Icon size is 16 pt, not the 18–20 px ui-spec §4.1 suggests.** That
//!    paragraph contradicts itself; the reasoning is repeated in full at
//!    [`super::ICON_PTS`].
//! 3. **The rail's keyboard reorder arrows kept their Unicode glyphs** in
//!    the salvage source, rather than inventing art the spec had not
//!    reviewed. `chevron-up.svg` was later authored anyway, when those
//!    glyphs were VERIFIED tofu in the running build.
//! 4. **`bookmarks.svg`, `layers.svg`, `signatures.svg` are additions the
//!    spec does not cover** — same situation as #1. All three panels shipped
//!    with **no operator-reachable control at all**: a pane subject, a panel
//!    body, and nothing to click. `signatures.svg` carries a constraint
//!    beyond style — it is deliberately **not** a seal, badge, shield or
//!    checkmark, because each of those reads as VALIDATED and pdfcer performs
//!    no cryptographic verification whatsoever. An icon is a claim too.
//! 5. **`fonts.svg` and `show-points.svg` are further additions of the same
//!    kind.** `fonts.svg`'s constraint is the mirror of `signatures.svg`'s:
//!    the Fonts panel is strictly read-only, so borrowing `add-text.svg`'s
//!    I-beam-plus or `edit.svg`'s pencil would have had the glyph promise an
//!    editing capability the panel does not have.
//! 6. **`form-field.svg`, `back.svg`, `close.svg`, `search.svg`,
//!    `chevron-up.svg` and `chevron-down.svg` were authored under the
//!    operator's 2026-08-06 ruling** that a missing glyph is **created** as
//!    part of the work rather than the feature reworded around it. The last
//!    four each replace a text character that was verified to have no face
//!    in the shipped font stack — `←` (U+2190), `✕` (U+2715), `▲` (U+25B2)
//!    and `▾` (U+25BE) — and so rendered as a tofu box on real controls.
//! 7. **Twenty-five glyphs were added on 2026-08-14 for controls the spec
//!    never saw**, and they are one deviation rather than twenty-five
//!    because they all have the same cause: the spec's §0 audited the OLD
//!    shell's toolbar, and this shell's ribbon carries controls that toolbar
//!    did not have. The page-display radio, the rulers/grid/guides row, the
//!    Window group (read mode, full screen, floating panels, reset layout),
//!    the Pages and Forms panel toggles, marquee zoom and zoom-to-selection,
//!    the Hand tool, Delete, Extract, Flatten and the two "manage a list"
//!    dialogs are all in that category. Each asset's own comment names which
//!    of them it is and what it was drawn to stay distinguishable from.
//!
//!    The occasion was the ribbon reading as half-finished: of 88 registered
//!    commands, 47 named an icon and 41 did not, so a band mixed glyphs and
//!    bare words with no rule behind which was which. Thirty of the 41 now
//!    have one; the other eleven are **recorded refusals** — deviation #8.
//!
//!    `list.svg` is the one of the twenty-five that contradicts a spec row
//!    rather than filling a gap: ui-spec §8.2 assigns `icon-ring.svg` to
//!    Manage Dimension Groups, and two concentric circles read as a target
//!    or a radio button at 16 px, not as a list of named things. That row
//!    was written at reservation depth before the Measure surface existed
//!    and offers no reasoning to weigh against; the asset carries the
//!    replacement's.
//! 8. **Eleven commands are deliberately left with no icon**, which is a
//!    deviation from the operator's "icons for all GUI features" instruction
//!    and is therefore recorded rather than assumed. Each is also stated at
//!    its own registration in `crate::shell::commands`:
//!
//!    > ★★★ **THE NUMBER AND THE LIST BELOW ARE HISTORICAL — the live count
//!    > is SEVEN as of 2026-09-05, and the only place that count is true is
//!    > the assertion in `crate::shell::commands`' test module, which fails
//!    > the build when it moves.** This paragraph is a dated block about one
//!    > pass and is kept in its own tense; do not read it as an inventory.
//!    > Six of the eleven have left it — the five **Render** knobs and
//!    > `view.app_initiative` moved off the ribbon into Settings, and
//!    > `file.recent` became an `Item::Custom` with no command behind it —
//!    > while `format.font`, `format.font_size` and `format.font_colour`
//!    > joined it when the Format tab's Font group shipped, and
//!    > `view.panel_close` joined and then left again on 2026-09-05 when it
//!    > took `close`.
//!    >
//!    > ⇒ **A count written into prose in one file cannot be kept true by any
//!    > mechanism in another.** This one drifted through at least four
//!    > separate changes without a single test going red, and what caught it
//!    > was a human reading two files side by side — which is exactly the
//!    > labour an assertion exists to replace. The seven that remain divide
//!    > into six with **no slot to put a glyph in** and one that would be
//!    > wearing the **wrong picture**; neither kind is about the supply of
//!    > art, and a session arriving here to draw something will find nothing
//!    > to draw.
//!
//!    * `view.zoom_actual` — ui-spec §3.2 is an explicit, reasoned
//!      recommendation AGAINST iconifying it ("a numeral read at a glance is
//!      clearer than any glyph substitute could be… both add a decode step a
//!      bare percentage does not need"). Honoured as written.
//!    * The five **Render** knobs (`view.render_strategy`, `render_quality`,
//!      `render_settle`, `render_thin_lines`, `render_antialias`) — their
//!      labels are the parameter names ("Strategy", "Raster scale", "Settle
//!      delay"), there is no industry-conventional glyph for any of them,
//!      and an invented one on a control whose whole content is its value is
//!      decoration. This is §3.2's reasoning applied to a whole group.
//!    * `view.app_initiative` — a three-position policy (Never · Ask ·
//!      Allowed) about whether the application may float a surface over the
//!      page on its own. Any honest drawing of it is a picture of the
//!      floating surface, i.e. of what the default FORBIDS. An icon is a
//!      claim (§5.4), and that one would be the wrong claim.
//!    * `file.recent` — predates this pass, and its reason is unchanged:
//!      reusing `open` would draw two adjacent controls in one band as
//!      though they were one control drawn twice.
//!    * `mode.read`, `mode.review`, `mode.edit` — not ribbon buttons at all.
//!      `egui_shell::ribbon::mode_selector` renders the three as **text
//!      segments** of a segmented control, and that module contains no icon
//!      path whatsoever (verified by reading it: the string `icon` does not
//!      appear in the file). A key on these would be art nothing can draw.
//!
//! ## §5b — One glyph added on 2026-08-14, for the text tool
//!
//! `text-select.svg`, and it is recorded here rather than folded into §5's
//! deviation #7 because that entry is a dated block about one pass and this is a
//! separate occasion. The cause is nonetheless #7's: the spec's §0 audited the
//! **old** shell's toolbar, and that toolbar had no text tool, because this
//! shell had no `CanvasTool::Text` until the same day.
//!
//! It sits beside `add-text.svg` in the I-beam family and the difference between
//! them is the badge: a plus **creates** text, and the bare beam **selects** it.
//! The asset's own comment carries the construction, the two refusals
//! (`fonts.svg`'s A-on-a-baseline, `edit.svg`'s pencil) and the one rejected
//! alternative (Acrobat's arrow-plus-I-beam pair, unreadable at 16 pt and
//! claiming the wrong half of the tool it switches away from).
//!
//! ## §6 — Rendering, and why no new dependency appears here
//!
//! These SVGs are **not** rasterized by any SVG library. [`super::svg`]
//! parses the subset of the path/rect/circle grammar these files use and
//! strokes it with `tiny-skia`, which is already reachable as
//! `pdfcer_render::tiny_skia`. Zero Cargo dependencies were added for this
//! icon set — in particular `resvg`/`usvg` (MPL-2.0) was considered and
//! **rejected by the operator**, and pre-rasterizing to PNG at build time
//! was rejected because it bakes in a resolution.
//!
//! Practical consequence for anyone editing an asset: the parser refuses
//! anything outside its subset rather than guessing, and `super::tests`
//! parses and rasterizes every asset. **A new or edited icon that uses
//! `<g>`, `<defs>`, a `transform`, a gradient, CSS, or an unsupported
//! `stroke-linecap` value will fail `cargo test`, not fail silently at
//! runtime.**

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
///
/// The exact 90-degree sibling of [`FIT_WIDTH`]: the same corner-bracket
/// family, rotated, so the ribbon's three fit glyphs read as one set and none
/// of them is mistaken for another at a glance.
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
///
/// Authored for pdfcer in the header §3 style contract. The filled half of the
/// black-arrow / white-arrow pair; its outline is byte-identical to
/// [`CURSOR_NODE`]'s and that is the whole message.
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
// The 2026-08-21 pass — the selection filter's rows (O17)
// ===========================================================================
//
// Five glyphs authored for `canvas::pick`'s eleven-row popup. The other six
// rows reuse icons the set already had (`text-select`, `image`, `show-points`,
// `markup`, `ruler`, `form-field`), because reuse of an ICON is free while
// reuse of an ASSET is not: `tests::only_the_folder_asset_is_shared` holds
// `folder.svg` as the set's one shared file and every other asset to exactly
// one `Icon`. These five exist because no glyph in the set meant what their
// row means — see each file's embedded comment for which neighbour it had to
// stay distinguishable from, and why.

/// `pick-text.svg` — the art for [`super::Icon::PickText`].
///
/// Authored for pdfcer in the header §3 style contract — three text lines with
/// two square grips on the diagonal. Deliberately frameless, so it cannot be
/// read as [`TEXT_FREETEXT`], and deliberately not an I-beam, so it cannot be
/// read as [`TEXT_SELECT`] one row below it.
pub(super) const PICK_TEXT: &str = include_str!("assets/pick-text.svg");

/// `pick-path.svg` — the art for [`super::Icon::PickPath`].
///
/// Authored for pdfcer in the header §3 style contract — one straight segment
/// crossing one curve, with **no nodes anywhere**, which is the only thing
/// separating it from [`EDIT_OBJECTS`] and [`SHOW_POINTS`].
pub(super) const PICK_PATH: &str = include_str!("assets/pick-path.svg");

/// `pick-part.svg` — the art for [`super::Icon::PickPart`].
///
/// Authored for pdfcer in the header §3 style contract — a three-segment chain
/// with a bracket under the middle segment only. The bracket's span is the
/// message: under the whole chain it would mean the Object rung instead.
pub(super) const PICK_PART: &str = include_str!("assets/pick-part.svg");

/// `pick-form-xobject.svg` — the art for [`super::Icon::PickFormXObject`].
///
/// Authored for pdfcer in the header §3 style contract — a frame holding three
/// unlike marks. The heterogeneous contents are what separate it from
/// [`TEXT_FREETEXT`]'s evenly spaced prose rules.
pub(super) const PICK_FORM_XOBJECT: &str = include_str!("assets/pick-form-xobject.svg");

/// `pick-link.svg` — the art for [`super::Icon::PickLink`].
///
/// Authored for pdfcer in the header §3 style contract — the box-with-escaping
/// -arrow every browser and office suite uses for "goes somewhere else".
/// Explicitly **not** [`LINK`], which is a chain and belongs to Combine.
pub(super) const PICK_LINK: &str = include_str!("assets/pick-link.svg");

// ══════════════════════════════════════════════════════════════════════════
// The 2026-09-04 batch — thirty-six glyphs adopted from the outside review
// ══════════════════════════════════════════════════════════════════════════
//
// Every one of these fills a gap that was already WRITTEN DOWN. Nine close a
// registration that carries a "No icon" refusal in prose — `file.new`,
// `file.ocr`, `markup.finish` and the rest — and the refusals all give the
// same reason: this directory is declared the operator's own art, reusing a
// neighbour's glyph would make two controls say the same thing, and naming a
// key that does not exist draws a slashed placeholder. Each refusal is
// discharged by the art existing, not by an argument.
//
// The rest replace a BORROWED glyph. Four form-field tools shared
// `form-field.svg` and four measure tools shared `measure.svg`: eight
// controls rendering as two pictures, which is the failure the set's
// one-asset-per-role rule exists to prevent.
//
// ★ Provenance: these are drawn from primitives for pdfcer in the same style
// contract as the rest of the directory — 48×48, stroke 2.5, round caps and
// joins, no fill except the redaction family. `assets/PROVENANCE.md` covers
// the whole directory and its terms are unchanged by this batch.

/// `apply-redactions.svg` — the art for [`super::Icon::ApplyRedactions`].
///
/// Edit ▸ Apply redactions (`edit.redact_apply`) — the one irreversible
pub(super) const APPLY_REDACTIONS: &str = include_str!("assets/apply-redactions.svg");

/// `attachment.svg` — the art for [`super::Icon::Attachment`].
///
/// Attachments (`edit.attachments`) — the files this document carries
pub(super) const ATTACHMENT: &str = include_str!("assets/attachment.svg");

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
/// Dimension groups — `measure.manage_groups`, and the caption's dock tab
pub(super) const DIMENSION_GROUPS: &str = include_str!("assets/dimension-groups.svg");

/// `document-next.svg` — the art for [`super::Icon::NextDocument`].
///
/// Switch to the next open document — `view.next_document` (Ctrl+Tab).
pub(super) const DOCUMENT_NEXT: &str = include_str!("assets/document-next.svg");

/// `document-previous.svg` — the art for [`super::Icon::PreviousDocument`].
///
/// Switch to the previous open document — `view.previous_document`
pub(super) const DOCUMENT_PREVIOUS: &str = include_str!("assets/document-previous.svg");

/// `drop-down.svg` — the art for [`super::Icon::DropDown`].
///
/// Place a **drop-down** (the `/Ch` choice field).
pub(super) const DROP_DOWN: &str = include_str!("assets/drop-down.svg");

/// `embed-fonts.svg` — the art for [`super::Icon::EmbedFonts`].
///
/// Embed the font programs a document references but does not carry
pub(super) const EMBED_FONTS: &str = include_str!("assets/embed-fonts.svg");

/// `expand.svg` — the art for [`super::Icon::Expand`].
///
/// A tree row whose children are **hidden** — press to reveal them.
pub(super) const EXPAND: &str = include_str!("assets/expand.svg");

/// `finish-shape.svg` — the art for [`super::Icon::FinishShape`].
///
/// Markup ▸ Finish shape (`markup.finish`) — commit the vertex run a
pub(super) const FINISH_SHAPE: &str = include_str!("assets/finish-shape.svg");

/// `lock.svg` — the art for [`super::Icon::Locked`].
///
/// A row the **document** forbids changing — an optional-content group
pub(super) const LOCK: &str = include_str!("assets/lock.svg");

/// `measure-angle.svg` — the art for [`super::Icon::MeasureAngle`].
///
/// Two-line measurement — `measure.two_line`.
pub(super) const MEASURE_ANGLE: &str = include_str!("assets/measure-angle.svg");

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

/// `redact-selection.svg` — the art for [`super::Icon::RedactSelection`].
///
/// Edit ▸ Redact selection (`edit.redact_selection`) — mark whatever is
pub(super) const REDACT_SELECTION: &str = include_str!("assets/redact-selection.svg");

/// `reflow.svg` — the art for [`super::Icon::Reflow`].
///
/// Reflow paragraph (`edit.reflow_block`) — re-wrap the paragraph the caret
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
/// Save compacted — `file.save_compacted`, the copy with unused objects
pub(super) const SAVE_COMPACT: &str = include_str!("assets/save-compact.svg");

/// `save-copy.svg` — the art for [`super::Icon::SaveCopy`].
///
/// Save a copy — `file.save_copy`.
pub(super) const SAVE_COPY: &str = include_str!("assets/save-copy.svg");

/// `unembed-fonts.svg` — the art for [`super::Icon::UnembedFonts`].
///
/// Remove embedded font programs, leaving the references behind
pub(super) const UNEMBED_FONTS: &str = include_str!("assets/unembed-fonts.svg");

/// `wheel-flip.svg` — the art for [`super::Icon::WheelFlip`].
///
/// The wheel-paging toggle on the status bar — `OPERATOR_REQUESTS.md` O30.
pub(super) const WHEEL_FLIP: &str = include_str!("assets/wheel-flip.svg");

// ── the last three aliases, broken 2026-09-04 ─────────────────────────────
//
// `properties`, `insert-pages` and `set-scale` were live icon KEYS that
// resolved to another role's asset: `document.svg`, `upload.svg` and
// `convert.svg`. That is the same defect as the four form tools sharing one
// glyph — a control wearing a picture drawn for something else — one level
// down, and it survived the 2026-09-04 batch because a mechanical pre-filter
// removed every proposed name that collided with an existing key, on the
// reasoning that a collision means a restyle.
//
// ★ The reasoning was right and the conclusion was wrong: there was no
// purpose-drawn art to restyle. Each of the three is the FIRST art drawn for
// its role. Found because the layout mockup draws shipped art beside proposed
// art and the adoption count did not add up — 36 + 26 = 62 of 65.

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
// Five glyphs drawn 2026-09-04 — ★ ALL FIVE NOW HAVE BUTTONS (2026-09-05)
// ══════════════════════════════════════════════════════════════════════════
//
// ★★★ THIS HEADING USED TO READ *"the commands the ribbon does not reach
// yet"*, AND IT WAS FALSE WITHIN THE DAY. Every one of the five is now named
// by a registered command:
//
// | glyph | command | where it is drawn |
// |---|---|---|
// | `export-image` | `file.export_image` | File ▸ Export |
// | `copy-as-vector` | `edit.copy_as_vector` (token 408) | Edit ▸ Clipboard, icon-only |
// | `encrypt` | `file.encrypt` (126) | File ▸ Security, large |
// | `permissions` | `file.permissions` (127) | File ▸ Security, large |
// | `open-in-acrobat` | `file.open_in_acrobat` | the ribbon's trailing item |
//
// The art was drawn in the morning of 2026-09-04 and four of the five buttons
// were wired the same afternoon, by the tracks this block names as future
// work. The block is corrected rather than deleted because **the shape of the
// mistake is the transferable part**: a sentence that says a thing is not
// built is a dated citation whose shelf life on this project has repeatedly
// been measured in hours, and this one was overtaken by the very tracks it
// predicted. ⇒ Write such a claim so it can be an assertion; where it cannot,
// date it and expect to return.
//
// ★ What did NOT need correcting is the paragraph at the foot of this block —
// *"a variant with no command is the SUPPORTED state"* — and the reason is
// instructive. That claim is about `Icon::ALL` membership and the tests that
// walk it, so it stayed true through the whole transition in both directions.
// A claim anchored to a mechanism does not rot; a claim anchored to a schedule
// does.
//
// The original argument for each glyph is kept below, in the tense it was
// written in, because each carries the reasoning for the picture and that
// reasoning is unaffected by whether a button exists.
//
// ★ `export-image` is the live one. `file.export_image` shipped earlier the
// same day wearing `export` (`download.svg`) with a paragraph in
// `shell::commands::catalog::file` defending the share — three export verbs,
// one act, and the FORMAT is "a word only a label can say". That argument is
// right about DXF and form data and wrong about a picture, because this set
// already draws a picture as a subject: `image.svg` exists and `insert-image`
// wears it, so the operator has already learned what a framed tile with a
// horizon means here. The registration is repointed and its comment records
// the reversal rather than being quietly rewritten.
//
// ★★ The other four were ART BEFORE BUTTON when drawn — each is annotated
// with what happened to it, so the prediction and the outcome stay side by
// side rather than the prediction being quietly deleted:
//
// * `open-in-acrobat` — a command being built on another track as this lands.
//   That track was told not to add icons; this is the icon it will name.
//   ✅ **It named it.** `file.open_in_acrobat` ships as the ribbon's trailing
//   item, gated on `acrobat.available`. The prediction was exactly right.
// * `copy-as-vector` — the clipboard's missing copy-out. Drawn so the layout
//   mockup could put the proposal in front of the operator as a picture rather
//   than as a sentence.
//   ✅ **BUILT 2026-09-04** as `edit.copy_as_vector`, token 408, drawn
//   icon-only beside Cut / Copy / Paste on Edit ▸ Clipboard. This entry said
//   *"not built, not scheduled"* and was overtaken the same day — the mockup
//   asking the question is what got it built, which is the mechanism working,
//   not a mis-prediction. What was wrong was writing "not scheduled" as a fact
//   about the future rather than as a reading of that morning.
// * `encrypt` and `permissions` — the engine grew both on 2026-09-04 and
//   nothing in this GUI reached either. `OPERATOR_REQUESTS.md` O119 is the
//   question to him, and it is a question about a SURFACE (a password box, a
//   permission list, a save that rewrites the file), not about a button. The
//   art did not pre-empt his answer; it let the mockup ask.
//   ✅ **HE ANSWERED — *"yes add encryption and permissions"*** — and both
//   ship as `file.encrypt` / `file.permissions` on a new File ▸ Security
//   group, both large, exactly where the mockup drew them. This is the one
//   entry whose reasoning was vindicated rather than merely overtaken: the art
//   existed so a question could be asked as a picture, the question was asked,
//   and the answer arrived within the day.
//
// ⇒ A variant with no command is the SUPPORTED state, not a loose end.
// [`super::Icon::EditObjects`] is the standing precedent — its command was
// deleted on 2026-08-31 and its variant remains — and the reason is
// mechanical rather than sentimental: `every_icon_parses`,
// `every_icon_rasterizes_to_visible_pixels`,
// `fill_is_semantic_and_the_set_that_uses_it_is_closed`,
// `crlf_line_endings_parse_identically` and
// `no_two_icons_render_as_the_same_picture` all iterate
// [`super::Icon::ALL`]. Art that is kept outside that list is art no test
// walks, and untested art rots quietly until somebody wires it up months
// later and finds it blank.
//
// ★ Provenance unchanged: drawn from primitives for pdfcer in the same style
// contract as the rest of the directory — 48×48, stroke 2.5, round caps and
// joins, no fill except the redaction family. `assets/PROVENANCE.md` covers
// the whole directory. ⚠ `open-in-acrobat` names a vendor in its LABEL and
// carries nothing of that vendor's mark in its ART; see the asset's own
// comment, which states the constraint first because the label is what
// invites the mistake.

/// `copy-as-vector.svg` — the art for [`super::Icon::CopyAsVector`].
///
/// Copy the selection to the clipboard as vector geometry rather than as a
/// picture of it — `edit.copy_as_vector`, token 408, drawn icon-only beside
/// Cut / Copy / Paste on Edit ▸ Clipboard since 2026-09-04. See the asset for
/// which glyph it must stay distinguishable from and by what cue.
pub(super) const COPY_AS_VECTOR: &str = include_str!("assets/copy-as-vector.svg");

/// `encrypt.svg` — the art for [`super::Icon::Encrypt`].
///
/// Put a password on this document — the engine's `set_encryption`. ★ Worn by
/// `file.encrypt` (token 126) on File ▸ Security since 2026-09-04; this line
/// said *"awaiting the operator's ruling as O119"* until 2026-09-05, and he
/// had answered *"yes add encryption and permissions"* the day before.
pub(super) const ENCRYPT: &str = include_str!("assets/encrypt.svg");

/// `export-image.svg` — the art for [`super::Icon::ExportImage`].
///
/// Export the page as a raster image — `file.export_image`, which wore
/// [`DOWNLOAD`] for one day. See the asset for the reversal and its reason.
pub(super) const EXPORT_IMAGE: &str = include_str!("assets/export-image.svg");

/// `open-in-acrobat.svg` — the art for [`super::Icon::OpenInAcrobat`].
///
/// Hand this file to the system's PDF viewer. ⚠ The label names a vendor; the
/// art carries nothing of that vendor's mark, and the asset's comment states
/// that constraint before it states anything else.
pub(super) const OPEN_IN_ACROBAT: &str = include_str!("assets/open-in-acrobat.svg");

/// `permissions.svg` — the art for [`super::Icon::Permissions`].
///
/// What the document permits — the engine's `set_permissions`. ★ Worn by
/// `file.permissions` (token 127) on File ▸ Security since 2026-09-04; O119 is
/// answered and closed, and this line described it as open until 2026-09-05.
pub(super) const PERMISSIONS: &str = include_str!("assets/permissions.svg");

/// `select-all.svg` — the art for [`super::Icon::SelectAll`].
///
/// ★ Added 2026-09-04 to correct a record, not to fill a gap. Its absence had
/// been argued as a refusal by a build session and then quoted until it read as
/// the operator's own ruling; it was not, and he said so. The asset's own
/// comment carries the whole account, including which half of the old argument
/// is drawn into the glyph rather than discarded.
pub(super) const SELECT_ALL: &str = include_str!("assets/select-all.svg");

/// `bold.svg` — the art for [`super::Icon::Bold`].
///
/// ★ Added 2026-09-04 to correct a refusal, not to fill a gap. `format.bold`
/// was registered with no icon because *"this build has no such art"* — a
/// statement about supply, and the operator's standing ruling is that a missing
/// glyph is **authored**. The asset's own comment carries the account, and the
/// reason its stroke is 4 rather than the set's 2.5.
pub(super) const BOLD: &str = include_str!("assets/bold.svg");

/// `italic.svg` — the art for [`super::Icon::Italic`].
///
/// ★ Added 2026-09-04 alongside [`BOLD`], on the same correction and for the
/// same reason. See the asset for why the slant is exaggerated and why the
/// serifs are offset rather than centred.
pub(super) const ITALIC: &str = include_str!("assets/italic.svg");

/// `line-weights.svg` — the art for [`super::Icon::LineWeights`].
///
/// ★★★ Authored 2026-09-05 for `view.line_weights` (O137), a control that had
/// been **deleted** on 2026-08-17 for want of an engine field. The field
/// arrived (`RenderOptions::stroke_display`, `Pass 254.0`), so the button came
/// back and the glyph was drawn for it — the operator's standing ruling is that
/// a missing glyph is authored, not worked around.
///
/// ★★ The **only** asset in this directory that does not stroke at a uniform
/// 2.5, and the asset's own comment carries why: the varying weight IS the
/// subject, so a glyph drawn at one width would be a picture of the feature
/// switched off. It also carries the 16 px measurement behind the thinnest
/// bar's 1.6, and the two axes that keep it clear of `list.svg`.
pub(super) const LINE_WEIGHTS: &str = include_str!("assets/line-weights.svg");
