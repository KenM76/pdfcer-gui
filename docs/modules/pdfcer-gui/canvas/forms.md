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
- **It does honour `/Q`** ([`boxes::editor_align`]) — see the ★ below.
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

### ★★★ …and the exact reach of that argument, which is easy to over-read

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

★ Kept rather than deleted because the sentence was **correct when
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

  ★★★ **The Forms PANEL is deliberately not in this list.** A panel row
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

### ★ The hit test takes no tolerance, and that is a decision

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
   turned out to be one rule, and [`choosing`]'s ★ is all of it.

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
