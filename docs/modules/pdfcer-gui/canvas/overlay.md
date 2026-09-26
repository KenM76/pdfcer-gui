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

## Item notes

### `const GHOST_ALPHA`

High enough to read as a *second* outline over dense linework — the whole
point is that the operator can see where the object is going — and low
enough that it never competes with the real outline, which is still on
screen showing where the object still is. Both boxes are visible during a
drag on purpose: the pair states the displacement, which one box alone
cannot.

### `const HIT_ALPHA`

Low enough that the *text under it stays readable* — the operator is
scanning the page to decide whether the hit is the one they want, and a
highlight that obscures its own subject defeats the purpose. Deliberately
the same order as the marquee's wash, which exists for the identical
reason.

### `const CURRENT_ALPHA`

**Emphasis, not hue, is what distinguishes the current hit** — and that
is a constraint rather than a preference. Every colour on this canvas comes
from the theme (see this module's header, and
`tools/gates/check-theme-colors.sh`), and the theme has no role meaning
"the search hit you are on". Borrowing one that means something else —
`warn_fg_color`, `error_fg_color` — would say *warning* on a control that
is not warning about anything, and would then be wrong the first time
somebody restyled the warning colour for warnings.

So the current hit is the same colour, more than twice as opaque, **and
stroked**. Two independent signals rather than one: alpha alone is a weak
difference on a dense drawing, and the outline is what carries it at a
glance. Acrobat and every browser use a second hue for this; pdfcer cannot,
and this is the honest substitute.

### Why 96, and the ceiling above it

**Measured on a screenshot of the running binary**, on `reflow.pdf`
searching `the`. Past roughly this value the wash becomes a solid block and
covers the word it marks — on that file the word `The` at the head of the
paragraph disappeared entirely under its own highlight. A highlight that
hides the text it is highlighting has defeated its purpose, because the
operator's next act is to *read* the hit and decide whether it is the one
they wanted.

⚠ That ceiling is invisible to a unit test: an over-opaque wash is inside
every asserted bound and passes every hue assertion, and the picture is
still wrong. It is why the alpha carries only half the emphasis and the
2.0 pt stroke carries the other half — two weak signals that survive a
dense drawing beat one strong signal that swallows its subject.

### `const TEXT_SELECTION_ALPHA`

**A wash must never hide its own subject.** [`CURRENT_ALPHA`] carries the
measured ceiling for that rule and the argument behind it; read it first.

It applies here with more force. A find hit is something the operator is
deciding about; a text selection is something they are **about to copy**,
and the only way to tell whether they swept the right words is to read them
through the wash. So this sits at the low end deliberately, at the same
value as a non-current find hit rather than the emphasised one: there is
nothing here to emphasise *against*, because a selection has no neighbours.

Equal to [`HIT_ALPHA`] and stated as its own constant rather than aliased to
it, because the two are equal by coincidence of judgement rather than by
construction — they answer different questions ("one of several answers" vs
"the thing you are copying") and a future change to either must not silently
move the other.

### `const CHUNK_OUTLINE_ALPHA`

Below the selection outline's full-strength stroke, deliberately. The two
are on the screen together and they say different things: the selection
outline says *this is what is selected*, the chunk boxes say *these are the
pieces it is made of*. If the pieces were drawn at the same weight as the
whole, the operator would read the strongest rectangle on the block as the
selection and aim at the wrong one, which is the confusion O215 ask 1
reports rather than a fix for it.

### `const FIELD_TARGET_ALPHA`

Well above [`FIELD_WASH_ALPHA`] because it is a *line* rather than an area —
a stroke at the wash's alpha over pale paper is invisible — and still short
of opaque so that a field drawn over dense linework does not read as part of
the drawing.

### `const SPOTLIGHT_WIDTH`

2.0 rather than the marquee's 1.0. This one has to be seen against a page
that may be dense linework at a fitted zoom, and it is transient — it is on
screen only while a row is focused, so it can afford to be assertive in a
way a permanent mark could not.

### `const FIELD_WASH_ALPHA`

28, against the marquee's 48. See that function's for why lower: this one
is on screen for as long as the document is open and sits under the field's
own text, where the band is transient and has nothing under it that has to
stay readable.

### `fn ink`

# Why this is a call and not `visuals.selection.stroke.color`

That address is the wrong one for a canvas. `egui::Visuals::selection` is
`egui`'s styling channel for **selected widgets**: `Style::button_style`
takes both fills *and the text colour* from it for anything drawn with
`Button::selected(true)` or `ui.selectable_label(true, …)`. Point that
channel at this canvas and the canvas wins — every selected chrome control
in the application is then painted with canvas ink, which measures as accent
text on a 27 % wash: a luminance gap of **72.5** in the Dark preset against
this project's readable floor of 90.

⚠ Reaching for it changes no *picture*, which is what makes it dangerous.
[`egui_shell::theme::Theme::canvas_selection_ink`] returns `palette.accent`,
bit-for-bit what that channel carries. The accessor's whole value is that
the colour arrives under a name saying **which role** it is, so re-tuning
chrome cannot silently re-tune the page overlay.
`tools/gates/check-selection-channel.sh` keeps the widget channel
unreachable from here.

Takes the [`Painter`] rather than a `&Context` because every drawing
function in this module already holds one and `Painter::ctx` is free. That
is deliberate: a helper whose argument the caller must go and *find* is a
helper people work around.

### `fn fill`

See [`ink`] for the whole argument: this is the other half of the same pair,
and the widget channel's `bg_fill` is off limits here for the same reason.
It returns `palette.selection_fill`.

### `fn pair`

For the functions that draw a washed rectangle *and* an outline around it —
[`draw_find_hits`] is the one — where fetching the two separately would
read as two unrelated colours and would take the theme lock twice inside a
loop.

### `fn wash`

Derived from the theme rather than named, so it tracks light and dark
without a second literal — and low enough that the content under the band
stays readable, because the operator is choosing what to enclose *by
looking at it*.

### `fn ghost`

Read back through `to_srgba_unmultiplied` rather than through `.r()`/`.g()`
/`.b()`. [`Color32`] stores **premultiplied** components, so the plain
accessors return a hue already darkened by whatever alpha the source
carried; re-premultiplying that at a new alpha darkens it a second time.
The selection stroke is opaque in both shipped themes, so the two spellings
agree today — which is exactly why the wrong one would go unnoticed until a
theme with a translucent selection stroke made the ghost a different colour
from the outline it is a copy of.
