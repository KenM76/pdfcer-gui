# `panels::forms::tab_order::model` — turning a document into a per-page
widget sequence, and reading what the file says about tab order

The whole of the Tab order view that is not drawing. [`collect`] walks every
page's `/Annots`, keeps the `/Widget` entries in the order the array lists
them, matches each one back to the field it belongs to, counts everything it
could **not** list, and reads each page's `/Tabs` entry. Nothing here touches
`egui`.

`crate::panels::comments::model` is the shape this follows and for the same
reason: every interesting decision in this view is a **classification**, and
a classification is only testable if it is separable from the widget that
shows it. The list of things that have to be right —

- which annotations are widgets and which are not,
- which widget belongs to which field, and which of that field's widgets it
  is,
- what the file's `/Tabs` entry says, whether it is on the page itself or on
  an ancestor, and what that means for whether this list *is* the tab order,
- and everything that cannot appear in the list at all, counted rather than
  dropped

— is exactly the list `tests` below sweeps, against real engine fixtures for
the ordinary shapes and against a hand-built graph for the shapes no fixture
in the corpus carries.

---

## 1. The order is `/Annots` array order, which is also PAINT order

Not `/AcroForm` `/Fields` order. The two commonly differ, and they answer
different questions: `/Fields` order is the document's declaration order,
which is what the existing fill list uses because it matches the printed
form; `/Annots` order is the order the annotations are listed **on one
page**, which is the order they are painted in and — absent a `/Tabs` entry
— the order a viewer tabs through them in.

`pdfcer_core::edit::EditSession::widget_rects`' own doc comment states both
halves: *"`/Annots` array order — which is **paint order**, and (absent
`/Tabs`) also the tab order. It is not `/AcroForm` `/Fields` order, and the
two commonly differ."*

The paint-order half is worth saying out loud in the view, because it is the
fact that would make a future *reorder* consequential: moving a widget
earlier in `/Annots` does not only move it earlier in the tab sequence, it
also moves it **underneath** anything that now follows it.

## 2. Why this walks `page_annotations` rather than `widget_rects`

`EditSession::widget_rects(page)` is the verb this shell asked for and got
(engine `e8e9881`), it returns exactly this order, and the canvas hit test
uses it. It is **not** the right source for a *list*, and the difference is
not stylistic.

Read against the engine, `EditSession::widget_rects` is
`annot::page_annotations` — the same walk this module uses — followed by
`filter(subtype == b"Widget")` and then a `filter_map` that drops any
annotation whose `/Rect` is absent **or** whose object identity is absent.
Both drops are right for a hit test: you cannot click a rectangle that does
not exist, and you cannot fill a widget you cannot name. Both are wrong for
a list, where a widget the file lists is a widget in the tab sequence
whatever else is missing from it — and dropping it silently would make this
view under-report the very thing it exists to report.

So this module calls `page_annotations` directly, which is what
`widget_rects` calls, and keeps what `widget_rects` filters out **as counts**
([`PageTabs::anonymous`]). The order is byte-identical because it is the same
array read by the same function.

## 3. `/P` is never consulted — which page a widget is on is answered by
`/Annots`

The correction `crate::canvas::forms::boxes::place` records, applied here
from the start. `/P` is *Optional* (§12.5.2 Table 164), it is frequently
absent in the wild, and `pdfcer-core` reads it without resolving through the
graph — so a direct rather than indirect `/P` also reads as absent. An
implementation that asked each widget which page it claims returns **nothing
at all** for a large class of real forms: no error, no refusal, no trace.

No test written against the fixture corpus can catch that, because all
eleven form fixtures in `D:\Dev\pdfcer\fixtures\synthetic\forms\` write `/P`
on every widget. The direction is therefore inverted here as it is there:
each **page** is asked which widgets it lists. `pdfcer_core::forms::Widget`'s
`page` field is not read anywhere in this module, and
[`tests::a_widget_with_no_p_entry_is_still_listed`] builds a form that omits
it so that every other assertion doubles as proof the key is unread.

## 4. `/Tabs` — what this reads, and the one place it departs from the
engine

### What the standard actually says

ISO 32000-2:2020 Table 31 (entries in a page object), verbatim from
`D:\Dev\Rag-Specialized\PDF_Spec\_sources\ISO_32000-2_sponsored_EC3.pdf`
p. 107:

> **`Tabs`** *name* — (Optional; PDF 1.5) A name specifying the tab order
> that shall be used for annotations on the page (see 12.5 "Annotations").
> If present, the values shall be one of `R` (row order), `C` (column
> order), and `S` (structure order). **Beginning with PDF 2.0, additional
> values also include `A` (annotations array order) and `W` (widget order).**
> Annotations array order refers to the order of the annotation enumerated
> in the `Annots` entry of the Page dictionary. Widget order means using the
> same array ordering but making two passes, the first only picking the
> widget annotations and the second picking all other annotations.

and §12.5.1, same document, p. 466, which is where `R`, `C` and `S` are
defined operationally: `R` and `C` are visited *"in rows running
horizontally across the page"* / *"in columns running vertically"*, ordered
by the viewer preferences' `/Direction`; `S` is *"the order in which they
appear in the structure tree"*, and *"the order for annotations that are not
included in the structure tree is determined in a manner of the interactive
PDF processor's choosing."*

That gives [`TabsMode`] its five named values and [`TabsMode::sequence`] its
whole content:

| `/Tabs` | Where the order comes from | Is this list the tab order? |
|---|---|---|
| `/A` | the `/Annots` array | **yes** — that is this list |
| `/W` | the `/Annots` array, widgets first | **yes**, for widgets |
| `/R` | where the fields sit on the page | no — derived, not stored |
| `/C` | where the fields sit on the page | no — derived, not stored |
| `/S` | the document's structure (tag) tree | no — derived, not stored |
| anything else | unknown | unknown |

### `/Tabs` is NOT an inheritable page attribute

⚠ It reads as one, it has been written down as one more than once, and the
primary source denies it twice:

1. ISO 32000-2 Table 31 marks `Rotate` *"(Optional; inheritable)"* and marks
   `Tabs` *"(Optional; PDF 1.5)"* — no inheritability marker. Verified by
   reading the two rows out of the sponsored copy in the spec RAG, p. 105
   and p. 107.
2. The table's own preamble (§7.7.3.3): *"Attributes that are **not**
   explicitly identified in the table as inheritable **shall not** be
   inherited."* `D:\Dev\Rag-Specialized\PDF_Spec\iso32000\iso32000__s__7.7.3.md`
   records the exhaustive search of that table and finds **exactly four**
   inheritable attributes: `Resources`, `MediaBox`, `CropBox`, `Rotate`.

So a `/Tabs` on an ancestor `Pages` node does not, per the standard, reach
the page. This module therefore does **not** silently inherit it — but it
does not silently ignore it either, because an ancestor `/Tabs` is a fact
about the file that changes what another viewer might do. It is a **third
state**, [`TabsEntry::OnAncestor`], reported in those words. Saying "no
`/Tabs`" over a file that plainly has one two levels up would be exactly the
kind of true-but-useless statement this view exists to avoid; treating it as
the page's own would be asserting an inheritance the standard denies.

`pdfcer-core` walks the ancestors too, in
`page_uses_structure_tab_order`, and that is correct for what it feeds:
`FieldAuthorDisclosures::structure_tab_order` warns an author that a newly
created field may have no tab position, and warning on an ancestor
`/Tabs /S` is the cautious direction. **A conservative walk is not an
application of inheritance**, and the distinction is what the two ends of
this seam have to keep saying the same way: a false sentence in a doc
comment propagates outward as a fact about the format.

### And a page with no `/Tabs` at all gets no mode name

[`TabsEntry::Absent`] is reported as *absent*, never as "manual",
"unspecified" or any other label. `D:\Dev\pdfcer`'s own roadmap records
*"what Acrobat's 'Unspecified' tab-order state mechanically denotes"* as
**unsourced after two attempts**, so naming it would be asserting something
nobody has been able to source. What the view says instead is what the file
says — there is no `/Tabs` here — plus the operationally useful half: absent
`/Tabs`, the `/Annots` order is what viewers use, and the `/Annots` order is
what is on screen.

## 5. What cannot appear, counted rather than dropped

Four things, and each is a different fact:

| Count | What it is | Why it cannot be a row |
|---|---|---|
| [`Listing::fields_without_widgets`] | a `/Fields` entry with no `/Widget` anywhere | tab order is a property of a page; a field with no widget is on no page |
| [`PageTabs::unclaimed`] | a `/Widget` in `/Annots` that no listed field owns | there is no field name to put in the row |
| [`PageTabs::anonymous`] | a `/Widget` written as a **direct dictionary** inside `/Annots` | Table 164 requires an indirect object; with no identity it cannot be matched to a field at all |
| [`PageTabs::other_annots`] | a non-`/Widget` annotation on the page | it is in the tab sequence and is not a form field — this list is form fields |

The last one matters more than it looks. §12.5.1's tab order is over
**annotations**, not over form fields: a `/Link` on the page occupies a
position in the sequence. A list of widgets alone is therefore a list of the
form fields *in* the sequence rather than the sequence itself, and the count
is what stops an operator concluding the numbering is wrong.

### `inline_field_roots` is NOT re-counted here

`pdfcer_core::forms::AcroForm::inline_field_roots` counts `/Fields` entries
written as direct dictionaries, which `parse_acroform` skips and which
`crate::panels::forms::header` already discloses above this section. Their
widgets, if any, would land in [`PageTabs::unclaimed`] — a count of
**widgets on a page**, which is a different number about a different thing.
The view's wording for that count says so rather than presenting it as a
second discovery.

## Grouping nodes have no place here

`AcroForm::groups` models non-terminal `/Fields` entries — the intermediate
nodes a fully-qualified name is built from. A grouping node has no `/Widget`
of its own by definition, so it is on no page, so it has no tab position.
Nothing here reads `groups`, and nothing should: inventing a row for one
would put something in a tab sequence that a viewer will never stop on.

## Read the SESSION, not the file on disk

[`collect`] takes an [`ObjectGraph`], and the body hands it
`doc.session.view()` — the base revision with **every unsaved edit
applied**, which is the same thing the canvas rasterizes.
`crate::panels::comments::model` and `crate::panels::forms` both carry this
sentence, and it binds here for a sharper reason than usual: a fill
regenerates a widget's `/AP` but does not touch `/Annots`, so the order is
stable across fills — but a future reorder would change it, and a view
reading the file on disk would show the operator the order they had just
changed away from.

## Item notes

### `fn empty_form`

Written out rather than defaulted because `AcroForm` has no `Default`
impl — deliberately, in the engine: every one of these entries is a fact
read off a real dictionary, and a default would invent a document.

### `struct FakeGraph`

Two required methods, and the other four come free from the trait's
provided implementations — including `resolved`, which is what
[`tabs_name`] calls. So a `HashMap<ObjId, Object>` is a complete graph
for these purposes, and it lets the `/Tabs` tests state a page tree in
six lines instead of hand-assembling a PDF.

It is built by hand rather than from a fixture because **a test that
cannot reach the case is satisfied by any implementation.** Exactly one
of the eleven form
fixtures carries a `/Tabs` at all, none carries one on an ancestor, and
none carries an unrecognised name — so inheritance, precedence and the
catch-all would every one of them be untested against the corpus, and a
build that ignored ancestors entirely would pass.

### `fn the_five_tabs_names_decode_and_imply_the_right_sequence`

The `A` and `W` rows are the reason this test is worth writing rather
than obvious. They are **PDF 2.0** additions, they were not in the
version of Table 30 the engine's comment cites, and they are the only
two values under which this view's list *is* the tab order. A build that
swept them into the catch-all would show the "this is not the tab order"
warning over a page whose file explicitly asked for exactly this order.

### `fn a_page_with_no_tabs_entry_is_absent_and_unnamed`

The constraint this whole view is built around. `/Tabs` is Optional and
most files omit it, so this is the common case — and the temptation is
to give it a label ("manual", "unspecified") because Acrobat shows one.
What that state mechanically denotes is recorded in `D:\Dev\pdfcer`'s
roadmap as **unsourced after two attempts**, so a label here would be an
assertion nobody can support.

### `fn a_tabs_on_an_ancestor_is_neither_inherited_nor_hidden`

The correction in this module's §4, pinned from both directions. ISO
32000-2 Table 31 marks `Rotate` inheritable and marks `Tabs` merely
"(Optional; PDF 1.5)", and the table's preamble makes non-marked
attributes non-inheritable — so an ancestor's value does not reach the
page.

Both halves matter. Reporting it as [`TabsEntry::OnPage`] would assert
an inheritance the standard denies; reporting it as [`TabsEntry::Absent`]
would hide a fact about the file that changes what another viewer might
do. The third state is what lets the view say both true things.

### `fn the_nearest_declaration_wins`

`PageSlot::ancestors` is root-first, so a `.first()` here would read the
root's value and silently ignore an intermediate `Pages` node — which is
precisely the shape of file this precedence rule exists for (a `/Tabs`
set once for a chapter, overridden for one page).

### `fn the_structure_tab_order_fixture_declares_it_on_the_page`

The one fixture in the corpus with a `/Tabs`, and the case the brief
named as the one most worth seeing. Its page dictionary is
`<< /Type /Page /Parent 2 0 R … /Tabs /S >>`, so this is `OnPage`, not
`OnAncestor` — which is the assertion that would fail if the walk read
the ancestors first.

Note what the fixture does **not** have: an `/AcroForm`, or a single
annotation. So it exercises the `/Tabs` read and nothing else, and the
row assertions below use the form fixtures instead. Stated rather than
left to be rediscovered by whoever opens it expecting a form.

### `fn the_ordinary_form_fixtures_declare_no_tabs_at_all`

The other direction, and the one that would go wrong silently: if
[`page_tabs`] ever returned something for a page that declares nothing,
every form in the corpus would acquire a tab-order mode it does not
have, and the mislabelling would look like a document fact.

### `fn a_real_form_is_numbered_from_one_on_every_page`

The end-to-end shape of the read path. It asserts what a screenshot
cannot: that the positions are a dense 1..n **per page** rather than a
document-wide running total, and that every name is a field of this
form rather than a widget id rendered as text.

### `fn a_multi_widget_field_appears_once_per_widget`

This is **correct, not a duplicate**, and it is the single most likely
thing for a later reader to "fix". Tab order is a property of a page and
a field is a document-level thing, so a field with three widgets
genuinely occupies three positions — possibly in three different
sequences on three different pages. A view that de-duplicated by field
name would show two of them nowhere at all.

### `fn a_field_with_no_widget_is_counted_and_never_listed`

Asserted against the arithmetic rather than a named fixture: whichever
documents carry such a field, the count must equal the number of
`/Fields` entries with an empty `widgets`, and no row may name one.

### `fn a_widget_with_no_p_entry_is_still_listed`

`/P` is Optional (§12.5.2 Table 164) and frequently absent, and
`pdfcer-core` reads it without resolving through the graph, so a direct
`/P` reads as absent too. The obvious implementation of *"which page is
this widget on?"* — look up `Widget::page` — returns **nothing at all**
on such a form.

Every one of the eleven form fixtures writes `/P` on every widget, so a
test opening a fixture cannot reach it; `crate::canvas::forms::boxes`
carries the same test for the same reason, and the engine team hit the
case when a deliberate sabotage of their own `/P` handling passed against
their whole corpus.

Here the form is built by hand with `page: None` on its widget, and the
assertion is that the row is produced anyway — so every other assertion
in this module is also an assertion that `/P` is not consulted.

### `fn a_document_with_widgets_and_no_acroform_lists_them_all`

Two different facts that a single "not listed" number would blur. An
A document with WIDGETS and NO `/AcroForm` lists every one of
them as unclaimed.

The state pdfcer manufactures and could not display. `insert_pages`
copies everything reachable from a page; `/Annots` reaches the widgets;
`/AcroForm` is a **catalog** entry and is never in the copied set. So a
form's pages inserted into a CAD drawing arrive as boxes that draw like
fields, swallow every keystroke, and belong to nothing at all.

Asserted at the model rather than through the panel because the model
is where the `Option<&AcroForm>` lives; the panel's half is covered by
the driven check that inserts a form's pages and registers one of the
orphans.

### `fn a_directly_written_widget_is_anonymous_not_unclaimed`

Table 164 requires an annotation to be an indirect object. One written
inline has no `ObjId`, so it cannot be matched to a field even in
principle — which is a different statement from "no field claims it",
and merging the two would tell an operator their form was malformed when
the *annotation* was.

### `fn every_page_index_is_inside_the_document`

The page index is fed straight to
[`crate::app::actions::Action::GoToPage`], so an out-of-range value
would be a navigation to nowhere. It cannot happen — the index is the
enumeration of `slots` — and it is pinned anyway, because this is the
one number that leaves the view.
