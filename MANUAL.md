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

Open a PDF by dragging it onto the window, or **Ctrl+O**.

The document appears in the middle. Around it:

| | |
|---|---|
| **top strip** | the quick buttons (open, save, undo, redo), then the tabs, then **Read / Review / Edit** on the right |
| **the ribbon** | the band of commands under the tabs. What is on it depends on the tab *and* on which of the three modes you are in |
| **left panel** | page thumbnails, bookmarks, and whatever else you switch it to |
| **bottom strip** | page number, zoom, the fit buttons, find |

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

★ Each mode's tabs are a superset of the one before, so moving up never takes
something away. If a command you expect is missing, you are probably a mode too
low.

---

## Moving around a drawing

**Pan** by dragging with the Hand tool (**H**), or with the middle mouse button
at any time.

**Zoom** with **Ctrl + mouse wheel**, or the zoom box at the bottom.

The four fit buttons at the bottom: **actual size**, **fit page**, **fit
width**, **fit height**. All four also centre the view — pressing fit width
puts the page in the middle of the window, not off to one side.

★ **pdfcer zooms much further than most viewers** — to a trillion percent — and
the detail is really there rather than a blur of a big picture. It stays fast
because it only ever draws what is on your screen, so a huge zoom costs no more
than a small one.

If you have set a maximum zoom in Settings and want more, that number is yours
to raise; pdfcer does not decide it for you.

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

★ Nothing is ever thrown away — it moves, in that order, and each stage hides a
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
**resize**, and use the round handle above it to **rotate**.

**Right-click** almost anything for the commands that apply to it.

The **Format** tab appears when something is selected, with its properties and
Delete. **Properties** also opens on a right-click.

---

## Measuring

The **Measure** tab. Distance, radius/diameter, perimeter, length along a path,
and the angle between two lines.

**Set the scale first** — Measure ▸ Set scale — or your numbers are in points
rather than in millimetres or inches. If a drawing has more than one scale on
it, **Dimension groups** lets you keep them apart instead of forcing everything
through one.

★ Measurements pdfcer writes are its own and are stored so it can read them back.
Dimensions that came from your CAD package are *content* and pdfcer will not
quietly change them.

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

★ Everything here is an *annotation*: it sits on top of the page and can be
removed without touching the drawing underneath. That is why underline and
highlight live here rather than with the text tools.

---

## Pages

The **Pages** tab and the page thumbnails panel.

Insert from another file, delete, extract, reorder (**Alt+↑** / **Alt+↓**),
split, merge, and rotate (**[** and **]**).

**Drag thumbnails between two open documents** to move pages across. Hold
**Shift** while dragging to move rather than copy.

★ A document will not let you remove its last page — there is no such thing as
a PDF with no pages.

---

## Text

**Ctrl+E** edits text that is already on the page. **Ctrl+Shift+E** adds new
text.

What pdfcer can do to **new** text: choose the typeface (including bold and
italic), the size, the colour, the alignment and the line spacing.

What it **cannot yet do** is restyle text that is already there — you can change
what it *says*, not how it *looks*. That is a known gap and it is one missing
capability rather than several.

---

## Finding text

**Ctrl+F**.

### When "no matches" does not mean "not there"

Some PDFs store text as drawings with **no record of which letters they are**.
It renders perfectly and prints perfectly, and nothing can search it.

If you search a document like that, pdfcer will tell you: *"No matches. 2 fonts
in this document store text that cannot be searched, so there may be more."*

★ Acrobat has the same limitation and says nothing at all. The text is not
missing and pdfcer is not failing — the *file* does not carry the information.
**Recognise text** (File ▸ Recognise) adds a searchable layer over such a page.

---

## Redaction

**Edit ▸ Protect ▸ Redact**, in Edit mode.

Mark what should go — by drawing boxes, or by searching for a word and marking
every match — and then apply. Applying is the point at which content is
actually removed, and **it cannot be undone**.

### Where the redacted document goes — three choices, and the first is the default

The Apply window asks before it does anything, and shows you exactly what will
be removed and anything it could not remove.

| choice | what happens |
|---|---|
| **This document** *(default)* | the content leaves the document you are looking at and **nothing is written**. Save or Save As decides where it goes, exactly like every other edit. |
| **A new file** | you are asked for a name, and it is never the name of the file you opened. The document you have open is untouched. |
| **Replace `<your file>`** | the file on disk is overwritten with the redacted version. One extra tick box, naming the file, before the button will work. |

**Replacing is the only one that cannot be recovered from.** The file you
replace is the last copy of the content you are removing, so once it is gone it
is gone. You are warned at the moment you choose it — warned, not stopped;
it is your file.

### ★★ Applying into the document clears your undo history

If you take the default, the redaction goes into the open document — and the
**whole undo history goes with it**, not just the redaction. You can carry on
editing afterwards; you cannot step back past that point. The window tells you
how many steps you are about to lose, before you press the button.

Nothing is on disk until you save. If you apply and then hand over the original
file without saving, you have handed over the unredacted document.

### ★★ The warning to read

If the document contains text that cannot be searched (see above), then
**"mark every match" cannot find it**. pdfcer says so, in the redaction panel:

> *N fonts in this document store text that cannot be searched. Any matches
> inside them were NOT marked and are still in the file.*

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

The **Forms panel** (View ▸ Panels) lists every field and fills them too. Use it
when a field will not take a click: some fields cannot be typed into on the page
— one with nothing drawn there, one on a rotated sheet, or a drop-down — and the
panel says how many and why.

### Making one

**Edit mode**, Edit ▸ Forms. Five buttons: text field, check box, radio button,
drop-down, button.

Press one, then either **click the page** to drop the field at its usual size,
or **drag a box** for an exact one. A window asks for the details. Nothing is
added to the document until you press **Add**, so pressing Escape costs you
nothing.

The settings you accept carry over to the next field you place — so a column of
identical check boxes is one set of choices and then a row of clicks.

★ **Give every field a different name.** Two fields with the same name are
*one* field shown twice: type in either and both change. pdfcer numbers new
fields for you so this cannot happen by accident, and it tells you if it does.

★★ **Radio buttons are the exception, and this is the bit worth reading.** A set
of radio buttons is *supposed* to share one name — that is what makes picking
one clear the others. So give every button in a set the **same group name**, and
a **different value**. pdfcer keeps the group name for you and advances the value
as you place them.

### Changing one that is already there

In **Edit** mode, click the field. Its details appear in the **Properties**
panel: what it is, which page it is on, what it holds, and which options are
switched on. From there you can **rename** it or **delete** it. A field drawn in
more than one place also offers *Delete this box*, which removes only the one
you clicked.

★ In Edit mode a click **selects** a field rather than filling it — the same
split every program that both fills and builds forms uses, because one click
cannot do both. Drop to Review to go back to filling on the page.

★ **Required, read-only, the tooltip and the border can only be set when a field
is placed.** pdfcer cannot change them afterwards yet. To change one, delete the
field and place a new one.

### The tooltip, and why pdfcer asks

Every new field asks for a tooltip. It is what a screen reader reads out, and
what shows on hover. Leaving it blank is a fine answer and pdfcer accepts it —
what it will not do is decide for you and write something you never chose.

### The button that cannot do anything yet

**Button** is on the ribbon and greyed. pdfcer can draw a button correctly and
cannot yet give it anything to *do*, so one placed today would look right and
sit there. It stays visible, greyed, rather than disappearing, so you can see
that it is coming.

---

## Printing

**Ctrl+P**. The preview shows the *printable rectangle*, not just the sheet —
what you see is what the printer can actually reach.

If a page will not fit, the button itself says so and tells you how many are
affected, rather than warning you afterwards.

★ **Enter** presses Print, and the Print button is drawn as the default so you
can see that before you press it.

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

★ Choosing a standard tells you **how much of itself that standard actually
specifies**. For PDF/X-4 it is one answer out of six; the rest are inferred or
are pdfcer's own judgement. It also names what the standard says nothing about,
so a button bearing a standard's name cannot imply more authority than it has.

### The ones worth knowing about

| setting | why you might touch it |
|---|---|
| **How CMYK colour is shown** | pure blacks in CAD line art. pdfcer's default keeps them neutral |
| **Shrinking a large image to fit** | pdfcer smooths; the alternative is faster and can make thin lines shimmer |
| **Overprint in print-ready files** | if overprinted areas look wrong |
| **A gradient fill that comes out scrambled** | rare, and this is the fix when it happens |

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
| **Ctrl+X / C / V** | Cut, copy, paste |
| **Ctrl+E** | Edit text |
| **Ctrl+Shift+E** | Add text |
| **Ctrl+F** | Find |
| **Ctrl+Shift+C** | Copy the page's text |

### Tools
| | |
|---|---|
| **V** | Select |
| **A** | Points |
| **T** | Text |
| **H** | Hand |

### View
| | |
|---|---|
| **Ctrl+0** | Actual size |
| **Ctrl+H** | Read mode |
| **F11** | Full screen |
| **Ctrl+Tab** / **Ctrl+Shift+Tab** | Next / previous document |
| **Ctrl+1 / 2 / 3** | Read / Review / Edit |

### Pages
| | |
|---|---|
| **Alt+↑** / **Alt+↓** | Move page up / down |
| **[** / **]** | Rotate left / right |

★ **Escape** steps back one thing at a time: it abandons a shape in progress,
then puts the tool away, then clears a selection. It does not do all three at
once.

---

## Where your settings live

Beside the program, in a folder called **`userdata`**:

| file | what it holds |
|---|---|
| `settings.txt` | the choices in the Settings window |
| `preferences.txt` | how pdfcer draws — sharpness, cache, zoom limits |
| `layout.ron` | your panel arrangement |
| `recent.txt` | recently opened files |

**Keep the `userdata` folder when you update pdfcer.** Replace everything else.

All of them are plain text you can read and edit. An unknown line is reported
and kept, not deleted, and a value pdfcer cannot understand falls back for that
one setting alone — one bad line never costs you the rest.

---

## When something goes wrong

**A page will not draw.** The status bar says why. Encrypted documents open
read-only.

**Text will not select.** The page may have no text on it — see
[Finding text](#finding-text). Recognise text will add some.

**A command is missing.** Check the mode selector at the top right; you may be
in Read when you want Edit. If the window is narrow, the section may have folded
into a `⌄` button, or scrolled off — look for `›` at the right of the ribbon.

**Something looks different from Acrobat.** That is often deliberate and the
Settings window explains which, and why. Colour and overprint are the usual two.

**pdfcer will not start at all.** If Windows reports a memory error, another
program is usually holding too many system resources. Restarting that program —
or the machine — clears it.

---

*This manual describes the portable build. `FEATURES.md`, shipped beside it,
lists every capability with what is and is not established about each.*
