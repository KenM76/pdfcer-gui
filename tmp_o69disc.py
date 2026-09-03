import io


def patch(path, pairs):
    s = io.open(path, encoding='utf-8').read()
    for old, new in pairs:
        if old not in s:
            raise SystemExit('MISS in %s: %r' % (path, old[:140]))
        s = s.replace(old, new, 1)
    io.open(path, 'w', encoding='utf-8').write(s)
    print('patched', path)


# ---- the sentence for the rung that had none ---------------------------
patch('crates/pdfcer-gui/src/text/status/selection.rs', [(
    '''#[cfg(test)]''',
    '''/// ★★★ **The cap fired on a PART, and nothing was said** —
/// `OPERATOR_REQUESTS.md` O69: *"the nodes are hard to see and click on."*
///
/// The sibling of [`crate::text::status::too_many_anchors`], and it exists
/// because that one's guard excluded the exact route the operator reported.
///
/// # What he saw, and why it read as broken rather than as limited
///
/// The Points tool puts the selection at the **Part** rung, so
/// `entered_object()` is `Some` — and the disclosure was gated on it being
/// `None`. A subpath with more than four hundred anchors therefore drew no
/// dots and said nothing: he armed the tool, clicked a shape, watched the
/// selection box change, and the program went quiet. A limit reported as an
/// absence is the failure `RESUME.md` records four separate occasions of.
///
/// # ★★ Why it is not the same sentence
///
/// [`crate::text::status::too_many_anchors`] ends *"Double-click into a part
/// of it, or use the Points tool, to see that part's"* — advice that is
/// correct at the Object rung and **wrong here**, because there is nothing
/// below a subpath to descend into. Reusing it would send him looking for a
/// rung that does not exist.
///
/// The remedy this one names is the one that now works: **zoom in**. Since
/// 2026-08-31 the cap counts what is on screen rather than what the path
/// contains, so magnifying the area genuinely makes the dots appear — which
/// it did not before, and which is why this sentence could not have been
/// written honestly until the cull shipped.
///
/// ★ It lives in `text::status::selection` rather than beside its sibling in
/// `text::status` because that module is at 1,482 lines against R2's 1,500.
/// The seam is noticed rather than trimmed, which is that file's own standing
/// note.
#[must_use]
pub fn too_many_anchors_in_part(count: usize, cap: usize) -> String {
    format!(
        "This part has {count} points and pdfcer draws at most {cap} at a time, so none are \\
         shown here. Zoom in to see the ones you are looking at."
    )
}

#[cfg(test)]'''),
])

# ---- the gate ----------------------------------------------------------
patch('crates/pdfcer-gui/src/canvas/painting.rs', [(
    '''    // ★ Only when the operator ASKED — `show_points` on — and not on the
    // descent path, where the cap has always fired silently and where the
    // operator's subject is the subpath they entered rather than the whole
    // object.
    if doc.view.show_points
        && selection.entered_object().is_none()
        && anchors.len() > crate::canvas::overlay::MAX_UNSELECTED_ANCHORS
    {
        crate::app::actions::record_note(
            doc.edit_epoch,
            crate::text::status::too_many_anchors(
                anchors.len(),
                crate::canvas::overlay::MAX_UNSELECTED_ANCHORS,
            ),
        );
    }''',
    '''    // ★★★ **Disclosed at EVERY rung since 2026-08-31** —
    // `OPERATOR_REQUESTS.md` O69.
    //
    // This used to read `doc.view.show_points && entered_object().is_none()`,
    // with the note *"not on the descent path, where the cap has always fired
    // silently"*. That note described the defect and treated it as a decision.
    //
    // The Points tool puts the selection at the **Part** rung, so
    // `entered_object()` is `Some` — which means the one route the operator
    // was reporting was the one route excluded. He armed the tool, clicked a
    // dense contour, and got no dots and no sentence. A limit reported as an
    // absence reads as a broken program, and this project has paid for that
    // confusion four times.
    //
    // ★ Two sentences, because the remedy differs by rung. At the Object rung
    // *"descend into a part"* is right; at the Part rung there is nothing below
    // a subpath and the remedy is to zoom, which the viewport cull shipped in
    // the same commit made true.
    //
    // ★★ `show_points` is no longer required. It gates whether the operator
    // ASKED to see an object's points, which is the right question for the
    // Object rung and the wrong one for the Points tool — arming that tool IS
    // the ask.
    let capped = anchors.len() > crate::canvas::overlay::MAX_UNSELECTED_ANCHORS;
    let at_object_rung = selection.entered_object().is_none();
    if capped && (doc.view.show_points || !at_object_rung) {
        let note = if at_object_rung {
            crate::text::status::too_many_anchors(
                anchors.len(),
                crate::canvas::overlay::MAX_UNSELECTED_ANCHORS,
            )
        } else {
            crate::text::status::selection::too_many_anchors_in_part(
                anchors.len(),
                crate::canvas::overlay::MAX_UNSELECTED_ANCHORS,
            )
        };
        crate::app::actions::record_note(doc.edit_epoch, note);
    }'''),
])
