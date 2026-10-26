# `canvas::forms` — filling a form **where it is drawn**

The operator's complaint that started this module is one sentence: *"today
fields can only be filled through the side panel, and every PDF reader lets
you click the field on the page and type."* This is the click, and the
typing.

It is a **second surface onto one implementation**, never a second
implementation. Everything below decides *which field the operator pointed
at* and *what they typed into it*; the moment either is settled it becomes
the same [`FormEdit`] the Forms panel raises, travels the same
`Action::Form` funnel, and reaches the same `EditSession` verb. There is no
fill path here. `grep FormEdit::` still answers "what can change a form?"
completely.

---

## 1. Why this is not a [`CanvasTool`] variant

[`crate::canvas::tool`]'s header sets the bar for admission — a candidate
must *"arrive with its own state"* and must be an answer to **what does the
primary button mean right now?** Form filling fails both, and the second
failure is the decisive one.

A form widget's `/Rect` is a small bounded region **the file itself marks as
interactive**, and its `/AP` already draws something that looks like an input
control, because that is what the document's author drew. There is no
ambiguity for a mode to resolve: a press inside that rectangle can only
sensibly mean *"fill this"*, and a press one point outside it can only mean
what a press on the page has always meant. Markup needed a variant because a
drag on blank paper is genuinely ambiguous between panning, marqueeing and
drawing. This is the opposite case.

`D:\Dev\pdfcer\docs\ui_specs\pass-7-form-fill.md` §1.1 reaches the same
conclusion and states the stronger half of the argument: gating filling
behind a mode *"would hide the primary reason most form documents exist"*.
`forms-panel.md` §1.2 records the same conclusion for this shell. And it is
what the operator's standing
tie-breaker — *"make it work the way other programs do"* — asks for: no
reader in the world has a form-fill mode.

So this lives in the ordinary [`CanvasTool::Select`] tool and is offered in
no other, which is one line ([`offered_in`]) rather than a variant.

---

## 2. The panel is not replaced, and could not be

[`crate::panels::forms`] stays exactly as it is, in exactly its role. Two
reasons, and the first is not a preference:

1. **Accessibility.** The panel's rows are real `egui` widgets in a real
   layout: they get tab order, they get AccessKit exposure, and their labels
   are `/TU` — the string a screen reader actually announces for an
   interactive field. What this module projects onto the canvas gets none of
   that, and cannot: the thing underneath it is a **page raster**, which is
   a picture with no text alternative. An operator who cannot see the page
   cannot discover that a field exists here, let alone which one it is.
2. **Everything this surface declines is still fillable there.** §5 lists
   four reasons a field is not offered on the page. Every one of them has
   the panel as its answer, and the panel says so
   ([`crate::text::forms::forms_canvas_undrawn_note`] and
   [`crate::text::forms::forms_canvas_unreachable_note`]).

The canvas is an **additional** way in. If it ever becomes the only one,
this build has lost a capability it currently has.

---

## 3. What the operator sees, and what it does not promise

**An unfocused widget is not painted at all.** Rule 4's one-line test is
*would a screenshot of the editing canvas differ from a screenshot of the
same document saved and reopened?* — so this module paints no highlight, no
outline and no tint over a field that is merely available. What it does
instead is change the **cursor** to an I-beam over a text field and a
pointing hand over a button, which rule 4 permits by name (*"a snap
indicator, a hover highlight, a rubber-band, a selection handle — these are
the cursor"*) and which costs the page nothing.

That makes the cursor the whole discovery mechanism, and it is worth saying
plainly that it is a weaker one than Acrobat's blue field tint. The honest
remedy is a **"highlight fillable fields" toggle**, which is a ribbon
command; this module deliberately adds none and the entry point is reported
rather than wired.

**A focused text field is replaced by an editor**, and the editor is
deliberately *not* a facsimile:

- It draws in the theme's own text-edit colours at a size derived from the
  widget's height on screen ([`editor_font_size`]), not from the field's
  `/DA`.
- **It does honour `/Q`** ([`boxes::editor_align`]) — see the below.
- It is never smaller than [`MIN_EDITOR`], so a 4 pt field at 25 % zoom is
  still something an operator can read what they typed in. It may therefore
  overhang the field it is editing. An editor that is honest about its
  position and unreadable is worse than one that is legible and a few points
  too big.

**The reason it cannot be a facsimile is not effort, it is arithmetic.**
The overlay is a font substitution by construction: the glyphs come from
egui's bundled UI font, the document's come from the field's `/DA` font, and
`pdfcer-core`'s variable-text generator refuses font substitution in variable
text *precisely because it changes glyph advances*. A box that pretended to
be the appearance stream would be making a fidelity claim it cannot keep —
the caret would drift from where the glyph will actually land, and it would
drift further the longer the string. So the editor says **"you are typing
here"** and the `/AP` regenerated on commit says **"and this is what it
looks like"**, one gesture later. That is the same bargain a spreadsheet
makes between the formula bar and the rendered cell.

### …and the exact reach of that argument, which is easy to over-read

The paragraph above is about **glyph advances** and nothing else. Read as
though it settled every appearance property it settles too much: the
argument is arithmetic, so it does not reach text alignment or the widget's
background colour, and treating those as decided by it leaves them an
unexamined gap wearing a decision's clothes.

⇒ **The test a property must pass to be honoured here is whether honouring
it makes a claim about where a particular glyph will land.** A font and a
size do — that is the whole substitution problem. `/Q` does not: it names
which end of the box the run is anchored to, and it names the same end
whatever font draws the run. So `/Q` is read, through
[`boxes::editor_align`], and the operator's text does not jump from the
left of a centred field to its middle at the moment they tab away.

The **background colour** half passes the same test, and the engine models
it: `pdfcer_core::forms::Widget` carries `/BG` and `/BC`, so this shell can
ask the question — and it answers it. `boxes::editor_fill` resolves `/MK`
`/BG` onto the cached
[`boxes::WidgetBox`], and the editor below pairs it through
`Theme::foreign_fill_pair` before handing the `TextEdit` a
`.background_color()` and a `.text_color()`. A tinted form box keeps its
tint while it is being edited, and the ink on it is chosen against that
tint rather than against the theme's.

Kept rather than deleted because the sentence was **correct when
written and correctly filed** — it is what the engine request was argued
from. What expired is the tense, and a claim whose history is erased
cannot be audited.

### What this surface cannot promise, stated rather than discovered

- **Glyph-accurate preview.** See above. The committed appearance is the
  truth; the editor is a place to type.
- **A caret where you clicked.** The first click is consumed by the page
  (§4), so the editor is created on the *next* frame and has no click to
  place a caret from. The caret goes to the end of the text, which is the
  least destructive place for it to be. A second click, once the editor
  exists, does place a caret — and a double-click selects a word — because
  by then the operator is interacting with a real `egui::TextEdit`.
- **Live agreement between two boxes of one field.** A field may have
  several widgets (§7). While one is being typed into, the others still
  show the `/AP` the document currently holds, and catch up on commit —
  because `/AP` is regenerated by the engine and there is no half-committed
  appearance to draw.

  **The Forms PANEL is deliberately not in this list.** A panel row
  showing the old value for as long as the operator keeps typing on the
  page reads as exactly the same lag — and is not. The sibling
  widget above shows an appearance nobody has generated yet; the panel row
  shows a `String` this module is holding. So the panel reads it
  ([`live_draft`]) rather than being told to expect a delay. The rule the
  pair states: **disclose a lag only when the value the surface would need
  does not exist.** Where it exists, a disclosure is an apology for a
  second draft store nobody had to keep.

---

## 4. Input layering — what claims a press, and what deliberately does not

[`crate::canvas::guides`]' header §3 is the precedent: a widget registered
**after** the page widgets is the topmost one under the pointer, so a press
on it never reaches the page's `Response` and therefore never reaches the
gesture machine. That is exactly what a focused field needs, and exactly
what an *unfocused* one must not have — an interactive rectangle sitting
over every fillable field would swallow marquees, would swallow the hover
that gates Ctrl+wheel zoom, and would do both silently.

So the two halves are asymmetric on purpose:

| | registers a widget? | consequence |
|---|---|---|
| an **unfocused** widget | **no** — the click is read from the page's own `Response` | pans, marquees, Ctrl+wheel and hover over the page are untouched |
| the **focused** text field | **yes** — a real [`egui::TextEdit`] | its drags select text, its double-clicks select words, and none of it reaches the gesture machine |

Reading the click off the page's `Response` rather than off a widget of our
own has one visible consequence and it is the right one: the click **also**
reaches the selection layer, so clicking a form field clears whatever vector
object was selected. That is what a click on "somewhere else" means
everywhere else in this canvas, and inventing an exception would be a rule
nobody could predict.

### The hit test takes no tolerance, and that is a decision

Every other hit test in `canvas/` passes [`PageMapping::tolerance`], because
[`crate::canvas::mapping`]'s header is about exactly one defect: *a screen
number used where a page number was meant*. This one passes nothing, and the
reason is that a tolerance answers a question a widget rect does not ask.

A tolerance exists so a **hairline** — a stroked path a fraction of a point
wide — can be hit at all. A form widget is an **area** the document
declares, typically 10–30 points tall, and form fields are routinely
*adjacent*: a table of boxes with a one-point gutter between them is the
ordinary shape of a real form. A 6-pixel catch radius there would make the
boundary between two fields ambiguous and would silently focus the neighbour
of the one the operator aimed at — a wrong answer, delivered confidently,
which is the failure mode `mapping`'s own header calls the worse one.

So [`hit`] is a plain containment test, and this paragraph exists so that
nobody "fixes" it by adding the tolerance every neighbouring call site uses.

---

## 5. Four reasons a field is not offered here — and all four keep the panel

[`NotOnCanvas`] is the complete list, and each entry is a fact the *file*
states rather than a limit this module chose:

1. **[`NotOnCanvas::NoAppearance`] — the widget has no `/AP` `/N`.** It is
   therefore drawn as *nothing*, and a click target over blank paper is an
   invisible affordance. The rule this module holds to is **the canvas
   offers exactly what the page draws**; anything else means either an
   unhittable target or a painted placeholder, and the "no placeholders"
   invariant forbids the second. The remedy already exists and is one
   button away: `FormEdit::RegenerateAppearances` draws every field's
   current value, after which the field is visible **and** clickable. The
   panel says so ([`crate::text::forms::forms_canvas_undrawn_note`]).
   `D:\Dev\pdfcer\fixtures\synthetic\forms\demo-form.pdf` carries exactly
   this case.
2. **[`NotOnCanvas::RotatedPage`] — the page's `/Rotate` is not 0.** The
   geometry is fine ([`widget_canvas_rect`] goes through the renderer's own
   transform, so the box lands in the right place at every rotation) — what
   breaks is the *editor*. `egui` cannot rotate a `TextEdit`, so on a
   `/Rotate 90` page the operator would type horizontally across text the
   `/AP` draws vertically. **Text fields only**: a check box has no text
   direction, so a button on a rotated page is offered exactly as it is
   anywhere else, and only the editor is withheld.
3. **[`NotOnCanvas::NotPlaced`] — no page's `/Annots` lists this widget
   with a usable rectangle.** One reason rather than two, and the merge is
   the point: [`boxes::place`] answers *"which page is this widget on?"* by
   walking each page's `/Annots`, so there is no `/P` to be absent and no
   second question to ask. A widget no page lists — or one whose listed
   rectangle has no area — has no place on the page whatever its own
   dictionary says. (§12.7.4.5 makes a zero-area `/Rect` *deliberate*
   invisibility for a signature field, which is not offered here anyway.)
4. **[`NotOnCanvas::NotOffered`] — this kind has no canvas gesture.**
   Read-only, signature and push-button fields (the panel's
   [`crate::panels::forms::rows::block_reason`], asked here rather than
   re-derived), rich text (which the panel offers a *conversion* rather
   than a box), a button with no on-state, and a choice field whose `/Opt`
   is empty.

   **A choice field with options is offered**, and that overturns what this
   section used to argue: that a page-anchored dropdown was "a second popup
   surface with its own placement rules and no gesture the panel does not
   already have". Both clauses were wrong. The gesture clause was wrong
   because a form's combo boxes and list boxes are on the sheet the
   operator is reading, so routing them to a panel is exactly the round
   trip §1 refuses for text — reading the option off the page and then
   hunting the same field in a list is the round trip, whatever widget sits
   at the far end of it. The placement clause was wrong because the rules
   turned out to be one rule, and [`choosing`]'s is all of it.

   Rule 4 is answered by what the popup does **not** draw: nothing at all
   over an unfocused field. A screenshot of the page with nothing focused
   is the screenshot of the saved document, which is the one-line test.

Two document-wide gates sit in front of all five, in [`offer`]:

- **`EditSession::fill_refusal`** — a certification signature forbids
  filling the whole document, so nothing is offered anywhere. Asked once per
  frame, exactly as the panel asks it once per frame (R83, know before you
  offer).
- **`OpenDoc::annotations_visible`** — with `view.show_annotations` off,
  `/Widget` appearances are not painted at all. Offering a click on a field
  the operator has asked not to see would be offering a click on nothing,
  and it is the same rule as reason 1 wearing the operator's hat instead of
  the document's.

---

## 6. Escape, and where it sits in the ladder

[`crate::canvas::keys`]' header ranks Escape's claimants. A focused field is
rung **0** — the most transient thing on the canvas, because it is the thing
the operator's hands are on — and it abandons the draft without writing
anything.

The exclusion is **mechanical rather than ordered**, which is worth being
precise about because it is the only rung that works that way. While this
module's `TextEdit` holds focus, `egui::Context::text_edit_focused` is true,
and that predicate already turns off *every* other claimant: `canvas_keys`
returns on it (`DEFECTS.md` D1's guard), and `interact` builds the gesture
machine's `cancel` flag from it. So no other claimant can even see the key.

What that leaves is the **other** direction: `egui`'s own `TextEdit`
surrenders focus on Escape, and it does so *before* `canvas_keys` runs. One
press would then abandon the draft **and** ascend a selection rung — the
exact double effect decision 025's L1 forbids. [`escape_spent`] is how this
module says it took the key, read by `canvas_keys` as claimant 0, in the
same report-rather-than-re-derive shape every other claimant uses.

---

## 7. The commit boundary, and one field with several widgets

**Focus-loss-and-changed, plus Enter** — and it is
[`crate::panels::forms::rows::commit`], the panel's own pure function,
called rather than restated. Both halves of its condition matter here for
the same reasons they matter there: `changed()` would make one typed word a
dozen undo entries and a dozen appearance regenerations, and an unchanged
field left without typing must write nothing at all. Enter needs no code —
`egui` surrenders a singleline `TextEdit`'s focus on Enter, so Enter *is*
focus loss; a multiline field keeps Enter for the newline `/Ff` `Multiline`
asks for.

`EditSession::fill_text_field` writes `/V` once and regenerates the `/AP` of
**every** widget of the field, as one undo entry, so the write is already
correct for a field presented in several places. What needs discipline is
the *draft*, and the discipline is the panel's: the draft is keyed by
`(path, edit_epoch)`, and an epoch change re-seeds it from the document
rather than surviving it. See [`Focus::sync`] for the one place this differs
from the panel and why.

---

## 8. Proving it from outside

The defects in this shell are found by running the program, not by the
suite. A hit test is invisible in a screenshot — a click
that focused the right field and a click that focused the field next to it
are the same picture at 100 % zoom. So every decision this module makes is
on the `PDFCER_DIAG` channel:

```text
form-boxes n=3 pages=1            # deduped: how many boxes exist at all
form-hit page=0 field=Name kind=text at=(126.4,203.1) rect=(120.0,198.0)+(200.0,14.0)
form-focus page=0 field=Name widget=0
form-commit field=Name chars=4
form-escape field=Name
form-button field=Agree state=Yes
```

The field **name** is on the line and the typed **value** never is. That is
the same split [`crate::panels::forms::edit::FormEdit::label`] makes and for
the same reason: a name is metadata a log needs in order to be useful, and a
value may be what an operator typed into a `/Ff` `Password` field, which
pdfcer has just warned them is stored in the clear.

## Item notes

### `fn sync`

# Where this differs from the panel, and why

[`crate::panels::forms::FormsUi`] keys its drafts on `(path, epoch)` and
**drops them all** when either moves. Doing that here would be wrong in
a case the panel does not have: clicking field B while field A is
focused commits A *and* focuses B in one frame, so the very next frame
carries a new epoch — and dropping the focus on it would put the caret
out of a field the operator had just clicked into.

So a path change discards, and an epoch change **re-seeds**: the focus
survives, the draft is replaced by what the document now holds. That is
the same correctness the panel's rule buys — after an undo the box shows
the reverted value rather than the typed one, so it cannot re-commit
what was just undone — with the one behaviour the panel does not need.

Nothing is lost by re-seeding for the same reason nothing is lost in the
panel: an epoch moves only when a document edit lands, every gesture
that can land one takes focus away first, and taking focus away commits.

### `const MAX_TRACED_BOXES`

Not a round number for its own sake: it is comfortably more than any real
form's *visible* field count and small enough that a truncated census is
still a few screens of trace rather than a wall. A form that exceeds it
says so on the next line rather than silently stopping, because a census
that ends without saying it ended is a census a harness would read as
complete.

### `fn editor`

Returns whether the editor claimed this frame's primary click, so
[`overlay`] does not also read the same press as a request to focus
something.

### `fn click`

The click is read from the **page's own `Response`** rather than from a
widget of this module's, which is the whole of the input-layering decision —
see the module header §4.

### `fn focus_button`

Without this a click on a check box would take the ring's place in the
form away from it: the page's own response is focusable, so the click
leaves egui's focus on the PAGE, `tabnav` finds no published owner, and
the next Tab walks the ribbon -- which is O204's complaint, reached from
the one gesture most likely to precede a Tab.

`seated: false` so [`tabbing::button_focus`] asks for the keyboard on the
next frame; `waiting: 0` because a box under the pointer is on screen, and
a frame that cannot draw it has genuinely lost it.

### `fn cursor`

**The whole of the discovery affordance**, and the only thing this module
puts on screen for a field that is not being edited — see the module header
§3 on why rule 4 permits a cursor and forbids a tint.

Set here, before
[`crate::canvas::interact`](crate::canvas::interact::interact) runs, so
[`crate::canvas::tool::cursor_for`] still has the last word: with the select
tool it has no opinion and this survives, and where it does have one — an
in-flight drag, a hovered resize grip — that opinion is about a gesture
already under way and outranks a hover.

### `struct Focus`

One draft, not a map: exactly one field on the canvas can hold focus, which
is the structural difference from [`crate::panels::forms::FormsUi`] — that
one draws every row every frame and so must keep a draft per row.

### `fn live_draft`

# Why this exists: two draft stores, one field

[`Focus::draft`] holds what is being typed on the canvas;
[`crate::panels::forms::FormsUi::drafts`] holds what is being typed in the
panel. The only thing that reconciles the two by itself is a **commit**,
which happens on focus loss — so without this function an operator filling
a field on the page watches the panel row beside it go on showing the old
value for as long as they keep typing: two boxes, side by side,
disagreeing about one field.

⇒ **The honest answer is to delete one of the two answers, not to disclose
the gap.** §3's list of what this surface cannot promise carries the
*sibling-widget* version of this lag, and that one is genuinely
undisclosable-away: a second widget of the same field draws the `/AP` the
document currently holds, and there is no half-committed appearance for it
to draw instead, because `/AP` is regenerated by the engine on commit. The
panel is not in that position at all. A panel row is an `egui::TextEdit`
over a `String`, and the `String` it should be showing is right here.

So the panel **reads this** rather than keeping a second opinion, which is
the same shape as [`placed`] one function below: one answer, two
surfaces, and no drift possible between them because there is nothing to
drift from.

# The `egui` focus check is the whole of the safety argument

A stored [`Focus`] is not by itself evidence that the operator is typing on
the page — it survives until the editor loses focus, and this function is
called from a panel that is drawn on every frame, including frames where
the canvas is not the thing being interacted with. Returning a draft in
that state would let the canvas's stale value overwrite the panel's live
one, which is the same disagreement wearing the other hat.

`ctx.memory(focused) == Some(editor_id)` is exact: `egui` has one focused
widget, so the predicate is true on precisely the frames the page's editor
owns the keyboard, and false on every frame the panel's own row does. It is
also what makes the commit path safe — the frame the editor loses focus is
a frame this returns `None`, so the panel is never mirroring a draft that
is on its way to becoming a document value.

# Why the `(path, epoch)` guard as well

The same reason [`Focus::sync`] carries it. A draft belonging to another
document is meaningless, and a draft taken at another revision describes a
value the operator has not seen since they typed it. The panel drops its
own drafts on both changes ([`crate::panels::forms::FormsUi::load`]), so
mirroring across either boundary would be handing the panel back the one
thing its key exists to throw away.

### `fn placed`

Held as an `Arc` so the per-frame read is a refcount bump rather than a
clone of every field name in the document.

The cache is keyed on `(path, edit_epoch)` and **not** on the zoom, the
scroll offset or the page: canvas space is invariant under all three, which
is the property that makes this cheap enough to consult on every frame in
order to set a cursor.

# `pub(crate)`, because the panel reads the same answer

[`crate::panels::forms::canvas_routing`] needs to know how many fields the
page cannot be clicked for, and the only correct source of that number is
the walk that decided it. Handing the panel this cache rather than letting
it repeat the walk buys three things: the count cannot disagree with the
behaviour, the form is parsed once per revision instead of twice per frame,
and `EditSession::widget_rects` is asked once per page per revision instead
of once per page per frame.

# Why the geometry comes from `widget_rects` and not from the form

`EditSession::widget_rects(page)` reports every `/Widget` **that page's
`/Annots` lists**, with corners already normalised and the session overlay
applied. See [`boxes::place`]'s section for why asking the pages is the
only correct direction and why asking the widgets' `/P` is a defect no
fixture in the corpus can catch.

### `fn escape_spent`

Read *and cleared*, so one press cannot be claimed twice. See the module
header §6 for why this rung is needed at all when `text_edit_focused`
already silences every other one.

### `fn overlay`

Called from [`crate::canvas::show_in`] immediately after
[`crate::canvas::guides::canvas_drag`] — the same place, the same layer and
for the same reason: everything registered here is later than every page
widget, so the focused editor is the topmost thing under the pointer and
the gesture machine never sees a press meant for it.

Nothing is mutated. Every outcome leaves as a [`FormEdit`] on `actions`,
which is the invariant the whole shell is built on and which this surface
honours by construction: `doc` arrives as a shared reference.

### `fn settle`

Reached when the tool changes, the annotations are hidden, the document is
certified between frames, or the focused field's page scrolls out of the
strip. **Committing rather than discarding** is the old spec's rule and it
is right: a half-drawn markup shape has nothing an operator would miss, and
a half-typed field value is something they typed on purpose.

### `fn commit`

[`crate::panels::forms::rows::commit`] is the rule — **the panel's own pure
function**, called rather than restated, so "tabbing through a field writes
nothing" is one statement with one test and not two.

### `fn stored_value`

Re-read from the session on every frame that needs it rather than carried
on [`Focus`], for the reason [`crate::panels::forms::rows`] re-reads it: the
stored value is the thing a draft is compared against, and a cached copy is
one more thing that can be stale at exactly the moment the comparison
decides whether to write.

### `fn keyboard_box`

Returns the `focusable_noninteractive` response every on-page field editor
is built on. `focusable_noninteractive` rather than a clickable sense
because sensing the press here would take it away from the page response
[`click`] reads, which is the input layering the module header §4 settles.

# Why the event filter is not optional

egui moves keyboard focus on a bare arrow key and *surrenders* it on
Escape, both inside `Memory::begin_pass`, before any of this module runs.
The default [`egui::EventFilter`] declines every lock, so a field that
reads Up/Down for its own purpose loses the ring on the same press it
acted on, and one that wants Escape to mean "close the list, keep the
ring" never gets a second press to interpret.

Both failures are **silent**: the field simply stops answering keys. The
only outside evidence is `form-choice-unfocused`, which exists for that
reason.

`tab` is deliberately left unlocked — Tab moves the field ring, and that
is `tabnav`'s job, not a key any one field consumes. Horizontal arrows are
left unlocked because no field editor reads them.

egui refuses to store a filter until the widget has held focus for a full
frame (`Memory::set_focus_lock_filter`), so this is called on every frame
rather than only on the one that requests focus.

### `fn ring_takes_space`

Asked by [`crate::canvas::tool::arm::space_held`]. The whole of the
argument is on [`crate::canvas::tabnav::owns_focus`]; this is the form
module's name for it, so that caller reads as a sentence about forms.
