# `panels::properties::text` — how the selected text LOOKS, and changing it

`RIBBON_IA.md` §5.8's Font controls — `format.font`, `format.font_size` —
built where that section says to build them:

> Build order: **panel first, tab second.** The panel is the harder half
> and the tab's contents are a subset of it, so building the tab first
> would mean writing the property editors twice.

## The operator's ask, twice

> *"We should also have all the font tools available that Word does."*
> — O37, 2026-08-25

> *"…when I have an object selected like text the Tool tab doesn't switch
> to giving me the editable stuff for that object."* — O46, 2026-08-26

## ★★★ The operand is the TEXT SELECTION, not the object selection

This is the decision most likely to be read as a shortcut, so it is argued
rather than asserted.

`EditSession::format_text` locates its operand by a **pinned byte span into
a decoded content buffer**, obtained from `GlyphProvenance` — which is
keyed on a *run* of the page's text extraction. The canvas object selection
is a `TargetId`: a **paint-order index** into `PageObjects`. The two index
spaces are unrelated, and nothing in either crate maps between them.

So an object-selection operand would have to be inferred — by bounding box
overlap, most plausibly — and an inference that picks the wrong run
restyles text the operator did not select, silently, in a file they then
send to somebody. The text sweep *is* a run range, exactly, by construction.

★ That is a real gap and it is named rather than hidden: clicking a text
object with the Select tool does not raise this section; sweeping across the
text does.


`OPERATOR_REQUESTS.md` **O89**: *"I don't see where I am able to edit the
color of text, vectors, etc."* He had clicked the text and found nothing but
a sentence telling him to sweep.

[`super::textobject`] now draws a **working colour control** for a clicked
text object, and it does not weaken one word of the argument above: its
operand is not inferred from geometry, it is the object's own `BT`…`ET`
**byte span** joined against each run's show-operator span — an exact
containment, in one buffer, of the same kind the pinned edit path already
stakes every restyle on. `crate::canvas::textedit::pin::object_text` is the
join and carries the argument.

⇒ Two consequences for a reader of this file. **(1)** The sentence formerly
ending this paragraph — *"the empty state says so in those words"* — has
moved with the state it describes; `route` and `ROUTE_REGION` are gone from
here, and the block where they stood says where they went and why the region
kept its old spelling. **(2)** This section is now exactly what its heading
claims: the editor for a **swept range**. With nothing swept it returns
`false` and says nothing, because something else is speaking.

★ Four of the five controls here are still sweep-only, and that is a
decision rather than a leftover: face, size, bold and italic each need a
reading of **one run** to be honest, and a whole object has no single answer
to any of them. Colour is the one property for which *"they disagree"* is
itself a displayable answer.

## ★★ Why the read-back is stamped and not re-read every frame

The values shown — face, size, colour — come from `GlyphProvenance`, and
provenance is **off** in the shared page-text cache. Reading it means an
extraction with `capture_provenance` on, which is the expensive thing this
shell does: **392 ms on the operator's benchmark sheet.**

A panel that did that per frame would take the application to under three
frames a second on exactly the drawings this program is for. So
[`TextStyleDraft`] carries a stamp — `(page, first run, edit epoch)` — and
re-reads only when it moves, which is the same shape as
[`super::geometry::GeometryDraft`] and for a much larger reason.

## ★★★ Bold and Italic are NEVER greyed, and that is the engine's ruling

`set_font` selects a real face and refuses when the page carries none.
`gate_synthesis` refuses synthesis when a real face **is** available.

★★★ **They are NOT exact complements, and this paragraph used to say they
were.** It read *"so between them every page is covered and there is no
page on which bold is unreachable"*, quoting the engine — who withdrew the
claim in writing on 2026-08-27 after reproducing the counter-example.
`gate_synthesis` prefers a real face by *family*, and the face it prefers
may not map every glyph in the run, in which case `set_font` refuses it and
synthesis is already gated off. On `textedit/format_family.pdf` bold is
reachable by neither verb. Filed, confirmed, and queued first by the
engine; `crate::app::actions::textstyle`'s header carries the whole of it.

★★ The conclusion survives its premise, which is why the two buttons still
do not grey. Greying them would mean predicting a refusal that depends on a
per-run glyph-coverage test this shell cannot run without doing the
engine's work. The honest behaviour is to try and to show the engine's own
named refusal, which is what happens.

pdfcer-core's instruction, verbatim: *"Do not grey out a bold button. Offer
it, and surface the disclosure when synthesis fires."*
`crate::app::actions::textstyle` takes whichever verb the page allows and
discloses which one it took.


The paragraph two up says greying would mean *"predicting a refusal that
depends on a per-run glyph-coverage test this shell cannot run without doing
the engine's work."* Both amendments are kept, in order, because the first
one's reasoning is what earned the second.


`EditSession::preview_style_resolution` said whether a real face resolves
and handed back the string to pass to `set_font`; `preview_font_resources`
said which resources `set_font` would accept for this run's characters, each
with the same kind of string. Comparing the two was a string equality
between two engine-issued selectors — not the coverage test re-implemented,
not the family heuristic re-derived. `StyleOutlook::FaceCannotCover` was
that prediction, and the buttons still did not grey, because the engine had
a queued fix that would turn the case into ordinary synthesis.


It was not a narrower `gate_synthesis`. `Pass 179.0`'s automatic **style
ladder** replaced the decision the gate was making, and under it *"a real
face claims the style and cannot show the run"* is **no longer an outcome
at all** — it is an entry in `StyleLadder::passed_over` on the way to a rung
that works. So `FaceCannotCover` is deleted rather than retargeted; a
sentence kept alive past its subject is how a shell ends up warning about a
limit that no longer exists.

⇒ ★★ And the instrument changed with it. The 2026-08-29 join was previewing
the **R90 gate**: one bit, *"is there a real face on this page that claims
this style"*. The gate is one input to the ladder's decision, not the
decision, and it **cannot see rung 2 by construction** — the standard-14
sibling of the run's own family is not on the page, which is the whole point
of it. On the commonest CAD page there is, a title block set in `Helvetica`
with no bold resource, the old hover promised thickened letters about a
press that binds `Helvetica-Bold` and produces genuinely bold type.

`EditSession::preview_style_ladder` (`Pass 295.0`, consumed here the day it
shipped) runs `plan_style_ladder` — **the function `format_text` runs** —
read-only against the staged content, walks the page once and stages
nothing. The preview and the commit are two readings of one answer rather
than two answers kept in step by hand, and the join, its load-bearing
ordering constraint in [`TextStyleDraft::sync`], and `FaceCannotCover` all
die together because they were one workaround.

⇒ ★★ **The buttons still do not grey**, and the reason has moved once more
— from *we cannot know*, to *knowing is not a reason to withhold*, to *the
only refusal left is the operator's own setting*:

1. The engine's instruction is unconditional and unwithdrawn.
2. The one predictable refusal is now [`StyleOutlook::Declined`] — the
   ladder reached rung 4 and `StylePolicy::Refuse` is set. That is a setting
   working, not a defect, and what it wants is a sentence naming the setting
   so the operator can change it in one move.
3. R9 reserves greying for the *temporarily* unavailable **and requires it to
   explain itself on hover**. The hover is where the explanation already is,
   and it now carries the whole answer — so greying would add a disabled
   control and no information.

What changed instead is which sentence the hover carries, and that is exactly
R83's size of change: the operator learns before the gesture rather than from
a refusal after it. [`bold_hint`] carries the seven-row table.

★ This is also why the two toggles do **not** show the run's current state.
There is no "is this run bold" bit in a PDF: weight is a property of the
*face* (`Helvetica-Bold` is a different font from `Helvetica`), and a
synthetic weight is a stroke width in the content stream. A toggle drawn
pressed-in would be claiming to have read a fact that is not recorded. They
are **buttons that apply**, not switches that reflect — and the face name
beside them is where an operator reads what the text actually is.


`pdfcer-core` v0.15.0 (`Pass 162.0`) closed the last of the four things the
operator named as not fully editable:

> **FONTS** — text can be restyled to a face the document **DOES NOT
> CONTAIN**, for the fourteen faces every PDF reader is required to have.
> pdfcer authors the font resource on demand, with widths, embedding nothing.
> A face outside those fourteen still refuses by name — that needs a real
> font program.

This section offered only what `preview_font_resources` returned, which by
construction enumerates *the page's own `/Font` resources* — so the new
capability was unreachable from any surface in the program.

⇒ The list now has **two groups** and they are two different acts:
rewriting one `Tf` operand, and rewriting it *plus writing a new `/Font`
object into the operator's file*. [`super::face`] owns the list, the
headings, and the disclosure the second group owes; [`face_row`] owns the
row it sits on. Both surfaces — this panel and the ribbon's Format ▸ Font
group — draw the identical body, because *"a face offered in one and not the
other"* is the divergence this project keeps finding.

★★ **The shell writes nothing.** `FormatPlan::created_font` puts the
resource write on the caller, and `EditSession::format_text` **is** that
caller: it folds the write into the same undo command on both its page and
form paths. [`super::face`]'s header carries the evidence. Nothing here
allocates an object or knows the shape of a font dictionary.

★ The refusal for a fifteenth face is a **sentence**, not a silence —
`crate::text::status::selection::TextStyleRefusal::FaceNotOnPage`, whose old
wording (*"pdfcer can only switch text to a font this page already carries"*)
stated a limit this engine no longer has and was corrected in the same
change.

## Rule 4: nothing here marks the canvas

Every disclosure this section causes — a synthetic weight, a colour space
narrowed, a real face used instead — lands in the status bar through
`crate::app::actions::disclosure`. **The restyled text renders exactly as
the saved file will render it.** No badge, no tint, no "provisional"
styling: the one-line test is whether a screenshot of the canvas would
differ from a screenshot of the same document saved and reopened, and
nothing in this module can make it differ.
