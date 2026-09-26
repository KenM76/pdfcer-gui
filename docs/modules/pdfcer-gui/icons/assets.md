# icons::assets — the icon set itself, and its provenance record

**Generated content.** Every constant below embeds one file of
`src/icons/assets/*.svg`, which are byte-for-byte copies of the salvage
source's `D:\Dev\pdfce\crates\pdfce-gui\assets\icons\*.svg` — XML
rationale comments included. Nothing was retyped, reformatted or
"tidied": each asset's own comment is the primary record of what the
glyph depicts and which neighbouring glyph it was drawn to stay
distinguishable from, and a paraphrase would lose exactly the part that
is expensive to re-derive.

## Why the art lives in `src/icons/assets/` rather than a top-level
## `crates/pdfcer-gui/assets/`

`include_str!` reaches a directory *inside* the icon module rather than a
sibling of `src/`. Two reasons:

1. **This module's write territory is `src/icons/`.** The rebuild runs
   several agents in parallel over one tree, and the boundaries between
   them are directories. Creating `crates/pdfcer-gui/assets/` would put
   art outside the territory the icon work owns.
2. **Co-location.** The art, the catalogue that names it, the parser that
   reads it and the painter that draws it are one subject. A reader who
   opens `src/icons/` sees all of it.

`include_str!` rather than a runtime file read, because pdfcer ships
single-folder portable: the executable must not depend on an `assets/`
directory travelling beside it, and an icon that fails to load at startup
is not a failure mode worth having. The whole set measured **79 files,
82,336 bytes** of text, roughly half of which is the embedded rationale
comments — the point of copying assets verbatim rather than re-emitting
them, see §4.

⚠ **Nothing tests that figure.** `Icon::ALL`'s size is pinned by an
assertion in `catalog.rs`; this one counts files on disk and no test walks
the directory, so it is a measurement of one moment and drifts silently
from what is there. Re-count before relying on it.

## Licensing

`assets/PROVENANCE.md` is the licensing record for this directory, and
`tools/gates/check-shipped-assets.py` requires it to exist and to name
terms. In one line: the art is the operator's own, under the project's own
MIT licence, which is why it needs **no** entry in `about.hbs` — the
shipped `LICENSE` already covers it, and there is no third-party grant to
reproduce. §1 below is the primary record of how that was established.

## Why the SVG text is NOT inlined into Rust source

The obvious alternative — a `const FOLDER: &str = r##"<svg …>"##;` with
the markup inline — was implemented first and then **withdrawn**,
because it fails `tools/gates/check-ui-strings.sh` on 138 lines and
cannot be exempted.

That gate is a line-oriented scanner over `.rs` files looking for string
literals containing whitespace, which is its proxy for "prose that
belongs in the ui-text catalog". Almost every line of an SVG asset trips
it, because SVG *attribute values* are quoted strings full of spaces:
`viewBox="0 0 48 48"`, `d="M14 10h10M19 10v28"`. Its escape hatch is a
`// ui-text-exempt:` marker on the offending line or in the comment block
immediately above it — and neither can reach these lines, because they
are **inside a raw string**: a marker written there would become part of
the asset and be handed to the SVG parser.

Keeping the art in `.svg` files is not a workaround for that gate; it is
the arrangement that makes the gate's question meaningful. XML markup is
not Rust source and should not be scanned as though it were. The finding
is recorded here because the inline form *looks* simpler and someone will
propose it again.

## Regenerating

The `.svg` files are copied from the salvage source; the constants below
are produced mechanically from the directory listing. If the set gains an
asset, drop the `.svg` in and add a constant here — **and add the
matching [`super::Icon`] variant to [`super::Icon::ALL`]**, or it ships
unverified (see `super::tests::every_icon_parses`).

---

# Provenance

Carried across from the salvage source's `assets/icons/PROVENANCE.md`,
which `docs/ui_specs/icon-set-and-toolbar.md` §7.2 required before any
art was bundled: the provenance of this set had to be **confirmed, not
assumed**. That record is a licensing artefact, so it travels with the
art it describes.

## §1 — Operator confirmation (the licensing question, closed)

The ui-spec flagged an open question: were `D:\Dev\ScripTree\icons\*.svg`
drawn from scratch for ScripTree, or adapted from a third-party icon pack
(Feather, Lucide, Font Awesome, …) whose own licence would then travel
with them into pdfcer's asset tree?


> "Scriptree icons are mine, use from it what makes sense and create new
> ones in its style when necessary, try to make them close to what
> inkscape and Adobe use for similar commands without running into
> copyright issues."

Consequences, all of them binding on this module:

* The ScripTree source art is the operator's own work. He owns both
  projects, so no third-party licence travels with the copied files and
  nothing here needs an upstream attribution entry.
* pdfcer's own licence is MIT; these assets ship under it like the rest of
  the tree.
* `THIRD_PARTY_LICENSES.md` is unaffected — it is generated by
  `cargo-about` from the **dependency** set, and this set adds zero
  dependencies (see §6).

## §2 — The metaphor-not-artwork rule (binding on every future icon)

The operator's instruction — *"try to make them close to what inkscape
and Adobe use for similar commands without running into copyright
issues"* — draws the line this module is held to:

* **Allowed: metaphor-level resemblance.** A magnifier means zoom. A
  curved arrow means undo. Scissors mean split. Corner brackets mean fit
  to frame. These are industry conventions with no single author, and
  matching them is what makes a toolbar legible to someone arriving from
  Acrobat or Inkscape.
* **Forbidden: asset-level copying.** No tracing, no importing, no
  "adapting" of any Adobe or Inkscape SVG, icon font, or screenshot.
  Every glyph here was constructed from primitives (rectangles, circles,
  line segments, arcs), and every asset's embedded comment says which
  concept it depicts.
* This mirrors the standing rule that Acrobat and Inkscape are
  **behavioural** references only, never sources of GUI structure or art.

This rule is *stricter* than copyright law strictly requires for simple
geometric glyphs. That is deliberate: it removes the question entirely
rather than leaving a judgement call in a file nobody will re-examine.

## §3 — Style contract (every asset here obeys it)

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48" fill="none" aria-hidden="true">
  <!-- Generic <concept> shape — placeholder, not a vendor trademark logo -->
  <path|rect|circle … stroke="currentColor" stroke-width="2.5"
        stroke-linecap="round" stroke-linejoin="round"/>
</svg>
```

* **48×48 viewBox**, content inset ~6–8 units from every edge. That edge
  length is [`super::svg::VIEWBOX`], and every coordinate is in it.
* **`fill="none"`, `stroke="currentColor"`** — pure outline, no baked
  colour. This is the property the whole pipeline rests on: one raster per
  icon, tinted per theme and per widget state at draw time. An asset with
  a hardcoded colour would break light/dark theming *structurally*, not
  just cosmetically.
* **`stroke-width="2.5"`**, round caps and joins throughout.
* A comment naming the concept and disclaiming trademark risk.

### The two deliberate exceptions

`redact.svg` is the only asset with a **filled** shape
(`fill="currentColor"` on the redaction bar). ui-spec §8.1 makes this an
explicit, rule-based exception, not style drift: a solid bar is what
redaction actually leaves behind, and an outline-only glyph would
visually understate a feature that irreversibly removes content. **A
future icon audit must not "fix" it back to an outline.**
`super::tests::fill_is_semantic_and_the_set_that_uses_it_is_closed`
asserts both halves — that redaction is filled and that nothing outside a
named set is.

⇒ **A rename carries its citations with it.** A gate cited under a name it
no longer has reads, to anyone who greps for it, as a gate that does not
exist — which blinds a reader exactly as thoroughly as deleting it, and
costs the same to fix.

`shape-highlight.svg` uses `stroke-width="1"` for its 45° hatch. Also
deliberate (ui-spec §3.3): the hatch is a *texture* standing in for
translucent highlighter colour, not a contour, and at 2.5 it would fill
the band solid and read as redaction.

## §4 — File-by-file origin

**Copied verbatim from `D:\Dev\ScripTree\icons\`** (the operator's own
art), byte for byte apart from the filename. Renamed to pdfcer's **role**
rather than ScripTree's shape name so a future re-draw changes one asset
and no call site:

| Here | ScripTree source | Role |
|---|---|---|
| `folder.svg` | `icon-folder.svg` | Open **and** Font folders — one asset, two roles |
| `document.svg` | `icon-document.svg` | Properties |
| `edit.svg` | `icon-edit.svg` | Edit Text |
| `tool.svg` | `icon-tool.svg` | Tools |
| `ruler.svg` | `icon-ruler.svg` | Measure |
| `link.svg` | `icon-link.svg` | Combine files… |
| `scissors.svg` | `icon-scissors.svg` | Split this document… |
| `upload.svg` | `icon-upload.svg` | Insert pages from a file… |

**Derived from a ScripTree file** (geometry reused, elements added):

| Here | Derived from | What changed |
|---|---|---|
| `zoom-out.svg` | `icon-search.svg` | Magnifier circle and handle reused verbatim; a minus bar added inside |
| `zoom-in.svg` | `icon-search.svg` | Same base, plus a cross instead of a bar |

### §4b — Copied from ScripTree with ONE comment added

Five more of the operator's own files, copied on the same terms. Their
**geometry is unmodified** — every `path`/`rect`/`circle` element is
byte-identical to the ScripTree original, and so is the original's own
`<!-- Generic … -->` comment. Each carries one addition: a **second XML
comment naming the ui-spec clause that assigns it**, because §3's style
contract wants both the trademark disclaimer *and* the citation. The §4
files above carry only their single original comment, and a mass edit to
"harmonise" them would break the byte-for-byte claim those rows make,
which is the more valuable property.

| Here | ScripTree source | Role | Assigned by |
|---|---|---|---|
| `printer.svg` | `icon-printer.svg` | Print | ui-spec §8.12 |
| `settings.svg` | `icon-settings.svg` | Settings | no spec row — see the asset |
| `download.svg` | `icon-download.svg` | Export DXF **and** Export form data | ui-spec §3.1 "save", closing paragraph |
| `image.svg` | `icon-image.svg` | Insert image | ui-spec §8.5 (reserved for OCR; see the asset for why Insert image is the primary claim) |
| `convert.svg` | `icon-convert.svg` | Set scale | ui-spec §8.2 |

**Authored new for pdfcer, in the §3 contract** — everything else. Each
asset's embedded XML comment carries that glyph's own construction note
and, where it has one, the distinction it is drawn to preserve against a
neighbouring glyph. Those notes are the reason the files were copied
verbatim rather than re-emitted.

## §5 — Deviations from the ui-spec, recorded

The standing convention is that the engineer implements a UI spec and
deviates only with a recorded reason. Eight:

1. **`edit-objects.svg` is an addition the spec does not cover.** The
   ui-spec audits the toolbar as it stood when the spec was written, and
   the "Obj" vector-edit toggle shipped afterwards. Its metaphor
   (a path with draggable nodes) is chosen to collide with nothing: not
   `edit.svg`'s pencil (page **text**), not `markup.svg`'s shapes
   (annotation **authoring**).
2. **Icon size is 16 pt, not the 18–20 px ui-spec §4.1 suggests.** That
   paragraph contradicts itself; the reasoning is repeated in full at
   [`super::ICON_PTS`].
3. **The rail's keyboard reorder arrows kept their Unicode glyphs** in
   the salvage source, rather than inventing art the spec had not
   reviewed. `chevron-up.svg` was later authored anyway, when those
   glyphs were VERIFIED tofu in the running build.
4. **`bookmarks.svg`, `layers.svg`, `signatures.svg` are additions the
   spec does not cover** — same situation as #1. All three panels shipped
   with **no operator-reachable control at all**: a pane subject, a panel
   body, and nothing to click. `signatures.svg` carries a constraint
   beyond style — it is deliberately **not** a seal, badge, shield or
   checkmark, because each of those reads as VALIDATED and pdfcer performs
   no cryptographic verification whatsoever. An icon is a claim too.
5. **`fonts.svg` and `show-points.svg` are further additions of the same
   kind.** `fonts.svg`'s constraint is the mirror of `signatures.svg`'s:
   the Fonts panel is strictly read-only, so borrowing `add-text.svg`'s
   I-beam-plus or `edit.svg`'s pencil would have had the glyph promise an
   editing capability the panel does not have.
6. **`form-field.svg`, `back.svg`, `close.svg`, `search.svg`,
   `chevron-up.svg` and `chevron-down.svg` were authored under the
   operator's standing ruling** that a missing glyph is **created** as
   part of the work rather than the feature reworded around it. The last
   four each replace a text character that was verified to have no face
   in the shipped font stack — `←` (U+2190), `✕` (U+2715), `▲` (U+25B2)
   and `▾` (U+25BE) — and so rendered as a tofu box on real controls.
7. **Twenty-five glyphs cover controls the spec never saw**, and they are
   one deviation rather than twenty-five because they all have the same
   cause: the spec's §0 audited the OLD
   shell's toolbar, and this shell's ribbon carries controls that toolbar
   did not have. The page-display radio, the rulers/grid/guides row, the
   Window group (read mode, full screen, floating panels, reset layout),
   the Pages and Forms panel toggles, marquee zoom and zoom-to-selection,
   the Hand tool, Delete, Extract, Flatten and the two "manage a list"
   dialogs are all in that category. Each asset's own comment names which
   of them it is and what it was drawn to stay distinguishable from.

   The occasion was the ribbon reading as half-finished: a band that mixes
   glyphs and bare words with no rule behind which is which reads as
   unfinished work. Every command that can carry a glyph has one; every
   command that cannot is a **recorded refusal** — deviation #8, which
   is also why no count of either appears here.

   `list.svg` is the one of the twenty-five that contradicts a spec row
   rather than filling a gap: ui-spec §8.2 assigns `icon-ring.svg` to
   Manage Dimension Groups, and two concentric circles read as a target
   or a radio button at 16 px, not as a list of named things. That row
   was written at reservation depth before the Measure surface existed
   and offers no reasoning to weigh against; the asset carries the
   replacement's.
8. **Some commands are deliberately left with no icon**, which is a
   deviation from the operator's "icons for all GUI features" instruction
   and is therefore recorded rather than assumed. Each is argued at its own
   registration in `crate::shell::commands`, and the only place the count
   of them is true is that module's own assertion — **not here**.


   ⚠ **Two kinds of refusal, and only one of them is about the supply of
   art.** A command with **no slot to put a glyph in** — a text segment, a
   combo box, a control whose whole content is its own value — gains
   nothing from new art; nor does one that would be wearing the **wrong
   picture**. A session arriving here to draw something for a refusal of
   either kind will find nothing to draw.

   * `view.zoom_actual` — ui-spec §3.2 is an explicit, reasoned
     recommendation AGAINST iconifying it ("a numeral read at a glance is
     clearer than any glyph substitute could be… both add a decode step a
     bare percentage does not need"). Honoured as written.
   * `mode.read`, `mode.review`, `mode.edit` — not ribbon buttons at all.
     `egui_shell::ribbon::mode_selector` renders the three as **text
     segments** of a segmented control, and that module contains no icon
     path whatsoever (verified by reading it: the string `icon` does not
     appear in the file). A key on these would be art nothing can draw.

## §5b — The text tool's glyph

`text-select.svg`, recorded apart from §5's deviation #7 because it is a
separate occasion, though the cause is #7's: the spec's §0 audited the
**old** shell's toolbar, and that toolbar had no text tool.

It sits beside `add-text.svg` in the I-beam family and the difference between
them is the badge: a plus **creates** text, and the bare beam **selects** it.
The asset's own comment carries the construction, the two refusals
(`fonts.svg`'s A-on-a-baseline, `edit.svg`'s pencil) and the one rejected
alternative (Acrobat's arrow-plus-I-beam pair, unreadable at 16 pt and
claiming the wrong half of the tool it switches away from).

## §6 — Rendering, and why no new dependency appears here

These SVGs are **not** rasterized by any SVG library. [`super::svg`]
parses the subset of the path/rect/circle grammar these files use and
strokes it with `tiny-skia`, which is already reachable as
`pdfcer_render::tiny_skia`. Zero Cargo dependencies were added for this
icon set — in particular `resvg`/`usvg` (MPL-2.0) was considered and
**rejected by the operator**, and pre-rasterizing to PNG at build time
was rejected because it bakes in a resolution.

Practical consequence for anyone editing an asset: the parser refuses
anything outside its subset rather than guessing, and `super::tests`
parses and rasterizes every asset. **A new or edited icon that uses
`<g>`, `<defs>`, a `transform`, a gradient, CSS, or an unsupported
`stroke-linecap` value will fail `cargo test`, not fail silently at
runtime.**
