# `app::doctabs` — the document tab strip, and the tab that springs open
under a drag

pdfcer's half of [`egui_shell::tabstrip`]. That module draws a row of tabs
and knows nothing about documents; this one turns [`Status`] into tabs,
turns the operator's clicks back into [`Action`]s, and owns the one
behaviour the shell deliberately refused: **spring-loading**.

---

## 1. Why there is a tab strip at all

Because the operator asked to *"open multiple PDFs at once"*, and every
application that does answers with tabs — Acrobat, Bluebeam, PDF-XChange,
Foxit, Illustrator, VS Code, every browser.

**Where the applications an operator already uses agree, their agreement
is the specification** — not a starting point for a better idea. It is the
standing rule for every interaction decision in this shell, and it is what
settles this one.

They agree, so this is a tab strip: labels left to right, the active one
emphasised, a ✕ on each, middle-click to close, Ctrl+Tab to cycle, an
overflow menu when there are too many. Nothing here is new and nothing here
is meant to be.

## 2. It is drawn whenever anything is open, including one document

Chrome, VS Code and Acrobat all show the strip with a single tab. Hiding it
below two documents would save 26 points of a CAD sheet and cost the
feature its discoverability — an operator who has never seen a tab has no
reason to believe a second document is possible.

**"It works and nobody can find it" is never a documentation problem.** A
capability with no visible entry point has not shipped, whatever the tests
say; the fix belongs in the chrome, not in the manual.

Its height is a **constant** ([`egui_shell::tabstrip::STRIP_HEIGHT`]) in an
`exact_size` panel, for R128's reason: a chrome surface whose height varies
above a viewport that fits a page to itself is a measured feedback loop.

## 3. Spring-loading — the gesture that makes a cross-document drag
possible

With one canvas and one Pages panel, only one document's page list is on
screen at a time. So how does a page get from document A's list to document
B's?

The answer every operator already has: **drag onto the other tab, wait, and
it opens.** Windows Explorer does it with folders and with taskbar buttons,
every browser does it with tabs, macOS Finder does it, Acrobat does it. It
is called spring-loading and nobody has ever had to be taught it.

So: while a page drag is in flight ([`crate::pagedrag`]), dwelling on a tab
for [`SPRING_DWELL`] activates that document. The drag continues — it lives
in `egui::Memory` precisely so that switching documents cannot destroy it —
and the operator drops into the newly shown page list or page view at a
caret.

### The dwell is real time, not frames

Measured against `egui`'s own input time so it behaves identically at 30
and 144 Hz. A frame count would spring in a third of a second on a fast
machine and a second and a half on a slow one.

### It is cancelled by moving to another tab, and by ending the drag

The timer records **which** tab is being dwelt on. Moving the pointer to a
different tab restarts it; leaving the strip clears it. Without that, a
pointer that swept across five tabs on its way somewhere would arrive
having activated whichever one it happened to be over when the clock ran
out.

## 4. What a tab says

[`crate::text::doctabs`] owns every string. The two decisions worth
knowing here:

- the **unsaved marker leads** the label, because the ellipsis eats the
  tail and a crowded strip is exactly when the marker matters;
- a tab whose file **failed to open** is still a tab, with the reason in
  its tooltip — see [`crate::app::documents`] §2 for why a failed open must
  not evict the operator's other documents.
