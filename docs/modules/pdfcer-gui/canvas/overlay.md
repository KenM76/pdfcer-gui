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

### `mod anchors`

Re-exported below rather than left behind a module path, so every call site
writes `overlay::ANCHOR_PX` / `overlay::draw_anchors` and nothing outside
`canvas/` learns that the anchors have a file of their own.

### `const SELECTION_OUTLINE_REGION`

It is the box the eight grips are laid out on, not the union of the
outlines: `visible_outline_rect` widens a degenerate outline to a minimum
extent so a hairline is still grabbable, and a check aiming at the un-widened
rect would miss the grips on exactly the objects that needed the widening.

It lives in the parent rather than in [`anchors`] because it names the
OUTLINE and the grips — `overlay`'s subject — and not the points. Three call
sites publish it: the annotation branch, the content grip box, and
`canvas::forms`' widget box.

### `const MIN_OUTLINE_EXTENT_PX`

Sized to be unmistakably visible without materially misreporting where the
object is: at 6 pt a horizontal rule's outline reads as a thin band centred
on the rule. The Properties panel states the object's true size, so the
enlargement can never be mistaken for the object's real extent — which is
what keeps this a legibility fix rather than a silent widening (rule 4 is
satisfied by disclosure, not by declining to draw).

### `fn visible_outline_rect`

# The bug this closes

A horizontal rule (`100 200 m 300 200 l S`) has the page bbox
`100,200 → 300,200`: real, finite, and exactly zero high. `rect_stroke`
with `StrokeKind::Inside` then has no interior band to fill and puts
nothing on the screen.

# Why in SCREEN space, and why symmetric

Applied after the canvas→screen projection, so the guaranteed thickness is
a constant number of on-screen points at every zoom — the same
zoom-invariance discipline
[`crate::canvas::mapping::screen_tolerance_to_page`] applies to the catch
radius. Growing symmetrically about the centre keeps the band straddling
the rule rather than sitting to one side of it, so the outline still says
truthfully *the object is here*.

A non-finite rect is returned unchanged: there is no meaningful centre to
grow about, and a NaN box is a bug to leave visible upstream rather than
repair here.

### `fn grip_box`

Shared by the painter and the hit test so the drawn grips and the live
grips are the same squares. Two derivations of one box is how an operator
ends up aiming at a handle and getting a marquee.

### `fn ghost_box`

# Why this exists rather than [`grip_box`] doing it

`grip_box` answers *"where are the grips for a **content** selection"*, and
its two callers both want exactly that: `pressing::grabbable` reaches it
only after the annotation, ce-dimension and widget arms have already
returned, and the rect published for a driven check to aim at is the
content one. Widening it would change what those two mean.

⚠ There is no `canvas-grip-box` trace region. The grip box is published
under [`SELECTION_OUTLINE_REGION`], deliberately, for the reason recorded at
that call site.

This answers a different question — *"what rectangle is the operator
dragging?"* — and for a markup annotation that is its `/Rect`, which lives
on [`crate::canvas::selection::AnnotSelection`] and not in
`SelectionState::outlines`.

# What it prevents — `OPERATOR_REQUESTS.md` O154 and O209

> *"the Markup Items don't have a live preview — the bounding box stays the
> same size when I drag the handles."*
>
> *"there is no live preview when I drag the handles to resize them"* — the
> same sentence one surface along, about a form field.

`selection.outlines()` is **empty** behind an annotation selection, so a
ghost measured against `grip_box` draws nothing at all and the `None` that
guard returns stops the ghost being reached in the first place. Every ghost
therefore measures against **this** function. The ghosts are separate
functions for a good reason — a move is one displacement, a resize is a map
— and that split is exactly what lets one of them quietly miss a kind of
selection the other handles, so the box they share is stated once, here.

# `widget`, and why it is a parameter rather than a third arm

A form widget is in **neither** of the two places this function can look. It
is not `selection.annot()` — `canvas::selection` excludes `/Widget` by name
— and it is not in `selection.outlines()`, because the selection lives on
`OpenDoc::selected_field` instead. So the guard answered `None` and the
resize ghost was unreachable for exactly the selection the operator was
dragging: O154's defect, verbatim, on the surface that had not been checked
for it.

It arrives as a parameter because this module is not allowed to know what a
form is. The caller passes `canvas::widgetdrag::grab_box`, which is the same
rectangle `pressing::grabbable` hands the drag itself — so the preview, the
hit test and the commit are one box and not three.

It takes **precedence**, which costs nothing: a widget selection and an
annotation or content selection are mutually exclusive, so at most one of
the three arms can be `Some` on any frame.

### `fn draw_selection`

# The ghosts live next door

[`draw_move_ghost`], [`draw_resize_ghost`] and [`draw_rotate_ghost`] are
separate functions, and each is offered only once the matching eligibility
check — [`crate::canvas::moving::eligible`] for a move,
`canvas::resizing::action` for a grip drag — has established that the
release will reach a real verb on real operands.

That condition is the rule, and it outlives any one gesture: **a
pre-commit affordance that describes something which does not happen is not
an affordance, it is a lie at a low alpha.** A ghost ships in the same change
as the verb it previews, never ahead of it.

### `fn draw_grips`

Filled with the theme's window background and stroked in the selection
colour: a filled square reads as a handle at any zoom and against any page
content, where an outline-only square disappears over dense linework —
which is precisely the document class pdfcer is for.

# `offer` is the hit test's own value, and that is the contract

It is [`crate::canvas::pressing::grabbable`]'s answer, passed straight
through — **never a predicate recomputed here**. Rule H7, and it is the
difference between two representations of one decision and one:

* a handle painted and **not** hit-tested is the *"visible control, silently
  inert"* failure this project spends its time removing;
* a handle hit-tested and **not** painted is worse — an invisible target
  that steals the press aimed at whatever is under it.

The two flags genuinely differ per selection — a ce dimension turns and does
not scale, a form field's box scales and does not turn — so this function
must never read *"there is a box"* as *"there are nine handles"*. It draws
exactly what it was told.

### `fn draw_grips_in`

The eight squares and the rotate handle follow the frame, so on a turned
annotation they sit on the outline that is actually drawn. Anchoring them to
the upright bound while the outline turned would leave eight squares
floating in the space between the mark and its bounding box — a picture that
says nothing true about anything.

### `const ROTATE_HANDLE_REGION`

Distinct from [`SELECTION_OUTLINE_REGION`] because the handle is the one
affordance that cannot be derived from the outline: it sits on a stem
outside the box. Its presence in a trace is also the honest answer to *"does
this selection offer a rotation at all?"* — which is a question a driven
check has to be able to ask about a **form field**, where the answer must be
no.

### `fn draw_annot_ghost`

Its own function beside [`draw_move_ghost`] rather than a case of it, and
the reason is what the two iterate. That one walks
`SelectionState::outlines()` -- the CONTENT selection's rectangles -- which
is empty for an annotation selection by construction, because the two
selections are mutually exclusive and live in different fields. Handing an
annotation drag to it would draw nothing at all, silently, which is the
"the gesture does nothing" symptom in the place hardest to notice: the drag
would still commit on release.

It takes the rectangle already computed rather than a delta plus the
selection, because `annotdrag` has to decide the same rectangle to know
whether a drag is eligible at all. One computation, one answer, and the
preview cannot promise a landing spot the commit disagrees with.

### `fn draw_move_ghost`

`delta` is in **canvas space** — the same space the cached outlines are in,
which is what makes this a translation and nothing more.

# Why this costs no re-raster and no re-decomposition

Three facts line up, and the preview is affordable because of all three:

1. **The outlines are cached in canvas space.**
   [`SelectionState`] keys them on
   `(page, edit epoch)`, neither of which moves during a drag, so no
   decomposition happens on any frame of the gesture.
2. **Canvas space is zoom-independent**, so translating a cached rect by a
   canvas-space delta and projecting the result is exact at every
   magnification — there is no per-frame re-derivation of geometry.
3. **Nothing touches the page texture.** A ghost is two strokes on the
   painter that is already open. The raster is invalidated by
   `Action::Move*` on *commit*, once, in `app::actions` — not by the
   preview, which is the whole reason the preview is a preview.

A ghost that re-rendered the page per frame would be a different feature
wearing the same name: on the CAD sheets pdfcer exists for, one raster is
tens of milliseconds and a drag is sixty frames a second.

# Rule 4

This is a *pre-commit affordance* — the cursor describing what is about to
happen — and rule 4 admits those explicitly, alongside the snap indicator,
the hover highlight and the rubber-band. What rule 4 forbids is marking
content that has **already been applied**, and nothing here survives the
release: the ghost exists only while the pointer is down. The one-line test
in this module's header still answers no — with nothing being dragged, this
paints nothing at all.

### `fn ghost_is_owed`

The ghost is the feedback of **last resort**: an outline per selected thing,
displaced by the drag. It is withheld in exactly one case — something better
is already on screen, which means a shape preview carrying the real anchors
travelling, and a perimeter box on top of that is O63's complaint.

`outline` is `pressing::grabbable`'s flag, true at the object rung, where
the ghost is always owed.

⚠ **A preview that exists and is EMPTY does not count**, and that is the
whole subtlety: `shapes::transformed` returns a preview with no shapes for a
text object, having no path to transform. An empty preview shows the
operator nothing, so it may not stand in for the ghost — a gate that tested
`is_some()` would withhold the only feedback a text-chunk drag has.

### `fn draw_rotate_ghost`

**It draws a quadrilateral, not a rect**, and that is the whole visible
difference. Drawing the rotated bounding box instead would show the operator
a shape that grew as they turned it — a preview of something the release
does not do.

The angle is the **un-negated** screen-space one from
`rotating::angle`, and the rotation is `rotating::rotate_about` — the same
function that module's own test pins against the measured bearing. The
commit negates once, at the page crossing; see `rotating::drag`.

# An ANNOTATION's ghost is drawn by this same function

…unlike the **move** ghost, which `canvas::painting` carries in a separate
`annot_ghost` slot. The asymmetry is deliberate and is about the arithmetic
rather than about tidiness:

* a move ghost is a **translated rectangle**, and `annotdrag` computes it in
  canvas space as part of deciding whether the move is eligible at all — so
  the value already exists and carrying it is free;
* a rotate ghost is **four corners turned about a centre**, and that is the
  identical calculation whether the corners came from a content outline or
  from an annotation's `/Rect`. Two functions would be one function written
  twice, and the second copy is where a preview and a commit come to
  disagree about which way round something went.

The annotation case **returns early**, mirroring `draw_selection`'s own
structure one screen up. An annotation selection and a content selection are
mutually exclusive by construction (`SelectionState` enforces it in one
place), so the early return is a statement of that invariant rather than a
precedence: `selection.outlines()` is empty behind an annotation anyway, and
falling through would have drawn nothing — silently, which is how a missing
preview reads as *"the drag stopped tracking"*.

### `fn draw_resize_ghost`

The anchor is in **screen** space, because that is the space the outlines
are projected into and the space [`crate::canvas::handles::Grip::anchor`]
already answers in. Converting to PDF for the preview and back again would
be two conversions for a picture that is thrown away next frame — and, worse,
a second place for the ghost and the commit to disagree about which corner
stayed still.

### `struct FindHighlight`

The whole vocabulary this module needs about Find: **where**, in canvas
space, and **whether it is the one the view is on**. Deliberately not a
`TextMatch`, not a page index and not a `Quad` — the projection from
unrotated PDF user space happens once, at search time, in
[`crate::find::Hit::canvas`], so this file is never told what a PDF is and
that file is never told what a `Painter` is.

### `fn draw_find_hits`

`hits` comes from [`crate::find::FindState::page_highlights`], which
yields **nothing at all** when the results are not current — so an edit,
a changed query or a closed bar all stop the highlights here by supplying
an empty iterator rather than by a check in this function. That is the
mechanism by which rule 4 is kept: this module cannot paint a mark over
content the search no longer describes, because it is never handed one.

# Rule 4

A find highlight is a **pre-commit affordance in the second category** of
this module's header — it describes what the operator is looking at, not a
property of the content, and it disappears the instant the bar closes.
Nothing here is keyed on a property of the document: it marks *the answer
to a question the operator just asked*, which is the same class as a hover
highlight. The one-line test still answers no — with the bar closed, this
paints nothing at all.

# Why the rects are grown

Through [`visible_outline_rect`], for the same reason a selection outline
is: a text run's quad can be degenerate on one axis (a page whose
producer emitted a zero-height box, or a hit so small at the current zoom
that it rounds to nothing), and a highlight that puts no pixels on the
screen is indistinguishable from a search that did not work.

### `fn draw_text_selection`

`boxes` come from [`crate::canvas::textsel::TextSelection::highlights`],
which yields **nothing at all** for another page or for a revision the
selection no longer describes — so an edit stops the wash here by supplying
an empty slice rather than by a check in this function, exactly as
`find::FindState::page_highlights` arranges for the search wash. That is the
mechanism by which rule 4 is kept: this module cannot paint a mark over
glyphs the selection no longer describes, because it is never handed one.

# Rule 4

A selection wash is a **pre-commit affordance** in the second category of
this module's header — it is the cursor, describing what a copy would take —
and it disappears the instant the selection does. Nothing here is keyed on a
property of the *content*: it marks a range the operator just swept. The
one-line test still answers no; with nothing selected this paints nothing.

# Unstroked, where a find hit is stroked

[`draw_find_hits`] strokes the **current** hit because it has to be told
apart from the other hits on the page. A text selection is one thing, so
there is nothing to distinguish it from — and a stroke round each line box
would draw a visible seam **between** the lines of one selection, which is a
boundary the operator did not make and which no text application draws.

# Why the boxes are grown

Through [`visible_outline_rect`], for the reason [`draw_find_hits`] gives:
a glyph box can be degenerate on one axis — a producer that emitted a zero
size, or a line so small at the current zoom that it rounds away — and a
selection that puts no pixels on the screen is indistinguishable from a
gesture that did not work.

### `fn draw_chunk_boxes`

`boxes` are canvas space, as [`crate::canvas::chunks::outlines`] returns
them; the projection is done here because it is this painter's mapping that
knows where the page currently sits.

# Outlines, not washes

[`draw_text_selection`] fills because a text *selection* is a region the
operator asked for and wants to see the extent of. These are not a
selection — they are the units a **future** click can reach, drawn so the
aim is possible. A wash over every chunk of a title block would obscure the
drawing it describes, and the thing being disclosed here is a boundary, so
a boundary is what is drawn.

# R8b

A pre-commit affordance, in the class the rule admits by name alongside
snap indicators and rubber-bands: nothing about the page's own content is
restyled, and the same document saved and reopened paints identically.
[`crate::canvas::chunks`] carries the whole argument.

# Returns how many rectangles reached the painter, and the caller
traces THAT

Not `boxes.len()`, which the caller already holds. The difference is the
whole evidentiary value of the trace line: a count taken from the argument
is true whether or not this function was called, so deleting the call would
leave the driven check passing over a canvas with nothing drawn on it. A
count taken from the loop cannot be produced without entering the loop, and
removing the call is then a compile error rather than a silent success.

⚠ Its reach ends at the painter. A stroke made transparent, a theme colour
equal to the page, or a mapping that puts every box off-screen would all
still count. Those have one oracle, and it is a rendered screenshot.

### `fn at_alpha`

Read back through `to_srgba_unmultiplied`, for the reason [`ghost`]
documents at length: [`Color32`] stores **premultiplied** components, so the
plain accessors return a hue already darkened by whatever alpha the source
carried, and re-premultiplying that darkens it a second time.

`pub(super)` rather than private because [`super::rulers`] needs the theme's
hairline at two grid alphas and [`super::guides`] needs the selection hue at
two guide alphas, and every one of those is the same premultiplication trap.
Four more spellings of it would be four more chances to reach for `.r()` and
produce a colour that is subtly wrong in exactly the theme nobody tests in.
The *alphas* stay with the surfaces that chose them, because each is an
argument about legibility over linework and belongs beside that argument.

### `fn draw_marquee`

A wash plus an outline. The wash matters on a dense drawing: an
outline-only band over a hatched region is hard to see at all, and a
rubber-band the operator cannot see is a rubber-band they cannot aim.

### `fn draw_field_shade`

`OPERATOR_REQUESTS.md` O96 — *"in our display section we should have an
option to shade the form fields like acrobat does."*

# Why this is an affordance and not the tint rule 4 forbids

The standing rule is *applied content renders exactly as saved content will
render*. A field is **not content**: it is a control, and this wash is the
same class of thing as the pointing hand `crate::canvas::forms` already puts
over a widget. It marks no inference and says nothing about pdfcer's
confidence in anything.

The property that keeps that true is **where it is painted**: here, in the
canvas overlay, over the finished page texture. It reaches no rasterizer, so
it cannot appear in a print, an export, a Save or a `render-page`.

# The colour, and why it is derived rather than named

The theme's own **hyperlink** colour at a low alpha. Not `selection`, which
is what a selected object wears and would make every fillable field look
selected; and not a literal, which would not track light and dark. Hyperlink
is the theme's "this is interactive, click it" role and that is exactly what
a fillable field is.

[`FIELD_WASH_ALPHA`] is lower than the marquee's, deliberately. A band is
transient and the operator is looking *at* it; this sits under the page's own
content for as long as the document is open, and a wash that made a filled
field's own text harder to read would have traded one legibility problem for
another.

### `fn draw_field_target`

A wash *and* a hairline, where [`draw_field_shade`] is a wash alone, and the
difference is what the two are for. The wash answers *"which of these can I
type in"* and sits under a document the operator is reading, so it is
deliberately faint. This answers *"where is the box I am about to click"*
in the mode whose whole job is moving and resizing those boxes — and a box
with no `/MK` background and an empty value has no pixels of its own at all,
so a faint fill over nothing is still nothing. The outline is what makes an
empty field a thing on screen.

Same hue as the wash, at [`FIELD_TARGET_ALPHA`], so the two read as one
family rather than as two unrelated marks — and *not* the selection ink,
which is reserved for the one box the operator has actually picked.

Rule 4 is satisfied for [`draw_field_shade`]'s reason and no other: this
is painted in the canvas overlay, over the finished page texture, so it
reaches no rasterizer, no print, no export and no Save. It marks no
inference — a widget either has a rectangle or it is not in the list.

### `fn draw_field_spotlight`

A **stroke**, where [`draw_field_shade`] is a fill, and the pair is
deliberate: every fillable field already wears the wash, so a spotlight that
was a stronger wash would be a difference of degree that an operator has to
compare two boxes to notice. An outline is a difference of *kind* and reads
at a glance, which is the whole job.

The theme's **selection stroke**, because that is what the operator is
doing — they have picked this field out of a list, and every other "this is
the one I mean" on this canvas wears the same colour.

`StrokeKind::Outside`, so the outline sits *around* the field rather than
over its first and last characters. A middle-aligned stroke on a tight text
box eats the glyphs at both ends, which is worst on exactly the short fields
— a date, a revision letter — where every character matters.
