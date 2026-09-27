//! # `guidemodel` — the ruler guides a document carries, and their on-disk spelling

use egui::{Pos2, Rect, pos2};

use crate::canvasmapping::PageMapping;

/// The separator between a guide's three fields.
pub const FIELD: char = ':';

/// How many guides one document may carry.
pub const MAX_PER_DOCUMENT: usize = 256;

/// The half-width, in logical points, of the band that catches a guide drag.
pub const CATCH_PTS: f32 = 4.0;

/// Which way a guide runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuideAxis {
    /// A horizontal line, pinning a canvas **y**. Dragged from the top ruler.
    Horizontal,
    /// A vertical line, pinning a canvas **x**. Dragged from the left ruler.
    Vertical,
}
impl GuideAxis {
    /// The on-disk spelling.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            // ui-text-exempt: on-disk spelling, never displayed as copy
            GuideAxis::Horizontal => "h",
            // ui-text-exempt: on-disk spelling, never displayed as copy
            GuideAxis::Vertical => "v",
        }
    }

    /// The axis an on-disk spelling names, or `None`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "h" => Some(GuideAxis::Horizontal),
            "v" => Some(GuideAxis::Vertical),
            _ => None,
        }
    }

    /// The component of a canvas point this axis pins.
    pub fn of(self, p: Pos2) -> f32 {
        match self {
            GuideAxis::Horizontal => p.y,
            GuideAxis::Vertical => p.x,
        }
    }
}
/// One guide: a line at a fixed place on one page.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Guide {
    /// The 0-based page it belongs to.
    pub page: usize,
    /// Which way the line runs.
    pub axis: GuideAxis,
    /// The canvas-space coordinate it pins.
    pub at: f32,
}
impl Guide {
    /// Its line on screen, spanning the page it belongs to.
    pub fn segment(self, map: PageMapping) -> [Pos2; 2] {
        let page = map.image_rect();
        match self.axis {
            GuideAxis::Horizontal => {
                let y = map.to_screen(pos2(0.0, self.at)).y;
                [pos2(page.min.x, y), pos2(page.max.x, y)]
            }
            GuideAxis::Vertical => {
                let x = map.to_screen(pos2(self.at, 0.0)).x;
                [pos2(x, page.min.y), pos2(x, page.max.y)]
            }
        }
    }

    /// The screen-space band a press must land in to grab this guide.
    pub fn band(self, map: PageMapping) -> Rect {
        let [a, b] = self.segment(map);
        Rect::from_two_pos(a, b).expand2(match self.axis {
            GuideAxis::Horizontal => egui::vec2(0.0, CATCH_PTS),
            GuideAxis::Vertical => egui::vec2(CATCH_PTS, 0.0),
        })
    }
}
/// Every guide one document carries.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Guides(Vec<Guide>);
impl Guides {
    /// Whether there are none at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// How many there are, across every page.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Every guide, in the order they will be written.
    pub fn iter(&self) -> impl Iterator<Item = Guide> + '_ {
        self.0.iter().copied()
    }

    /// The guides on `page`, with their index in the whole collection.
    pub fn on_page(&self, page: usize) -> impl Iterator<Item = (usize, Guide)> + '_ {
        self.0
            .iter()
            .enumerate()
            .filter(move |(_, g)| g.page == page)
            .map(|(i, g)| (i, *g))
    }

    /// Add `guide`, unless the document is already at [`MAX_PER_DOCUMENT`].
    pub fn add(&mut self, guide: Guide) -> bool {
        if self.0.len() >= MAX_PER_DOCUMENT || !guide.at.is_finite() {
            return false;
        }
        self.0.push(guide);
        true
    }

    /// Replace the guide at `index`, if there is one.
    pub fn replace(&mut self, index: usize, guide: Guide) {
        if guide.at.is_finite()
            && let Some(slot) = self.0.get_mut(index)
        {
            *slot = guide;
        }
    }

    /// Remove the guide at `index`, if there is one.
    pub fn remove(&mut self, index: usize) {
        if index < self.0.len() {
            self.0.remove(index);
        }
    }

    /// The on-disk payload — the part of a line before the tab.
    pub fn encode(&self) -> String {
        self.0
            .iter()
            .map(|g| format!("{}{FIELD}{}{FIELD}{}", g.page, g.axis.id(), g.at))
            .collect::<Vec<_>>()
            .join(" ") // ui-text-exempt: the on-disk field separator, never displayed
    }

    /// Parse an on-disk payload, dropping anything that does not parse.
    pub fn decode(payload: &str) -> Self {
        let mut out = Self::default();
        for token in payload.split_whitespace() {
            let mut parts = token.split(FIELD);
            let (Some(page), Some(axis), Some(at), None) =
                (parts.next(), parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            let (Ok(page), Some(axis), Ok(at)) = (
                page.parse::<usize>(),
                GuideAxis::from_id(axis),
                at.parse::<f32>(),
            ) else {
                continue;
            };
            out.add(Guide { page, axis, at });
        }
        out
    }
}
