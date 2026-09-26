# `pdfcer-gui/canvas/presspick`

## Item notes

### `const CHANGED_KEY`

Frame-local gesture state, kept where [`crate::canvas::input`]'s header says
gesture state belongs and for its reason: it has no meaning across a
document, and a value that survived one would be a claim about a file it was
not measured on.

### `fn take`

Three outcomes, in precedence order, and the first two are what O215 ask 1
is about.

# 1. A press that is already inside this object stays inside it

The operator: *"sometimes it moves the chunk and sometimes it takes the
whole block."* [`covers`] declined a moment ago, which at the Part rung
means the press landed **outside the selected chunk's box** — on a
neighbouring chunk, or in the white between two of them. Selecting the whole
object there is what promotes the operand back to the block mid-gesture, and
the drag that follows moves everything.

So a press on the object the operator is already inside **re-picks within
it** rather than leaving it. The rung is the operator's, and only a press on
something else takes them out of it.

⚠ Gated on [`crate::canvas::chunks::boxed`] — text, more than one chunk, and
the switch on — so it is offered exactly where a box is drawn to aim at. A
path's subpaths have no such affordance, and a press on a selected shape at
the Part rung goes on behaving as it always has.

# 2. A press on what is already the whole selection changes nothing

Re-selecting an object that is already selected alone, at the Object rung,
is a write with no effect — but it is not a *report* with no effect, because
[`changed_selection`] is what the click path reads to decide whether this is
the operator's first click on this block or their second. Saying "changed"
here would make the chunk rung unreachable through the gap between two
chunks, which is where [`covers`] declines most often on a CAD note.

# 3. Anything else selects, exactly as it always did

### `fn holds_part`

Part-rung only: at the Node rung `subpath` names the container an anchor
lives in, not a chunk, so a node selection would answer this question about
a different index space.

### `fn press_selects`

**Five** conditions, each with its own reason in
[`interact`]'s step 1b: the press **landed on this canvas**, the primary
button went down **this frame**, the plain Select tool is armed, the mode
may edit content, and no region zoom is waiting to be spent.

`Shift` declines here rather than at the call site so that the whole
predicate reads in one place — a reader asking *"when does a press select?"*
gets one answer rather than a function plus a condition beside it.

# The first condition: the press must have landed on this canvas

`OPERATOR_REQUESTS.md` row **O75**:

> *"When I am working in the right side panel objects are getting selected
> through the side panel when I am trying to edit fields in the Properties
> section."*

Asking the **`Context`** — i.e. the whole window — *"did the primary button
go down this frame?"* and then mapping `press_origin()` through the page's
affine transform is what produces that. [`crate::viewer::screen_to_page`] is
unbounded and unclamped, so **any** screen point converts to a valid page
coordinate: a press on a `TextEdit` in the right dock resolves to real page
content and replaces the selection the operator was editing the properties
of.

It hides at fit zoom, because there the dock maps off the sheet and the hit
test misses. Zoom past fit on a CAD sheet — which is every working session
on an A1 drawing — and the whole window maps inside the page.

**This is `DEFECTS.md` D1's class, arrived at through the pointer
instead of the keyboard**: a guard asking exactly the right question of
exactly the wrong object. Every signal in [`interact`]'s `PointerFrame` must
come from the page's own [`egui::Response`], never from the `Context`.

# Why [`egui::Response::is_pointer_button_down_on`] and nothing else

egui resolves it from `Memory::interaction()`'s `potential_click_id` /
`potential_drag_id`, which are assigned with **full layer and z-order
awareness**. So one term rejects, at once and with no rectangle
arithmetic: a press on either dock, on the ribbon, on the document tab
strip, on the status bar, on the find bar's floating `Area`, on a
context-menu popup, and on a modal `Window`. A `clip.contains(origin)`
test would cover **none** of the last three, because all three sit
geometrically *inside* the canvas viewport.

It is also the term that keeps a legitimate gesture alive: it stays true
for the widget that **owns** the press for the whole gesture, so a marquee
dragged off the page and out over a dock — which [`interact`]'s step 1
explicitly protects — is unaffected.

**Do not substitute [`egui::Context::egui_wants_pointer_input`].** It is
true whenever *any* egui widget wants the pointer, and this canvas's page
**is** an egui widget, so it would be true during a legitimate canvas press
and would suppress selection entirely. It is the pointer twin of the D1
trap and swapping one for the other would trade a wrong selection for no
selection. (`Context::is_pointer_over_area` does not exist in egui 0.35;
it was looked for three ways before this was written.)

# Why BOTH terms, and not just the new one

`is_pointer_button_down_on` is true for **every frame the button is held**,
not only the frame it went down. `button_pressed` is the this-frame edge.
Collapsing them into one would re-run the pick on every frame of a drag —
a different defect, and a worse one, because it would re-select mid-move.

# The one narrowing this accepts, recorded rather than left to be found

`response` is the **acting** page's response — the page
[`crate::canvas::present`] chose as `active`, which is also the page
`map` describes and the page `page_index` names. In a continuous strip a
press on a *neighbouring* page therefore no longer selects. That is not a
regression: such a press was already being mapped through the acting
page's transform, so it was already resolving to a point on the wrong
sheet. It is now refused instead of answered wrongly. If per-page pressing
is wanted in the strip, the fix is to make the pressed page the acting
page, not to widen this guard.

### `fn covers`

# The guard that keeps a press on an existing selection from re-selecting

It asks `handles::grip_at` against the box
[`crate::canvas::pressing::grabbable`] answers with — the **same** two
functions [`crate::canvas::pressing::look`] asks a moment later — rather
than testing the selection's entries. A second opinion computed differently
here would disagree with the gesture machine at the margins, and every
disagreement is a press that selects when it should have transformed.

# Why the grips, and not just the box

**The rotate handle sits OUTSIDE the box** — `handles::rotate_rect` puts it
above the top edge — so a press on it is not "covered" by the body. Testing
`grip_box.contains(point)` alone leaves any object underneath the handle to
be selected by the body below, and the rotate becomes a select-and-move.

The eight resize grips are inside the box and are never at risk from that,
but they go through the same call anyway rather than through an argument
that they are safe: an argument is a thing that stops being true.

A working gesture aimed at the wrong verb is this canvas's recurring failure
mode, and it is worth naming because it never *looks* broken from a chair —
something moves.

# The box must come from `pressing::grabbable`, not `overlay::grip_box`

`overlay::grip_box` derives its answer from the selection's cached
**content** outlines, which `select_annot` clears — an annotation is not
content and has nothing decomposed to cache. Ask it about a markup or a ce
dimension and it answers `None`, `covers` is **false**, and the press falls
into the select-on-press body below — which picks the topmost *content*
object at that point and **replaces the annotation selection with it**,
twenty points away from the shape the operator was aiming at. Then
`pressing::look`, on the very next statement, finds a content selection and
the release rotates a page object: a perfect gesture, on something the
operator never selected.

`pressing::grabbable` is the one function that knows all four kinds of
grabbable box — page content, a markup, a ce dimension and a form field's
widget — which is why it is the one called here.

**The rule is about phrasing, not about this call.** "The same two
functions `pressing::look` asks" is a claim about a call site somewhere
else, held together by nothing. A guard that must agree with another module
has to **call that module**, not resemble it.

# The companion rule: order against the fork

**A guard written in one destination's vocabulary must stand AFTER the
branch that picks the destination, never before it.** Three destinations
share the rotate gesture and four share a press; a content-shaped test such
as `selection.object_indices_on(page).is_empty()` placed in front of the
fork answers about a subject the gesture may already have routed away from,
and returns before the routing decision is reached at all.
`canvas::rotating`'s header carries the same rule for its own fork.

# `GripSet::all()` here, where `pressing::look` narrows it

`look` passes `grabbable`'s own `offer`, which is narrower for three of the
four kinds. This passes `all()` unconditionally, and the difference errs
toward **declining**: this answers `Some(grip)` for points `look` will call
`None`, so the press falls through to the gesture machine unchanged instead
of re-selecting under a node the operator is in the middle of editing.
Erring the other way would be a press that silently leaves node editing.

It matters in the new direction too. A selected **ce dimension** is
offered `GripSet::rotate_only()`, so `look` will not call a corner press a
resize — but `all()` here still claims that corner for the *existing
selection*, which is right: whatever the press turns out to mean, it is
about the dimension the operator already has, not about the linework
underneath it.
