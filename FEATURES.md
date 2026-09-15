# pdfcer-gui — what is built, and what is next

**Updated:** 2026-09-15, 04:55 EDT (**thirty-fifth revision** — re-measured against the build about to ship, engine **`pdfcer-core` v0.53.0**, a **git dependency on the local engine repository**, pinned at **`b2f54228`**, which is the tip of its `main` — level in documentation and level in every line of code, both read out of the lock and out of the engine's log for this header rather than carried from the paragraph below. ★★★ **AND THIS ONE HAS SOMETHING IN IT YOU CAN SEE, which the last one did not: you can drag ONE LINE of text on its own.** Pick a line inside a block of text, drag it, and only that line moves. The lines under it stay where they are. Until this build the whole text object moved together, so nudging one line of a title block dragged the three lines below it along with it. ★★ **And the route to it no longer has to be guessed:** right-click on a line and the menu names that line rather than the block it sits in. ⚠ **Where it still cannot, it now tells you WHICH of two reasons applies**, because the two need different things from you: either the line carries no position of its own and inherits one from the line above it, or moving it would drag the next line with it. One sentence for each. The old behaviour was one vague refusal covering both, which is the same as no answer.)

This is the per-surface capability register for the new shell — what an
operator can reach in a real build, and what is planned next — and it is the
acceptance contract every claim about this project is measured against.

Scope is the shell only. `pdfcer-core` and `pdfcer` capabilities live in
`D:\Dev\pdfcer\docs\FEATURES.md`, whose **`gui` column is this project's
acceptance criteria**: nothing there may regress at fold-in.

Two lists. The first is what works **today, in the running binary**. The
second is what does not, **in the order it is planned**.

**A row is ticked only when an operator can reach it in a real build.** Not
when the code exists, not when a test passes. A panel can ship with a body, a
rail entry, a diagnostic step, no control anyone can click, and a green suite
for the whole of its life; that is what this bar exists to prevent.

**Legend:** ✅ reachable · 🔨 in progress · ◑ partly reachable · ⬜ planned ·
⛔ blocked, with the blocker named · ❌ not planned · ⚠ a caveat or a known
gap attached to the row it follows.

---

## Where it stands

Every figure here moves, so each row carries the command that produces it
rather than the number it produced last.

| | |
|---|---|
| **Stages complete** | S0 skeleton · S1 `ui-verify` · S2 ribbon · S3 panels and dock · S4 selection · S5 salvage (print, forms, icons, settings) · Phase 3 navigation, Find, thumbnails, rulers, grid, guides · Phase 4 page-display modes · Phase 5 text editing · Phase 1 transform, complete but for the cross-process clipboard |
| **Tests** | `cargo test --workspace`, summing every `test result` line; cross-count the total with `cargo test --workspace -- --list` filtered to its test lines |
| **Driven checks** | `cargo run --release -q -p ui-verify -- --list` — the binary's own roster, never a grep over its source. A full sweep takes about 95 minutes and aborts if any `.rs` or `.toml` is edited while it runs; Markdown edits are safe |
| **Gates** | `bash tools/gates/run-all.sh`, or `--no-cargo` for the fast subset. Exit `0` pass, `1` fail, `3` a gate was skipped, and a skip is not a pass. Every grep-over-source gate carries `--self-test`, which plants a violation and asserts the gate catches it |
| **Source** | `wc -l` over `git ls-files '*.rs'`, split by crate. The harness is a large and deliberate share of it: a phase is not done until a check drives the real binary, so every capability carries harness code the way it carries tests |
| **Commands** | the shipping binary's own start-up trace line, `pdfcer-diag shell commands=… planned=… directed=…`, on an off-screen smoke launch |
| **Engine** | the pin in this file's header, and the single `pdfcer?branch=main#<sha>` revision in `Cargo.lock`; `tools/gates/check-pin-citation.sh` asserts they agree and fails if the header phrase goes missing. Every sentence of the form *the engine cannot do X* is a claim about that pin |
| **Panels** | `Panel::ALL` in `crates/pdfcer-gui/src/panels/mod.rs` |
| **Ribbon surface built** | `grep -c 'caption:' crates/pdfcer-gui/src/shell/ron/built_in.ron` — captioned groups; the tabs themselves are in the same manifest |

---

## Complete

### Shell and chrome

- ✅ **A dialog is a real OS window** — Print opens as its own movable,
  resizable top-level viewport rather than a modal trapped inside the frame
- ✅ **And so are the other thirteen** — About, Render diagnostics, Export to
  DXF, Insert image, Insert pages, New document, Recognise text, Apply
  redactions, Set scale, Keyboard shortcuts, Settings, the note editor and the
  unsaved-changes prompt
- ✅ **Sign a document** — `pdfcer_core::sign` through File ▸ Security, and the
  signature is read back out of the saved **file** rather than out of the session
- ⬜ **Put a password on a drawing, and choose what it says it allows** —
  File ▸ Security, immediately after Export. `Encrypt…` sets the open password
  and the permission flags. **Built and undriven**
- ✅ **Ribbon, seven tabs** — File · View · Pages · Edit · Markup · Measure ·
  Tools, plus the contextual **Format** tab that appears on selection
- ✅ **The ribbon is data, not code** — a serializable manifest, which is what
  makes it customizable *and* reusable by another application
- ✅ **Mode selector** — Read / Review / Edit, right-aligned on the tab row,
  driving both the tab set and the panel layout
- ✅ **Quick access toolbar** — Open, Save a copy, Undo, Redo, icon-only
- ✅ **Open, Recent, Close** — a second document can be opened, and `Ctrl+O`
  resolves through the manifest keymap like every other chord
- ✅ **Keyboard chords derive from the manifest keymap** — `keyboard.rs` spells
  the key, looks it up and returns a command id through the same dispatcher a
  ribbon click reaches, so a customization layer rebinds it with no code change
- ✅ **`Ctrl+1/2/3` switch mode**, `Ctrl+0` is actual size — one owner per
  chord, with a test that fails naming any chord claimed twice
- ✅ **Every ribbon control that can be ON renders pressed** — including the
  hand tool and marquee zoom, whose armed flag lives in `egui::Memory`, which
  is why `conditions()` takes the `Context`
- ✅ **Group captions, enforced by construction** — one closure draws every
  group, so a caption cannot be omitted
- ✅ **Band overflow** — reserved-space "⏷ N more", hit-testable at a width
  narrow enough to hide groups
- ✅ **Tab-strip overflow** — the active tab is pinned; below about 47 pt the
  strip collapses to the affordance rather than hiding the tab you are on
- ✅ **Status bar** — render-notes disclosure (closed by default), Actual size /
  Fit width / Fit height / Fit page, zoom −/%/+, page ⏴ n/N ⏵, and the
  **Flip pages** wheel toggle beside them on single-page displays
- ✅ **A PDF that contradicts itself opens, and says what pdfcer decided** — a
  file may name one key twice in a dictionary with two different values, and
  the resolution is reported on the status bar rather than taken silently
- ✅ **Find** — `Ctrl+F` and a status-bar toggle open a floating box at the
  page's top right: query, `3 of 47`, Enter / Shift+Enter to step, hits
  highlighted with the current one distinguished, and Match case · Whole word ·
  Wildcards behind an Options disclosure
- ✅ **Editable page box** — type `37`, press Enter. Commits on Enter or focus
  loss, clamps out of range and says so, rejects non-numeric without discarding
  what you typed
- ✅ **Three themes**, installed by `Theme::apply`, with a rendered-pair
  contrast gate over all five widget states
- ✅ **Closing a document asks about unsaved edits** — `Action::CloseDocument`
  routes through the prompt `file.close`'s tooltip promises
- ✅ **Every setting the engine carries has a control, and a gate says so** —
  `pdfcer-core` gains settings on its own schedule, so the coverage is asserted
  rather than remembered

### Panels and dock

- ✅ **Dock** — multiple columns per side, vertical stacks, tabbed groups,
  draggable splitters
- ✅ **Tab overflow** — reserved-space menu, and no per-group pane cap: nine
  panels in one stack, tested
- ✅ **Layout persistence** — `<settings dir>/layout.ron`, debounced 750 ms
  with a 5 s ceiling, per-item fail-soft loading
- ✅ **Named workspaces**, and **a mode is a workspace** — leaving Edit and
  returning restores your arrangement, not a default
- ✅ **Scoped reset** — right dock alone, left alone, or all
- ✅ **Bookmarks land on the detail, not merely the page** — `Destination::Page`
  carries a `page_index` **and** a `view`, and the panel honours both
- ✅ **A clickable table of contents works** — a `/Link` in Read or Review
  navigates page, zoom and scroll through the same `destination` actions a
  bookmark uses
- ✅ **A bookmark can be written** — `EditSession::add_outline_item`
- ✅ **…and renamed and removed** — clicking a row opens a Selected bookmark
  block above the list, with its name and a Remove
- ✅ **Tool status** — a one-line strip in the right dock's chrome naming what
  is armed at frame one; it cannot be closed and is drawn in every mode. Its
  live controls are in Properties, and it carries no tool list
- ✅ **Dimension groups** — a dock panel with six foldable sections, scrollable,
  so no part of it can push its own title bar off the desktop
- ✅ **Attachments** — `attach_file`, `detach_file`, `list_attachments` and
  `extract_attachment`, each with a control
- ✅ **Layers** — full `/RBGroups` radio semantics, locked-member handling, and
  Reset to the document's own default
- ⬜ **Signatures — three facts, reported separately, and "not checked" says
  so.** **Built and undriven**: `trust_store` has never been run
- ⬜ **…and the anchors are the operator's own Acrobat trust list, opt-in and
  off by default** — Settings ▸ Digital signatures. **Built and undriven**
- ✅ **Fonts** — inventory, embed status, byte cost, font-folder resolution, and
  the two verbs it reports on: Tools ▸ Embed fonts and Tools ▸ Remove embedded
  fonts
- ✅ **Objects** — every object on the page, front-most first; 129,758 of them
  on the benchmark drawing
- ✅ **Properties** — facts for a selection, plus the editable sections named
  under Canvas below
- ✅ **Pages** — thumbnail grid, click to navigate, multi-select (click /
  Ctrl+click / Shift+extend), context menu of the six page verbs. Selection is
  marked by a *shape* change plus a written count, never colour alone
- ✅ **The page-previews tick is the operator's alone** — a page costing more
  than the limit no longer switches previews off behind your back, and the
  limit itself is a box you can type in
- ✅ **Drag a page to where it goes, and see where that is before you let go** —
  a caret between tiles, rather than two arrows that move one place at a time
- ✅ **Comments** — every annotation, page order then `/Annots` order.
  `/Widget` is excluded because Forms owns it, `/Popup` because it is an
  implementation detail of another annotation
- ✅ **…and the Comments panel writes** — Add note / Edit note / Remove note on
  every row, plus delete, filter by author, by kind and by has-words, sort, and
  a **Go to** that navigates and opens the comment's pop-up on the same frame
- ✅ **A stamp's text size is the operator's to choose** — a Size chooser beside
  the gallery: *Fit the box I drew*, then 8 to 72 pt, with the box grown to
  hold what was asked for
- ✅ **Markup can be authored see-through** — an Opacity control on
  Markup ▸ Style, applied to every kind including the sticky note, the text box
  and the stamp
- ✅ **Forms can be authored, not only filled** — five commands on Edit ▸ Forms,
  one per kind the engine has a verb for: text field, check box, radio button,
  drop-down, push button
- ✅ **Clicking a field on the page opens its properties** — in Edit a click
  selects the field; in Read and Review it fills it
- ✅ **Forms — fill, in all three modes including Read**, reading the session so
  unsaved edits show. Filling changes no page content, which is why Read keeps it

### Canvas

- ✅ **The font offer tests the character you could not type, not the words
  already on the page** — `preview_font_resources_for`'s candidate,
  `FontPreflight::standard_14` and `FontAcceptance` decide the list from the
  keystroke that was refused
- ✅ **A key the run's font cannot carry is refused as you type it, by the name
  of the character** — `run_repertoire`, so a subset font declines at the
  keystroke rather than at the save
- ⚠ A test that re-implements the chooser and asserts on its copy proves
  nothing; both font-chooser tests call `choices` itself
- ✅ **A field too small for its text says the text will overflow** —
  `FillOutcome::applied_autosize_bound` names which constraint bound the size
- ⚠ `AutoFitBound` is `#[non_exhaustive]`: a new upstream variant is a compile
  error here only where the match is exhaustive, so the wildcard arm carries the
  honest sentence
- ⚠ An absence claim is a claim about every route. Grep for the sentence that
  denies a capability before believing it, and again after wiring the verb
- ✅ **A click on blank paper starts new text** — `NoRun` becomes an origin
  instead of refusing; only an encrypted document still declines, because that
  says *this cannot be done here* rather than *there is nothing here*
- ✅ **A text file becomes pages** — `file.import_text`
- ⚠ R2 (no source file over 1,500 lines) is a build gate, not advice:
  `check-file-size.sh` fails the commit
- ✅ **The line-weights mode reports when it has nothing to do** —
  `Diagnostics::strokes_hairlined` distinguishes *thinned nothing* from
  *did not run*
- ⚠ A driven check reporting SKIP is not green and not red; it measures nothing,
  so a SKIP is treated as a failure to run
- ✅ **Rotation composes, the angle is typed, and the engine owns both** — the
  grip and the typed angle raise the same verb
- ⚠ `Annotation::appearance_rotation_degrees` returns a **signed** `atan2`: a
  quarter turn clockwise reads `-89.15`, not `270.85`. Normalise before display
- ✅ **A sticky note keeps an icon name pdfcer does not model** — §12.5.6.4's
  seven names are a standard set, not a closed one, so a producer's own name is
  carried through rather than replaced
- ✅ **A selected mark's outline is drawn at its own angle** — the dashed
  outline, the eight scale grips and the rotate handle all turn with the artwork
- ◑ **A typed angle in the Properties panel** — degrees anticlockwise under
  Width and Height, seeded from the appearance `/Matrix` so it survives a
  save-and-reopen. Read driven, write undriven
- ⬜ **Rotating a mark twice makes it bigger** — an engine defect, reproduced in
  pixels; filed
- ✅ **Annotations and ce dimensions rotate** — a ninth grip, a circle on a stem
  above the selection box, Shift snapping to 15°. A ce dimension gets the circle
  and nothing else, because its size is the measurement
- ✅ **Clicking an object inside a form XObject selects that object** — the hit
  test descends into the block rather than returning the block
- ✅ **"Select the form"** — the deliberate second act, because the hit test now
  excludes forms outright and a form is still a legitimate thing to select
- ✅ **The status bar, the Objects panel and Properties all read the selection**
  rather than the edit-operand list, so none of them goes silent on the
  selection it was written for
- ✅ **The measure tools see inside a form too** — the same leaf id type
- ✅ **Render** — off-thread, generation-counted, cancellable between
  content-stream operators
- ✅ **Zoom ladder** with a per-page raster ceiling that accounts for
  `pixels_per_point`
- ✅ **Three fit modes — page, width and height — and each places the view as
  well as setting the scale.** Fit page centres the sheet; Fit width and Fit
  height centre the axis they fit and keep your position on the other
- ✅ **The mouse wheel turns pages, if you ask it to** — a toggle beside the
  page buttons, in single-page and facing displays only, because a continuous
  display scrolls the document by definition
- ✅ **Cursor-anchored `Ctrl`+wheel zoom** — under 0.01 px drift
- ✅ **Middle-drag pan**, wheel scroll, **hand tool and space-to-pan**
- ✅ **Four page-display modes** — Single · Continuous · Facing ·
  Facing-continuous, as a radio whose active position renders pressed. Single
  page is provably unchanged: one row, no gap, so its scroll range, centring
  margin and pan clamp are the same arithmetic asserted as an equality
- ✅ **An undrawn page says so** — its real boundary, a fill that is visibly not
  paper, and a sentence centred in the part of the page **on screen**
- ✅ **Selection as identity, not position** — page + object + subpath + node,
  four integers and no coordinate
- ✅ **Selection survives navigation** — zoom, pan, fit, view rotation,
  page-display mode and tab change, all asserted byte-identical
- ✅ **Multi-select**, marquee, and a level ladder with Escape ascending one rung
- ✅ **A right-to-left box takes what it touches**; a left-to-right box takes
  only what it encloses
- ✅ **…and that also reaches an object dropped off the sheet**
- ✅ **Delete** — click then Delete, verified end to end against the real binary
- ✅ **`Ctrl` takes things out of a selection**; Shift adds to it
- ✅ **Redaction works on real drawings** — a mark whose content lives inside a
  form no longer refuses the whole apply
- ✅ **The tab order list is reordered by dragging**, with an insertion marker
- ✅ **Clicking a form row lights that field on the page**
- ✅ **Fillable fields wear a wash**, and the display buttons are two rows
- ✅ **The title bar says when this binary was built**
- ✅ **Context menus** — canvas object, canvas empty, Objects row, Pages row,
  dock tab. A menu with nothing to offer never opens
- ✅ **Read mode does not edit the document** — capability is derived from the
  **mode's tab list in the manifest**, not from the string `"read"`, so the
  canvas and the ribbon read one sentence
- ✅ **Find** — case, whole word with a configurable word rule (ISO 32000-1
  declines to define "word"), and **wildcards off by default**, because
  `find_text` enables them and a literal `?` would otherwise not be findable
- ✅ **Find never searches on a keystroke** — the engine re-extracts the
  document's text every call, so a search runs on Enter, the step buttons, or an
  option change after a search has already run
- ✅ **The Find bar has a Zoom tick-box** — unticked, a hit jumps to the page
  and leaves the zoom alone
- ✅ **Stale hits are cleared and said** — an edit clears the highlights, keeps
  the query, and the readout reads *Document changed*, because a quad recorded
  before a delete can cover different glyphs afterwards
- ✅ **Move** — drag a selection, live ghost preview with no re-raster, and a
  multi-select that moves as **one** command
- ✅ **Subpath and node moves** — the Part rung raises `move_subpath`, the Node
  rung `move_node`; a text run declines rather than borrowing the Object rung's
  verb
- ✅ **Delete removes one line, one label or one corner point** — `delete_objects`,
  `delete_objects_in_form`, `delete_subpath`, `delete_text_run` and
  `delete_node`, each addressed in its own index space
- ✅ **Several anchors move together** — a multi-node selection moves as one
  command, not one node at a time
- ✅ **The anchors are visible** at every rung; entering the Part rung is what
  draws them, because `draw_anchors` draws from that rung up
- ✅ **Bézier handles drag** — `EditSession::move_handle`, with the `v`/`y`
  re-spelling path and the disclosure contract behind it
- ✅ **Escape cancels a drag in flight** without committing, and never both
  cancels and ascends a rung
- ✅ **Eight handles — and they commit**, through `move_nodes` with the grip's
  opposite corner as the pivot. Cursors correct, and the drag is consumed so it
  cannot fall through to a marquee
- ✅ **Dock-tab context menu** — `egui-shell` hands the tab's `Response` to the
  application; the built-in Close survives for consumers that do not opt in

### Several documents at once

| | |
|---|---|
| **Several documents open** | ✅ a **document tab strip** under the ribbon — labels, an unsaved `*` marker, a close cross, middle-click to close, `Ctrl+Tab` and `Ctrl+Shift+Tab` to cycle, `Ctrl+W` to close, an overflow menu past the width. Opening a file that is already open activates its tab instead of opening it twice |
| **Drag a page between documents** | ✅ press a thumbnail, rest on the other document's **tab** until it springs open, drop at a caret between two sheets. The drag survives the document switch because it lives in `egui::Memory` rather than on the panel that started it |
| **Drop onto the page view** | ✅ the same gesture released on the canvas, with a horizontal caret in the gap between two sheets, in every display mode including single page |
| **It is a copy, and it says so** | ✅ the status row names the operation before the button is released. A cross-document move would be two commands on two undo stacks with no single `Ctrl+Z` able to reverse it |
| **Open and New do not replace the open document** | ✅ and by **removing** a guard rather than adding one: with nothing being destroyed, an unsaved-edits prompt in front of them would state something untrue, and a confirmation that says something untrue is how an operator learns to dismiss confirmations unread |
| **Live window title** | ✅ the active document, the count of open documents and the application name, so Alt-Tab and the taskbar say what you left open |
| **Shift makes it a move** | ✅ Windows' own drag modifier — Ctrl copies, Shift moves — sampled at the **release** as Explorer does, so the caption follows the key under your hand. The source's pages go only if the target's insert actually happened, and if the removal is the half that fails it says so, with the remedy |
| **Tabs rearrange by dragging** | ✅ a caret marks the boundary, resolved by neighbours' centres. **The document on screen does not change** — the active document follows its own tab through the permutation |
| **Right-click on a tab** | ✅ Close, and Close others. Close is `file.close` itself with the tab as its operand rather than a second command, which two gates refused and were right to |
| ⚠ | The drag caption must not be a wrapping label above the drop target: its height depends on its own wording, so it moved the aimed-at row mid-gesture. It lives on the status bar, which has a fixed height by construction. And `Response::hovered()` is false on every widget but the dragged one, so a spring-loaded tab has to be resolved geometrically |

### Verification

- ✅ **`ui-verify`** — drives the real binary through the OS and asserts on the
  diagnostic trace **and** the pixels. A trace-only assertion cannot tell a
  drawn canvas from a blank one
- ✅ **The registered check roster** is the binary's own:
  `cargo run --release -q -p ui-verify -- --list`
- ✅ **The gate roster** is the runner's own tally:
  `bash tools/gates/run-all.sh`
- ✅ **`PDFCER_DIAG`** — canvas layout, pointer in three coordinate spaces,
  object counts, named UI rects

---

## Planned, in order

### Next — small, unblocked

| | |
|---|---|
| ✅ | **Chords are gated on the active mode** — a chord may reach a command the active mode **shows**, or a command that lives on **no ordinary tab at all**. The second clause is what makes an exception list unnecessary: undo and redo sit on the QAT, which every mode draws, so they keep working in Read for free, as do `edit.find`, `view.read_mode`, `view.fullscreen` and the three `mode.*` commands. A contextual tab counts as no tab. The whole consequence is pinned as an exact set, so moving a command between tabs fails loudly |
| ✅ | **The text-copy commands live on File ▸ Export** — copying reads the page and writes to the clipboard, changing nothing, so it is not authoring and does not belong on the Edit tab: `file.copy_page_text` and `file.copy_document_text`, with the chord following the command in the manifest keymap. They are in Export because an export is content written out to somewhere that is not this document. Edit ▸ Clipboard was deleted rather than emptied — a captioned band with nothing under it is the placeholder R9 forbids |
| ✅ | **Read selects text, and copies it** — drag sweeps a range, double-click a word, triple-click a line, Shift+click extends from the anchor, Escape clears, `Ctrl+A` takes the page, `Ctrl+C` copies. It needs no capability flag, because selecting text authors nothing. It needs the **primary button**, so the rule is one predicate: a press means text when the select tool is active and the mode cannot select content. The two meanings are mutually exclusive by construction, both branches reading the same flag |
| ✅ | **Vertical text behaves like vertical text** — the engine publishes a glyph's advance and size as *lengths* and not a direction, which broke line segmentation, glyph boxes and the hit test on rotated strings. The direction is **measured**: three consecutive glyphs each exactly one advance from the last along a common non-horizontal direction. A page with no rotated text never reaches any of it, because the direction census comes back empty and every branch is keyed on that. 180° reaches the defect through a different clause than 90°/270°. The I-beam turns by generating its own rotated artwork. Filed with the engine; the shell-side recovery deletes when it lands |
| ✅ | **Panel toggles** — pressing an open panel's control closes it, through `Panel::from_command_id`, the single id-to-panel binding. Three states, and the middle one is the trap: a panel behind a sibling tab is not on screen, so its control **raises** it; `DockState::is_on_screen` is the dock's predicate for that distinction. `file.properties` and `markup.comments` are deliberately **not** toggles — the rule is about the control, not the panel: *is this panel open?* toggles, *tell me about this thing* shows |
| ✅ | **Edit-disclosure surface** — the move and delete verbs return operator-facing disclosures when the surgery changed an operator's form (an `re` rectangle expanded into explicit segments, an implicit `m` materialised). One line in the status bar's left half, in every mode, keyed on the **edit epoch**, so an undo retires it with nothing having to remember to clear it. Core's sentences pass through verbatim; the catalog contributes the mark, the lead-in and the separator. Shares one body with the form-fill disclosure: bounded width, fixed row, elide-not-wrap, hover for the rest |
| ✅ | **Insert an image** — pick a picture and a window states what it is: format, pixel size **as displayed** (an EXIF-rotated photograph is transposed by the importer), and its natural size on paper together with whether the resolution behind that number was **declared** by the file or **assumed** by pdfcer. Place it by a rectangle in millimetres, previewed from `NewImage::placed_rect()`. Placement is numeric rather than a drag: a placed image is page content, carries no `/Rect`, and cannot be moved afterwards. Refusals are the engine's own words, because they name the operator's file. A box off the sheet is refused, not clamped; an overhang is allowed, because bleeding past the crop box is deliberate |
| ✅ | **…and the resolution is previewed** — `NewImage::effective_dpi()` and `below_screen_resolution()` are pure, and `add_image` calls them rather than repeating the formula. Under `Contain` the placed rectangle is the *letterboxed* sub-rectangle, not the box typed into the spinner, so a local re-derivation would report a figure low by exactly the letterbox ratio |
| ✅ | **The keyboard reference is derived from the keymap that dispatches** — every row is a fold over the same `Keymap` `app::keyboard` resolves a keystroke against, so a binding that exists is listed by construction and a wrong listing is not a thing this window can produce. A chord naming an **unregistered** command is dropped (R8) and the count of drops is disclosed, because a stripped build genuinely has fewer shortcuts. Two chords on one command are one row naming both. A string in `text::shortcuts` may describe the reference; it may not be part of it |
| ✅ | **The measure tools say what they are about to pick, before the first click** — the **entity** under the pointer and the **node** it will snap to. The snap indicator had nothing to snap relative to until a point existed, so the one moment an operator most needs to know what they are aiming at was the one moment nothing was drawn |
| ✅ | **The I-beam is legible on white** — two-tone, so it survives both a white sheet and a dark object under it |
| ✅ | **Font folders** — Settings ▸ Fonts holds the folders pdfcer may take a font from when it has to embed one a document names but does not carry; Tools ▸ Font folders is a second route. Empty by default, and the emptiness is honest: guessing `C:\Windows\Fonts` would embed whatever a machine happens to hold into somebody's document, which is a licensing decision and not pdfcer's to make silently. Repeated `font_folder =` keys in `preferences.txt`, order is search order, duplicates refused, capped at sixteen with the cap explained on hover. `EmbedRequest::supplied` is a donor map the shell resolves — the engine never goes looking — so `tools.embed_fonts` depends on this list |
| ✅ | **The fonts installed on this computer, behind a checkbox** — Settings ▸ Fonts, **off by default**, listing the two folders it resolves to: the machine's, and `…\AppData\Local\Microsoft\Windows\Fonts`, where a plain double-click installs a font on a modern Windows. It does not overrule the licensing argument, it satisfies it: the objection is to pdfcer deciding silently, and a visible, persistent, off-by-default switch is the operator deciding once. A route that exists because of one setting lands on that setting: Tools ▸ Font folders expands the group and scrolls to it, once |
| ✅ | **pdfcer's own fourteen faces answer when nothing of yours can** — with no font folder configured, a drawing asking for Helvetica still embeds. The condition is disclosed loudly and is the same flag as a correctness guard: each row says pdfcer used its own copy and that it is a stand-in, and the shell reports the rung as `FontMatch::Bundled`, because `is_substitute` is what the engine's symbolic-font guard turns on. Four rungs, four sentences, with a test that they differ. It is the **last** resort, not the second |
| ✅ | **Save a compacted copy** — File ▸ Save ▸ *Save a compacted copy…* rewrites the whole file, dropping everything no longer used. The window's size line is a **measurement, not an estimate**: it serialises the document before it opens, because the operator is trading three irreversible things — a revision history, possibly every signature, the original's role as canonical — for a saving, and the measured bytes are then the ones written. A second command rather than a better Save, because appending keeps the previous revision recoverable and every signature valid |
| ✅ | **Zoom holds its place at the top of its range** — the tier hand-over fires where an `f32` offset stops addressing every pixel, which is the right question for a hard cap and the wrong one for choosing between two position models: holding a point under the cursor is a **proportional** requirement and addressing a pixel is an **absolute** one, so the error in page points is constant while the tolerance shrinks. The constant was re-derived against the measured failure rather than re-tuned, and the drift now holds at a third of tolerance at every stage |
| ✅ | **A placed markup can be moved** — the canvas forked on *is an annotation selected?* and the only module on that branch answered for ce dimensions, so a stamp took the branch, got `None`, and the content branch was unreachable by construction. A fork whose branches can all answer *not mine* needs a third arm or an explicit decline. A move has two halves and only one shows up in a render: `/Rect` moves the painted result, while `/L`, `/Vertices`, `/InkList` and `/QuadPoints` hold absolute page coordinates and are what any *other* tool rebuilds an appearance from. The shell sends a delta, never a rectangle, so it cannot send half |
| ✅ | **Embed the fonts a document is missing** — Tools ▸ Embed fonts carries the engine's own plan: every font that will gain a program and the file it comes from, the most the file can grow by, how many fonts will still have none afterwards and why. No settings on it, because the only configuration an embed has is which folders, and that lives in Settings. A document whose fonts are all embedded opens **no window** and says so on the status line. Resolution is `pdfcer_render::FontEnvironment::resolve_for_embedding`: the shell owns the **filesystem**, not the matching rules — every CAD exporter writes `Helvetica`, which no Windows machine has. A filename-stem hit is reported as `Alias`, never `Exact`. Bundled faces are not offered here |
| ✅ | **Remove embedded fonts** — the window lists every embedded font that will go with what it costs in bytes, and every one left alone with the engine's reason. Nothing moves on the page: `/Widths` is untouched and no content stream is rewritten, so every letter keeps its position and only its shape becomes the viewer's. It does **not** make the file smaller — pdfcer saves by appending and leaves the earlier revision intact — so the window states the reclaimable figure **and** that Save will not deliver it, above the button and outside the scroll area |
| ✅ | **Merge a document into this one** — Pages ▸ Merge into this document appends a whole PDF **with the things that make its pages work**: its form, its bookmarks, its named destinations. `insert_pages` orphans the widgets on the pages it takes and a field that arrives that way is drawn and unfillable; `merge_document` re-parents each widget to its field. Four disclosures, each only when it fires: pages merged, fields that arrived fillable, **fields renamed** (anything that fills this form by name will no longer match them), and **link targets renamed** (pdfcer cannot rewrite a link in a document it did not copy, so an outside reference now resolves to this document's target). `InsertPosition::End`, because a merge that opened a position dialog would be `pages.insert_from_file` under another label |
| ⬜ | **Scoped reset chooser** — reset applies `All`; the scoped variants need three commands and a split-button item kind |

### The tool is the rung

The selector is predictable because the tool *is* the rung, which is what every
editor settled on decades ago. Bare letters, because that is the layout
Illustrator, Photoshop, InDesign, Figma, Affinity and Inkscape converged on, and
this shell binds no bare letter to anything else; `canvas::keys` gates every
keystroke on `text_edit_focused()`.

| to reach this | the rung it takes |
|---|---|
| type one character | one key, one click |
| move an end point | one key, one click |

| | | |
|---|---|---|
| ✅ | **V — Select** | click a shape, drag to move, drag paper to marquee |
| ✅ | **A — Points** | click a shape and every point appears at once; click one, Shift-click for more, drag to move; a point on a curve shows its Bézier handles |
| ✅ | **T — Text** | click text to edit it, click empty space to start new text; drag still sweeps for copying |
| ✅ | **H — Hand** | pan |

`SelectionState::click_direct` is the entry point and is deliberately not a flag
inside `click`, because the two branch differently at every decision. Entering
the Part rung on a click that names a subpath *is* showing the anchors, because
`draw_anchors` draws from that rung up. Two driven checks assert the **count** —
one key, one click — because a check that armed the tool from the ribbon first
would pass on a build that takes four steps. Files dropped on the window are
read: a PDF opens, an image inserts through the same placement window the ribbon
opens, and every refusal says what to do instead.

### Selection, and what a press means

| | |
|---|---|
| ✅ | **A press on an object selects it, and the same drag moves it.** Selection happened on the click, which in egui means on release, so a press-and-drag on anything not already selected used to marquee across it instead |
| ✅ | **The application opens in the mode you left it in** — the mode is persisted, not taken from the manifest's first entry, so the restored per-mode layout and the restored mode agree |
| ✅ | **The status bar says what is selected, and how deep the click reached** — the outline drawn round the page edge looked exactly like *the page is selected*, a state this program does not have |
| ✅ | **`Alt`+click walks the stack** — `hit_test_all` returns every candidate under a point front to back; each `Alt`+click goes one deeper and wraps, and moving more than 4 pt resets the cursor, so it is a gesture about *this* point rather than a mode |
| ✅ | **One selection, read by everything** — a row click raises `Action::SelectObject` and every surface reads the same selection; three parallel notions of *the thing I am working on* is what an unpredictable selector is made of |
| ✅ | **Resize** — `move_nodes` takes a list of per-node deltas and a scale **is** a list of per-node deltas, `d = (p − pivot) × (s − 1)`, computed once per anchor from the grip's opposite corner |
| 🔨 | **Object clipboard** — cut, copy and paste, scoped to what the engine can express: `annot_author::spec_from_dict` reads a markup out and `add_markup` writes one back, so markup and comments round-trip |
| ✅ | **Restyling a placed markup** — colour, line width and opacity through `EditSession::set_markup_style` on a selected annotation, in the **Properties** panel, which is already the surface that appears on selection and already reads the annotation |
| ✅ | **Editable geometry** — a position change is `Action::MoveSelection` and a size change is `resizing::action`, so this surface computes two factors and a delta and contributes no geometry of its own |
| ⬜ | **Undo/redo of the command log** — the action funnel exists precisely so this is possible; the log is not yet surfaced |

### Deep zoom — the maximum is yours to set

| | |
|---|---|
| ✅ | **Click the zoom percentage on the status bar** — a list of maximums from 800 % to a trillion, and the shipped default is the highest. A capability you have to find a preferences file to switch on is one most of its users never have |
| ✅ | **The readout was already the natural home** — the one control on the bar that is about zoom and did nothing when clicked. A label that turns out to be a button is this surface's existing idiom, and a separate control would have cost width on a bar whose height and right-hand cluster are both fixed |
| ✅ | **Also editable in `preferences.txt`** as `max_zoom_percent` |
| ✅ | **`max_zoom_percent`, up to a trillion percent** — no guard, no warning and no preflight: the setting's job is to be honest and to work. Defaults to 800 % |
| ✅ | **It is also the dial for comparing the two rendering paths** — pdfcer rasterizes the whole page while it can and switches to the visible region only when it cannot. Set the maximum low and you never leave the whole-page path; set it high and you exercise the region path. A threshold rather than a mode, because a threshold explains itself |
| ✅ | **Panning keeps full detail** — whole-page rasterizing is what makes panning free, because the texture already exists and the view moves over it, so the region path engages only above the pixmap ceiling |
| ✅ | **The `+` button climbs past 800 %** by doubling once the named rungs run out — eleven presses from 800 % to a million, where a fixed increment would need thousands |
| ⚠ | Two defects were caught before they shipped and both would have been silent: a plain maximum broke the promise that the default changes nothing, because a large page at a 1.5× display tops out below 800 %; and the `+` button stalled at 800 % however high the setting went |
| ✅ | **It renders past the raster ceiling** — above roughly 1000 % pdfcer rasterizes only the visible rectangle, whose size follows the window rather than the zoom, so the requested picture stays the same size however deep you go |
| ✅ | **Measured on the operator's own `banana.pdf`: 2,118,335 %, page drawn, no failed renders** |
| ✅ | **Panning stays free below the crossover**, which is every zoom that could render whole-page before. Above it a pan costs a redraw, and the raster carries half a screen of margin on every side, so movement inside that costs nothing and the redraw comes at most once per half-screen of travel |
| ✅ | **A trillion percent, with the page drawn**, measured on the same file with no failed rasters |
| ✅ | **Three ceilings fell, each found by asking where precision was actually lost** — the raster limit went to region rendering; the 32-bit scroll offset went to a 64-bit anchor; and the third was not a type but a derivation, the drawn rectangle being computed from the page's whole screen rect, a value around 10¹² px where an `f32`'s spacing is 131,072 px |
| ✅ | **Not forming the large number beats widening it** — carrying a huge intermediate more precisely is worse than not computing it: the fix costs nothing, needs no second layout path, and removes the limit rather than moving it |
| ✅ | **Panning below the crossover is untouched**, and the deep path is a hard branch rather than a re-parameterisation, because this canvas has twice been broken by a change meant to affect only deep zoom |

#### Confirmed on screen

| | |
|---|---|
| ✅ | **The page is drawn at every decade from 114 % to 999,999,995,904 %**, asserted from **pixels**: `the_page_still_renders_at_every_decade_of_zoom` photographs the window at each tier and requires three things together — the canvas region is not near-uniform, the canvas reported `drawn ≥ 1`, and no render failed |
| ✅ | **Measured on `banana.pdf`**: at 504,845 % one mitochondrion with the cell wall behind it; at 3,730,330 % cristae; at 41,120,084 % the mtDNA nucleoid, mitoribosomes and ATP synthase heads — 10 nm features. At the ceiling the viewport spans 6 × 10⁻⁸ pt and the field is a flat fill, which is correct |
| ⚠ | From about 10⁷ % the requested region stopped shrinking and floored at fifty thousand times the viewport, its raster painted far off-screen: `render::strategy::region_for` snapped the region origin by dividing an **absolute page coordinate** by a **relative extent** |
| ⚠ | **A canvas can satisfy every trace-based assertion while showing blank paper.** The raster was produced, `drawn=1` was traced, the position arithmetic was independently correct, and a *window* capture is never uniform because a window always contains a ribbon. Only a screenshot aimed at the canvas itself can tell |
| ✅ | **The `−` button is the inverse of `+`** — it halves above the named rungs instead of returning to 800 % from anywhere above. Pinned as a round trip rather than against fixed numbers, because the property is reversibility |
| ✅ | **A zoom no longer discards where you panned to** — the anchor solve was clamped to `display − viewport`, which is zero or negative when the page is no larger than the window; the clamp moved to the one place that knows the pasteboard-extended range |
| ✅ | **The tier hand-over holds the view** — three faults met there: the `f64` anchor was seeded from the previous frame's scroll offset, nothing called `DeepAnchor::zoomed_about`, and the scroll area kept its old offset for one frame |
| ✅ | **The readout survives its own range** — it returned a `u32`, and `as u32` saturates, so past 42,949,672 % the bar showed `u32::MAX` presented as a measurement. It reads 999,999,995,904 % at the top, which is honest rather than rounded: the zoom is an `f32` and that is the nearest value it holds |
| ⚠ | The recurring shape of all of these is one pattern: a limit lifted in one place while a narrower type downstream keeps enforcing the old one, silently |
| ⬜ | **What is out of reach, stated rather than implied** — anything inside an atom. A carbon nucleus 200 px across needs 7 × 10¹⁴ %, about 700× past the ceiling; a proton needs 4 × 10¹⁵ %. At maximum zoom one screen point is 35 femtometres |
| ✅ | **An atom at 1:1, magnified until it is legible** — a benzene molecule at true scale (aromatic C–C 0.140 nm, carbon van der Waals radius 0.170 nm) on screen with bonds, hydrogens and van der Waals spheres at 36,919,476,224 %, driven through the release binary and photographed |
| ⚠ | The same molecule was blank at page (540, 560) while rendering near the origin, because the renderer's path coordinates were `f32`, whose step there is 21.5 nm against an atom's 0.34 nm. The engine's coordinates are wider now; a limit measured at the origin is not a limit |

### Free navigation — the pasteboard

| | |
|---|---|
| ✅ | **Any corner of the page can be brought to any point of the screen** — one whole viewport of empty space on every side, expressed as a **fraction of the viewport** rather than a number of points, so it keeps satisfying the requirement at every zoom and page size |
| ✅ | **It replaces a clamp to the page**, which is the one place that decision lives: `canvas::geometry` clamps to the pasteboard-extended range, not to the sheet |
| ✅ | **It also fixed the rotate handle at the top of a sheet** — the handle sits 20 pt above the selection box, so an object flush with the top of the view had its handle drawn outside the canvas, clipped away, with a press landing on the ribbon |
| ⚠ | The cause of three misdiagnoses was one missing coordinate conversion: the rect deciding which pages are laid out at all was built from a scroll offset in the content's space and intersected with the strip's. Before a pasteboard those were the same rectangle |
| ⚠ | Every failing trace carried `canvas-unavailable reason=nothing-visible`, which states the cause exactly, and it was never grepped for because each search was for a *symptom*. Search the trace for the stated reason before reasoning about the arithmetic |
| ✅ | **A driven guard came out of it** — `scrolling_far_keeps_the_canvas_its_pointer_input` wheels the view to 1,600 pt and asserts the canvas still receives the pointer |
| ✅ | **Objects off the page are reachable, and they are drawn** — the space around the pages senses hover and accepts presses, and off-page content is laid out rather than clipped away |
| ✅ | **…and whether any of it is shown is a per-mode, remembered switch** — `View ▸ Display ▸ Off-page content`, written the instant it is given and surviving a restart, with an independent answer for Read, Review and Edit |
| ⚠ | Four existing off-page checks had to be told to open in Edit, because their subject is off the sheet and Read now hides it. Adding a default silently re-points every instrument that relied on the old one |

### Phase 3 — viewer conventions

| | |
|---|---|
| ✅ | **Hand tool**, space-to-pan — Space is read by the canvas itself, so it needs no keymap entry and cannot be unbound by accident |
| ✅ | **Cursor-anchored discrete zoom** — the rule is decided once, in `canvas::zoom::anchor_point`, and all five paths route through it: wheel, the three commands, and the framing verbs |
| ✅ | **Zoom to selection** — gated on `selection.bounds`, which is *not* `selection.any`: an identity can outlive the box it described, and framing nothing is a jump to the origin that looks like a bug |
| ✅ | **Marquee zoom to region** — one rubber band shared with marquee-select, branched only at release; a zoom marquee never touches the selection and decomposes nothing |
| ✅ | **Recent files** — `recent.txt` beside `layout.ron`, capped at 10, move-to-front; missing entries dropped at display time only, throttled so a dead network path cannot block the UI thread |
| ✅ | **Rulers** — in **points**, or in the document's own unit at its own scale when its dimension sidecar carries one, through the same `pdfcer-core` `format_measurement` a dimension label uses, so a ruler and a dimension across one span agree to the digit. The gutters take a constant bite out of the viewport, so switching them on costs exactly one re-fit |
| ✅ | **Grid** — per page, in page space, clipped to the sheet, so it scrolls with the drawing and the row gaps between sheets carry none. Every numbered ruler tick has a grid line under it, because both come from one 1-2-5 ladder, and the pitch is bounded on the **drawn** step rather than the labelled one |
| ✅ | **Guides** — belong to a **page**, dragged out of a ruler, moved on the canvas, deleted by dragging off it or by a double-click. A guide's catch band is registered after every page widget, so grabbing one cannot also rubber-band a selection. Persisted per document in `guides.txt` |
| ✅ | **A new panel reaches an operator who upgrades** — a layout records which panels existed when it was written, so a genuinely new one appears while one closed on purpose stays closed |
| ✅ | **Thumbnail grid** — one tile per frame, on-screen only, current page first, 64-texture cap, and a hard stop past 400 ms naming the page and its cost |
| ✅ | **Worded decline** — `ZoomOutcome::NoBounds` and `NoCanvas` reach the operator instead of the floor, through `app/status/decline.rs` and the status bar's left half. Same surface as the edit disclosure, different store: a decline changes no document, so the edit-epoch key does not apply |
| ✅ | **…and how zoom-to-selection is reached was decided rather than left open** — SolidWorks and Acrobat both reach it by right-click and only Inkscape binds a key, so `view.zoom_selection` joined the `canvas.object` context menu |

### Phase 4 — page display modes

| | |
|---|---|
| ✅ | **Continuous scroll**, and Read defaults to it. Single page stays the default everywhere else and is unchanged — the strip lays out one row for it, so its size, scroll range and centring margin are the same arithmetic, asserted as an equality rather than as an intention |
| ✅ | **Facing** and **facing-continuous**, cover page alone. Fit and the raster ceiling are per-**row**: a spread fits as a spread, and the ceiling is a minimum over the row's pages because a spread is two pixmaps |
| ✅ | **A continuous fit does not depend on where you scrolled to** — under a continuous mode the fit is taken over the document's tightest row, per axis; under Single and Facing it is the current row, which the operator chose. `page_index` is derived from the scroll in a continuous mode, so fitting the current row would make the zoom depend on the scroll and the scroll on the zoom |
| ✅ | **Per-document persistence of the choice** in `page-display.txt`, so a sheet set does not inherit a report's setting |
| ✅ | **Only visible pages rasterize**, one at a time, nearest the viewport centre first, bounded by a texel budget |
| ✅ | **An undrawn page says so** — its real boundary, a fill that is visibly not paper, and a sentence naming the page and its state, centred in the part of the page on screen |

### Phase 5 — text editing

| | |
|---|---|
| ✅ | **The tool itself** — `CanvasTool::TextEdit(TextEditKind)`: click to place a caret, type, Enter or a click elsewhere commits, Escape abandons, a mode change disarms *and* abandons. Gated on `edit_content`, so Read cannot reach it. It takes a click and no drag, the shape `press_kind` already uses for measure and vertex markup |
| ✅ | **Alignment and rotation fixes** — one pure `choose(text_matrix, ctm, alignment)` at the single commit site: rotation is rung 1 and outranks alignment, non-left alignment is rung 2, both selecting `FollowerDisposition::Pin`, which the engine carries for a justified or right-aligned tail that must not move |
| ✅ | **Text inside a drawing block is editable** — click a label, a title-block field or a *pdf dimension* callout, get a caret, retype it. On a CAD sheet that text is nearly all the text there is, and it lives in a **form XObject** |
| ✅ | **Shared content is disclosed, in the engine's own words** — a drawing program may place one copy of a title block and paint it on six sheets, and no clause in either edition of the spec binds a form to a page, so editing text inside a shared one changes every sheet it appears on |
| ✅ | **Reflow shifts the rest of the line by the advance delta** — the engine's walk stops at the `Td`/`TD`/`T*` boundary rather than running forward through every absolute `Tm`, which on a CAD stream is the difference between one label and every label on the sheet |
| ⛔ | **Live re-layout while typing** — blocked on the engine. Measured per keystroke: 0.49 ms on a 3-line fixture, 102.77 ms on a SolidWorks sheet, 356 ms on an A3 benchmark. `plan_edit` already computes `advance_delta` before any write but is `pub(crate)`, so every public route performs a full incremental save. Request: a `measure_edit` that returns the delta without writing |
| ✅ | **Re-wrapping a paragraph is a command** — Edit ▸ Reflow paragraph, and a right-click inside the text you are editing |
| ✅ | **Text on a line shared with other runs** — on a real drawing almost every line is several runs, so an edit must address the run and leave its neighbours' positions alone |
| ✅ | **New text gets a font, a size and a colour** — `AddTextRequest` carries `face`, `size` and `color`, and the apply arm overrides none of them. A **tool option**, not a Format property, on the split `canvas::markup::pen` already makes: Format describes the placed thing, a tool option describes what the next thing will be |
| ✅ | **New text can be multi-line** — arm Edit ▸ Add text and **drag a rectangle** instead of clicking: Enter starts a new line, text wraps at the right edge, `Ctrl+Enter` commits. A plain click still places one line at a point and Enter there still commits |
| ✅ | **The navigation keys walk the page, not the fragment** — ↓/↑ move to the line below or above, including into the next block of text; End/Home reach the end and start of the visual line |
| ✅ | **There is a selection inside a text draft** — Shift+arrows, Shift+Home/End and `Ctrl+A` select; typing replaces the selection, Backspace and Delete remove it and nothing else, and **any movement without Shift drops it** |
| ✅ | **A dialog is owned by the window it belongs to** — `egui-winit` does not pass down the parent relationship egui tracks in `viewport_parents` and does not hand back the child's handle, so ownership is set from the platform side once the child exists |

### Phase 1 — the transform verb, and what it retired

| | |
|---|---|
| ✅ | **Move, resize and rotate any object** — the whole shell side had been built and waiting since S4: selection, eight grips, the drag machinery. What it needed was the engine verb |
| ✅ | **A grip on the page's own edge can be grabbed** — select the sheet border, a full-bleed image or a title block and drag its south-east corner. The press was understood as a resize and never delivered, because egui gives a widget covering the whole canvas the drag before the grip under the pointer sees it |
| ✅ | **The move drag forks rather than switching**, and it is a decision about the *file* rather than about the API. `move_objects` rewrites coordinates in place and adds nothing; a transform adds a `q`, a `cm` and a `Q` **per object per gesture**, so routing every move through the transform would grow the content stream on every drag |
| ✅ | **A rotate handle** — a circle above the selection box on a short stem, which is PowerPoint's, Illustrator's, Figma's, Inkscape's, Visio's and Konva's arrangement. A **circle** rather than a ninth square, because every square on this canvas resizes |
| ⬜ | **The transform preflight** — `transform_preview` is `&self` and shares one body with the verb, so `preview(..).is_ok()` *is* the predicate, and an object whose own CTM is singular means **do not offer a handle**. Today pdfcer offers one and the operator finds out by dragging. Not built because the preview **decomposes the page**, which costs seconds on a dense sheet |

### Defects found by driving

| | |
|---|---|
| ✅ | **Full screen turns off again** — `toggle_fullscreen` read `ViewportInfo::fullscreen` to decide what to ask for next, and a `ViewportCommand` is **queued and answered by the backend**, so a second press before it caught up asked for full screen again and the window filled the display with no route back |
| ✅ | **Two driven checks were accusing the build of the fixture's properties** — an insert-image check's pixel floor was *one in five hundred of the page*, which is a statement about the fixture: on a 1584 × 1224 pt CAD sheet at 0.3× a 64 × 16 pt picture is 19 × 5 screen pixels, so a **correct** insert failed it |
| ✅ | **The baked-gap gate had a hole the width of a closing brace** — its pattern enumerated the character before a suspicious run of spaces as a fixed set that omitted `}`, which in Rust is one of the likeliest characters to end a clause. Four already-shipped defects had gone through it; widening the set to any non-space except an opening bracket closed it |

### The selection filter — what a click can land on

A filter menu on the status bar naming every class a click may land on, to
replace declaring an intention in the ribbon before pointing at anything.

| | |
|---|---|
| 🔨 | **A `Select` button on the status bar, and eleven classes behind it** — Text, Lines, Pictures, Blocks, Parts, Points, Markup, Dimensions, Form fields, Links and Characters, each with a glyph and a checkbox, plus All and None. A class switched off is not selectable in Read, not in Review, not in Edit. **Not yet driven**, and a green suite is not a report of working software (R1). The popup publishes an indexed diagnostic rect per row so a harness can choose from it rather than only prove it opens |
| ✅ | **The diagnosis is about gesture design rather than about two buttons** — Edit ▸ Content asks the operator to declare an intention before pointing at anything, and the hit test then obeys the declaration rather than the drawing, so the same click on the same pixel means different things depending on a control that is not on screen while you aim. A filter on the status bar is always visible, one click from anywhere, and says what it is doing while you do it |
| ✅ | **It is subtractive, always, and that is what makes it safe** — it can only take candidates away, never make something pickable that was not, so `PickFilter::default()` is *defined* as everything the shell can pick today and **R6 holds by construction** rather than by testing every path twice. The tempting alternative, *Points on means a click lands straight on the nearest anchor*, was refused on measurement: one CAD export in the fixture set holds **6,681 anchors in a single path object** |
| ✅ | **It sits above the mode, which is not the same as replacing it** — the composition is `capability_allows(class, mode) && filter.allows(class)`, **both**. Switching a class on cannot grant Read the ability to edit content, and nothing in the filter can reach a capability; collapsing the two questions is how a filter turns into a hole in the mode system |
| ✅ | **The class list is derived, not invented** — every one of the eleven is something the hit test can already distinguish, routed through `panels::objects::summary::object_kind` rather than re-deriving it, because a second kind classifier is the divergence that module exists to prevent. A row nothing consults is a lie told once per session; a distinguishable class with no row is a thing the operator cannot reach |
| ✅ | **A form XObject is called a *Block*** — an entire nested drawing the page treats as one opaque object, usually a title block or a border, and the usual answer to *why is the selection box so big?* *Groups* would collide with optional content groups, which this shell has a panel for; *Form XObject* is correct and undecodable by anyone who has not read the specification |
| ✅ | **Switching everything off is legal, and the bar says so while it lasts** — *"Nothing on the page can be selected"*, on the left with the narration, ahead of the decline note because it outranks it: a decline explains why one gesture did nothing, this explains why every gesture will. Off-canvas, per the disclosure rule |
| ✅ | **It survives a restart**, in one readable line beside `settings.txt`, written immediately rather than debounced because a filter can only change on a discrete click. Three on-disk states are kept distinct and the third is the one that matters: no file means *never configured*, a file with names means *these classes*, and an **empty** file means *everything is switched off* |
| ⬜ | **The View twin** — a second popup beside it governing what is drawn with a bounding box or node markers while unselected. Not started |
| ⬜ | **Select other on right-click** — walking the stack of objects under the cursor. `hit_test_all` already returns the full depth-ordered list and every pick funnels through it, `hit_test` being defined as its head; what is missing is the command and the menu, not the query. Until they exist, **an object completely covered by another is unreachable by any click** |
| ⬜ | **Deleting Edit ▸ Content** — the third button is already gone; what remains is *Edit text* and *Add text*. Whether those two are deleted or kept as a redundant path once the filter lands is the operator's call |
| ✅ | **Redact by selecting on the canvas** — before it there were exactly two marking routes, the search box (which reaches text pdfcer can read as text) and *mark whole page*. On a CAD drawing almost everything worth redacting is between them: a title-block value drawn as vector strokes, a scanned stamp, a logo, a signature image, a run in an unmappable font |
| ⬜ | **Push buttons can be placed and can never do anything**, and that is a pdfcer-wide policy rather than a shell gap: the engine authors no action of any kind on a button it creates — no submit, no reset, no navigate, no script — because `/A` reaches launch actions, network submits and JavaScript, which pdfcer recognises and preserves and never writes. Disclosed on every creation; filed to the engine as a policy question in its narrowest useful form, **Reset only** |
| ◐ | **Turn a form field's box** — Turn left and Turn right in the field's Properties. It turns what is drawn **inside** the box; the box itself stays put. The direction is the whole risk: a widget's `/MK /R` is **counterclockwise** while a page's `/Rotate` is **clockwise**, and the standard's two sentences are otherwise word for word identical, so a shell that misses it ships two buttons that both write a legal angle and both turn the wrong way |
| ◐ | **Say something other than the measurement** — a text box on a ce dimension's properties. It does **not** change what was measured: the measurement stays underneath, so clearing the box restores exactly the number that was there rather than re-measuring, and the receipt names the number both going on and coming off. There is no Clear button — emptying the box **is** the restore. Built; the driven check is still owed |
| ✅ | **Deleting makes the object go away at once** — it used to sit on screen for a second or two while the page redrew with no gesture in flight to explain the wait, so a delete looked like it had done nothing and the natural response was to press Delete again, which deletes something else |
| ✅ | **And when it cannot show you, it tells you** — *"Your change is made — the picture of it is still being drawn."* There is no shape to slide when you change a colour, press Bold, delete text, mark a redaction or rotate a page, and no cheap redraw either: a two-pixel region render costs 691 ms on the benchmark site plan. Silent under 0.4 s, so it never flickers on quick edits, and an unedited document never says it |
| ✅ | **The shape follows your hand** — drag a line's end point and the line **bends**; drag a shape and the shape moves; resize and the shape scales. Every gesture used to draw a bounding box and nothing else. It uses the geometry pdfcer has already parsed, so it costs no engine call and no redraw and runs at pointer speed, and for geometry it is exact rather than approximate |
| ✅ | **The old position stops showing** — the picture underneath still has the object where it was and cannot be redrawn in under about 0.7 s on a dense drawing, so without this you see the thing twice. It erases the object's **own outline** rather than a box over it, so a polyline blanks a line's own width and not a title-block cell; anything drawn underneath the object's own ink disappears until the redraw lands, which is the one part that is not exact |
| ✅ | **It stops snapping back** — the preview stays on screen until the page catches up. Releasing a drag used to throw the preview away while the picture still showed the old position, so the object appeared to jump back and then forward, which reads as the program refusing the edit and changing its mind. Holding it is honest rather than optimistic: the edit **has** happened and only the picture is behind. A refused edit holds nothing, and a preview nothing ever arrives to replace is dropped after four seconds |
| ✅ | **All three driven and falsified** — `dragging_a_node_bends_the_line` asserts the preview is built, that it reaches the painter, and that it outlives the release, and it was then run against the **previous** release, which has none of this, and failed on the first assertion. A check that cannot tell two builds apart makes a green result mean nothing |
| ✅ | **The Bold tooltip stopped telling you the opposite of what Bold does** — hover Bold on a `Helvetica` title block and it used to say pdfcer would thicken the letters; press it and you got real `Helvetica-Bold`. The hover was asking a different question and answering it accurately, which is the failure shape: two instruments, one page, opposite answers, and the tooltip is the one read first |
| ✅ | **Bold stops thickening the letters on a title block** — press Bold on a drawing whose only font is `Helvetica` and pdfcer sets the text in `Helvetica-Bold`, one of the fourteen faces every reader must carry, so no font file is embedded, the file does not grow, and what comes off the plotter is a real weight rather than a stroke drawn round the plain letter |
| ✅ | **Bold uses a real bold font again** — on a page that carries a real bold face, pdfcer sets the text in it rather than thickening the plain face. It had stopped without anything failing: an engine pass turned a refusal this shell was **built on** into a success, so the retry that reaches the real face stopped firing and Bold began faking a weight into the saved file |
| ✅ | **And a rung that was never there** — where the real face found cannot show the text (`Times-Bold` has no `o`, so it cannot set *hello world* on a page where `Calibri-Bold` can), the old behaviour gave up and named a face that had just failed. The rung is now the engine's, last of four, reached only after a real face on the page and the standard-14 sibling have both been tried |
| ✅ | **Faking bold and italic is your choice** — Settings ▸ Fonts: fake it quietly (the default), fake it and say so plainly, or never fake it. A real face is preferred under all three, and the window says so under the group, because *never fake it* reads like *never change my font* and is not |
| ⚠ | **The two form-field rows above are unverified by driving**, and the reason is the machine: every input-driving check reported *"the window could not be brought to the front"* — Windows refusing `SetForegroundWindow` to a background process — including checks that had passed an hour earlier unchanged. That is a way of running the suite wrongly (a busy foreground), not a defect in what shipped. Two checks are written and registered and waiting |

### Phase 1¾ — the clipboard

| | |
|---|---|
| ✅ | **Cut, copy and paste of page content** — select a line, a shape or a piece of text and `Ctrl+C`; `Ctrl+V` lands it 10 pt down and right so the copy is visible, or in place on a different page. `Ctrl+X` is one undo entry, because only one half of a cut is an edit. Any mixture of kinds, and the clip owns its resources by value, so a copy survives closing the source document |
| ✅ | **…and binding the chords was necessary and not sufficient** — `egui-winit`'s key handler pushes `Event::Copy`, `Cut` and `Paste` and **returns before the `Event::Key` push**, so the chord matcher saw nothing while the manifest contained the binding, unit tests asserted it, and the menu displayed the shortcut. **A keymap lookup is not a keystroke** |
| ✅ | **…and the same defect was live in two more places** — `canvas::textsel::clipboard` asked `key_pressed(Key::C)`, the identical mistake one grep away from the paragraph explaining why it cannot work, so sweeping text on the page and pressing `Ctrl+C` had never copied it, in any mode, since the day it was written |
| ⬜ | **Across two pdfcer windows**, and **to another program**. The first needs the clip registered under a private Windows clipboard format, a call this shell does not make yet. The second needs the selection rendered as a standalone one-page PDF, deliberately *not* the same bytes: a one-page PDF cannot carry which byte range was which object, so re-deriving that on the way back in would make a paste between two pdfcer windows worse than a paste into Illustrator. Two formats, two jobs |
| ✅ | **Form fields, both senses** — `Ctrl+V` pastes a copied field as a **new, independent** field with a free name; `Ctrl+Shift+V` pastes it as **another box for the same field**, so typing in either fills both. Before this there was no path at all: `canvas::clipboard::copy` reads the object selection, a selected field lives on `doc.selected_field`, and `/Widget` is deliberately excluded from the annotation clipboard |
| ✅ | **Cut, copy and paste for everything else** — **cut is greyed with a reason and the chord is refused too**, because a chord is dispatched through the keymap **without** consulting command enablement, so greying the button alone would delete a redaction mark on `Ctrl+X` while putting nothing on the clipboard. It refuses **before** the copy, asserted separately: refusing after would leave the mark on the page and a copy of it on the clipboard |
| ⬜ | **Dimensions** — annotations rather than page content, so these verbs cannot address them at all, and a pasted dimension needs a sidecar record and a group. Filed |

### Phase 1½ — the conventions sweep

`D:/dev/rag/ui-conventions/` is five gesture classes, each a numbered list
carrying where the rule comes from and the failure mode when it is absent;
`tools/gates/check-conventions.sh` makes every registered surface answer every
row **in its own source**. It cannot check behaviour and does not pretend to —
it checks that the question was asked, which is the whole of the problem.

| | |
|---|---|
| ✅ | **Shift constrains every drag on the canvas, and says so** — a **resize** keeps the shape's proportions, applying the factor the pointer travelled furthest to produce, which for a mid-edge grip falls out as proportional-resize-from-an-edge with **no special case**, because the idle axis sits at exactly 1.0 and can never win. A move, a ce-dimension label, a perimeter corner and a Bézier handle each constrain in their own terms |
| ✅ | **A perimeter corner snaps back onto the geometry it came off** — the tool that **placed** the vertex snapped and the drag that moved it did not, so a corner could be put onto a line and never put back. It now uses the same `snap_candidates` query, the same tolerance and the same *Snap to content* switch a measure pick uses, through one function, because a drag that snapped by its own rules would honour a different switch from the tool beside it. Alt suspends it |
| ⬜ | Still open from the sweep: no right-click to add or remove a perimeter point (both engine verbs exist), a zero-travel release still raises an action in three of four drag paths, an unfilled `/Square` still claims its interior, and caret indices are characters rather than grapheme clusters |

### Phase 6 — markup completeness

| | |
|---|---|
| ✅ | **Markup tool substrate, and four kinds placing** — Rectangle, Ellipse, Arrow, Highlight. One `CanvasTool::Markup(MarkupKind)` carrying a kind rather than one variant per shape, so a tool that is two kinds at once is unrepresentable and the four ribbon controls behave as a radio with nothing enforcing it. Rubber-band preview, `Action::CommitMarkup`, an apply arm calling `EditSession::add_markup`. **A click with no drag places nothing** |
| ✅ | **Drag origins are the button-down point** — `Response::drag_started()` fires only once egui has *decided* an interaction is a drag, by which time `interact_pointer_pos()` reports where the pointer has travelled to: measured at **94 PDF points** of error on A1 at 0.21× zoom. The fix went into `PointerFrame::press_origin`, not into markup, because the marquee and the move drag had it too |
| ✅ | **Polyline, polygon, ink** — ink is drag-shaped and reuses the existing path; polyline and polygon are click-shaped and get a live click and no drag from `press_kind`, in an early return beside the measure tools'. Ending follows the circular measure tool exactly: a double-click and a registered `markup.finish`, through one commit path |
| ✅ | **A shape you drew can have its nodes moved and deleted** — select a polyline or a polygon and every node gets a small square: **drag** to move it, and with the Points tool armed (`A`), **Ctrl-drag** to add a node after the one you grabbed and **Ctrl+Shift-drag** to delete it. The gesture is identical to the one content geometry uses |
| ✅ | **…and the ending disagreement is a worked example of the reference-app rule** — Inkscape and SolidWorks both close a shape by clicking the first vertex; **Acrobat does not, and won on applicability rather than head-count**, because `/Polygon` closes back to its first vertex by §12.5.6.13, so a click-the-first-vertex rule would author a duplicate vertex and a zero-length closing segment. SolidWorks' Escape-to-end was refused outright: here Escape means *abandon* |
| ✅ | **Underline, strikeout, squiggly** — select text, then press the control: the selection is the operand, so there is no `CanvasTool` variant, no new gesture and no second text-range resolver. The **wash is the preview** |
| ✅ | **…and Edit reaches them too** — with the text tool armed, Edit sweeps text, `selection.text` becomes true and the three controls enable. `text_tool_selects_and_marks_in_edit` drives dead control → arm tool → sweep → the same control now invoking, and then **retires the tool and proves the sweep stops working**, which is the phase that falsifies it |
| ✅ | **Revision clouds** — `MarkupSpec::Cloud` carries vertices, border, interior, width and intensity. `MarkupKind::Cloud` joins `is_vertex` and nowhere else in that impl, so the polygon gesture carries it |
| ✅ | **Note text** — the engine writes `/Contents`, `/T` and `/M`, this shell calls `EditSession::set_markup_note` from `app/actions/annots.rs` and edits it in the Comments panel's note editor |
| ✅ | **Style: width and fill** — width in the Properties panel's restyle section, fill on the Format tab's Markup band and in the panel |
| ✅ | **Style: opacity** — `EditSession::set_markup_style` takes `MarkupStyle { opacity: Some(StyleEdit::Set(a)) }` and writes `/CA`, with `StyleEdit::Clear` removing it. What is genuinely absent is narrower: `MarkupSpec` carries **no alpha**, so a mark is authored opaque and then restyled |
| ✅ | **Line style (dashed)** — `MarkupStyle` carries a dash, so a dashed border can be preserved, authored **and** removed; `app/actions/apply.rs:636` passes `dash: pen.dash_option()` |
| ✅ | **A width asked for on a text markup is refused, not silently dropped** — the engine returns `EditError::StylePropertyNotApplicable`, which says *the call asked for something that cannot apply here*, as against a dropped property, which says *the file lost something* |
| ✅ | **Line endings can be cleared** — `endings` is a `StyleEdit`, so removing `/LE` **removes** it rather than writing an explicit pair of `None`s. Same picture, different bytes, and *is this byte-identical to what my client sent me* is a question that gets asked about a signed drawing |
| ✅ | **A text box's painted words are rewritten, not left behind** — the note verb re-bakes the `/AP` `/N` stream and reports through `appearance_rebaked` whether it did; `app/actions/annots.rs` consumes it and `text/textannot.rs:403` carries the surviving narrow disclosure |
| ⚠ | Driven verification is still owed on the four rows above. Wired and unit-tested is not driven (R1) |

### Phase 7 — measure completeness

`measure_tool.rs` came across whole as `canvas/measure/{pick,scale,state}.rs`
with the snap primitives as `canvas/snap.rs`; `CanvasTool` grew a
`Measure(MeasureKind)` variant on the same one-variant-carrying-a-kind
argument markup settled.

| | |
|---|---|
| ✅ | **Linear ce dimension** — click A, click B, click where it sits. The third click **is** the commit: a separate accept/reject box was retired by operator instruction, and application-initiated floating windows default to Never |
| ✅ | **Two-line ce dimension** — pick two lines already on the drawing; `pick_line_in_page` resolves each click against the page's own geometry, so the shell and `pdfcer dimension-add` resolve the same click to the same line |
| ✅ | **Escape is two rungs, not one** — a measure pick is a sequence of *clicks*, so there is no drag to cancel, yet a dimension with point A taken and B not is unmistakably in flight. One Escape corrects a mis-aimed pick; a second puts the tool down |
| ✅ | **Radius and diameter, as a point tool** — it used to hit-test for a path **object** and feed every anchor of every subpath of that object to the fit, so picking near a hole in a dense drawing fitted a circle to the whole polyline. It now fits to the points the operator picked |
| ✅ | **The measure preview draws in the right place** — `measure::preview` converted PDF user space through `viewer::pdf_space_to_canvas` and handed the result straight to `ui.painter()`, which lands in **canvas** space (page top-left origin, no zoom) while the painter speaks **screen**, so the snap indicator and the linear preview were offset by wherever the view had scrolled |
| ✅ | **Set scale** — `measure.set_scale` opens the dialog, *Measure it on the drawing* **hides** the window and arms `MeasureKind::Scale`, the pick completes, and the **same** window comes back with the measured length in it. It used to close and rebuild, which discarded every field already filled in |
| ✅ | **Snapping** — a pick lands on the geometry rather than near it, which on a CAD sheet is the difference between a dimension that measures a line and one that measures *close to* a line, worse than none because it is wrong by an amount nobody can see. `snap_candidates` is the **engine's** query, not a second one here |
| ✅ | **A derived candidate is confirmed, never assumed** — an inferred centerline takes two clicks: the first promotes, the second commits. The call site passed a constant `is_derived: false` until the query existed, which silently turned an inference into a commitment |
| ✅ | **The snap indicator has its own colour** — the marker used to draw in the selection colour, so the cue distinguishing *pdfcer proposing a point* from *a committed thing being selected* was not there |
| ✅ | **Dimension groups are manageable** — `measure.manage_groups` on Measure ▸ Scale, with the window's controls rather than a sentence where three controls should have been |
| ✅ | **A placed ce dimension is editable** — select one and Properties shows its group, what it measured, a radius/diameter switch for a circular one, and all eleven properties of the style cascade's bottom tier: unit, precision, decimal marker, drafting standard, text height, line width, arrow length, arrow form, colour, tolerance and suffix |
| ✅ | **Renamed, deleted and re-grouped** — delete **asks the orphan question** rather than refusing or guessing, because `pdfcer-core` refuses a populated group with the member count in the refusal |
| ✅ | **A group's unit is settable, and was never missing** — `set_group_scale` takes a whole `NumberFormat` and `NumberFormat.unit` is public, so the unit was reachable and merely undiscoverable. The window's unit combo goes through that path, carrying the group's own scale through unchanged |
| ⬜ | **Area** and **Angular** — the two a takeoff needs. `DimensionKind::Angular` exists in the engine and the two-line tool already authors one when the picked lines are angled, so what is missing is a *direct* Angular tool (pick an apex and two directions). `MeasureKind` has no variant for it: shell-only work |
| ⬜ | Count tool and a takeoff schedule |

| | |
|---|---|
| ✅ | **Three item sizes in the ribbon band, learned from Word by driving it** — `tools/word-ribbon-study.ps1` photographed Word at twelve widths and `tools/our-ribbon-study.ps1` photographed this shell at the same ones. At 884 client points Word put ten groups on the band and this shell put three; Word's density comes from mixing Large, Medium and Small items rather than from drawing everything one size |
| ✅ | **An item can be hidden by condition, and its space is reclaimed** — `visible_when` on a ribbon or menu item, evaluated against the same `ConditionSet` that decides enablement and applied **before** measurement, so the group re-flows and a group with nothing left is not drawn at all, separator included. This is visibility, not enablement, and R9 draws the line |
| ✅ | **A search that finds nothing says whether it could have** — a zero-result search has two causes that look identical: the word is not in the document, or the document's text was never recoverable as letters, so no word could ever have matched. The second does not look broken, because the text renders perfectly |
| ✅ | **Sections re-wrap onto a third row as you narrow the window, before anything is hidden** — the band's answer to running out of width is a four-rung ladder: natural layout, re-wrap onto up to three rows, collapse whole groups to a captioned button, scroll the band. Each rung hides strictly more than the one before |
| ✅ | **The ribbon scrolls sideways instead of hiding commands in a menu** — a `›` at the band's right edge shifts the band and a `‹` appears once it has. The overflow dropdown is **gone**, not supplemented, because two affordances for one job is a defect; the trade is stated rather than glossed, since a menu *names* what it hid and a scroll does not |
| ✅ | **…and a tab with nothing left to show is not shown either** — hiding every item on a tab used to leave an operator a tab they could click with an empty band beneath it. That symmetry is what makes a **generous** tab list usable, so a mode can name a tab it does not always need |
| ✅ | **Overprint in print-ready files is a setting you can reach** — `page_blend_space_source` arrived in the engine with no control, so it was changeable only by hand-editing `settings.txt`, which is not a user interface; this shell's own coverage gate caught it. Filed under **Colour**, by the symptom that sends somebody looking |
| ✅ | **Grey over a spot colour has its own setting** — a grey fill that overprints a spot colour either knocks the spot out or lets it show through, and the standard genuinely does not say which. The coverage gate caught the new axis within one dependency update, which is the mechanism doing what it was built for |
| ✅ | **A fit survives a window resize, and a pan gets you out of it** — press Fit page, Fit width or Fit height and then resize the window or drag a dock panel: the page re-fits **and** stays centred, where it used to re-scale correctly and stay anchored wherever it happened to sit. A pan leaves the fit, so the position you chose by hand is not overruled by the next resize |
| ✅ | **Inkscape-style scale switches** — three checkboxes on the Tool row, live whenever the Select tool is armed: **Scale line weight**, **Keep the inner margins the same size** and **Allow the artwork to distort**. All three ship off, which is the default Acrobat, Illustrator and Inkscape all use |
| ✅ | **Edit ▸ Content is two verbs, not three** — *Edit text* and *Add text*. The third, `edit.objects`, is **deleted** rather than wired: it was a route whose tooltip described the Points tool while it armed the Select tool. The rule it leaves behind is that a route's target is derived from what the operator will SEE HAPPEN, never from what the source command is called — and no route table can catch that, only reading the two side by side |
| ✅ | **Right-click a form field and you get the field's menu** — a right-click on a text box used to offer four zoom levels, because the canvas had exactly two questions (*is an object under the pointer* and *is anything*) and a form field is neither: it is not in the object selection at all, by design |
| ✅ | **The first driven context menu in this project's history** — canvas menus existed since Phase 1 and not one check had ever opened one: everything asserted about them asked whether the manifest *would* offer something, which is a different question from whether a right-click on a pixel opens anything. The harness had no `right_click_at` to call |
| ✅ | **The comments you write are signed and dated** — `/T` and `/M` were never written, so the author column in Acrobat's comment list and in this shell's own Comments panel was blank on everything pdfcer made. A comment nobody signed is a comment nobody can answer. Settings ▸ Comments carries the name |
| ✅ | **A page's colours no longer change with the zoom** — a page that declares a subtractive blending space is composited in a four-colorant buffer at 20 B/px, and past a ceiling the engine refuses it, composites in sRGB and says so, which moved the patches. The ceiling is now disclosed rather than met silently |
| ✅ | **The chrome survives a narrow window and a large UI scale** — at `ui_scale = 1.80` the zoom control, the fit buttons, Find and the selection filter were off the left edge at negative coordinates, with the left-hand notes drawn underneath the fit group, and the bar was two points shorter than its own controls at every scale |
| ✅ | **The Properties panel scrolls, and its Apply button can be reached** — the panel drew its sections straight into the dock tile with no scroll area around them, so with an object selected the Apply button was below the bottom of the tile and had never been pressed by anybody |
| ✅ | **Two-row ribbon bands** — `render_group` emitted a single `ui.horizontal`, so controls never wrapped however wide the group's content was; `GROUP_ROWS = 2` wraps them |
| ✅ | **…and the group padding is drawn** — controls sat flush against their group box and the separators, measured at an inset of 0.0. It cost **zero** width budget, because `plan::GROUP_PADDING` had been budgeting 6 pt a side that the renderer never drew |
| ⬜ | **…and the remaining overflow is not a layout problem** — Style and Comments still overflow at 1,100 px and no row count can fix them, because they are one-item groups and have nothing to wrap. That is the Markup-tab taxonomy question, and it is the operator's, not a defect |

### OCR

The engine recognises text end to end — `ocr::layer` writes the invisible
sandwich at render mode 3 — and the licensing question is answered: the model
ships in this repository with credit. What remains when OCR lands is one line
in `PAYLOAD_ASSET_DIRS`, one section copied from the engine's `about.hbs` and
one `Attribution` entry, and **the gate refuses the first without the other
two**, which is what makes *with proper credit* enforced rather than
remembered.

| | |
|---|---|
| ✅ | **Find offers OCR when there is no text to find** — the trigger is `readout == Empty` **and** `!page_has_extractable_text()`, the second behind a short-circuit so it is never evaluated unless a search already came back empty. The falsifying test is the one that matters: `an_ordinary_empty_result_on_a_text_page_offers_nothing` |
| ✅ | **OCR is an edit to the document you have open** — `add_ocr_layer` used to take an immutable `&Document` and return a whole new PDF, so recognition was the one capability that could not be undone, saved over, or applied to the file in front of you |
| ✅ | **Recognise more than one page** — All pages, this page, or a typed range. **All pages is first and is the default**, which is what every surveyed recogniser does |
| ✅ | **A page that already has text is skipped by default, and that is a hazard guard rather than a preference** — a second pass over a recognised page took it from **427 character codes to 854**. `add_ocr_layer` *adds* a layer rather than replacing one, and the layer is invisible, so nothing on screen changes and nothing warns |
| ✅ | **OCR in Read mode** — `file.ocr` is reachable in Read and the driven check asserts that specifically. Recognition is an edit, so what governs it in Read is whatever governs every other edit |
| ✅ | **…and it lives on File ▸ Recognise, not Tools ▸ Recognise** — Tools is not in Read's tab list, so a command refused where the operator needs it means the **tab** is wrong |
| ✅ | **More scanning resolution makes OCR worse, and 300 DPI is the worst of five** — on the benchmark CAD sheet against its own vector text as ground truth: 72 → 56.5 %, 100 → 56.7 %, 150 → 54.5 %, 200 → 53.9 %, 300 → 35.1 %. `ocrs` resizes every image to its model's input size, so extra pixels are thrown away after costing time |
| ✅ | **A run says how far it has got, and Stop and Cancel mean different things** — pages done and characters found while it runs; **Stop** keeps what has been recognised so far, **Cancel** throws it away |
| ⚠ | **Recognition quality on real scans is still unproven, and the gap is narrower than it was** — a genuinely scanned document exists on this machine, an 883-page image-only parts manual at `/Rotate 270` throughout, on which the recogniser returns about 440 words a page at 2.6 s a page. What is still missing is a ground truth to score those words against |
| ✅ | **What the operator is told about confidence** — `ocrs` scores nothing, so the dialog recognises and **discloses**. The force behind the disclosure is `Ctrl+Z`, named in the outcome sentence for that reason, because the layer is already in the document by the time it is read |
| ✅ | **An attribution surface for shipped assets** — `PROVENANCE.md` per asset directory, `about.toml` and `about.hbs` generating `THIRD_PARTY_LICENSES.md` into the package, a **File ▸ About** dialog, and a `check-shipped-assets` gate with a seven-check self-test |
| ⚠ | **…and building it found three third-party works already being redistributed with no notice at all** — a pre-existing gap rather than one OCR introduced: the 14 Foxit CFF faces (BSD-3-Clause), the Adobe Core-14 AFM metrics (APAFML) and the Adobe Glyph List (BSD-3-Clause), all compiled into the binary. They appear in the notice now |

**Read may produce a new document; it may not modify this one.** That is the
general form of the rule OCR's placement was argued from, and it is the better
one: it covers OCR and settles in advance every future capability of the same
shape — flatten, redact-apply, PDF/A convert, page export. It sits alongside
the two exceptions Read carries, **form filling** and **OCR**, and explains why
both are exceptions rather than inconsistencies: neither changes the document
the operator was given. It becomes load-bearing the day an in-place `Save`
lands. There is one save command today, `file.save_copy`, and the label is
load-bearing: pdfcer never overwrites the original unless the operator picks
it, because a button labelled `Save` promises in-place saving, which cannot
ship before autosave and crash recovery exist.

### Commands that were registered and inert

| | |
|---|---|
| ✅ | **Save a copy** — `file.save_copy` was registered, on the quick access toolbar, bound to `Ctrl+S` and printed the chord in its own tooltip, with **no dispatch arm**: nothing this shell built could be written to disk. It writes an incremental save |
| ✅ | **New PDF** — `file.new`, `Ctrl+N`. The engine has no document-creation verb and `document.rs` carries a named permanent invariant forbidding one, so a **443-byte blank A4 template** ships as an asset and New opens it |
| ✅ | **v0.1.0 released** — `github.com/KenM76/pdfcer-gui`, public and MIT, with a portable Windows x64 zip that needs no install. Engine dependencies moved from relative paths to a pinned git revision, verified byte-identical to the source every binary in the session was built from |
| ⚠ | **An audit after that release found the save defect was not isolated** — the commands with no dispatch arm and no guard arm included `edit.undo` and `edit.redo`, on the QAT and bound to three chords, and every page operation, six of which the Pages thumbnail context menu offered |
| ✅ | **Undo and redo** — the same shape as `file.save_copy`, and worse once saving worked, because every authoring feature became reachable by an operator with no way to take any of it back. Both go through `vector_edit` |
| ✅ | **…and the check does not rest on counts, because a plant proved counts are not enough** — a build that mutated the session but never bumped the epoch had every annotation count already correct and left the canvas showing the undone state. It was caught by two oracles from other subsystems: `objects n=` is emitted once per page and epoch, so a new line means the epoch moved |
| ⬜ | **Undo tooltips do not name the operation** — `undo_kind()` returns the `CommandKind` and the catalog could phrase it, but `Command::tooltip` is a `String` fixed at registration and `CommandRegistry` exposes no `get_mut`; it lives in `egui-shell`. Rebuilding all the commands every frame for one string would also change an icon-only control's accessible name under the pointer |
| ✅ | **Page operations — rotate, delete, move up and down, extract** — the Pages context menu has no inert row left. `resync()` is a single choke point hung off `vector_edit`'s success path rather than off the four arms, so undo gets the page refresh for free |
| ⚠ | **No Pages command is registered and inert.** `pages.merge_into` has a live dispatch arm (`app/dispatch.rs:275`) and ships as Pages ▸ Merge into this document; `pages.split` is **not registered at all** — it sits in the PLANNED register, because R9 makes an unbuilt capability render nothing. `pages.insert_from_file` ships |
| ✅ | **Command reachability is checked** — `shell::commands::reach`: every registered command must reach a dispatch arm or carry an **argued** entry in `SCAFFOLDED`. A reason under 40 characters, or one that merely restates the id, is refused, as is an entry whose command has since been wired |
| ✅ | **…and the dispatch arms for commands that do not exist are gone** — `view.zoom_in`, `view.zoom_out`, `view.next_page` and `view.prev_page`, each deleted only after **both** of its live routes was verified individually, the keyboard layer and a status-bar control. `UNREACHED_ARMS` is empty and kept, its length pinned at 0, so a fifth cannot be added quietly |
| ✅ | **Read mode and Full screen** — `Ctrl+H` and `F11` had a control, a glyph, a group and a keymap entry each, and no behaviour. **Read mode is not a duplicate of the Read *mode***: they are orthogonal and compose, `mode.read` being a capability set and `view.read_mode` a chrome stance |
| ✅ | **Render diagnostics is a report** — the status bar keeps its one-line disclosure and the dialog gets the room; both read the **same** derivation, so the editorial rules are stated once |
| ✅ | **…and it says what colour the page was blended in, and who decided** — *Blended in CMYK ink* or *Blended in screen colour (RGB)*, and under it the origin: the page's own `/Group` said so, the screen's colour stands because the page declared nothing, or the file's print output intent decided it |
| ✅ | **A blank drawing at deep zoom says what to do, and the engine stopped printing its own crash text on it** — running out of raster room used to kill the drawing worker outright, then refused politely while painting a slice-index panic message onto the canvas |
| ✅ | **The status bar says when a comment is on the page and pdfcer drew nothing for it** — an annotation that arrives without a baked appearance stream renders as nothing at all, and there is no symptom to notice, because clean paper is what an unannotated drawing looks like |
| ✅ | **…and when a page brought no resources of its own** — a page with no `/Resources` on itself or any ancestor used to cost the operator the whole document, on every verb that walks the page tree |
| ✅ | **Document properties stopped saying your signatures were broken** — the Stamps section printed *"names no page in this document"* against **every** stamp whenever the file's page tree could not be read. The sentence was not merely unhelpful, it was false |
| 🔨 | **`view.show_points`** — an object's anchors drawn without descending into it, registered, on View ▸ Display and wired as a `ViewChrome` toggle. Its recorded blocker had gone stale before it was wired: `canvas::overlay::draw_anchors` and the objects provider's `object_node_points_of` and `subpath_node_points_of` already enumerated a part's anchors, so what it needed was wiring rather than a mechanism. A driven check is still owed |
| ⚠ | **View's seven groups overflow at the shipped 1,100 px default** — found by driving. Only `page_display`, `render` and `navigate` fit; `zoom`, `display`, `panels` and `window` fold into the band's overflow, so Read mode and Full screen are reachable at that size only through the overflow or their chords |

### Phase 6/S6 — rendering

| | |
|---|---|
| ⛔ | **Deep zoom via `render_page_region`** — the API shipped, but per-viewport regions cost about 700 ms each on a dense sheet. **Blocked on a reusable parsed handle**, filed with the engine. Without it this trades smooth pan for zoom range |
| ⬜ | **`MAX_ZOOM` from measured performance** — not from `f32` (sub-pixel accuracy holds to about 5,000×) and not from the pixmap guard |
| ❌ | **Tiled rendering — cancelled.** A one-point-square region costs 691 ms, so a 3 × 3 ring is a 9× regression |

### Standing backlog — shell-only work

Exists in `pdfcer-core` or `pdfcer`; needs a surface, not an engine.

| | |
|---|---|
| ⬜ | **Pages export as PNG, JPEG, SVG or EMF** — `ImageFormat::ALL` is `[Png, Jpeg, Svg, Emf]`, each with its own writer. Built and undriven |
| ✅ | **The Attachments panel** — `attach_file`, `detach_file`, `list_attachments_with_notes`, `extract_attachment` and `sanitize_attachment_name` had all shipped in the engine, with fixtures and a hazard analysis, and this shell had no way to reach any of them |
| ✅ | **A text tool for Edit** — `CanvasTool::Text`, armed by `view.tool_text` beside the hand tool in View ▸ Navigate. The reference applications disagreed and the disagreement is the interesting part: Acrobat and SolidWorks resolve text-versus-object contextually inside one tool, Inkscape uses a separate Text tool, and Inkscape won |
| 🔨 | **Annotations are selectable** — one answer to four separate reports: editing a placed stamp, reaching dimension-group editing by clicking, and two about basic things not being enabled |
| ✅ | **A placed form field's properties are editable** — Required, Read only and a tooltip for every type, plus multiple lines, hide as typed, equal cells and a maximum length on a text field. The pane used to answer a click by offering to delete the field |
| ✅ | **Flatten works from the ribbon** — `edit.form_flatten` sat on Edit ▸ Forms with an icon and an enable predicate and no dispatch arm, while the Forms panel's own Flatten button called `flatten_fields` perfectly well |
| ✅ | **The Format tab's Font group, and three surfaces that say how to reach it** — sweep text and the contextual Format tab carries a Font band: face, size, Bold, Italic, colour, the same five controls as the Properties panel |
| ✅ | **Restyling existing text** — sweep text on the page (press **T** to arm the text tool, then drag) and Properties grows a *This text* section: font, size, Bold, Italic, colour |
| ⛔ | **The rest of the Format contextual tab** — twenty-four property editors across six selection types are specified; the tab can carry the ones whose engine verbs exist, and the remainder wait on them |
| ✅ | **New at a chosen page size** — an engine gap wearing a shell gap's clothes: nothing in `pdfcer-core` wrote a `/MediaBox`, so the only implementation available to a shell was one checked-in template asset per size |
| ⬜ | **An open drawing's sheets can be put on a different size of paper** — `pages.resize` on Pages ▸ Transform, beside Rotate. Pick sheets in the Pages panel or none at all, the same operand rule every other `pages.*` command uses. Built and undriven |
| ✅ | **A crosshair pdfcer draws itself** — `CursorIcon::Crosshair` resolves to Windows' monochrome `IDC_CROSS`, whose colour belongs to the operator's pointer scheme, so on a white sheet it is invisible. This one is drawn by the canvas |
| ✅ | **An icon in the executable** — Explorer, the Start menu, the pinned taskbar entry and the *Open with* dialog all read it **without running the program**, so it has to live in the PE image's `.rsrc` section |
| ⬜ | Imposition in the print dialog · insert blank page · push-button field creation |
| ⬜ | Script-driven-field census · unencrypted-wrapper warning |

### Built, and not yet driven

Every row here was verified at source — a registered command id, a reachable
control, or a dispatch arm that does the thing — and none has been exercised in
a running binary. Under this file's bar that is not a tick, however green the
suite is. These are the runs the next session with the machine to itself owes,
in slices of six to eight.

| | |
|---|---|
| ⬜ | **The Format tab has a Markup band** — five custom controls in `app/markupband.rs`, on `fontband`'s architecture: line colour, fill (with *No fill*), line width, opacity and arrowheads |
| ⬜ | **Fill (`/IC`) and arrowheads (`/LE`) have a surface** |
| ⬜ | **A markup shape has a right-click menu** — `canvas.markup`, the sixth canvas context |
| ⬜ | **A markup can be positioned by typing, duplicated, nudged and re-stacked** — typed X, Y, W and H for a selected annotation on the convention Properties already uses for content; `Ctrl+D` duplicates; arrow keys nudge; four z-order commands |
| ⬜ | **The default colours are Adobe's, measured from its registry rather than remembered** — `ACROBAT_DEFAULTS.md` holds the float triples, the hex, an adopted-or-declined verdict per row, and the command to re-run the read |
| ⬜ | **A text box's painted words going stale is disclosed** — narrowed to what the engine still leaves behind now that the note verb re-bakes `/AP` `/N` |
| ⬜ | **Open in the mode you were last in** — dead since the day it shipped and reported *working* by the start-up trace the whole time: `PdfcerApp::new` built `RibbonState` **before** calling `modes::start` and seeded it with the manifest's first mode |
| ⬜ | **A sticky note, text box or stamp's style controls commit** — the colour swatch and the opacity row appeared, live, and every press was refused, because `spec_from_dict` matches ten subtypes and `/Text`, `/FreeText` and `/Stamp` were not among them |
| ⬜ | **`visible_when` is read by the menu resolver** — `menu::plan::resolve` never called `Item::visible_condition()`; only the ribbon did, so **every menu row meant to vanish was greyed instead, R9 inverted** |
| ⬜ | **One edit draft is no longer shared between a content object and an annotation with the same number** — the geometry panel keyed its draft on page, index and epoch, and content objects are numbered by **paint order** while annotations are numbered by **object id**, both small integers |
| ⬜ | **`Alt+Arrow` no longer nudges and pages at once** — egui's `consume_key` matches with `matches_logically`, which **ignores extra Shift and Alt**, so a bare-arrow consumer also fires on `Alt+Arrow`, which the keymap binds to `pages.move_up`. The nudge reads `i.modifiers` itself |
| ⬜ | **A sticky note can be read on the page** — a pop-up with the author, the date and the words, and a hover tooltip before you click. It works in Read, structurally: the pop-up runs its own hit test rather than borrowing the selection |
| ⬜ | **The Comments panel can be used, not just read** — delete, filter by author, kind and has-words, sort, and a Go to that navigates and opens the comment's pop-up on the same frame |
| ⬜ | **Sticky notes, stamps, text boxes, links and attachments copy and paste** — the clipboard is the engine's own `copy_selection`; all five used to answer `Ctrl+C` with *that annotation is not one pdfcer authors* |
| ⬜ | **Redaction stopped costing the undo stack** — applying **arms the next save** instead of rewriting at the click, so the base, the overlay and the whole undo stack survive, and a Cancel disarms it |
| ⬜ | **pdfcer can say whether a signature's bytes were altered, and whether you trust who signed it** — the operator's own Acrobat trust list, imported from Settings ▸ Digital signatures, with a live resolved-state line and an Inspect |
| ⬜ | **A drawing can be given a password, and told what it allows** — File ▸ Security, both controls Large, on the File tab, which is present in all three modes |
| ⬜ | **The colour of text you clicked on** — a colour control on a **selected** text object rather than only a swept range; both previous controls greyed on a click. Multi-object recolour carries a real indeterminate state |
| ⬜ | **Selecting an object says which layer it is on** — in the Layers panel as a plate on the matching row, and on the status line, both reading the same name from the same function. Three-valued on purpose: on this layer, on none, or on several |
| ⬜ | **The print preview pops out into its own window** — a second OS viewport, resizable, with its own taskbar entry, and the preview column inside the dialog collapses to nothing while it is out, so the room is given away to the options column |
| ⬜ | **Embedding pdfcer's own fonts is your choice** — a checkbox, off when the window opens, drawn only when the bundled faces would actually change the outcome. The reason is a licence, not a rendering preference |
| ⬜ | **An object 0.85 pt across can be moved** — the grip anchor box is pushed outward by `max(0, (20 pt − extent) / 2)` per side, so the objects with least body to spare stop being floored to a size at which they had none |
| ⬜ | **Text comes out; nothing goes back in** — `file.export_text` on File ▸ Export, and the absence of an import was proved on all five routes: no command in the registry, no dispatch arm, no dialog, no `.txt` filter on any picker |
| ⬜ | **A drawing can be pasted into Word as lines, not as a picture** — `edit.copy_as_vector`, four formats in one clipboard transaction in preference order: SVG, EMF, PNG, DIBV5 |
| ⬜ | **Pages come out as PNG, JPEG, SVG or EMF** — PNG carries straight alpha and real physical resolution; JPEG has a worded refusal for a transparent page rather than a silent flattening |

### Where the engine's `gui` column and this shell disagree

`D:\Dev\pdfcer\docs\FEATURES.md`'s `gui` column is this project's acceptance
criteria (R6), so a disagreement is worth finding in both directions. Every row
below was decided by reading this shell's source — a registered command id, a
dispatch arm, a call site — not by reading either document. **No `[x] gui` row
has been found to be a false tick**: every backticked command id the engine
names was diffed against the live registry, and the one that is absent,
`pages.split`, already has an empty box.

| engine row | who is right, and how it was determined |
|---|---|
| undo-preserving deferred redaction | This shell has it. All four verbs are consumed in non-test source, and `edit.redact_apply` is registered and dispatched |
| verify a signature's integrity and coverage | Neither document was right. `verify_all` **is** called from `trust/mod.rs` and the panel prints *Intact:* |
| encrypt, set permissions, remove encryption | This shell has it. `file.encrypt` and `file.permissions` are registered, dispatched and on File ▸ Security; all three verbs are called from `protect/mod.rs` |
| import an Acrobat trust store | This shell has it. `trust/mod.rs` calls `trust_store::load_from_path`; the control is in Settings ▸ Digital signatures |
| evaluate trust against those anchors | This shell has it. `verify_all_with_trust`, reported per signature |
| the persistent use-the-Acrobat-trust-store setting | This shell has it. Settings ▸ Digital signatures, off by default |
| deterministic RFC 5280 path validation | This shell has it, with the caveat its own row states |
| export to PNG and JPEG | Neither document was right. `file.export_image` is registered, dispatched and on the ribbon |
| export to SVG | Neither. Same window, same row of radios |
| export to EMF | Neither. This project's backlog held it open on the ground that it had not been driven, which is a different bar from the one that file uses everywhere else |
| copy page content to the clipboard as vector | Neither, and the most stale row in either document: the two recorded blockers, a workspace crate that would have to be created and `forbid(unsafe_code)`, are both gone — `crates/native-clipboard` exists |

Two engine rows have the right checkbox and the wrong prose, reported to the
engine rather than corrected here, because `D:\Dev\pdfcer` is read-only from
this tree:

- the split row says `pages.split` is drawn on the Pages tab and falls through
  to `command-unimplemented`. It is not drawn anywhere and there is no such
  fall-through: it sits in the PLANNED register and R9 makes an unbuilt
  capability render nothing. The verdict is right, the mechanism is not;
- the maintenance note calls Flatten the worked example of a scaffolded, inert
  ribbon command. `edit.form_flatten` is registered, has a live dispatch arm
  and is on the ribbon.

A capability list is written by the session that built the thing, and the
sentence about what is *missing* is the half nobody re-reads. Ticks get
flipped; the prose beside them does not.

### Dragging a tab off the strip into a new window

The one open scope question, and it is open because the two applications an
operator would compare it to **disagree about what it means**.

| | what dragging a tab off does |
|---|---|
| Chrome, Firefox, VS Code | the document **moves** to a new window, fully functional |
| Acrobat's `Window ▸ New Window`, Bluebeam's split | a **second view** opens and the document stays where it was |

Where the product class converges, the convergence is the specification. Here
it does not, so this is a decision rather than a derivation, and the two cost
very different things. A **second view** (canvas, page list and status bar,
with the document staying in the strip) is roughly a day, and it delivers the
original two-page-lists request from another direction. A **full move** needs
every window to own its ribbon state, dock arrangement, mode and panel state —
a `Workspace` extraction — because a document that has left the strip must
still be editable, and all of that is application-scoped today. Building the
wrong one is worse than building neither: a window the document cannot be
edited in is a trap, and a second view that pretends to be a move is a lie
about where the operator's work is.

### Not salvaged yet

Present in the old shell, not yet carried across; none is a rewrite.

| | |
|---|---|
| ✅ | **Print dialog** with live preview — split into five files at the three questions it answers. Fixed on the way: stale device capabilities after a printer change, `/Rotate` ignored so a rotated page was planned portrait and rendered landscape, and a texture-name collision |
| ✅ | **Printing reaches a printer** — the dialog used to open, say *This build cannot reach a print device*, and draw no printer selector, no preview, no page controls and no commit button, on a machine with twelve printers installed |
| ✅ | **Paper size, tray and the driver's own Properties…** — all three were one engine surface, reached from beside the printer drop-down as every other program on this desktop does it |
| 🔨 | **Measure tools** — salvaged whole as `canvas/measure/{pick,scale,state}.rs` plus `canvas/snap.rs`, with every test carried and no engine API moved. Linear, two-line and radius/diameter place dimensions and the snap query is wired |
| ✅ | **Redaction — mark, review, apply, with the true-removal proof salvaged** — mandatory rather than convenient: core has **no verdict type**, `apply_redactions` returns a *report*, and `pdfcer` exits SUCCESS on a file it never verified, so the proof came across from the old shell with the code and is unskippable four ways |
| ✅ | **Markup ▸ Style** — the manifest had declared `Item::custom("colour_swatch")` since S2 and **no renderer ever matched that kind**, so the shell reserved the item's space, the application declined to draw it, and the band was a caption over nothing |
| ⬜ | **Canvas drag-to-mark for redaction** — panel marking (whole page, literal text, pattern) covers all three routes the tooltip promises, none of which is a drag. The gesture needs a `CanvasTool` variant, a capability entry, an Escape rung, an overlay preview and an `Action` carrying page-space quads |
| 🔨 | **Forms** — fill works in all three modes, and Reset and Flatten are reachable **through the panel** as buttons in its own collapsed sections, calling `reset_form` and `flatten_fields` |
| ✅ | **Fill a field on the canvas** — click the widget on the page and type; Enter or clicking away commits **one** `FormEdit::FillText`, Escape abandons the draft without also ascending a selection rung. Check boxes toggle, radio widgets select that widget's on-state. **No `CanvasTool` variant**: a widget's `/Rect` is a region the file itself marks interactive |
| ✅ | **What a fill inferred is in the status bar** — `applied_autosize` and `unencodable_chars` are the only two facts of a fill not re-derivable from the saved document; afterwards they look exactly like the author's decision |
| ⬜ | **Highlight fillable fields** — a View ▸ Display toggle tinting widget rects, Acrobat-style. Today the cursor is the whole discovery mechanism, which is honest but weaker, and it is the only thing that would make an **undrawn** field discoverable |
| ✅ | **The shell's home-grown text locator is deleted** — `Reading::find`, a longest-contiguous-glyph-stretch walk built on a mechanism this project invented and never measured |
| ✅ | **…and so is the second one** — a pinned restyle used to need a `find` string naming a contiguous sub-range inside the operator, so this shell sliced each run's text into per-operator pieces to build them, which was a second locator living beside the engine's |
| ✅ | **The font list stopped being a guess** — both face choosers now list exactly the faces `EditSession::set_font` has already said it would accept **for this run**, through `preview_font_resources` |
| ✅ | **Bold reaches the covering face on a page that has one** — a driven Bold press found the engine refusing synthesis because *a real bold face is available* while naming one that remaps `o` to a bullet and cannot show the run |
| ✅ | **A box's border and where it is shown** — the *This box* section carries all four of `WidgetEdit`'s properties: position, size, border style and width, where it is visible, and the caption |
| ✅ | **A form field's box can be moved and resized** — X, Y, Width, Height and a caption with an Apply. **Moving is free and resizing is not, and the pane says which you are about to do**, because §12.5.5 derives the appearance matrix from the appearance box's corners |
| ⚠ | **The Properties pane is taller than its dock slot**, and that is the operator-visible cost of the two rows above: a selected form field draws about 450 pt of content into a slot that is about 180 pt on an 1,100 × 800 window |
| ✅ | **Form data imports, so the round trip closes** — File ▸ Export ▸ Import form data reads an FDF, XFDF or CSV and sets this document's field values from it, with the **extension deciding the parser** exactly as it decides the format on the way out |
| ✅ | **Form data exports as FDF, XFDF or CSV** — one save picker, and the extension decides the format, which is how every application on this desktop does it and is one modal window rather than two |
| ✅ | **Field list in tab order, and it reorders by dragging** — a per-page, widget-level view beside the fill list, which stays in `/AcroForm /Fields` order because that matches the printed form. Drag a row, an insertion caret shows where it lands, and release permutes the page's `/Annots` through `EditSession::reorder_annotations` |
| ✅ | **Reordering tab navigation** — read ⛔ for nineteen days on the ground that no verb could do it; `EditSession::reorder_annotations(page, &[ObjId])` shipped and it was wired the same morning |
| ✅ | **Settings dialog** — `file.settings` was registered, drawn on File ▸ pdfcer and inert, and nine of its thirteen settings had never been read by anything. All thirteen spec-ambiguity choices are reachable, in seven groups |
| ✅ | **The page cache was a frame buffer with extra steps** — `StripRasters::retain` took the **visible** page set and dropped everything outside it, every settle, so turning back one page re-rendered it |
| ✅ | **The seven `view.*` settings — two built, five deleted.** All seven were registered, drawn on the View tab and inert; checked against the engine, **render strategy** has no tiled-progressive path in this shell and **thin lines** and **antialiasing** had no `RenderOptions` field at all |
| ✅ | **Line weights off — the CAD hairline display mode** — the one control in this file that was deleted and came back, on the operator's report that it had been removed and had never worked before that |
| ✅ | **Text box, sticky note and stamp** — all three were registered, drawn on the Markup tab and had no dispatch arm for the whole life of the project; the recorded reason was accurate rather than an excuse, being a different gesture (place, then type) from the geometric kinds |
| ✅ | **Revision clouds** — this row is kept because of what it got wrong: it read that the cloud primitives appear nowhere in `pdfcer-core`, which was true when it was written and had stopped being true without anything failing. **A blocker naming a repository this project does not build has to be re-read, not remembered** |
| ✅ | **Set the scale by measuring something on the drawing** — the model had been there since the measure salvage and the dialog's own header had already named the gap |
| ✅ | **UI scale** — a capability switched off rather than never built: `egui` has offered `Context::set_zoom_factor` throughout and **no line in this crate ever called it** |
| ✅ | **The freehand tolerance follows the pen again** — `ink::SIMPLIFY_TOLERANCE_PTS` was a `const` equal to a quarter of the pen width, and the derivation is not decorative: Ramer–Douglas–Peucker bounds how far the drawn centreline can move, so ε has to track the stroke's half-width or a wide pen loses its shape |
| ✅ | **The opening view — two more preferences.** `ViewState::default` held fit-page and all three View ▸ Display overlays off as compiled-in constants: the toggles existed, the default was not settable |
| ✅ | **…and the copy contract now covers the newest group** — `every_setting_states_its_silence_and_its_radius` listed exactly the `pdfcer_core::settings` entries, so a test reading as though it covered the window was checking a subset of it |
| ✅ | **`userdata/preferences.txt`** — the shell's own store, beside `settings.txt` under the same roof so the update instructions cover it unchanged. Separate from `pdfcer_core::settings` because every entry in that file cites a clause the standard leaves silent, and *how sharply a page is drawn* cites nothing |
| ✅ | **Icons** — 72 glyphs, rasterized at physical pixel size, tinted from the theme, with every key named by a command asserted to resolve against the live registry. An unknown key draws a **visible slashed mark**, never a blank, and the label fallback is decided upstream of the painter |
| ✅ | **Every ribbon control is a glyph or a recorded refusal** — it was 47 named and 41 bare with no rule behind which was which, so a band drew pictures and words side by side and the ribbon read as half-finished because it was. Each refusal is argued at its own registration |
| ⬜ | **Text editing** — the whole tool |
| ✅ | **A refused text edit says why, in the operator's terms** — driven on his own file at his own typo, where text added in this session was editable and text that arrived with the document was not |
| ✅ | **A click on a page that has any text can start new text** — `hit_test` is bounded to one line-height, so a click far from any run is an origin rather than a miss that captures the page |

---

## Not planned, and why

| | |
|---|---|
| ❌ | **A Home tab** — would mirror commands across tabs, and a command in two places is a command with two enable predicates that drift |
| ❌ | **Automatic reflow on edit** — reflow invents line breaks the file never stated. It is a **command**, and will not become automatic |
| ❌ | **Tiled rendering** — measured as a 9× regression |
| ❌ | **JavaScript execution** — standing refusal |
| ❌ | **Provisional styling on the canvas** — disclosure lives off-canvas, and a second rendering path for the same content drifts from the first |
