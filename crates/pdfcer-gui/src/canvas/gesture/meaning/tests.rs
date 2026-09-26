//! # `canvas::gesture::meaning` tests — the precedence table, driven as a table
//!
//!
//! ## The seam is the ordinary one, and the half it leaves behind is the
//! interesting one
//!
//! [`super`] is **one pure function and the two enums it decides between** —
//! `(tool, grip, dimension, three booleans, capabilities)` in, one meaning or
//! `None` out. That is what makes the whole precedence testable as a table, and
//! it is also why the tests are bulky: a table with seven inputs is exercised by
//! enumerating it, not by three cases.
//!
//! ⇒ So this is a split between **the rule** and **the enumeration of the
//! rule** — a real subject boundary rather than a size-driven cut. Nothing here
//! holds state, touches egui, or knows that frames exist, which is the same
//! property the parent has and the reason both can be read alone.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/gesture/meaning/tests.md`.

// The INNER attribute, not just the `mod tests;` declaration in the parent.
//
// `check-ui-strings.sh`'s exclusion 2b recognises a whole test file **from the
// file** rather than from its name, and it exists because the last R2 split of
// a test module reported 28 assertion messages as operator-facing copy. Without
// this line the gate is right that they are string literals and wrong that
// anybody will read one on screen — and the noise is the real hazard: 125 of
// the old shell's 140-hit floor were test assertions, which is what the
// exclusion was written to remove.
#![cfg(test)]

use super::*;

/// A press that landed on nothing, for the table-driven cases below.
fn probe(tool: CanvasTool) -> Press {
    Press {
        tool,
        grip: None,
        handle: None,
        dimension: None,
        annot_rotate: None,
        markup_node: None,
        markup_body: false,
        markup_grip: false,
        widget_body: false,
        widget_grip: false,
        zoom_armed: false,
    }
}

use crate::canvas::textedit::TextEditKind;

/// **The caret tool takes the click, leaves the drag, and needs
/// `edit_content`** — its whole rung, over every capability combination.
#[test]
fn the_caret_tool_clicks_for_a_caret_and_drags_for_a_box_on_edit_content_alone() {
    for edit_content in [false, true] {
        for author_markup in [false, true] {
            for author_measure in [false, true] {
                let mut caps = Capabilities::NONE;
                caps.edit_content = edit_content;
                caps.author_markup = author_markup;
                caps.author_measure = author_measure;
                for kind in [TextEditKind::Edit, TextEditKind::Add] {
                    let m = press_kind(probe(CanvasTool::TextEdit(kind)), caps);
                    assert_eq!(
                        m.click, edit_content,
                        "the caret needs `edit_content` and nothing else"
                    );
                    assert_eq!(
                        m.drag == Some(DragKind::TextBox),
                        edit_content,
                        "the box needs `edit_content` and nothing else — a mode that may not \
                         change page content must not offer a rectangle to type into"
                    );
                }
                // An armed region zoom does NOT take the box away, which
                // is the one interaction worth pinning: the zoom marquee
                // outranks a text SWEEP (`textsel`) and must not outrank a
                // tool the operator explicitly armed to author with.
                let zoomed = press_kind(
                    Press {
                        zoom_armed: true,
                        ..probe(CanvasTool::TextEdit(TextEditKind::Edit))
                    },
                    caps,
                );
                assert_eq!(zoomed.drag == Some(DragKind::TextBox), edit_content);
            }
        }
    }
}

// -----------------------------------------------------------------
// What a press means
// -----------------------------------------------------------------

/// **The armed markup tool outranks the grips and the region zoom.**
#[test]
fn an_armed_markup_tool_outranks_the_grips_and_the_region_zoom() {
    let armed = CanvasTool::Markup(MarkupKind::Rectangle);
    for grip in [None, Some(Grip::SouthEast), Some(Grip::Move)] {
        for zoom in [false, true] {
            assert_eq!(
                press_kind(
                    Press {
                        tool: armed,
                        grip,
                        handle: None,
                        dimension: None,
                        annot_rotate: None,
                        markup_node: None,
                        markup_body: false,
                        markup_grip: false,
                        widget_body: false,
                        widget_grip: false,
                        zoom_armed: zoom
                    },
                    Capabilities::FULL
                ),
                PressMeaning::dragging(DragKind::Markup(MarkupKind::Rectangle)),
                "grip={grip:?} zoom_armed={zoom}"
            );
        }
    }
}

/// …and with no markup armed, the precedence is exactly what it was before
/// the markup tool existed. Without this, the test above would pass on a
/// build where every press had become a markup.
#[test]
fn without_a_markup_tool_the_press_precedence_is_unchanged() {
    let select = CanvasTool::Select;
    let full = Capabilities::FULL;
    assert_eq!(
        press_kind(
            Press {
                tool: select,
                grip: Some(Grip::SouthEast),
                handle: None,
                dimension: None,
                annot_rotate: None,
                markup_node: None,
                markup_body: false,
                markup_grip: false,
                widget_body: false,
                widget_grip: false,
                zoom_armed: false
            },
            full
        ),
        PressMeaning::dragging(DragKind::Resize(Grip::SouthEast))
    );
    assert_eq!(
        press_kind(
            Press {
                tool: select,
                grip: Some(Grip::Move),
                handle: None,
                dimension: None,
                annot_rotate: None,
                markup_node: None,
                markup_body: false,
                markup_grip: false,
                widget_body: false,
                widget_grip: false,
                zoom_armed: false
            },
            full
        ),
        PressMeaning::dragging(DragKind::Move)
    );
    assert_eq!(
        press_kind(
            Press {
                tool: select,
                grip: None,
                handle: None,
                dimension: None,
                annot_rotate: None,
                markup_node: None,
                markup_body: false,
                markup_grip: false,
                widget_body: false,
                widget_grip: false,
                zoom_armed: true
            },
            full
        ),
        PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Zoom))
    );
    assert_eq!(
        press_kind(
            Press {
                tool: select,
                grip: None,
                handle: None,
                dimension: None,
                annot_rotate: None,
                markup_node: None,
                markup_body: false,
                markup_grip: false,
                widget_body: false,
                widget_grip: false,
                zoom_armed: false
            },
            full
        ),
        PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select))
    );
    // A grip beats an armed zoom, as it always did.
    assert_eq!(
        press_kind(
            Press {
                tool: select,
                grip: Some(Grip::SouthEast),
                handle: None,
                dimension: None,
                annot_rotate: None,
                markup_node: None,
                markup_body: false,
                markup_grip: false,
                widget_body: false,
                widget_grip: false,
                zoom_armed: true
            },
            full
        ),
        PressMeaning::dragging(DragKind::Resize(Grip::SouthEast))
    );
}

// -----------------------------------------------------------------
// The mode gate
// -----------------------------------------------------------------

/// **A mode that cannot edit content gives every content press no
/// meaning** — and leaves the region zoom alone.
#[test]
fn read_mode_gives_a_content_press_no_meaning_but_keeps_the_region_zoom() {
    let select = CanvasTool::Select;
    let read = Capabilities::NONE;
    // Asserted as the ABSENCE OF EVERY CONTENT MEANING rather than as
    // `PressMeaning::NOTHING`, and the change of shape is the point.
    //
    // `NOTHING` was the right assertion while Read had no press meaning at
    // all. It is the wrong one now, twice over: it would fail against the
    // feature the operator asked for, and — worse — a build that had taken
    // text selection away again would make it *pass*. What the operator
    // actually asked for is that nothing on the page can be selected,
    // moved or resized, so that is what is asserted, by naming the meanings
    // that must not appear.
    //
    // The grip rows are unreachable in practice (a grip is drawn only for a
    // content selection, which Read cannot make) and are checked anyway,
    // because "it is safe because nothing can be selected" is an argument
    // that holds only for as long as its other half does, and its other
    // half is in a different file, so this checks the relation rather than
    // the magnitude: the rows must stay absent however the other file changes.
    for grip in [None, Some(Grip::SouthEast), Some(Grip::Move)] {
        let meaning = press_kind(
            Press {
                tool: select,
                grip,
                handle: None,
                dimension: None,
                annot_rotate: None,
                markup_node: None,
                markup_body: false,
                markup_grip: false,
                widget_body: false,
                widget_grip: false,
                zoom_armed: false,
            },
            read,
        );
        assert!(
            !matches!(
                meaning.drag,
                Some(
                    DragKind::Resize(_)
                        | DragKind::Move
                        | DragKind::Marquee(MarqueeIntent::Select)
                        | DragKind::Markup(_)
                )
            ),
            "Read gave a content meaning to a press over {grip:?}: {meaning:?}"
        );
    }
    assert_eq!(
        press_kind(
            Press {
                tool: select,
                grip: None,
                handle: None,
                dimension: None,
                annot_rotate: None,
                markup_node: None,
                markup_body: false,
                markup_grip: false,
                widget_body: false,
                widget_grip: false,
                zoom_armed: false
            },
            read
        ),
        PressMeaning {
            drag: Some(DragKind::TextSelect),
            click: true,
        },
        "a bare press in a reading mode sweeps TEXT — never content"
    );
    assert_eq!(
        press_kind(
            Press {
                tool: CanvasTool::Markup(MarkupKind::Arrow),
                grip: None,
                handle: None,
                dimension: None,
                annot_rotate: None,
                markup_node: None,
                markup_body: false,
                markup_grip: false,
                widget_body: false,
                widget_grip: false,
                zoom_armed: false
            },
            read
        ),
        PressMeaning::NOTHING,
        "no markup, even with the tool somehow armed — and no text either, \
         because an armed pen keeps its own press"
    );
    assert_eq!(
        press_kind(
            Press {
                tool: select,
                grip: None,
                handle: None,
                dimension: None,
                annot_rotate: None,
                markup_node: None,
                markup_body: false,
                markup_grip: false,
                widget_body: false,
                widget_grip: false,
                zoom_armed: true
            },
            read
        ),
        PressMeaning {
            drag: Some(DragKind::Marquee(MarqueeIntent::Zoom)),
            click: true,
        },
        "a region zoom is navigation and survives every mode; it outranks the \
         text sweep because the operator armed it"
    );
}

/// **No press ever means both a text sweep and a content marquee.**
#[test]
fn no_press_offers_both_a_text_sweep_and_a_content_marquee() {
    for caps in [
        Capabilities::NONE,
        Capabilities::FULL,
        Capabilities {
            edit_content: false,
            author_markup: true,
            author_measure: true,
        },
    ] {
        let text = matches!(
            press_kind(
                Press {
                    tool: CanvasTool::Select,
                    grip: None,
                    handle: None,
                    dimension: None,
                    annot_rotate: None,
                    markup_node: None,
                    markup_body: false,
                    markup_grip: false,
                    widget_body: false,
                    widget_grip: false,
                    zoom_armed: false
                },
                caps
            )
            .drag,
            Some(DragKind::TextSelect)
        );
        let content = matches!(
            press_kind(
                Press {
                    tool: CanvasTool::Select,
                    grip: None,
                    handle: None,
                    dimension: None,
                    annot_rotate: None,
                    markup_node: None,
                    markup_body: false,
                    markup_grip: false,
                    widget_body: false,
                    widget_grip: false,
                    zoom_armed: false
                },
                caps
            )
            .drag,
            Some(DragKind::Marquee(MarqueeIntent::Select))
        );
        assert!(text ^ content, "exactly one, for {caps:?}");

        // …and with the tool armed, the answer is text in every one of them,
        // whatever the pointer is over. The grip rows are what a build that
        // put the new rung *below* the content branch would fail — silently,
        // and only in Edit.
        for grip in [None, Some(Grip::SouthEast), Some(Grip::Move)] {
            assert_eq!(
                press_kind(
                    Press {
                        tool: CanvasTool::Text,
                        grip,
                        handle: None,
                        dimension: None,
                        annot_rotate: None,
                        markup_node: None,
                        markup_body: false,
                        markup_grip: false,
                        widget_body: false,
                        widget_grip: false,
                        zoom_armed: false
                    },
                    caps
                ),
                PressMeaning {
                    drag: Some(DragKind::TextSelect),
                    click: true,
                },
                "an armed text tool sweeps text over {grip:?} in {caps:?}"
            );
        }
    }
}

/// **The armed text tool takes the press in EDIT** — the row the whole
/// tool exists for, asserted by itself so that a failure names it.
#[test]
fn the_text_tool_sweeps_in_edit_and_retiring_it_gives_the_marquee_back() {
    let edit = Capabilities::FULL;
    assert_eq!(
        press_kind(
            Press {
                tool: CanvasTool::Text,
                grip: None,
                handle: None,
                dimension: None,
                annot_rotate: None,
                markup_node: None,
                markup_body: false,
                markup_grip: false,
                widget_body: false,
                widget_grip: false,
                zoom_armed: false
            },
            edit
        ),
        PressMeaning {
            drag: Some(DragKind::TextSelect),
            click: true,
        },
        "Edit is the mode this tool was built for"
    );
    assert_eq!(
        press_kind(
            Press {
                tool: CanvasTool::Select,
                grip: None,
                handle: None,
                dimension: None,
                annot_rotate: None,
                markup_node: None,
                markup_body: false,
                markup_grip: false,
                widget_body: false,
                widget_grip: false,
                zoom_armed: false
            },
            edit
        ),
        PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select)),
        "…and putting it down restores the content marquee unchanged"
    );
    // A resize grip is still a resize with the tool down — the precedence
    // below rung 2 is untouched.
    assert_eq!(
        press_kind(
            Press {
                tool: CanvasTool::Select,
                grip: Some(Grip::SouthEast),
                handle: None,
                dimension: None,
                annot_rotate: None,
                markup_node: None,
                markup_body: false,
                markup_grip: false,
                widget_body: false,
                widget_grip: false,
                zoom_armed: false
            },
            edit
        ),
        PressMeaning::dragging(DragKind::Resize(Grip::SouthEast)),
    );
}

/// **An armed region zoom outranks the armed text tool, and an armed pen
/// outranks the zoom.**
#[test]
fn a_region_zoom_outranks_the_text_tool_but_not_a_pen() {
    for caps in [Capabilities::NONE, Capabilities::FULL] {
        assert_eq!(
            press_kind(
                Press {
                    tool: CanvasTool::Text,
                    grip: None,
                    handle: None,
                    dimension: None,
                    annot_rotate: None,
                    markup_node: None,
                    markup_body: false,
                    markup_grip: false,
                    widget_body: false,
                    widget_grip: false,
                    zoom_armed: true
                },
                caps
            ),
            PressMeaning {
                drag: Some(DragKind::Marquee(MarqueeIntent::Zoom)),
                click: true,
            },
            "the zoom is a one-shot the operator armed; the text tool is back next press \
             ({caps:?})"
        );
    }
    assert_eq!(
        press_kind(
            Press {
                tool: CanvasTool::Markup(MarkupKind::Rectangle),
                grip: None,
                handle: None,
                dimension: None,
                annot_rotate: None,
                markup_node: None,
                markup_body: false,
                markup_grip: false,
                widget_body: false,
                widget_grip: false,
                zoom_armed: true
            },
            Capabilities::FULL
        ),
        PressMeaning::dragging(DragKind::Markup(MarkupKind::Rectangle)),
        "a pen still outranks the zoom, because a mark that is never made is a silent loss"
    );
}

/// **A vertex markup tool takes the CLICK and offers no drag** — the row
/// added on 2026-08-14, and the one a build that folded PolyLine into the
/// band rung would fail.
#[test]
fn a_vertex_markup_tool_takes_the_click_and_offers_no_drag() {
    let review = Capabilities {
        edit_content: false,
        author_markup: true,
        author_measure: true,
    };
    for kind in [MarkupKind::PolyLine, MarkupKind::Polygon] {
        let armed = CanvasTool::Markup(kind);
        for grip in [None, Some(Grip::SouthEast), Some(Grip::Move)] {
            for zoom in [false, true] {
                for caps in [review, Capabilities::FULL] {
                    assert_eq!(
                        press_kind(
                            Press {
                                tool: armed,
                                grip,
                                handle: None,
                                dimension: None,
                                annot_rotate: None,
                                markup_node: None,
                                markup_body: false,
                                markup_grip: false,
                                widget_body: false,
                                widget_grip: false,
                                zoom_armed: zoom
                            },
                            caps
                        ),
                        PressMeaning::clicking(),
                        "{kind:?} grip={grip:?} zoom={zoom} {caps:?}"
                    );
                }
            }
        }
        // …and a mode that cannot author markup gives it nothing at all,
        // which is the same answer an armed band kind gets in Read.
        assert_eq!(
            press_kind(
                Press {
                    tool: armed,
                    grip: None,
                    handle: None,
                    dimension: None,
                    annot_rotate: None,
                    markup_node: None,
                    markup_body: false,
                    markup_grip: false,
                    widget_body: false,
                    widget_grip: false,
                    zoom_armed: false
                },
                Capabilities::NONE
            ),
            PressMeaning::NOTHING,
            "{kind:?} in a mode that authors no markup"
        );
    }
    // The freehand kind is the other half of the same routing rule and must
    // go the OTHER way: Ink is drag-shaped, so it keeps the band rung's
    // answer. A build that classified by "is it in the new set of kinds?"
    // rather than by the gesture would break exactly here.
    assert_eq!(
        press_kind(
            Press {
                tool: CanvasTool::Markup(MarkupKind::Ink),
                grip: None,
                handle: None,
                dimension: None,
                annot_rotate: None,
                markup_node: None,
                markup_body: false,
                markup_grip: false,
                widget_body: false,
                widget_grip: false,
                zoom_armed: false
            },
            Capabilities::FULL
        ),
        PressMeaning::dragging(DragKind::Markup(MarkupKind::Ink)),
        "freehand is a DRAG, whatever else it shares with the vertex kinds"
    );
}

/// **Review places markup and does not touch content** — the middle row
/// of `MODES_AND_PANELS.md`'s gesture table, which is the row that proves
/// the gate is per-capability rather than a single on/off.
#[test]
fn review_mode_places_markup_but_refuses_content() {
    let review = Capabilities {
        edit_content: false,
        author_markup: true,
        author_measure: true,
    };
    assert_eq!(
        press_kind(
            Press {
                tool: CanvasTool::Markup(MarkupKind::Rectangle),
                grip: None,
                handle: None,
                dimension: None,
                annot_rotate: None,
                markup_node: None,
                markup_body: false,
                markup_grip: false,
                widget_body: false,
                widget_grip: false,
                zoom_armed: false
            },
            review
        ),
        PressMeaning {
            drag: Some(DragKind::Markup(MarkupKind::Rectangle)),
            click: false,
        },
        "a reviewer draws their own markup, and their click selects nothing"
    );
    assert!(
        !matches!(
            press_kind(
                Press {
                    tool: CanvasTool::Select,
                    grip: Some(Grip::Move),
                    handle: None,
                    dimension: None,
                    annot_rotate: None,
                    markup_node: None,
                    markup_body: false,
                    markup_grip: false,
                    widget_body: false,
                    widget_grip: false,
                    zoom_armed: false
                },
                review
            )
            .drag,
            Some(DragKind::Move | DragKind::Resize(_))
        ),
        "a reviewer does not move the page's own content"
    );
    assert_eq!(
        press_kind(
            Press {
                tool: CanvasTool::Select,
                grip: None,
                handle: None,
                dimension: None,
                annot_rotate: None,
                markup_node: None,
                markup_body: false,
                markup_grip: false,
                widget_body: false,
                widget_grip: false,
                zoom_armed: false
            },
            review
        ),
        PressMeaning {
            drag: Some(DragKind::TextSelect),
            click: true,
        },
        "…and does not marquee-select it either — with the pen down, a \
         reviewer's bare press sweeps text, which is what an underline or a \
         strikeout will need"
    );
}

// -----------------------------------------------------------------
// The rotate handle of a selected annotation
// -----------------------------------------------------------------

/// **A press on a selected MARKUP's rotate handle is a rotate, in
/// REVIEW** — the mode markup is authored in and the mode `edit_content` is
/// false in.
///
#[test]
fn a_markups_rotate_handle_turns_it_in_review() {
    let review = Capabilities {
        edit_content: false,
        author_markup: true,
        author_measure: true,
    };
    assert_eq!(
        press_kind(
            Press {
                grip: Some(Grip::Rotate),
                annot_rotate: Some(RotatableAnnot::Markup),
                ..probe(CanvasTool::Select)
            },
            review
        )
        .drag,
        Some(DragKind::Rotate),
        "a markup turns in the mode it is drawn in"
    );
}

/// **A ce dimension's rotate handle turns it, and it needs
/// `author_measure` rather than `author_markup`.**
#[test]
fn a_dimensions_rotate_handle_is_gated_on_measure_not_markup() {
    let measure_only = Capabilities {
        edit_content: false,
        author_markup: false,
        author_measure: true,
    };
    let markup_only = Capabilities {
        edit_content: false,
        author_markup: true,
        author_measure: false,
    };
    let press = Press {
        grip: Some(Grip::Rotate),
        annot_rotate: Some(RotatableAnnot::CeDimension),
        ..probe(CanvasTool::Select)
    };
    assert_eq!(
        press_kind(press, measure_only).drag,
        Some(DragKind::Rotate),
        "turning a dimension writes the measurement sidecar and one annotation and touches no \
         page content, so it is a MEASURE edit — the same ruling the vertex drag ships under"
    );
    assert_ne!(
        press_kind(press, markup_only).drag,
        Some(DragKind::Rotate),
        "…and `author_markup` alone must not reach it: a mode that may comment on a drawing is \
         not thereby a mode that may re-orient its dimensions"
    );
    assert_ne!(
        press_kind(
            Press {
                annot_rotate: Some(RotatableAnnot::Markup),
                ..press
            },
            measure_only
        )
        .drag,
        Some(DragKind::Rotate),
        "and the gate runs the other way too — `author_measure` alone does not turn a markup"
    );
}

/// **The handle claims the press ABOVE the two annotation rungs below it,
/// and above `edit_content`.**
#[test]
fn the_handle_outranks_the_content_branch_in_edit() {
    assert_eq!(
        press_kind(
            Press {
                grip: Some(Grip::Rotate),
                annot_rotate: Some(RotatableAnnot::Markup),
                // The body flag is FALSE by construction — the handle is
                // outside the `/Rect` — which is precisely why the markup rung
                // below cannot claim this press and why this rung had to exist.
                markup_body: false,
                markup_grip: false,
                ..probe(CanvasTool::Select)
            },
            Capabilities::FULL
        )
        .drag,
        Some(DragKind::Rotate)
    );
}

/// **And with no annotation selected the handle still belongs to page
/// content**, which is the assertion that stops the test above passing on a
/// build where every `Grip::Rotate` became an annotation rotation.
#[test]
fn without_an_annotation_the_handle_is_still_the_content_rotate() {
    assert_eq!(
        press_kind(
            Press {
                grip: Some(Grip::Rotate),
                annot_rotate: None,
                markup_node: None,
                ..probe(CanvasTool::Select)
            },
            Capabilities::FULL
        )
        .drag,
        Some(DragKind::Rotate),
        "the content rotate is unchanged; what is new is a second route to the same DragKind"
    );
}

/// **A press on one of a markup's RESIZE GRIPS scales it — in Review,
/// where the content branch does not run.**
#[test]
fn a_grip_outside_a_markups_box_still_resizes_it_in_review() {
    let review = Capabilities {
        edit_content: false,
        author_markup: true,
        author_measure: true,
    };
    for grip in [
        Grip::NorthEast,
        Grip::NorthWest,
        Grip::SouthEast,
        Grip::SouthWest,
        Grip::North,
        Grip::East,
    ] {
        assert_eq!(
            press_kind(
                Press {
                    grip: Some(grip),
                    markup_body: false,
                    markup_grip: true,
                    ..probe(CanvasTool::Select)
                },
                review
            )
            .drag,
            Some(DragKind::Resize(grip)),
            "a press on the {grip:?} grip of a selected markup, outside its box, in the mode \
             markup is authored in"
        );
    }
}

/// **…and the body still means MOVE**, which is what makes the two flags
/// two rather than one.
#[test]
fn the_body_of_a_markup_still_means_move_not_resize() {
    let review = Capabilities {
        edit_content: false,
        author_markup: true,
        author_measure: true,
    };
    assert_eq!(
        press_kind(
            Press {
                grip: Some(Grip::Move),
                markup_body: true,
                markup_grip: false,
                ..probe(CanvasTool::Select)
            },
            review
        )
        .drag,
        Some(DragKind::Move)
    );
}

/// **A grip flag alone does not license a drag: the capability still
/// gates it.**
#[test]
fn a_markup_grip_in_a_mode_that_authors_no_markup_resizes_nothing() {
    assert_ne!(
        press_kind(
            Press {
                grip: Some(Grip::NorthEast),
                markup_body: false,
                markup_grip: true,
                ..probe(CanvasTool::Select)
            },
            Capabilities::NONE
        )
        .drag,
        // `assert_ne!` against the resize rather than `assert_eq!` against
        // `None`, and the difference is a real fact about Read: a press there
        // is a **text sweep**, so the honest assertion is *"it is not a
        // resize"*, not *"it means nothing"*. Writing the stronger form first
        // failed with `Some(TextSelect)` — the reading branch answering
        // correctly — which would have been a test forcing the mode to stop
        // selecting text in order to go green.
        Some(DragKind::Resize(Grip::NorthEast)),
        "Read may not author markup, so it may not scale one either"
    );
}

/// **The same repair for a form field's box, and the same gate.**
#[test]
fn a_grip_outside_a_form_fields_box_still_resizes_it_in_edit() {
    assert_eq!(
        press_kind(
            Press {
                grip: Some(Grip::SouthEast),
                widget_body: false,
                widget_grip: true,
                ..probe(CanvasTool::Select)
            },
            Capabilities::FULL
        )
        .drag,
        Some(DragKind::Resize(Grip::SouthEast))
    );
    assert_ne!(
        press_kind(
            Press {
                grip: Some(Grip::SouthEast),
                widget_body: false,
                widget_grip: true,
                ..probe(CanvasTool::Select)
            },
            Capabilities {
                edit_content: false,
                author_markup: true,
                author_measure: true,
            }
        )
        .drag,
        Some(DragKind::Resize(Grip::SouthEast)),
        "a field's box is an Edit-only surface — Review may not scale one"
    );
}
