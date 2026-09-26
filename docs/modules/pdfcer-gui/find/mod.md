# find — searching the page text, and showing the operator where it is

The whole of Find, across three files:

| module | subject |
|---|---|
| this one | the query and its options ([`FindState`]), the one place a search is run ([`search`]), the wrap rule, and what the position readout says ([`FindState::readout`]) |
| [`bar`] | the floating box: where it sits, the field, the step buttons, the readout, the options menu, and the three keys the field owns |
| [`reveal`] | how one hit reaches the operator's eye — the two-frame handshake, the scroll solve, and the projection out of PDF space |

## ★ The trap, stated first because it is the whole reason this module
is written the way it is

`EditSession` has two search verbs and they are **not** interchangeable:

| verb | wildcards |
|---|---|
| `find_text(needle, case_insensitive)` | **on** — it passes `with_wildcards(true)` |
| `find_text_with(needle, &options)` | whatever the options say; the default is **off** |

In wildcard mode `#` matches any ASCII digit and `?` matches any single
character. **The old pdfcer shell's Find bar ran through `find_text`**, so
typing a `?` into it matched every character on the page and nothing said
why. `pdfcer-core` records that this was fixed *in the front end*, on
purpose: `find_text`'s pattern behaviour is its documented contract and
silently changing it would move results under every existing caller, so
what moved was [`pdfcer_core::edit::TextSearchOptions`]'s **default**, and
a front end that wants patterns now has to ask.

So this module calls **[`pdfcer_core::edit::EditSession::find_text_with`]
and never `find_text`**, wildcards default to off, and the control that
turns them on is labelled with what `#` and `?` do
([`crate::text::find::wildcards`]). `tests::the_default_search_is_literal`
and `tests::a_wildcard_search_is_only_ever_asked_for_explicitly` are the
regression tests; either one fails if someone reaches for the shorter
verb.

### The hazard next door, left where the next person will find it

`EditSession::mark_redactions_by_search` matches **literally** while
`find_text` patterns. A future *"redact every hit"* button built on a
wildcard search would therefore highlight hits that the redaction then
declines to mark — the highlight and the removal disagreeing about which
text exists, which on a redaction is not a cosmetic difference. **This
build has no such button.** If one is added, it must either force
wildcards off for its own search or refuse while they are on;
[`crate::text::find::wildcards_tooltip`] already tells the operator the
two match literally-versus-not, so the words exist and only the control
is missing.

## ★ Where the bar is, and why

**A compact box floating over the top-right of the page**, drawn as an
`egui::Area` positioned from the canvas viewport's own rect — not from the
window's, so a dock opening moves it with the page rather than leaving it
stranded over a panel.

That is where Acrobat Reader, Chrome's PDF viewer and Edge's all put
theirs, and matching them is most of the argument: Ctrl+F is a chord an
operator arrives already knowing, and the surface it produces should be
where their eyes go. This application is meant to replace Reader, and
Reader's Ctrl+F box is a field, two arrows and a settings dropdown in the
top right of the document view.

The second reason is measured rather than conventional. **A docked bar was
built first**, spanning the window above the status bar, and driving the
binary showed what docking costs: the bar takes its height out of the
canvas, the canvas feeds `ViewState::apply_fit`, and under *Fit page*
pressing Ctrl+F moved the zoom from **85 % to 81 %** — and back to 85 % on
close. The page jumps every time the operator goes looking for a word on
it, and jumps back when they stop. An overlay consumes no layout, so it
cannot do that: the page does not move at all.

It costs what an overlay always costs — it covers a corner of the sheet.
Two things keep that small: the box is deliberately narrow (the four search
options are behind an `Options` menu rather than laid out along the row —
see [`bar`]), and it is at the **top** right, which on a drawing sheet is
usually clear where the bottom right is the title block.

**R128 does not reach this surface, and that is a consequence rather than
an exemption.** The rule is *a panel whose size feeds a fit-to-viewport
computation has a fixed size*, and an `egui::Area` feeds no such
computation. The box's width is fixed all the same, for a different reason
of its own: it is anchored by its top-right corner, so a width that changed
with the readout's text would move every control on it. [`bar`]'s header
carries that argument.

## ★ What happens to stale results

**An edit clears the highlights, keeps the query, and says so.**

A hit is a *quad*, and a quad is a claim about where particular glyphs
are. `delete_*` excises byte spans and renumbers; `move_*` rewrites
operands. After either, a quad recorded beforehand can cover different
glyphs, no glyphs, or the right glyphs in the wrong place — and rule 4
forbids painting a mark over content that does not say what the mark
claims. There is no cheap way to tell which hits survived: the geometry
comes from a full document text extraction, so "re-check one hit" costs
the same as "re-run the search".

The three available answers, and why this one:

| answer | rejected because |
|---|---|
| re-search automatically | a search is a whole-document text extraction — 5.6 MB of CAD drawing per edit, on the frame after every nudge of an object |
| keep drawing the old hits | draws a highlight that may be over the wrong text, which is the one thing rule 4 forbids outright |
| **clear the geometry, keep the query, say so** | ✔ |

Mechanically: [`Results`] records the `edit_epoch` it was computed at,
[`FindState::readout`] returns [`Readout::Stale`] the instant that epoch
moves, [`FindState::current_hit`] returns `None`, and the overlay
therefore draws nothing. The bar shows *"Document changed"* with a
tooltip saying to press Enter. Re-running is one keypress and it is the
**operator's** keypress.

Closing the document is the harder version of the same event and is
handled by [`FindState::forget_document`], called from the same two
places `crate::panels::PanelsState::forget_document` is
(`PdfcerApp::open_path` and `PdfcerApp::close_document`) — a hit list
naming pages of a file that is no longer open is not stale, it is
nonsense.

## ★ Searching is not free, and nothing here searches on a keystroke

[`pdfcer_core::edit::EditSession::find_text_with`] runs
`text_extract::extract_document_view` over the **whole document** on
every call — every page, every content stream, decoded, tokenised and
walked, with fonts resolved. There is no cache in `pdfcer-core` and none
here. On the project's benchmark sheet (`ncored-benchmark-cad-drawing.pdf`,
5.6 MB, 129,758 objects on one page) that is a measurable fraction of a
second, and it is a *whole-document* cost that grows with page count
rather than with what is on screen.

A find bar that searched per keystroke would therefore run that
extraction once per character typed — five extractions to type `total`,
four of them for prefixes nobody asked about, each one blocking the UI
thread it is dispatched from. Incremental search is a feature of editors
whose document is already in memory as text; a PDF's is not, and
pretending otherwise is how a viewer becomes unusable on exactly the
files it exists for.

So: **a search runs only when the operator commits one** — Enter in the
field, the step buttons when the results are not current, or a change to
an option after a search has already been run (an explicit click, not a
keystroke, and one whose whole purpose is to change the hit list). Every
run reports its own cost on the `PDFCER_DIAG` channel:

```text
pdfcer-diag find needle="total" hits=47 current=1 page=3 ms=214 \
           case=insensitive whole=off wildcards=off boundary=Alphanumeric
```

…so the number in this header can be re-measured on any file by anyone,
rather than being a claim about one machine on one day.

## Actions, not mutations

Nothing in [`bar`] touches a document. Every commit becomes an
[`crate::app::actions::Action::Find`] carrying a [`FindRequest`], applied
after the frame through the one funnel — and that is not ceremony here,
it is a requirement: the search needs `&mut EditSession`, which means
`Arc::get_mut` on `OpenDoc::session`, which fails while the render worker
holds its clone. The funnel is what makes it legal to stop the worker
first. See [`search`] for that protocol and for how it differs from
`app::actions::vector_edit`'s.

## Why the state lives on `PdfcerApp` and not on `OpenDoc`

`crate::app`'s rule: *state that dies with the document lives on
`OpenDoc`; state that outlives it lives on `PdfcerApp`.* A find query and
its options outlive a document — closing one file and opening another is
the most likely moment to search for the same term again — so
[`FindState`] sits beside [`crate::panels::PanelsState`], with the same
`forget_document` seam for the half that does *not* outlive it.

The one piece that does live on the document is [`Reveal`], and it has
to: it is *view* bookkeeping that spans two frames, exactly like
`ViewFrame::zoom_anchor`, and for the same reason — the page it targets has
not been navigated to yet on the frame the request is made.
