//! # `panels` — the dock's panel bodies
//!
//! The panels of [`Panel::ALL`], each a **function the dock can call**. This module owns the
//! set, the dispatch, the little state the bodies share, and the two layout
//! rules that every one of them has to get right.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/mod.md`.

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::shell::menus::MenuHost;
use egui_shell::HandlerToken;

pub mod align;
pub mod attachments;
pub mod bookmarks;
pub mod comments;
pub mod dimension_groups;
/// **The document's own properties**, a panel of its own rather than a
/// permanently-drawn section of [`properties`]. Its header carries the
/// argument.
pub mod docprops;
pub mod fonts;
pub use pdfcer_gui_base::panelfooter as footer;
pub mod forms;
pub mod layers;
pub mod objects;
pub mod pages;
pub mod properties;
pub mod redact;
pub mod signatures;

pub use pdfcer_gui_base::panelid::Panel;

/// Draw `panel`. The one entry point a dock calls.
#[must_use]
pub fn show(
    panel: Panel,
    ui: &mut egui::Ui,
    doc: Option<&OpenDoc>,
    state: &mut PanelsState,
    host: Option<&MenuHost<'_>>,
    actions: &mut Vec<Action>,
) -> Vec<HandlerToken> {
    scroll_style(ui);
    let Some(doc) = doc else {
        // Nothing open: forget the operator's tree state. Doing this
        // here rather than in each body is what makes it unforgettable —
        // a panel that never draws while the shell is empty would never
        // get the chance. (The document's own caches need no equivalent:
        // they live on `OpenDoc` and were dropped with it.)
        state.forget_document();
        ui.label(crate::text::panels::panel_no_document());
        return Vec::new();
    };
    state.sync(doc);
    match panel {
        Panel::Attachments => attachments::body(ui, doc, state, actions),
        Panel::Bookmarks => bookmarks::body(ui, doc, state, actions),
        Panel::Layers => layers::body(ui, doc, state, actions),
        Panel::Signatures => signatures::body(ui, doc, state, actions),
        Panel::Fonts => fonts::body(ui, doc, state, actions),
        Panel::Objects => return objects::body(ui, doc, state, host, actions),
        Panel::Properties => properties::body(ui, doc, state, actions),
        Panel::DocumentProperties => docprops::body(ui, doc, state.docprops_mut(), actions),
        Panel::Forms => forms::body(ui, doc, state, actions),
        // The second panel with a menu, and the second `return` for the
        // same reason: it has tokens to hand back.
        Panel::Pages => return pages::body(ui, doc, state, host, actions),
        Panel::Comments => comments::body(ui, doc, state, actions),
        Panel::Redact => redact::body(ui, doc, state, actions),
        Panel::DimensionGroups => dimension_groups::body(ui, doc, state, actions),
        Panel::AlignDistribute => align::body(ui, doc, &mut state.align, actions),
    }
    Vec::new()
}

/// The little state the panel bodies own between frames.
#[derive(Default)]
pub struct PanelsState {
    /// The `(page index, edit epoch)` [`Self::tree`] describes, or `None`
    /// before anything has been drawn.
    ///
    /// The page is in the key because a paint-order index is a position on
    /// **one page**: keeping a focus or an expansion set across a page step
    /// would silently point them at a different object with the same number.
    /// The epoch is in it because an edit renumbers everything after the
    /// object it touched, which does the same thing without moving page.
    tree_key: Option<(usize, u64)>,
    /// The Align panel's settings. They answer to the operator, not the
    /// document, so [`Self::forget_document`] carries them.
    pub align: align::AlignUi,
    /// What the operator has opened and picked in the Objects tree.
    ///
    /// A struct rather than three loose fields so the grouping says what it
    /// is: the operator's own state, not a cache of the document. Everything
    /// derived from the document now lives on `OpenDoc`, and this is what was
    /// left when it went.
    tree: ObjectTreeUi,
    /// The Pages panel's picked sheets and its thumbnail cache.
    ///
    /// # Why this cache lives here and not on `OpenDoc`
    ///
    /// Every other derived cache moved to `crate::app::state::OpenDoc` at S4,
    /// and the argument for that move — *"the document's own lifetime bounds
    /// them and no identity key is needed at all"* — applies to thumbnails
    /// word for word. It is still right for this one to be here, for a reason
    /// that is about **borrowing** rather than about lifetime.
    ///
    /// A panel body is handed `&OpenDoc`, shared, which is the compile-time
    /// half of actions-not-mutations. Filling a cache on `OpenDoc` would
    /// therefore need interior mutability — which the two existing caches
    /// have (`RefCell`) and which this module's own header permits for
    /// *derived* data. But a thumbnail cache is not only filled: it is
    /// **evicted from, and stopped**, by a control the operator clicks, and
    /// `ThumbnailCache::force_on` is state that decides whether work happens
    /// at all. Putting a `RefCell` around that would put an operator
    /// instruction behind interior mutability, which is precisely the line
    /// this module's header draws for the Layers panel.
    ///
    /// `&mut PanelsState` is already threaded to every body, so the panel's
    /// own state needs no such device — and the forgetting is free, because
    /// [`Self::forget_document`] resets this struct whole.
    pages: pages::PagesUi,
    /// The Redact panel's half-typed search and its match mode.
    ///
    /// Here rather than on `OpenDoc` for the same reason [`Self::pages`] is:
    /// a panel body is handed `&OpenDoc`, shared, and a text field the operator
    /// types into is **their** state rather than a derived cache of the
    /// document's. It is also state that arms a verb — a query plus a mode
    /// decides what a marking click will cover — and this module's header draws
    /// exactly that line for the Layers checkbox: interior mutability is for
    /// derived data whose filling nothing can observe, never for an operator
    /// instruction.
    ///
    /// Reset with the document by [`Self::forget_document`], which resets this
    /// struct whole. That is not tidiness: a search term left over from a
    /// previous file is one an operator could run against a document it was
    /// never meant for, and on this feature a search authors marks over whatever
    /// it hits.
    redact: redact::RedactUi,
    /// **What the operator has typed into the Layers panel's search box.**
    ///
    /// Here rather than on the panel for [`Self::redact`]'s first reason: a
    /// `TextEdit` needs a `&mut String` that survives the frame, and a panel
    /// body is handed `&OpenDoc` — shared, deliberately, so that it cannot
    /// mutate. It is the operator's own typing rather than a derived cache,
    /// which is the line this module's header draws.
    ///
    /// Reset with the document by [`Self::forget_document`], and here that
    /// is a straightforward good rather than a safety property: a query left
    /// over from a previous file would open the next one showing a filtered
    /// layer list with no obvious cause. Unlike `redact`'s, this search
    /// authors nothing — the worst it can do is hide rows — so the reset is
    /// about not confusing the operator rather than about not marking the
    /// wrong document.
    layers_search: String,
    /// The **Document properties** panel's half-typed metadata.
    ///
    /// Here for [`Self::pages`]' reason and one of its own: a `TextEdit` needs
    /// a `&mut String` that survives the frame, and a panel body is handed
    /// `&OpenDoc`, shared. It is also the operator's own typing rather than a
    /// derived cache — this module's header draws exactly that line.
    ///
    /// Reset with the document by [`Self::forget_document`], and that matters
    /// more here than for a search term: a half-typed `/Author` carried into a
    /// second file would be written into **that** file's metadata by the next
    /// focus change, silently, in a field nobody looks at twice.
    ///
    /// Named after [`Panel::DocumentProperties`] and never `properties`: a
    /// field named after a panel that does not draw it is how the next reader
    /// looks in the wrong place, and this struct already holds a `geometry`, a
    /// `text_style` and a `text_object` that ARE the Properties panel's.
    docprops: docprops::InfoDrafts,
    /// The form-field rename draft, and which field it is for. See
    /// [`Self::field_rename_mut`] for why the key is stored beside it.
    field_rename: String,
    /// The fully-qualified name [`Self::field_rename`] was seeded from.
    field_rename_key: Option<String>,
    /// The two TYPED properties of the selected form field — its tooltip and
    /// its maximum length — and the `(name, epoch)` they were read at.
    ///
    /// Only two, and the omission is the design: every other property in
    /// `properties::fieldedit` is a checkbox that reads `field.flags` straight
    /// from the session each frame, so a press the engine refuses leaves the
    /// box where it was. A draft-backed boolean would show the operator's
    /// intent while the document silently disagreed with it.
    ///
    /// Here rather than on `OpenDoc` by this struct's own rule: a half-typed
    /// tooltip is the operator's state, not the document's.
    field_props: properties::fieldedit::FieldPropsDraft,
    /// The selected choice field's **option list** as it is being typed,
    /// and the `(name, epoch)` it was read at.
    ///
    /// Separate from [`Self::field_props`] rather than a field inside it,
    /// because it is a list of rows rather than a handful of scalars and
    /// because `properties::choiceopts` destructures it into three disjoint
    /// borrows in one frame. Same stamp rule, different shape.
    choice_opts: properties::choiceopts::ChoiceOptsDraft,
    /// The selected WIDGET's four typed numbers and its caption, and the
    /// `(name, widget index, epoch)` they were read at.
    ///
    /// Separate from [`Self::field_props`] rather than a field inside it,
    /// because the two have different stamps: a field draft is keyed on the
    /// name and this one has to be keyed on the name **and the placement**. One
    /// field can be drawn in three places with three different boxes, and a
    /// merged struct would need both keys and one reset rule for two lifetimes.
    widget_props: properties::widgetedit::WidgetPropsDraft,
    /// The Properties panel's **geometry** draft — the four typed numbers, and
    /// the `(page, object, epoch)` they were seeded from.
    ///
    /// Separate from `properties` above rather than a field inside it, because
    /// the two have different lifetimes and different reset conditions: the
    /// metadata drafts survive a selection change (they describe the document),
    /// and this one must not (it describes one object). Merging them would make
    /// one struct with two reset rules.
    geometry: properties::geometry::GeometryDraft,
    /// The selected text's face, size and colour, and the size being typed.
    ///
    /// Held here for a stronger reason than its neighbours: the read-back needs
    /// an extraction with provenance on — 392 ms on the operator's benchmark
    /// sheet — so a section that re-read it every frame would take the whole
    /// application to under three frames a second on exactly the drawings this
    /// program is for. The struct carries a `(page, run, epoch)` stamp and
    /// re-reads only when it moves.
    text_style: properties::text::TextStyleDraft,
    /// The **clicked text object's** run range and colour — O89's object
    /// route.
    ///
    /// Held here for exactly [`Self::text_style`]'s reason and at exactly its
    /// cost: `properties::textobject` reads the object's runs and their fills
    /// out of one extraction with provenance capture on, which is 392 ms on the
    /// operator's benchmark sheet, and a section that re-read it every frame
    /// would take the application to under three frames a second on the
    /// drawings this program is for. The struct carries a
    /// `(page, object, epoch)` stamp and re-reads only when it moves.
    ///
    /// Separate from [`Self::text_style`] rather than a second case inside
    /// it, and the reason is the stamp: that one is keyed on a **run** and this
    /// on an **object**, and the two selections are different index spaces that
    /// can both be absent, either be present, and — since the Text tool can be
    /// armed in Edit — both be present at once. One struct with two stamps is
    /// one struct with two reset rules.
    text_object: properties::textobject::TextObjectDraft,
    /// **The character an edit was refused for, the face the operator
    /// picked to answer it, and the revision both were live for** —
    /// `OPERATOR_REQUESTS.md` O141.
    ///
    /// Held here for [`Self::text_style`]'s reason at its exact cost: filling
    /// the offer's face list is one extraction with provenance capture plus one
    /// `preview_font_resources`, which is 392 ms for the first alone on the
    /// operator's benchmark sheet, so a block that re-read it every frame would
    /// hold the program under three frames a second for as long as the refusal
    /// was on screen. The struct carries a `(page, run, epoch)` stamp.
    ///
    /// It is also more than a cache, which is why it could not live behind
    /// interior mutability on `OpenDoc` — this module's header draws that line
    /// for the Layers checkbox and it binds here. `RefusedCharUi::taken` records
    /// that **the operator pressed a row in this block**, which is an operator
    /// instruction and the only thing that distinguishes the face swap they
    /// asked for from any other edit that would retire the report.
    ///
    /// Reset with the document by [`Self::forget_document`], for
    /// [`Self::bookmarks`]' reason: a `(page, run)` pair names different text in
    /// a different file, so an offer carried across would restyle a run nobody
    /// asked about.
    refused_char: properties::refusedchar::RefusedCharUi,
    /// The memoised answer to *what would go with deleting the selected
    /// annotation?* — `EditSession::annotation_deletion_preview`.
    ///
    /// Held here for [`Self::text_style`]'s reason at a smaller magnitude, and
    /// the shape of the argument is what matters rather than the milliseconds.
    /// The query is `&self` and side-effect-free, but it walks the page's whole
    /// `/Annots` array looking for `/IRT` referrers — O(annotations) per call —
    /// and the old shell paid that per *row* and gated it on hover for exactly
    /// that reason. This section has one subject rather than a list, so the
    /// worst case is one call per frame; the `(annotation id, edit epoch)` stamp
    /// takes it down to none. See `properties::annotdelete`'s header.
    ///
    /// Not a document cache smuggled into the operator's own state. What is
    /// stored is the finished **sentence**, which is drawing state, and it is
    /// reset with the document by [`Self::forget_document`] like everything else
    /// here — an object id carried into a second file names a different object
    /// there, which for a cached collateral warning would mean describing one
    /// document's reply thread while the operator looks at another's.
    annot_delete: properties::annotdelete::DeletionPreview,
    /// The Bookmarks panel's half-typed title and its chosen parent.
    ///
    /// Here for [`Self::pages`]' reason: a panel body is handed `&OpenDoc`,
    /// shared, and a text field the operator types into is **their** state.
    ///
    /// Reset with the document by [`Self::forget_document`], and the parent
    /// is why that matters more than for a search term: an `ObjId` carried into
    /// a second file names a different object there, and a bookmark would be
    /// filed under whatever happens to hold that number.
    bookmarks: bookmarks::BookmarksUi,
    /// The Attachments panel's half-typed description.
    ///
    /// Here for [`Self::properties`]' reason, and the hazard is the same one
    /// stated more sharply: `attach_file` takes a description **at attach
    /// time** and no verb edits one afterwards, so a draft carried into a
    /// second document would be written permanently into that file's `/Desc`
    /// by the next attach, describing one operator's spreadsheet with another
    /// document's note. [`Self::forget_document`] resets this struct whole,
    /// which is what makes that unrepresentable rather than merely avoided.
    attachments: attachments::AttachmentsUi,
    /// The dimension-groups panel's selected row, its half-typed names and its
    /// pending *Set scale…* request.
    ///
    /// `pub(crate)` rather than private, and it is the only field here that is:
    /// `crate::app::PdfcerApp::docks` drains
    /// [`dimension_groups::DimensionGroupsUi::take_scale_request`] after the
    /// dock body closes. A panel cannot open a window from inside its own body
    /// — it is handed `&OpenDoc` and `&mut PanelsState` and nothing else — so
    /// the request has to leave through the state it is allowed to touch.
    ///
    /// Reset with the document by [`Self::forget_document`], for the reason
    /// [`Self::bookmarks`] gives and one of its own: a `GroupId` names a
    /// different group in a different file, so a selection carried across would
    /// point the appearance controls at somebody else's group.
    pub(crate) dimension_groups: dimension_groups::DimensionGroupsUi,
    /// **The comment note being typed, and the annotation it belongs to.**
    ///
    /// Here for [`Self::pages`]' reason — a `TextEdit` needs a `&mut String`
    /// that outlives the frame and a panel body is handed `&OpenDoc`, shared —
    /// and the Comments panel is the surface that most recently claimed to have
    /// no such state at all. It has one now, and
    /// [`comments::note::NoteDraft`]'s header carries the argument for why it
    /// is a draft rather than a live binding.
    ///
    /// Reset with the document by [`Self::forget_document`], and here that
    /// matters as much as it does for a half-typed `/Author`: an `ObjId` names
    /// a different annotation in a different file, so a draft carried across
    /// would offer to write one document's comment onto another document's
    /// shape.
    comments: comments::note::CommentsUi,
}

/// What the operator has opened and picked in the Objects tree.
#[derive(Default)]
pub struct ObjectTreeUi {
    /// Which object rows are expanded, by paint-order index.
    pub(crate) objects_expanded: std::collections::BTreeSet<usize>,
    /// Which part rows are expanded, by `(object, part)`.
    pub(crate) parts_expanded: std::collections::BTreeSet<(usize, usize)>,
    /// One paint-order index, **retired**: nothing in production writes or
    /// reads it.
    ///
    /// **It is not a selection**, and the distinction is load-bearing enough
    /// to have its own name. A selection is document-scoped, multi-valued,
    /// survives a page change, drives the contextual Format tab, and is what
    /// an edit acts on. This is one `usize` naming a row, page-scoped and
    /// cleared by [`PanelsState::sync`] on any page or revision change.
    ///
    /// # A panel-local focus is DELETED, never grown
    ///
    /// The distinction is what stops a shell acquiring two selections.
    /// [`properties`] reads `OpenDoc::selection`, the Objects panel's row
    /// highlight reads the same, and a row click raises `Action::SelectObject`
    /// rather than writing here. Growing this instead — making it a `Vec`,
    /// letting it survive a page change, letting an edit act on it — produces
    /// a second, weaker selection that the canvas and the panel have to keep
    /// in step, and the drift between them is invisible until an edit acts on
    /// the wrong object.
    ///
    /// # Why the field is still declared
    ///
    /// Because three tests in `app::files` and `app::lifecycle` use it to pin a
    /// property that is still real and still worth pinning: **closing or
    /// re-opening a document must forget the paint-order indices the panels
    /// hold**, because an index names a position in a document that is no
    /// longer open. Those tests reach for the one index-bearing field they can
    /// set from outside, and deleting it would delete the assertion with it.
    ///
    focus: Option<usize>,
}

impl ObjectTreeUi {
    /// The retired paint-order focus index. See the field for why it is still
    /// declared.
    #[must_use]
    pub fn focus(&self) -> Option<usize> {
        self.focus
    }

    /// Set the retired focus index, **toggling**: setting the index already
    /// there clears it, so a row click would be its own undo.
    pub fn set_focus(&mut self, index: usize) {
        self.focus = if self.focus == Some(index) {
            None
        } else {
            Some(index)
        };
    }

    /// Toggle an object row's expansion.
    pub(crate) fn toggle_object(&mut self, index: usize) {
        if !self.objects_expanded.remove(&index) {
            self.objects_expanded.insert(index);
        }
    }

    /// Toggle a part row's expansion.
    pub(crate) fn toggle_part(&mut self, object: usize, part: usize) {
        if !self.parts_expanded.remove(&(object, part)) {
            self.parts_expanded.insert((object, part));
        }
    }
}

impl PanelsState {
    /// Drop anything that no longer describes `doc`'s current page.
    fn sync(&mut self, doc: &OpenDoc) {
        let key = (doc.view.page_index, doc.edit_epoch);
        if self.tree_key != Some(key) {
            self.tree_key = Some(key);
            self.tree = ObjectTreeUi::default();
        }
    }

    /// Forget everything about whatever document was open.
    pub fn forget_document(&mut self) {
        // THE TWO PAGE-PREVIEW PREFERENCES ARE CARRIED ACROSS THE RESET,
        // and this is the whole of O187 working or not working.
        //
        // They are seeded once, in `PdfcerApp::new`, from `preferences.txt`.
        // A `*self = Self::default()` over them runs the moment a document
        // opens — which is every launch, because the shell is started on a
        // file. The operator clears the tick, the file is written correctly,
        // the next launch reads it correctly, and the answer is thrown away
        // before the panel draws.
        //
        // ⚠ Only driving the binary can see this. Every unit test of the
        // feature constructs a `ThumbnailCache` directly, so the whole suite
        // and every gate stay green: the defect lives in the frame BETWEEN
        // the seed and the first draw.
        //
        // Why a carry rather than a reseed at the call site: this
        // function is also called from `show` EVERY FRAME while
        // nothing is open, and that call site has no `Prefs` to reseed from.
        // A carry holds for every caller, present and future, and it is
        // idempotent — `set_budget` returns early on an unchanged value.
        //
        // ⇒ The rule this states, and the one a future field should be
        // tested against: `*self = Self::default()` forgets DOCUMENT state.
        // A field on this struct that answers to the operator rather than to
        // the document does not belong to the reset, and must be carried
        // here explicitly.
        let previews_on = self.pages.cache.previews_on();
        let preview_budget = self.pages.cache.budget();
        let align = self.align;
        *self = Self::default();
        self.align = align;
        self.pages.cache.force_on(previews_on);
        self.pages.cache.set_budget(preview_budget);
        properties::refusedchar::forget_document();
    }

    /// The operator's state in the Objects tree — what is expanded, and what
    /// is focused.
    pub fn tree_mut(&mut self) -> &mut ObjectTreeUi {
        &mut self.tree
    }

    /// Which object the Properties panel is describing.
    #[must_use]
    pub fn focus(&self) -> Option<usize> {
        self.tree.focus()
    }

    /// Point the Properties panel at an object. See
    /// [`ObjectTreeUi::set_focus`].
    pub fn set_focus(&mut self, index: usize) {
        self.tree.set_focus(index);
    }

    /// The Pages panel's own state — its picked sheets and its thumbnails.
    pub fn pages_mut(&mut self) -> &mut pages::PagesUi {
        &mut self.pages
    }

    /// The Redact panel's own state — the search query and the match mode.
    pub fn layers_search_mut(&mut self) -> &mut String {
        &mut self.layers_search
    }

    pub fn redact_mut(&mut self) -> &mut redact::RedactUi {
        &mut self.redact
    }

    /// The **Document properties** panel's metadata drafts.
    pub fn docprops_mut(&mut self) -> &mut docprops::InfoDrafts {
        &mut self.docprops
    }

    /// **The Comments panel's note draft.**
    ///
    /// Same shape as [`Self::pages_mut`], [`Self::redact_mut`] and
    /// [`Self::properties_mut`]: the body is handed `&mut PanelsState` and
    /// reaches its own state through an accessor, so the field stays private
    /// and no other panel can write it.
    pub fn comments_mut(&mut self) -> &mut comments::note::CommentsUi {
        &mut self.comments
    }

    /// **The rename draft for the selected form field**, re-seeded whenever the
    /// selection moves.
    pub fn field_rename_mut(&mut self, for_field: &str) -> &mut String {
        if self.field_rename_key.as_deref() != Some(for_field) {
            self.field_rename_key = Some(for_field.to_owned());
            self.field_rename = for_field.rsplit('.').next().unwrap_or(for_field).to_owned();
        }
        &mut self.field_rename
    }

    /// The Properties panel's geometry draft.
    /// The selected text's style draft, for `properties::text`.
    pub fn text_style_mut(&mut self) -> &mut properties::text::TextStyleDraft {
        &mut self.text_style
    }

    /// The clicked text object's draft, for `properties::textobject`.
    pub fn text_object_mut(&mut self) -> &mut properties::textobject::TextObjectDraft {
        &mut self.text_object
    }

    /// The refused-character offer's state, for `properties::refusedchar`.
    pub fn refused_char_mut(&mut self) -> &mut properties::refusedchar::RefusedCharUi {
        &mut self.refused_char
    }

    /// The selected annotation's memoised deletion collateral, for
    /// `properties::annotdelete`.
    pub fn annot_delete_mut(&mut self) -> &mut properties::annotdelete::DeletionPreview {
        &mut self.annot_delete
    }

    /// The selected form field's typed-property draft.
    ///
    /// No re-seed argument, like [`Self::text_style_mut`] and unlike
    /// [`Self::field_rename_mut`]: the draft owns its own `(name, epoch)` stamp
    /// and decides for itself when what it holds is stale, which is right here
    /// because the staleness condition includes the edit epoch and a caller
    /// would have to be handed that as well.
    pub fn field_props_mut(&mut self) -> &mut properties::fieldedit::FieldPropsDraft {
        &mut self.field_props
    }

    /// The selected choice field's option-list draft. See
    /// [`Self::field_props_mut`]; this one owns the same `(name, epoch)` stamp.
    pub fn choice_opts_mut(&mut self) -> &mut properties::choiceopts::ChoiceOptsDraft {
        &mut self.choice_opts
    }

    /// The selected widget's typed-property draft. See
    /// [`Self::field_props_mut`]; this one's stamp carries the placement too.
    pub fn widget_props_mut(&mut self) -> &mut properties::widgetedit::WidgetPropsDraft {
        &mut self.widget_props
    }

    pub fn geometry_mut(&mut self) -> &mut properties::geometry::GeometryDraft {
        &mut self.geometry
    }

    /// The Bookmarks panel's authoring state.
    pub fn bookmarks_mut(&mut self) -> &mut bookmarks::BookmarksUi {
        &mut self.bookmarks
    }

    /// The Attachments panel's authoring state — the optional description.
    pub fn attachments_mut(&mut self) -> &mut attachments::AttachmentsUi {
        &mut self.attachments
    }

    /// **The pages the operator has picked in the Pages panel.**
    pub fn selected_pages(&self) -> &std::collections::BTreeSet<usize> {
        self.pages.selection.pages()
    }
}

/// Apply this project's scroll-bar style to `ui`.
pub fn scroll_style(ui: &mut egui::Ui) {
    let mut scroll = egui::style::ScrollStyle::solid();
    scroll.foreground_color = true;
    scroll.bar_width = 10.0;
    ui.style_mut().spacing.scroll = scroll;
}

/// The width a scrolling container must declare so its rows are not
/// squeezed.
#[must_use]
pub fn content_width(row_widths: impl IntoIterator<Item = f32>, viewport: f32) -> f32 {
    row_widths
        .into_iter()
        .filter(|w| w.is_finite())
        .fold(viewport, f32::max)
}

/// The character this crate ends a shortened row with.
pub const ELLIPSIS: char = '\u{2026}';

/// **Shorten `label` until it fits `available`, or say that it already does** —
/// `OPERATOR_REQUESTS.md` **O123**: *"rows that ellipsise with a tooltip
/// instead of hard-clipping mid-character."*
#[must_use]
pub fn elide_to_width(
    label: &str,
    available: f32,
    measure: impl Fn(&str) -> f32,
) -> Option<String> {
    if !available.is_finite() || available <= 0.0 {
        // No width to fit into. Nothing sensible to shorten to, and returning
        // `None` here is deliberate: the caller draws the whole label, `egui`
        // clips it, and the frame in which a pane has no width is not one worth
        // making a layout decision inside.
        return None;
    }
    if measure(label) <= available {
        return None;
    }
    let chars: Vec<char> = label.chars().collect();
    // `lo` always fits, `hi` never does. `lo` starts at zero because the bare
    // ellipsis is the floor of what this function will return.
    let (mut lo, mut hi) = (0usize, chars.len());
    while lo + 1 < hi {
        let mid = lo + (hi - lo) / 2;
        let mut candidate: String = chars[..mid].iter().collect();
        candidate.push(ELLIPSIS);
        if measure(&candidate) <= available {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let mut out: String = chars[..lo].iter().collect();
    out.push(ELLIPSIS);
    Some(out)
}

/// Measure the intrinsic width of a row's text, in points.
#[must_use]
pub fn text_width(ui: &egui::Ui, text: &str) -> f32 {
    let font_id = egui::TextStyle::Body.resolve(ui.style());
    ui.painter()
        .layout_no_wrap(text.to_owned(), font_id, egui::Color32::PLACEHOLDER)
        .rect
        .width()
}

mod tests;
