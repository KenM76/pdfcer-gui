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
