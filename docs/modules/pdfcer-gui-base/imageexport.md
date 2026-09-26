# `imageexport` — what an image export IS, decided before
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

### `enum ImageFormat`

Deliberately a shell enum rather than a string or an extension. The
extension is *derived* from it ([`Self::extension`]) rather than being it —
the reverse would mean a plan could hold `"jpg"` and `"jpeg"` as two
different formats and the match arms would have to keep agreeing.

### `const ALL`

PNG first because it is the answer that is right for a drawing and
wrong for nothing; JPEG second because it is the one an operator will
look for by name; SVG third because it is the one whose consequences
(text becomes outlines) need reading about first.

**EMF last, and that is a statement rather than an afterthought.**
It is the specialist answer — the one to reach for when a named
program refused the SVG — and putting it above SVG would offer the
narrower format to an operator who has not yet discovered they need
it. Its hint says which programs, by name.

### `fn can_be_transparent`

The one function in the module that is a statement about the file
formats rather than about pdfcer. PNG has an alpha channel (ISO
15948 §6.1); SVG is a document with a background nobody has to paint;
JPEG (ITU-T T.81) has neither and no version of it ever will.

# EMF is `true`, and the reason is subtler than the other three

A metafile is a **list of drawing commands**, not a surface, so there
is nothing to be transparent: where nothing was painted, nothing was
recorded, and whatever the metafile is played onto shows through. That
is what `EmfOptions::background: None` means — *do not record an
opening fill* — and it is the engine's own default.

The engine's CLI states the same reading in the same words at the site
that uses it: *"`--transparent` is EMF's natural state (nothing is
drawn where nothing was painted) and `--background` an opaque first
fill."* Sourced there rather than inferred here, because "can this
format hold transparency" is a claim about somebody else's format and
getting it wrong in the optimistic direction ships a window that
promises a clear background and a file that has a white one.

What EMF cannot do is **per-primitive** alpha — a half-opaque
rectangle. That is a different property, it is not what this predicate
asks, and it is disclosed by name after the export
(`crate::text::export_image::emf_fidelity`) because every such
primitive silently became a bitmap.

### `fn is_vector`

Read by the window to decide whether to offer a JPEG quality control,
and by the resolution hint, which means a different thing for a vector
format and has to say so.

EMF joins SVG here on the property that is actually being asked
about: the resolution is a **recording scale** rather than a pixel
count, so the hint has to say the second thing, and there is no
quality control because nothing is being compressed.

It is deliberately **not** the predicate that chooses the writer.
`crate::app::actions::export::image` matches on the format itself, so
that adding a fifth format is a compile error there rather than a
silent routing into whichever branch this happens to select. Two
vector formats going through one `if` is exactly how the second one
gets written out as the first.

### `enum PageScope`

Lives here rather than in the dialog because [`resolve_pages`] is the
function worth testing and it takes one, and because a test of "what does
*All* mean on a three-page document" should not need a window.

### `enum Impossible`

See the module header. One variant today; the enum exists so that the
sentence and the condition cannot drift apart, and so that a second one
cannot silently borrow this one's wording.

### `fn impossible`

The whole of the module header's argument lands here. It is checked in
**two** places and that is deliberate rather than redundant:

1. The window, which disables the control and draws
   `crate::text::export_image::jpeg_has_no_alpha` beside it, so the
   combination cannot ordinarily be requested at all.
2. The writer, which refuses and writes nothing.

Two mechanisms for one rule, because they fail differently. A window
can be bypassed — a keymap, a restored plan, a later build with a
different window — and the property that must survive all of those is
*pdfcer never flattens a page onto white without saying so*. A guard
only in the window would make that property a property of the window.

### `struct EmfCounts`

# Why this exists at all, when `pdfcer_render::emf::EmfOutcome` already
carries every one of these numbers

Because `EmfOutcome` **cannot be constructed from outside
`pdfcer-render`.** It is `#[non_exhaustive]` and derives no `Default`, so
a unit test in this crate has no way to make one — not even an empty one.

⇒ That is not a complaint about the engine's API; `#[non_exhaustive]`
without `Default` is the correct shape for a value only the engine should
ever produce. But it means that if
`crate::text::export_image::emf_fidelity` took an `&EmfOutcome`, **the
mapping from counters to sentences would be untestable** — the one part of
the whole EMF path that is pure, that has eleven branches, and that is
therefore the part most worth testing.

The sibling `ExportTally` derives `Default`, which is exactly why
`svg_fidelity` was able to take the engine's own type and be tested
against it. The asymmetry in the engine's derives is the whole reason for
the asymmetry here.

# The conversion is deliberately a dumb field copy

[`Self::from`] does nothing but move eleven numbers across. It has no
branch, no arithmetic and no judgement, so the thing that could go wrong
in it is a *transposition* — reading `blend_modes_dropped` into
`gradients_rasterised` — and that is caught by reading eleven adjacent
lines rather than by a test. Everything that requires judgement happens
downstream of this struct, where a test can reach it.

A future field on `EmfOutcome` will NOT appear here and will NOT be
disclosed. That is the standing cost of the copy, it is the reason the
conversion lists the engine's field names verbatim, and it is what
`emf_fidelity`'s "everything else was geometry" line would then quietly
over-claim. If the engine adds a counter, add it here in the same commit.

### `fn is_exact`

**Not** `tally.is_exact()` on its own. The tally describes the
*recording*, which is shared with the SVG writer and knows nothing
about EMF's missing alpha; a page that recorded perfectly and then had
forty translucent rectangles turned into bitmaps has an exact tally
and an inexact metafile. Asking the tally alone is how a disclosure
comes to say "nothing had to be approximated" over a file that is half
pictures.

`dashed_strokes_pre_applied` counts here for the reason
`svg_fidelity` counts it: the picture is right and the *editability*
is gone, which is a loss an operator who opens the file to change a
dash pattern meets and nothing else would have told them about.

### `fn scale_for`

PDF user space is 72 units to the inch (ISO 32000-1 §8.3.2.3), so this is
the definition rather than a convention — the engine's own SVG writer
computes `raster_dpi / 72.0` in the same words.

It is a function rather than an inline division at three call sites
because the three call sites are the raster render, the SVG options and the
pixel-size preview the window draws, and a preview that disagreed with the
render by a stray rounding would be a preview that lies about the file.

The division itself has moved one level further out, to
[`crate::units::scale_from_dpi`]. The argument above is unchanged and is
now a smaller version of the same one: three call sites in this module
wanted one spelling, and three modules in this crate wanted one spelling.
What stays here is the guard and the fallback, which are about a corrupted
preference rather than about an inch.

### `fn pixel_size`

Rounded the way the renderer rounds — `ceil`, so a page never loses its
last column — and returned as the pair the window shows and the guard
checks.

### `fn resolve_pages`

The typed case delegates to `crate::dialogs::print::tabs::parse_page_range`
— **the print dialog's parser, called rather than copied**. Three surfaces
already do this (`dialogs::ocr`, `dialogs::insert_pages`, the print dialog
itself), and that module's own doc gives the reason: a second range parser
is a second set of answers to *"is `1,1` two exports of page one?"* and
*"does `5-3` mean anything?"*, and an operator who learns the syntax in one
window is entitled to it in the next.

`None` is *"you typed something that names no page"*, which is the window's
signal to refuse the Export button and say so. An empty `Some` is not
produced — the parser does not return one — but is treated as `None` by the
caller anyway, on `dialogs::ocr`'s precedent.

### `fn output_path`

# The multi-file convention, and why it is not invented here

One page: the file is exactly what they typed. Several: the chosen name
becomes a **stem**, and each page gets `-p<N>` before the extension, 1-based
so it matches the page number in the window and on the status bar.

That is Acrobat's own behaviour for *Export ▸ Image* (it writes
`Base_Page_1.png` from one save dialog), and `crate::text::tool`'s standing
rule about conventions makes a reference application's answer the default
rather than a shortcut. The alternative — a folder picker plus a separate
name box — is two questions for one act, and it asks the second one before
the operator has thought about the first.

The window states the pattern **before** the save dialog opens
(`crate::text::export_image::multi_page_naming`), because a save dialog
cannot say "the name you type is a stem" and an operator who did not expect
it would go looking for a file that is not there.

# The extension comes from the FORMAT, not from what was typed

An operator who types `drawing.pdf` into a PNG export gets `drawing.png`.
`export_form`'s opposite rule — the extension picks the format — is right
there because four formats share one picker and the operator has no other
way to choose. Here the format is already chosen, in a radio group, above
the button they pressed; letting a stray extension override it would mean a
window that shows one format and writes another.

# `set_file_name` with the extension already on it, NOT `set_extension`

This looks like the long way round and it is the only correct one, and the
finding is worth keeping because a neighbouring module gets it wrong in a
comment.

`Path::set_extension` is the obvious call here and it is the wrong one: it
replaces everything after the LAST dot, so `plan.rev2` +
`set_extension("png")` is `plan.png` — the revision silently deleted. The
reasoning that makes it look safe — *"a document called `plan.rev2.pdf` has
a stem of `plan.rev2`, so appending would produce `plan.rev2.png` either
way"* — is true in its first clause and does not reach its conclusion.

⇒ On a CAD desktop that is not a cosmetic difference. `plan.rev2.pdf` and
`plan.rev3.pdf` both export to `plan.png`, and the second one **overwrites
the first** in a save dialog that offers to do exactly that. So the name is
assembled as one string and set once, and
[`tests::a_dotted_document_name_keeps_its_revision`] is what holds it there.

### `fn suggested_path`

Beside the document and named after it, with the chosen format's extension
— `super::export::suggested_path`'s rule and its reason: *"a picker that
opens in the last-used directory of some other application is a picker that
makes the operator navigate back to their own project every time."*

Assembled the same way [`output_path`] is, and for that function's stated
reason: `set_extension` on a `plan.rev2` stem eats the revision.
