---
name: learn-a-reference-app-by-photographing-it
description: Ken asks for parity with Word/Acrobat by name — drive the real app and photograph it at a series of widths; its layout rules are in no API.
metadata:
  type: feedback
---

When Ken says *"learn how Word handles X"* or *"Adobe has Y"*, he means **go
and look at the real application on this machine**, not reason from memory.
Both Word and Acrobat are installed; SolidWorks too.

**Why:** 2026-08-24, he asked for ribbon parity and added *"if you can, drive
word as it is installed on this machine."* Word is COM-automatable and
**cannot answer** a layout question — `CommandBars` is the 2003 toolbar
surface and the ribbon's scaling rules live inside the Office UI framework,
exposed nowhere. The instrument is a **camera**:
`tools/word-ribbon-study.ps1` resizes the window through twelve widths and
captures each. Twelve pictures answered everything.

**How to apply:** photograph the reference AND our own build at the **same
widths**, into `evidence/`. Two series give a number — *10 groups on the band
versus 3 at 884 points* — where one series gives an opinion, and the number is
what sizes the work and then measures it. Largest width first (an incremental
re-layout means a growing series photographs the recovery path). Use the first
**visible** top-level window of the pid, never `Process.MainWindowHandle`.

★ And do not copy the reference's rule wholesale. Word's icon-only clusters
work because `B`/`I`/`U` are findable by shape; the same treatment on
*Export form data…* would be two mystery glyphs. Size is a property of the
**command**, not of the surface — which is why it ended up declared per item
in the manifest and why the File tab was deliberately left alone.

Related: [[feedback_use_the_conventional_interaction_never_invent_one]],
[[feedback_never_drive_the_published_build]].
