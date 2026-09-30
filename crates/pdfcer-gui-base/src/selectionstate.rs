//! # `selectionstate` — The canvas selection: the chosen objects, the rung they were chosen at, their cached outlines, and the selected annotation.

use std::collections::BTreeSet;

use egui::Rect;

use crate::annotselection::AnnotSelection;
use crate::canvastarget::{CanvasTargetProvider, TargetId};
use crate::selectionidentity::{ClickHit, EscapeOutcome, Selection, SelectionLevel};

/// The whole of the canvas's selection state.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectionState {
    /// The selected entries, in `(page, object, subpath, node)` order.
    entries: Vec<Selection>,
    /// The same entries in the order they joined the selection, which
    /// `entries` cannot hold because it is kept sorted. Read by Align and
    /// Distribute's *first selected* / *last selected*. Kept in step by
    /// [`Self::normalise`]; read through [`Self::in_selection_order`].
    order: Vec<Selection>,
    /// The rung the operator has entered.
    level: SelectionLevel,
    /// Canvas-space outline rects for the entries on the resolved page, in
    /// the order they should be painted.
    outlines: Vec<(Selection, Rect)>,
    /// The `(page, edit epoch)` [`Self::outlines`] describes, or `None`
    /// before the first resolve.
    resolved_for: Option<(usize, u64)>,
    /// The selected **annotation**, if one is selected instead of content.
    ///
    /// # Why it is a field here rather than a second selection elsewhere
    ///
    /// Because the two are **mutually exclusive**, and that has to be enforced
    /// somewhere rather than remembered everywhere. Putting it on
    /// `OpenDoc` beside this state would make "what is selected?" a question
    /// with two answers that could both be yes — which is exactly the *"second
    /// selection"* `panels::ObjectTreeUi::focus`' docs refuse, arriving through
    /// a field instead of through a type.
    ///
    /// Here, [`Self::select_annot`] and the content paths are the only writers
    /// and each clears the other. One canvas, one selection.
    ///
    /// # Why it needs a `resolved_for` twin
    ///
    /// An annotation's outline is its `/Rect`, four numbers in a dictionary,
    /// and reading it costs one `/Annots` walk with no decomposition. **That
    /// makes it cheap to read; it does not make it safe to carry.** An outline
    /// cached at click time goes stale the moment an edit changes the `/Rect`,
    /// and the painter goes on stroking the box the mark has left: a quarter
    /// turn with the rotate handle takes the `/Rect` from 473.7 × 249.6 to
    /// 256.6 × 477.4 while the outline stays at the first, coming back to its
    /// mark only when the operator clicks away and clicks the shape again.
    ///
    /// ⇒ [`Self::resolve_annot`] is the twin, keyed on `(page, epoch)` exactly
    /// as [`Self::resolve`] is. Being cheap is what makes re-running it the fix
    /// rather than invalidating more aggressively.
    ///
    /// *"It is cheap to read"* and *"it does not need re-reading"* are
    /// different claims, and only the first is true here. See `annot`'s
    /// header table for the four ways content and annotation selections
    /// differ.
    annot: Option<AnnotSelection>,
    /// The `(page, edit epoch)` [`Self::annot`]'s geometry was last re-read
    /// for, or `None` before the first resolve.
    ///
    /// Separate from [`Self::resolved_for`] because the two are refreshed by
    /// different work — one needs a decomposition and the other needs an
    /// `/Annots` walk — and a shared key would make an undecodable page stop
    /// refreshing the annotation outline as well.
    annot_resolved_for: Option<(usize, u64)>,
}
impl SelectionState {
    /// The selected entries, in document order.
    #[must_use]
    pub fn entries(&self) -> &[Selection] {
        &self.entries
    }

    /// Whether anything is selected — **content or annotation**.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.annot.is_none()
    }

    /// The selected annotation, if the selection is one.
    #[must_use]
    pub const fn annot(&self) -> Option<&AnnotSelection> {
        self.annot.as_ref()
    }

    /// The selected entries in the order the operator added them: a
    /// Shift-click appends, a marquee appends its hits in document order.
    pub fn in_selection_order(&self) -> Vec<Selection> {
        let held: BTreeSet<Selection> = self.entries.iter().copied().collect();
        let mut out: Vec<Selection> = self
            .order
            .iter()
            .filter(|e| held.contains(e))
            .copied()
            .collect();
        let listed: BTreeSet<Selection> = out.iter().copied().collect();
        out.extend(self.entries.iter().filter(|e| !listed.contains(e)).copied());
        out
    }

    /// Drop from `order` what left `entries`; append what joined it.
    fn sync_order(&mut self) {
        let held: BTreeSet<Selection> = self.entries.iter().copied().collect();
        let mut seen = BTreeSet::new();
        self.order.retain(|e| held.contains(e) && seen.insert(*e));
        for e in &self.entries {
            if seen.insert(*e) {
                self.order.push(*e);
            }
        }
    }

    /// Select an annotation, **replacing** whatever was selected.
    pub fn select_annot(&mut self, selection: AnnotSelection) {
        self.entries.clear();
        self.order.clear();
        self.outlines.clear();
        self.resolved_for = None;
        self.annot_resolved_for = None;
        self.level = SelectionLevel::Object;
        self.annot = Some(selection);
    }

    /// Drop an annotation selection, reporting whether there was one.
    pub fn clear_annot(&mut self) -> bool {
        self.annot.take().is_some()
    }

    /// **Drop the selection entirely**, reporting whether there was one.
    ///
    /// Back to the ground state — no entries, no outlines, the ladder at
    /// [`SelectionLevel::Object`] — which is what makes this different from
    /// [`Self::escape`]: escape *ascends one rung* and is the operator walking
    /// back up a structure they entered, while this is the selection ceasing to
    /// exist. Resetting the rung matters as much as emptying the list: a
    /// `Node`-rung state with nothing in it would put the next click's first
    /// selection straight into a rung the operator never entered.
    ///
    /// `resolved_for` is cleared too, so the empty state is not mistaken for
    /// one already resolved against a page and an epoch it no longer describes.
    ///
    /// The one caller is `PdfcerApp::on_mode_capabilities_changed`, entering a
    /// mode that cannot select page content — see there for why a selection is
    /// not "work" for the purposes of rule 1. It is deliberately **not** wired
    /// to any gesture: a click on empty paper narrows the selection through
    /// [`Self::click`]'s own rules, and a mis-aimed right-click is documented
    /// in `pdfcer_gui::canvas::menus::select_under_right_click` as something that
    /// must *not* destroy a set the operator spent five clicks building.
    ///
    /// # It clears CONTENT only, and that is deliberate
    ///
    /// An annotation selection is governed by a different capability —
    /// `author_markup`, which **Review grants and Read does not**, where
    /// content selection needs `edit_content`, which only Edit grants. A mode
    /// change that lost one may keep the other, and Review is exactly that
    /// case: entering it from Edit must drop a selected path and keep a
    /// selected stamp.
    ///
    /// So the caller clears each half against its own predicate, and this one
    /// does not reach for [`Self::clear_annot`]. Folding them together here
    /// would be the "one on/off" the per-capability gate was built to avoid,
    /// reintroduced at the one place it is least visible.
    pub fn clear(&mut self) -> bool {
        if self.entries.is_empty() {
            return false;
        }
        self.entries.clear();
        self.order.clear();
        self.outlines.clear();
        self.level = SelectionLevel::Object;
        self.resolved_for = None;
        self.annot_resolved_for = None;
        true
    }

    /// How many entries are selected — the `sel=` the diagnostic trace
    /// reports, and the number `ui-verify` reads to tell a click that landed
    /// from one that did not.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// The rung the operator has entered.
    #[must_use]
    pub fn level(&self) -> SelectionLevel {
        self.level
    }

    /// The object the operator is inside, if any.
    #[must_use]
    pub fn entered_object(&self) -> Option<Selection> {
        (self.level != SelectionLevel::Object)
            .then(|| self.entries.first().copied())
            .flatten()
    }

    /// The canvas-space outlines to draw, with the entry each came from.
    #[must_use]
    pub fn outlines(&self) -> &[(Selection, Rect)] {
        &self.outlines
    }

    /// The union of the current outlines, in canvas space — the box the
    /// resize grips are placed around.
    #[must_use]
    pub fn outline_union(&self) -> Option<Rect> {
        self.outlines
            .iter()
            .map(|(_, r)| *r)
            .reduce(|acc, r| acc.union(r))
    }

    /// The **page** paint-order indices selected on `page`, ascending — the
    /// operand list for a batched edit.
    #[must_use]
    pub fn object_indices_on(&self, page: usize) -> Vec<usize> {
        self.entries
            .iter()
            .filter(|e| e.page == page)
            .filter_map(|e| e.object.page_object_index())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// The indices into `PageObjects::leaves` selected on `page`, ascending
    /// and unique — the half [`Self::object_indices_on`] drops.
    #[must_use]
    pub fn leaf_indices_on(&self, page: usize) -> Vec<usize> {
        self.entries
            .iter()
            .filter(|e| e.page == page)
            .filter_map(|e| e.object.leaf_index())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Every target selected on `page`, both lists, ascending and unique.
    #[must_use]
    pub fn targets_on(&self, page: usize) -> Vec<TargetId> {
        self.entries
            .iter()
            .filter(|e| e.page == page)
            .map(|e| e.object)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// The **Object-rung** indices on `page` — empty at the Part and Node
    /// rungs.
    #[must_use]
    pub fn deletable_objects_on(&self, page: usize) -> Vec<usize> {
        if self.level != SelectionLevel::Object {
            return Vec::new();
        }
        self.object_indices_on(page)
    }

    /// Apply a **completed click** — never a press.
    pub fn click(&mut self, page: usize, hit: ClickHit, shift: bool, double: bool) {
        // A content click drops an annotation selection — HERE, in the type
        // that owns the exclusion, not at the call site.
        //
        // `canvas::interact` also clears it when a click misses every
        // annotation, and that is a different case: there, the click may go on
        // to mean *text* and never reach this function at all. Relying on that
        // one alone would leave the invariant owned by a caller — and an
        // invariant a caller maintains is one the next caller will not.
        //
        // Unconditional, including for a shift-extend: extending a content
        // selection while a stamp is selected still means the stamp is no
        // longer what is selected.
        self.annot = None;
        if double {
            self.descend(page, hit);
            return;
        }
        match self.level {
            SelectionLevel::Object => self.click_at_object_rung(page, hit, shift),
            SelectionLevel::Part | SelectionLevel::Node => self.click_inside(page, hit, shift),
        }
    }

    /// **The Node tool's click** — direct selection, with no descent ritual.
    pub fn click_direct(&mut self, page: usize, hit: ClickHit, shift: bool) {
        // A content click drops an annotation selection, here, in the type that
        // owns the exclusion — exactly as `click` does and for the reason its
        // comment gives: an invariant a caller maintains is one the next caller
        // will not.
        self.annot = None;

        let Some(object) = hit.object else {
            // Shift over empty paper clears too, and that is deliberate.
            // Shift means "add to what I have", and there is nothing there to
            // add; preserving the selection would make an aimless Shift-click a
            // no-op the operator cannot distinguish from a missed anchor.
            self.entries.clear();
            self.order.clear();
            self.level = SelectionLevel::Object;
            return;
        };

        let entry = Selection {
            page,
            object,
            subpath: hit.part,
            node: hit.node,
        };

        // Shift only ever *extends within the same object*. Extending across
        // objects would build an operand list `move_nodes` cannot accept — it
        // addresses anchors within one object — and the refusal would arrive
        // after the operator had watched an outline slide.
        let same_object = self
            .entries
            .first()
            .is_some_and(|e| e.object == object && e.page == page);

        if shift && same_object && hit.node.is_some() {
            if let Some(at) = self.entries.iter().position(|e| *e == entry) {
                self.entries.remove(at);
                // Never leave the set empty at the Node rung: an empty set with
                // `level == Node` is the inconsistent state `normalise` exists
                // to prevent, and it would make the next plain click ambiguous.
                if self.entries.is_empty() {
                    self.entries.push(Selection {
                        page,
                        object,
                        subpath: hit.part,
                        node: None,
                    });
                    self.level = SelectionLevel::Part;
                }
            } else {
                self.entries.push(entry);
                self.level = SelectionLevel::Node;
            }
            self.normalise();
            return;
        }

        self.entries = vec![entry];
        self.level = if hit.node.is_some() {
            SelectionLevel::Node
        } else if hit.part.is_some() {
            // The Part rung, NOT the Object rung, and this single line is most
            // of the fix. It is what makes the anchors appear on the very first
            // click — `painting::draw_anchors` draws the entered subpath's
            // anchors from the Part rung up, so entering it *is* showing them.
            SelectionLevel::Part
        } else {
            SelectionLevel::Object
        };
        self.normalise();
    }

    /// **Take a band's hits OUT of the selection** — `OPERATOR_REQUESTS.md`
    /// O104.
    pub fn marquee_remove(&mut self, page: usize, hits: &[TargetId]) {
        if hits.is_empty() {
            return;
        }
        let doomed: Vec<Selection> = hits
            .iter()
            .map(|&object| Selection::object(page, object))
            .collect();
        self.entries.retain(|e| !doomed.contains(e));
        self.normalise();
    }

    /// Replace or extend the selection with a marquee's hits.
    pub fn marquee(&mut self, page: usize, hits: &[TargetId], shift: bool) {
        let found: Vec<Selection> = hits
            .iter()
            .map(|&object| Selection::object(page, object))
            .collect();
        if shift {
            self.entries.extend(found);
        } else {
            self.order.clear();
            self.entries = found;
        }
        self.level = SelectionLevel::Object;
        self.normalise();
    }

    /// **Select one object outright**, because the program just put it
    /// there.
    pub fn select_placed(&mut self, page: usize, object: TargetId) {
        self.select_only(page, object, "placed");
    }

    /// **Select exactly one object, from somewhere that is not the canvas.**
    pub fn select_only(&mut self, page: usize, object: TargetId, why: &'static str) {
        self.entries = vec![Selection::object(page, object)];
        self.level = SelectionLevel::Object;
        self.normalise();
        crate::diag::trace(move || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            // The list is named as well as the index. `object=7` was
            // unambiguous while a page had one index space; it has two now,
            // and a trace that cannot tell `objects[7]` from `leaves[7]` is a
            // trace that cannot be read back — which is how a wrong aim goes
            // six days without failing.
            let list = if object.is_leaf() { "leaf" } else { "object" };
            format!(
                "selection-set page={page} {list}={} via={why}",
                object.raw()
            )
        });
    }

    /// **Select ONE PART of one object, and stand at the Part rung** —
    /// [`Self::select_only`]'s sibling, one rung down.
    pub fn select_part(&mut self, page: usize, object: TargetId, part: usize, why: &'static str) {
        self.entries = vec![Selection {
            page,
            object,
            subpath: Some(part),
            node: None,
        }];
        self.level = SelectionLevel::Part;
        self.normalise();
        crate::diag::trace(move || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            // Names the list as well as the index, for `select_only`'s
            // reason one function up: a page has two index spaces, and a trace
            // that cannot tell `objects[7]` from `leaves[7]` cannot be read
            // back.
            let list = if object.is_leaf() { "leaf" } else { "object" };
            format!(
                "selection-set page={page} {list}={} part={part} level=part held=1 via={why}",
                object.raw()
            )
        });
    }

    /// **Select several chunks of one object outright** — the plural twin of
    /// [`Self::select_part`].
    pub fn select_parts(
        &mut self,
        page: usize,
        object: TargetId,
        parts: &[usize],
        why: &'static str,
    ) {
        if parts.is_empty() {
            self.clear();
            crate::diag::trace(move || {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("selection-set page={page} part=none level=object held=0 via={why}")
            });
            return;
        }
        self.entries = parts
            .iter()
            .map(|&part| Selection {
                page,
                object,
                subpath: Some(part),
                node: None,
            })
            .collect();
        self.level = SelectionLevel::Part;
        self.normalise();
        let held = self.entries.len();
        let first = self.entries.first().and_then(|e| e.subpath).unwrap_or(0);
        crate::diag::trace(move || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            // `part=` still names the first entry and `held=` says how many
            // there are, exactly as the status bar's rung line spells the same
            // pair. A check that read `part=` alone off a set of four would
            // report a working band as a broken one.
            let list = if object.is_leaf() { "leaf" } else { "object" };
            format!(
                "selection-set page={page} {list}={} part={first} level=part held={held} via={why}",
                object.raw()
            )
        });
    }

    /// Every selected **anchor** on one object of one page, object-scoped,
    /// ascending and unique.
    #[must_use]
    pub fn selected_nodes_on(&self, page: usize, object: TargetId) -> Vec<usize> {
        self.entries
            .iter()
            .filter(|e| e.page == page && e.object == object)
            .filter_map(|e| e.node)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Every selected **part** on one object of one page, object-scoped,
    /// ascending and unique — a text object's chunks, or a path's subpaths.
    #[must_use]
    pub fn selected_parts_on(&self, page: usize, object: TargetId) -> Vec<usize> {
        self.entries
            .iter()
            .filter(|e| e.page == page && e.object == object)
            .filter_map(|e| e.subpath)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Ascend one rung, or clear, or decline the key. See [`EscapeOutcome`].
    pub fn escape(&mut self) -> EscapeOutcome {
        match self.level.ascend() {
            Some(SelectionLevel::Part) => {
                for entry in &mut self.entries {
                    entry.node = None;
                }
                self.level = SelectionLevel::Part;
                self.normalise();
                EscapeOutcome::LeftLevel(SelectionLevel::Part)
            }
            Some(SelectionLevel::Object) => {
                for entry in &mut self.entries {
                    entry.subpath = None;
                    entry.node = None;
                }
                self.level = SelectionLevel::Object;
                self.normalise();
                EscapeOutcome::LeftLevel(SelectionLevel::Object)
            }
            // `ascend()` never returns `Node`; the arm exists so adding a
            // fourth rung is a compile error here rather than a silent
            // fall-through to "clear the selection".
            Some(SelectionLevel::Node) => EscapeOutcome::Nothing,
            None if !self.entries.is_empty() => {
                self.entries.clear();
                self.order.clear();
                self.outlines.clear();
                EscapeOutcome::ClearedSelection
            }
            None => EscapeOutcome::Nothing,
        }
    }

    /// **Re-resolve against a fresh decomposition** — invariant 3.
    pub fn resolve(&mut self, targets: Option<&dyn CanvasTargetProvider>, page: usize, epoch: u64) {
        if self.resolved_for == Some((page, epoch)) {
            return;
        }
        self.resolved_for = Some((page, epoch));
        let Some(targets) = targets else {
            self.outlines.clear();
            return;
        };

        // Drop only entries ON THIS PAGE that no longer resolve.
        self.entries
            .retain(|e| e.page != page || targets.bounds(page, e.object).is_some());
        if self.entries.is_empty() {
            self.level = SelectionLevel::Object;
        }

        self.outlines = self
            .entries
            .iter()
            .filter(|e| e.page == page)
            .filter_map(|e| Some((*e, self.outline_rect(targets, e)?)))
            .collect();
    }

    /// **Re-read the selected annotation's geometry from the document** —
    /// the annotation half of invariant 3.
    pub fn resolve_annot(
        &mut self,
        view: &pdfcer_core::view::DocumentView<'_>,
        page: Option<&pdfcer_core::page_tree::Page>,
        page_index: usize,
        epoch: u64,
    ) {
        if self.annot_resolved_for == Some((page_index, epoch)) {
            return;
        }
        // The key is recorded even when there is nothing to do, on
        // `resolve`'s own argument: a page with no annotation selected must not
        // re-walk `/Annots` on every frame merely because it found nothing to
        // update the first time.
        self.annot_resolved_for = Some((page_index, epoch));
        let (Some(selected), Some(page)) = (self.annot.as_mut(), page) else {
            return;
        };
        if selected.target.page != page_index {
            // The selection is on another page and this walk has nothing to say
            // about it — the same rule `resolve` states at length for content.
            return;
        }
        let Some(found) = pdfcer_core::annot::page_annotations(view, page.id)
            .into_iter()
            .find(|a| a.id == Some(selected.target.id))
        else {
            return;
        };
        let Some(rect) = found.rect else {
            return;
        };
        let Some(outline) =
            crate::canvasmapping::annot_canvas_rect([rect.llx, rect.lly, rect.urx, rect.ury], page)
        else {
            return;
        };
        selected.outline = outline;
        // The `/F` bit 8 flag is re-read too. A lock applied while the mark
        // is selected must take the grips away on the next frame, not on the
        // next click — `Grabbable`'s annotation arm reads `target.locked` and
        // would otherwise go on offering nine handles the file forbids.
        selected.target.locked = found.flags.locked();
        selected.oriented = crate::annotquad::oriented(view, &found)
            .filter(|q| !q.is_upright())
            .and_then(|q| crate::canvasmapping::oriented_canvas_quad(q.corners, page));
    }

    /// Whether [`Self::resolve`] would do any work for `(page, epoch)`.
    #[must_use]
    pub fn needs_resolve(&self, page: usize, epoch: u64) -> bool {
        self.resolved_for != Some((page, epoch))
    }

    /// The canvas-space rect to outline for one entry: the **part's** box
    /// once the operator is inside one, the object's box otherwise.
    fn outline_rect(&self, targets: &dyn CanvasTargetProvider, entry: &Selection) -> Option<Rect> {
        // A leaf has no page paint-order index, so it has no *part* box
        // either — the part rung is not offered for one. Its object box is,
        // and that is what gets outlined: `bounds` answers for both lists.
        entry
            .object
            .page_object_index()
            .and_then(|object| {
                entry
                    .subpath
                    .and_then(|part| targets.part_bounds(entry.page, object, part))
            })
            .or_else(|| targets.bounds(entry.page, entry.object))
    }

    /// A plain or shift click while at the Object rung.
    fn click_at_object_rung(&mut self, page: usize, hit: ClickHit, shift: bool) {
        match (shift, hit.object) {
            (false, Some(object)) => {
                if let Some(part) = hit.part
                    && hit.chunk
                    && self.entries == [Selection::object(page, object)]
                {
                    // Through the verb rather than by assembling the entry:
                    // `select_part` is where "stand at the Part rung, on this
                    // part" is defined, traced and normalised, and a second
                    // statement of it here is a second thing to keep in step.
                    self.select_part(page, object, part, "chunk-click");
                } else {
                    self.entries = vec![Selection::object(page, object)];
                }
            }
            (false, None) => self.entries.clear(),
            (true, Some(object)) => {
                let entry = Selection::object(page, object);
                if let Some(at) = self.entries.iter().position(|e| *e == entry) {
                    self.entries.remove(at);
                } else {
                    self.entries.push(entry);
                }
            }
            (true, None) => {}
        }
        self.normalise();
    }

    /// A plain or shift click while inside an object.
    fn click_inside(&mut self, page: usize, hit: ClickHit, shift: bool) {
        let Some(entered) = self.entered_object() else {
            // No entry to be inside of: the level and the entries disagreed,
            // which `normalise` prevents. Recover rather than panic.
            self.level = SelectionLevel::Object;
            self.click_at_object_rung(page, hit, shift);
            return;
        };
        let same_object = hit.object == Some(entered.object) && page == entered.page;

        if same_object && self.level == SelectionLevel::Node && hit.node.is_some() {
            self.pick_within(entered, hit.part.or(entered.subpath), hit.node, shift);
            return;
        }
        if same_object && let Some(part) = hit.part {
            // Either a re-pick at the Part rung, or the Node rung falling
            // back one rung onto a part.
            self.level = SelectionLevel::Part;
            self.pick_within(entered, Some(part), None, shift);
            return;
        }
        // The click left the object. Ascend and treat it as an ordinary
        // Object-rung click, which also covers "clicked a different object"
        // and "clicked empty paper".
        self.level = SelectionLevel::Object;
        self.click_at_object_rung(page, hit, shift);
    }

    /// Select a part or a node inside the entered object.
    fn pick_within(
        &mut self,
        entered: Selection,
        part: Option<usize>,
        node: Option<usize>,
        shift: bool,
    ) {
        let entry = Selection {
            page: entered.page,
            object: entered.object,
            subpath: part,
            node,
        };
        if shift {
            if let Some(at) = self.entries.iter().position(|e| *e == entry) {
                self.entries.remove(at);
            } else {
                self.entries.push(entry);
            }
        } else {
            self.entries = vec![entry];
        }
        self.normalise();
    }

    /// Descend one rung into whatever is under a double-click.
    fn descend(&mut self, page: usize, hit: ClickHit) {
        // **A LEAF DESCENDS TOO** — `OPERATOR_REQUESTS.md` O70.
        //
        // A leaf is painted from inside a form XObject, and the ladder may
        // descend into one only because every rung below it is addressable:
        // `provider::geometry` answers where a leaf's subpaths and anchors are,
        // `canvas::input::probe` asks it, the anchors draw, and the drag routes
        // to `pdfcer-core`'s `*_in_form` verbs.
        //
        // ⚠ All four are one condition. Entering the Part rung with nothing
        // addressable in it makes `canvas::painting` decline to draw anchors
        // and `pressing::grabbable` withhold the outline, so a second
        // double-click makes the selection box VANISH and offers nothing in its
        // place. A guard here would be the wrong repair for that; the right one
        // is the rung being addressable in the first place.
        let Some(object) = hit.object else {
            // A double-click is also a click, and a click on empty paper
            // leaves. Doing anything else here strands the operator.
            self.level = SelectionLevel::Object;
            self.entries.clear();
            self.order.clear();
            self.outlines.clear();
            return;
        };
        let entered = self.entered_object();
        let same_object = entered.is_some_and(|e| e.object == object && e.page == page);

        let (level, entry) = match (same_object, self.level) {
            // Already inside this object at the Part rung: descend to Node.
            // With no anchor within tolerance the rung is still entered —
            // "inside this part, nothing picked yet" is a real state, and
            // refusing to descend would make the gesture feel unreliable on
            // exactly the curves whose anchors are hard to hit.
            (true, SelectionLevel::Part) => (
                SelectionLevel::Node,
                Selection {
                    page,
                    object,
                    subpath: hit.part.or_else(|| entered.and_then(|e| e.subpath)),
                    node: hit.node,
                },
            ),
            // Nothing is below a point.
            (true, SelectionLevel::Node) => return,
            // Entering an object (possibly a different one) from the top.
            _ => (
                SelectionLevel::Part,
                Selection {
                    page,
                    object,
                    subpath: hit.part,
                    node: None,
                },
            ),
        };
        self.level = level;
        self.entries = vec![entry];
        self.normalise();
    }

    /// Restore the two structural rules the rest of the module relies on.
    fn normalise(&mut self) {
        self.entries.sort_unstable();
        self.entries.dedup();
        self.sync_order();
        if self.entries.is_empty() {
            self.level = SelectionLevel::Object;
            self.outlines.clear();
            return;
        }
        if self.level != SelectionLevel::Object {
            let first = self.entries[0];
            if self
                .entries
                .iter()
                .any(|e| e.object != first.object || e.page != first.page)
            {
                self.level = SelectionLevel::Object;
                for entry in &mut self.entries {
                    entry.subpath = None;
                    entry.node = None;
                }
                self.entries.sort_unstable();
                self.entries.dedup();
                self.sync_order();
            }
        }
        // The outlines describe the entries; any change to the entries makes
        // them stale, and a stale outline is a box drawn around something the
        // operator no longer has selected.
        self.resolved_for = None;
        self.annot_resolved_for = None;
    }
}

#[cfg(test)]
mod order_tests {
    use super::*;

    fn ids(s: &SelectionState) -> Vec<TargetId> {
        s.in_selection_order().iter().map(|e| e.object).collect()
    }

    #[test]
    fn additions_keep_the_order_they_arrived_in() {
        let mut s = SelectionState::default();
        s.marquee(0, &[TargetId::Object(3)], false);
        s.marquee(0, &[TargetId::Object(1)], true);
        s.marquee(0, &[TargetId::Object(2)], true);
        assert_eq!(
            ids(&s),
            [
                TargetId::Object(3),
                TargetId::Object(1),
                TargetId::Object(2)
            ]
        );
        // `entries` stays sorted; only the order view differs.
        assert_eq!(s.entries()[0].object, TargetId::Object(1));
    }

    #[test]
    fn removal_drops_from_the_order_and_a_fresh_marquee_restarts_it() {
        let mut s = SelectionState::default();
        s.marquee(0, &[TargetId::Object(3)], false);
        s.marquee(0, &[TargetId::Object(1)], true);
        s.marquee(0, &[TargetId::Object(2)], true);
        s.marquee_remove(0, &[TargetId::Object(1)]);
        assert_eq!(ids(&s), [TargetId::Object(3), TargetId::Object(2)]);
        s.marquee(0, &[TargetId::Object(5), TargetId::Object(4)], false);
        assert_eq!(ids(&s), [TargetId::Object(4), TargetId::Object(5)]);
        s.clear();
        assert!(ids(&s).is_empty());
        s.marquee(0, &[TargetId::Object(9)], false);
        s.marquee(0, &[TargetId::Object(4)], true);
        assert_eq!(ids(&s), [TargetId::Object(9), TargetId::Object(4)]);
    }
}
