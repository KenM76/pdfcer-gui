# `ui-verify/checks/page_drag_between_documents`

`a_page_dragged_between_documents_is_copied` — the operator's whole
request, performed end to end.

> *"make it so we can open multiple PDFs at once and drag and drop pages
> from one thumbnail image sidebar to another … and insert them in between
> the pages we've dragged to."*

# The gesture, and why it is one gesture

Press on a page tile in the document that is open, walk the pointer onto
the **other document's tab** and rest there until it springs open, walk on
into the page grid that is now showing the other document, and release
between two sheets.

That is four mechanisms in one held button, and no unit test can reach any
of them:

| # | mechanism | what a unit test sees |
|---|---|---|
| 1 | the tile senses a press and publishes a `PageDrag` | a pure function it never calls |
| 2 | the drag **survives a document switch** — it lives in `egui::Memory`, not on `PanelsState`, which is reset by an activation | nothing: there is no activation in a unit test |
| 3 | a tab under the pointer for `SPRING_DWELL` activates its document | a timer no test advances |
| 4 | the release resolves a gap in the *new* document and raises a cross-document insert | two functions, separately tested, that have never met |

Mechanism 2 is the one worth naming. `PanelsState::forget_document` is
`*self = Self::default()`, and switching documents calls it — so a drag
stored on the Pages panel would be **destroyed by the very tab-spring that
makes the feature possible**. It is in `egui::Memory` for exactly that
reason, and this check is the only thing in the workspace that can observe
whether that decision actually holds.

# The assertion that says it is a COPY

`copied=1` on the release line, and the source document's page count
**unchanged**. A cross-document drag does not remove the page from where it
came: `crate::app::actions::crossdoc` §2 carries the reason, which is that
a move would be two commands on two undo stacks with no single Ctrl+Z able
to reverse it.

An operator who assumed a move would discover their source drawing intact
tomorrow — or, worse, assume it was not and delete the wrong copy. So the
caption says *copy* before the button is released, and this check asserts
the behaviour matches the promise.

# What a passing run does NOT prove

That the pages arrived at the right index. `insert-pages-landed at=` is
read and cross-checked against the gap the release reported, which rules
out the two answers that are wrong by a whole document (the start and the
end). It does not verify the *content* of the inserted sheets; that is
`pdfcer-core`'s `insert_pages` and it has its own corpus.
