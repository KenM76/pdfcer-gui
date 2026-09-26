# `pdfcer-gui/shell/manifest/markup`

The **Markup** tab — *what am I adding for someone else to read?*

`RIBBON_IA.md` §5.5. Five groups: Shapes, Text markup, Notes, Style,
Comments.

# Why it is not called Review

What lives here is markup *authoring* — shapes, notes, stamps.
"Review" promises a review *workflow*: compare revisions, resolve
comments, track changes. pdfcer does not have that yet, and when it does
it will want the name. `Markup` is also the term this project's
audience uses; Bluebeam and every drafting office call it that.


Note that the *mode* called Review is a different thing and the
collision is deliberate rather than accidental: Review mode is the
stance in which a reviewer works, and this tab is one of the five it
contains.

# The Style group sets the style of the *next* markup

Not of the selected one. Changing an existing markup's style happens on
the contextual Format tab, and `RIBBON_IA.md` §5.5 is explicit that
both surfaces must exist — *"today only the first does, which is why a
placed markup feels final."*

Colour is the only style property with a control today, and it is not a
button: it is a swatch that opens a colour picker. That is what
`egui_shell::manifest::Item::Custom` is for. The shell reserves the
space and hands the `kind` back; it draws nothing and interprets
nothing. Modelling the swatch as a `Command` would have meant either
lying about what the control is or growing the framework a
`ColourSwatch` item variant, which is the road by which a reusable
shell stops being reusable.

Line width, fill and opacity are **N** and join the swatch when they
exist.

# ★ ONE of ten markup kinds is missing, and it is the one that matters most


## The count, and what each correction to it taught


> **the count that mattered was never the count of kinds, it was the count of
> gestures.**

* **Underline, StrikeOut and Squiggly** left first. Their blocker was never
  the engine — all three have authored appearance streams since Pass 6.1 —
  but the *gesture*: they mark text, and this shell had no way to select any.
  `canvas::textsel` closed that and `canvas::markup::text` spent it.
* **Ink, Polygon and PolyLine** left second, later the same day, and their
  blocker was the same word for a different reason: they were *"engine-ready
  but not drag-shaped"*, so the two-point rubber band could not express them.
  `canvas::markup::vertex` gave the two click-shaped kinds a gesture with two
  endings, and `canvas::markup::ink` gave the freehand one a trail. Again no
  engine change; again a gesture.

What is left after both is the one kind whose blocker really *is* the
engine — which is the distinction this section existed to make and had been
obscuring by counting all four together.

`RIBBON_IA.md` §5.5 lists `Line` among the four **G** shapes. The
shipped build has four markup kinds — Rectangle, Ellipse, `Arrow line`
and `Highlight band` — and no plain line: the parenthetical *"(Arrow is
`Arrow line`)"* resolves which of the two the existing control is, and
the answer is the arrow. A plain line is therefore **N** and is in
PLANNED rather than emitted, which is the conservative reading and the
one P3 requires: a button that arms a tool that does not exist is
exactly the placeholder the rule forbids.
