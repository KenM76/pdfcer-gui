# `pdfcer-gui/app/actions/outward`

**Applies a link leaving the document.** For `WebLink`, `prefs.trust`
decides: Open hands the address to `outward::open`; Refuse writes a status
line naming where the refusal is changed; Ask raises the Security Warning.
For `WebAnswer`, a ticked Remember stores the answer for the site and saves
the preferences, then Allow opens and Block writes a status line.

Trace: `weblink-decision decision=open|refuse|ask site=…`. Every outcome is a
status line; nothing is refused silently. Design: `DESIGNS.md` § O320.
