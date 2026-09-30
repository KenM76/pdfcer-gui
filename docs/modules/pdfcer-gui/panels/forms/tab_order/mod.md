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

## Item notes

### `const MAX_LIST_HEIGHT`

See this module's header, "The one layout thing this section does". The
number is a judgement rather than a measurement: tall enough that a page of
eight or nine widgets is read without scrolling, short enough that the fill
list below stays on screen in a dock pane opened at its default height.

### `const REGION_HEADER`

`pdfcer_core::forms::MAX_FORM_FIELDS` is 500,000, so an uncapped per-row
census on a pathological form would bury every other line in the capture —
the same failure `crate::canvas::forms` caps its `form-box` census for. The
summary lines are never capped, so the *counts* stay provable even when the
enumeration stops.
The region the collapsing header publishes, so a driven check can open it.

### `fn page_block`

A page with no widget on it is still drawn. Its `/Tabs` state is a fact about
the document, and a gap in the page numbering would read as a bug in the
view rather than as an empty page.

### `fn tabs_note`

# Which states warn, and why exactly those

A warning here means *"what you are looking at is not the answer to the
question you are asking"*. That is true under `/R`, `/C` and `/S`, where the
order is **derived** — from geometry, or from the tag tree — rather than
stored, so the `/Annots` sequence on screen is a different sequence. It is
also true, more weakly, under a `/Tabs` name this build cannot interpret.

It is **not** true for an absent `/Tabs` (the `/Annots` order is what viewers
use), nor under `/A` or `/W` (the standard defines those *as* the `/Annots`
order), nor for an ancestor's `/Tabs` — which is disclosed as a fact about
the file but is not applied, because `/Tabs` is not an inheritable page
attribute. See [`model`]'s §4.

The `bool` rather than an enum is deliberate at this size: there are two
visual treatments in this crate for a line of disclosure (warn colour, or
small-and-weak), and a two-valued answer is the honest shape for a two-valued
question. It becomes an enum the day a third treatment exists.

### `const REGION_CHOOSER_PREFIX`

Page `n`'s tab-order combo publishes `forms.tab_order.tabs.<n>` (clip-aware)
and each popup option `….<absent|R|C|S|A|W>`. `ui-verify`'s
`page_tabs_chooser` drives them.

### `fn tabs_chooser`

The "Tab order:" combo above a page's rows. It shows the page's own `/Tabs`;
an inherited value shows as Not stated, because choosing writes the page's own
key and the note below already discloses the inheritance. A change raises
`FieldAction::SetPageTabs`; the engine owns every refusal (`/A` and `/W` below
PDF 2.0, PDF/UA's `/S`, a certified document), and its sentence reaches the
status line through the edit funnel.

### `fn choice_name`, `fn tabs_name`

The operator's name for a value, and the key it writes (`absent` for none):
the second names regions and trace fields. `PageTabs` is `#[non_exhaustive]`,
so an unknown future variant names itself `?`.

### `fn every_tabs_choice_is_named_and_addressable_apart`

Two choices sharing a name or a region suffix would be indistinguishable to
the operator or to the harness.

### `fn mode_name`

Used only where the *name* is quoted back — the ancestor sentence and the
trace. For an unrecognised value this is the raw bytes as decoded, never a
substitute: a name pdfcer has never seen is a document fact, and printing
something else in its place would make the view claim the file said
something it did not.

### `fn trace`

# Why this is more than a debug print

R1: correctness is established by driving the binary, not by a passing
test — and a picture of the result is not evidence when two very different
results look alike, so what the running application chose has to be printed.
This view has that property in a sharper form than most: a
screenshot of a list of names in an order cannot tell you that the order is
the one the file lists, that a widget the file lists was skipped, or that a
`/Tabs` came from two levels up the page tree. Every one of those is text.

Every number here drives something on screen, which is the test for what
belongs: `tabs` decides the per-page sentence and its colour, `rows`,
`unclaimed`, `anonymous` and `other_annots` each decide a line, and
`no_widget_fields` decides the document-wide note. If a number here is wrong,
something on screen is wrong with it.

### `fn the_derived_orders_warn_and_the_stored_ones_do_not`

The single most consequential mapping in this view, and it is wrong in
two opposite and equally bad ways. Warning on `/A` or `/W` would tell an
operator the list is unreliable on the one kind of page where the file
explicitly asks for exactly this order. *Not* warning on `/R`, `/C` or
`/S` would let them read a sequence that is not the tab order and
believe it is — which is the failure this whole view is designed around.

### `fn an_absent_tabs_entry_is_named_absent_and_nothing_else`

The constraint this view is built around, asserted as a property of the
string an operator actually reads rather than of the enum behind it.
`D:\Dev\pdfcer`'s roadmap records what Acrobat's "Unspecified" tab-order
state mechanically denotes as **unsourced after two attempts**, so any
of these words on this page would be an assertion nobody can support.

It also must not warn: with no `/Tabs`, the `/Annots` order is what
viewers use, so the list *is* the answer and a warning would be false.

### `fn an_ancestor_tabs_is_disclosed_without_being_applied`

Three assertions because the sentence has three jobs, and dropping any
one of them produces a different wrong answer. It must say the page has
none of its own (or it asserts an inheritance ISO 32000-2 Table 31
denies); it must name the ancestor's value (or it hides a fact that
changes what another viewer does); and it must not warn (because this
build does not apply it, so the sequence on screen is still the
`/Annots` order that an absent `/Tabs` implies).

### `fn every_tabs_state_has_its_own_sentence`

Six states, six sentences. Two that read alike would send an operator
looking for the wrong cause — and the pair most likely to be collapsed
by someone tidying up is `/R` and `/C`, which differ by one word and
describe genuinely different sequences.

### `fn a_warning_glyph_is_never_load_bearing`

`RIBBON_IA.md` R84 — never a colour-class cue alone. `⚠` is exactly
that, and it is doubly load-bearing here because these sentences are
also drawn in the warn colour: a reader who sees neither the glyph nor
the colour must still get the whole meaning from the words.

### `fn the_page_index_travels_zero_based_and_prints_one_based`

The off-by-one that would otherwise be invisible.
[`Action::GoToPage`] takes a 0-based index — the convention
`crate::panels::bookmarks` and `crate::panels::comments` both pin from
their own side — and every string a human reads takes the number one
higher. Getting it backwards produces a view that navigates one page
past every heading, which looks like a document defect.

### `fn the_explainer_teaches_the_drag_and_no_longer_claims_to_be_read_only`

**A drag with no visible handle is undiscoverable.** There is no button,
no grip dots, no "Move up" — the only thing that can tell an operator
this list is draggable is the sentence above it. So the sentence is load
bearing, and two things about it are pinned:

1. It says how (`drag`) AND what will be shown (`line`). The operator
   asked for *"clear markers of where the field is going to move to"*; a
   caret nobody expects is not a clear marker.
2. It does not claim the view leaves the order alone. Copy calling a
   view read-only while the view reorders is the class of stale string
   that survives longest, because nothing about it looks wrong — so the
   wordings are asserted against by name.

### `fn no_row_carries_a_labelled_reorder_button`

The gesture is the feature — the operator asked for drag and drop by
name — and a "Move up / Move down" pair beside every row would double
the height of a list that already runs to 200 rows on a real form. The
absence is worth pinning because the obvious response to "the drag is
hard to discover" is to add buttons, and the right response is to fix
the sentence that teaches it.

This is NOT a permanent prohibition and should not be read as one.
A keyboard route to reordering is an accessibility gap this view has,
and if it is filled the right way — a keymap command, not a pair of
buttons per row — this test is what should be revisited, with its
reasoning, rather than deleted quietly.

### `fn section`

Called from [`super::body`] with the `/AcroForm` it has already parsed and
the `DocumentView` it already holds — neither is re-derived here, because two
parses of one form per frame is a cost with no benefit and because a second
`parse_acroform` could in principle disagree with the first one the panel is
drawing from.

`actions` is pushed at most once, with [`Action::GoToPage`].
