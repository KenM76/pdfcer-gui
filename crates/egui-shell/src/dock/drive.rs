//! A dock driven through real `egui` events, frame by frame — the driver the
//! gesture test files share.
//!
//! # ★ Why the first frame of every fixture is empty
//!
//! `egui` resolves a press against the widget rectangles of the **previous**
//! frame. A fixture that pressed on its first frame would press on nothing, and
//! would report exactly what a build with no drag sensing reports — a green
//! test over a dead feature. [`Harness::warm`] is that frame, and every file
//! using this driver owes a positive control that says the pump reaches the
//! widgets at all.
//!
//! # Fonts
//!
//! None is installed. With no font a label measures zero and every tab is
//! exactly [`super::plan::MIN_TAB_WIDTH`] wide, which is a constant — so a
//! fixture that reads its positions back out of [`super::DockState::geometry`]
//! rather than computing them from text is unaffected. A test that needs text
//! metrics installs [`super::testfont`] itself.

use egui::{Event, PointerButton, Pos2, Rect, Vec2};

use super::geometry::StackAddr;
use super::{Dock, DockLayout, DockState};

/// One `egui::Context` driven frame by frame, with the dock's state carried
/// across them.
pub(super) struct Harness {
    /// The context every frame runs in — one per harness, because `egui`
    /// memory is where a drag lives between frames.
    pub ctx: egui::Context,
    /// The dock's state, carried across frames as an application would.
    pub state: DockState,
    /// What the last frame reported.
    pub report: super::DockFrameReport,
    /// Every region the last frame published, by name.
    rects: Vec<(String, Rect)>,
    /// The window every frame is given.
    window: Vec2,
}

impl Harness {
    /// A harness over a 1400 × 900 window.
    pub fn new(layout: DockLayout) -> Self {
        Self::sized(layout, Vec2::new(1400.0, 900.0))
    }

    /// A harness over a window of a stated size, warmed.
    pub fn sized(layout: DockLayout, window: Vec2) -> Self {
        let mut h = Self {
            ctx: egui::Context::default(),
            state: DockState::new(layout),
            report: super::DockFrameReport::default(),
            rects: Vec::new(),
            window,
        };
        h.warm();
        h
    }

    /// Run one frame with the given events.
    pub fn frame(&mut self, events: Vec<Event>) {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, self.window)),
            events,
            ..Default::default()
        };
        let state = &mut self.state;
        let mut report = super::DockFrameReport::default();
        let mut rects: Vec<(String, Rect)> = Vec::new();
        let mut sink = |r: &super::report::RectReport<'_>| rects.push((r.name.to_string(), r.rect));
        let _ = self.ctx.clone().run_ui(input, |ui| {
            report = Dock::new()
                .reporting_rects_to(&mut sink)
                .show(ui, state, |_, _| {});
        });
        self.report = report;
        self.rects = rects;
    }

    /// A frame with no input, so the next frame's press has rectangles to
    /// land on. See the module header.
    pub fn warm(&mut self) {
        self.frame(Vec::new());
    }

    /// The region published under `name` on the last frame, if any.
    pub fn rect(&self, name: &str) -> Option<Rect> {
        self.rects
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, r)| *r)
    }

    /// The centre of a drawn tab, from the layout's own record.
    pub fn tab_centre(&self, stack: StackAddr, index: usize) -> Pos2 {
        self.state
            .geometry()
            .tab_rect(stack.tab(index))
            .unwrap_or_else(|| panic!("tab {index} of {stack:?} was not drawn"))
            .center()
    }

    /// A drawn compartment, from the layout's own record.
    pub fn stack_rect(&self, stack: StackAddr) -> Rect {
        self.state
            .geometry()
            .stack_rect(stack)
            .unwrap_or_else(|| panic!("{stack:?} was not drawn"))
    }
}

/// Press the primary button at `pos`.
pub(super) fn press(pos: Pos2) -> Vec<Event> {
    vec![
        Event::PointerMoved(pos),
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
    ]
}

/// Move the pointer to `pos` with the button still down.
pub(super) fn drag_to(pos: Pos2) -> Vec<Event> {
    vec![Event::PointerMoved(pos)]
}

/// Release the primary button at `pos`.
pub(super) fn release(pos: Pos2) -> Vec<Event> {
    vec![
        Event::PointerMoved(pos),
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        },
    ]
}
