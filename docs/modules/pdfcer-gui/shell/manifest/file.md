# `pdfcer-gui/shell/manifest/file`

The **File** tab — *what do I do with the file as a whole, or with
pdfcer itself?*

`RIBBON_IA.md` §5.1. Six groups: File, Save, Export, Print, Document,
pdfcer.


# What this tab stopped being

The salvage source's File tab was, in its own document's words, *"a
junk drawer"*: Properties, Copy this page's text, Copy the whole
document's text, Export DXF, Print, Reset layout, Settings, keyboard
shortcuts. Two of those are content operations and one is a view
operation. Meanwhile it had no New, no Recent, no Close — and no Open
and no Save, because those lived only on the quick-access toolbar and
the old reading of the one-command-one-tab rule forbade a tab from
mirroring them.

It also had no **Recent**, which is the absence an operator meets on the
second document rather than the first: with no Open command at all, the
only way to look at a file was to start the process with it on the command
line. Both are here now — `Open…` as a command, `Recent ⌄` as the gallery
§5.1 specifies (see the group below for why that is an `Item::Custom`).

Three things therefore happen here:

1. **`Open…` and `Save a copy…` appear on a tab**, under amendment P1a
   — *the QAT and the status bar are shortcut surfaces, not tabs; a
   command may appear on exactly one tab and additionally on the QAT.*
   A user who wants to open a file looks under File, and finding
   nothing there teaches them the ribbon is not where commands live.
2. **Copy page text / copy document text leave**, to Edit ▸ Clipboard.
   Copying text out of a document is a content operation.


And one thing arrives: **Fonts**, from View ▸ Panels. The Fonts panel
answers *"what is inside this file"*, not *"what is on my screen"*, so
it sits with Properties as document-level inspection. `RIBBON_IA.md`
§5.1 flags this as a real improvement on the current build rather than
a re-parenting — the panel is good and nobody was going to find it
under View.

# Why the Save group holds one command

`Save` — the one that overwrites in place — cannot ship before autosave
and crash recovery exist; that dependency predates this document. Under
P3 it is therefore **absent**, not greyed with an explanatory tooltip,
and `Save a copy…` stands alone in the band. `Revert` is absent for the
same reason: it is meaningless without a save point to revert to.
