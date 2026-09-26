# `ui-verify/checks/master_detail`

`the_inspector_is_one_master_detail_column` — Objects over Properties, in
the room the Tool panel used to take, with rows that fit.

# What this is for — `OPERATOR_REQUESTS.md` **O123**, parts 3, 4 and 6

> *"Objects and Properties become master–detail in one panel with a
> draggable split, ellipsis and tooltip on rows … I'd also like those one to
> appear in the space where the tool dock currently shown … Default dock
> width 360 px in Edit."*

Three claims, and each of them is a layout claim, which on this project has
exactly one oracle: **a rendered screenshot.** A unit test can pin the
arrangement `app::modes::defaults` intends; nothing but a driven run can say
that the dock drew it.

# The five things it asserts, and why none of them is redundant

| # | assertion | the build it fails on |
|---|---|---|
| 1 | Objects' body and Properties' body are **both** on screen | one behind the other in a tabbed stack, which is what "one panel" would become if somebody merged the two stacks into one |
| 2 | Objects sits **above** Properties, in the same x range | a side-by-side split, or the two in different columns |
| 2b | the detail pane holds **no document metadata**, *and* the document's own properties are a mounted tab | a build that draws the file's `/Info` form at the foot of this pane — and, through the second half, every build that "fixes" it by drawing nothing |
| 3 | a **splitter** publishes between them | a fixed split — the thing part 3 says the pair must not be |
| 4 | **no row is elided at the width the dock opens at** on this fixture | a row form carrying the full object description, on which every row is elided |

## Why 2b is here rather than in a check of its own

Because it is a claim about **this pane**, and this is the check that
already has the pane's rectangle in hand. Document metadata shown in the
properties tab is precisely something which is not the detail of the
selection drawn inside the detail half of this master–detail column, so the
assertion belongs with the one that establishes the column exists.

⚠ **And it is a PAIR, deliberately.** The obvious assertion — *the metadata
region is not inside the Properties body* — passes on a build where the
Properties panel draws nothing at all, where the docprops panel failed to
mount, and where the dock failed to draw. So it is read together with the
presence of `dock.tab.file.document_properties`, which says the metadata
went **somewhere** rather than merely leaving. That shape — a negative
paired with the positive control that stops it being vacuous — is the one
this suite keeps having to relearn.

# Why assertion 4 is about the ROW and not about the dock's width

The application's own `objects-rows` line on this fixture reads
`pane=314.0 overflow=473.6` — the widest row wants **473.6 pt** where the
pane offers **296 pt** of text room, and measured headlessly the
*narrowest* row of this fixture wants **306.3 pt**. Every row is over, so
no width fixes it: a dock wide enough is about 526 pt, half of an 1,100 pt
window.

The pane is also not necessarily the default one. A trace reading
`mode-changed from=Some("read") to=edit remembered=true` is a **restored
workspace**, which never consults `EDIT_INSPECTOR_WIDTH` at all — so a
failure message naming that constant names something the failing run never
read.

⇒ The row is where the answer lives: `panels::objects` draws a headline
(index, kind, the facts that identify the object, one disclosure mark) and
hovers the full description. See that module's header.

The lesson for this file, and it is the suite's own recurring one: **a
failure message that lists candidate causes is a hypothesis, and it goes
stale exactly like a comment.** The message below names the measurement
(`overflow=` against `pane=`) instead of guessing, because the trace already
carries the number that decides between the candidates. The headless half of
the same question is
`panels::objects::tests::every_object_row_of_the_a1_sheet_fits_the_measured_pane`.

Assertion 4 is the one that needed a channel built for it, and the channel
is the application's own `objects-rows` line. That is the app marking its own
homework, and it is published anyway because the harness cannot read the text
a panel renders — there is no AccessKit reader, no OCR, no text extraction
from a screenshot.

## So it is made non-circular the way `read_mode_chrome` is: a pixel

The **right-hand strip of the Objects pane** is sampled and must be
near-uniform — the panel's ground, with no glyph running into it. *"The rect
is exact and cheap and would be satisfied by a build that moved the canvas
without repainting anything; the pixels cannot be faked by an arithmetic
error."* An elision arithmetic that lied in the trace would still leave ink
against the pane's right edge, and this is the assertion that sees it.

⚠ Note the polarity, because it is the opposite of every other uniformity
assertion in this suite: here `is_uniform` is the **pass**. A strip of pane
that is *not* uniform is a row running off the edge.

# The fixture is pinned, and it is pinned for a stated reason

`fixtures/a1-titleblock.pdf` — an A1 CAD sheet whose text objects carry
subset-tagged font names (`AAAAAA+SpaceGrotesk-Bold`). Those are the rows
that get cut mid-character at 320 pt, and they are why the fixture has to be
one whose text objects carry subset-tagged names. On a document of short
rows this check could not fail.
