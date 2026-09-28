# `dialogs::settings::widgets` — the shapes every setting is made of

Every setting on every page is built from these functions. A settings window
whose entries are hand-laid-out drifts into slightly different layouts, and
the reader notices the inconsistency before the content. Because every
option name passes through them, they are also what the search reads.

## [`header`]'s signature is where obligation 2 and 3 are enforced

`crate::dialogs::settings`' header names three things this window must show
that a conventional settings screen omits. Two of them are properties of
*every* setting, and rather than trusting each group module to remember
them, they are **required arguments**:

```text
header(ui, title, silence, radius)
              │       │       └── which way costs what
              │       └────────── what the standard leaves open
              └────────────────── what the setting is
```

A setting cannot be added without answering all three, because the code
does not compile otherwise. `crate::text::settings` mirrors the shape —
`*_title`, `*_silence`, `*_radius` for all thirteen — and its own tests
assert none of the answers is empty.

Obligation 1 — *whether the default is a guess* — is **not** enforceable
this way: it belongs to one option rather than to the setting, and only
some options have anything to say. It is pinned by a test over the catalog
instead.

## Item notes

### `fn set_query`, `fn collect`, and the private `fn name`

Every title and option label goes through `name`, which does two things.
While [`collect`] runs it records the name, lower-cased; `nav` builds the
search index this way, by drawing each page once invisibly. And when the
name contains the frame's search (set by [`set_query`]) it is marked:
underlined and in the palette's `notice` colour.

**Not `.strong()`** — `DEFECTS.md` D11. egui has no separate role for
emphasised text: `strong_text_color()` returns `widgets.active.fg_stroke`,
the foreground of the accent-filled state, so on an ordinary panel in any of
this project's themes `.strong()` renders pale on pale. The titles are plain
text for the same reason; `tools/gates/check-strong-text.sh` refuses a bare
`.strong()`.

The collector and the query are thread-locals rather than arguments because
the pages call these functions a hundred times and none of those calls
should have to carry the search.

### `fn header`

Always in this order, and the order is the argument the window makes. The
operator reads *what this is*, then *why they are being asked* — which is
the sentence that stops a pdfcer/Acrobat difference being read as a bug —
and then *what changing it will do to their file*, which is the one they
need before touching a radio rather than after.

`.small().weak()` for the second and third: they are context for the choice
rather than the choice, and at the same weight as the title they would make
every setting look like three settings.

The title is **plain text**, not `.strong()` — see [`collect`] above and
`DEFECTS.md` D11. Being the only
one of the three lines that is not small and weak is the whole of its
emphasis, and it is enough.

### `fn option`

# Why the note is an `Option`

A few labels are self-describing — *"Carriage return then newline"* needs
no gloss — and padding them out to match their neighbours would be noise.
The rule this window inherits about tooltips applies one layer down: text
that says nothing trains the reader to stop reading the text that does.

Exactly two of the thirteen settings' options pass `None`, and both are in
the *Saving files* group where the label names a byte sequence.

### `fn toggle`

# The fourth shape, and why a two-option radio group was refused

This module's header opens *"the three shapes every setting is made of"*,
and a fourth arriving needs a better reason than convenience. It has one: a
**switch is not a choice between named alternatives**.

[`option`] draws a radio, which is the right control when the operator is
picking one of several *named things* — `Nearest sample`, `Average the
area` — and the names carry the content of the choice. A visibility toggle
has no such names. Rendering it as a radio group would mean inventing the
pair *"Shown" / "Hidden"*, which says nothing the checkbox's own label does
not, and it would draw **six** controls for the three overlays where three
belong. Worse, three adjacent two-radio groups read as though the six were
somehow related — a reader scanning them has to work out that they are three
independent switches and not one six-way choice.

# Why the label is on the checkbox rather than in a [`header`]

Because these are the sub-parts of **one** setting rather than settings in
their own right. The Drawing-the-page group's overlay control has a single
header — one title, one silence line, one radius line — and three switches
under it, because the three interlock: a guide is dragged out of a ruler, so
switching guides on without rulers places nothing. Giving each its own
header would print that explanation three times, or once, in a place two of
the three readers would not look.

`note` is `Option` for the same reason it is on [`option`]: a label that
needs no gloss should not get a padded one, because text that says nothing
trains the reader to stop reading the text that does.

### `fn text_value`

> *"can the size of the buffer be increased? Allow the user to set the size
> up to the maximum possible?"*

# Why it does not validate as you type, and does not refuse

`parse` is run on every frame and its result is **shown**, not imposed. A
field that rejected keystrokes would make `2` untypeable on the way to
`256mib`, and one that reverted on blur would silently discard what was
typed. So:

* **parses** → the value is written to the draft and the parsed form is
  echoed back (`= 256 MiB`), which is how an operator learns that `0.25gb`
  and `256mb` are the same number here;
* **does not parse** → the draft is left ALONE and the field says so. The
  last good value stands, so Apply cannot commit a half-typed string.

There is no upper bound and that is the operator's ruling, the same one
that governs the maximum zoom. A ceiling the machine cannot honour is not a
crash — the engine allocates fallibly and refuses down its ordinary disclosed
path — so this states the cost and does not prevent the choice.

`format` and `parse` are the CALLER's, and in the one use they are
`pdfcer_core::settings::format_byte_size` / `parse_byte_size` — the same pair
`settings.txt` itself uses. That is the point: the window and the file accept
and show identical strings, so an operator who reads one and types into the
other is never surprised. Writing a second parser here would have been the
obvious shortcut and the one thing guaranteed to drift.

### `fn disclosure`

`.small()` and deliberately **not** `.weak()`, which is the whole point of
its existing separately from [`option`]'s note. There are exactly three of
these in the window and each is a disclosure rather than a description:

- the CMYK intent group's *"pdfcer's default deliberately differs from
  Acrobat here"*, which the person reading that radio group is precisely the
  person who needs;
- the replacement-text group's bound, which applies **whichever option is
  chosen** and would be misread as an argument for one of them if it sat
  inside a note;
- the unknown-theme sentence, which explains why none of the three radios is
  selected.

Weak-grey is for context. A disclosure that pdfcer owes the operator is not
context, and greying it would be the quiet version of not saying it.
