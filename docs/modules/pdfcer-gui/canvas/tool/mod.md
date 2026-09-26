# `canvas::tool` — which pointer tool the canvas is in, and the space bar that borrows it

## What this module is for

`GUI_ROADMAP.md` Phase 3.2: *"There is no hand tool at all; panning is
middle-drag only."* This is the hand tool, and the space bar that borrows
it for as long as it is held.

It owns exactly one question — **what does the primary button mean right
now?** — and answers it as a pure function of two inputs: the tool the
operator *chose* ([`selected`]) and whether the space bar is *down*
([`space_held`]). Everything else in `canvas/` reads [`active`] and
branches on the answer.

## ★ Why the space override is derived and never stored

The requirement is *"space held = temporary pan, releasing returns to the
previous tool"*, and the obvious implementation — remember the previous
tool on key-down, restore it on key-up — is the one that fails. It fails
in the ordinary way (an interrupted key-up: the window loses focus mid-pan,
the operator alt-tabs, a dialog steals the release) and the failure is
*sticky*: the canvas is left in a hand tool the operator never chose and
cannot leave except by choosing something else. Every application that has
ever shipped a modal space-pan has shipped that bug at least once.

So there is **no stored override and nothing to restore**. [`selected`] is
the only persistent value; the space bar is read fresh from
[`egui::InputState`] on every frame and composed with it by [`resolve`].
"Returning to the previous tool" is then not an action that can be missed —
it is what the next frame computes when the key is no longer down. A lost
key-up costs one frame of pan, not a stuck mode.

## ★ The text-field guard is not optional

Space is a *character*. A canvas that panned on any Space keypress would
pan while the operator typed a page number into the status bar's page box
or a value into the Properties panel. The guard is
[`egui::Context::text_edit_focused`] — the same predicate, for the same
reason, as `DEFECTS.md` D1's Delete-key fix, and deliberately **not**
`egui_wants_keyboard_input()`, which is true whenever *any* widget has
focus and would therefore disable space-pan after a single click on the
canvas (the canvas takes focus on click, which is exactly how D1 happened).

## ★★★ The bar a new variant has to clear

This is **not** a general "tool" enum with one member per authoring surface.
It answers a narrow question — **does a primary drag select, move the paper,
or draw?** — and a candidate is a *mode* that arms a whole surface until it
clears one bar: it must **arrive with its own state**, and it must need this
enum to hold no more than *which kind is armed*.

[`CanvasTool::Markup`] is the shape every later admission is measured
against. It arrives with [`markup::MarkupKind`], with a `DragKind` and a
`GestureOutcome` of its own in [`crate::canvas::gesture`], with a rubber
band, a commit path and an `Action` — and nothing of it outlives a frame
except **which kind is armed**, which is precisely one enum value and
exactly the kind of thing this module already stores. So the enum grows by
one variant *carrying* the kind, rather than by four.

Two rules keep the growth cheap, and both are enforced here rather than at
call sites: [`CanvasTool::pans_with_primary`] is the single predicate the
pan and gesture-suppression paths share, and [`CanvasTool::cursor`] is the
single place a tool's cursor is decided.

## ★ What each later variant had to bring

**[`CanvasTool::Measure`]** reads like a counter-example — a two-point pick
with a snap indicator and a live readout — and is not one: that machinery is
[`crate::canvas::measure::pick`]'s, and none of it has to be held here. What
crosses the boundary is one [`MeasureKind`], exactly as markup's one
[`markup::MarkupKind`] does.

**Text selection** ([`CanvasTool::Text`]) **clears the bar more cleanly than
either**, and the set it arrives with is:

| it arrives with | where |
|---|---|
| a selection type, with its own staleness rule | [`crate::canvas::textsel::TextSelection`] |
| a [`PressMeaning`](crate::canvas::gesture::PressMeaning) and a `DragKind` | [`crate::canvas::gesture::DragKind::TextSelect`] |
| a resolver — one pass producing the string, the canvas boxes and the page quads | `canvas::textsel::resolve`, reached through `drag` / `click` / `select_all` |
| a commit path, in the only sense it has one: three markup kinds whose operand is the selection | [`crate::canvas::markup::text`] |
| two keyboard verbs of its own | [`crate::canvas::textsel::clipboard`] |

…and **the only thing it needs to persist is that it is armed.** Not a range,
not a caret, not an anchor: the range lives on the document beside the object
selection, the anchor is re-derived from the press origin on every frame of a
sweep (`textsel::drag`'s own header says why that is exact rather than lazy),
and there is no caret at all (`textsel` §1.2 — a caret promises an insertion
point, and there is nothing to insert). So the variant carries **nothing**,
where `Markup` and `Measure` each carry a kind. That is the smallest thing
this enum can be asked to hold and still be worth holding.

What the variant *buys* is two things at once. Without it, `canvas::textsel`
§3 gives a press its text meaning *"when the select tool is active and the
mode cannot select content"*, which yields Read ✓, Review ✓, **Edit ✗** — a
reviewer can sweep text and an editor cannot, and, worse, the three
text-markup controls drawn on Edit's Markup tab can **never enable**, because
`selection.text` is never true there. That is a live tension with
`RIBBON_IA.md` P3, which reserves greying for *temporarily* unavailable, and
it cannot be closed by hiding the controls, because a command lives on
exactly one tab and the Markup tab is in both Review and Edit. One variant
closes both.

### ★ The reference applications DISAGREE here, and Inkscape wins

The standing instruction is to match Inkscape, Acrobat and SolidWorks, and
to say which won where they disagree. On this question they genuinely do:

* **Acrobat and SolidWorks resolve text-versus-object *contextually*, within
  one tool** — hover text, get an I-beam; hover an object, get an arrow.
* **Inkscape uses a separate Text tool**, distinct from its Selector.

**Inkscape wins, and the reason is not a head-count.** An object marquee over
*vector content* is a surface Acrobat does not have at all: its "objects" are
annotations and form fields, never the page's own path and text operators, so
its contextual answer is not an answer to this conflict — it never has the
conflict. The conflict exists only in the Inkscape-shaped mode, which is what
makes Inkscape's resolution the applicable one rather than merely the
outvoted one.

The concrete failure a contextual press would produce is the deciding
argument. In Edit the primary drag is the content marquee, and the commonest
gesture on a drawing sheet is a marquee over a *region* — which on any real
sheet contains text. Under a contextual rule that drag would mean "sweep
text" or "marquee objects" depending on whether the pixel under the button-
down happened to be inside a glyph's box, a distinction the operator cannot
see and cannot aim at. A tool makes the answer a thing they chose.

### Text EDITING is a separate admission, and it made the argument again

Selecting text and editing text are different features with different state.
The second is [`CanvasTool::TextEdit`], and it was admitted against this same
bar and against a sharper objection — that a caret in a re-laid-out box would
drag a whole subsystem's state through this type. That variant's own
documentation is where the argument is made and where the objection is
answered; it carries one [`TextEditKind`], and the draft, the caret, the
anchor and the original text live in `egui::Memory`. Whoever brings the next
authoring surface should have to make the argument again, in this file, in
the same place.

## Where the state lives, and why `egui::Memory` is right here when it was
wrong for the selection

`canvas/mod.rs`'s seam 1 records the selection being *moved out* of
`egui::Memory` because it is **document-scoped**: closing a document must
forget it, and `Memory` outlives documents. A tool is the opposite — it is
**application-scoped**, like the ribbon tab or the theme. An operator who
picks the hand tool, opens another drawing and finds themselves back in the
select tool would report that as a bug. So the tool stays in `Memory`
precisely *because* `Memory` outlives documents, which is the property that
disqualified it for the selection.
