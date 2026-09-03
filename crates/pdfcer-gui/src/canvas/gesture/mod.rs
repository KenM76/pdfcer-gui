//! # `canvas::gesture` — press, drag, release, and the clear that must not happen on a press
//!
//! This file is the **state machine**: one [`PointerFrame`] in, one
//! [`GestureOutcome`] out, and a single `Option` of carried state between
//! frames. What a press *means* — the [`DragKind`] it produces, the
//! [`MarqueeIntent`] a rubber band carries to its release, and the precedence
//! that decides between them when more than one meaning is available — is a
//! pure decision function and lives next door in `meaning.rs`
//! (`canvas::gesture::meaning`). It is re-exported from here, so
//! `canvas::gesture::press_kind` and `canvas::gesture::DragKind` still name it
//! from this module and no caller has to know about the split.
//!
//! The division is by subject, not by size: `meaning.rs` answers *"what does
//! this press mean?"* from `(tool, grip, zoom_armed, capabilities)` with no
//! state at all, and is where marquee-select-versus-marquee-zoom and the
//! per-mode gate on a press are documented. This file answers *"what is
//! happening to that meaning now?"* across the frames of one gesture — the
//! press that decides nothing, the drag in flight, the release that commits,
//! and the Escape or interruption that abandons.
//!
//! ## ★ Invariant 2, and it lives entirely in this file
//!
//! `GUI_ROADMAP.md` Phase 1, the second of the three ways a selection model
//! loses *"selection survives navigation"*:
//!
//! > **Selection cleared by a click that was really a drag.** A pan gesture
//! > begins with a press on the canvas. If press-on-empty clears the
//! > selection, every pan that starts on blank paper destroys it. The clear
//! > must be driven by a *completed click* with no drag, not by a press.
//!
//! [`GestureState::update`] returns [`GestureOutcome::Idle`] on the press
//! frame — always, unconditionally, whatever the press landed on. A press
//! records where the gesture began and does nothing else. Only a *completed*
//! interaction produces an outcome, and only [`GestureOutcome::Click`] can
//! reach [`crate::canvas::selection::SelectionState::click`].
//!
//! The distinction is egui's to make and it already makes it correctly:
//! `Response::clicked()` is true for a press-and-release that did **not**
//! exceed the drag threshold, and `drag_started`/`dragged`/`drag_stopped` are
//! true for one that did. The two are mutually exclusive on one interaction.
//! What this module adds is the guarantee that **nothing else is consulted**
//! — in particular not `is_pointer_button_down_on`, which is true on the
//! press frame and is exactly how the defect above gets written.
//!
//! ## Primary button only, and why every canvas gesture must say so
//!
//! `Response::drag_started()` is button-agnostic: it is true for a middle-
//! and a right-drag as well as a left one. That is harmless until the middle
//! button means something — and here it means **pan**. A pan read as a
//! selection gesture would make dragging across a drawing replace the
//! selection, or, once a move verb is wired, silently rewrite the page.
//!
//! So the canvas reads `..._by(PointerButton::Primary)` and this module never
//! sees any other button. The right button is excluded for the same reason,
//! before the context menus of Phase 1.1 give it a job.
//!
//! ## Marquee versus pan: settled by the button and the tool, not by a heuristic
//!
//! The old shell left this open (*"a drag starting on empty canvas is
//! ambiguous between pan and marquee-select"*). It is not ambiguous here, and
//! it was decided at S0 rather than now: `canvas/mod.rs` switches egui's
//! button-agnostic drag-to-scroll **off** and implements panning against the
//! scroll offset on the middle button, with the stated reason *"the left
//! button is reserved for the selection marquee that arrives at S4"*. Left
//! drags marquee; middle drags pan; neither can be mistaken for the other,
//! and no distance threshold or modal state is involved.
//!
//! Phase 3.2 adds the hand tool and space-to-pan, which give the *primary*
//! button a second meaning — and the resolution keeps the same shape. The hand
//! tool is not a third `DragKind`: when [`crate::canvas::tool::active`] says
//! `Hand`, `canvas/mod.rs` hands this machine a **blank** [`PointerFrame`], so
//! a pan is not a gesture this module can see, let alone one it could confuse
//! with a marquee. One state machine, one meaning per frame, and the branch is
//! in one `if` at the boundary rather than a flag threaded through every arm.

mod meaning;
/// What a gesture PRODUCED — the phase, the outcome vocabulary, and the
/// in-flight drag that turns one into the other. Its header carries the seam.
mod outcome;

use outcome::Drag;
pub use outcome::*;

pub use meaning::{
    DimensionPress, DragKind, MarqueeIntent, Press, PressMeaning, RotatableAnnot, press_kind,
};

use egui::Pos2;

use crate::canvas::handles::Grip;

/// What the pointer did over the page this frame, already converted to
/// **canvas space**.
///
/// Assembled in `canvas/mod.rs` from one egui [`egui::Response`] and handed
/// here as plain data, which is what makes the whole state machine testable
/// without a window. Every field is a question egui has already answered; the
/// value of naming them is that the *set* is closed — a future gesture that
/// wants some other signal has to add it here, in front of this module's
/// docs, rather than reaching into a `Response` at a call site.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PointerFrame {
    /// A primary-button drag began this frame.
    pub drag_started: bool,
    /// A primary-button drag is in flight.
    pub dragging: bool,
    /// A primary-button drag ended this frame.
    pub drag_stopped: bool,
    /// A primary-button **click** completed this frame — pressed and
    /// released without exceeding the drag threshold.
    pub clicked: bool,
    /// The completed click was the second of a double-click.
    pub double_clicked: bool,
    /// The completed click was the **third** of a triple-click.
    ///
    /// Mutually exclusive with [`Self::double_clicked`] — egui counts clicks and
    /// reports `is_double()` as `count == 2` against `is_triple()`'s `count ==
    /// 3`, so exactly one of the two can be set on any one release. That is what
    /// lets `canvas::textsel::click` test them in order without the third click
    /// of a triple also re-selecting a word.
    ///
    /// Read only by the text gesture. A triple-click means nothing to the
    /// selection ladder (which descends one rung per double-click) or to the
    /// measure tools (whose double-click is the radius/diameter tool's *ending*,
    /// and a triple would be a second ending nobody asked for), so both continue
    /// to see it as an ordinary click.
    pub triple_clicked: bool,
    /// Where the pointer is, in canvas space, if it is anywhere the canvas
    /// can see. `None` for a gesture whose pointer has left the window.
    pub pos: Option<Pos2>,
    /// **Where the button actually went down**, in canvas space — the corner
    /// the operator chose.
    ///
    /// # ★ Why this exists, and the defect it closes
    ///
    /// `Response::drag_started()` does **not** fire on the frame of the press.
    /// It fires once the pointer has travelled far enough for egui to call the
    /// interaction a drag rather than a click — which is the right rule, and it
    /// is what makes [`GestureOutcome::Click`] and a drag mutually exclusive.
    /// But by then `interact_pointer_pos()` reports where the pointer has
    /// **already travelled to**, so a gesture anchored on it starts short of
    /// the press by however far the hand moved in that first interval.
    ///
    /// **Measured on this build, not reasoned about.** An arrow drawn on
    /// `fixtures/a1-titleblock.pdf` at zoom 0.2131 through a real OS-injected
    /// drag reported its tail at PDF `(807.18, 649.37)` when the button had
    /// gone down at `(713.3, 588.4)` — the shape began **94 points** along the
    /// drag from the corner the operator picked. The magnitude is
    /// `first-interval travel ÷ zoom`, so it is worst exactly where it is least
    /// forgivable: on a large sheet zoomed out to see all of it. The old shell
    /// measured the same thing from the other end (`main.rs:19716`) — a drag
    /// that should have spanned 50.5 points produced 42.0 — and fixed it the
    /// same way.
    ///
    /// It is carried on the frame rather than read inside
    /// [`GestureState::update`] for the reason every other signal here is: this
    /// module is drivable with no window, and a hidden read of
    /// `egui::InputState` would take that away. `None` is the honest answer for
    /// a frame that has no press behind it, and [`GestureState::update`] falls
    /// back to [`Self::pos`] — which is exactly the previous behaviour, so a
    /// caller that does not supply it loses accuracy and never correctness.
    ///
    /// All four drag kinds get the fix, not just the markup band: a marquee
    /// that starts late encloses less than the operator drew round, and a move
    /// whose origin is late under-moves the object by the same distance.
    pub press_origin: Option<Pos2>,
    /// Whether Shift was held. Read once, here, so every gesture agrees about
    /// what "extend" means.
    pub shift: bool,
    /// **Escape was pressed this frame**, and the canvas is entitled to it —
    /// i.e. no text field has focus.
    ///
    /// # ★ Why the abort arrives as an input rather than as a method call
    ///
    /// Because a drag in flight and the selection ladder both want Escape, and
    /// exactly one of them may have it per press. Routing the key through the
    /// same `PointerFrame` every other signal arrives on is what makes the
    /// precedence a single readable branch at the top of
    /// [`GestureState::update`] — the drag wins, and it says so by returning
    /// [`GestureOutcome::Cancelled`], which is the caller's cue to leave the
    /// ladder alone this frame.
    ///
    /// A `GestureState::cancel()` method would have worked and would have put
    /// the precedence at the *call site*, where the next reader has to
    /// reconstruct it from two `if`s in different functions. That is how
    /// "Escape cancels the drag AND ascends a rung" ships.
    pub cancel: bool,
}

/// The canvas's pointer-gesture state between frames.
///
/// One `Option`. Everything else is derived from the frame's own signals,
/// which is deliberate: gesture state that outlives its gesture is how a
/// canvas ends up in a mode the operator cannot see and cannot leave.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GestureState {
    drag: Option<Drag>,
}

impl GestureState {
    /// Advance the machine by one frame.
    ///
    /// `press_kind` is consulted **only** on the frame a drag starts; on
    /// every other frame it is ignored, so the caller may compute it however
    /// cheaply it likes without worrying about which frame it is.
    ///
    /// # The order of the branches is the invariant
    ///
    /// 0. **Escape abandons a drag in flight**, and only then. `Option::take`
    ///    does both halves in one expression: it clears the gesture and
    ///    reports whether there was one to clear, so an Escape with no drag
    ///    under it changes nothing here and reaches the ladder untouched.
    /// 1. **A press starts a drag and returns `Idle`.** Nothing else. No hit
    ///    test, no clear, no selection change. This is invariant 2.
    /// 2. **A drag in flight owns the frame**, so a stray `clicked` cannot be
    ///    read out of the middle of one.
    /// 3. **A completed click** is the only thing that reaches the selection
    ///    by hit test.
    ///
    /// Reordering 1 and 3 is exactly the defect: `clicked` would still be
    /// false on the press frame, but a future edit that "helpfully" hit-tested
    /// on press would have nowhere obvious to be wrong. Keeping the press arm
    /// first, and empty, is what makes the rule visible to the next reader.
    ///
    /// Branch 0 sits above the press for a smaller but real reason: a frame
    /// carrying both a cancel and a fresh `drag_started` is a *new* gesture,
    /// and the abandoned one must not be able to resurrect itself by having
    /// its origin overwritten.
    ///
    /// # ★ `press_kind: None` — the press means nothing in this mode
    ///
    /// [`press_kind`] returns `None` when the active mode forbids the meaning
    /// this press would have had (see its own header). `None` suppresses
    /// **two** branches, and it is worth naming both because suppressing only
    /// the first would look like it worked:
    ///
    /// * **branch 1**, so no drag ever starts — no band, no ghost, no release
    ///   to refuse;
    /// * **branch 3**, so no `Click` is reported — and *this* is the one that
    ///   makes a click in Read select nothing. A click is not a drag and does
    ///   not consult `press_kind` on its own, so gating the drag alone would
    ///   leave the single most common gesture on the canvas ungated.
    ///
    /// Branch 0 deliberately still runs: an in-flight drag stays cancellable
    /// whatever the mode has since become. Branch 2 likewise still completes a
    /// drag already in flight — unreachable in practice, because a mode change
    /// cancels the gesture on the way in (`PdfcerApp`'s mode-change arm), but a
    /// state machine that silently dropped a gesture it had already started
    /// would be wrong regardless of whether anything could reach it.
    pub fn update(&mut self, frame: PointerFrame, press: PressMeaning) -> GestureOutcome {
        if frame.cancel && self.drag.take().is_some() {
            return GestureOutcome::Cancelled;
        }

        if frame.drag_started {
            // ★ The press, not the position the drag was RECOGNISED at — see
            // `PointerFrame::press_origin` for the measurement. `latest` is
            // still the live position, so the very first in-flight frame
            // already describes a band from the true corner to the pointer.
            //
            // A press whose meaning this mode forbids (`press_kind` is `None`)
            // starts no drag — and still returns `Idle`, exactly as a
            // permitted press does. There is no third outcome for "refused",
            // because nothing was refused: in that mode the primary button
            // simply does not mean this.
            if let Some(kind) = press.drag
                && let Some(origin) = frame.press_origin.or(frame.pos)
            {
                self.drag = Some(Drag {
                    origin,
                    latest: frame.pos.unwrap_or(origin),
                    kind,
                    shift: frame.shift,
                });
            }
            return GestureOutcome::Idle;
        }

        if let Some(drag) = &mut self.drag {
            if let Some(pos) = frame.pos {
                drag.latest = pos;
            }
            let drag = *drag;
            if frame.drag_stopped {
                self.drag = None;
                return drag.outcome(Phase::Complete);
            }
            if frame.dragging {
                return drag.outcome(Phase::InFlight);
            }
            // Neither dragging nor stopped: the gesture was interrupted —
            // the window lost focus, the pointer left, a dialog opened.
            // Abandon it rather than commit it. An interrupted drag whose
            // release nobody saw must not resize a drawing.
            self.drag = None;
            return GestureOutcome::Idle;
        }

        // `press.click` is the click's half of the mode gate — see this
        // function's header and `PressMeaning`'s. It is asked separately from
        // `press.drag` because an armed measure tool has no drag and needs the
        // click, and a mode that cannot select content must still let it through.
        if press.click
            && (frame.clicked || frame.double_clicked || frame.triple_clicked)
            && let Some(point) = frame.pos
        {
            return GestureOutcome::Click {
                point,
                shift: frame.shift,
                double: frame.double_clicked,
                triple: frame.triple_clicked,
            };
        }

        GestureOutcome::Idle
    }

    /// Whether a drag is in flight — the canvas asks before setting a cursor,
    /// so a gesture keeps its own cursor even when the pointer has wandered
    /// off the thing it started on.
    #[must_use]
    pub fn active(&self) -> Option<DragKind> {
        self.drag.map(|d| d.kind)
    }
}

#[cfg(test)]
mod tests {
    // ★ Imported in the TEST module only. The 2026-08-18 R2 split moved the
    // outcome vocabulary — and with it every production reference to
    // `MarkupKind` — into `outcome`, so a module-level import would be unused
    // in the shipping build and clippy refuses it. The tests still name the
    // kind because they assert on the outcome the machine reports, which is
    // the surface, not the file it lives in.
    use crate::canvas::markup::MarkupKind;
    use egui::{Rect, Vec2};

    use super::*;

    fn at(x: f32, y: f32) -> Option<Pos2> {
        Some(Pos2::new(x, y))
    }

    /// ★ **A press produces nothing at all** — invariant 2, at its source.
    ///
    /// Whatever the press landed on, and whatever else the frame carries, the
    /// press frame is `Idle`. Nothing downstream of this can clear a
    /// selection, because nothing downstream of this is called.
    #[test]
    fn a_press_alone_produces_no_outcome() {
        for kind in [DragKind::Marquee(MarqueeIntent::Select), DragKind::Move] {
            let mut g = GestureState::default();
            let out = g.update(
                PointerFrame {
                    drag_started: true,
                    pos: at(10.0, 10.0),
                    ..PointerFrame::default()
                },
                PressMeaning::dragging(kind),
            );
            assert_eq!(out, GestureOutcome::Idle, "a press must decide nothing");
        }
    }

    /// ★ **A press on blank paper that becomes a drag never yields a click.**
    ///
    /// The whole sequence, frame by frame, as the roadmap describes it: press
    /// on empty canvas, move, release. If any frame produced a `Click`, the
    /// selection would be cleared by hit test — which is the defect.
    #[test]
    fn a_press_that_becomes_a_drag_never_yields_a_click() {
        let mut g = GestureState::default();
        let mut outcomes = Vec::new();
        outcomes.push(g.update(
            PointerFrame {
                drag_started: true,
                pos: at(0.0, 0.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select)),
        ));
        for step in 1..=5u8 {
            outcomes.push(g.update(
                PointerFrame {
                    dragging: true,
                    pos: at(f32::from(step) * 10.0, 0.0),
                    ..PointerFrame::default()
                },
                PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select)),
            ));
        }
        outcomes.push(g.update(
            PointerFrame {
                drag_stopped: true,
                pos: at(50.0, 20.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select)),
        ));

        assert!(
            !outcomes
                .iter()
                .any(|o| matches!(o, GestureOutcome::Click { .. })),
            "a drag produced a click: {outcomes:?}"
        );
        assert_eq!(
            outcomes.last(),
            Some(&GestureOutcome::Marquee {
                rect: Rect::from_two_pos(Pos2::ZERO, Pos2::new(50.0, 20.0)),
                // Dragged rightwards, so a window rather than a crossing one.
                crossing: false,
                shift: false,
                intent: MarqueeIntent::Select,
                phase: Phase::Complete,
            })
        );
    }

    /// A completed click with no drag is the one thing that reaches the
    /// selection by hit test.
    #[test]
    fn a_completed_click_is_reported_once_with_its_modifiers() {
        let mut g = GestureState::default();
        assert_eq!(
            g.update(
                PointerFrame {
                    clicked: true,
                    pos: at(7.0, 9.0),
                    shift: true,
                    ..PointerFrame::default()
                },
                PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select)),
            ),
            GestureOutcome::Click {
                point: Pos2::new(7.0, 9.0),
                shift: true,
                double: false,
                triple: false,
            }
        );
        // The frame after carries nothing, so the click is applied once.
        assert_eq!(
            g.update(
                PointerFrame::default(),
                PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select))
            ),
            GestureOutcome::Idle
        );
    }

    /// A double-click is reported as one, so the ladder descends instead of
    /// re-picking at the same rung.
    #[test]
    fn a_double_click_is_reported_as_a_double() {
        let mut g = GestureState::default();
        let out = g.update(
            PointerFrame {
                clicked: true,
                double_clicked: true,
                pos: at(1.0, 2.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select)),
        );
        assert_eq!(
            out,
            GestureOutcome::Click {
                point: Pos2::new(1.0, 2.0),
                shift: false,
                double: true,
                triple: false,
            }
        );
    }

    /// The marquee draws while in flight and commits once, on release — not
    /// on every frame it is dragged across.
    #[test]
    fn a_marquee_draws_in_flight_and_commits_once() {
        let mut g = GestureState::default();
        g.update(
            PointerFrame {
                drag_started: true,
                pos: at(100.0, 100.0),
                shift: true,
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select)),
        );
        let mid = g.update(
            PointerFrame {
                dragging: true,
                pos: at(40.0, 30.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select)),
        );
        assert_eq!(
            mid,
            GestureOutcome::Marquee {
                // Dragged up and left: normalised, or it would contain nothing.
                rect: Rect::from_two_pos(Pos2::new(100.0, 100.0), Pos2::new(40.0, 30.0)),
                // ★ …and leftwards, so this is a CROSSING window. The same drag
                // that motivated the normalisation note above is the one O88 is
                // about, which is a coincidence worth not tidying away: the
                // fixture for "a band may be dragged backwards" was already here.
                crossing: true,
                shift: true,
                intent: MarqueeIntent::Select,
                phase: Phase::InFlight,
            },
            "an in-flight marquee must be drawn, and must not commit"
        );
        let end = g.update(
            PointerFrame {
                drag_stopped: true,
                pos: at(40.0, 30.0),
                // Shift released before the button: the gesture keeps the
                // meaning it started with.
                shift: false,
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select)),
        );
        assert_eq!(
            end,
            GestureOutcome::Marquee {
                rect: Rect::from_two_pos(Pos2::new(100.0, 100.0), Pos2::new(40.0, 30.0)),
                crossing: true,
                shift: true,
                intent: MarqueeIntent::Select,
                phase: Phase::Complete,
            },
            "the modifier is sampled at the press, not at the release"
        );
    }

    /// ★ **A zoom marquee is the same band with the other intent** — same
    /// rect, same normalisation, same phases, same shift handling.
    ///
    /// Asserted by driving both intents through the identical frame sequence
    /// and comparing the outcomes field by field: everything but `intent` must
    /// match. That is the mechanical form of *"do not add a second rubber band
    /// with different pixels"* — if the two ever diverged geometrically, the
    /// canvas would be drawing two bands from one `draw_marquee`, and the
    /// operator would see a zoom box that did not agree with the box that had
    /// been selecting a moment earlier.
    #[test]
    fn a_zoom_marquee_is_the_same_band_with_the_other_intent() {
        fn run(intent: MarqueeIntent) -> Vec<GestureOutcome> {
            let mut g = GestureState::default();
            let kind = DragKind::Marquee(intent);
            vec![
                g.update(
                    PointerFrame {
                        drag_started: true,
                        pos: at(90.0, 70.0),
                        shift: true,
                        ..PointerFrame::default()
                    },
                    PressMeaning::dragging(kind),
                ),
                g.update(
                    PointerFrame {
                        dragging: true,
                        // Dragged up and left: normalisation has to be
                        // identical too, or one of the two would enclose
                        // nothing.
                        pos: at(20.0, 15.0),
                        ..PointerFrame::default()
                    },
                    PressMeaning::dragging(kind),
                ),
                g.update(
                    PointerFrame {
                        drag_stopped: true,
                        pos: at(20.0, 15.0),
                        ..PointerFrame::default()
                    },
                    PressMeaning::dragging(kind),
                ),
            ]
        }

        let select = run(MarqueeIntent::Select);
        let zoom = run(MarqueeIntent::Zoom);
        assert_eq!(select.len(), zoom.len());
        for (s, z) in select.iter().zip(zoom.iter()) {
            match (s, z) {
                (
                    GestureOutcome::Marquee {
                        rect: sr,
                        shift: ss,
                        intent: si,
                        phase: sp,
                        ..
                    },
                    GestureOutcome::Marquee {
                        rect: zr,
                        shift: zs,
                        intent: zi,
                        phase: zp,
                        ..
                    },
                ) => {
                    assert_eq!(sr, zr, "the two bands must be the same rectangle");
                    assert_eq!(ss, zs);
                    assert_eq!(sp, zp);
                    assert_eq!(*si, MarqueeIntent::Select);
                    assert_eq!(*zi, MarqueeIntent::Zoom);
                }
                (a, b) => assert_eq!(a, b, "the two gestures must run in lockstep"),
            }
        }
        assert!(matches!(
            zoom.last(),
            Some(GestureOutcome::Marquee {
                intent: MarqueeIntent::Zoom,
                phase: Phase::Complete,
                ..
            })
        ));
    }

    /// ★ **The intent is sampled at the press.** Disarming the zoom mid-drag —
    /// which is what the release itself does, and what a competing surface
    /// could do — must not turn a zoom marquee into a selection marquee
    /// halfway across the page.
    ///
    /// Modelled the way the machine actually experiences it: the caller
    /// reports `Select` on every frame after the press, exactly as it would
    /// once the arming flag had been cleared.
    #[test]
    fn a_marquee_keeps_the_intent_it_started_with() {
        let mut g = GestureState::default();
        g.update(
            PointerFrame {
                drag_started: true,
                pos: at(0.0, 0.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Zoom)),
        );
        let out = g.update(
            PointerFrame {
                drag_stopped: true,
                pos: at(40.0, 40.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select)),
        );
        assert!(
            matches!(
                out,
                GestureOutcome::Marquee {
                    intent: MarqueeIntent::Zoom,
                    ..
                }
            ),
            "the release must honour the intent the press carried, got {out:?}"
        );
    }

    /// ★ **A hand-tool frame produces no gesture at all** — the shape
    /// `canvas::interact` relies on so that a pan cannot also marquee.
    ///
    /// The canvas hands this machine a blank `PointerFrame` while the hand
    /// tool is active. This pins what "blank" is worth: whatever the pointer
    /// is doing on screen, nothing starts, nothing draws, nothing commits.
    #[test]
    fn a_blank_frame_starts_nothing_however_hard_the_pointer_is_working() {
        let mut g = GestureState::default();
        for _ in 0..5 {
            assert_eq!(
                g.update(
                    PointerFrame::default(),
                    PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select))
                ),
                GestureOutcome::Idle
            );
            assert_eq!(g.active(), None);
        }
    }

    /// …and a drag already in flight when the tool changes is **abandoned**,
    /// not committed. Reaching for the space bar mid-marquee is a change of
    /// mind, and the worst outcome available must be that nothing happened.
    #[test]
    fn a_drag_interrupted_by_the_hand_tool_commits_nothing() {
        let mut g = GestureState::default();
        g.update(
            PointerFrame {
                drag_started: true,
                pos: at(0.0, 0.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select)),
        );
        g.update(
            PointerFrame {
                dragging: true,
                pos: at(80.0, 60.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select)),
        );
        // The space bar goes down: the canvas stops describing the pointer.
        let out = g.update(
            PointerFrame::default(),
            PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select)),
        );
        assert_eq!(out, GestureOutcome::Idle);
        assert_eq!(
            g.active(),
            None,
            "the gesture must not survive the tool change"
        );
    }

    // -----------------------------------------------------------------
    // The markup band
    // -----------------------------------------------------------------

    /// ★ **A markup band reports its endpoints RAW, in drag order** — the
    /// property an arrow's head depends on.
    ///
    /// Asserted against a drag that goes **up and to the left**, because that
    /// is the case a normalising implementation gets wrong: `from` would come
    /// back as the smaller corner, which for this drag is the *head*.
    #[test]
    fn a_markup_band_reports_its_endpoints_in_drag_order() {
        let mut g = GestureState::default();
        let kind = DragKind::Markup(MarkupKind::Arrow);
        g.update(
            PointerFrame {
                drag_started: true,
                pos: at(400.0, 500.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(kind),
        );
        let mid = g.update(
            PointerFrame {
                dragging: true,
                pos: at(120.0, 90.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(kind),
        );
        assert_eq!(
            mid,
            GestureOutcome::Markup {
                kind: MarkupKind::Arrow,
                from: Pos2::new(400.0, 500.0),
                to: Pos2::new(120.0, 90.0),
                phase: Phase::InFlight,
            },
            "the band must not be normalised: an arrow's head is its `to`"
        );
        let end = g.update(
            PointerFrame {
                drag_stopped: true,
                pos: at(120.0, 90.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(kind),
        );
        assert_eq!(
            end,
            GestureOutcome::Markup {
                kind: MarkupKind::Arrow,
                from: Pos2::new(400.0, 500.0),
                to: Pos2::new(120.0, 90.0),
                phase: Phase::Complete,
            }
        );
    }

    /// ★ **A markup drag keeps the kind it was armed with**, even if the
    /// caller reports a different one on every later frame — which is what
    /// would happen if the operator's next click landed on another Markup
    /// button while the button was still down.
    #[test]
    fn a_markup_drag_keeps_the_kind_it_started_with() {
        let mut g = GestureState::default();
        g.update(
            PointerFrame {
                drag_started: true,
                pos: at(0.0, 0.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Markup(MarkupKind::Ellipse)),
        );
        let out = g.update(
            PointerFrame {
                drag_stopped: true,
                pos: at(30.0, 40.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Markup(MarkupKind::Rectangle)),
        );
        assert!(
            matches!(
                out,
                GestureOutcome::Markup {
                    kind: MarkupKind::Ellipse,
                    ..
                }
            ),
            "the release must honour the kind the press carried, got {out:?}"
        );
    }

    /// ★ **Escape abandons a markup drag without authoring anything.**
    ///
    /// The existing cancellation test covers the three older kinds; this adds
    /// the one where an un-cancelled release would write to the document. A
    /// `Complete` here would be an annotation in the file that the operator
    /// explicitly abandoned.
    #[test]
    fn escape_abandons_a_markup_drag_without_committing() {
        let mut g = GestureState::default();
        let kind = DragKind::Markup(MarkupKind::Rectangle);
        g.update(
            PointerFrame {
                drag_started: true,
                pos: at(10.0, 10.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(kind),
        );
        g.update(
            PointerFrame {
                dragging: true,
                pos: at(200.0, 160.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(kind),
        );
        let out = g.update(
            PointerFrame {
                dragging: true,
                pos: at(200.0, 160.0),
                cancel: true,
                ..PointerFrame::default()
            },
            PressMeaning::dragging(kind),
        );
        assert_eq!(out, GestureOutcome::Cancelled);
        assert_eq!(g.active(), None);
        // …and the release that follows commits nothing either.
        assert_eq!(
            g.update(
                PointerFrame {
                    drag_stopped: true,
                    pos: at(200.0, 160.0),
                    ..PointerFrame::default()
                },
                PressMeaning::dragging(kind),
            ),
            GestureOutcome::Idle
        );
    }

    // -----------------------------------------------------------------
    // The mode gate
    // -----------------------------------------------------------------

    /// ★ **A press whose meaning is forbidden starts no drag, and a click in
    /// that mode reports nothing.**
    ///
    /// The state-machine half of the gate. The click assertion is the
    /// load-bearing one: a click is not a drag, so a build that gated only the
    /// press would still select on every click — which is the single most
    /// common gesture on the canvas.
    #[test]
    fn a_forbidden_press_starts_nothing_and_a_forbidden_click_reports_nothing() {
        let mut g = GestureState::default();
        let out = g.update(
            PointerFrame {
                drag_started: true,
                pos: at(10.0, 10.0),
                press_origin: at(10.0, 10.0),
                ..PointerFrame::default()
            },
            PressMeaning::NOTHING,
        );
        assert_eq!(out, GestureOutcome::Idle);
        assert_eq!(g.active(), None, "no drag was started");

        // Dragging on: still nothing, because there is nothing in flight.
        let mid = g.update(
            PointerFrame {
                dragging: true,
                pos: at(60.0, 60.0),
                ..PointerFrame::default()
            },
            PressMeaning::NOTHING,
        );
        assert_eq!(mid, GestureOutcome::Idle, "no band, no ghost");

        // And a completed click reports nothing at all.
        let mut g = GestureState::default();
        let out = g.update(
            PointerFrame {
                clicked: true,
                pos: at(10.0, 10.0),
                ..PointerFrame::default()
            },
            PressMeaning::NOTHING,
        );
        assert_eq!(
            out,
            GestureOutcome::Idle,
            "a click in a mode that cannot select must not reach the selection"
        );
    }

    /// A drag already in flight stays cancellable whatever the mode became —
    /// branch 0 of [`GestureState::update`], which `None` deliberately does not
    /// suppress.
    #[test]
    fn a_drag_in_flight_is_still_cancellable_after_the_mode_forbids_it() {
        let mut g = GestureState::default();
        g.update(
            PointerFrame {
                drag_started: true,
                pos: at(0.0, 0.0),
                press_origin: at(0.0, 0.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select)),
        );
        assert!(g.active().is_some(), "a drag is in flight");
        let out = g.update(
            PointerFrame {
                cancel: true,
                ..PointerFrame::default()
            },
            PressMeaning::NOTHING,
        );
        assert_eq!(out, GestureOutcome::Cancelled);
        assert_eq!(g.active(), None);
    }

    /// ★ **A drag is anchored at the press, not at the frame the drag was
    /// recognised on.**
    ///
    /// The regression test for the 94-point offset measured on a real drag —
    /// see [`PointerFrame::press_origin`]. It is stated as a **magnitude**
    /// against the press point rather than as "the band is on the page",
    /// because the defective build put the band on the page too.
    ///
    /// The fallback is asserted in the same test: a frame with no press origin
    /// behaves exactly as it did before the field existed, so supplying it is
    /// an accuracy improvement and never a behaviour change.
    #[test]
    fn a_drag_is_anchored_at_the_press_not_at_the_frame_it_was_recognised_on() {
        for kind in [
            DragKind::Markup(MarkupKind::Arrow),
            DragKind::Marquee(MarqueeIntent::Select),
            DragKind::Move,
        ] {
            let mut g = GestureState::default();
            // egui reports the drag one interval late: the button went down at
            // (100, 100) and by this frame the pointer is already at (120, 88).
            g.update(
                PointerFrame {
                    drag_started: true,
                    pos: at(120.0, 88.0),
                    press_origin: at(100.0, 100.0),
                    ..PointerFrame::default()
                },
                PressMeaning::dragging(kind),
            );
            let out = g.update(
                PointerFrame {
                    drag_stopped: true,
                    pos: at(300.0, 40.0),
                    ..PointerFrame::default()
                },
                PressMeaning::dragging(kind),
            );
            match out {
                GestureOutcome::Markup { from, to, .. } => {
                    assert_eq!(from, Pos2::new(100.0, 100.0), "{kind:?}");
                    assert_eq!(to, Pos2::new(300.0, 40.0), "{kind:?}");
                }
                GestureOutcome::Marquee { rect, .. } => {
                    assert_eq!(
                        rect,
                        Rect::from_two_pos(Pos2::new(100.0, 100.0), Pos2::new(300.0, 40.0)),
                        "{kind:?}"
                    );
                }
                GestureOutcome::Move { delta, .. } => {
                    assert_eq!(delta, Vec2::new(200.0, -60.0), "{kind:?}");
                }
                other => panic!("{kind:?} produced {other:?}"),
            }
        }

        // …and with no press origin reported, the origin is the position on the
        // recognised frame, exactly as before.
        let mut g = GestureState::default();
        g.update(
            PointerFrame {
                drag_started: true,
                pos: at(120.0, 88.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Move),
        );
        assert_eq!(
            g.update(
                PointerFrame {
                    drag_stopped: true,
                    pos: at(300.0, 40.0),
                    ..PointerFrame::default()
                },
                PressMeaning::dragging(DragKind::Move),
            ),
            GestureOutcome::Move {
                delta: Vec2::new(180.0, -48.0),
                phase: Phase::Complete,
            }
        );
    }

    /// A press with no position **and** no press origin starts nothing, and a
    /// press origin with no current position still anchors correctly — the two
    /// halves of the fallback, so neither can be dropped silently.
    #[test]
    fn a_press_origin_without_a_current_position_still_anchors() {
        let mut g = GestureState::default();
        g.update(
            PointerFrame {
                drag_started: true,
                pos: None,
                press_origin: at(50.0, 60.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Markup(MarkupKind::Rectangle)),
        );
        assert_eq!(g.active(), Some(DragKind::Markup(MarkupKind::Rectangle)));
    }

    /// A drag keeps the kind it started with, even when the pointer leaves
    /// the grip, the object and the page.
    #[test]
    fn a_drag_keeps_the_kind_it_started_with() {
        let mut g = GestureState::default();
        g.update(
            PointerFrame {
                drag_started: true,
                pos: at(0.0, 0.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Resize(Grip::SouthEast)),
        );
        // The caller now says "Marquee" every frame — the pointer has left
        // the grip. The gesture must not change under it.
        let out = g.update(
            PointerFrame {
                dragging: true,
                pos: at(-500.0, -900.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select)),
        );
        assert_eq!(
            out,
            GestureOutcome::Resize {
                grip: Grip::SouthEast,
                delta: Vec2::new(-500.0, -900.0),
                phase: Phase::InFlight,
            }
        );
        assert_eq!(g.active(), Some(DragKind::Resize(Grip::SouthEast)));
    }

    /// A move drag reports the travel since the press, not since last frame —
    /// so a caller applying it once on `Complete` moves the object by the
    /// whole gesture rather than by its last twitch.
    #[test]
    fn a_move_drag_reports_the_whole_travel() {
        let mut g = GestureState::default();
        g.update(
            PointerFrame {
                drag_started: true,
                pos: at(10.0, 10.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Move),
        );
        for step in 1..=3u8 {
            g.update(
                PointerFrame {
                    dragging: true,
                    pos: at(10.0 + f32::from(step) * 5.0, 10.0),
                    ..PointerFrame::default()
                },
                PressMeaning::dragging(DragKind::Move),
            );
        }
        let end = g.update(
            PointerFrame {
                drag_stopped: true,
                pos: at(40.0, 25.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Move),
        );
        assert_eq!(
            end,
            GestureOutcome::Move {
                delta: Vec2::new(30.0, 15.0),
                phase: Phase::Complete,
            }
        );
    }

    /// ★ **An interrupted drag is abandoned, never committed.**
    ///
    /// Focus loss, a dialog, the pointer leaving the window: egui stops
    /// reporting the drag without ever reporting a stop. Committing on the
    /// next frame that happens to look like a release would apply an edit the
    /// operator never finished.
    #[test]
    fn an_interrupted_drag_is_abandoned_rather_than_committed() {
        let mut g = GestureState::default();
        g.update(
            PointerFrame {
                drag_started: true,
                pos: at(0.0, 0.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Move),
        );
        let out = g.update(
            PointerFrame::default(),
            PressMeaning::dragging(DragKind::Move),
        );
        assert_eq!(out, GestureOutcome::Idle);
        assert_eq!(
            g.active(),
            None,
            "the gesture must not survive its interruption"
        );
    }

    /// ★ **Escape abandons a drag in flight, and it commits nothing.**
    ///
    /// The gesture ladder's escape hatch: a move drag that is halfway across
    /// the page and clearly wrong must be abandonable without an undo. The
    /// frame that carries the cancel produces `Cancelled` — never a
    /// `Complete`, which is the outcome that would have rewritten the page.
    #[test]
    fn escape_abandons_a_drag_in_flight_without_committing() {
        for kind in [
            DragKind::Move,
            DragKind::Marquee(MarqueeIntent::Select),
            DragKind::Resize(Grip::SouthEast),
        ] {
            let mut g = GestureState::default();
            g.update(
                PointerFrame {
                    drag_started: true,
                    pos: at(0.0, 0.0),
                    ..PointerFrame::default()
                },
                PressMeaning::dragging(kind),
            );
            g.update(
                PointerFrame {
                    dragging: true,
                    pos: at(80.0, 40.0),
                    ..PointerFrame::default()
                },
                PressMeaning::dragging(kind),
            );
            let out = g.update(
                PointerFrame {
                    // egui still reports the button as down: the operator has
                    // not let go, they have changed their mind.
                    dragging: true,
                    pos: at(80.0, 40.0),
                    cancel: true,
                    ..PointerFrame::default()
                },
                PressMeaning::dragging(kind),
            );
            assert_eq!(out, GestureOutcome::Cancelled, "{kind:?}");
            assert_eq!(g.active(), None, "{kind:?} survived its cancellation");
        }
    }

    /// …and the release that follows a cancel commits nothing either. The
    /// operator's finger comes off the button some frames later, and by then
    /// there is no gesture for `drag_stopped` to complete.
    #[test]
    fn the_release_after_a_cancel_commits_nothing() {
        let mut g = GestureState::default();
        g.update(
            PointerFrame {
                drag_started: true,
                pos: at(0.0, 0.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Move),
        );
        g.update(
            PointerFrame {
                dragging: true,
                pos: at(90.0, 0.0),
                cancel: true,
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Move),
        );
        let out = g.update(
            PointerFrame {
                drag_stopped: true,
                pos: at(90.0, 0.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Move),
        );
        assert_eq!(
            out,
            GestureOutcome::Idle,
            "a cancelled drag must not commit on its release"
        );
    }

    /// ★ **Escape with no drag under it is NOT consumed**, so it reaches the
    /// selection ladder and one press still ascends exactly one rung.
    #[test]
    fn escape_with_no_drag_leaves_the_key_for_the_ladder() {
        let mut g = GestureState::default();
        assert_eq!(
            g.update(
                PointerFrame {
                    cancel: true,
                    pos: at(5.0, 5.0),
                    ..PointerFrame::default()
                },
                PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select)),
            ),
            GestureOutcome::Idle,
            "reporting Cancelled here would make Escape need two presses to \
             leave a rung"
        );
    }

    /// A cancel arriving on the same frame as a fresh press abandons the old
    /// gesture and does not let the new one inherit it — the new press starts
    /// cleanly on the frame after.
    #[test]
    fn a_cancel_on_a_press_frame_does_not_resurrect_the_old_drag() {
        let mut g = GestureState::default();
        g.update(
            PointerFrame {
                drag_started: true,
                pos: at(0.0, 0.0),
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Move),
        );
        let out = g.update(
            PointerFrame {
                drag_started: true,
                pos: at(500.0, 500.0),
                cancel: true,
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Move),
        );
        assert_eq!(out, GestureOutcome::Cancelled);
        assert_eq!(g.active(), None);
    }

    /// A press with no pointer position starts nothing — a trackpad gesture
    /// can arrive with the pointer off-window, and a drag anchored at a
    /// fabricated origin would move an object by a page's width.
    #[test]
    fn a_press_with_no_position_starts_no_drag() {
        let mut g = GestureState::default();
        let out = g.update(
            PointerFrame {
                drag_started: true,
                pos: None,
                ..PointerFrame::default()
            },
            PressMeaning::dragging(DragKind::Move),
        );
        assert_eq!(out, GestureOutcome::Idle);
        assert_eq!(g.active(), None);
    }
}
