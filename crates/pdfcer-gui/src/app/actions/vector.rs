//! # `app::actions::vector` — everything that changes page GEOMETRY
//!
//! ## Why this is its own file
//!
//! **R2**, and the seam [`super::Action`] already draws twice:
//! [`super::dimensions`] is *what happens to the dimensioning model*,
//! [`super::pages`] is *what happens to a page*, [`super::annots`] is *what
//! happens to an annotation that already exists*. This is **what happens to the
//! marks on the page**.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/vector.md`.

use pdfcer_core::edit::{CommandKind, EditSession};

pub use pdfcer_gui_base::subactions::VectorAction;

/// **How many objects the page has, and how many parts one of them has** —
/// read once, for the trace line the three part-deletes write.
fn census(
    doc: &crate::app::state::OpenDoc,
    object: usize,
    parts: impl Fn(&crate::panels::objects::provider::ObjectModelProvider, usize) -> usize,
) -> (usize, usize) {
    doc.page_objects().map_or((0, 0), |provider| {
        (
            provider.page_objects().objects.len(),
            parts(&provider, object),
        )
    })
}

/// Write the census line for one part-delete.
///
/// `unit` is the operator's word for what was counted — `lines`, `runs`,
/// `points` — so a reader of the trace does not have to know which verb wrote
/// it to know what the number means. It reaches the diagnostic channel only;
/// `check-ui-strings.sh`'s exemption is on the literal below.
fn trace_part_delete(
    label: &'static str,
    unit: &'static str,
    page: usize,
    object: usize,
    part: usize,
    before: (usize, usize),
    after: (usize, usize),
) {
    crate::diag::trace(move || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        //
        // trace-name-exempt is NOT needed: every label here carries the
        // `-applied` suffix the naming convention requires, so none of them
        // collides with the funnel's bare `delete-subpath` / `delete-text-run`
        // / `delete-node`.
        format!(
            "{label} page={page} object={object} part={part} \
             objects_before={} objects_after={} {unit}_before={} {unit}_after={}",
            before.0, after.0, before.1, after.1,
        )
    });
}

/// Fold this gesture's `pieces` engine commands into one undo entry, and say so
/// when the fold does not happen.
fn fold_undo(
    session: &mut EditSession,
    pieces: usize,
    kind: CommandKind,
    disclosures: &mut Vec<String>,
) {
    let mut seen: Vec<String> = Vec::new();
    for sentence in disclosures.drain(..) {
        if !seen.contains(&sentence) {
            seen.push(sentence);
        }
    }
    *disclosures = seen;
    if !session.coalesce_last(pieces, kind) {
        disclosures.push(crate::text::arrange::line_takes_several_undo_presses(
            pieces,
        ));
    }
}

/// **Apply one geometry verb**, as one undoable command.
pub(super) fn apply(doc: &mut crate::app::state::OpenDoc, action: VectorAction) {
    // `vector_edit_on_page`, not `vector_edit` — `OPERATOR_REQUESTS.md`
    // O74. Every verb in this module addresses paint-order indices **into one
    // page's content stream**, which is the module header's own opening claim
    // and the property that makes the whole file a subject rather than a
    // size-driven cut. That property is exactly the one `EditScope::Page`
    // requires: it is true of the VERB, on every document and every operand,
    // not an observation about a particular call.
    //
    // So this is the strongest narrowing available anywhere in `actions`,
    // and it is the one the operator asked about — "when I make edits … all of
    // the page previews get re-rendered". A node drag on sheet 12 now leaves
    // the other thirty-five thumbnails alone.
    //
    // It stays bounded even if that reasoning is ever wrong: `pages::resync`
    // runs after the bump and raises `bump_all` whenever the page SET moved, so
    // a mistake here can be about which page's content changed and never about
    // which sheet an index names.
    use super::apply::vector_edit_on_page;
    match action {
        VectorAction::DeleteSelection { page, objects } => {
            if !objects.is_empty() {
                // THE FOOTPRINT, TAKEN BEFORE THE COMMIT DESTROYS IT.
                //
                // `OPERATOR_REQUESTS.md` O63. A deleted object stays on screen
                // until the page redraws — one to two seconds on a dense
                // drawing — with no gesture in flight to explain the wait. What
                // the operator sees is a delete that did nothing, and the
                // natural response is to press Delete again, which deletes
                // something else.
                //
                // The order is load-bearing. `page_objects` is keyed on
                // `(page, edit_epoch)` and `vector_edit` bumps the epoch, so the
                // geometry is thrown away by the very edit it describes. Built
                // here, and the `Ref` dropped before `vector_edit` takes
                // `&mut doc`.
                let preview = doc
                    .page_objects()
                    .and_then(|provider| crate::canvas::shapes::erased(&provider, &objects));
                // HELD BEFORE THE COMMIT, and getting this backwards is a
                // silent quarter-second bug.
                //
                // `hold_preview` stamps `edit_epoch` AS IT IS WHEN CALLED, and
                // `held_preview_to_draw` reads "the epoch has not moved" as
                // "the commit has not landed yet" — a state it allows for only
                // 250 ms, because past that it means the engine REFUSED.
                //
                // So a hold taken after the bump looks permanently un-committed
                // and expires in a quarter of a second, which on the drawings
                // this exists for is a fifth of the wait. Taken before, the
                // bump moves the epoch past it and the normal rule applies:
                // draw until the raster carries the edit.
                //
                // ⇒ And the refusal case comes out right for free. If
                // `delete_objects` fails the epoch never moves, the grace
                // expires, and the operator does not spend four seconds looking
                // at a hole where their object still is.
                if let Some(preview) = preview {
                    doc.hold_preview(preview);
                }
                vector_edit_on_page(doc, "delete-objects", page, objects.len(), |session| {
                    session.delete_objects(page, &objects)
                });
            }
        }
        VectorAction::MoveSelection {
            page,
            objects,
            dx,
            dy,
        } => {
            if !objects.is_empty() {
                vector_edit_on_page(doc, "move-objects", page, objects.len(), |session| {
                    session.move_objects(page, &objects, dx, dy)
                });
            }
        }
        VectorAction::DeleteLeavesInForm { page, leaves } => {
            if !leaves.is_empty() {
                // No shell-side invalidation here either — see
                // `MoveLeavesInForm`'s arm for the counter that stood in both
                // places for four hours and the engine commit that removed the
                // need for it.
                vector_edit_on_page(
                    doc,
                    "delete-leaves-in-form",
                    page,
                    leaves.len(),
                    |session| {
                        session
                            .delete_objects_in_form(page, &leaves)
                            .map(|outcome| outcome.disclosures)
                    },
                );
            }
        }
        // The three deeper-rung deletes, and the census line each writes.
        //
        // `-applied` suffixes, per `tools/gates/check-trace-names.py`: the
        // funnel writes `delete-subpath page=… n=… epoch=… disclosures=…` under
        // the bare label, and a module line sharing that first token would be
        // the one `Trace::last` returns — which is how a driven check reports
        // *"the verb did nothing"* about a verb that worked. Three recorded
        // instances; this is the convention that ended them.
        VectorAction::DeleteSubpath {
            page,
            object,
            subpath,
        } => {
            let before = census(doc, object, |p, o| p.subpath_count(o));
            vector_edit_on_page(doc, "delete-subpath", page, 1, |session| {
                session.delete_subpath(page, object, subpath)
            });
            let after = census(doc, object, |p, o| p.subpath_count(o));
            trace_part_delete(
                "delete-subpath-applied",
                "lines",
                page,
                object,
                subpath,
                before,
                after,
            );
        }
        VectorAction::DeleteTextLine { page, object, line } => {
            // The run range is read BEFORE `&mut doc` is taken, and the `Ref`
            // is dropped at the end of the statement. `None` means the line
            // index out-ran the decomposition — a stale selection, which does
            // nothing and earns no sentence, because nothing was refused.
            if let Some(runs) = doc
                .page_objects()
                .and_then(|provider| provider.text_line_runs(object, line))
            {
                let pieces = runs.len();
                let before = census(doc, object, |p, o| p.text_line_count(o));
                vector_edit_on_page(doc, "delete-text-line", page, pieces, |session| {
                    let mut disclosures = Vec::new();
                    // DESCENDING. `plan_delete_text_run` excises
                    // `TextRun::bytes`, so removing an earlier run shifts every
                    // later one in the same object. Walking backwards means no
                    // index this loop still holds has moved.
                    for run in runs.clone().rev() {
                        disclosures.extend(session.delete_text_run(page, object, run)?);
                    }
                    fold_undo(
                        session,
                        pieces,
                        CommandKind::DeleteTextRun,
                        &mut disclosures,
                    );
                    Ok::<_, pdfcer_core::edit::EditError>(disclosures)
                });
                let after = census(doc, object, |p, o| p.text_line_count(o));
                trace_part_delete(
                    "delete-text-line-applied",
                    "text-lines",
                    page,
                    object,
                    line,
                    before,
                    after,
                );
            }
        }
        // The disclosures are surfaced by the funnel, not here — see the
        // variant's own docs for why a second `record_note` beside it would be
        // the mechanism that forgets to retire itself.
        VectorAction::DeleteNode { page, object, node } => {
            let before = census(doc, object, |p, o| p.object_node_points(o).len());
            vector_edit_on_page(doc, "delete-node", page, 1, |session| {
                session.delete_node(page, object, node)
            });
            let after = census(doc, object, |p, o| p.object_node_points(o).len());
            trace_part_delete(
                "delete-node-applied",
                "points",
                page,
                object,
                node,
                before,
                after,
            );
        }
        VectorAction::MoveHandleInForm {
            page,
            leaf,
            node,
            handle,
            to,
        } => {
            vector_edit_on_page(doc, "move-handle-in-form", page, 1, |session| {
                session
                    .move_handle_in_form(page, leaf, node, handle, to)
                    .map(|outcome| outcome.disclosures)
            });
        }
        VectorAction::MoveSubpathInForm {
            page,
            leaf,
            subpath,
            dx,
            dy,
        } => {
            vector_edit_on_page(doc, "move-subpath-in-form", page, 1, |session| {
                session
                    .move_subpath_in_form(page, leaf, subpath, dx, dy)
                    .map(|outcome| outcome.disclosures)
            });
        }
        VectorAction::MoveNodeInForm {
            page,
            leaf,
            node,
            to,
        } => {
            vector_edit_on_page(doc, "move-node-in-form", page, 1, |session| {
                session
                    .move_node_in_form(page, leaf, node, to)
                    .map(|outcome| outcome.disclosures)
            });
        }
        VectorAction::MoveNodesInForm { page, leaf, moves } => {
            // The count is the number of ANCHORS, which is what the funnel's
            // trace reports as the operand size — one object, many points, and
            // the number a reader comparing a trace against a drag wants.
            let n = moves.len();
            vector_edit_on_page(doc, "move-nodes-in-form", page, n, |session| {
                session
                    .move_nodes_in_form(page, leaf, &moves)
                    .map(|outcome| outcome.disclosures)
            });
        }
        VectorAction::MoveLeavesInForm {
            page,
            leaves,
            dx,
            dy,
        } => {
            if !leaves.is_empty() {
                //
                // `pdfcer-core` `6e2b69e` folded the descended-form set into the
                // digest the same night, so the invalidation is computed on the
                // inside, where this shell said it belonged. The counter is
                // gone; nothing replaces it.
                vector_edit_on_page(doc, "move-leaves-in-form", page, leaves.len(), |session| {
                    session
                        .move_objects_in_form(page, &leaves, dx, dy)
                        .map(|outcome| outcome.disclosures)
                });
            }
        }
        VectorAction::MoveSubpath {
            page,
            object,
            subpath,
            dx,
            dy,
        } => {
            vector_edit_on_page(doc, "move-subpath", page, 1, |session| {
                session.move_subpath(page, object, subpath, dx, dy)
            });
        }
        VectorAction::MoveTextLine {
            page,
            object,
            line,
            dx,
            dy,
        } => {
            if let Some(runs) = doc
                .page_objects()
                .and_then(|provider| provider.text_line_runs(object, line))
            {
                let runs: Vec<usize> = runs.collect();
                vector_edit_on_page(doc, "move-text-line", page, runs.len(), |session| {
                    session.move_text_runs(page, object, &runs, dx, dy)
                });
            }
        }
        VectorAction::MoveTextLineInForm {
            page,
            leaf,
            line,
            dx,
            dy,
        } => {
            if let Some(runs) = doc.page_objects().and_then(|provider| {
                provider.text_line_runs_of(
                    crate::panels::objects::provider::TargetId::Leaf(leaf as u64),
                    line,
                )
            }) {
                let runs: Vec<usize> = runs.collect();
                vector_edit_on_page(doc, "move-text-line-in-form", page, runs.len(), |session| {
                    session
                        .move_text_runs_in_form(page, leaf, &runs, dx, dy)
                        .map(|outcome| outcome.disclosures)
                });
            }
        }
        VectorAction::MoveTextLines {
            page,
            object,
            lines,
            dx,
            dy,
        } => {
            // Every run of every line, resolved BEFORE the session opens.
            // `text_line_runs` reads the provider, which borrows the document;
            // resolving inside the edit closure would ask the model about a
            // revision the closure is in the middle of replacing.
            let runs: Vec<usize> = doc
                .page_objects()
                .map(|provider| {
                    lines
                        .iter()
                        .filter_map(|&line| provider.text_line_runs(object, line))
                        .flatten()
                        .collect()
                })
                .unwrap_or_default();
            if !runs.is_empty() {
                vector_edit_on_page(doc, "move-text-lines", page, runs.len(), |session| {
                    session.move_text_runs(page, object, &runs, dx, dy)
                });
            }
        }
        VectorAction::MoveTextLinesInForm {
            page,
            leaf,
            lines,
            dx,
            dy,
        } => {
            let target = crate::panels::objects::provider::TargetId::Leaf(leaf as u64);
            let runs: Vec<usize> = doc
                .page_objects()
                .map(|provider| {
                    lines
                        .iter()
                        .filter_map(|&line| provider.text_line_runs_of(target, line))
                        .flatten()
                        .collect()
                })
                .unwrap_or_default();
            if !runs.is_empty() {
                vector_edit_on_page(
                    doc,
                    "move-text-lines-in-form",
                    page,
                    runs.len(),
                    |session| {
                        session
                            .move_text_runs_in_form(page, leaf, &runs, dx, dy)
                            .map(|outcome| outcome.disclosures)
                    },
                );
            }
        }
        VectorAction::MoveNode {
            page,
            object,
            node,
            to,
        } => {
            vector_edit_on_page(doc, "move-node", page, 1, |session| {
                session.move_node(page, object, node, to)
            });
        }
        // A resize, and it is `move_nodes` because there is no scale
        // verb — see `VectorAction::MoveNodes` and `crate::canvas::resizing`.
        //
        // ONE call with every node in it, deliberately: the slice is what
        // makes a whole resize one command and one undo entry, and a loop
        // over `move_node` would be neither.
        //
        // The count passed to `vector_edit` is the number of NODES, which
        // is what its trace line reports as the operand size. That is the
        // honest figure for this edit — one object, many points — and it is
        // the number a reader comparing a trace against a drag would want.
        // A handle drag, and the only vector edit in this crate whose
        // RETURN VALUE is a disclosure rather than a count.
        //
        // `move_handle` answers with a list of sentences that is empty
        // unless a `v`/`y` segment had to be re-spelled as `c` — see the
        // variant's own docs. The curve draws identically either way, so
        // this is an inference the operator cannot see, which is exactly
        // the case rule 4 says still owes an off-canvas report.
        VectorAction::MoveHandle {
            page,
            object,
            node,
            handle,
            to,
        } => {
            //
            // It cloned the disclosure list out of the closure and then called
            // `record_note` once per sentence. `record_note` is the singular of
            // `record_notes`, and the slot holds **one** disclosure rather than
            // a queue: a second call REPLACES the first. So a two-sentence
            // answer showed only the second, chosen by loop order — which is
            // exactly the hazard `record_notes`' own doc comment was written
            // about, reproduced two files away from where it is described.
            //
            // ⇒ Returning the list from the closure is the whole of the job.
            // `vector_edit_on_page` records it, in one call, stamped with the
            // epoch the edit produced, for every verb in this module. That is
            // why `DeleteNode` below needs no code of its own for a disclosure
            // the engine's docs call mandatory.
            vector_edit_on_page(doc, "move-handle", page, 1, |session| {
                session.move_handle(page, object, node, handle, to)
            });
        }
        VectorAction::MoveNodes {
            page,
            object,
            moves,
        } => {
            let count = moves.len();
            vector_edit_on_page(doc, "move-nodes", page, count, |session| {
                session.move_nodes(page, object, &moves)
            });
        }
        // Move / resize / rotate anything. One call, one command, one
        // undo entry, whatever the selection is made of.
        //
        // `TransformOptions::default()` and no override, and both halves
        // of that are decisions the operator made himself before the
        // question reached him: *"make things work both ways as options.
        // default it to your best guess as to what would be normally
        // expected."* The engine shipped both, so the defaults are
        //
        //   * a **mixed selection transforms whole** rather than refusing —
        //     an operator who drags a grip round a picture and a box means
        //     both, and `RefuseHeterogeneous` exists for a caller that
        //     needs the other answer;
        //   * a **singular matrix refuses by name** rather than clamping —
        //     a clamp silently substitutes a shape nobody drew, and
        //     `SingularPolicy::Clamp` exists for a caller that wants it.
        //
        // A commit-on-release gesture makes exactly-zero scale nearly
        // unreachable anyway, and `resizing::is_usable` refuses before we
        // ever get here.
        //
        // `objects_transformed`, not `objects.len()`, is what the trace
        // reports — see the variant's docs for what the engine collapses and
        // why. A count taken from our own slice would be a number this
        // shell wished were true.
        // PASTE. The operator's oldest open request, and the arm is short
        // because the engine's clip owns everything it needs.
        //
        // The deserialisation is INSIDE the closure, so a payload that is not
        // a clip refuses through `vector_edit`'s own channel with the engine's
        // sentence — `ClipError::NotAClip`, checked before any length prefix is
        // read. A shell that unwrapped here would have to invent a sentence for
        // a case the engine already words.
        //
        // `resources_added` is on the trace at the engine's suggestion:
        // *"every paste adds fresh /Resources entries, so a shell that pastes
        // the same clip forty times and wonders why the file grew has the
        // answer in hand."* Not on the status row — it is a fact about the
        // file rather than about the page, and rule 4 asks for a disclosure in
        // terms of what the operator can see.
        // ⚠ A paste does not select what it pasted, while a place does
        // (`actions::apply` calls `select_placed`). `DEFECTS.md` D42.
        VectorAction::PasteObjects { page, clip, at } => {
            let mut added = 0_u64;
            let mut pasted = 0_u64;
            //
            // It is on the trace because it is the number a wrong build gets
            // wrong **invisibly**: a clip whose annotation payload the
            // serialiser dropped, or an index list that never reached
            // `copy_selection`, pastes its content perfectly and reports
            // `pasted=3` either way. `pasted=` counts content objects only, so
            // an annotation-only paste traces `pasted=0` — which is
            // indistinguishable from a paste that did nothing at all unless
            // this field is beside it.
            let mut annots = 0_u64;
            vector_edit_on_page(doc, "paste-objects", page, clip.len(), |session| {
                let clip = pdfcer_core::vector::ObjectClip::from_bytes(&clip)?;
                session.paste_objects(page, &clip, at).map(|outcome| {
                    added = outcome.resources_added;
                    pasted = outcome.objects_pasted;
                    annots = outcome.annotations_pasted;
                    outcome.disclosures
                })
            });
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "paste-objects-applied page={page} pasted={pasted} annots={annots} \
                     resources_added={added} at=[{:.4} {:.4} {:.4} {:.4} {:.2} {:.2}]",
                    at.a, at.b, at.c, at.d, at.e, at.f,
                )
            });
        }
        VectorAction::TransformObjects {
            page,
            objects,
            matrix,
        } => {
            let mut transformed = 0_u64;
            vector_edit_on_page(doc, "transform-objects", page, objects.len(), |session| {
                session
                    .transform_objects(
                        page,
                        &objects,
                        matrix,
                        pdfcer_core::vector::TransformOptions::default(),
                    )
                    .map(|outcome| {
                        transformed = outcome.objects_transformed;
                        outcome.disclosures
                    })
            });
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // It carries the MATRIX, and it has to. A line saying only
                // "a transform committed" would be identical for a build
                // that translated when it meant to scale, scaled about the
                // wrong pivot, or applied the transform in the object's
                // local space instead of the page's — which is the one error
                // here that lands the object at a plausible wrong distance
                // with nothing erroring. `resize-commit`'s own note makes
                // the same argument: a trace line must carry the number a
                // wrong build would get wrong.
                format!(
                    "transform-objects-applied page={page} asked={} \
                     transformed={transformed} m=[{:.4} {:.4} {:.4} {:.4} {:.2} {:.2}]",
                    objects.len(),
                    matrix.a,
                    matrix.b,
                    matrix.c,
                    matrix.d,
                    matrix.e,
                    matrix.f,
                )
            });
        }
        VectorAction::MergeTextRuns { page, object, runs } => {
            vector_edit_on_page(doc, "merge-text-runs", page, runs.len(), |session| {
                session
                    .merge_text_runs(
                        page,
                        object,
                        &runs,
                        &pdfcer_core::text_edit::merge::MergeOptions::default(),
                    )
                    .inspect_err(|e| {
                        crate::app::status::decline::record_run_merge(
                            crate::text::runmerge::RunMergeRefusal::of_format(e),
                        );
                    })
                    .map(|report| {
                        crate::diag::trace(|| {
                            // ui-text-exempt: diagnostic trace, never displayed.
                            format!(
                                "merge-text-runs-applied page={page} object={object} merged={} scale={:?}",
                                report.runs_merged, report.h_scale_change,
                            )
                        });
                        let mut said = report.disclosures;
                        if let (Some((before, after)), true) =
                            (report.h_scale_change, said.is_empty())
                        {
                            said.push(crate::text::runmerge::width_changed(before, after));
                        }
                        said
                    })
            });
        }
    }
}
