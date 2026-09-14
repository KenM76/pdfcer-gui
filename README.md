# pdfcer

**A PDF viewer and editor that shows you what is really in the file.**

pdfcer is a single Windows executable. Unzip it, run it — nothing is installed.

It opens the ordinary things: reports, scans, forms, print-shop artwork. It
also opens the awkward ones — the A1 site plan, the 5 MB CAD export with forty
thousand vector paths, the file another viewer calls damaged. Most of the work
in pdfcer has gone into reading PDFs faithfully, and that shows up as files
simply looking right.

### [⬇ Download the latest build](https://github.com/KenM76/pdfcer-gui/releases/latest)

---

## Why files look right in it

The compatibility work is done against corpora of thousands of real-world PDFs
from every kind of producer — not against a handful of samples.

**CMYK is converted with measured data, not a formula.**
The PDF standard deliberately says nothing about how CMYK should look on a
screen, so every program has to choose. pdfcer's choice is a 1,296-point table
fitted to measured render output: solid cyan, solid black ink and every
overprint of them are the values somebody actually measured, not what
arithmetic predicts. The formula it replaced was off by an average of 32 shades
out of 255; this one is off by about one. Every CMYK colour in the file goes
through it — fills, images, and the colour the properties panel shows you — so a
rectangle and a photograph of the same colour agree. No colour profile is
shipped or needed, so there is nothing to license or install.

**And when a file asks to be composited in ink, it is.**
Artwork that declares a CMYK blending space is blended on separate colorant
planes — the four process inks, plus a plane of its own for each named spot, a
varnish or a Pantone — with overprint and blend modes worked out there and the
conversion to screen happening once, at the end. Blending in the wrong space is
how a rich black quietly becomes grey.

**Transparency behaves.**
All sixteen blend modes, transparency groups and soft masks — including the
four awkward ones, hue, saturation, colour and luminosity, that are the easiest
to get wrong.

**Damaged files still open.**
When a PDF's internal index is broken, pdfcer ignores it and scans the file for
the objects themselves. Measured over a 1,109-file corpus of real-world PDFs,
that one recovery path covers 85% of the files that failed to load at all.

---

## What you can do with it

**Open a big drawing and actually move around it.**
Panning and zooming stay smooth on sheets that make other viewers stutter,
because the last good picture stays on screen — in the right place — while the
next one is drawn. Small moves cost you nothing at all; a fast sweep right
across a big sheet still has to catch up.

**Zoom in until you can see anything.**
Up to a trillion percent, and the detail is genuinely there rather than a
blurred enlargement. A benzene molecule drawn at true scale is legible on
screen. How far it will go is a setting you own — how much performance to spend
on magnification isn't pdfcer's decision to make.

**Measure on a scaled drawing.**
Click two points, pick two lines already on the drawing, or click a hole for its
radius or diameter. Picks snap to the real geometry rather than near it, the
scale you calibrate is written into the file and read back next time, and a
measurement stays editable after you place it. *(Area and angle aren't there
yet.)*

**Mark up a set for somebody else.**
Sticky notes, text boxes, arrows, shapes, revision clouds, freehand,
highlight / underline / strike-through, and stamps. Copy and paste markup
between pages and between documents with its appearance intact — and where a
kind cannot be copied, it is refused by name rather than dropped.

**Read other people's comments properly.**
Click a sticky note on the page and it opens with its author, its date and its
words. The comment list filters by author, by type, and by whether a comment
actually carries any text — then jumps you to it on the page.

**Change what's on the page, not just what's stuck to it.**
Edit text in place, and reflow a paragraph around the change when you ask it
to. Drag the nodes of a vector path. Select an object and edit its real
properties — position, size, colour. Colour controls show a proper
indeterminate state when a selection disagrees with itself, and refuse *by
name* when a colour would destroy a spot ink.

**Reorganise pages — across documents.**
Insert, extract, rotate and delete pages. Open several documents at once and
drag pages between them — dragging copies them, holding Shift moves them, and
the status line says which before you let go.

**Redact so that it stays redacted.**
The content is removed, not covered with a black rectangle that a text search
finds anyway. pdfcer tells you what it could and could not read while doing it.

**Fill in a form — or build one.**
Text fields, checkboxes, radio buttons and dropdowns — filling them in and
authoring them both.

**Find out whether a signature means anything.**
Who signed, what they covered, and whether the bytes still match — and where
the answer is no, it says *why*. Checking the signer against Acrobat's own
certificate list is there to switch on.

**Print what you meant to print.**
A live preview beside the settings, so you can see the sheet and what you are
changing at the same time — and the commit button says in words what will be
clipped.

**Get the content back out.**
Export to DXF, PNG, JPEG, SVG, EMF, plain text or form data — or copy page
content straight to the clipboard as **vector**, which pastes into Word as
shapes you can still pull about rather than as a picture of shapes. (For a PDF
out, it is Save a copy — or Save a compacted copy, which rewrites it smaller.)

**Three modes, so the tools match the job.**
**Read** hides the editing machinery entirely. **Review** gives you markup and
page operations. **Edit** gives you everything. Panels dock, split and
rearrange, and remember where you left them between sessions.

---

## Two things it does differently, on purpose

**It never scribbles on your document to tell you something.**
No dashed outlines, red flags or "provisional" tints painted over the page.
What you see while editing is exactly what you'll get when you save. When
pdfcer has had to guess at something — a substituted font, a best-fit scale, an
OCR result you can't see — it says so in the status bar or in a report, off the
page, where it can't be mistaken for part of your document.

**It would rather tell you than quietly cope.**
Every page that gets rendered comes back with around a hundred named counts of
what had to be approximated, and the program turns those into sentences. If a
colour can't be applied, a font isn't installed, a search couldn't read a
region, or a page wouldn't render, you get a sentence that names the thing and
what to do about it. Silence is the one failure mode that costs you a file you
thought was fine.

---

## Status, honestly

pdfcer is **pre-1.0 and under active development** — new builds most days, and
each release note says what changed in plain terms. It is used daily on real
files by its author, which is why it gets fixed fast; it is not yet a program
with years of other people's edge cases behind it.

**[`FEATURES.md`](FEATURES.md) is the authoritative list**, and it makes a
distinction worth knowing about: a feature is only ticked when somebody has
driven it in a running window. "The code exists and the tests pass" gets a
different mark, because this project has shipped features that every test
passed and that did not work. Some of what is listed above is newer than its
verification; `FEATURES.md` marks every one of those separately, and if one of
them misbehaves, that is worth a line in the issues — it is the half of the
list nobody has walked yet.

---

## Getting started

| | |
|---|---|
| **[`MANUAL.md`](MANUAL.md)** | the user manual — the first five minutes, every shortcut, and what to do when something goes wrong |
| **[`FEATURES.md`](FEATURES.md)** | what works today, and what is coming next in order |
| **[Releases](https://github.com/KenM76/pdfcer-gui/releases)** | every build, newest first, with notes |

Open a PDF by dragging it onto the window or pressing **Ctrl+O**. Scroll to
move, **Ctrl+scroll** to zoom at the cursor. The rest is on the ribbon.

---

## Licence

pdfcer-gui is **MIT** — see [`LICENSE`](LICENSE). That covers everything in
this repository, including the icons, which are the author's own work.

It does not cover everything inside the executable: pdfcer links its PDF engine
statically, and that engine embeds third-party font faces and data tables.
Their notices travel with every build, in full, as
`THIRD_PARTY_LICENSES.md` — and in the program itself under
**File ▸ pdfcer ▸ About pdfcer**.

## Built on

The **pdfcer engine** — [KenM76/pdfcer](https://github.com/KenM76/pdfcer) —
which does the PDF reading, writing and rasterizing, and has its own command
line. This repository is the desktop program: the window, the ribbon, the
canvas and the editing.

## For developers

[`DEVELOPING.md`](https://github.com/KenM76/pdfcer-gui/blob/main/DEVELOPING.md) is the engineering front page — architecture,
the reusable `egui-shell` crate, the build and packaging tooling, the measured
performance work, and the project's own record of the times its documentation
was wrong. [`RESUME.md`](https://github.com/KenM76/pdfcer-gui/blob/main/RESUME.md) is where a working session starts.
