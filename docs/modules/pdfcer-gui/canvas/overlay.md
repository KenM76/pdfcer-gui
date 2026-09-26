# `canvas::overlay` — what the selection looks like, and what it must never look like

## Rule 4 is the whole design constraint of this file

`D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`, second and fourth
clauses of the disclosure rule:

> **Applied content renders exactly as saved content will render.** No
> badge, red flag, dashed outline or "provisional" layer drawn into the
> page view. […] **A pre-commit affordance is not content marking.** A
> snap indicator, a hover highlight, a rubber-band, a selection handle —
> these are the *cursor*; they describe what is about to happen and they
> are welcome.

Everything this module paints is in the second category and nothing is in
the first. Outlines, grips, rubber-bands and a move ghost all describe
*what the operator is about to act on*, and all of them disappear the
instant the selection does. Nothing here is keyed on a property of the
**content** — not "this text was OCRed", not "this bound is approximate",
not "this font was substituted". Those are inferences, they owe an
off-canvas report, and `panels`' own header records where that report goes:
a sentence in the Properties panel, never a dashed outline on the page.

The one-line test, from the same source: *would a screenshot of the
editing canvas differ from a screenshot of the same document saved and
reopened?* With nothing selected, this module paints **nothing at all**,
so the answer is no by construction.

## Colours come from the theme, never from a literal — and by their ROLE NAME

Every colour here is read from the theme through the two purpose-named
accessors at the bottom of this file, [`ink`] and [`fill`]. A hard-coded
colour would be correct in one theme and invisible or shouting in the
other, and `panels`' scroll-bar note records that exact failure already
measured once in this project: a control that was present, opaque,
correctly sized and invisible in a capture.

**Never `visuals.selection.stroke` or `visuals.selection.bg_fill`.**
`egui::Visuals::selection` is `egui`'s channel for styling **selected
widgets**, not a canvas role. Point it at the canvas and every selected
chrome control in the application — nineteen `selectable_label` and
`Button::selected` sites — is painted with the colours on this page: a
measured luminance gap of **72.5** in the Dark preset, against a floor of 90.

The two addresses carry the same values; only the role name differs, and that
is the point. The standing lesson is *a correctly-sourced value used for the
wrong role passes every gate — expose the pair behind a purpose-named
function.* `tools/gates/check-selection-channel.sh` fails the build for any
file outside the theme module that reads the widget channel.

## Why the outline is grown before it is drawn

[`visible_outline_rect`], and the reason is legibility. A horizontal rule
has a real, finite page bbox that is **exactly zero high**; it hit-tests,
selects and lists correctly, and its outline puts nothing on the screen.
The operator's click was right, the selection state was right, and the
feedback was a blank page — a correct action with no feedback is
indistinguishable from a broken one.
