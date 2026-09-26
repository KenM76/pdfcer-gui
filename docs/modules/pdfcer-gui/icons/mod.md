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

## ★ Why a hand-rolled SVG subset parser instead of a crate

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

## ★ Theming: one raster per icon, tinted at draw time

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

## ★ Weight, and why selected state is not colour alone

[`IconWeight::Bold`] rasterizes the same art with the stroke width
multiplied. It exists because of the standing rule that **selected state
is never colour alone**. Text toggles satisfy that rule by going bold; an
icon has no text to embolden, so the *glyph* goes bold instead.

Note the seam limitation recorded in [`paint`]: `IconRequest` does not
carry the control's selected state, so the ribbon path cannot apply the
weight cue today. It is fully implemented and reachable through
[`toggle_image`] for anything the application draws itself.

## ★ DPI

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
