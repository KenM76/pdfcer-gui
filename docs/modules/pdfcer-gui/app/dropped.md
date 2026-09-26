# `app::dropped` — **files dragged onto the window**

## What this closes

The operator: *"also can't drag and drop a jpg file onto a new pdf, and the
insert image button doesn't insert it either."*

Only the first half was true. The Insert-image button works —
`insert_image_places_a_picture` drives it end to end on a real JPEG — and
**nothing read `dropped_files` at all**, so a file dragged onto the window
did nothing, silently, with no cursor feedback on the way in. This module is
the reading.

## Why "does nothing" is worse here than almost anywhere else

Because drag-and-drop is the one gesture with **no discoverable
alternative**. A missing menu item can be looked for; a missing chord can be
found in a shortcuts window. A drop that is ignored teaches the operator
that this program does not accept drops — a conclusion they will not revisit,
and one they reached about a program that opens documents for a living.

⇒ It also costs more than the feature it is. The same report named the
Insert-image button, which works: a drop that fails silently makes a working
button look broken, because both get tried in the same minute and only one
of them tells the operator anything.

## What a drop means, by what was dropped

| dropped | action |
|---|---|
| a **PDF** | open it — the same [`Action::Open`] the File ▸ Open picker raises |
| a **raster image** (png/jpg/bmp/tif) with a document open | insert it, straight into the placement window |
| a raster image with **no** document open | say so, and say what to do about it |
| anything else | say what pdfcer accepts |

**A dropped PDF opens rather than being inserted**, and that is the
decision most worth stating because the opposite is defensible. Every viewer
in this class opens a dropped PDF; `pages.insert` is a deliberate act with a
position and a page range, and inferring it from a drag would make the
commonest gesture in the product do the rarer of two things. The operator who
wants to insert has a command that asks them where.

## Why the drop is read where the ribbon is, and not in the canvas

`egui` reports drops on the **`Context`**, not on a widget — `RawInput`
carries `dropped_files` for the whole window and nothing narrows it to a
rect. So a drop anywhere on the window is one event, and reading it inside
the canvas would be reading a window-scoped fact in a page-scoped place: a
drop on the ribbon or on a dock panel would be missed, and the operator would
learn that the program accepts drops *sometimes*, which is worse than never.

## This module is the FALLBACK, and it is handed its files

`OPERATOR_REQUESTS.md` O67 asks for a drop onto the **thumbnails** to
import pages, which needs the one thing the paragraph above says does not
exist: a position. [`crate::app::filedrag`] supplies it — from the
operating system, because the toolkit discards it — and lets a surface
**claim** a drop that landed on it.

⇒ So this module answers a drop that **nobody claimed**, it runs at the END
of the frame after every surface has had its chance, and it does not read
`egui`'s input itself — it is handed the paths. Two readers of one
`dropped_files` would each see it and each act.

The fallback is unconditional, which is the safety property: a surface
that forgets to claim costs a feature and never a file — the failure is
*"it opened in a tab instead of inserting"*, which the operator can see and
undo.

## What is deliberately NOT here

- **Multi-file drops.** Only the first is acted on, and the rest are named in
  the disclosure. Opening five documents at once is a tabbed shell this one
  is not; inserting five images is five placement windows, and the second
  would open over the first with no way to tell them apart.
- **Hover feedback on the way in.** `egui` offers `hovered_files`, and a
  preview would be the right thing eventually. It is left out today rather
  than done badly: a tint that appeared on any hover, including over a
  document that cannot take the file, would promise a drop that then refused.
