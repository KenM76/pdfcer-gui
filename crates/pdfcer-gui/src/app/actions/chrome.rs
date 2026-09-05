//! # `app::actions::chrome` — which piece of View ▸ Display an action is about
//!
//! One enum and its two methods, split out of [`super`] on 2026-08-19 when that
//! file crossed R2's 1,500-line ceiling.
//!
//! ## Why this is the seam
//!
//! [`super`] is the **action vocabulary** — one enum, and the argument for
//! every variant in it. `ViewChrome` is not an action; it is an *operand* of
//! one, and it is the only operand in that file that has its own type, its own
//! `ALL`, and its own id mapping. Everything else a variant carries is a
//! `usize`, a `Point` or a type from `pdfcer-core`.
//!
//! That makes it separable in the way `tools/gates/check-file-size.sh` asks
//! for — *one subject per file* — and it is separable **without** the enum
//! itself moving, which nothing could do.
//!
//! ## ★ Why it is here and not in `canvas`
//!
//! Unchanged from where it was written, and worth keeping because it is the
//! question a reader will ask first: it is the operand of an **action**, and
//! `shell::commands` maps a command id to one. Putting it in `canvas` would
//! make the shell's id map reach into the canvas to name a value, which is a
//! dependency in the wrong direction for a type that is about *what the
//! operator asked for* rather than about *what draws it*.

/// Which piece of View ▸ Display chrome a [`Action::ToggleViewChrome`] is
/// about.
///
/// An enum rather than three action variants — see that variant's own docs —
/// and it lives here rather than in `canvas` because it is the *operand of an
/// action*, and `shell::commands` (which maps ids to it) must not have to
/// reach into the canvas to name one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewChrome {
    /// `view.rulers` — the gutters along the canvas edges.
    Rulers,
    /// `view.grid` — the drawing grid over each page.
    Grid,
    /// `view.guides` — whether the operator's guides are shown and draggable.
    Guides,
    /// `view.show_points` — an object's anchors, without descending into it.
    ///
    /// ★ The fourth variant, and the enum's own docs predicted the shape of
    /// what would go wrong if one were added carelessly: *"a fourth toggle
    /// added to the enum with no registration would draw nothing and nothing
    /// else in the suite would notice."* Both directions are asserted, so this
    /// one could not be added that way.
    ShowPoints,
    /// `view.line_weights` — **are strokes drawn at the widths the file
    /// declares, or every one of them at one device pixel?**
    ///
    /// `OPERATOR_REQUESTS.md` **O137**, asked for by name:
    /// *"the button to show all lines without their thickness — thin lines or
    /// something like cad has … I do want that display option!"*
    ///
    /// ★★★ **The fifth variant, and the only one that is not chrome DRAWN OVER
    /// the page.** The other four add a mark the canvas paints on top of a
    /// finished texture; this one changes what the texture *is*
    /// ([`pdfcer_render::font::RenderOptions::stroke_display`], engine
    /// `Pass 254.0`). It is in this enum anyway, and the reason is worth
    /// stating rather than leaving to be re-litigated: what this enum models is
    /// **View ▸ Display's independent toggles** — a set of switches an operator
    /// flips while reading, each of which renders pressed and each of which is
    /// dispatched by the same action. A fifth mechanism for the fifth switch
    /// would have been a second `Action`, a second `selected:` publisher and a
    /// second id mapping, to express the same gesture.
    ///
    /// ⚠ What it DOES need beyond the other four is a **stale raster**. See
    /// [`crate::viewer::ViewState::line_weights`]: the answer is part of
    /// [`crate::render::worker::RenderKey`], because a cache that served the
    /// texture drawn under the opposite answer would make this toggle look
    /// exactly as inert as the dead button it replaces.
    ///
    /// ★ `true` (the default) is *faithful widths*. This is the one variant
    /// whose "on" is the shipped behaviour rather than an addition — the
    /// operator's gesture is turning it **off**.
    LineWeights,
}

impl ViewChrome {
    /// Every variant, in the order View ▸ Display lists them.
    ///
    /// Iterated by the tests that assert each has a command and each command
    /// has a `selected:` condition — the same both-directions check
    /// `PageDisplay::ALL` exists for, and for the same reason: a fourth toggle
    /// added to the enum with no registration would draw nothing and nothing
    /// else in the suite would notice.
    pub const ALL: &'static [ViewChrome] = &[
        ViewChrome::Rulers,
        ViewChrome::Grid,
        ViewChrome::Guides,
        ViewChrome::ShowPoints,
        ViewChrome::LineWeights,
    ];

    /// Read this toggle out of a view state.
    #[must_use]
    pub fn read(self, view: &crate::viewer::ViewState) -> bool {
        match self {
            ViewChrome::Rulers => view.rulers,
            ViewChrome::Grid => view.grid,
            ViewChrome::Guides => view.guides,
            ViewChrome::ShowPoints => view.show_points,
            ViewChrome::LineWeights => view.line_weights,
        }
    }

    /// Write this toggle into a view state.
    ///
    /// The pair with [`Self::read`], so the enum's mapping onto
    /// [`crate::viewer::ViewState`]'s three fields is stated exactly twice, in
    /// adjacent functions, instead of once per consumer.
    pub fn write(self, view: &mut crate::viewer::ViewState, on: bool) {
        match self {
            ViewChrome::Rulers => view.rulers = on,
            ViewChrome::Grid => view.grid = on,
            ViewChrome::Guides => view.guides = on,
            ViewChrome::ShowPoints => view.show_points = on,
            ViewChrome::LineWeights => view.line_weights = on,
        }
    }
}
