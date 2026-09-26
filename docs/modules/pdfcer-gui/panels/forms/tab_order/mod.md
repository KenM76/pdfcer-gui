# `panels::forms::tab_order` — the order this form is tabbed through, per
page, and the drag that changes it

A **second list beside the fill list**, answering a different question. The
fill list ([`super::field_list`]) is in `/AcroForm` `/Fields` order because
that matches the printed form, and it is not changed by anything here. This
one is in each page's `/Annots` order, which is the order a viewer paints the
widgets in and — absent a `/Tabs` entry — the order it tabs through them in.

[`model`] does the walking, the matching and the `/Tabs` reading, and carries
the whole argument: §1 why the order is `/Annots` order, §2 why it walks
`page_annotations` rather than `widget_rects`, §3 why `/P` is never
consulted, §4 the primary-source reading of `/Tabs` (including the finding
that it is **not** inheritable), §5 what is counted rather than listed. This
file is the drawing, the disclosures and the one action the view can raise.

## The affordance exists because the verb exists, and not before

The drag is offered here only because `EditSession::reorder_annotations` can
commit it. R9 is the rule, and it is the one to apply to the next gap in
this view: an affordance that cannot commit teaches the operator a gesture,
lets them perform it, and then either does nothing or lies. So for any
reorder this engine has no verb for there is no handle, no `Sense::drag`, no
up/down pair, not even a disabled one. The test is applied per capability,
never once for the section as a whole.

## What the drag is, and what it deliberately is not

[`drag`] holds the gesture and carries the argument in full. In outline:

- **A row is a drag source; the marker is an insertion caret**, the same one
  `crate::panels::pages` draws for the page rail, because the operator named
  that panel as the reference — *"like we can with pages in the page
  preview"*.
- **Widgets move among widget slots and nothing else moves at all.** The
  list is form fields; the array also holds every `/Link` and markup
  annotation on the page, and `/Annots` order is *paint* order, so moving
  one would silently change what is drawn on top of what.
- **It does not write `/Tabs`.** Sourced, and counter-intuitive enough to
  restate: `/Tabs /A` is PDF 2.0 only, and PDF/UA-1 §7.18.3 *requires*
  `/Tabs /S`, so writing `/A` as a side effect of a drag would make a
  conforming file non-conforming with nobody asking. Acrobat's own manual
  tab order is an `/Annots` permutation with no `/Tabs` written.

Which means the per-page `/Tabs` sentence this view has always shown is
now doing a second job. On a page whose `/Tabs` says `/S` or `/R` or `/C`,
a drag changes the array and **a conforming reader may still tab in the
order the file states**. The sentence is what stops that reading as a bug in
the drag. It stays above the rows, unconditional, on every page.

## Actions, not mutations — and it raises exactly one

[`Action::GoToPage`], from a page heading's **Go to** control, exactly as
`crate::panels::bookmarks` and `crate::panels::comments` do. That is
navigation, not authoring: it moves the view, never the document. The body is
handed `&OpenDoc` — a **shared** reference, so this is a compile-time fact
and not a convention.

## Rule 4: this is disclosure, and it draws nothing on the page

Not one pixel on the canvas. No numbered badge over each widget, no
highlight on the row under the pointer, no arrows between fields. The
one-line test is *would a screenshot of the editing canvas differ from a
screenshot of the same document saved and reopened?* — and the answer here
must stay "no".

It is worth naming what rule 4 would *permit*, so nobody later reads its
absence as a prohibition: a hover highlight on the widget belonging to the
row under the pointer is the fourth clause's *"a snap indicator, a hover
highlight, a rubber-band, a selection handle — these are the cursor"*, and it
would be a genuinely good affordance here. It is not built for the same
reason `super`'s header gives for the fill rows: the panel→canvas channel
for *which row is hovered* does not exist in this build, and `crate::canvas`
is not this module's to extend.

## Where it sits, and why it is a section rather than a panel

Inside the existing Forms panel, between the whole-form controls and the fill
list, as a collapsing section that is **closed by default**. Three reasons,
in order of weight:

1. **It needs nothing from the shell owner.** A new panel needs a `Panel`
   variant, a command id, a registry entry, a manifest reference, a RON
   regeneration and a mode-arrangement change — six files this work may not
   touch, any one of which missing produces the unreachable panel
   `crate::panels`' header is about.
2. **It is about the same subject as the panel it is in.** An operator
   asking "what order does this form tab in?" is already looking at the form.
3. **Closed by default** because it answers an occasional question, and
   because the panel's primary job is filling.

## The two layout rules, and which one applies

1. **Scrollbars must be visible.** `crate::panels::scroll_style` is applied
   by `crate::panels::Panel::show` before any body runs, and this section
   inherits it through the `Ui` it is handed.
2. **A fixed-size child inside a scroll area needs the container's width
   stated.** **There is no fixed-size child here**, so
   `crate::panels::content_width` is deliberately not called — stated rather
   than left to look like an omission, because skipping it silently is
   exactly how the Objects panel shipped clipped rows. Every child is a
   `Label`, which wraps to whatever width it is given; the only fixed-width
   child is the **Go to** button, at a couple of dozen points against a dock
   that opens at 320. `crate::panels::comments` records the same reasoning
   for the same shape.

### The one layout thing this section does that no panel body does

Its list is inside a scroll area with a **stated maximum height**
([`MAX_LIST_HEIGHT`]). Every other list in this crate is the last thing in
its panel and takes whatever vertical space is left. This one is *not* last —
the fill list is below it — and the Forms panel's top level does not scroll,
so an unbounded list here would push the fill list off the bottom of a
non-scrolling container, where nothing would indicate it had gone. That is
the same class of defect as the invisible scrollbar: content clipped at a
container edge with nothing to say so.

## `PDFCER_DIAG` proves what this computed

One `forms-tab-order` summary line plus one `forms-tab-page` line per page —
carrying the page's `/Tabs` state, where it was found, and every count — and
then one `forms-tab-row` line per row, capped. Written whenever the **panel**
draws, not only when the section is open, so the order is provable from a
trace without anyone having to click anything.

That is R1 — correctness is established by driving the binary, not by a
passing test — applied to a surface whose correctness is entirely sequence
and arithmetic: a screenshot of this list cannot tell you that the
numbering skipped a widget the file lists, that a `/Tabs` came from an
ancestor rather than the page, or that four annotations on the page are in
the tab sequence and not in the list. Every one of those is in the trace.
