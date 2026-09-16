//! # `canvas::forms::ring` — the order Tab visits a page's fields in
//!
//! `OPERATOR_REQUESTS.md` O204, the field half.
//!
//! ## Contract
//!
//! [`rings`] answers, for one document revision, *which boxes of
//! [`super::placed`]'s list are tab stops, on which page, in what order* —
//! together with everything that had to be inferred or dropped to say so.
//! Stops are **indices into the caller's `&[WidgetBox]`**, because that list is
//! already the thing every other part of this module addresses a field by.
//!
//! ## The order is the engine's, and this module sorts nothing
//!
//! `EditSession::page_tab_sequence` implements §12.5.1 — `/Tabs` `/A`, `/W`,
//! `/R`, `/C` and `/S`, the `/Rect` normalisation, the page `/Rotate`, the
//! `/R2L` viewer preference and the four exclusions a reader never visits. A
//! second implementation of a rule that fiddly disagrees with the first the day
//! a page is rotated, and the disagreement would show up as *the wrong field*
//! rather than as an error. So nothing here compares a rectangle.
//!
//! What this module does is the one step the engine's own doc says a caller
//! owes it: **collapse each radio group to a single stop**, which is a grouping
//! of fields rather than of annotations and so cannot be done from an `/Annots`
//! walk.
//!
//! ## Where the disclosure goes
//!
//! `TabSequence::notes` are operator-readable sentences about inferences the
//! operator cannot see — a `/Tabs` pdfcer chose a convention for, a tail the
//! standard contradicts itself about. Rule 4 requires them to be reported and
//! forbids marking the canvas with them, so they are carried on [`TabRings`],
//! traced, and left for an off-canvas surface to print verbatim. Nothing here
//! paints.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;
use crate::canvas::forms::boxes::{BoxKind, WidgetBox};

/// `ctx.data` key for the per-revision ring table.
const RINGS_KEY: &str = "pdfcer-canvas-form-rings"; // ui-text-exempt: internal memory id, never displayed

/// Trace slot for the ring census.
const RINGS_SLOT: &str = "canvas-form-rings"; // ui-text-exempt: trace slot name, never displayed

/// One box, reduced to what ordering needs to know about it.
///
/// A borrow-free projection of [`WidgetBox`] so that [`assemble`] — the only
/// part of this module with a decision in it — can be tested without a
/// document, a page tree or an `egui::Context`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stop {
    /// Where this box sits in the caller's list.
    pub index: usize,
    /// The widget annotation's object id, which is the vocabulary the engine's
    /// answer is written in.
    pub id: ObjId,
    /// The field's fully-qualified name — the key a radio group collapses on.
    pub field: String,
    /// Whether this box is one widget of a radio group.
    pub radio: bool,
}

/// What [`assemble`] produced, and what it had to leave out to produce it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Assembled {
    /// The tab stops, as indices into the caller's list.
    pub stops: Vec<usize>,
    /// Boxes on the page that the engine's order does not name.
    ///
    /// Non-zero means a widget a reader does **not** tab to — hidden, or an
    /// `/Annots` entry written as a direct dictionary, which has no identity to
    /// be named by. Counted rather than appended: tabbing to a hidden field is
    /// interacting with one, which §12.5.3 forbids, and appending a stop the
    /// operator cannot see is the invisible inference rule 4 is about.
    pub unnamed: usize,
    /// Widgets folded into a radio group's single stop.
    pub collapsed: usize,
}

/// **Order one page's boxes, and collapse its radio groups.**
///
/// `order` is `TabSequence::order` — every annotation a reader visits on that
/// page, widgets and non-widgets alike, in visit order. Ids naming something
/// that is not in `stops` (a `/Link`, a `/Text` note) are skipped without
/// comment: they are annotations, not fields, and this ring is over fields.
///
/// An **empty** `order` means the engine could not derive one — `/Tabs /S`, or
/// an `/Annots` that is not an array. The fallback is `stops` in the order they
/// arrived, which is `/Annots` order, and the caller says so off-canvas rather
/// than letting the two cases look alike.
#[must_use]
pub fn assemble(order: &[ObjId], stops: &[Stop]) -> Assembled {
    let mut out = Assembled::default();
    // A named helper rather than a closure, because the borrow checker needs
    // to see that the field name outlives the set it is inserted into, and a
    // closure capturing the set cannot express that.
    fn push<'a>(stop: &'a Stop, groups: &mut HashSet<&'a str>, out: &mut Assembled) {
        if stop.radio && !groups.insert(stop.field.as_str()) {
            out.collapsed += 1;
            return;
        }
        out.stops.push(stop.index);
    }
    let mut groups: HashSet<&str> = HashSet::new();

    if order.is_empty() {
        for stop in stops {
            push(stop, &mut groups, &mut out);
        }
        return out;
    }

    let by_id: HashMap<ObjId, usize> = stops
        .iter()
        .enumerate()
        .map(|(i, stop)| (stop.id, i))
        .collect();
    let mut taken = vec![false; stops.len()];
    for id in order {
        let Some(&i) = by_id.get(id) else {
            continue;
        };
        if taken[i] {
            continue;
        }
        taken[i] = true;
        push(&stops[i], &mut groups, &mut out);
    }
    out.unnamed = taken.iter().filter(|seen| !**seen).count();
    out
}

/// The tab rings of every page that has one, for one document revision.
#[derive(Debug, Clone, Default)]
pub struct TabRings {
    /// `(page index, stops)`, ascending by page, with no empty entries.
    ///
    /// Empty rings are dropped rather than carried because
    /// [`crate::canvas::tabnav::step`] takes the table as the definition of
    /// where a press can land, and a page with nothing to focus is a page a
    /// press must walk past rather than onto.
    pub rings: Vec<(usize, Vec<usize>)>,
    /// The engine's own disclosure sentences, verbatim, in page order.
    pub notes: Vec<String>,
    /// See [`Assembled::unnamed`], summed over the pages.
    pub unnamed: usize,
    /// See [`Assembled::collapsed`], summed over the pages.
    pub collapsed: usize,
    /// Pages whose order the engine declined to derive, and which therefore
    /// fell back to `/Annots` order.
    pub not_derived: usize,
}

impl TabRings {
    /// The `(page, length)` table [`crate::canvas::tabnav::step`] walks.
    #[must_use]
    pub fn lens(&self) -> Vec<(usize, usize)> {
        self.rings
            .iter()
            .map(|(page, stops)| (*page, stops.len()))
            .collect()
    }

    /// Where a box sits in its page's ring, given its index in the caller's
    /// list.
    ///
    /// `None` for a box that is on no ring — a hidden widget, or one whose page
    /// has no ring at all.
    #[must_use]
    pub fn locate(&self, page: usize, index: usize) -> Option<(usize, usize)> {
        let (_, stops) = self.rings.iter().find(|(p, _)| *p == page)?;
        stops
            .iter()
            .position(|i| *i == index)
            .map(|pos| (page, pos))
    }

    /// The caller's list index at one position of one page's ring.
    #[must_use]
    pub fn at(&self, page: usize, pos: usize) -> Option<usize> {
        let (_, stops) = self.rings.iter().find(|(p, _)| *p == page)?;
        stops.get(pos).copied()
    }
}

/// **The ring table for this document revision**, built once and cached.
///
/// Keyed on `(path, edit epoch)`, exactly as [`super::placed`] is and for the
/// same reason: the stops are indices into that list, so the two must be
/// rebuilt on the same boundary or an index can name a box that has moved.
///
/// ★ Built for every page that has a box rather than for every page in the
/// document, which is what keeps the cost proportional to the form instead of
/// to the file. It is still strictly less work than the cache miss beside it —
/// [`super::placed`] asks `widget_rects` for *every* page — and it happens on
/// the same boundary, so a form is not parsed twice per revision.
pub(crate) fn rings(ctx: &egui::Context, doc: &OpenDoc, list: &[WidgetBox]) -> Arc<TabRings> {
    let id = egui::Id::new(RINGS_KEY);
    let key = (doc.path.clone(), doc.edit_epoch);
    if let Some((cached, table)) =
        ctx.data(|d| d.get_temp::<((std::path::PathBuf, u64), Arc<TabRings>)>(id))
        && cached == key
    {
        return table;
    }

    let mut table = TabRings::default();
    let mut page = None;
    let mut stops: Vec<Stop> = Vec::new();
    // `list` is already sorted by page — `boxes::place` walks the pages in
    // order — so one pass with a running page is enough, and it is the pass
    // that keeps `rings` ascending without a sort.
    for (index, widget_box) in list.iter().enumerate() {
        if page != Some(widget_box.page) {
            flush(doc, page, &stops, &mut table);
            page = Some(widget_box.page);
            stops.clear();
        }
        stops.push(Stop {
            index,
            id: widget_box.id,
            field: widget_box.field.clone(),
            radio: matches!(widget_box.kind, BoxKind::Radio { .. }),
        });
    }
    flush(doc, page, &stops, &mut table);

    let table = Arc::new(table);
    ctx.data_mut(|d| d.insert_temp(id, (key, Arc::clone(&table))));

    crate::diag::trace_changed(RINGS_SLOT, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "form-rings pages={} stops={} unnamed={} collapsed={} not_derived={} notes={}",
            table.rings.len(),
            table.rings.iter().map(|(_, s)| s.len()).sum::<usize>(),
            table.unnamed,
            table.collapsed,
            table.not_derived,
            table.notes.len(),
        )
    });
    // The engine's sentences, verbatim and one per line, because rule 4's
    // surviving half is that an inference the operator cannot see still owes a
    // report — and a count of notes is not the report, it is the promise of
    // one.
    if crate::diag::enabled() {
        for note in &table.notes {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("form-ring-note {note}")
            });
        }
    }
    table
}

/// Ask the engine for one page's order, assemble it, and fold the result in.
fn flush(doc: &OpenDoc, page: Option<usize>, stops: &[Stop], table: &mut TabRings) {
    let Some(page) = page else {
        return;
    };
    if stops.is_empty() {
        return;
    }
    let (order, mut notes, not_derived) = match doc.session.page_tab_sequence(page) {
        Ok(sequence) => {
            let not_derived = sequence.order.is_empty();
            (sequence.order, sequence.notes, not_derived)
        }
        // A page whose `/Annots` is not an array, or an index the page tree
        // does not reach. The array order is the only thing left, and saying so
        // is better than refusing to tab.
        Err(err) => (Vec::new(), vec![format!("{err}")], true),
    };
    let assembled = assemble(&order, stops);
    if assembled.stops.is_empty() {
        return;
    }
    if not_derived {
        table.not_derived += 1;
    }
    table.unnamed += assembled.unnamed;
    table.collapsed += assembled.collapsed;
    table.notes.append(&mut notes);
    table.rings.push((page, assembled.stops));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(n: u32) -> ObjId {
        ObjId::new(n, 0)
    }

    fn text(index: usize, n: u32, field: &str) -> Stop {
        Stop {
            index,
            id: obj(n),
            field: field.to_owned(),
            radio: false,
        }
    }

    fn radio(index: usize, n: u32, field: &str) -> Stop {
        Stop {
            index,
            id: obj(n),
            field: field.to_owned(),
            radio: true,
        }
    }

    /// The order is the engine's, whatever order the list arrived in.
    #[test]
    fn the_stops_come_out_in_the_engines_order_not_the_lists() {
        let stops = vec![text(0, 10, "A"), text(1, 11, "B"), text(2, 12, "C")];
        let out = assemble(&[obj(12), obj(10), obj(11)], &stops);
        assert_eq!(out.stops, vec![2, 0, 1]);
        assert_eq!(out.unnamed, 0);
    }

    /// `TabSequence::order` covers every annotation a reader visits, not only
    /// the widgets — a `/Link` in the middle of the page is in it and is not a
    /// field.
    #[test]
    fn an_id_that_is_not_one_of_our_boxes_is_skipped_without_comment() {
        let stops = vec![text(0, 10, "A"), text(1, 11, "B")];
        let out = assemble(&[obj(10), obj(99), obj(11)], &stops);
        assert_eq!(out.stops, vec![0, 1]);
        assert_eq!(out.unnamed, 0);
    }

    /// A widget the engine excluded — hidden, or a direct dictionary — is
    /// dropped from the ring and counted, never appended.
    #[test]
    fn a_box_the_order_does_not_name_is_dropped_and_counted() {
        let stops = vec![text(0, 10, "A"), text(1, 11, "Hidden"), text(2, 12, "C")];
        let out = assemble(&[obj(10), obj(12)], &stops);
        assert_eq!(out.stops, vec![0, 2]);
        assert_eq!(out.unnamed, 1);
    }

    /// The one step the engine's doc says the caller owes it: several
    /// annotations, one field, one tab stop.
    #[test]
    fn a_radio_group_is_one_stop_however_many_widgets_it_has() {
        let stops = vec![
            text(0, 10, "Name"),
            radio(1, 11, "Colour"),
            radio(2, 12, "Colour"),
            radio(3, 13, "Colour"),
            text(4, 14, "Note"),
        ];
        let out = assemble(&[obj(10), obj(11), obj(12), obj(13), obj(14)], &stops);
        assert_eq!(out.stops, vec![0, 1, 4]);
        assert_eq!(out.collapsed, 2);
    }

    /// Two radio groups are two stops, and the first widget of each is the one
    /// that carries it.
    #[test]
    fn two_radio_groups_stay_two_stops() {
        let stops = vec![
            radio(0, 10, "Left"),
            radio(1, 11, "Right"),
            radio(2, 12, "Left"),
            radio(3, 13, "Right"),
        ];
        let out = assemble(&[obj(10), obj(11), obj(12), obj(13)], &stops);
        assert_eq!(out.stops, vec![0, 1]);
        assert_eq!(out.collapsed, 2);
    }

    /// `/Tabs /S`: the engine hands back an empty order deliberately, and the
    /// fallback is the array order the list is already in.
    #[test]
    fn an_empty_order_falls_back_to_the_lists_own_order() {
        let stops = vec![text(0, 10, "A"), radio(1, 11, "R"), radio(2, 12, "R")];
        let out = assemble(&[], &stops);
        assert_eq!(out.stops, vec![0, 1]);
        assert_eq!(out.collapsed, 1);
        assert_eq!(out.unnamed, 0);
    }

    /// An order that names the same annotation twice visits it once. Neither
    /// edition forbids a repeated `/Annots` entry.
    #[test]
    fn a_repeated_id_is_visited_once() {
        let stops = vec![text(0, 10, "A"), text(1, 11, "B")];
        let out = assemble(&[obj(10), obj(10), obj(11)], &stops);
        assert_eq!(out.stops, vec![0, 1]);
        assert_eq!(out.unnamed, 0);
    }

    #[test]
    fn the_table_locates_a_stop_and_reads_one_back() {
        let table = TabRings {
            rings: vec![(0, vec![3, 7]), (4, vec![1])],
            ..TabRings::default()
        };
        assert_eq!(table.lens(), vec![(0, 2), (4, 1)]);
        assert_eq!(table.locate(0, 7), Some((0, 1)));
        assert_eq!(table.locate(0, 9), None);
        assert_eq!(table.locate(2, 3), None);
        assert_eq!(table.at(4, 0), Some(1));
        assert_eq!(table.at(4, 1), None);
    }
}
