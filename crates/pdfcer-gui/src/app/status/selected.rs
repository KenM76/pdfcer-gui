//! # `status::selected` — what is selected, said in words
//!
//! One line at the left of the status bar, naming the thing the operator has
//! selected and — when it matters — how many other things were under the same
//! click.
//!
//! ## The state this exists to make legible
//!
//! The operator:
//!
//! > *"when I click on one of the objects all I get is the page selected."*
//!
//! That report is precise. A file that wraps the whole visible body of the
//! sheet in a page-sized form XObject gives that form a bounding box which
//! wins every click at every point, and the engine does not enter one — so
//! what is selected really is a page-sized object.
//!
//! Without this line **nothing on screen says so**. The selection outline is
//! drawn round the page edge, which looks exactly like *"the page is
//! selected"* — a state this program does not have. No other surface says
//! *"you have selected a Form containing 214 objects"*, which is a diagnosis,
//! and from a diagnosis the next question follows on its own.
//!
//! This line does not fix the selecting. It makes the selecting **legible**,
//! which is what turns an unexplainable interface into a solvable one — and it
//! is the surface every refusal sentence is printed on.
//!
//! ## Why the left, and why it is the thing that yields
//!
//! The right-hand cluster is fixed controls the operator reaches for — page,
//! zoom, fit, Find, the pick filter — and `status::fitting` may shed only two
//! of those, because the rest have no other home. This is a **readout**: it
//! costs nothing to lose, because everything it says is also visible in the
//! Objects panel and in the selection outline.
//!
//! So it goes on the left with the other narration, where `egui`'s left-to-right
//! run gives up its space first, and it elides rather than pushing. That is
//! `status`'s own rule about what yields, applied to the newest thing on the
//! bar rather than exempting it.
//!
//! ## What it says, and what it refuses to say
//!
//! | state | line |
//! |---|---|
//! | nothing selected | nothing at all |
//! | one object | its kind, and its size in points |
//! | one object, more underneath | `… · 1 of 5 here` |
//! | several objects | `3 objects selected` |
//!
//! **Nothing when nothing is selected**, rather than *"Nothing selected"*.
//! A status bar that narrates the absence of a thing spends a permanent line on
//! the most common state in the program. A tutorial string there may well be
//! worth having, but that would be a decision about **teaching** and this line
//! is a decision about **reporting**. They should not be made at once and they
//! should not be made by the same code.

use egui::Ui;

use crate::app::state::OpenDoc;
use crate::text::status as t;

/// The region this line publishes, so a driven check can find it.
pub const REGION: &str = "status-group:selected"; // ui-text-exempt: trace region name, never displayed

/// `status-rung kind=text|path part=N of=M` — the rung clause this bar
/// appended, stated on the channel a harness can read.
///
/// # Why a label's own words need a trace line at all
///
/// [`crate::diag::ui_rect`] publishes WHERE this label was drawn and never
/// WHAT it says. That is the right division for a layout oracle and it is
/// useless for a content one: a build that drew the readout and dropped the
/// rung clause publishes a byte-identical region, so a check asserting the
/// region is satisfied by both outcomes and measures neither.
///
/// ⇒ So the clause states itself. The line is emitted from the same arm
/// that builds the clause, out of the same numbers, on the frame the label
/// is drawn — so a harness that sees this line AND the region on the same
/// frame has measured that the sentence exists and that the bar drew it.
/// Neither half alone says that, which is why a check should assert both.
///
/// **"From the same arm" is load-bearing.** An emission placed ABOVE the
/// `match` and keyed on the same `PartKind` the arms are keyed on reads as
/// equivalent and is not: falsification recipe (4) of the driven check —
/// replace both arms with `(line, None)` — leaves such a trace firing and
/// the check PASSING on a build that discloses nothing. It goes through
/// [`trace_rung`], called from the two producing arms and from nowhere else.
///
/// It is a trace of the DECISION, not a transcription of the string.
/// Echoing the rendered text would make every wording change a harness
/// change and would tempt a check into asserting English; `kind`, `part`
/// and `of` are the three facts the clause is computed from, and a build
/// that gets any of them wrong gets the sentence wrong too.
///
/// Routed through [`crate::diag::trace_changed`] rather than
/// [`crate::diag::trace`], because this is drawn sixty times a second and
/// a selection that is sitting still would otherwise bury the channel —
/// on `canvas-pointer` a stationary pointer writes fifty identical lines in
/// nine seconds. The de-duplication is on the
/// rendered line, so a harness must not assume one press produces one line:
/// an EARLIER gesture that produced the identical clause suppresses the
/// later one. A check wanting a before/after verdict asserts that the
/// count before its gesture was ZERO, rather than that a new line follows
/// a mark.
const RUNG_SLOT: &str = "status-rung"; // ui-text-exempt: trace slot name, never displayed

/// Draw the selection readout, or nothing.
///
/// Takes `&OpenDoc` and the context: the selection is on the document, and the
/// **depth** of the click that made it is in `egui::Memory` — see
/// [`crate::canvas::depth`] for why those two live apart.
pub(super) fn show(ui: &mut Ui, doc: &OpenDoc) {
    let page = doc.view.page_index;
    // `targets_on`, NOT `object_indices_on`.
    //
    // This is a **readout**, and a readout must describe what the operator can
    // see. `object_indices_on` answers about the page's own paint order only —
    // it is the edit-operand funnel and it drops every target that lives inside
    // a form XObject, correctly, because no paint-order verb can address one.
    //
    // Reading the operand list here would have made this line go **silent** on
    // exactly the selection it was written for: the operator clicks an object
    // inside a form, sees an outline, and the bar says nothing. That is worse
    // than the "page selected" state it replaced, because at least that one
    // showed something to be puzzled by.
    let targets = doc.selection.targets_on(page);
    let Some(&first) = targets.first() else {
        // **NOTHING SELECTED, BUT STILL INSIDE SOMETHING** —
        // `OPERATOR_REQUESTS.md` O70, and it is the state that needs saying
        // most.
        //
        // One Escape clears the selection and leaves the operator scoped to the
        // container they entered, so their next click still resolves inside it.
        // With no line here that state is completely invisible: nothing is
        // outlined, nothing is armed, and the only evidence is that clicks
        // behave differently from the same clicks a moment earlier — which
        // reads as the program having gone wrong.
        //
        // ⇒ Rule 4 in its plainest form: the canvas is not marked, and the
        // fact is stated off it. The sentence names the way out, because a
        // scope with no visible exit is the stranding this whole arm was
        // designed to make unrepresentable.
        let slot = crate::pagedrag::active(ui.ctx()).unwrap_or_default().slot;
        if crate::canvas::smart::entered(ui.ctx(), page, slot).is_some() {
            let response = ui.label(t::inside_container());
            crate::diag::ui_rect(REGION, response.rect);
        }
        return;
    };

    let text = if targets.len() > 1 {
        t::selection_many(targets.len())
    } else {
        // The kind and the size come from the DECOMPOSITION, not from the
        // selection: a selection is four integers and knows nothing about what
        // it names. `page_objects` is the cache the canvas and the Objects
        // panel already read, so this adds no work on a frame that has drawn
        // either of them — and on a frame that has not, it is the same
        // extraction they would have paid for anyway.
        let described = doc.page_objects().and_then(|provider| {
            let model = provider.page_objects();
            // Both index spaces, resolved by the id rather than by a caller
            // that had to remember which one it was holding. `nesting` is
            // `None` for a page object and `Some(depth)` for a form-interior
            // one — which is the only thing the two branches disagree about.
            let (object, nesting) = match first {
                crate::canvas::target::TargetId::Object(i) => {
                    (model.objects.get(usize::try_from(i).ok()?)?, None)
                }
                crate::canvas::target::TargetId::Leaf(i) => {
                    let leaf = model.leaves.get(usize::try_from(i).ok()?)?;
                    (&leaf.object, Some(leaf.containment.len()))
                }
            };
            let kind = crate::text::panels::objects::object_kind_label(
                crate::panels::objects::summary::object_kind(object),
            );
            // Canvas space, from the provider's own projection rather than a
            // second one built here — `bounds` is what the overlay draws the
            // selection outline from, so the number in this line and the box on
            // screen cannot describe different rectangles. It answers for both
            // lists, so this call does not branch.
            let size = provider
                .bounds(page, first)
                .map(|r| (r.width(), r.height()));
            Some(match (size, nesting) {
                (Some((w, h)), None) => t::selection_one(kind, w, h),
                (None, None) => t::selection_one_unsized(kind),
                (Some((w, h)), Some(n)) => t::selection_one_in_form(kind, w, h, n),
                (None, Some(n)) => t::selection_one_in_form_unsized(kind, n),
            })
        });
        // A selection naming an object the decomposition does not have is a
        // stale index — which `SelectionState::resolve` exists to prevent and
        // which is therefore not expected. Saying the plain thing is better
        // than saying nothing: the operator still learns that something is
        // selected.
        described.unwrap_or_else(|| t::selection_many(1))
    };

    // The stack count, appended rather than separate, because it is a fact
    // about the SAME selection: *"this one, and there were four others."* A
    // second label would read as a second subject.
    //
    // `canvas::depth` returns `None` unless there were at least two candidates,
    // so the common case adds nothing — and it returns `None` for a selection
    // that did not come from a click at all, which is what stops this claiming
    // a stack the operator is not pointing at.
    //
    // It is keyed on the `TargetId`, so a depth measured for `leaves[7]` is
    // never claimed by a selection of `objects[7]`. A page has two index
    // spaces, so a bare integer key would let one answer the other.
    let text = match crate::canvas::depth::taken(ui.ctx(), page, first) {
        Some(depth) => t::selection_with_depth(&text, depth.taken + 1, depth.of),
        None => text,
    };

    // **The rung, said in words** — `OPERATOR_REQUESTS.md` O188(A).
    //
    // Appended before the layer clause and after the depth clause, which is
    // the order the three facts narrow in: *what it is* (kind and size),
    // *which of the ones under the pointer* (depth), *how much of it you
    // hold* (rung), *where it lives* (layer). Each clause is about the one
    // before it.
    let (text, hint) = with_part(doc, page, first, text);

    let text = with_layer(doc, &text);

    let response = ui.label(text);
    crate::diag::ui_rect(REGION, response.rect);
    // The hover carries the verbs, not the readout. `disclosure`'s rule:
    // eliding defers rather than loses — the bar has room for *1 line of
    // 27* and not for the sentence that says what Delete will do with it.
    if let Some(hint) = hint {
        response.on_hover_text(hint);
    }
}

/// **Is the selection narrower than the object it names, and by how
/// much?**
///
/// Returns the line with a rung clause appended, and the hover that belongs
/// behind it — or the line unchanged and `None`.
///
/// # Why this reads the level rather than the entry's `subpath`
///
/// Both would work today. `SelectionLevel` is the **stated** answer, kept in
/// step by `normalise`, and `subpath: Some(_)` is the representation that
/// happens to carry it; a readout that inferred the rung from the
/// representation would be a second definition of what Part means. The
/// level is asked first and the index is read only once the level has said
/// there is one.
///
/// # A leaf produces no clause, and that is not a hole
///
/// The Part rung is unreachable inside a form XObject: `part_hits_of`
/// matches on a page-object index and returns nothing for a leaf, so the
/// ladder caps itself at the object rung there by construction. Requiring
/// `page_object_index` here is therefore an assertion of that fact rather
/// than a case being dropped — and if it ever stops being true, the clause
/// goes quiet rather than printing a total it computed from the wrong index
/// space.
///
/// # The total is re-read every frame
///
/// From the same `page_objects` cache the outline was drawn from, keyed on
/// `(page, edit_epoch)`. A reflow that changes how many runs the object has
/// changes this number on the next frame, which is the only behaviour that
/// keeps *1 line of 27* from becoming a claim about a document revision the
/// operator is no longer looking at.
fn with_part(
    doc: &OpenDoc,
    page: usize,
    first: crate::canvas::target::TargetId,
    line: String,
) -> (String, Option<&'static str>) {
    use crate::canvas::selection::SelectionLevel;
    use crate::panels::objects::provider::PartKind;

    if doc.selection.level() != SelectionLevel::Part {
        return (line, None);
    }
    let Some(part) = doc
        .selection
        .entries()
        .iter()
        .find(|e| e.page == page && e.object == first)
        .and_then(|e| e.subpath)
    else {
        return (line, None);
    };
    let Some(index) = first.page_object_index() else {
        return (line, None);
    };
    let Some(provider) = doc.page_objects() else {
        return (line, None);
    };
    let of = provider.part_count(index);
    // A part index outside the object's own count is a stale selection the
    // resolver is supposed to have cleared. Saying nothing beats saying
    // *1 line of 3* about a run that is not there.
    if part >= of {
        return (line, None);
    }
    let kind = provider.part_kind(index);
    drop(provider);

    // **The trace is emitted from INSIDE the producing arms, and that
    // placement is the whole of its value.**
    //
    // Keyed on `kind` above this `match`, it would still fire on a build whose
    // arms had been replaced with `(line, None)` — a build where the operator
    // stands on one line of six and the status bar says nothing about it,
    // which is exactly what the line exists to report. Everything such a trace
    // says is true; it is a statement about the four early returns above
    // rather than about the clause, and an assertion both outcomes satisfy
    // measures neither.
    //
    // ⇒ The emission and the sentence are produced by the same arm, so there
    // is no edit that removes the clause and leaves the trace. See
    // [`RUNG_SLOT`] for why a rect cannot make this claim and why the fields
    // are the DECISION rather than the sentence.
    //
    // The `None` arm is deliberately silent rather than tracing
    // `kind=none`. A part selection whose object reports no part kind produces
    // no clause BY DESIGN, and a trace line there would make the absence of a
    // clause indistinguishable from the presence of one to any check that only
    // counts lines.
    match kind {
        Some(PartKind::Run) => {
            trace_rung(RUNG_TEXT, part, of);
            (
                t::selection_part_of_text(&line, of),
                Some(t::selection_part_of_text_hint()),
            )
        }
        Some(PartKind::Subpath) => {
            trace_rung(RUNG_PATH, part, of);
            (
                t::selection_part_of_path(&line, of),
                Some(t::selection_part_of_path_hint()),
            )
        }
        None => (line, None),
    }
}

/// The `kind=` token for one line of a text object. See [`RUNG_SLOT`].
// ui-text-exempt: diagnostic trace fragment, never displayed in the UI
const RUNG_TEXT: &str = "text";

/// The `kind=` token for one subpath of a shape. See [`RUNG_SLOT`].
// ui-text-exempt: diagnostic trace fragment, never displayed in the UI
const RUNG_PATH: &str = "path";

/// Say which rung the status bar just disclosed, on the frame it disclosed it.
///
/// Called from the two arms of [`with_part`] that build a rung clause, and from
/// nowhere else — which is the property the driven check depends on, and the
/// reason this is a function rather than four lines repeated twice: a second
/// call site added anywhere would be visible here, and a reader who wants to
/// know what can emit this line has one place to look.
///
/// Routed through [`crate::diag::trace_changed`] because the status bar is
/// built sixty times a second and an unconditional trace would write fifty
/// identical lines in nine seconds — the `canvas-pointer` lesson. The
/// de-duplication is keyed on the RENDERED line, which has one consequence a
/// harness must honour: a check wanting a before/after verdict asserts that the
/// count BEFORE its gesture was zero, rather than that a new line follows a
/// mark. A line identical to one already written is suppressed, and a
/// mark-relative assertion would read that suppression as the feature being
/// broken.
fn trace_rung(kind: &str, part: usize, of: usize) {
    crate::diag::trace_changed(RUNG_SLOT, || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("{RUNG_SLOT} kind={kind} part={part} of={of}")
    });
}

/// **Which layer the selection is on, appended to the line that already
/// names it.**
///
/// # Why this is on the status bar and not only in the Layers panel
///
/// **The canvas is the primary surface, never a panel.** `pdfcer-core`
/// answers *"which layer is this object on"* through `VectorObject::oc()`, so
/// clicking the object has to be able to *reach* that answer. A capability
/// whose only route is a panel is a capability the operator must already know
/// exists before they can use it — and this one is asked for as *"selecting
/// an object highlights that layer"*, which is a sentence about **clicking**,
/// not about a panel.
///
/// The panel is the supplement: it is where the answer can be acted on, by
/// switching the layer off. This is where it can be *seen*, with nothing open.
///
/// # Rule 4: nothing is drawn on the drawing
///
/// No badge, tint, dashed outline or provisional layer is painted over the
/// selected content to express its membership. The selection handles are the
/// cursor and are untouched. **Render normally; report separately. Both.**
///
/// # Silent on a document with no optional content, which is nearly all of
/// them
///
/// The engine measures **0.6 %** of a 500-file corpus as carrying optional
/// content at all. On the other 99.4 % *"which layer"* is not a question the
/// operator has, and `not on a layer` after every single click would be a
/// permanent line about a feature the document does not use — the same fault
/// as narrating "Nothing selected", which this module's header refuses for
/// the same reason.
///
/// So the clause appears only once `read_layers` says the document declares
/// groups. That read walks `/OCProperties` and its `/OCGs` array — a handful
/// of dictionary lookups, no content stream — and it is behind the
/// `targets.first()` guard above, so it costs nothing on a frame with nothing
/// selected. The Layers panel makes the same call every frame it is open.
fn with_layer(doc: &OpenDoc, line: &str) -> String {
    let view = doc.session.view();
    let read = pdfcer_core::layers::read_layers(&view);
    if read.diagnostics.no_optional_content {
        return line.to_owned();
    }
    let membership = crate::panels::layers::highlight::resolve(doc);
    // The name comes from the panel's own `row_name`, through
    // `layer_name_for`. One spelling of what a layer is called: a bar that
    // read `/Name` itself would print an empty string where the panel prints
    // its placeholder, for the same layer, on two surfaces visible at once.
    let name = membership
        .highlighted()
        .and_then(|id| crate::panels::layers::layer_name_for(&read, id));
    let Some(clause) = crate::text::panels::layers::layer_clause(membership, name.as_deref())
    else {
        return line.to_owned();
    };
    crate::diag::trace(|| {
        // Published for the driven check, and it carries the ANSWER rather
        // than merely that an answer happened. A trace saying "a layer clause
        // was drawn" would pass under a build that always names the same
        // layer — the vacuous shape a fixture whose objects all sit on one
        // layer hides, which is why the check's fixture carries two layers and
        // an object on neither.
        //
        // `kind`/`reason` rather than `{:?}`: see `Membership::kind`. The
        // NAME is quoted last, because it is the one field that may contain a
        // space and a `key=value` parser must not have to cope with one in the
        // middle of a line.
        format!(
            "layer-membership page={page} answer={} reason={} name={:?}",
            membership.kind(),
            membership.reason(),
            name.as_deref().unwrap_or_default(),
            page = doc.view.page_index,
        )
    });
    crate::text::panels::layers::selection_with_layer(line, &clause)
}
