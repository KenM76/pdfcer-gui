# `app::actions::imageexport` — what an image export IS, decided before
anything is written

## Why this is a module and not four fields on a `WriteAction` variant

[`super::write::WriteAction::Dxf`] carries `pdfcer_core::export::dxf::
DxfOptions` — **the engine's own struct** — and its doc says why: *"it **is**
the value the writer takes, and rebuilding it in the apply arm would put a
second constructor in the path."*

There is no engine struct to carry here, and that is a fact about the
feature rather than a gap in the engine. The engine offers four unrelated
writers — `export::encode_png`, `export::encode_jpeg`, `svg::export_svg_view`,
`emf::export_emf_view` — each with its own options type and its own error
type, and *"which of them, over which pages, at what resolution, keeping
transparency or not"*
is a question none of them asks. It is a **shell** question, invented by the
window, and the shell owns the type that answers it.

⇒ So this module holds the vocabulary and, more importantly, **every part of
the decision that can be got wrong without a window open**: which
combinations are impossible, which pages a scope names, what each file is
called, and what raster scale a resolution is. All of it is pure, all of it
is tested, and none of it needs an `egui::Context` to prove.

## [`Impossible`] is the point of the module

The operator asked for *"full support (including transparency where
supported!)"*. The parenthesis concedes that one of the four formats cannot
do it and asks pdfcer to be the one that says so. The engine's note is
imperative about the same thing:

> **refuse a "transparent" JPEG by name in your UI, never flatten silently**

A `bool` returned from a validity check would satisfy the letter of that and
lose the name. [`Impossible`] is an enum with one variant for exactly that
reason: `crate::text::export_image::refused` matches on it, so the day a
second impossibility is named the compiler asks for its sentence rather than
letting it inherit the first one's. **A refusal without a name is a refusal
that will one day be worded for the wrong reason.**

## Why the plan carries a RESOLVED page list

`pages: Vec<usize>`, not a scope and a string. The window already has to
parse the typed range to decide whether its Export button is usable, so
parsing again in the apply phase would be a second call to the same parser
against a document that may have changed pages in between — the exact
staleness `super::export::dxf`'s header refuses for `PageObjects`, arriving
by a different door.

`ExportDxfDialog` freezes its page index at open for the same reason and
states it: *"an operator who opens this on page 7 and pages away must not
export page 9."*

## Rule 15

Nothing here reads the dimensioning model, so neither **ce dimensions** nor
**pdf dimensions** appear in this module. [`ImagePlan::dpi`] is a
*resolution*, which is a claim about pixels; it is not a *scale*, which is a
claim about the real world and is what `export::dxf` needs a whole window to
establish. **A picture has no scale to get wrong**, which is why this
feature is safe to offer without the calibration machinery the DXF export
cannot ship without.

## Item notes

### `fn a_transparent_jpeg_is_impossible_and_says_which_combination_it_is`

The assertion this module exists for. The engine's note: *"refuse a
'transparent' JPEG by name in your UI, never flatten silently."* The
operator's own parenthesis — *"(including transparency where
supported!)"* — is what makes it a requirement rather than a nicety.

Note what is asserted about the plan itself: `transparent` is still
`true` afterwards. A plan that quietly cleared the flag would pass a
naive check and would BE the silent flatten.

### `fn jpeg_is_the_only_format_with_no_alpha`

EMF is asserted **transparent-capable** here, which is the claim
most likely to be "corrected" wrongly by a future reader who knows
that a metafile has no alpha channel. It has no *surface*: nothing is
recorded where nothing was painted, so what the metafile is played
onto shows through. `EmfOptions::background: None` — the engine's own
default — is that state, and the engine's CLI calls it *"EMF's natural
state"* at the site that sets it.

### `fn both_vector_formats_report_as_vector_and_the_rasters_do_not`

`is_vector` decides what the *resolution hint* says and whether a JPEG
quality control is drawn. It deliberately does not choose the writer:
`app::actions::export::image` matches on the format, so a fifth format
is a compile error there rather than a silent write of an EMF through
the SVG encoder — which is precisely what an `if is_vector()` would
have done on the day EMF was added.

### `fn no_two_formats_share_an_extension`

A duplicate here would be invisible: the window would show four
radios, two of them would suggest the same file, and the second export
would silently overwrite the first behind the save dialog's generic
warning — the same defect shape `a_dotted_document_name_keeps_its_revision`
records, arriving by a different door.

### `fn several_pages_become_a_stem_and_a_page_number_starting_at_one`

The number is what the operator sees on screen, not the index — an
off-by-one here produces a set of files whose names disagree with the
document by one, which is exactly the kind of wrongness nobody notices
until they are matching drawings to a schedule.

### `fn the_extension_is_the_formats_and_overrides_whatever_was_typed`

The window shows a radio group; a stray extension in the save dialog
must not silently override it, or the window shows one format and the
file is another.

### `fn a_dotted_document_name_keeps_its_revision`

`plan.rev2.pdf` is a real CAD filename shape, and it is the shape
`Path::set_extension` gets wrong: that call replaces everything after
the LAST dot, so `plan.rev2` becomes `plan.png` and the revision is
gone.

⇒ Why that matters more than tidiness: `plan.rev2.pdf` and
`plan.rev3.pdf` would both suggest `plan.png`, and the second export
**overwrites the first**, in a save dialog whose only warning is the
generic one about a file already existing. See [`output_path`]'s
header, which carries the same argument for the per-page namer.

### `fn a_range_naming_no_page_is_refused_rather_than_exported_empty`

The window turns this into a sentence beside the box and a disabled
Export button. Collapsing it to `Some(vec![])` would let the operator
answer a save dialog and receive nothing.

### `fn the_raster_scale_is_the_resolution_over_seventy_two`

The 72 is what makes a 300 DPI export of an A4 page 2480 pixels wide
instead of some other number, and it is the same arithmetic the engine
does internally for SVG. A shell that used 96 here would produce files
a third too large with a `pHYs` chunk that says otherwise — which is
precisely the Word-places-it-wrong defect, arriving from our side.

### `fn a_nonsense_resolution_falls_back_to_print_grade`

The engine does the same for `SvgOptions::raster_dpi`, and the reason
is shared: a zero scale produces a zero-pixel render, which surfaces as
an engine failure about a raster size the operator never typed.
