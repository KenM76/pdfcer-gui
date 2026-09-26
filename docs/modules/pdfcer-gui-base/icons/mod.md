# icons — the SVG-path → tiny-skia → egui-texture icon pipeline

Turns a set of hand-authored outline SVGs into tinted, DPI-correct egui
images for the ribbon, the menus and any hand-drawn control. Nothing in
this module knows what any icon *means* — the meaning lives in
[`Icon`]'s variant names and their doc comments; this module knows how to
turn a path `d` attribute into pixels, how not to do it twice, and how to
report the one thing it cannot draw.

It is split along the seams of that pipeline:

| module | what it owns |
|---|---|
| [`assets`] | the art itself, verbatim, and its **provenance record** |
| [`catalog`] | [`Icon`] — which glyphs exist, what each one means, and the key vocabulary |
| [`svg`] | the SVG element scanner and the tiny-skia rasterizer (the path `d` grammar is `svg::path`) |
| [`cache`] | one raster per (icon, physical size, weight), memoized |
| [`paint`] | the `egui-shell` painter seam, and the missing-icon mark |

## Why a hand-rolled SVG subset parser instead of a crate

egui renders no vector art natively, so an SVG has to become a raster
somewhere. The three candidate pipelines, and why this one won:

1. **A runtime SVG crate (`resvg`/`usvg`).** Correct and general, but a
   NEW Cargo dependency — and `resvg` is MPL-2.0 (weak copyleft). The
   standing rule is that an agent does not add a dependency solo,
   copyleft or not, and this was an explicit operator go/no-go.
   **Rejected by the operator.**
2. **Pre-rasterize to PNG at build time.** Zero dependencies, but the
   resolution is baked: a raster sized for a 16 pt slot at 100% display
   scale is visibly soft at 150%/200% Windows scaling, and would be wrong
   again for any future "larger toolbar icons" accessibility option. It
   was also *not executable on the operator's machine* — no SVG
   rasterizer is installed (no Inkscape, no ImageMagick, and
   `cairosvg`'s libcairo fails to load), so the conversion step had no
   tool to run.
3. **Parse the path data ourselves and stroke it with `tiny-skia`** —
   what this module does. `tiny-skia` is ALREADY reachable as
   `pdfcer_render::tiny_skia`, so this adds **zero** new crates. It
   rasterizes at whatever physical pixel size the current display scale
   implies, so icons are crisp at any DPI — strictly better than (2) and
   free of (1)'s licensing question.

**This is the load-bearing decision of the whole module.** Everything else
here — the cache key
including the physical size, the mask-plus-tint theming, the refusal to
guess at malformed path data — follows from choosing (3), and any future
"simplification" that pre-bakes rasters gives up crispness at every
display scale the operator's machine is not currently set to.

The cost of (3) is that a subset SVG parser now exists in this repo, and
a parser that silently mis-reads its input is worse than no parser at
all. That risk is contained two ways: the parser **refuses** rather than
guesses (see [`svg`]), and [`tests::every_icon_parses`] parses every
shipped asset, so a malformed or out-of-subset icon fails the test gate
instead of shipping as a wrong glyph.

## Theming: one raster per icon, tinted at draw time

Every asset is `stroke="currentColor"` — a single-colour outline with no
palette — so each icon is rasterized ONCE as a white-on-transparent
coverage mask and takes its colour at draw time from the **theme's
foreground**, never from a baked constant. In the ribbon that colour
arrives as `IconRequest::tint`, which `egui-shell` reads from
`ui.style().interact(&response).fg_stroke.color`; in a hand-drawn control
it is [`image`]'s `ui.visuals().text_color()`.

Consequences, all of them deliberate:

* Light theme, dark theme, hovered, active and disabled all share ONE
  raster. There are no light/dark asset pairs to keep in sync, and
  structurally no way for an icon to end up hardcoded-black on a dark
  background — the failure `tools/gates/check-theme-colors.sh` exists to
  catch is removed here by construction rather than by policing.
* The tint is therefore **not** part of the cache key. Re-tinting is
  free; re-rastering is not.
* Disabled controls need no fade logic at all. egui's own
  `Ui::disable()` multiplies the painter's opacity, which applies to
  textured meshes exactly as it applies to text — so an icon fades
  precisely the way the text beside it does.

## Weight, and why selected state is not colour alone

[`IconWeight::Bold`] rasterizes the same art with the stroke width
multiplied. It exists because of the standing rule that **selected state
is never colour alone**. Text toggles satisfy that rule by going bold; an
icon has no text to embolden, so the *glyph* goes bold instead.

Note the seam limitation recorded in [`paint`]: `IconRequest` does not
carry the control's selected state, so the ribbon path cannot apply the
weight cue today. It is fully implemented and reachable through
[`toggle_image`] for anything the application draws itself.

## DPI

Icons are laid out in **logical points** ([`ICON_PTS`], or whatever
square the ribbon reserved) but rasterized at
`points * pixels_per_point()` **physical pixels**, then drawn back at the
logical size. Rasterizing at the logical size instead would make every
icon visibly soft on any HiDPI display. Because the physical size is part
of the cache key, a display-scale change re-rasterizes automatically
rather than reusing a stale, wrongly-sized texture — asking for the right
size *is* the cache invalidation.

## Wiring it into the ribbon

```ignore
let mut icons = pdfcer_gui::icons::paint_ribbon_icon;
let report = Ribbon::new(&registry, &conditions, &manifest)
    .with_icon_painter(&mut icons)
    .render(ui, &mut ribbon_state);
```

Until that painter is supplied, `egui-shell` draws text labels
everywhere: `ribbon::qat::shows_label` refuses to go icon-only unless the
application can actually paint. See [`paint`]'s header — supplying a
painter is what turns the ribbon from a row of text buttons into a
ribbon.

## Item notes

### `fn every_icon_parses`

This is the gate that makes the hand-rolled parser safe to rely on: a
malformed or out-of-subset icon fails `cargo test` rather than
shipping as a blank button. It walks [`Icon::ALL`], which is why
`catalog::tests::all_is_exhaustive_and_free_of_duplicates` exists —
a variant missing from `ALL` is not merely untested, it is *silently*
untested.

### `fn fill_is_semantic_and_the_set_that_uses_it_is_closed`

A future "style cleanup" must not quietly turn redaction's honest
solid bar into an outline, and no other icon may drift into being
filled. The fill is semantic, not decorative: every other tool in
this application draws or measures, and this one obliterates.

It is also the pipeline's only coverage of the fill path, so an
"audit" that outlined it would silently delete a test as well as a
meaning.

**The assertion is membership of a NAMED SET with a reason per
member**, not `icon == Icon::Redact`. The rule it enforces is *fill is
semantic, never decorative*, and the black-arrow / white-arrow pair is
the purest available instance of it: `cursor` and `cursor-node` have
**byte-identical outlines** and differ only in fill, and that difference
has meant "the whole object" versus "the points inside it" in every
vector editor since Illustrator 88. An audit that outlines any member of
the set deletes a meaning as well as a test.

### `const FILLED`

- [`Icon::Redact`] — every other tool draws or measures; this one
  obliterates.
- [`Icon::Cursor`] — the filled half of the arrow pair. Its hollow
  twin is the ONLY thing distinguishing the two tools.
- [`Icon::CursorNode`] — the hollow arrow, plus one filled anchor
  square among three outlined ones, which says *this is the point
  you picked* in the same language `canvas::overlay` draws on the
  page itself.
- [`Icon::RedactSelection`] and [`Icon::ApplyRedactions`] —
  **they inherit the reason rather than extending it.** Both are
  members of the redaction family, both draw the
  same solid bar [`Icon::Redact`] draws, and both act on the same
  irreversible thing. An outline-only redaction glyph understates a
  feature that removes content permanently, and that argument does
  not weaken because the command is scoped to a selection or is the
  apply step. Adding them was a decision, not a formality: the
  honest alternative was to outline these two and leave the fill to
  the parent tool, and it was rejected because it would make the
  family's most destructive member — Apply, the one that cannot be
  undone — the palest picture of the three.

### `const DELIBERATELY_ALIKE`

- The **magnifier family**. `zoom-in`, `zoom-out` and `zoom-region`
  share a lens because they are three aims of one act, and every
  application that has ever had a zoom control draws them that way.
  Their whole distinction is the mark inside the lens, which is a
  few pixels at 16 px by construction. ⚠ `zoom-out ~ zoom-in`
  measures **0.103**, which is genuinely tight — it is recorded here
  rather than smoothed away, and if the operator ever reports that
  the two zoom buttons are hard to tell apart, this line is the
  evidence that it was known and where to look.
- The **arrow-leaves-container family**. `upload` (worn by
  `insert-pages` and `import-form-data`) and `export` both show
  content crossing a boundary, because both commands move data
  across the document's edge. They differ in direction, which is the
  distinction that matters and is the one an operator reads.

### `fn crlf_line_endings_parse_identically`

The assets are ordinary text files, so a repository with
`* text=auto` converts them to CRLF on checkout under
`core.autocrlf=true` — which means the `&'static str` every
`include_str!` in [`assets`] produces gains a `\r` before every `\n`
on a fresh clone, on a machine that has never built this tree before.

SVG is text and CRLF is harmless *provided* the scanner treats `\r`
as a separator everywhere `\n` is one. This pins that: the same asset
with every line ending doubled must produce identical geometry AND an
identical raster, so a fresh clone on a machine with autocrlf on
cannot ship blank icons.

### `fn every_command_icon_key_exists_in_the_catalogue`

This is the mismatch that puts blank boxes back in the ribbon, and it
is invisible to the compiler: `Command::with_icon` takes a `String`,
so a key with no glyph is a perfectly well-typed program that draws a
slashed box where an operator expects a control.

The check is done against the real command registry rather than
against a hand-maintained list, because a hand-maintained list is one
more thing to forget to update — and forgetting it would make this
test pass while the ribbon was wrong, which is worse than not having
it.

### `mod glyphs`

Test-only, and the sibling of [`paint_missing_mark`] on the text side —
same question ("what happens when a mark cannot be drawn?"), different
pipeline. It carries the widened glyph gate over `crate::text`, and the
finding that `egui`'s own `has_glyph` answers the question incorrectly.
See `DEFECTS.md` D12 and the module header.

### `const ICON_PTS`

The ribbon does not use this — it reserves a square from
`egui_shell::theme::Metrics::icon_pts` and hands the rect to the painter,
which is the right layering: the shell owns its own metrics. This is the
size for menus, the status bar and anything else drawn with [`image`].

# Recorded deviation: 16 pt, not the 18–20 px the ui-spec suggested

A toolbar button is 28×24 pt and egui's default `button_padding` is (4,1),
leaving a 20×22 pt content box. The ui-spec
§4.1 asked for "roughly 18–20px … leaving a few px of padding on every
side"; those two halves of the sentence conflict — 18 pt in a 20 pt box
leaves 1 pt, not "a few".

16 pt honours the paragraph's actual intent (the click target stays
meaningfully larger than the visible glyph — the Fitts's-law win the spec
is really asking for), leaves 2 pt of padding horizontally and 3 pt
vertically, and pairs optically with egui's 12.5 pt body text on
icon+text controls, which an 18 pt glyph does not. It also happens to
match `egui_shell`'s own `Metrics::icon_pts` for the Quiet and Dark
presets, so the two families of control agree by default.

### `fn image_tinted`

The texture is rasterized at `ICON_PTS * pixels_per_point()` PHYSICAL
pixels and then declared to be `ICON_PTS` logical points wide, which is
what makes it crisp on a HiDPI display instead of a stretched blur. See
this module's header, "DPI".

Prefer [`paint_icon`] where a `Painter` and a rect are already in hand
(anything inside a laid-out widget); this returns an [`egui::Image`]
widget for the case where the caller is composing a layout and wants the
icon to take part in it.

### `fn image`

The tint is `ui.visuals().text_color()` read from the CALLER's `Ui`,
which is what makes an icon inside `add_enabled_ui(false, …)` fade in
lockstep with the text beside it, with no disabled-state logic of its
own. It is also why no colour is chosen here: the theme already set that
visual.

### `fn selected_image`

Two cues at once, neither of which is the background fill egui already
paints: the accent tint AND [`IconWeight::Bold`]. That layering is the
standing "selected state is never colour alone" rule surviving the loss
of a text label to embolden.

# WHICH BACKGROUND THIS GLYPH IS DRAWN ON, since it is not this
function that paints it

The plate underneath is **`egui`'s selected-widget fill** — the theme's
[`egui_shell::theme::Palette::selected_plate`]. `egui` substitutes it into
both `bg_fill` and `weak_bg_fill` for anything carrying `SELECTED_CLASS`
(`egui-0.35.0/src/widget_style.rs:151-154`), so a toggle drawn with
`Button::image(...).selected(true)` gets that plate whether or not the call
site mentions a colour. This function's only job is to put the ink that
reads on it into the glyph.

[`Theme::selected_widget_ink`] is *defined* as that ink, and
`egui_shell::theme::tests::the_selected_widget_accessors_agree_with_the_style_egui_will_paint`
asserts it equals `visuals.selection.stroke.color` in every preset — so the
pairing is held by an assertion, not by this paragraph.

# Why not `ui.visuals().selection.stroke.color`, which is the same value

Same value, different promise, and `check-selection-channel.sh` forbids the
raw read here for that reason. `visuals.selection` is a raw `egui` channel
whose meaning the theme decides, and re-pointing it is cheap: it has
variously carried the canvas's 27 % wash (defect T2), `accent` + `on_accent`
(which breaks the focused-`TextEdit` ring `egui` drives from the *same*
field), and `selected_plate` + `accent`. Each re-pointing silently changes
what this glyph is tinted with, and nothing here fails. A named accessor
cannot drift that way: it is checked against the shipped style, and a
re-pointing has to walk past a red test that names this call site.

Note the ink is deliberately NOT [`egui_shell::theme::Theme::accent_pair`]'s
`on_accent`. That pair is the *emphasised action* surface — the full accent
at full strength — and a selected toggle is a quieter thing: a diluted plate
with accent ink. Tinting this glyph `on_accent` would put a near-white mark
on a pale plate, which is `DEFECTS.md` D2 exactly.
