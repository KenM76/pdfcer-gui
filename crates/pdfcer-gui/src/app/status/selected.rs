//! # `status::selected` — what is selected, said in words
//!
//! One line at the left of the status bar, naming the thing the operator has
//! selected and — when it matters — how many other things were under the same
//! click.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/status/selected.md`.

use egui::Ui;

use crate::app::state::OpenDoc;
use crate::text::status as t;

/// The region this line publishes, so a driven check can find it.
pub const REGION: &str = "status-group:selected"; // ui-text-exempt: trace region name, never displayed

/// `status-rung kind=text|path part=N held=H of=M` — the rung clause this
/// bar appended, stated on the channel a harness can read.
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
    // *1 line of 3* about a line that is not there.
    if part >= of {
        return (line, None);
    }
    let kind = provider.part_kind(index);
    drop(provider);
    // How many chunks are held, not how many the first entry is. A Shift-click
    // at this rung adds a second, and every consumer below — the drag, Delete,
    // this sentence — takes the whole set as its operand.
    let held = doc.selection.selected_parts_on(page, first).len().max(1);

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
        Some(PartKind::TextLine) => {
            trace_rung(RUNG_TEXT, part, held, of);
            (
                t::selection_part_of_text(&line, held, of),
                Some(t::selection_part_of_text_hint()),
            )
        }
        Some(PartKind::Subpath) => {
            trace_rung(RUNG_PATH, part, held, of);
            (
                t::selection_part_of_path(&line, held, of),
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
fn trace_rung(kind: &str, part: usize, held: usize, of: usize) {
    crate::diag::trace_changed(RUNG_SLOT, || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("{RUNG_SLOT} kind={kind} part={part} held={held} of={of}")
    });
}

/// **Which layer the selection is on, appended to the line that already
/// names it.**
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
