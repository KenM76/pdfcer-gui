# pdfcer GUI — Ribbon Information Architecture

Where every user-reachable command lives, and why. This is the settled spec for
the shell; it is not improvised around.

| Companion | What it holds |
|---|---|
| `FEATURES.md` | what is built. Authoritative for status |
| `GUI_ROADMAP.md` | when each part gets built |
| `MODES_AND_PANELS.md` | the Read/Review/Edit selector and the flexible panel system |
| `RIBBON_SCALING.md` | how a band packs its rows and how the ribbon narrows |
| `mockups/` | what it looks like |

---

## 1. What this document is for

Seven fixed tabs and one contextual tab. Each tab answers one stated question,
each command has exactly one home, and each placement carries the reasoning that
produced it. It is written so the ribbon could be rebuilt from this document
alone.

**It specifies placement, never status.** A row here says where a command lives
*when it exists*; it is not a claim that it ships. R9 settles the rest: an
unregistered command renders nothing, and a group whose every command is
unregistered is absent too — so a reader can never be misled by the ribbon
itself, only by reading a row here as a status line. `FEATURES.md` is
authoritative for what is built; `GUI_ROADMAP.md` for when.

**The layout below is Edit mode's.** Read and Review are *subsets* of it, not
different layouts: the mode selector governs which tabs and panels are
**present**, not where a command lives when it is. Read's tab list is
`["file", "view"]` (`crates/pdfcer-gui/src/shell/ron/built_in.ron`).

---

## 2. Principles

**P1 — One command, one tab.** No command appears on two ribbon tabs, contextual
tabs included. A Format tab that re-hosted a Markup tab command produces the
two-answers problem on a surface that appears and disappears, which is worse
than doing it on a fixed tab. Enforced by `check_one_command_one_tab`
(`crates/egui-shell/src/manifest/validate.rs:339`).

The reason is navigational: if a command can be on two tabs then *"where is Fit
page?"* has two answers, and an operator who found it once on View has learned
nothing about where anything else lives. One home per command makes the tab set
a **map** instead of a menu that repeats itself. Mirroring a command onto every
tab also breaks the band: the ribbon renders only the active one, and the
mirrored command becomes unreachable from the rest.

A P1 violation is not a local failure. `Shell::validate` rejects the whole
manifest, and `Capabilities::for_mode` falls back to `Capabilities::FULL` when
the shell is absent — so an invalid manifest silently grants every authoring
capability to every mode.

**P1a — The QAT, the status bar and the trailing region are shortcut surfaces,
not tabs.** A command may appear on exactly one tab and additionally on any of
them. This is how Office defines its QAT, and it does not reintroduce P1's
defect, which was a command reachable from **no** tab once the band collapsed —
the opposite failure. `check_one_command_one_tab` walks tabs only; the QAT is
checked separately for self-duplication alone. A shortcut to a known home is not
a second place to hunt.

**P2 — The ribbon picks the activity; the sidebar holds that activity's
controls.** The Measure tab arms *Linear ce dimension*; the group picker, scale
entry, number format and drafting standard live in the Tool Options pane. The
fix for an underfilled tab is never to move sidebar controls up into it.

**P3 — No placeholders.** An unavailable capability renders nothing, not a
disabled stub. Greying is reserved for *temporarily* unavailable — no document
open, document encrypted, undo stack empty, this one mark locked — and is always
explained on hover. This is R9 in ribbon terms.

**P4 — Group captions are mandatory.** Every group draws its caption beneath its
controls, guaranteed by routing all groups through one closure rather than by
each group remembering to.

**P5 — Nothing floats over the canvas.** Tool options live in the dock, not in
canvas-anchored overlays: accept/reject boxes that moved on every zoom were a
reported defect.

### A refusal in a mode means the tab is wrong

**A command refused in a mode where the operator plainly needs it is evidence
that the command's tab is wrong, not that the mode gate needs an exception.**
Three placements are that rule applied — `view.panel_forms`,
`file.copy_page_text` / `file.copy_document_text`, and `file.ocr` — and §7
carries each one with its reasoning.

### Where the mockup is the spec, and where it is not

The operator approved the mockup, so where the mockup and the build disagree
**the mockup is right and the difference is a defect in the build** — except in
three cases, where the build is right: a capability shipped after the mock was
drawn; the build's choice carries a written argument and the mock's carries
none; or the mock's picture would break P1 or R9.

**A mockup is a specification only where it is DERIVED.**
`mockups/build-pdfcer-shell.py` reads the glyph inventory out of `icons/assets/`
and never types it, so *"this glyph ships"* is true there by construction. The
ribbon in that mock is **typed**, so it can describe a build that does not exist
and nothing in the mock will say so. `python tools/compare-mockup-ribbon.py` is
the derivation the ribbon does not otherwise have: it compares band structure
and, per group, item presence and order.

It compares **the asset each control draws**, resolved through
`asset_by_icon_key`, not icon key strings. The two sides key differently on
purpose — the mock's key is the asset basename, the build's key is the *role*, so
that two commands sharing one picture stay two commands — and comparing the key
schemes asks a question neither side is answering. It cannot see item **size**,
item **label**, or the mock's column split, and `CUSTOM_GLYPHS` in it is a
hand-maintained claim about what each `Item::Custom` paints. It reads the ribbon
alone: the left rail carries bands of its own (`Navigate`, `Select`, `Rotate`),
one colliding by name with View ▸ Navigate, so a green ribbon line says nothing
about the left edge.

### The icon-adoption rule

**A glyph is adopted only when a command or role in this build would use it
today.** A glyph that restyles art already held is deferred — a restyle is a
different decision from filling a gap. A glyph with no home is deferred: an
`Icon` variant nobody names is dead weight a future reader must still evaluate.
This is R9 applied to the icon set — an unavailable capability renders nothing,
and an unused glyph is a capability nobody has.

Two corollaries:

- **Check the command is registered before adopting its art.** A glyph drawn for
  a mock of a command that no longer exists is how a deleted feature comes back.
- **Distinct roles get distinct art.** Shared art is allowed only where the
  controls have the same *subject*, and even then the catalogue keeps a distinct
  key over the shared art (`crates/pdfcer-gui/src/icons/catalog/mod.rs`), so that
  two commands remain two commands. An icon key aliased onto art drawn for a
  different role is a gap wearing a solution's clothes — the same defect class as
  two commands sharing one glyph, one level down.

---

## 3. The failures this layout exists to prevent

Each is the reason a later section refuses something that otherwise looks
reasonable.

**A tab whose name promises a subject must carry that subject.** A View tab
without zoom, page layout, view rotation, read mode or full screen teaches the
operator that the ribbon is not where commands live. Keyboard-only access to a
capability named on a tab is the same failure.

**A capability reachable only from a panel is not discoverable.** The ribbon is
the discoverable path and the panel is the fast path (§5.3).

**A junk-drawer tab is worse than an empty one.** File holds what is done to the
file as a whole or to pdfcer itself, and nothing else. Panel-layout reset is a
view concern and lives on View ▸ Window.

**An empty band reads as an unfinished program** regardless of how much
capability sits behind the controls that are there. The remedy is the tab set of
§4, never moving Tool Options controls up into the band (P2).

**Every control carries a word.** An icon alone is not a label, and an
abbreviation is not a word — `Obj` is not a word. The Content group's buttons
read *Edit text* and *Add text*, with their icons kept.

---

## 4. The tab set

| # | Tab | The question it answers |
|---|---|---|
| 1 | **File** | What do I do with the file as a whole, or with pdfcer itself? |
| 2 | **View** | What is on my screen, and how is the page laid out? |
| 3 | **Pages** | What am I doing to the set of pages? |
| 4 | **Edit** | What am I changing about content that is already there? |
| 5 | **Markup** | What am I adding for someone else to read? |
| 6 | **Measure** | What am I measuring, and in what units? |
| 7 | **Tools** | What do I run across files, or configure once? |

Plus one contextual tab:

| Tab | Appears when |
|---|---|
| **Format** | A markup, ce dimension, image, vector object or text run is selected |

**Why seven.** Six is one too few for the amount of capability behind them, and
the sixth ends up carrying two unrelated jobs. The split is: page operations out
of hiding into their own tab, View given the view controls its name promises,
and File left as an actual file tab.

**Why Markup and not Review.** What lives there is markup *authoring* — shapes,
notes, stamps. "Review" promises a review **workflow**: compare revisions,
resolve comments, track changes. pdfcer does not have that, and when it does it
will want the name. `Markup` is also the term this audience uses; Bluebeam and
every drafting office call it that.

**Why Format is contextual.** It is how the P2 tension resolves. The Markup
tab's Style group sets the style of the *next* mark, so without a contextual tab
a placed mark has nowhere to be changed and feels final the moment it lands.

---

## 5. Tab specifications

Groups are `**Group**`. `⌄` means the control is a split button or dropdown.

---

### 5.1 File — *what do I do with the file as a whole, or with pdfcer itself?*

| Group | Commands |
|---|---|
| **File** | New (blank) · New from template… (page size) · Open… · Recent ⌄ · Close |
| **Recognise** | OCR… |
| **Save** | Save · Save as… · Save a copy… · Save compacted · Revert |
| **Export** | Export DXF… · Export image… (PNG/JPEG/TIFF, DPI picker) · Export text… · Import text · Stamp collection · Export form data ⌄ (FDF / XFDF / CSV) · Import form data · Copy this page's text · Copy the whole document's text |
| **Security** | Encrypt · Permissions · Sign |
| **Print** | Print… · Imposition… (n-up / booklet / poster) |
| **Document** | Properties · Fonts |
| **pdfcer** | Settings… · Keyboard shortcuts · About |

**Save writes the open file in place, and the write is atomic.** pdfcer writes
an **incremental update** — the previous revision stays in the file — so the
format is itself the crash recovery that an in-place `Save` was once held back
waiting for. The hazard was the write rather than the format: a truncate-then-
stream `fs::write` loses the document if the process dies mid-write, so
`app::save::save_in_place` materialises the whole replacement in a temporary
beside the target and renames it over. The group keeps `Save as…` and `Save a
copy…` because a reviewer often wants the original left untouched, and `Revert`
follows `Save` rather than leading it — there is no save point to revert to
until something has been saved. The standing rule *Read may produce a new
document; it may not modify this one* is load-bearing from the day in-place Save
landed, and is enforced by the mode's capabilities rather than by the Save
button.

**Text copy is on Export, not on Edit ▸ Clipboard**, and OCR is in Recognise
rather than on Tools. Both are §2's mode rule applied; §7 carries them.

**Fonts is document-level inspection, not a view.** The Fonts panel answers
*what is inside this file*, so it sits with Properties rather than under View ▸
Panels, where nobody will find it.

---

### 5.2 View — *what is on my screen, and how is the page laid out?*

| Group | Commands |
|---|---|
| **Page display** | Single page · Continuous · Facing · Facing continuous |
| **Navigate** | Select · Points · Text · Hand · Smart select |
| **Render** | Strategy: Whole page · Tiled progressive · Raster scale ⌄ (quality) · Settle delay · Antialias ⌄ (text / vector) |
| **Rotate view** | Rotate view left / right |
| **Zoom** | Zoom to selection · Zoom to region (marquee) · Actual size · Fit page · Fit width · Fit height |
| **Display** | Line weights · Show annotations · Show points · Off-page · Rulers · Grid · Guides |
| **Panels** | Sidebar ⌄ · Pages · Bookmarks · Layers · Objects · Signatures · Forms |
| **Window** | Previous / Next document · Close other documents · Read mode · Full screen · Dock all panels · Auto-hide ribbon · Auto-hide left strip · Floating panels: Off · Allowed · App initiative: Never · Ask · Allowed · Save workspace… · Load workspace ⌄ · Reset layout… |

**Single page is the default** and stays so: paging one drawing sheet at a time
is the right model for drafting review. Continuous is a *mode you choose*
beside it, for the document that is a 40-page specification rather than a sheet
set. The four are radio-style — exactly one active — and the choice persists
**per document**, so opening a drawing set does not inherit a report's setting.
Defaults per mode come from `viewer::display::PageDisplay::default_for_mode`.

**Line weights is a canvas-only display aid.** Print, preview and every export
render real widths. It sits in Display rather than in Render because it is
flipped while reading a sheet rather than set once, which is P2's distinction.

**Render is a stated trade, not a better and a worse.** pdfcer caches one
whole-page texture and scales it with linear filtering during the settle, which
on a large drawing is *smoother* to pan and zoom than progressive tile rendering
— no seams, no piece-by-piece fill-in — at the cost of a full re-raster once
motion stops. Which wins depends on the sheet and the machine, so the strategy
is a choice on this tab, with whole-page as the default because it is what
measured better. The settle delay and the raster-scale multiplier are the two
constants that become knobs beside it.

**Zoom here does not duplicate the status bar in spirit.** The status bar keeps
the continuous controls reached for constantly. This tab adds the two *targeted*
zooms that have no status-bar home — to selection and to a marquee region — and
mirrors the named zoom levels under P1a, so that an operator looking under View
for zoom finds zoom.

**Panels order puts Forms last**, so the operator meets the read-only surfaces
first, and Objects before Signatures.

**Window is ordered in two tiers**: the reversible remedies first, and the one
that discards the operator's own arrangement — Reset layout — last. The two
auto-hide toggles sit after Dock all panels and before Reset layout.

**Neither auto-hide toggle renders pressed**, and that is the convention rather
than an omission: Office's own ribbon-display control is a caret that opens a
menu, not a lit toggle. The surface that carries the *state* is the Settings
window, where both are checkboxes.

---

### 5.3 Pages — *what am I doing to the set of pages?*

| Group | Commands |
|---|---|
| **Insert** | Insert blank · Insert from file… · Insert scan |
| **Clipboard** | Cut · Copy · Paste |
| **Organise** | Delete · Extract… · Replace… · Move up / Move down · Split… · Merge into this document… |
| **Transform** | Rotate left / right · Crop… · Resize… |
| **Stamp** | Watermark… · Header & footer… · Bates numbering… |

**Organise is ordered Delete, Extract, Replace, Move, Split, Merge** — merge
last. The order is the spec; a build shows that list with its unbuilt rows
absent.

Every command here operates on **the current document's page set** and respects
the thumbnail rail's current selection when there is one. That is the tab's
organising rule and it is what distinguishes it from Tools: **Pages changes
*this* document; Tools produces *new* files.**

The thumbnail rail keeps its selection action bar. That is not a P1 violation —
the rail is a panel, not a tab, and a selection-scoped action bar next to the
selection is correct.

---

### 5.4 Edit — *what am I changing about content that is already there?*

| Group | Commands |
|---|---|
| **Content** | Select all · Edit text · Add text · Reflow block |
| **Insert** | Image… · Attachments · Shape ⌄ |
| **Arrange** | Align ⌄ · Distribute ⌄ · Bring forward / Send backward · Group / Ungroup · Flip horizontal / vertical |
| **Clipboard** | Cut · Copy · Paste · Paste in place · Copy as vector · Duplicate |
| **Forms** | Create field ⌄ (text, check box, radio button, choice, push button) · Manage fields · Flatten |
| **Protect** | Redact ⌄ (mark page / by text / by pattern) · Redact selection · Off-page · Apply redactions · Sanitise… |

**Reflow follows the tool that does the retyping**, not the tool that selects:
an operator reflows a paragraph because they have just retyped a sentence in it.

**The two pastes share one glyph deliberately.** A second paste picture would be
a distinction the operator has to learn for no gain; the two are told apart by
their labels and by the chord in the tooltip, which is how Word and Acrobat tell
their paste variants apart too.

**Copy as vector is icon-only.** A fifth long label widens the group past its
neighbours.

**Not here:** *Copy page text* / *Copy document text* are `file.copy_*` on File ▸
Export, and *Fill form* is `view.panel_forms` on View ▸ Panels — both by §2's
mode rule, spelled out in §7. Edit ▸ Forms keeps create, manage and flatten. The
tool that edits an object's own points is `view.tool_node` on View ▸ Navigate,
because it arms the canvas and every canvas tool lives beside Select, Text and
Hand.

**There is no `Editing on` master toggle**, and there is not going to be one. No
mainstream editor has a global editing switch: in Acrobat, Bluebeam, Word and
Illustrator alike, selection and Delete are always live and picking a tool arms
*that tool* until Escape or another tool. There is no state in which a click
does nothing without the application saying so. Nothing replaces the gate: an
unarmed canvas already does modeless select-and-delete, and every authoring
gesture already requires its own tool to be armed. If a genuine read-only mode
is ever wanted it is a **document** state — opened read-only, encrypted, or
signature-locked — with a visible badge, not a hidden global toggle.

---

### 5.5 Markup — *what am I adding for someone else to read?*

| Group | Commands |
|---|---|
| **Shapes** | Rectangle · Ellipse · Line · Arrow · Polyline · Polygon · Cloud · Ink (freehand) · Finish |
| **Text markup** | Highlight · Underline · Strikeout · Squiggly |
| **Notes** | Text box · Sticky note · Callout · Stamp ⌄ |
| **Style** | Colour · Line width · Fill · Opacity · Line style |
| **Arrange** | Bring to front · Bring forward · Send backward · Send to back |
| **Comments** | Comments panel · Clear page · Clear all |

Cloud — revision clouds — is AEC table stakes and is the one this audience names
first.

**The Style group is a pen: it sets the style of the NEXT mark, not of the
selected one.** Changing a mark already placed happens on the contextual
**Format** tab (§5.8). Both surfaces must exist, and the pen/mark distinction is
what keeps them from collapsing into one.

#### Why Arrange is a group of its own and not four rows inside Style

Style and Arrange answer **opposite questions about tense**. Style is the pen;
these four act on a mark that is **already placed**, are drawn only while one is
selected, and reach a different engine verb entirely — `reorder_annotations`,
which permutes the page's `/Annots`, where Style reaches `set_markup_style`.
Folding them into Style would put a live control and a greyed one under one
caption whose word describes neither.

**Placed after Style and before Comments.** The tab reads left to right as a
sequence of tenses, and this is the seam between its two halves:

| Groups | Tense |
|---|---|
| Shapes · Text markup · Notes | what I am about to add |
| Style | how the next one will look |
| **Arrange** | **what I have already added** |
| Comments | what everyone has added |

Before Style would split the three authoring groups from the pen that governs
them. After Comments would put a selection-scoped group after a document-scoped
panel toggle, which is the widening this order otherwise never reverses.

**The four are labelled, and that is a refusal rather than an omission.** There
is no front, back, arrange or stacking glyph in the icon set, and both
near-misses are refused: `chevron-up` and `chevron-down` already mean *move this
up the page list* — they are `pages.move_up`'s glyphs, so borrowing them would
tell an operator that Bring forward reorders the document; `show-points` and
`edit-objects` depict a shape's anatomy, not its depth, and a picture that
describes the wrong operation is worse than a word. With no glyph to name,
asking for `icon_only` would fall back to labels anyway, so requesting it would
be a line of code stating an intention the build does not honour. The four words
are the ones every drawing program uses.

**Two rows, not four** — a `prefer_rows` hint, which the packer reads as *skip
the fits-already short-circuit and search*, then returns the narrowest packing
within the band's row ceiling. Being a hint and not a layout, the item order is
what decides which controls share a row: the two that move a mark **forward**
come first and the two that move it **back** follow, the reading that survives
whichever shape the packer picks. `RIBBON_SCALING.md` holds the packing rules.

**The ids are `markup.*` although the group is called Arrange.** A command's
handler token must sit inside the hundred belonging to its id's prefix, and
*Arrange* names a **group**, not a tab. §5 names groups freely and ids by tab —
`markup.highlight` is in Text markup, `file.encrypt` in Security — so the id
prefix says which tab, the group name says which band, and neither is an
abbreviation of the other.

#### Two groups are captioned Arrange, and neither moves

§5.4's Edit tab has a group captioned **Arrange** too, and two of the four
command names are the same words. They are not duplicates in the P1 sense,
because their subjects are disjoint:

| | Edit ▸ Arrange (§5.4) | Markup ▸ Arrange (§5.5) |
|---|---|---|
| acts on | the **drawing's own** objects — page content | a **markup annotation** you placed |
| what it reorders | paint order inside the content stream | the page's `/Annots` array |
| engine verb | none | `reorder_annotations` |

**The captions are deliberately not disambiguated.** Renaming either to *Arrange
marks* or *Arrange objects* would be a caption explaining the ribbon to itself.
The contextual rule already separates them — one appears on Markup beside markup
tools, the other on Edit beside content tools — and no operator sees both at once
with one selection.

---

### 5.6 Measure — *what am I measuring, and in what units?*

| Group | Commands |
|---|---|
| **ce dimension** | Linear · Aligned · Angular · Radius / Diameter · Two-line · Perimeter · Length · Finish |
| **Quantity** | Distance · Area · Count |
| **Scale** | Set scale · Calibrate from a known length · Manage ce-dimension groups… |
| **Takeoff** | Schedule panel · Export CSV |

The **Scale ▸ group** model — named groups carrying a shared scale and drafting
standard — is better than what the comparison product does, and nothing here
should dilute it. Area and Angular are the conspicuous absences for anyone doing
takeoff on a drawing.

Aligned is a constraint on the linear tool rather than a separate tool.
Perimeter and Length are in the ce dimension group rather than in Quantity
because they author a mark on the page; Quantity holds the readings that author
nothing.

**Everything on this tab authors a ce dimension**, which is the dimension
pdfcer draws and owns. A **pdf dimension** is page content a CAD exporter wrote
— pdfcer reads it, renders it and measures against it, and no command here
changes one. The group's caption in the band is the single word *Dimension*,
because the tab has already said which kind; the prose never leaves it to be
inferred.

---

### 5.7 Tools — *what do I run across files, or configure once?*

| Group | Commands |
|---|---|
| **Batch** | Merge files… · Split files… · Batch print… |
| **Compare** | Compare documents… |
| **Fonts** | Font folders… · Embed fonts · Unembed fonts |
| **Validate** | PDF/A validate & convert… · Optimise… |
| **Diagnostics** | Render diagnostics |

Tools is the tab for things that either operate on files other than the open
one, or are configured once and rarely touched. Redact is on Edit ▸ Protect,
where an operator editing a document looks for it. OCR is on File ▸ Recognise
(§7).

---

### 5.8 Format — *contextual*

Appears only while something is selected and disappears on deselect. Its
`visible_when` is `selection.formattable` — the union of an object selection and
a live text selection — because the tab has two kinds of subject and neither
operand alone is its question.

Contents vary by selection type:

| Selection | Groups |
|---|---|
| Text run | Font · Size · Bold · Italic · Colour · Delete |
| Markup | Colour · Fill · Line width · Line style · Opacity · Arrowheads · *Note text* · Delete |
| ce dimension | Group · Scale · Precision · Units · Standard · Witness lines · Delete |
| Image | Size · Position · Crop · Opacity · Replace · Delete |
| Vector object | Stroke · Fill · Winding rule · Node tools · Delete |
| Pages (rail) | Rotate · Delete · Extract · Move |

Band order on the tab is **Font, Markup, Selection**. Every row above ends in
Delete, so reading left to right goes *change how this looks*, then *describe
it*, then *destroy it* — increasing commitment, which is the ordering rule the
Selection group already follows internally.

This tab is what makes selection *mean* something: without it, selecting an
object gives an object-tree row and no way to act on it.

**Where it appears, and what it must never do when it does.** A contextual
tab is appended **after every fixed tab**, and appearing **never activates
it**. Both halves are load-bearing and neither is cosmetic: a tab that
inserted itself among the fixed ones would move the others under a pointer
that was already travelling toward one, and a tab that activated on appearing
would replace the band an operator is working in **because they selected
something** — which is to say, every time they touch the page. The active
tab therefore only ever changes when the operator changes it, or when the tab
they were on stops being visible; in that second case the fallback is the
leftmost **fixed** tab, which is reachable precisely because the contextual
ones sort last.

The same rule governs any contextual tab a manifest adds later — it lives in
`egui-shell`’s ribbon, not in anything that knows what Format is.

#### The Font group

Order within the group is Word's: face, size, a rule, then Bold, Italic, colour.
The rule separates *which typeface* from *how it is set*.

**Bold and Italic are in.** Word's Font group is unusable without them. They
pass the build-order test below because the Properties panel's *This text*
section already carries them, so the tab's contents remain a subset of the
panel's.

**Spacing and Alignment are out, and it is not a scheduling deferral.**
`EditSession` has no verb for either: `format_text` sets face, size, weight and
fill and nothing else, so nothing writes `Tc`, `Tw` or `TL` for an existing run.
Alignment is worse than missing — it is not a property a PDF text run *has*, it
is a consequence of where each show operator was positioned, so re-aligning
existing text means re-laying it out.

**Grow and Shrink are refused although Word has them**, for the build order's own
reason: they exist in no panel section, so adding them here would make the tab a
**superset** of the panel, which is exactly the writing-the-editors-twice the
build order exists to prevent.

**Two conditions carry it, and the split is R9 rather than convenience.**
`mode.edit_content` is each item's `visible_when`, so the whole band is
**absent** in Read and Review, which cannot change page content at all;
`selection.text` is each command's `enabled_when`, so inside Edit the controls
**grey** until something is swept, with a tooltip that says how to sweep. That
greyed state is deliberate: it is the only surface that can tell an operator to
press `T` first.

#### The Markup group

**Style is the pen; this band is the mark you already made.** That is the whole
division between it and §5.5's Style group.

*Note text* is not on the tab and lives in the **Properties panel** instead:
`/Contents` is `MarkupNote`'s, a different struct behind a different verb, and a
note is **prose** — a ribbon band is the wrong surface for a paragraph. §5.8's
own division of labour puts everything not reached for mid-gesture in the panel.
*Delete* is the Selection group's, exactly as it is for every other row of the
table above.

**Two controls exceed the panel, and the build order permits it.** The panel's
*This mark* section offers colour, width and opacity; this band adds **Fill** and
**Arrowheads**. That is not the superset the Font group refuses, because these
two are written **once**, here. The panel deliberately declines Fill
(`canvas::markup::spec` authors `interior: None` so a comment does not hide the
drawing under it) and declines Arrowheads (they mean something for `/Line`
alone). Both are offered here as *restyling an existing mark* rather than as a
change to what pdfcer authors, and neither changes the pen.

**The Line style control can be honest because the engine preserves a dash it
did not author.** `/BS` `/S` and `/D` are read back on the way in, and a restyle
that does not mention `dash` keeps the one already there — so the chooser can
show *the file's own pattern* as a state and mean it: leaving the control alone
genuinely keeps it. Over an engine that solidified a dash on every restyle, this
control would be one its own neighbours undid.

**Whether a control applies is asked of the engine, not hard-coded.**
`MarkupStyleSupport::for_subtype` answers it before the press — a width on a text
markup is refused rather than silently discarded — so neither surface needs its
own knowledge that a highlight has no border.

**The arrowhead chooser offers four POSITIONS, not nine pairs.** `/LE` is two
independent endings over three shapes each. The control asks *which ends* and
**preserves the shape the mark already carries**, so a closed arrowhead stays
closed. A chooser that normalised every arrow to `/OpenArrow` would silently
rewrite a producer's `/ClosedArrow` — invisibly here and visibly elsewhere.

**One condition carries it, and the split from the Font group's two is R9.**
`selection.markup_restylable` is each item's `visible_when` **and** each
command's `enabled_when`: a markup annotation is selected and the mode may author
markup. The Font group greys because a greyed control there is the only way to
say *press T first*; a greyed Markup control could only say *select a mark*,
which the operator has already done or the contextual tab would not be drawn. R9
then requires **absence** — in Read, and with any other kind of selection.

**Greying is left for the lock alone.** The locked bit is a fact about one
annotation rather than about the build or the mode — click a different mark and
the controls work — so a locked mark greys, with the Properties panel's own
sentence, rather than making the band vanish.

**A ce dimension is not a markup, and the guard is in the type.** A ce dimension
is `set_dimension_style`'s, and its `/Subtype` is `/Line`, identical to an
arrow's — so the guard is a `match` on `AnnotKind` that the compiler checks, in
the condition and again where the operand is built. A string comparison would
restyle the operator's ce dimensions into bare lines and look correct doing it.

#### Both surfaces, not one

The contextual tab and a **persistent properties panel** both ship. They are not
redundant; they answer different questions.

| | Format tab | Properties panel |
|---|---|---|
| Lives | in the ribbon, appears on selection | right dock, always available |
| Holds | the edits reached for mid-gesture | the complete property set |
| Survives a tab switch | no | yes |
| Costs | nothing when nothing is selected | about 200 px of width |
| Discoverable | high — it appears, which is itself the affordance | medium |

The **tab** carries what an operator changes *while working* — colour, width,
style, align, delete. The **panel** carries everything, including the read-only
facts (winding rule, node count, embedded-font status, exact geometry) that
belong beside the Objects panel's inventory rather than in a ribbon band. The
panel is also where the **editable geometry** lives — X, Y, W, H as typed values
— which is how `/Rect` move-and-resize becomes reachable without a drag, and
that matters where the number is known and the mouse is imprecise.

**Build order: panel first, tab second.** The panel is the harder half and the
tab's contents are a subset of it, so building the tab first means writing the
property editors twice.

A third surface, the **context menu**, carries the same commands again for the
operator who right-clicks. That is not duplication in the P1 sense — context
menus are not tabs — and it is the path most users try after the keyboard.

---

## 6. Surfaces that are not tabs

**Quick Access Toolbar** — Open, Save, Undo, Redo. The handful of controls that
must never sit behind a tab switch: verbs used continuously, on the left, where
reading starts. A P1a shortcut surface, not a home. The second slot is `Save`
rather than `Save a copy` because a quick-access Save that opens a file dialog is
not the control a returning operator's hands are reaching for. The four are
pinned by a test: adding to this surface is a decision, and drifting into it is
not.

**Status bar** — Find toggle, actual size, fit width, fit page, zoom −/%/+,
page ◀ n/N ▶, and an editable page-number box. The controls an operator touches
constantly, where they never disappear behind a tab change. Render diagnostics
sits behind a disclosure here.

**Trailing region** — the fourth region of the tab-strip row, past the mode
selector:

```text
[QAT] │ File View Pages ⏷ 2 more   ( Read │ Review │ Edit )   [Acrobat]
                                                              └ trailing ┘
```

> A trailing control **ends the current activity** rather than advancing it.

*Open in Acrobat* qualifies exactly: it closes the document and hands the file to
another program. A control reached for *during* the work belongs on a tab, or on
the QAT if it is used all day.

`file.open_in_acrobat` is on **no** tab, and the trailing region is its only
home — which is what makes it unique here. Undo and redo are on no tab either
(§7), and six commands are reached from a context menu alone. Its home is a fixed
position in chrome visible in every mode, which is a *stronger* discoverability guarantee than a tab gives;
putting it on File as well would give two places to press for one act, one of
which comes and goes with what is installed.

P3 applies here with full force, and it is why the region holds manifest `Item`s
resolved through the command registry rather than a callback: on a machine with
no Acrobat the control is **absent**, through `visible_when`, and nothing can
draw a control here that has no command behind it.

It is also the one reserved region on that row that may be **squeezed out**. The
QAT, the mode selector and the two overflow affordances are promises the
interface has already made; a trailing control's absence is a state the
application already handles, so on a very narrow window it is dropped whole —
never a sliver — and the drop is disclosed as `ribbon-trailing-dropped`.

**The left rail** — the icon strip down the dock's left edge. It is a panel
strip, not a tab, so everything on it also has a tab home; what it adds is a
route that survives a mode. Four bands, in this order:

| Band | Holds |
|---|---|
| *(uncaptioned)* | the panel toggles: Pages, Bookmarks, Layers, Signatures, Comments, Fonts |
| **Navigate** | the five canvas tools — Select, Points, Text, Hand, Smart select |
| **Select** | Select all |
| **Rotate** | Rotate left, Rotate right |

The uncaptioned band is the rail's own tab strip and carries no caption for the
same reason a dock's tabs carry none. The Navigate band folds only while a tool
is armed; Select and Rotate fold whole.

**The rail is the only route to `markup.comments` in Read**, whose tab list is
`["file", "view"]` and therefore has no Markup tab. That is why a hiding rail is
never wholly gone — see `MODES_AND_PANELS.md`.

**Context menus** — a menu is a shortcut surface, not a tab, so a command on one
still has its tab home; what a menu adds is the path most operators try after
the keyboard, and it is the other half of making selection mean something. Each
menu carries the commands its selection's Format row carries, plus cut, copy,
paste and delete where they apply. Ten contexts are in the manifest's `menus`
block:

| Context | Commands, in order |
|---|---|
| `canvas.object` | Zoom to selection · Properties · Select this line of text *(when a run pick is offered)* · Select the form · Give this page its own copy · Mark this for redaction · Delete *(when deletion is permitted)* |
| `canvas.read-object` | Copy · Zoom to selection |
| `canvas.empty` | Fit page · Fit width · Fit height · Actual size |
| `canvas.field` | Properties · Delete *(when permitted)* |
| `canvas.markup` | Properties · Add a point here · Remove this point *(each when the mark offers it)* · Cut · Copy · Paste · Delete *(when permitted)* |
| `canvas.text` | Reflow paragraph |
| `dock.tab` | Float panel *(when docked)* · Dock panel *(when floating)* · Close panel · Reset layout |
| `document.tab` | Close · Close others |
| `objects.row` | Properties |
| `pages.row` | Move up · Move down · Extract · Rotate left · Rotate right · Delete |

Six registered commands are reachable from a menu and from nowhere else —
`view.panel_float`, `view.panel_dock`, `view.panel_close`, `markup.add_node`,
`markup.remove_node` and `format.select_text_line`. **Each one's operand is the
thing the operator gestured at**, and a ribbon button has no such operand: by
the time it is pressed the pointer is on the ribbon, so the button would have to
invent a subject and would then act on something other than what was pointed at.
The `TAB_SCOPED` register carries one reason per id, so "on no tab" stays a
decision rather than an oversight, and a test walks it in both directions.

Discoverability is answered one rung up instead: View ▸ Window's *Dock all
panels* and *Reset layout* teach that panels float and that there is a way back,
and View ▸ Navigate's *Points* teaches that a drawn shape has corners to aim at.
`format.select_text_line` is the exception and says so — this menu is its only
route, and a ribbon home is owed the day the control has a subject it can name.

**The keyboard** — the manifest's `keymap`, one chord to one command id. It is
a shortcut surface in P1a's sense: a chord never gives a command a second home,
and a command with a chord still lives on its tab. The bindings are not listed
here because the list would drift; `built_in.ron`'s `keymap` block is the
register, and the *Keyboard shortcuts* window on File ▸ pdfcer is a fold over
that same map, so a binding that exists is listed by construction. A chord
naming a command this build did not register is **counted and not listed** (R8:
an absent capability is expressed by an absent registration, and listing the key
would promise a keystroke that does nothing); the count of drops is disclosed,
because a stripped build genuinely has fewer shortcuts.

**Tool Options pane** — per P2, every armed tool's parameters.

---

## 7. Placements that get re-litigated

These are the placements a reader keeps wanting to move back. Each has been
decided and each decision holds.

| Command | Lives on | Not on | Because |
|---|---|---|---|
| `file.copy_page_text`, `file.copy_document_text` | File ▸ Export | Edit ▸ Clipboard | copying is not authoring — it reads the page and writes the clipboard, and cannot change a byte. Read, whose standard is Acrobat Reader, copies text |
| `view.panel_forms` | View ▸ Panels | Edit ▸ Forms | filling is not authoring, and it is a panel toggle rather than a verb, so it sits with the other panel toggles |
| `file.ocr` | File ▸ Recognise | Tools ▸ Recognise | OCR must be reachable in Read and Tools is not in Read's tab list. File over View because OCR's product is a new file. Giving Read the whole Tools tab was the alternative and was refused: it would hand a reading stance batch merge, batch split and font embedding |
| `view.reset_layout` | View ▸ Window | File | it resets panel geometry, which is a view concern (§3) |
| `file.fonts` | File ▸ Document | View ▸ Panels | the Fonts panel answers *what is inside this file*, not *what is on my screen*, so it sits with Properties (§5.1). The left rail carries it too, and a rail is a panel strip rather than a tab |
| `markup.comments` | Markup ▸ Comments | View ▸ Panels | comments are markup someone else reads, so the panel toggle stays with the marks it lists. It is on the left rail as well |
| `edit.undo`, `edit.redo` | the QAT alone | any tab | mirroring them onto every tab is what made the band render only the active tab and left undo unreachable (§2). The `edit.` prefix says which tab they would take if they ever got one; no tab is safe for them only because the QAT is always visible |

Three commands sit where a first-time operator looks rather than where a
returning one remembers: *Copy text*, *Redact* (Edit ▸ Protect) and *Rotate page*
(Pages ▸ Transform). That is the trade this layout accepts.

---

## 8. Open questions

1. **Autosave** — in-place `Save` ships and the format carries the previous
   revision, so a recovery timer is an addition rather than a prerequisite. Is
   one wanted, and does `file.revert` mean *back to the last save* or *back to
   the revision the file was opened at*? The two are different commands.
2. **Compare** — worth building, or out of scope? It is the one absence an AEC
   reviewer names first, and it is a large build.
3. **Multi-run text editing** (`GUI_ROADMAP.md` Phase 5d) — is *edit one run at
   a time, clearly disclosed* an acceptable resting state, or is this the thing
   that has to be right?
4. **Column-major groups.** The mockup stores a group's items as an array of
   columns and lays each out as a flex column, so an operator reads *down* the
   first column and then *down* the second; `wrap_group` partitions the same
   items into contiguous **rows**, so the operator reads *across*. Both produce a
   block of the same footprint and the item in each cell differs. The column
   split does not exist in `built_in.ron`, so matching the mock means changing
   `egui_shell::manifest::Group`, the RON generator, `measure_group` and
   `captioned_group` — and the mock is not self-consistent about it either. It is
   the largest open ribbon question.
5. **Two controls on the View tab draw the same word.** `view.tool_node` in
   Navigate and `view.show_points` in Display are both labelled *Points* — one
   arms a tool, the other toggles whether points are drawn for parts other than
   the selected one. Only the tooltips tell them apart. Either one is renamed or
   the pair is stated to be deliberate.
