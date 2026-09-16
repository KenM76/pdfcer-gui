# pdfcer — user manual

For the portable build. Unzip it anywhere, run **`pdfcer-gui.exe`**. There is no
installer and nothing is written outside the folder you unzipped, except the
settings described under [Where your settings live](#where-your-settings-live).

---

## Contents

1. [The first five minutes](#the-first-five-minutes)
2. [Read, Review, Edit — the three modes](#read-review-edit--the-three-modes)
3. [Moving around a drawing](#moving-around-a-drawing)
4. [The ribbon, and what happens when you make the window narrow](#the-ribbon)
5. [Selecting and changing things](#selecting-and-changing-things)
6. [Measuring](#measuring)
7. [Marking up for someone else](#marking-up-for-someone-else)
8. [Pages](#pages)
9. [Text](#text)
10. [Finding text — and when "no matches" does not mean "not there"](#finding-text)
11. [Redaction](#redaction)
12. [Forms — filling one in, and making one](#forms)
13. [Printing](#printing)
14. [Settings, and why pdfcer asks you things other viewers do not](#settings)
15. [Every keyboard shortcut](#every-keyboard-shortcut)
16. [Where your settings live](#where-your-settings-live)
17. [When something goes wrong](#when-something-goes-wrong)

---

## The first five minutes

Open a PDF by dragging it onto the window, or **Ctrl+O**. An encrypted document
asks for its password first, in a window of its own.

The document appears in the middle. Around it:

| | |
|---|---|
| **top strip** | the quick buttons (open, save a copy, undo, redo), then the tabs, then **Read / Review / Edit** on the right |
| **the ribbon** | the band of commands under the tabs. What is on it depends on the tab *and* on which of the three modes you are in |
| **left panel** | page thumbnails, bookmarks, and whatever else you switch it to |
| **bottom strip** | the **Select** filter, the page number, zoom, the fit buttons, find |

**You can close every panel.** The document is the point; the chrome is not.
Drag a panel by its tab to move it, or use **View ▸ Panels** to switch them on
and off. **View ▸ Reset layout** puts everything back.

---

## Read, Review, Edit — the three modes

The control at the top right. It is not a toolbar filter — it changes what the
program will *let* you do.

| mode | for | what you get |
|---|---|---|
| **Read** | looking at a drawing | File and View only. Nothing can be changed by accident |
| **Review** | commenting on someone else's work | adds Pages, Markup and Measure |
| **Edit** | changing the document | everything |

**Ctrl+1**, **Ctrl+2**, **Ctrl+3**.

Each mode's tabs are a superset of the one before, so moving up never takes
something away. If a command you expect is missing, you are probably a mode too
low.

A command a mode does not allow is **not drawn at all**, rather than drawn
greyed. Greying is kept for something that is unavailable right now for a reason
that will pass — no selection yet, no printer answering.

Filling in a form is deliberately outside all of this: it works in Read as well
as in Edit, because that is what a reader is for.

### Read *mode* and read *view* are different things

The selector above changes what you are **allowed to do**. **Ctrl+H** changes
what is **on screen**: it hides the ribbon and the side panels so the drawing
has the whole window.

While it is on, the title bar says how to get out — `Read mode — Ctrl+H to
exit` — and so does the bar along the bottom. That is deliberate: the control
that turns it off lives on the ribbon, and the ribbon is the thing it just hid.

**F11** is full screen and is separate; its button stays on screen, so it needs
no such note.

### Making the chrome get out of the way on its own

**Settings ▸ Display** offers to auto-hide the **ribbon** and the **left rail**.
Both are off unless you turn them on, and both work the way Word's *Show Tabs*
does:

- the row of tab names never disappears — **the thing you have to point at is
  always there**;
- the buttons appear **over** the drawing rather than pushing it down, so
  nothing you were about to click moves;
- the rail leaves a narrow marked edge, and the panel beside it does not change
  width.

---

## Moving around a drawing

**Pan** by dragging with the Hand tool (**H**), or with the middle mouse button
at any time.

**Zoom** with **Ctrl + mouse wheel**, with **Ctrl+=** and **Ctrl+-**, or from the
zoom box at the bottom. The plain wheel turns pages.

The four fit buttons at the bottom: **actual size**, **fit page**, **fit
width**, **fit height**. All four also centre the view — pressing fit width
puts the page in the middle of the window, not off to one side. A fit stays in
force while you pan, and survives a window resize with the same part of the
drawing still centred.

### How far it zooms, and how to change the ceiling

**pdfcer zooms much further than most viewers**, and the detail is really there
rather than a blur of a big picture. It ships at the top of its range.

**Click the zoom percentage in the bottom bar** to set the maximum. The popup
lists the rungs, has buttons to step between them, and accepts anything from
10 % upward. There is no warning attached to it: how much of a performance cost
a deep zoom is worth is your decision.

The one fact worth knowing when you choose: **below about 1000 % pdfcer draws
the whole page, so panning is instant; above it, only what is on screen is
drawn and panning redraws.**

The zoom in and zoom out controls step a fixed ladder of familiar percentages,
so zooming in and back out returns you exactly where you started.

### Rulers, grid and guides

Three switches in the **View** tab's Display group, all off to begin with:

| switch | what it does |
|---|---|
| **Rulers** | a measuring strip down the top and left edges, reading in points — or in this document's own units if you have set a measurement scale. The strip comes out of the space the drawing gets |
| **Grid** | a drafting grid over each page, spaced to match the rulers. It belongs to the sheet and scrolls with it, is never part of the document, and never prints |
| **Guides** | shows the guide lines placed on this document's pages and lets them be moved |

**A guide is dragged out of a ruler, so switch the rulers on as well** — with
the rulers hidden there is nothing to drag from, and turning Guides on by itself
looks like a feature that does not work. Drag a guide off the page to remove it,
or double-click it. pdfcer remembers a document's guides for next time, and a
document that already carries guides opens showing them.

The three switches are otherwise per document and are not remembered, so they
start off each time you open something. **Settings ▸ What is switched on when a
document opens** sets them for the documents you open from then on; it does not
change the one on screen, and it does not change any file.

### There is room around the page, and things out there are real

You can scroll well past the edge of the sheet — far enough to bring any corner
of the drawing to any corner of your screen. That grey space is not empty
padding. **If an object has been dropped off the page, it is drawn out there and
you can click it, box it and drag it back.**

This matters because a CAD export can put geometry outside the sheet without
telling you, and because it is easy to drag something off the edge by accident.
In most viewers that object is simply gone. Here it is where you left it.

Two ways to get one back:

* **Scroll to it and click it**, the same as anything on the sheet.
* **Box it.** Start the box on the page and drag out into the grey, or start it
  in the grey and drag back — either way a **right-to-left** box takes
  everything it *touches*, so it does not have to surround the object.

If you cannot find it, **Ctrl+A** — *Edit ▸ Content ▸ Select all* — takes
everything on the page **including** whatever is off it. Drag the lot back on,
or just look at where the handles appear to find what had wandered.

**If you scroll so far that no page is on screen at all**, the wheel still turns
pages and **Ctrl + wheel** still zooms. Those two gestures are kept alive on a
frame that drew nothing, precisely so an empty window is never a dead end;
anything that needs a pointer position on the page is not offered there.

---

## The ribbon

Seven tabs, and each answers one question:

| tab | the question it answers |
|---|---|
| **File** | what do I do with the file as a whole, or with pdfcer itself? |
| **View** | what is on my screen, and how is the page laid out? |
| **Pages** | what am I doing to the set of pages? |
| **Edit** | what am I changing about content that is already there? |
| **Markup** | what am I adding for someone else to read? |
| **Measure** | what am I measuring, and in what units? |
| **Tools** | what do I run across files, or configure once? |

An eighth, **Format**, appears only when you have something selected.

### When you make the window narrow

The ribbon gives ground in three stages, in this order:

1. **Commands re-wrap onto more rows** inside their section — up to three.
   Nothing is hidden.
2. **Whole sections fold into a single labelled button** with a `⌄`. Click it
   and the section's commands appear underneath. The section carrying the
   reason you are on that tab never folds — Save never folds on File, Zoom
   never folds on View.
3. **The band scrolls sideways**, with a `›` at the right edge and a `‹` once
   you have moved it.

Nothing is ever thrown away — it moves, in that order, and each stage hides a
little more than the one before. The ribbon's height never changes, so the page
below it does not jump about while you resize.

---

## Selecting and changing things

Four pointer tools, on the View tab and by single keystroke:

| key | tool | what it does |
|---|---|---|
| **V** | Select | click an object to select it; drag a box to select several |
| **A** | Points | shows an object's corner points so you can drag them |
| **T** | Text | select text by sweeping across it |
| **H** | Hand | pan the page |

A selected object gets handles: drag inside to **move**, drag a corner to
**resize**, and use the round handle above it to **rotate**. **Shift** while
dragging locks the movement to one axis; **Alt** suspends the snap for that one
drag.

**The arrow keys nudge what is selected** — one point a press, a quarter of a
point with **Ctrl** held. A point is the unit a PDF is defined in, so four
presses is four points and matches the number you would type into a properties
box. Shift and Alt are not modifiers here: Shift already means the axis lock,
and **Alt+↑ / Alt+↓** move a page. **Delete** or **Backspace** removes what is
selected.

**Which way you drag a box matters**, the same as it does in AutoCAD and
SOLIDWORKS:

| direction | what you get |
|---|---|
| **left → right** | only what the box completely **surrounds** |
| **right → left** | everything the box **touches**, even partly |

Right-to-left is the one to reach for on a dense sheet, and it is the only one
that can pick up a table hard against the edge of the page — there is no room
to draw a surrounding box around it.

Hold **Shift** while clicking or boxing to **add** to what you have; hold
**Ctrl** to **take things out** again. A Ctrl box that hits nothing leaves your
selection alone rather than clearing it.

**Ctrl+A** — *Edit ▸ Content ▸ Select all* — takes every object on the page,
including any that are off the sheet. It **replaces** what you had selected
rather than adding to it.

**Right-click** almost anything for the commands that apply to it.

The **Format** tab appears when something is selected, with its properties and
Delete. **Properties** also opens on a right-click.

The document's own title, author, subject and keywords are not properties of
anything you have selected, so they have their own window: **File ▸ Document ▸
Document properties**, beside Fonts, open in all three modes.

### The Select filter — deciding what a click may land on

The **Select** button at the left of the bottom bar opens a list of the eleven
kinds of thing a click can find, each with a switch:

| class | what it covers |
|---|---|
| **Characters** | the letters you sweep to copy |
| **Text** | a whole text object, as a thing to move or restyle |
| **Path** | any path — geometry, leaders, hatching, borders, the drawing itself |
| **Image** | a raster picture |
| **Form XObject** | a nested drawing treated as one object, such as a title block |
| **Part** | the rung below a whole object — one subpath, or one run of text |
| **Node** | the anchors on a subpath |
| **Markup** | a comment somebody drew |
| **ce dimension** | a measurement pdfcer wrote |
| **Form field** | one field on the page |
| **Link** | a link annotation, off unless you switch it on |

**The filter only ever takes things away.** Switching a class on never makes
something reachable that was not reachable before — it restores it. Switching
everything off is a legal state and a useful one on a dense drawing — let me pan
and read without grabbing anything — and the bar says so while it is in force,
so a canvas that has stopped responding to clicks is never a mystery.

It sits on the status bar rather than in the ribbon because it is a control you
need to see **while you aim**, and it composes with the mode: the filter can
only narrow what the mode already allows. Changing it is not an undoable edit —
it is a property of your session, not of the document — and your choice is
written out as you make it and is there next time.

---

## Measuring

The **Measure** tab. Distance, radius/diameter, perimeter, length along a path,
and the angle between two lines.

**Set the scale first** — Measure ▸ Set scale — or your numbers are in points
rather than in millimetres or inches. If a drawing has more than one scale on
it, **Dimension groups** lets you keep them apart instead of forcing everything
through one.

Picks snap to the real geometry rather than near it. **Alt** refuses the snap
for one pick, and **Tab** steps through the candidates under the cursor while a
measure tool is armed.

Measurements pdfcer writes are its own — **ce dimensions** — and are stored so it
can read them back. Dimensions that came from your CAD package are *content* and
pdfcer will not quietly change them.

**Area, and an angle tool you aim at an apex, are not there yet.** An angle
comes from picking two lines that are already on the drawing.

### Changing a measurement after you have placed it

Select one and the Properties panel shows its group, what it measured, a
radius/diameter switch for a circular one, and its unit, precision, decimal
marker, drafting standard, text height, line width, arrow length, arrow form,
colour and tolerance — each with a tick box to override what the group supplies
and a sentence saying where the value in force came from.

Moving a measurement into another group **re-measures it** against that group's
scale, so the printed number changes.

To change the corners of a perimeter or path measurement, select it and arm the
**Points** tool (**A**):

- **drag** a corner to move it;
- **Ctrl+drag** adds a new corner just after the one you grabbed, where you let
  go;
- **Ctrl+Shift+drag** removes that corner.

Each is one **Ctrl+Z**, and each says what changed — *a corner was added, how
many there are now, and what the measurement reads now against what it read
before*. A shape already at its minimum (three corners closed, two open) says so
instead of doing nothing.

**Dragging the label moves the label only.** No drag anywhere in this family can
alter the printed number except a corner drag, which re-measures honestly; an
angular dimension refuses a label drag outright.

---

## Marking up for someone else

The **Markup** tab: rectangle, ellipse, arrow, polyline, polygon, revision
cloud, freehand, plus highlight / underline / strikeout / squiggly for text,
sticky notes, text boxes and stamps.

Pick a colour and a line width in **Style** *before* drawing; they apply to what
you draw next.

For the shapes that take several clicks — polyline, polygon, cloud — click each
corner and then press **Finish shape** (or double-click) to end it. **Escape**
abandons the one in progress.

Everything here is an *annotation*: it sits on top of the page and can be
removed without touching the drawing underneath. That is why underline and
highlight live here rather than with the text tools.

### Changing the corners of a shape you have already drawn

Select the shape and arm the **Points** tool (**A**). Anchors appear on the
shapes whose corners can actually be moved, and on no others — an anchor is
drawn only where there is something a command can do with it.

| shape | move a corner | add one | remove one |
|---|---|---|---|
| polygon, revision cloud | yes | yes | yes, down to three |
| polyline | yes | yes | yes, down to two |
| line and arrow | either end only | no | no |
| freehand ink | yes, any point of any stroke | yes | yes, down to two per stroke |
| rectangle, ellipse, text markup | no | no | no |

The gestures are the same as a measurement's: **drag** moves, **Ctrl+drag**
adds, **Ctrl+Shift+drag** removes, with the Points tool armed. **Right-click an
anchor** instead and the menu offers *Add a point here* and *Remove this point*
with no tool armed at all — a menu row says what it does, so it needs no
modifier to tell it apart from a plain drag.

Freehand ink is a list of separate strokes, and a point is addressed by which
stroke it belongs to as well as where it sits in that stroke. Adding a point
after a stroke's last one **extends that stroke** rather than joining it to the
next, and no segment is ever drawn between two strokes, because the file holds
none.

If a stroke was drawn and smoothed by another program, re-baking it straightens
it. pdfcer tells you that off-canvas, in the same list that reports a stale
measurement — never as a mark on your drawing.

Each edit is one **Ctrl+Z**.

### Reading a comment somebody left you

**Click a sticky note on the page** and it opens where it sits, showing who
wrote it, when, and what it says. Click it again to close it. Hovering gives you
the gist without opening anything.

**This works in Read as well as Review and Edit.** Reading a comment is reading.
A note that the file itself was saved *open* opens with the document.

The **Comments** panel is the whole list — every annotation on every page. Use
it to work through a review rather than hunting the sheet:

- **Filter** by who wrote it, by what kind it is, by review status, or to just
  the ones that actually carry words. Most shapes pdfcer draws carry none.
- **Sort** by page, author or type.
- **Go to** takes you to the comment *and opens it*.
- **Reply** adds your answer to the thread.
- **Record status** — accepted, rejected and the rest — is **appended**, not
  set. A PDF keeps a status as a separate note pointing at the comment it
  judges, one chain per reviewer, so two people may hold opposite views of the
  same mark and the file holds both. The row shows whose status it is, and how
  many times that reviewer has changed their mind.
- **Delete** removes a comment. Available in Review and Edit; Read shows the
  words and offers no way to change them.
- Replies appear as a thread, gathered from wherever they live — a reply may
  legally sit on a different sheet from the comment it answers.

Reach it from the **left rail**, which every mode shows, or from
**Markup ▸ Comments**.

### Copying a comment to another place

**Ctrl+C** and **Ctrl+V** carry an annotation whole — sticky note, stamp, text
box, link, attachment, cloud — with its appearance, its author and its date
intact, within a document or across two open ones.

Paste works in **Review** as well as Edit. Review may paste a *comment*; it may
not paste *page content*, and it says so by name rather than doing nothing.

### Making another one of the same comment — without losing your clipboard

**Ctrl+D** puts a second copy of the selected comment on the same page, a little
down and to the right, and **does not touch the clipboard**. That is the whole
reason it exists: marking up a row of revision bubbles with Ctrl+C / Ctrl+V
costs you whatever you were carrying, once per bubble.

### When two comments overlap

To change which one is on top, select it and use **Markup ▸ Arrange**:

| | |
|---|---|
| **Ctrl+]** | bring it forward one step |
| **Ctrl+[** | send it back one step |
| **Ctrl+Shift+]** | bring it right to the front |
| **Ctrl+Shift+[** | send it right to the back |

These are the **square brackets with Ctrl held**. The bare **[** and **]**
rotate the *page* — a different job, and no modifier means no comment is
involved.

---

## Pages

The **Pages** tab and the page thumbnails panel.

Insert from another file, delete, extract, reorder (**Alt+↑** / **Alt+↓**),
split, merge, and rotate (**[** and **]**). **PageDown** and **PageUp** step
through pages; **Home** and **End** go to the first and last.

**Drag thumbnails between two open documents** to move pages across. Hold
**Shift** while dragging to move rather than copy.

A document will not let you remove its last page — there is no such thing as a
PDF with no pages.

### The check that runs before pdfcer writes a page change

pdfcer compares the page tree against itself before writing, and **will not
write a file it knows is damaged**. It shows you the two numbers that disagree
instead, and **Ctrl+Z** puts the pages back.

The refusal tells you four things, in this order: no file was written, your edit
is still here, the two numbers and what the damage looks like in Acrobat, and
how to undo the page removal. When the damage was already in the file you
opened, it says that instead — you have not caused it, and undoing your own edit
will not cure it.

The reason it is worth knowing about: a multi-sheet set from SOLIDWORKS or
another CAD exporter stores its pages in groups, and a file whose group totals
and declared total have drifted apart opens in Acrobat with blank pages at the
end. That is the shape of damage this catches before it reaches your disk.

---

## Text

**Ctrl+E** edits text that is already on the page. **Ctrl+Shift+E** adds new
text.

**Adding text.** Click the page for a single line, or **drag a rectangle** for
text that wraps: Enter starts a new line and **Ctrl+Enter** commits it. A PDF
has no paragraphs — every visual line is a separate instruction at its own
position — so wrapping needs a width to wrap against, which is what the
rectangle gives it. You choose the typeface (including bold and italic), the
size, the colour, the alignment and the line spacing.

**Changing how text already on the page looks.** Press **T** to arm the text
tool, sweep across the text, and the Properties panel grows a **This text**
section: font, size, **Bold**, **Italic** and colour. Each control is one edit
and one **Ctrl+Z**. Where a change cannot be made — a font the file does not
carry the information to substitute safely, for instance — pdfcer names the
reason rather than doing nothing.

**Re-wrapping a paragraph.** *Edit ▸ Reflow paragraph*, or right-click inside
the text while you are editing it. Use it after changing the words, when the
line breaks no longer sit where they should.

---

## Finding text

**Ctrl+F**.

### When "no matches" does not mean "not there"

Some PDFs store text as drawings with **no record of which letters they are**.
It renders perfectly and prints perfectly, and nothing can search it.

If you search a document like that, pdfcer says so and counts the fonts in it
that store text that way, so that "no matches" is never mistaken for "not in
this document".

Acrobat has the same limitation and says nothing at all. The text is not missing
and pdfcer is not failing — the *file* does not carry the information.

**Recognise text** (File ▸ Recognise) reads such a page and builds a searchable
layer for it. It works in all three modes.

It recognises, **shows you what it read**, and only then writes the layer into
the document you have open — as an ordinary edit. That means the title bar gains
its modified marker and **Undo takes the whole run back in one step**; nothing is
written to disk until you save, and you choose then whether that is over the
original or to a new name. Every word an OCR layer holds is a guess, which is why
you see the result before it is applied and why one undo is enough to drop it.

---

## Redaction

**Edit ▸ Protect ▸ Redact**, in Edit mode.

Mark what should go — by drawing boxes, or by searching for a word and marking
every match — and then apply. Marking is reversible; applying is not.

The Apply window asks where the redacted document goes before it does anything,
and shows you exactly what will be removed and anything it could not remove.

| choice | what happens |
|---|---|
| **This document — the removal happens when you Save** *(default)* | nothing is removed and nothing is written yet. **The page does not change**: you still see the marks and the content under them. Undo still works, and the window can call the whole thing off |
| **This document, now — the page changes immediately** | the content leaves the open document at the press. **Your undo history goes with it**, said at the control rather than discovered afterwards: the document is rebuilt around the redacted content, so nothing before the redaction can be stepped back. No file is written; the document on disk is unchanged until you save. The page you were looking at is where you stay |
| **A new file — you choose the name** | you are asked for a name, and it is never the name of the file you opened. The document you have open is untouched |
| **Replace the file you opened** | the file on disk is overwritten with the redacted version. One extra tick box, naming the file, before the button will work |

**Replacing is the only one that cannot be recovered from.** The file you
replace is the last copy of the content you are removing, so once it is gone it
is gone. You are warned at the moment you choose it — warned, not stopped; it is
your file.

There is no keyboard shortcut for Apply and **Enter does not press it**. It is
reached by pressing the button, deliberately.

### While a removal is armed

Taking the default arms a removal that is carried out at the write. Until then:

- **Save**, **Save As** and **Save a copy** all carry it out, and that is the
  only way this document can be saved while the removal is armed. Anything else
  that tries to write it is refused by name rather than quietly producing a
  half-redacted file;
- the save is a **full rewrite**, so the saved file keeps no earlier revision of
  itself — which is the point, because an earlier revision would still hold the
  content;
- **the removal stays armed after the save.** The window still shows the marks
  and the content, because the removal happened at the write and left the open
  document alone, and every further save does it again until you call it off;
- **your undo history is kept.** Arming a removal costs you nothing, and up
  until the save you can keep editing and undoing, this included. Once the
  file is written, undo still reaches your edits — it does not reach the file
  that has been written, and nothing does;
- **Cancel** un-arms it, leaves the marks in place, and lets ordinary saves work
  again;
- the document counts as unsaved, so closing it asks.

Nothing is on disk until you save. If you arm a removal and then hand over the
original file without saving, you have handed over the unredacted document.

### The warning to read

If the document contains text that cannot be searched (see
[Finding text](#finding-text)), then **"mark every match" cannot find it**.
pdfcer says so in the redaction panel, counting the fonts involved and stating
that any matches inside them were not marked and are still in the file.

Take that seriously. It is the one operation where a thing you believe finished
may not have, and you will not discover it by looking at the page. Check those
areas by eye and mark them with a box.

---

## Forms

### Filling one in

Click the field on the page and type, exactly as you would in any reader — in
**Read** and **Review** mode. Tick a check box by clicking it. There is no form
mode to enter and never will be; filling a form is the reason most form
documents exist.

**Tab moves to the next field** and **Shift+Tab** to the previous one, in the
order the form itself declares rather than the order the boxes happen to sit in.
It carries on across pages, scrolling only as far as it must to bring the field
into view and leaving your magnification alone. **Space** ticks a check box you
have tabbed to, instead of scrolling the page. A radio group is one stop, not
one per button.

On a page with no form, click the page and **Tab** walks the objects drawn on it
instead. Either way it stays in the document: it will not jump up into the
ribbon and leave you pressing Escape to get back.

The **Forms panel** (View ▸ Panels) lists every field and fills them too. Use it
when a field will not take a click: some fields cannot be typed into on the page
— one with nothing drawn there, one on a rotated sheet, or a drop-down — and the
panel says how many and why. It is also the surface a screen reader can work
through, so it is the accessible way in and the canvas is the second one.

The panel also **resets** a form, **redraws** the appearances a producer left
wrong, **flattens** fields into ordinary page content, and removes a whole field
group at once.

**pdfcer never runs a document's JavaScript.** Calculations it recognises — the
common sum, product, average and simple field arithmetic — it recomputes itself,
natively; anything outside that is left alone and said so. A form built as XFA
is disclosed as such rather than half-drawn.

### Making one

**Edit mode**, Edit ▸ Forms. Five buttons: text field, check box, radio button,
drop-down, button.

Press one, then either **click the page** to drop the field at its usual size,
or **drag a box** for an exact one. A window asks for the details. Nothing is
added to the document until you press **Add**, so pressing Escape costs you
nothing.

The settings you accept carry over to the next field you place — so a column of
identical check boxes is one set of choices and then a row of clicks.

**Give every field a different name.** Two fields with the same name are *one*
field shown twice: type in either and both change. pdfcer numbers new fields for
you so this cannot happen by accident, and it tells you if it does.

**Radio buttons are the exception, and this is the bit worth reading.** A set of
radio buttons is *supposed* to share one name — that is what makes picking one
clear the others. So give every button in a set the **same group name**, and a
**different value**. pdfcer keeps the group name for you and advances the value
as you place them.

### What a button does when it is pressed

Placing a button asks, seven ways: nothing, clear the form, go to a page, run a
named viewer command, show or hide fields, open a web address, or send the
form's data somewhere. Each one states what it reaches — including the ones that
reach nothing in this build — because a button's destination is written into the
document and nobody can see it by looking at the page.

Two of them write an address that another program may act on. Sending form data
over an unencrypted address is stated rather than refused: it is legal PDF, and
the decision is yours.

A button already on the page is in one of four states — no action, an action
pdfcer models, an action it can read but not offer, or an action it cannot read
— and the Properties panel says which, so replacing one is never a guess.

### Changing one that is already there

In **Edit** mode, click the field. Its details appear in the **Properties**
panel: what it is, which page it is on, what it holds, and which options are
switched on.

**What belongs to the field**, and therefore changes everywhere that field is
drawn: its **name**, **Required**, **Read only**, the **tooltip**, plus
**multiple lines**, **hide as typed**, **equal cells** and a **maximum length**
on a text field, **cannot be cleared** on a radio group, and **drop-down** and
**allow several** on a choice. A field drawn in more than one place changes in
every one of them, and pdfcer says how many are affected.

**What belongs to the box you clicked**, and changes only there: its
**rectangle**, its **border and background**, whether it is **visible** and
whether it **prints**, and a button's **caption**. *Delete this box* removes
only the one you clicked; deleting the field removes them all.

**Moving a box and resizing it are not the same operation.** A move carries
the field's artwork with it exactly and costs nothing. A change of size
would otherwise stretch that artwork, so pdfcer redraws it at the new size
and tells you it did — *the box moved* and *the box was resized and its
contents redrawn* are different things to have done to a file.

Each change is one press and one **Ctrl+Z**.

**The colours.** The box has a **background** and a **border**, and the text
drawn in it has a colour of its own — they are different things and they are set
separately. You can pick all of them before you place the first field, in the
placement window, and pdfcer carries the answer into the next field you place.
Each colour also offers **remove**, which makes the file silent about it, and
the background offers **no colour** as well; on a push button those two look
different, one giving no plate at all and the other the grey one. Whichever
would change nothing is greyed and says why.

**The field's own text is one group — font, size and colour together.** That is
how a PDF stores it: one line holding all three. So pdfcer reads all three back
before it writes any one of them, and changing the colour cannot quietly change
the typeface. A size of **Auto** means the reader picks a size that fits the box
and re-picks it as the value changes, which is usually what a new field wants.
The fourteen faces offered are the ones built into every PDF reader, so a form
using them looks the same everywhere and carries no embedded font. A signature
has no text of its own and so has no such group.

If a field's text is set in an ink with no single screen colour — a four-ink
separation, say — pdfcer says so and leaves it exactly as it is, rather than
showing you an approximation you would overwrite the moment you pressed Set.

**What cannot be changed is the kind of field it is.** A text field cannot
become a check box; place the one you want and delete the other.

Three things pdfcer tells you about rather than silently tidying up, because
each is a decision only you can take: a maximum length set shorter than what the
field already holds, a chosen drop-down option being removed, and a check box's
value being changed while it is ticked, which leaves it rendering unticked.

### The tooltip, and why pdfcer asks

Every new field asks for a tooltip. It is what a screen reader reads out, and
what shows on hover. Leaving it blank is a fine answer and pdfcer accepts it —
what it will not do is decide for you and write something you never chose.

---

## Printing

**Ctrl+P**. The print window is a window of its own: it has a taskbar entry and
can be dragged onto a second monitor.

Three tabs:

| tab | the question it answers |
|---|---|
| **Pages & Layout** | which pages print, and how each one lands on the sheet |
| **Copies & Finishing** | how many sheets come out, in what order, and on how many sides |
| **Comments & Resolution** | what is painted onto each page, and how finely |

**The preview shows the printable rectangle, not just the sheet** — what you see
is what the printer can actually reach. Pan and zoom it, or pop it out into a
window of its own.

**If a page will not fit, the button itself says so**, with how many pages are
affected, rather than warning you afterwards. The dialog is the confirmation;
there is no second "are you sure".

**Enter presses Print**, and Print is drawn in the accent colour so you can see
which button that is before you press it.

Three ways to leave, and they mean different things:

| | |
|---|---|
| **Print** | spools the job, and keeps the settings that describe you rather than this document — printer, orientation, duplex, tray, paper, scale, copies, collation, page subset and order, and the resolution ceiling |
| **Keep and close** | keeps those same settings and prints nothing |
| **Cancel**, Escape, or the window's close button | puts the settings back to what they were when the window opened |

A job the driver refuses **leaves the window open** with the reason on the
footer, so nothing is lost to a closing window.

**Printer properties** opens the driver's own sheet. Paper size is a *request*
to the driver, and pdfcer says so: the sheet you get is the driver's answer, and
the preview redraws against it. Choosing paper automatically picks the form the
pages themselves ask for, and says when the pages are of mixed sizes or larger
than anything the device offers.

N-up, booklet and poster layouts are not in the GUI.

---

## Settings

**File ▸ Settings**.

Most viewers decide these things for you. pdfcer asks, because the PDF standard
genuinely does not say — and where it is silent, two viewers can both be right
and disagree. Every setting tells you *what the standard leaves open*, what each
option does, and whether it affects only what you see or also what is saved.

### Presets

At the top: **pdfcer recommended**, which puts everything back if you have been
experimenting, and a list of published standards — PDF/X, PDF/A, PDF/UA.

Choosing a standard tells you **how much of itself that standard actually
specifies**, as a tally: how many of its answers the standard states, how many
are inferred from it, and how many are pdfcer's own judgement where it is
silent. For PDF/X-4, exactly one of six answers is a claim about the standard at
all. It also names the axes the standard says nothing about — those keep
whatever you have set — and where a standard specifies no rendering behaviour at
all, as PDF/UA does not, it says so in a sentence rather than showing three
zeroes.

Two standards that differ only in what they demand of the *file* — embedded
fonts, an output intent — render identically, and the window says so instead of
implying a change you would not see.

### The ones worth knowing about

| setting | why you might touch it |
|---|---|
| **How CMYK colour is shown** | pure blacks in CAD line art. pdfcer's default keeps them neutral |
| **Shrinking a large image to fit** | pdfcer smooths; the alternative is faster and can make thin lines shimmer |
| **Overprint in print-ready files** | if overprinted areas look wrong |
| **A gradient fill that comes out scrambled** | rare, and this is the fix when it happens |
| **What is switched on when a document opens** | rulers, grid and guides, for the next document you open |
| **Which paste chord does which** | by default **Ctrl+V** places a new field and **Ctrl+Shift+V** another view of the same one; the alternative is Acrobat's assignment, the two swapped |
| **Give the drawing more room** | auto-hide the ribbon, the rail, or the title bar |

The export windows remember what you last chose — what to export, the page
separator, the line endings and the byte-order mark for text; the units, arc
fitting and text handling for DXF. A drawing that carries its own calibration
overrules the DXF units you set, always in that direction.

---

## Every keyboard shortcut

### Files
| | |
|---|---|
| **Ctrl+N** | New |
| **Ctrl+Alt+N** | New from template |
| **Ctrl+O** | Open |
| **Ctrl+S** | Save |
| **Ctrl+Shift+S** | Save a copy |
| **Ctrl+W** | Close |
| **Ctrl+P** | Print |

### Editing
| | |
|---|---|
| **Ctrl+Z** | Undo |
| **Ctrl+Y** *or* **Ctrl+Shift+Z** | Redo |
| **Ctrl+A** | Select everything on the page, including anything off the sheet |
| **Ctrl+X / C / V** | Cut, copy, paste |
| **Ctrl+Shift+V** | Paste a form field as another view of the *same* field, not a new one — swappable with Ctrl+V in Settings |
| **Ctrl+D** | Duplicate the selected comment in place — the clipboard is left alone |
| **Ctrl+E** | Edit text |
| **Ctrl+Shift+E** | Add text |
| **Ctrl+F** | Find |
| **Ctrl+Shift+C** | Copy the page's text |
| **Delete** *or* **Backspace** | Delete what is selected |
| **Arrow keys** | Nudge what is selected one point — a quarter point with **Ctrl** |

### Stacking order
| | |
|---|---|
| **Ctrl+]** / **Ctrl+[** | Bring forward / send backward one step |
| **Ctrl+Shift+]** / **Ctrl+Shift+[** | Bring to front / send to back |

### Tools
| | |
|---|---|
| **V** | Select |
| **A** | Points |
| **T** | Text |
| **H** | Hand |
| **Alt** *(held)* | Suspend the snap for one pick or one drag |
| **Shift** *(held)* | Lock a drag to one axis |
| **Tab** | Step through the snap candidates under the cursor, while a measure tool is armed |

### Forms
| | |
|---|---|
| **Tab** / **Shift+Tab** | Next / previous field, in the form's own order, across pages |
| **Space** | Tick the check box you have tabbed to |

### View
| | |
|---|---|
| **Ctrl+0** | Actual size |
| **Ctrl+=** *or* **Ctrl++** | Zoom in one rung |
| **Ctrl+-** | Zoom out one rung |
| **Ctrl+H** | Read view |
| **F11** | Full screen |
| **Ctrl+Tab** / **Ctrl+Shift+Tab** | Next / previous document |
| **Ctrl+1 / 2 / 3** | Read / Review / Edit |

### Pages
| | |
|---|---|
| **PageDown** / **PageUp** | Next / previous page |
| **Home** / **End** | First / last page |
| **Alt+↑** / **Alt+↓** | Move page up / down |
| **[** / **]** | Rotate left / right — *no Ctrl*; with Ctrl they restack a comment |

**Escape** steps back one thing at a time, in this order: it leaves a form field
you are typing in, then abandons a drag in flight, then a guide you are
dragging, then a measurement or vertex run in progress, then puts the armed tool
away, then clears the selection. One press, one effect.

The window under **Help ▸ Keyboard shortcuts** is generated from the same keymap
the program dispatches, so it cannot drift from what the keys do.

---

## Where your settings live

Beside the program, in a folder called **`userdata`**:

| file | what it holds |
|---|---|
| `settings.txt` | the choices in the Settings window |
| `preferences.txt` | how pdfcer draws and behaves — sharpness, cache, zoom ceiling, print habits, paste chords |
| `layout.ron` | your panel arrangement |
| `select-filter.txt` | which classes the Select filter allows |
| `recent.txt` | recently opened files |

**Keep the `userdata` folder when you update pdfcer.** Replace everything else.

All of them are plain text you can read and edit, apart from the layout. An
unknown line is reported and kept, not deleted, and a value pdfcer cannot
understand falls back for that one setting alone — one bad line never costs you
the rest. Deleting a file resets exactly what it held and nothing else.

Document passwords are never written to any of them.

---

## When something goes wrong

**A page will not draw.** The status bar says why.

**A document asks for a password.** It is encrypted. pdfcer asks for the
password in a real window of the operating system's own, needs it to read the
file at all, and does not store it — so it asks again next time, and there is no
"remember this password" to tick.

**Text will not select.** The page may have no text on it — see
[Finding text](#finding-text). Recognise text will add some. Or the **Select**
filter at the bottom left has that class switched off.

**Nothing on the page can be clicked at all.** Check the Select filter — it is
legal to have switched everything off, and the bar says when you have.

**A command is missing.** Check the mode selector at the top right; you may be
in Read when you want Edit. If the window is narrow, the section may have folded
into a `⌄` button, or scrolled off — look for `›` at the right of the ribbon.

**Guides will not go on the page.** Switch the rulers on too — a guide is
dragged out of a ruler.

**The drawing has gone and the window is grey.** Scroll with the wheel or zoom
with Ctrl + wheel; both still work when no page is on screen.

**Something looks different from Acrobat.** That is often deliberate and the
Settings window explains which, and why. Colour and overprint are the usual two.

**pdfcer will not start at all.** If Windows reports a memory error, another
program is usually holding too many system resources. Restarting that program —
or the machine — clears it.

---

*This manual describes the portable build. `FEATURES.md`, shipped beside it,
lists every capability with what is and is not established about each.*
