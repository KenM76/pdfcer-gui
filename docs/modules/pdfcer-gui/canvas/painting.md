# `canvas::painting` — everything the canvas draws, once everything is
decided


## Why this is a seam and not a size

`tools/gates/check-file-size.sh`'s own header refuses a split made to fit a
number: *"Split the module along its seams — one subject per file."* The
seam was already written into `interact`, as the numbered sections of its
own body:

> 1 the pointer · 2 what a press would land on · 3 advance the gesture ·
> 4 the decomposition · 5 apply the gesture · 5b the right-click · 6 keys ·
> 7 re-resolve · **8 draw**

Sections 1 to 7 answer *what happened and what does it mean*. This answers
*what does that look like*: it reads values the first seven produced, writes
nothing but pixels, raises no `Action`, and makes no decision.

What stayed behind in section 8 is everything that is **not** painting — the
typing loop, the keyboard-ownership check and the cursor icon — which had
ended up under the same heading because they run at the same moment, not
because they are the same subject.

## The layer order IS this module's content

Every position in the sequence is an argument, and each one travelled here
with the code rather than being summarised:

| layer | why it is where it is |
|---|---|
| **OCR veil** | under everything, the grid included: the only thing here whose subject is the *raster*, and it fades the picture of the page without fading the page |
| **grid** | under everything else: the only other thing here about the *paper* rather than about something the operator has selected, searched for or is dragging |
| **OCR text** | above the grid because it is page content the file carries and does not draw; below the find wash because a search answer outranks anything merely on the page |
| **find highlights** | a wash answering *where is the text I asked about*, under the outline, which is a statement about what a verb would act on |
| **selection outlines**, **grips** | |
| **marquee** | |
| **guides** | on TOP of the selection — a guide is a line the operator aligns to, and an outline a few points across does not hide a hairline crossing it. The reverse order would hide the guide behind the very object being aligned to it |
| **move ghost**, **resize ghost** | over the real outline, and both stay visible: the pair is what states the change |
| **markup band**, **freehand trail**, **vertex run**, **measure preview** | last, over everything: while a gesture is in flight the shape IS the cursor, and anything drawn over it obscures the one thing being aimed |

**Re-ordering any two of these is a behaviour change.**

## Item notes

### `fn draw_chunks`

[`crate::canvas::chunks`] decides *whether* and *which*; this decides
nothing except how the answer is reported. The split is deliberate: the
decision is testable without a painter, and the painter cannot grow a second
opinion about what a chunk is.

# Why every path writes a line, and why they share one slot

The same argument [`draw_anchors`] makes: a driven check that finds no boxes
has to be able to tell *the operator turned them off* from *the click
missed* from *the program is broken*, and silence says all three at once.

Through [`crate::diag::trace_changed`] rather than `trace`, because this
runs once per page per frame and an unchanging state re-reported at sixty
hertz buries the transitions that are the news. **Both the drawn line and
the declined line take the one slot**, so an alternation between them is
never collapsed — only a repeat of the identical line is.

The first token is `canvas-chunks-declined`, not `canvas-chunks`, for the
reason `draw_anchors` states at length: `tools/ui-verify` keys on first
tokens, and a reader asking for the drawn line must not be handed a decline
carrying none of the fields it reads.

### `fn draw_anchors`

# Why this is a function here rather than three lines at the call site

Because it is the **only** place in the paint pass that needs the object
model, and reaching for it costs a `Ref` into the document's decomposition
cache. Keeping that borrow inside one short function is what guarantees it is
released before the rest of the frame — the same discipline
`app::cache::page_objects`' own docs set out, and the reason
`canvas::interact` has a comment about dropping its `Ref` explicitly.

It draws nothing at the Object rung. An object's anchors are not the
operator's subject there — the object is — and painting thousands of hollow
squares over a selection they are about to *move as a whole* would be noise
with a rendering cost.
