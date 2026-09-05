//! # `panels::forms::tab_order::model` — turning a document into a per-page
//! widget sequence, and reading what the file says about tab order
//!
//! The whole of the Tab order view that is not drawing. [`collect`] walks every
//! page's `/Annots`, keeps the `/Widget` entries in the order the array lists
//! them, matches each one back to the field it belongs to, counts everything it
//! could **not** list, and reads each page's `/Tabs` entry. Nothing here touches
//! `egui`.
//!
//! `crate::panels::comments::model` is the shape this follows and for the same
//! reason: every interesting decision in this view is a **classification**, and
//! a classification is only testable if it is separable from the widget that
//! shows it. The list of things that have to be right —
//!
//! - which annotations are widgets and which are not,
//! - which widget belongs to which field, and which of that field's widgets it
//!   is,
//! - what the file's `/Tabs` entry says, whether it is on the page itself or on
//!   an ancestor, and what that means for whether this list *is* the tab order,
//! - and everything that cannot appear in the list at all, counted rather than
//!   dropped
//!
//! — is exactly the list `tests` below sweeps, against real engine fixtures for
//! the ordinary shapes and against a hand-built graph for the shapes no fixture
//! in the corpus carries.
//!
//! ---
//!
//! ## ★ 1. The order is `/Annots` array order, which is also PAINT order
//!
//! Not `/AcroForm` `/Fields` order. The two commonly differ, and they answer
//! different questions: `/Fields` order is the document's declaration order,
//! which is what the existing fill list uses because it matches the printed
//! form; `/Annots` order is the order the annotations are listed **on one
//! page**, which is the order they are painted in and — absent a `/Tabs` entry
//! — the order a viewer tabs through them in.
//!
//! `pdfcer_core::edit::EditSession::widget_rects`' own doc comment states both
//! halves: *"`/Annots` array order — which is **paint order**, and (absent
//! `/Tabs`) also the tab order. It is not `/AcroForm` `/Fields` order, and the
//! two commonly differ."*
//!
//! The paint-order half is worth saying out loud in the view, because it is the
//! fact that would make a future *reorder* consequential: moving a widget
//! earlier in `/Annots` does not only move it earlier in the tab sequence, it
//! also moves it **underneath** anything that now follows it.
//!
//! ## ★ 2. Why this walks `page_annotations` rather than `widget_rects`
//!
//! `EditSession::widget_rects(page)` is the verb this shell asked for and got
//! (engine `e8e9881`), it returns exactly this order, and the canvas hit test
//! uses it. It is **not** the right source for a *list*, and the difference is
//! not stylistic.
//!
//! Read against the engine (`edit.rs:15786-15809`), `widget_rects` is
//! `annot::page_annotations` — the same walk this module uses — followed by
//! `filter(subtype == b"Widget")` and then a `filter_map` that drops any
//! annotation whose `/Rect` is absent **or** whose object identity is absent.
//! Both drops are right for a hit test: you cannot click a rectangle that does
//! not exist, and you cannot fill a widget you cannot name. Both are wrong for
//! a list, where a widget the file lists is a widget in the tab sequence
//! whatever else is missing from it — and dropping it silently would make this
//! view under-report the very thing it exists to report.
//!
//! So this module calls `page_annotations` directly, which is what
//! `widget_rects` calls, and keeps what `widget_rects` filters out **as counts**
//! ([`PageTabs::anonymous`]). The order is byte-identical because it is the same
//! array read by the same function.
//!
//! ## ★ 3. `/P` is never consulted — which page a widget is on is answered by
//! `/Annots`
//!
//! The correction `crate::canvas::forms::boxes::place` records, applied here
//! from the start. `/P` is *Optional* (§12.5.2 Table 164), it is frequently
//! absent in the wild, and `pdfcer-core` reads it without resolving through the
//! graph — so a direct rather than indirect `/P` also reads as absent. An
//! implementation that asked each widget which page it claims returns **nothing
//! at all** for a large class of real forms: no error, no refusal, no trace.
//!
//! No test written against the fixture corpus can catch that, because all
//! eleven form fixtures in `D:\Dev\pdfcer\fixtures\synthetic\forms\` write `/P`
//! on every widget. The direction is therefore inverted here as it is there:
//! each **page** is asked which widgets it lists. `pdfcer_core::forms::Widget`'s
//! `page` field is not read anywhere in this module, and
//! [`tests::a_widget_with_no_p_entry_is_still_listed`] builds a form that omits
//! it so that every other assertion doubles as proof the key is unread.
//!
//! ## ★ 4. `/Tabs` — what this reads, and the one place it departs from the
//! engine
//!
//! ### What the standard actually says
//!
//! ISO 32000-2:2020 Table 31 (entries in a page object), verbatim from
//! `D:\Dev\Rag-Specialized\PDF_Spec\_sources\ISO_32000-2_sponsored_EC3.pdf`
//! p. 107:
//!
//! > **`Tabs`** *name* — (Optional; PDF 1.5) A name specifying the tab order
//! > that shall be used for annotations on the page (see 12.5 "Annotations").
//! > If present, the values shall be one of `R` (row order), `C` (column
//! > order), and `S` (structure order). **Beginning with PDF 2.0, additional
//! > values also include `A` (annotations array order) and `W` (widget order).**
//! > Annotations array order refers to the order of the annotation enumerated
//! > in the `Annots` entry of the Page dictionary. Widget order means using the
//! > same array ordering but making two passes, the first only picking the
//! > widget annotations and the second picking all other annotations.
//!
//! and §12.5.1, same document, p. 466, which is where `R`, `C` and `S` are
//! defined operationally: `R` and `C` are visited *"in rows running
//! horizontally across the page"* / *"in columns running vertically"*, ordered
//! by the viewer preferences' `/Direction`; `S` is *"the order in which they
//! appear in the structure tree"*, and *"the order for annotations that are not
//! included in the structure tree is determined in a manner of the interactive
//! PDF processor's choosing."*
//!
//! That gives [`TabsMode`] its five named values and [`TabsMode::sequence`] its
//! whole content:
//!
//! | `/Tabs` | Where the order comes from | Is this list the tab order? |
//! |---|---|---|
//! | `/A` | the `/Annots` array | **yes** — that is this list |
//! | `/W` | the `/Annots` array, widgets first | **yes**, for widgets |
//! | `/R` | where the fields sit on the page | no — derived, not stored |
//! | `/C` | where the fields sit on the page | no — derived, not stored |
//! | `/S` | the document's structure (tag) tree | no — derived, not stored |
//! | anything else | unknown | unknown |
//!
//! ### ★ `/Tabs` is NOT an inheritable page attribute, and this is a
//! correction
//!
//! Both `pdfcer-core` (`edit.rs:7508`, *"`/Tabs` is inheritable through the page
//! tree (Table 30), so an absent entry on the page itself is not the answer —
//! the ancestors are walked"*) and the brief that commissioned this view state
//! that `/Tabs` is inheritable. **The primary source says otherwise, and it
//! says so twice.**
//!
//! 1. ISO 32000-2 Table 31 marks `Rotate` *"(Optional; inheritable)"* and marks
//!    `Tabs` *"(Optional; PDF 1.5)"* — no inheritability marker. Verified by
//!    reading the two rows out of the sponsored copy in the spec RAG, p. 105
//!    and p. 107.
//! 2. The table's own preamble (§7.7.3.3): *"Attributes that are **not**
//!    explicitly identified in the table as inheritable **shall not** be
//!    inherited."* `D:\Dev\Rag-Specialized\PDF_Spec\iso32000\iso32000__s__7.7.3.md`
//!    records the exhaustive search of that table and finds **exactly four**
//!    inheritable attributes: `Resources`, `MediaBox`, `CropBox`, `Rotate`.
//!
//! So a `/Tabs` on an ancestor `Pages` node does not, per the standard, reach
//! the page. This module therefore does **not** silently inherit it — but it
//! does not silently ignore it either, because an ancestor `/Tabs` is a fact
//! about the file that changes what another viewer might do. It is a **third
//! state**, [`TabsEntry::OnAncestor`], reported in those words. Saying "no
//! `/Tabs`" over a file that plainly has one two levels up would be exactly the
//! kind of true-but-useless statement this view exists to avoid; treating it as
//! the page's own would be asserting an inheritance the standard denies.
//!
//! The engine is not wrong in *effect* for its own purpose — it uses the walk
//! only to decide whether to warn an author that a newly created field has no
//! tab position, and warning on an ancestor `/Tabs /S` is the cautious
//! direction. It is wrong as a statement about the format, and this shell must
//! not repeat it as one.
//!
//! ### And a page with no `/Tabs` at all gets no mode name
//!
//! [`TabsEntry::Absent`] is reported as *absent*, never as "manual",
//! "unspecified" or any other label. `D:\Dev\pdfcer`'s own roadmap records
//! *"what Acrobat's 'Unspecified' tab-order state mechanically denotes"* as
//! **unsourced after two attempts**, so naming it would be asserting something
//! nobody has been able to source. What the view says instead is what the file
//! says — there is no `/Tabs` here — plus the operationally useful half: absent
//! `/Tabs`, the `/Annots` order is what viewers use, and the `/Annots` order is
//! what is on screen.
//!
//! ## ★ 5. What cannot appear, counted rather than dropped
//!
//! Four things, and each is a different fact:
//!
//! | Count | What it is | Why it cannot be a row |
//! |---|---|---|
//! | [`Listing::fields_without_widgets`] | a `/Fields` entry with no `/Widget` anywhere | tab order is a property of a page; a field with no widget is on no page |
//! | [`PageTabs::unclaimed`] | a `/Widget` in `/Annots` that no listed field owns | there is no field name to put in the row |
//! | [`PageTabs::anonymous`] | a `/Widget` written as a **direct dictionary** inside `/Annots` | Table 164 requires an indirect object; with no identity it cannot be matched to a field at all |
//! | [`PageTabs::other_annots`] | a non-`/Widget` annotation on the page | it is in the tab sequence and is not a form field — this list is form fields |
//!
//! The last one matters more than it looks. §12.5.1's tab order is over
//! **annotations**, not over form fields: a `/Link` on the page occupies a
//! position in the sequence. A list of widgets alone is therefore a list of the
//! form fields *in* the sequence rather than the sequence itself, and the count
//! is what stops an operator concluding the numbering is wrong.
//!
//! ### `inline_field_roots` is NOT re-counted here
//!
//! `pdfcer_core::forms::AcroForm::inline_field_roots` counts `/Fields` entries
//! written as direct dictionaries, which `parse_acroform` skips and which
//! `crate::panels::forms::header` already discloses above this section. Their
//! widgets, if any, would land in [`PageTabs::unclaimed`] — a count of
//! **widgets on a page**, which is a different number about a different thing.
//! The view's wording for that count says so rather than presenting it as a
//! second discovery.
//!
//! ## Grouping nodes have no place here
//!
//! `AcroForm::groups` models non-terminal `/Fields` entries — the intermediate
//! nodes a fully-qualified name is built from. A grouping node has no `/Widget`
//! of its own by definition, so it is on no page, so it has no tab position.
//! Nothing here reads `groups`, and nothing should: inventing a row for one
//! would put something in a tab sequence that a viewer will never stop on.
//!
//! ## Read the SESSION, not the file on disk
//!
//! [`collect`] takes an [`ObjectGraph`], and the body hands it
//! `doc.session.view()` — the base revision with **every unsaved edit
//! applied**, which is the same thing the canvas rasterizes.
//! `crate::panels::comments::model` and `crate::panels::forms` both carry this
//! sentence, and it binds here for a sharper reason than usual: a fill
//! regenerates a widget's `/AP` but does not touch `/Annots`, so the order is
//! stable across fills — but a future reorder would change it, and a view
//! reading the file on disk would show the operator the order they had just
//! changed away from.

use std::collections::HashMap;

use super::tabs::{Sequence, TabsEntry, page_tabs};
use pdfcer_core::annot::page_annotations;
use pdfcer_core::forms::AcroForm;
use pdfcer_core::graph::ObjectGraph;
use pdfcer_core::object::ObjId;
use pdfcer_core::page_tree::PageSlot;

/// The whole view's content, computed from one walk of the document.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Listing {
    /// One entry per page, in page order. A page with no widget on it is still
    /// present — its `/Tabs` state is a fact about the document, and a gap in
    /// the page numbering would read as a bug.
    pub pages: Vec<PageTabs>,
    /// Fields in `/AcroForm` `/Fields` with **no widget at all**.
    ///
    /// They cannot appear in any page's list, because tab order is per page and
    /// a field with no widget is on no page. Disclosed rather than silently
    /// absent; see this module's §5.
    pub fields_without_widgets: usize,
}

impl Listing {
    /// How many widget rows the whole document produced.
    #[must_use]
    pub fn total_rows(&self) -> usize {
        self.pages.iter().map(|p| p.rows.len()).sum()
    }

    /// Whether **any** page in the document declares a `/Tabs` on itself.
    ///
    /// Used only by the trace. Deliberately not used to decide anything on
    /// screen: every page states its own `/Tabs` situation beside its own rows,
    /// because a document-wide summary of a per-page property would be right
    /// about the document and wrong about the page the operator is reading.
    #[must_use]
    pub fn any_page_declares_tabs(&self) -> bool {
        self.pages
            .iter()
            .any(|p| matches!(p.tabs, TabsEntry::OnPage(_)))
    }

    /// How many pages show a sequence that is **not** the tab order.
    ///
    /// A page under `/Tabs /R`, `/C` or `/S` has an order a viewer *derives*
    /// rather than one the file stores, so the `/Annots` sequence on screen is
    /// not it. Counted so the trace can prove the disclosure fired.
    #[must_use]
    pub fn pages_with_derived_order(&self) -> usize {
        self.pages
            .iter()
            .filter(|p| matches!(p.tabs.sequence(), Sequence::Derived))
            .count()
    }
}

/// One page: what the file says about its tab order, and what is on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageTabs {
    /// **0-based** page index — what [`crate::app::actions::Action::GoToPage`]
    /// takes. The `+ 1` happens only where a human reads it.
    pub page_index: usize,
    /// What the file's `/Tabs` entry says, and where it was found.
    pub tabs: TabsEntry,
    /// Every `/Widget` this page's `/Annots` lists that a field claims, in
    /// `/Annots` order.
    pub rows: Vec<TabRow>,
    /// `/Widget` annotations with an object identity that no listed field
    /// claims — **the ids themselves**, in `/Annots` order. See this module's
    /// §5.
    ///
    /// # ★ Why this is a list of ids and was a bare count
    ///
    /// It was `usize` for the whole of its life, because the only thing anybody
    /// could do with an unclaimed widget was be told about it. The sentence it
    /// fed even said so, hedging at the end — *"if the form declares entries
    /// pdfcer could not read, these may be theirs"* — which is a guess offered
    /// because there was nothing better to offer.
    ///
    /// `EditSession::adopt_widget` shipped 2026-08-19 and **takes an `ObjId`**.
    /// The moment a verb exists that can act on one of these, a count is the
    /// wrong shape: it can be displayed and it cannot be pressed. Widening it
    /// here rather than re-walking `/Annots` at the button keeps one walk as
    /// the single answer to *"which widgets does this page list that no field
    /// owns"*, which is what stops the list and the action from ever
    /// disagreeing about the set.
    ///
    /// Order is `/Annots` order, which is the order [`Self::rows`] is in, so a
    /// registered widget appears in the rows at the position its unclaimed
    /// entry occupied — the list does not reshuffle under the operator when
    /// they press the button.
    pub unclaimed: Vec<Unclaimed>,
    /// `/Widget` annotations written as **direct dictionaries** inside
    /// `/Annots`, which have no identity to match against. See §5.
    pub anonymous: usize,
    /// **Every indirect entry of this page's `/Annots`, in array order** —
    /// the sequence a reorder is expressed over.
    ///
    /// # ★★ Why the rows are not enough, which is the whole reason this exists
    ///
    /// [`Self::rows`] is **widgets a field claims**. `/Annots` holds more: the
    /// unclaimed widgets, the anonymous ones, and every `/Link`, `/Text` and
    /// markup annotation on the page. `EditSession::reorder_annotations` takes
    /// *"the page's indirect `/Annots` entries, each once, in the wanted
    /// order"*, so a list built from the rows alone is not a permutation of it
    /// — it is a permutation of a subset, and the engine refuses it by name
    /// (`AnnotsNotAPermutation { missing, .. }`). Correctly: silently dropping
    /// the entries a form panel does not care about would delete a page's
    /// links.
    ///
    /// # ★ Entries with no id are absent from this list, deliberately
    ///
    /// A `/Widget` written as a direct dictionary has nothing to name it by.
    /// The engine's contract is that such an entry is **pinned** — it keeps its
    /// index and the rest flow around it — and the way to ask for that is to
    /// omit it. So this list is shorter than the array whenever
    /// [`Self::anonymous`] is non-zero, and that is the correct shape rather
    /// than a lossy one.
    pub annots: Vec<ObjId>,
    /// Annotations on this page that are not `/Widget`s. They are in the tab
    /// sequence and are not form fields. See §5.
    pub other_annots: usize,
}

/// One `/Widget` this page lists that no field in the form claims.
///
/// # ★ Why this is a struct and not the bare `ObjId`
///
/// Because two independent things are true of it and both are needed at the
/// same moment: it is a **thing to register** (the id, which
/// `EditSession::adopt_widget` takes) and it is a **place in the tab
/// sequence** (the position, which is how an operator finds the box on the
/// page — they tab to it and watch the focus ring land).
///
/// The position cannot be recovered from the id afterwards without walking
/// `/Annots` again, and a second walk is a second answer to a question this
/// module exists to answer once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unclaimed {
    /// The widget's object identity — what `adopt_widget` takes.
    pub id: ObjId,
    /// **1-based** position among the widgets on this page, on the same
    /// numbering as [`TabRow::position`].
    pub position: usize,
}

impl PageTabs {
    /// How many `/Widget` annotations this page's `/Annots` listed, whether or
    /// not a row could be made for each.
    #[must_use]
    pub fn widgets_seen(&self) -> usize {
        self.rows.len() + self.unclaimed.len() + self.anonymous
    }
}

/// One widget, as a row.
///
/// Owned strings rather than borrows. The `AcroForm` is parsed fresh inside the
/// panel body and dropped when the frame ends, so borrowing would tie the
/// listing's lifetime to a temporary; and the whole listing is a few hundred
/// short strings at most.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabRow {
    /// **The annotation's object id** — what a reorder is expressed in.
    ///
    /// ★★★ Added 2026-09-02 with `OPERATOR_REQUESTS.md` O99, and the engine
    /// asked for it by name: `EditSession::reorder_annotations` takes
    /// `&[ObjId]`, **not indices**, and its reply says why in one sentence —
    /// *"the index you hold is almost never a raw `/Annots` index:
    /// `page_annotations` skips null and non-dictionary entries, so the
    /// numberings diverge on exactly the malformed files where a guess costs
    /// most."*
    ///
    /// ★★ [`Self::position`] is therefore **not** an address. It is a number an
    /// operator counts while tabbing, it is 1-based, it counts widgets only,
    /// and passing it to the engine would be wrong on three axes at once. This
    /// field is the address; that one is the label.
    pub id: ObjId,
    /// **This row's index within [`PageTabs::annots`]** — the slot it occupies
    /// in the page's array of indirect annotations.
    ///
    /// # ★★ Why a second number, when the row already has two
    ///
    /// Because [`Self::position`] is a **label** and this is an **address**,
    /// and they count different things. `position` is 1-based and counts
    /// widgets only — it is the number of Tab presses that reach this box, and
    /// an unclaimed widget or a `/Link` between two rows moves one and not the
    /// other. Using `position` to index the array would be wrong on three axes
    /// at once (off by the base, off by the non-widgets, off by the entries
    /// with no id).
    ///
    /// A drag reorders the *rows*; the commit has to reorder the *array*. This
    /// is the only thing that maps one onto the other, and computing it at the
    /// drag by re-walking `/Annots` would be a second walk that could disagree
    /// with the first.
    pub slot: usize,
    /// **1-based** position among the *widgets* on this page.
    ///
    /// Among the widgets rather than among all annotations, because that is the
    /// number an operator counts when they tab through a form. The
    /// [`PageTabs::other_annots`] count is what makes that honest: the true
    /// annotation sequence is longer, and the view says so.
    pub position: usize,
    /// The field's **fully-qualified name** — what every fill verb takes, and
    /// what an operator matching this against a data file needs.
    pub field: String,
    /// `/TU`, the field's alternate (accessible) name, when it has a non-blank
    /// one.
    ///
    /// The same preference `crate::panels::forms::rows::row_label` applies, and
    /// for the same reason: `/TU` is what a screen reader announces, so it is
    /// the string the operator should be reading. The raw
    /// [`Self::field`] name is still one hover away, through
    /// `crate::text::forms::form_field_row_tooltip` — the same pairing the fill
    /// rows use, because this view's whole job is to be matched against the
    /// file and `/TU` is not what a data file matches on.
    pub label: Option<String>,
    /// **0-based** index of this widget within its field's `widgets`.
    pub widget: usize,
    /// How many widgets the field has in total, across the whole document.
    ///
    /// Carried so a row can say *"widget 2 of 3"*. A field with widgets on
    /// several pages appears once per widget, on each page it has one on, and
    /// that is **correct rather than a duplicate**: tab order is per page and a
    /// field is a document-level thing, so the same field genuinely occupies a
    /// position in two different sequences.
    pub widget_count: usize,
}

/// Build the listing for a whole document.
///
/// `graph` must be the **session view**, not the loaded file — see this
/// module's header. `slots` is `EditSession::page_slots()`, whose index is the
/// page index every row carries and whose `ancestors` [`page_tabs`] reads.
/// `form` is the parsed `/AcroForm` the panel body already has.
///
/// # Cost
///
/// One `/Annots` walk per page, bounded by
/// `pdfcer_core::annot::MAX_ANNOTS_PER_PAGE`, plus one `HashMap` of widget ids
/// built once for the whole document rather than once per page. That is the
/// Comments panel's cost exactly, and its header's measurement applies:
/// negligible beside a raster on anything this project measures against.
#[must_use]
pub fn collect<G: ObjectGraph + ?Sized>(
    graph: &G,
    slots: &[PageSlot],
    form: Option<&AcroForm>,
) -> Listing {
    // ★★ `Option`, and `None` is the case this view matters MOST in.
    //
    // A document can carry `/Widget` annotations and **no `/AcroForm` at all**,
    // and pdfcer makes exactly that: `insert_pages` copies everything reachable
    // from a page, `/Annots` reaches the widgets, and `/AcroForm` is a catalog
    // entry that is never in the set being copied. Insert a form's pages into a
    // CAD drawing and every widget that arrives is unclaimed, because there is
    // nothing in the target that could claim one.
    //
    // Taking `&AcroForm` made that state unrepresentable here, so the panel
    // returned early on it and this section — the one surface that lists those
    // widgets and offers to register them — was unreachable in the only
    // situation that produces them. Found by a driven run on 2026-08-19, and it
    // is the second instance of that shape in one day: the Bookmarks panel had
    // the same early return over an empty outline.
    //
    // With `None` the owner map is empty, every widget falls to `unclaimed`,
    // and the rest of this function is unchanged. That is the correct answer
    // rather than a special case.
    // Widget object id -> (field index, widget index within that field).
    //
    // Built once for the document rather than per page: a form with 400 fields
    // on 36 sheets would otherwise do 36 linear scans of the whole field tree.
    // First listing wins — a widget claimed by two fields is malformed, and the
    // earlier field is the one `/Fields` declares first.
    let mut owner: HashMap<ObjId, (usize, usize)> = HashMap::new();
    let fields: &[pdfcer_core::forms::Field] = form.map_or(&[], |f| f.fields.as_slice());
    for (field_index, field) in fields.iter().enumerate() {
        for (widget_index, widget) in field.widgets.iter().enumerate() {
            owner
                .entry(widget.id)
                .or_insert((field_index, widget_index));
        }
    }

    let mut listing = Listing {
        pages: Vec::with_capacity(slots.len()),
        fields_without_widgets: fields.iter().filter(|f| f.widgets.is_empty()).count(),
    };

    for (page_index, slot) in slots.iter().enumerate() {
        let mut page = PageTabs {
            page_index,
            tabs: page_tabs(graph, slot),
            rows: Vec::new(),
            annots: Vec::new(),
            unclaimed: Vec::new(),
            anonymous: 0,
            other_annots: 0,
        };

        // Every `/Widget` this page lists advances this, whether or not a row
        // can be made for it. See the comment at the increment.
        let mut widget_ordinal = 0usize;
        for annot in page_annotations(graph, slot.id) {
            // ★ FIRST, and for EVERY entry — before the widget test, before the
            // ownership test, before anything this panel cares about.
            //
            // This is the array a reorder is a permutation of, and it is the
            // page's array, not this panel's view of it. A `/Link` is not a
            // form field and is not listed anywhere below; it is still an entry
            // that has to appear in a permutation or the engine refuses the
            // whole call. The one exclusion is an entry with no id, which is
            // pinned by omission — see [`PageTabs::annots`].
            if let Some(id) = annot.id {
                page.annots.push(id);
            }
            // Not a widget: it is still in the tab sequence, and it is not a
            // form field. Counted, never listed — see §5.
            if !annot.is_widget() {
                page.other_annots += 1;
                continue;
            }
            // No object identity: a `/Widget` written as a direct dictionary
            // inside `/Annots`, which Table 164 forbids. There is nothing to
            // match it against, so no row can name a field for it.
            let Some(id) = annot.id else {
                // Advances the ordinal for the same reason the unclaimed case
                // does: it IS a widget in the page's `/Annots`, so a viewer
                // tabbing through the page stops on it. It cannot be listed —
                // there is no identity to name it by — but the widgets after it
                // are genuinely one position later than they would otherwise be.
                widget_ordinal += 1;
                page.anonymous += 1;
                continue;
            };
            // ★ Counted here, BEFORE the ownership question — which is the
            // fix for a numbering defect this widening exposed.
            //
            // The position used to be `rows.len() + 1`, and `rows` holds only
            // the widgets a field claims. So an unclaimed or anonymous widget
            // between two rows did not advance the count, and every row after
            // it was numbered one too low — while the field's own doc comment
            // said the number was *"among the widgets on this page"*, which is
            // what it now is.
            //
            // Invisible while the other two were bare counts: an operator could
            // read "3 widgets belong to no field" and had nothing to line the
            // numbers up against. It stops being invisible the moment those
            // widgets are listed with positions beside the rows, and two
            // sequences interleaved on one page must agree or neither is worth
            // printing. It also matters more than a display nicety, because the
            // number is what an operator uses to *find* the box: they press Tab
            // that many times and expect the focus ring to land on it.
            widget_ordinal += 1;
            let Some(&(field_index, widget_index)) = owner.get(&id) else {
                page.unclaimed.push(Unclaimed {
                    id,
                    position: widget_ordinal,
                });
                continue;
            };
            let field = &fields[field_index];
            page.rows.push(TabRow {
                // ★ The address a reorder is expressed in — see the field.
                id,
                // The entry this loop pushed at the top of THIS iteration, so
                // it is this annotation's slot by construction rather than by
                // arithmetic. `- 1` cannot underflow: the push above ran, and
                // it ran unconditionally for any entry that reaches here (an
                // entry with no id took the `else` branch and never arrives).
                slot: page.annots.len() - 1,
                // 1-based, and among the WIDGETS — so it is the number an
                // operator counts while tabbing.
                position: widget_ordinal,
                field: field.fully_qualified_name.clone(),
                // A blank-but-present `/TU` is treated as absent rather than
                // shown, the same judgement `rows::row_label` makes: the file
                // has technically supplied a label and honouring it literally
                // would produce a row identified by nothing.
                label: field
                    .alternate_name
                    .as_ref()
                    .map(|b| String::from_utf8_lossy(b).into_owned())
                    .filter(|s| !s.trim().is_empty()),
                widget: widget_index,
                widget_count: field.widgets.len(),
            });
        }

        listing.pages.push(page);
    }

    listing
}

#[cfg(test)]
mod tests {
    // `Object` is a TEST-only need here since the `/Tabs` reading moved to
    // `super::tabs`: the fake graph builds page dictionaries by hand.
    use pdfcer_core::object::Object;
    // Tests that exercise the `/Tabs` reading through `collect` still name its
    // types; the reading itself now lives in `super::tabs`.
    use super::super::tabs::TabsMode;
    use super::*;
    use crate::panels::objects::test_support::engine_fixture;
    use pdfcer_core::document::Document;
    use pdfcer_core::edit::EditSession;
    use pdfcer_core::object::Dict;

    /// An `/AcroForm` with no fields.
    ///
    /// Written out rather than defaulted because `AcroForm` has no `Default`
    /// impl — deliberately, in the engine: every one of these entries is a fact
    /// read off a real dictionary, and a default would invent a document.
    fn empty_form() -> AcroForm {
        AcroForm {
            fields: Vec::new(),
            groups: Vec::new(),
            need_appearances: false,
            sig_flags: 0,
            signatures_exist: false,
            append_only: false,
            calc_order_count: 0,
            calc_order: Vec::new(),
            has_default_resources: false,
            default_appearance: None,
            quadding: pdfcer_core::vartext::Quadding::Left,
            xfa: pdfcer_core::forms::XfaPresence::None,
            inline_field_roots: 0,
        }
    }

    /// Open a fixture and collect it through the same path the body uses.
    fn listing(rel: &str) -> Listing {
        let path = engine_fixture(rel);
        let doc = Document::load(&path).expect("the fixture loads");
        let session = EditSession::new(doc);
        let slots = session.page_slots().expect("a page tree");
        let view = session.view();
        let form = pdfcer_core::forms::parse_acroform(&view).unwrap_or_else(empty_form);
        collect(&view, &slots, Some(&form))
    }

    // =======================================================================
    // A hand-built graph, for the shapes no fixture in the corpus carries
    // =======================================================================

    /// The smallest thing that satisfies [`ObjectGraph`].
    ///
    /// Two required methods, and the other four come free from the trait's
    /// provided implementations — including `resolved`, which is what
    /// [`tabs_name`] calls. So a `HashMap<ObjId, Object>` is a complete graph
    /// for these purposes, and it lets the `/Tabs` tests state a page tree in
    /// six lines instead of hand-assembling a PDF.
    ///
    /// It is built by hand rather than from a fixture for the reason
    /// `HANDOFF.md` §2 keeps making: **a test that cannot reach the case is
    /// satisfied by any implementation.** Exactly one of the eleven form
    /// fixtures carries a `/Tabs` at all, none carries one on an ancestor, and
    /// none carries an unrecognised name — so inheritance, precedence and the
    /// catch-all would every one of them be untested against the corpus, and a
    /// build that ignored ancestors entirely would pass.
    struct FakeGraph(HashMap<ObjId, Object>);

    impl ObjectGraph for FakeGraph {
        fn value(&self, id: ObjId) -> Option<&Object> {
            self.0.get(&id)
        }
        fn trailer_entry(&self, _key: &[u8]) -> Option<&Object> {
            None
        }
    }

    /// `/Type /Pages` or `/Type /Page`, with an optional `/Tabs`.
    fn node(tabs: Option<&[u8]>) -> Object {
        let mut d = Dict::new();
        if let Some(t) = tabs {
            d.insert(
                pdfcer_core::object::Name(b"Tabs".to_vec()),
                Object::Name(pdfcer_core::object::Name(t.to_vec())),
            );
        }
        Object::Dict(d)
    }

    /// A three-level tree: root(4) -> mid(3) -> page(1), with whatever `/Tabs`
    /// each level carries.
    fn tree(page: Option<&[u8]>, mid: Option<&[u8]>, root: Option<&[u8]>) -> (FakeGraph, PageSlot) {
        let page_id = ObjId::new(1, 0);
        let mid_id = ObjId::new(3, 0);
        let root_id = ObjId::new(4, 0);
        let graph = FakeGraph(HashMap::from([
            (page_id, node(page)),
            (mid_id, node(mid)),
            (root_id, node(root)),
        ]));
        let slot = PageSlot {
            id: page_id,
            parent: Some(mid_id),
            index_in_parent: 0,
            // ROOT FIRST — the engine's documented order, and the thing
            // `page_tabs` reverses.
            ancestors: vec![root_id, mid_id],
            inherited: pdfcer_core::page_tree::InheritedRaw::default(),
        };
        (graph, slot)
    }

    /// **★ Every `/Tabs` name the standard defines is decoded, and the
    /// sequence each implies is the one ISO 32000-2 states.**
    ///
    /// The `A` and `W` rows are the reason this test is worth writing rather
    /// than obvious. They are **PDF 2.0** additions, they were not in the
    /// version of Table 30 the engine's comment cites, and they are the only
    /// two values under which this view's list *is* the tab order. A build that
    /// swept them into the catch-all would show the "this is not the tab order"
    /// warning over a page whose file explicitly asked for exactly this order.
    #[test]
    fn the_five_tabs_names_decode_and_imply_the_right_sequence() {
        assert_eq!(TabsMode::from_name(b"R"), TabsMode::Row);
        assert_eq!(TabsMode::from_name(b"C"), TabsMode::Column);
        assert_eq!(TabsMode::from_name(b"S"), TabsMode::Structure);
        assert_eq!(TabsMode::from_name(b"A"), TabsMode::AnnotsArray);
        assert_eq!(TabsMode::from_name(b"W"), TabsMode::Widgets);

        // Derived: the order is worked out, not stored, so this list is not it.
        for m in [TabsMode::Row, TabsMode::Column, TabsMode::Structure] {
            assert_eq!(m.sequence(), Sequence::Derived, "{m:?}");
        }
        // Stored, and stored as exactly the array this view walks.
        for m in [TabsMode::AnnotsArray, TabsMode::Widgets] {
            assert_eq!(m.sequence(), Sequence::AnnotsOrder, "{m:?}");
        }

        // A name nobody has defined is carried verbatim and answers "unknown"
        // — never "derived" (which would claim to know it is not this order)
        // and never "annots order" (which would claim it is).
        let odd = TabsMode::from_name(b"Q");
        assert_eq!(odd, TabsMode::Unrecognised("Q".to_owned()));
        assert_eq!(odd.sequence(), Sequence::Unknown);
        // PDF names are case-sensitive: `/r` is a different name from `/R`.
        assert_eq!(
            TabsMode::from_name(b"r"),
            TabsMode::Unrecognised("r".to_owned()),
            "a lower-case name is a DIFFERENT name, not a spelling variant"
        );
    }

    /// **★ A page with no `/Tabs` anywhere is ABSENT, and gets no mode name.**
    ///
    /// The constraint this whole view is built around. `/Tabs` is Optional and
    /// most files omit it, so this is the common case — and the temptation is
    /// to give it a label ("manual", "unspecified") because Acrobat shows one.
    /// What that state mechanically denotes is recorded in `D:\Dev\pdfcer`'s
    /// roadmap as **unsourced after two attempts**, so a label here would be an
    /// assertion nobody can support.
    #[test]
    fn a_page_with_no_tabs_entry_is_absent_and_unnamed() {
        let (graph, slot) = tree(None, None, None);
        assert_eq!(page_tabs(&graph, &slot), TabsEntry::Absent);
        // …and absent still answers the operationally useful question: with no
        // `/Tabs`, the `/Annots` order is what viewers use, which is this list.
        assert_eq!(TabsEntry::Absent.sequence(), Sequence::AnnotsOrder);
    }

    /// **★ A `/Tabs` on an ancestor is reported as an ancestor's, never as the
    /// page's own.**
    ///
    /// The correction in this module's §4, pinned from both directions. ISO
    /// 32000-2 Table 31 marks `Rotate` inheritable and marks `Tabs` merely
    /// "(Optional; PDF 1.5)", and the table's preamble makes non-marked
    /// attributes non-inheritable — so an ancestor's value does not reach the
    /// page.
    ///
    /// Both halves matter. Reporting it as [`TabsEntry::OnPage`] would assert
    /// an inheritance the standard denies; reporting it as [`TabsEntry::Absent`]
    /// would hide a fact about the file that changes what another viewer might
    /// do. The third state is what lets the view say both true things.
    #[test]
    fn a_tabs_on_an_ancestor_is_neither_inherited_nor_hidden() {
        let (graph, slot) = tree(None, None, Some(b"R"));
        assert_eq!(
            page_tabs(&graph, &slot),
            TabsEntry::OnAncestor(TabsMode::Row),
            "an ancestor's /Tabs must be reported AS an ancestor's"
        );
        // …and it does not change what the sequence on screen is taken to be,
        // because this build does not apply it.
        assert_eq!(
            page_tabs(&graph, &slot).sequence(),
            Sequence::AnnotsOrder,
            "an unapplied ancestor value must not relabel the sequence"
        );
    }

    /// **The page's own `/Tabs` wins over an ancestor's, and the NEAREST
    /// ancestor wins over the root.**
    ///
    /// `PageSlot::ancestors` is root-first, so a `.first()` here would read the
    /// root's value and silently ignore an intermediate `Pages` node — which is
    /// precisely the shape of file this precedence rule exists for (a `/Tabs`
    /// set once for a chapter, overridden for one page).
    #[test]
    fn the_nearest_declaration_wins() {
        let (graph, slot) = tree(Some(b"S"), Some(b"C"), Some(b"R"));
        assert_eq!(
            page_tabs(&graph, &slot),
            TabsEntry::OnPage(TabsMode::Structure),
            "the page's own entry must win"
        );

        let (graph, slot) = tree(None, Some(b"C"), Some(b"R"));
        assert_eq!(
            page_tabs(&graph, &slot),
            TabsEntry::OnAncestor(TabsMode::Column),
            "the nearest ancestor must win over the root — `ancestors` is \
             root-first, so this fails if the walk reads it forwards"
        );
    }

    /// A `/Tabs` that is not a name at all reads as absent rather than being
    /// coerced.
    #[test]
    fn a_malformed_tabs_value_reads_as_absent() {
        let page_id = ObjId::new(1, 0);
        let mut d = Dict::new();
        d.insert(
            pdfcer_core::object::Name(b"Tabs".to_vec()),
            // A string, not a name. Malformed, and "R" written this way is not
            // pdfcer's to interpret.
            Object::String(b"R".to_vec()),
        );
        let graph = FakeGraph(HashMap::from([(page_id, Object::Dict(d))]));
        let slot = PageSlot {
            id: page_id,
            parent: None,
            index_in_parent: 0,
            ancestors: Vec::new(),
            inherited: pdfcer_core::page_tree::InheritedRaw::default(),
        };
        assert_eq!(page_tabs(&graph, &slot), TabsEntry::Absent);
    }

    /// **★ `tagged-struct-tabs.pdf` really carries `/Tabs /S` on its page.**
    ///
    /// The one fixture in the corpus with a `/Tabs`, and the case the brief
    /// named as the one most worth seeing. Its page dictionary is
    /// `<< /Type /Page /Parent 2 0 R … /Tabs /S >>`, so this is `OnPage`, not
    /// `OnAncestor` — which is the assertion that would fail if the walk read
    /// the ancestors first.
    ///
    /// Note what the fixture does **not** have: an `/AcroForm`, or a single
    /// annotation. So it exercises the `/Tabs` read and nothing else, and the
    /// row assertions below use the form fixtures instead. Stated rather than
    /// left to be rediscovered by whoever opens it expecting a form.
    #[test]
    fn the_structure_tab_order_fixture_declares_it_on_the_page() {
        let path = engine_fixture("forms/tagged-struct-tabs.pdf");
        let doc = Document::load(&path).expect("the fixture loads");
        let session = EditSession::new(doc);
        let slots = session.page_slots().expect("a page tree");
        assert_eq!(slots.len(), 1);
        let view = session.view();
        assert_eq!(
            page_tabs(&view, &slots[0]),
            TabsEntry::OnPage(TabsMode::Structure)
        );
        assert_eq!(
            page_tabs(&view, &slots[0]).sequence(),
            Sequence::Derived,
            "under /Tabs /S the order comes from the tag tree, so an /Annots \
             list is NOT the tab order and the view must say so"
        );

        // …and the document has no form at all, so the listing is empty rather
        // than wrong.
        let l = listing("forms/tagged-struct-tabs.pdf");
        assert_eq!(l.total_rows(), 0);
        assert_eq!(l.pages.len(), 1);
    }

    /// **★ Every form fixture in the corpus reports its pages as `/Tabs`-less.**
    ///
    /// The other direction, and the one that would go wrong silently: if
    /// [`page_tabs`] ever returned something for a page that declares nothing,
    /// every form in the corpus would acquire a tab-order mode it does not
    /// have, and the mislabelling would look like a document fact.
    #[test]
    fn the_ordinary_form_fixtures_declare_no_tabs_at_all() {
        for f in [
            "forms/demo-form.pdf",
            "forms/multi-widget-form.pdf",
            "forms/nested-form.pdf",
            "forms/radio-group-form.pdf",
        ] {
            let l = listing(f);
            assert!(!l.pages.is_empty(), "{f} has no pages");
            for p in &l.pages {
                assert_eq!(p.tabs, TabsEntry::Absent, "{f} page {}", p.page_index);
            }
            assert!(
                !l.any_page_declares_tabs(),
                "{f} must declare no /Tabs anywhere"
            );
            assert_eq!(l.pages_with_derived_order(), 0, "{f}");
        }
    }

    /// **★ A real form produces rows, numbered from one per page, naming
    /// fields the form really has.**
    ///
    /// The end-to-end shape of the read path. It asserts what a screenshot
    /// cannot: that the positions are a dense 1..n **per page** rather than a
    /// document-wide running total, and that every name is a field of this
    /// form rather than a widget id rendered as text.
    #[test]
    fn a_real_form_is_numbered_from_one_on_every_page() {
        let path = engine_fixture("forms/demo-form.pdf");
        let doc = Document::load(&path).expect("the fixture loads");
        let session = EditSession::new(doc);
        let slots = session.page_slots().expect("a page tree");
        let view = session.view();
        let form = pdfcer_core::forms::parse_acroform(&view).expect("the fixture has a form");
        let l = collect(&view, &slots, Some(&form));

        assert!(l.total_rows() > 0, "a real form produced no rows at all");
        let names: std::collections::BTreeSet<&str> = form
            .fields
            .iter()
            .map(|f| f.fully_qualified_name.as_str())
            .collect();
        for page in &l.pages {
            for (i, row) in page.rows.iter().enumerate() {
                assert_eq!(row.position, i + 1, "positions restart at 1 on each page");
                assert!(
                    names.contains(row.field.as_str()),
                    "{} is not a field of this form",
                    row.field
                );
                assert!(row.widget < row.widget_count, "{row:?}");
            }
        }
    }

    /// **★ A field with several widgets appears once per widget, and each row
    /// says which one it is.**
    ///
    /// This is **correct, not a duplicate**, and it is the single most likely
    /// thing for a later reader to "fix". Tab order is a property of a page and
    /// a field is a document-level thing, so a field with three widgets
    /// genuinely occupies three positions — possibly in three different
    /// sequences on three different pages. A view that de-duplicated by field
    /// name would show two of them nowhere at all.
    #[test]
    fn a_multi_widget_field_appears_once_per_widget() {
        let l = listing("forms/multi-widget-form.pdf");
        let multi: Vec<&TabRow> = l
            .pages
            .iter()
            .flat_map(|p| p.rows.iter())
            .filter(|r| r.widget_count > 1)
            .collect();
        assert!(
            !multi.is_empty(),
            "this fixture exists to carry a multi-widget field; without one \
             the assertions below prove nothing: {:?}",
            l.pages
        );

        // Every widget of such a field is listed, each with a distinct index,
        // and the count on each row agrees with how many were listed.
        let mut by_field: HashMap<&str, Vec<usize>> = HashMap::new();
        for r in &multi {
            by_field.entry(r.field.as_str()).or_default().push(r.widget);
        }
        for (field, mut indices) in by_field {
            let expected = multi
                .iter()
                .find(|r| r.field == field)
                .expect("just collected")
                .widget_count;
            indices.sort_unstable();
            let before = indices.len();
            indices.dedup();
            assert_eq!(before, indices.len(), "{field} listed one widget twice");
            assert_eq!(
                indices.len(),
                expected,
                "{field} claims {expected} widgets and {} were listed",
                indices.len()
            );
        }
    }

    /// **A field with no widget cannot be a row, and is counted.**
    ///
    /// Asserted against the arithmetic rather than a named fixture: whichever
    /// documents carry such a field, the count must equal the number of
    /// `/Fields` entries with an empty `widgets`, and no row may name one.
    #[test]
    fn a_field_with_no_widget_is_counted_and_never_listed() {
        for f in [
            "forms/demo-form.pdf",
            "forms/nested-form.pdf",
            "forms/unfillable-fields-form.pdf",
        ] {
            let path = engine_fixture(f);
            let doc = Document::load(&path).expect("the fixture loads");
            let session = EditSession::new(doc);
            let slots = session.page_slots().expect("a page tree");
            let view = session.view();
            let form = pdfcer_core::forms::parse_acroform(&view).expect("a form");
            let l = collect(&view, &slots, Some(&form));

            let widgetless: Vec<&str> = form
                .fields
                .iter()
                .filter(|fd| fd.widgets.is_empty())
                .map(|fd| fd.fully_qualified_name.as_str())
                .collect();
            assert_eq!(l.fields_without_widgets, widgetless.len(), "{f}");
            for name in widgetless {
                assert!(
                    !l.pages
                        .iter()
                        .any(|p| p.rows.iter().any(|r| r.field == name)),
                    "{f}: {name} has no widget and must not be listed"
                );
            }
        }
    }

    /// **★ A widget with no `/P` entry is still listed** — the defect no
    /// fixture in the corpus can catch.
    ///
    /// `/P` is Optional (§12.5.2 Table 164) and frequently absent, and
    /// `pdfcer-core` reads it without resolving through the graph, so a direct
    /// `/P` reads as absent too. The obvious implementation of *"which page is
    /// this widget on?"* — look up `Widget::page` — returns **nothing at all**
    /// on such a form.
    ///
    /// Every one of the eleven form fixtures writes `/P` on every widget, so a
    /// test opening a fixture cannot reach it; `crate::canvas::forms::boxes`
    /// carries the same test for the same reason, and the engine team hit the
    /// case when a deliberate sabotage of their own `/P` handling passed against
    /// their whole corpus.
    ///
    /// Here the form is built by hand with `page: None` on its widget, and the
    /// assertion is that the row is produced anyway — so every other assertion
    /// in this module is also an assertion that `/P` is not consulted.
    #[test]
    fn a_widget_with_no_p_entry_is_still_listed() {
        use pdfcer_core::forms::{Field, FieldFlags, FieldType, FieldValue, Widget};
        use pdfcer_core::vartext::Quadding;

        let widget_id = ObjId::new(7, 0);
        let page_id = ObjId::new(1, 0);

        let field = Field {
            id: ObjId::new(6, 0),
            fully_qualified_name: "Name".to_owned(),
            partial_name: None,
            alternate_name: None,
            mapping_name: None,
            rich_value: None,
            default_style: None,
            field_type: Some(FieldType::Text),
            button_kind: None,
            flags: FieldFlags(0),
            value: FieldValue::Absent,
            default_value: FieldValue::Absent,
            default_appearance: None,
            quadding: Quadding::Left,
            max_len: None,
            options: Vec::new(),
            top_index: 0,
            selected_indices: Vec::new(),
            widgets: vec![Widget {
                id: widget_id,
                rect: None,
                appearance_state: None,
                on_states: Vec::new(),
                // ★ `rotation` arrived with the engine's `rotate_widget` Pass on
                // 2026-08-30. `None` here means the file states none, which is what
                // every fixture in this shell wants: a widget with no `/MK /R`.
                rotation: None,
                has_off_appearance: false,
                // ★ THE POINT OF THIS FIXTURE.
                page: None,
                caption: None,
                // `Pass 146.0`'s three, in a test fixture: the file states no
                // border and no unusual flags. `None` is the honest value for a
                // synthetic widget — it means "this file says nothing", which is
                // exactly true of one built in a test.
                // ★ `background` arrived with the engine's field-shading Pass on
                // 2026-09-04, alongside `border`. `None` is the honest value for a
                // synthetic widget for the same reason the two below it are: it
                // means "this file states no /MK /BG", which is exactly true of one
                // built in a test.
                background: None,
                border: None,
                visibility: None,
                annot_flags: pdfcer_core::annot::AnnotFlags(0),
                has_normal_appearance: true,
                merged: false,
            }],
            merged: false,
            has_additional_actions: false,
            shares_parent_name: false,
            parent: None,
        };
        assert!(
            field.widgets[0].page.is_none(),
            "the fixture must omit /P, or this test proves nothing"
        );
        let form = AcroForm {
            fields: vec![field],
            ..empty_form()
        };

        // A page whose `/Annots` lists the widget, and the widget's own
        // dictionary — which carries no `/P` either.
        let mut page = Dict::new();
        page.insert(
            pdfcer_core::object::Name(b"Annots".to_vec()),
            Object::Array(vec![Object::Reference(widget_id)]),
        );
        let mut w = Dict::new();
        w.insert(
            pdfcer_core::object::Name(b"Subtype".to_vec()),
            Object::Name(pdfcer_core::object::Name(b"Widget".to_vec())),
        );
        w.insert(
            pdfcer_core::object::Name(b"Rect".to_vec()),
            Object::Array(vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(10),
                Object::Integer(10),
            ]),
        );
        let graph = FakeGraph(HashMap::from([
            (page_id, Object::Dict(page)),
            (widget_id, Object::Dict(w)),
        ]));
        let slot = PageSlot {
            id: page_id,
            parent: None,
            index_in_parent: 0,
            ancestors: Vec::new(),
            inherited: pdfcer_core::page_tree::InheritedRaw::default(),
        };

        let l = collect(&graph, std::slice::from_ref(&slot), Some(&form));
        assert_eq!(
            l.total_rows(),
            1,
            "a widget with no /P must still be listed from its page's /Annots"
        );
        assert_eq!(l.pages[0].rows[0].field, "Name");
        assert_eq!(l.pages[0].rows[0].position, 1);
        assert!(l.pages[0].unclaimed.is_empty());
        assert_eq!(l.pages[0].anonymous, 0);
    }

    /// **★ A widget no field claims is counted, and a non-widget annotation is
    /// counted separately.**
    ///
    /// Two different facts that a single "not listed" number would blur. An
    /// ★★ A document with WIDGETS and NO `/AcroForm` lists every one of
    /// them as unclaimed.
    ///
    /// The state pdfcer manufactures and could not display. `insert_pages`
    /// copies everything reachable from a page; `/Annots` reaches the widgets;
    /// `/AcroForm` is a **catalog** entry and is never in the copied set. So a
    /// form's pages inserted into a CAD drawing arrive as boxes that draw like
    /// fields, swallow every keystroke, and belong to nothing at all.
    ///
    /// Before 2026-08-19 this was not merely untested — it was
    /// **unrepresentable**: `collect` took `&AcroForm`, so the panel had to
    /// return before reaching it, and the one section that lists these widgets
    /// and offers to register them sat behind a guard the state cannot pass.
    ///
    /// Asserted at the model rather than through the panel because the model is
    /// where the `Option` now lives; the panel's half is covered by the driven
    /// check that inserts a form's pages and registers one of the orphans,
    /// which is what found this.
    #[test]
    fn a_document_with_widgets_and_no_acroform_lists_them_all() {
        let page_id = ObjId::new(1, 0);
        let a = ObjId::new(8, 0);
        let b = ObjId::new(9, 0);
        let mut page_dict = Dict::new();
        page_dict.insert(
            pdfcer_core::object::Name(b"Annots".to_vec()),
            Object::Array(vec![Object::Reference(a), Object::Reference(b)]),
        );
        let widget = || {
            let mut d = Dict::new();
            d.insert(
                pdfcer_core::object::Name(b"Subtype".to_vec()),
                Object::Name(pdfcer_core::object::Name(b"Widget".to_vec())),
            );
            Object::Dict(d)
        };
        let graph = FakeGraph(HashMap::from([
            (page_id, Object::Dict(page_dict)),
            (a, widget()),
            (b, widget()),
        ]));
        let slot = PageSlot {
            id: page_id,
            parent: None,
            index_in_parent: 0,
            ancestors: Vec::new(),
            inherited: pdfcer_core::page_tree::InheritedRaw::default(),
        };

        // `None` — the document has no `/AcroForm` at all.
        let l = collect(&graph, std::slice::from_ref(&slot), None);
        let page = &l.pages[0];
        assert!(
            page.rows.is_empty(),
            "no field exists, so no row can name one"
        );
        assert_eq!(
            page.unclaimed.len(),
            2,
            "every widget is unclaimed when there is no form to claim it"
        );
        assert_eq!(
            page.widgets_seen(),
            2,
            "and they are still counted as widgets the page lists"
        );
        assert_eq!(
            page.unclaimed[0].position, 1,
            "positions still number from the top of the tab sequence"
        );
        assert_eq!(page.unclaimed[1].position, 2);
    }

    /// unclaimed widget means the form's `/Fields` does not reach it — which is
    /// what an `inline_field_roots` entry looks like from this side. A
    /// non-widget annotation is a `/Link` or a note, which **is** in the tab
    /// sequence and is simply not a form field, so the row numbering below it
    /// is genuinely not the annotation numbering.
    #[test]
    fn an_unclaimed_widget_and_a_plain_annotation_are_counted_apart() {
        let page_id = ObjId::new(1, 0);
        let orphan = ObjId::new(8, 0);
        let link = ObjId::new(9, 0);

        let mut page = Dict::new();
        page.insert(
            pdfcer_core::object::Name(b"Annots".to_vec()),
            Object::Array(vec![Object::Reference(orphan), Object::Reference(link)]),
        );
        let named = |k: &[u8], v: &[u8]| {
            let mut d = Dict::new();
            d.insert(
                pdfcer_core::object::Name(k.to_vec()),
                Object::Name(pdfcer_core::object::Name(v.to_vec())),
            );
            Object::Dict(d)
        };
        let graph = FakeGraph(HashMap::from([
            (page_id, Object::Dict(page)),
            (orphan, named(b"Subtype", b"Widget")),
            (link, named(b"Subtype", b"Link")),
        ]));
        let slot = PageSlot {
            id: page_id,
            parent: None,
            index_in_parent: 0,
            ancestors: Vec::new(),
            inherited: pdfcer_core::page_tree::InheritedRaw::default(),
        };

        let l = collect(&graph, std::slice::from_ref(&slot), Some(&empty_form()));
        let p = &l.pages[0];
        assert!(p.rows.is_empty(), "no field claims either annotation");
        assert_eq!(
            p.unclaimed.len(),
            1,
            "the widget belongs to no listed field"
        );
        assert_eq!(p.other_annots, 1, "the link is not a widget");
        assert_eq!(p.anonymous, 0, "both were written as indirect objects");
    }

    /// **A widget written as a direct dictionary has no identity, and is
    /// counted as such rather than as unclaimed.**
    ///
    /// Table 164 requires an annotation to be an indirect object. One written
    /// inline has no `ObjId`, so it cannot be matched to a field even in
    /// principle — which is a different statement from "no field claims it",
    /// and merging the two would tell an operator their form was malformed when
    /// the *annotation* was.
    #[test]
    fn a_directly_written_widget_is_anonymous_not_unclaimed() {
        let page_id = ObjId::new(1, 0);
        let mut inline = Dict::new();
        inline.insert(
            pdfcer_core::object::Name(b"Subtype".to_vec()),
            Object::Name(pdfcer_core::object::Name(b"Widget".to_vec())),
        );
        let mut page = Dict::new();
        page.insert(
            pdfcer_core::object::Name(b"Annots".to_vec()),
            Object::Array(vec![Object::Dict(inline)]),
        );
        let graph = FakeGraph(HashMap::from([(page_id, Object::Dict(page))]));
        let slot = PageSlot {
            id: page_id,
            parent: None,
            index_in_parent: 0,
            ancestors: Vec::new(),
            inherited: pdfcer_core::page_tree::InheritedRaw::default(),
        };

        let l = collect(&graph, std::slice::from_ref(&slot), Some(&empty_form()));
        assert_eq!(l.pages[0].anonymous, 1);
        assert!(l.pages[0].unclaimed.is_empty());
        assert!(l.pages[0].rows.is_empty());
    }

    /// **Every listed page can be navigated to.**
    ///
    /// The page index is fed straight to
    /// [`crate::app::actions::Action::GoToPage`], so an out-of-range value
    /// would be a navigation to nowhere. It cannot happen — the index is the
    /// enumeration of `slots` — and it is pinned anyway, because this is the
    /// one number that leaves the view.
    #[test]
    fn every_page_index_is_inside_the_document() {
        let l = listing("forms/demo-form.pdf");
        assert!(!l.pages.is_empty());
        for (i, p) in l.pages.iter().enumerate() {
            assert_eq!(p.page_index, i, "the index must be the enumeration");
        }
    }
}
