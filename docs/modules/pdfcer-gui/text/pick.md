# `text::pick` — every word the selection filter says

The strings for [`crate::canvas::pick`] and for the status-bar popup that
drives it. That module's header carries the design and the invariants; this
file carries the copy.

## ★ The vocabulary is the OPERATOR's, not the PDF specification's

This is the rule the whole file follows, and it is worth stating because
every row here has a perfectly good technical name that would be the wrong
label.

| this file says | the specification calls it | why the operator's word wins |
|---|---|---|
| **Lines** | path object | It is the word he used when he asked for the feature — *"text, points, lines, etc"*. On a CAD sheet, path objects **are** the line work |
| **Points** | anchor, on a subpath | Likewise his word. "Anchor" is Illustrator's, "node" is Inkscape's, "vertex" is CAD's, and "point" is what he says |
| **Blocks** | form XObject | ★ See below — this is the most load-bearing choice in the file |
| **Pictures** | image XObject / inline image | Already the shell's word elsewhere: *"Select a picture and drag it"* |
| **Characters** | the text-selection sweep | Names the unit, which is what distinguishes it from the Text row |

### ★★ Why a form XObject is called a "Block"

A form XObject is an entire nested drawing that the page treats as one
opaque object. There is no everyday English word for that — and there is a
perfect **CAD** word for it, which is the vocabulary this operator actually
has: a *block*. An AutoCAD block is precisely this. One insert, one
selectable thing, a hundred visible marks inside it.

The two alternatives were both worse:

- *"Groups"* collides with **layers** (optional content groups), which this
  shell has a whole panel for. Two different things called groups, one
  panel apart.
- *"Form XObjects"* is correct, is what the file contains, and is
  meaningless to anybody who has not read the specification. A filter row
  nobody can decode is a row nobody switches.

This is the *"use the conventional interaction, never invent one"* rule
applied to a noun rather than to a gesture: the product class the operator
comes from already named this thing, and borrowing that name costs nothing
and teaches nothing new.

## Why "Dimensions" is not a Rule 15 violation

Rule 15 forbids a bare *"dimension"* in code, comments, commits and specs,
because **ce dimensions** (the ones pdfcer authors) and **pdf dimensions**
(CAD-exported page content pdfcer must not alter) have opposite properties.

The label below is deliberately bare anyway, and the reason is that the
ambiguity does not exist on this surface. The row filters `AnnotKind::
CeDimension` — annotations pdfcer itself wrote. A pdf dimension exported by
a CAD package is page content and is picked by the **Lines** and **Text**
rows like any other ink; it can never arrive at this row. The operator has
exactly one kind of thing to think about here, and the shell already says
"Dimension" and "Dimension groups" to him elsewhere. A label reading "ce
dimensions" would introduce a distinction on the one surface where it
cannot apply.

The doc comments below stay precise; the labels stay short.

## Two rules inherited from `text::tool`, and they hold here too

**Every sentence states a fact, never a tip.** *"Clicks pass through text"*
is a statement. *"Try switching text off to reach the drawing underneath!"*
is a tip, and there are none here.

**A tooltip says what switching the row OFF does**, not what the class is.
The label already names the class; the operator hovering it is asking what
the control does, and for a subtractive filter the interesting direction is
always off.
