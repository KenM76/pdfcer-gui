# `dialogs::textannot` — the words half of a text-bearing annotation

The second half of the place-then-type gesture. The canvas has taken a
rectangle (or a point); this asks what goes in it, and **nothing reaches
the document until Accept.**

## Why a dialog, when markup authors on release

`crate::dialogs`' header draws the line: *"a dialog is a single transaction
with a start and an end… a panel is somewhere an operator dips in and out
of while working."* Typing a callout is unmistakably the first — it begins
when the box is drawn, it ends when the words are accepted or abandoned,
and there is nothing to dip back into afterwards.

The alternative — an in-place editor drawn over the page, the way a word
processor would — was rejected on the standing rule that **nothing floats
over the canvas** except the Find bar, which is a documented exception the
operator granted for one surface. It would also have needed a caret, a
selection and a hit test over text this shell does not own, which is a text
editor rather than a dialog.

## It is deliberately NOT modal to the document

The reference line stays drawn, the page stays where it was, and the dialog
is `default_pos` rather than anchored so it can be dragged aside. An
operator writing a callout is usually looking at the thing they are calling
out, and a window pinned over it would make them close the window to read
what they were annotating.

## The three kinds meet three different questions

| kind | what this asks |
|---|---|
| text box | *what should it say?* — a multi-line field, because a callout wraps |
| sticky note | *what is the note?* — the same field; the words live in a popup rather than on the page, and the window says so |
| stamp | *which stamp?* — a gallery, and **no text field at all** |

The stamp's absence of a field is the important one. `manifest/markup.rs`
recorded the blocker as *"a stamp with no chooser has no operand"*, and the
converse is just as true: a stamp with a free-text field is a text box with
a border, and offering both would be two controls for one feature with no
way for an operator to tell which they wanted.

## Item notes

### `const FOCUS_ATTEMPT_FRAMES`

The retry exists because the dialog's first frame races the pointer release
that opened it (see [`TextAnnotDialog::field`]); it is bounded because
asking forever would take focus back from Cancel and from the stamp gallery,
and a window that cannot be dismissed is worse than one that cannot be typed
into.

Eight frames is a shade over a tenth of a second at 60 Hz — longer than any
number of frames a release takes to resolve, and far shorter than a human
noticing the window and reaching for the mouse. Nothing here depends on the
exact value; it only has to sit inside that gap, which is two orders of
magnitude wide.

### `const WINDOW_PTS`

Wide enough for a four-line callout at the body text size without the field
wrapping every sentence, and short enough that the window reads as a
question rather than as a second document. The stamp gallery is the taller
of the two bodies and fits inside it.

### `const STICKY_EXTRA_PTS`

# A constant added to [`WINDOW_PTS`], not a size measured from the body

`print/layout.rs`' rule and `Host::fit`'s: **a size measured from the
content it sizes is R128**, and this project has met that three times. So
the number is derived from what is being added — seven rows at roughly the
row height plus a heading and a wrapped small line — and stated as a
constant that can be read and argued with, rather than queried from a `Ui`
that is being laid out inside the window this decides the height of.

⚠ It is deliberately generous. Over-tall costs the operator nothing on a
dialog they can resize and drag; under-tall costs them the Accept button.

### `const CUSTOM_STAMP_ROW_PTS`

⚠ That is a measurement of a ROW's pitch, taken once, from a trace — not a
size queried from the `Ui` this function helps to size. The distinction is
R128's: a constant a reader can argue with, versus a feedback loop.

### `const CUSTOM_DYNAMIC_NOTE_PTS`

The sentence appears the moment he selects a dynamic stamp. Counted up
front rather than when it appears, because a window that grows on a click
moves the control that was clicked, which is a worse behaviour than being
22 pt taller than it strictly needs.

### `const CUSTOM_EXTRA_MAX_PTS`

Roughly eleven rows. Past that the body scrolls, which is what a scroll
area is for.

A cap is needed even though [`window_size`] already clamps to the
application window. Without one a collection of forty stamps opens a dialog
the full height of the window it belongs to, standing over the drawing he
is annotating — the exact thing this module's header argues against for the
dialog's POSITION. Scrolling for the fortieth stamp is a smaller cost than
losing sight of the sheet for all forty.

### `fn custom_extra_pts`

`Pass O172` added the custom half to the stamp gallery and did not change
the window's height. The first driven run of
`custom_stamp_reaches_the_page` captured the result: a dialog showing the
seven standard stamps and the Add/Cancel row, with **none** of the
operator's three stamps on the screen. They were laid out below the
scrolled body's fold, published as rectangles in the scrolled content, and
invisible.

It was not a clipping bug and it was not a layout bug. [`window_size`] adds
a per-kind constant to [`WINDOW_PTS`] and the stamp's constant was written
for a body that did not yet have this section in it. **A guessed size is a
claim about the content, and the content changed under the claim.**

# Why counting the library is NOT the R128 feedback loop

The rule this module states three times — *a size measured from the content
it sizes is R128* — is about querying a `Ui` that is being laid out inside
the window whose size is being decided. This function queries no `Ui`. It
reads two integers off a [`Library`] that was scanned off the disk when the
dialog opened, before any layout ran, and multiplies them by constants
stated above. The result is the same kind of number as
[`STICKY_EXTRA_PTS`] — derived from what is being added, and arguable by a
reader — except that "what is being added" is data rather than a fixed list
of seven radios.

⇒ There is no loop, because nothing about the laid-out window can change
the count of files in his stamps folder.

### `const MIN_WINDOW_PTS`

The same floor handed to `Host` as its `min_size`, read from one constant so
the two cannot disagree. A dialog squeezed below the size it refuses to be
dragged to would be a window in a state the operator could not return it to.

### `fn window_size`

Derived from [`WINDOW_PTS`], [`MIN_WINDOW_PTS`] and the *outer* rectangle —
never from anything the body lays out. That is `print/layout.rs`'s rule and
`Host::fit`'s: a size measured from the content it sizes is R128, which this
project has met three times.

### `fn opening_position`

Centred horizontally and a **third** of the way down, not half — the same
placement the Set-scale dialog uses, and for the same reason: a window
centred vertically sits exactly over the middle of the page, which on a
drawing sheet is where the content is.

# This is not, and must not become, a click-relative position

The review that found A16c described the discarded computation as
*"click-relative"*. It never was, and making it so would contradict this
module's own header: *"an operator writing a callout is usually looking at
the thing they are calling out, and a window pinned over it would make them
close the window to read what they were annotating."* A dialog that opened
on top of the note would be a worse answer than the corner, not a better
one. What A16c is about is the dialog reaching **the position it computed**
instead of the corner.

The clamp onto the application window lives in
`dialogs::host::placement`, not here. This function's job is to say where
the dialog belongs; keeping it free of edge cases is what lets it be a
three-line expression that can be read at a glance and tested without a
window.

### `fn restored_kind`

Four answers, not three, and the fourth is the one worth having:

| Word | Means |
|---|---|
| `default` | nothing was remembered -- first stamp of the session |
| `standard` | a remembered standard face was re-selected |
| `custom` | one of the operator's own stamps was found again and re-selected |
| `gone` | a custom stamp WAS remembered and is no longer in the folder |

`gone` exists because the operator is told nothing when it happens, and
that silence is deliberate: to him, *"the collection I deleted"* and *"the
first stamp of this session"* are the same situation, so a sentence about it
would be noise about his own housekeeping. But the two are NOT the same
thing to a harness -- one is a fallback, the other is a fresh start -- and
collapsing them into `standard` would have made a check that watches a
deleted collection indistinguishable from one whose memory never arrived.

⚠ Do not route this word to the operator. It is a diagnostic, and R8b
rule 4 keeps a disclosure off-canvas AND out of his way when there is
nothing he can act on.

### `fn icons`

Radios over a combo box, matching [`Self::gallery`] one function down
and for its stated reason: seven entries is a set an operator reads at a
glance, and a combo would hide six of them behind a click for no saving.
Using the *same* control for the two choosers is deliberate — they are
the same act (*pick one of seven*) and a window that answered it two
ways would be teaching the operator a distinction that does not exist.

The disclosure under it is not optional. The engine's sticky author
passes the icon to `/Name` and **nowhere else** — the marker artwork is
the same dog-eared page glyph for all seven, by the trade-dress
decision `annot_author` records as *R44 choice (a)*: the spec supplies
no icon artwork, so pdfcer authors its own plain marker and never a
reproduction of Acrobat's set. So an operator who picks *Key* sees no change
here and a key in another reader, and
[`crate::text::textannot::sticky_icon_bound`] is where they are told
that before it happens rather than after.

### `fn sizes`

# A combo, where the two galleries beside it are radios

[`Self::icons`] argues for radios and gives the reason — *seven entries
is a set an operator reads at a glance* — and then says using the same
control for both choosers is deliberate because *"they are the same act
(pick one of seven)"*. This is a different act, and the difference is
the point rather than an exception to that rule:


# What the disclosure under it is, and what it is NOT

[`crate::text::textannot::stamp_size_bound`] tells the operator the box
will widen if the words need it. That is **not** an R8b rule 4
disclosure of an inference — a grown box is visible on the canvas as
itself, and a screenshot of it matches a screenshot of the saved file.
It is told before the act, so the drag means what the operator thinks
it means.

### `fn select_standard`

# Why this is a method and not two lines at the call site

Because the gallery's selection spans **two fields**, and the invariant
*"exactly one of them is the selection"* is the only thing that stops a
click on `Approved` from placing the operator's signature. `radio_value`
cannot express it — it writes one variable and knows nothing about the
other — so the two writes have to travel together, and a pair of writes
that must travel together is a function.

⇒ The payoff is that the invariant becomes **testable without a
window**. Held at two inline call sites it could only be checked by
laying out a frame and synthesising a click, which is a driven check's
job and not a unit test's; held here it is two calls and an assertion.

### `fn select_custom`

[`Self::stamp`] is deliberately left alone rather than reset. It is
not read on this route — `app::actions::apply` forks on `custom` before
it looks at anything else — so clearing it would buy nothing, and it
means a click back onto a standard stamp restores the one he had
chosen before rather than snapping to `Approved`.

### `fn gallery`

# Why `ui.radio(..).clicked()` and not `ui.radio_value(..)`

Because the selection spans **two** fields. `radio_value` writes one
variable and knows nothing about the other, so a click on `Approved`
would set [`Self::stamp`] and leave [`Self::custom`] holding his
signature — and the commit path reads `custom` first, so the operator
would have picked `Approved` and got his signature. Silent, and
reproducible on the first click of a second choice.

⇒ Each arm therefore writes both halves. The invariant *"exactly one of
the two is the selection"* is held here, at two call sites, and asserted
in this module's tests because two call sites is not a type.

### `fn custom_stamps`

Renders **nothing at all** when he has none: no heading, no empty
group, no *"you can add your own"* invitation. R9. A machine with no
Acrobat, or with an Acrobat nobody has ever made a stamp in, gets the
seven standard entries and no evidence that a second half exists.

# Returns

**Whether anything was drawn.** Not used by the caller, and that is the
point: it exists so R9 can be asserted by a unit test rather than only
by a driven check. [`crate::diag::ui_rect`] is write-only and silent
unless the diagnostic channel is on, so *"the region was not emitted"*
is not a question a test in this crate can ask — and *"the operator
sees no custom half"* is exactly the claim R9 makes. A returned `bool`
is the smallest thing that makes the claim checkable in-process.

⚠ It is a claim about this function only. That the *gallery* calls it,
and that a real folder produces a real list, are separate claims and
the driven check owes both.
