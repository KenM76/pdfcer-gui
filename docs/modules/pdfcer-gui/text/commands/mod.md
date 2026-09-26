# text::commands — the label and tooltip of every ribbon command

One function per command, each returning a [`CommandText`]. The ribbon's
*structural* strings — tab labels, tab questions, group captions, mode
labels — live next door in [`crate::text::ribbon`].

## Why a pair rather than two functions

Every command needs both a label and a tooltip, and they are written
together: the tooltip's job is to say what the label cannot fit, so
reviewing one without the other is reviewing half a sentence. Two
functions per command would also double a file that is already the
longest in the catalog, for no gain — nothing ever wants one without
being able to reach the other.

## Every command has a tooltip. That is a rule, not an accident.

`RIBBON_IA.md` P3 reserves greying for *temporarily* unavailable — no
document open, undo stack empty — and requires that it *"is always
explained on hover."* A command with no tooltip cannot honour that, so
[`CommandText`] has no way to express "no tooltip" and a test below
asserts none is empty.

The salvage source got this wrong in exactly one place and it is
instructive: the four Measure buttons (`Linear Dimension`,
`Radius / Diameter Dimension`, `Set Group Scale…`, `Manage Dimension
Groups…`) were rendered as text-only selectables **with no tooltip at
all** — the four controls on the tab most likely to be used by someone
who has never used a PDF measuring tool.

## Voice, carried across from the salvage source deliberately

pdfcer's tooltips are unusually long and unusually specific, and that is
a deliberate quality of the product rather than an accident of who
wrote them. They say what a command *changes* ("This changes the
document, not just the view"), what it *cannot* do, and what is
*irreversible*. Where the salvage source's wording said something worth
keeping, it is kept close to verbatim.

**DO NOT QUOTE A LIVE TOOLTIP HERE AS AN EXEMPLAR.** A header that
holds one up as a model makes a SECOND COPY of that string's claim, in a
file nobody opens when the claim expires. Two such exemplars stood in this
paragraph and both were false before anyone noticed — one denying
verification the engine had grown, one calling an apply irreversible after
it began staging into the next save.

⇒ Name the **shape** of the good sentence, not its text. The examples
above survive because each describes a permanent property of a tooltip
rather than a measurement of the build.

Two things are trimmed:

1. **Tooltips that enumerate the alternatives.** The old `Add Text`
   tooltip explained itself by contrast with three other commands over
   four sentences. One contrast is a clarification; three is a menu.
2. **Tooltips that describe a defect.** `"click-to-place editing on the
   canvas is coming"` is a roadmap entry, not a tooltip.

## Labels: three renames that `RIBBON_IA.md` §5.4 requires

`Aa`, `I⁺ Aa` and `Obj` become **Edit text**, **Add text** and **Edit
objects**. They are the primary content-editing tools and were the
least legible controls in the application — and the first two returned
the *same literal*, `"Aa"`, distinguished only by icon and tooltip.

## Item notes

### `mod view`

Glob-re-exported, so a caller sees no seam at all: every call site still
writes `crate::text::commands::view_zoom_in()`. See that module's header
for why the cut is there and nowhere else.
