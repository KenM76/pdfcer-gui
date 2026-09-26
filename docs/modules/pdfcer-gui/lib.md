# pdfcer-gui — native desktop shell (rebuild), library root

**This crate is a library with a thin binary in front of it.** The
binary (`src/main.rs`) does one thing: read `argv`, call [`run`]. Every
module, every type and every test lives here.

## Why a library at all, when the product is an executable

Three reasons, and the third is the one that bites daily.

1. **`tools/ui-verify` and any integration test can `use pdfcer_gui::…`.**
   Without a library target every assertion has to cross the process
   boundary, even when the question is really a unit-level one — "does
   this manifest validate?" does not need a window.
2. **`cargo doc` documents something.** A binary crate's rustdoc is
   empty, and this project's standing rule is that the documentation is
   the logic. Docs that cannot be browsed are docs that rot.
3. **`main.rs` stops being a contention point.** With the module tree in
   a binary, *every* new module must edit the same file — which is the
   one guaranteed merge conflict when work runs in parallel, and this
   project runs work in parallel by design. Moving the tree here does not
   remove the shared file, but it removes it from the path of the
   `argv`-and-viewport code that has nothing to do with it.

Converted at the S2 → S3 boundary, deliberately: it changes visibility
across every module, so it wants a moment when nothing else is in
flight, and it wants to happen *before* the panel modules multiply.
`PROJECT_PLAN.md` §4.2b records the decision.

## Where everything lives

| module | responsibility | headlessly testable |
|---|---|---|
| [`app`] | the one owner of state; frame composition; actions | partly |
| [`shell`] | the ribbon/mode/keymap definition, **as data** | **yes** |
| [`viewer`] | page index, zoom ladder, fit math, raster ceiling | **yes** |
| [`render`] | off-thread rasterization; pixmap → texture | worker keys only |
| [`canvas`] | drawing the page, wheel/ctrl-wheel/middle-drag input | geometry only |
| [`find`] | the search query, its options, stepping, staleness and the bar | mostly |
| [`panels`] | the dock's panel bodies, and the page object model behind them | mostly |
| [`text`] | every operator-visible string (the ui-text catalog) | n/a |
| [`diag`] | the opt-in `PDFCER_DIAG` trace channel | n/a |

The split is driven by testability: a windowed UI cannot run on a CI
runner, so every piece of *logic* that could be wrong in a way a human
would notice — an off-by-one page step, a fit scale that overflows an
axis, a zoom that blows the rasterizer's allocation guard — is pushed
into a pure function with a unit test. What is left is wiring. Wiring
can be reviewed; arithmetic needs tests.

## Privacy posture, carried across unchanged

This crate makes no network calls of any kind. The only file it opens is
the one it is asked to open.
