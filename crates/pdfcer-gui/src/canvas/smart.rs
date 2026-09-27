//! # `canvas::smart` — **click selects the whole thing; double-click goes
//! # inside it**
//!
//! ## The request
//!
//!
//! > *"we should have a checkbox in navigate for a Smart-Selector option, in
//! > Edit Mode this makes it so if I click on an object and it is enabled to be
//! > selected in the Smart Selector it puts it in a bounding box with handles to
//! > move, resize and rotate, if a click selects an object that is made of
//! > multiple objects (group, form, etc) a double click should bring me further
//! > down the chain, until a double click reaches the bottom and lets me edit
//! > the nodes. If I recall this is similar to how Inscape does things and we
//! > should follow that convention."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/smart.md`.

use crate::panels::objects::provider::TargetId;

/// Whether Smart-Selector is on. Memory key.
const ENABLED_KEY: &str = "pdfcer.smart-select.enabled"; // ui-text-exempt: a memory key, never displayed

/// The container the pointer is currently working inside. Memory key.
const ENTERED_KEY: &str = "pdfcer.smart-select.entered"; // ui-text-exempt: a memory key, never displayed

/// **The container a click is currently scoped to.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entered {
    /// The page the container is on.
    pub page: usize,
    /// The container's **page paint-order index** — a form XObject.
    pub form: u64,
    /// Which open document this was recorded for.
    ///
    /// The tab slot, published every frame by `crate::pagedrag::active`.
    /// Without it, entering a title block in one drawing and switching tabs
    /// would scope clicks in the other drawing to whatever object happened to
    /// share that index — which is *"in range and wrong"*, the failure
    /// `TargetId`'s own header names as the dangerous one.
    pub slot: usize,
}

/// One `egui::Id` per key, spelled once.
fn id(key: &str) -> egui::Id {
    egui::Id::new(key)
}

/// **Is Smart-Selector on?** Defaults to `true`.
#[must_use]
pub fn enabled(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(id(ENABLED_KEY)))
        .unwrap_or(true)
}

/// Turn it on or off.
pub fn set_enabled(ctx: &egui::Context, on: bool) {
    ctx.data_mut(|d| d.insert_temp(id(ENABLED_KEY), on));
    if !on {
        // Switching it off must also leave whatever container the operator
        // was inside. A scope that outlived the mechanism that created it
        // would make the next click resolve by a rule no longer switched on,
        // which is unexplainable from the surface.
        leave(ctx);
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("smart-select enabled={on}")
    });
}

/// **Mirror the persisted answer into the live one**, once per frame.
pub fn sync(ctx: &egui::Context, on: bool) {
    if enabled(ctx) != on {
        ctx.data_mut(|d| d.insert_temp(id(ENABLED_KEY), on));
    }
}

/// **The container the pointer is scoped to**, if the record is still valid for
/// this page and this document.
#[must_use]
pub fn entered(ctx: &egui::Context, page: usize, slot: usize) -> Option<Entered> {
    ctx.data(|d| d.get_temp::<Entered>(id(ENTERED_KEY)))
        .filter(|e| e.page == page && e.slot == slot)
}

/// Enter a container.
pub fn enter(ctx: &egui::Context, entered: Entered) {
    ctx.data_mut(|d| d.insert_temp(id(ENTERED_KEY), entered));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "smart-enter page={} form={} slot={}",
            entered.page, entered.form, entered.slot
        )
    });
}

/// **Leave whatever container was entered**, and report whether there was one.
pub fn leave(ctx: &egui::Context) -> bool {
    let had = ctx.data_mut(|d| {
        let had = d.get_temp::<Entered>(id(ENTERED_KEY)).is_some();
        d.remove::<Entered>(id(ENTERED_KEY));
        had
    });
    if had {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "smart-leave".to_owned()
        });
    }
    had
}

/// **The scope one frame's clicks resolve in** — read once, passed down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Scope {
    /// Whether the substitution happens at all.
    pub enabled: bool,
    /// The container currently entered on this page, as a page paint-order
    /// index.
    pub entered: Option<u64>,
}

impl Scope {
    /// A scope that changes nothing — for tests, and for surfaces that have no
    /// context to read.
    #[must_use]
    pub const fn off() -> Self {
        Self {
            enabled: false,
            entered: None,
        }
    }

    /// **What a click on `target` should actually select.**
    #[must_use]
    pub fn resolve(
        self,
        targets: &dyn crate::canvas::target::CanvasTargetProvider,
        page: usize,
        target: TargetId,
    ) -> TargetId {
        if !self.enabled || !target.is_leaf() {
            return target;
        }
        let Some(container) = targets.containing_form(page, target) else {
            // A leaf whose container cannot be resolved is left alone rather
            // than dropped: the operator can still select it, which is what
            // they could do before this module existed.
            return target;
        };
        if self
            .entered
            .is_some_and(|f| TargetId::Object(f) == container)
        {
            return target;
        }
        //
        // Resolving a leaf to its container is right for a title block and
        // wrong for the commonest form in the world: every CAD exporter wraps a
        // drawing's whole visible body in one page-sized form, and a `/BBox` is
        // a clipping extent (§8.10.1) rather than a claim about ink. So the
        // wrapper contains everything, wins every click, and "select the
        // container first" became "select the whole drawing, every time".
        //
        // ⇒ Which is the operator's own HEADLINE complaint, verbatim, restored
        // by the feature built to improve selection:
        //
        //   "There are obviously more than one item on the page, but when I
        //    click on one of the objects all I get is the page selected."
        //
        //
        // Entering such a form is untouched. A double-click descends, the
        // Objects panel lists it, the canvas menu reaches it. Reachable on
        // purpose was always the design; winning by DEFAULT is what was wrong,
        // both times.
        if !targets.container_is_worth_selecting(page, container) {
            return target;
        }
        container
    }
}

/// Read this frame's scope for `page`.
#[must_use]
pub fn scope(ctx: &egui::Context, page: usize) -> Scope {
    let slot = crate::pagedrag::active(ctx).unwrap_or_default().slot;
    Scope {
        enabled: enabled(ctx),
        entered: entered(ctx, page, slot).map(|e| e.form),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::targetstub::StubTargets;

    /// A stub whose leaves all belong to page object 0.
    fn stub() -> StubTargets {
        StubTargets::new(
            0,
            [egui::Rect::from_min_max(
                egui::pos2(0.0, 0.0),
                egui::pos2(100.0, 100.0),
            )],
        )
        .with_leaves([egui::Rect::from_min_max(
            egui::pos2(10.0, 10.0),
            egui::pos2(20.0, 20.0),
        )])
        .with_containers([(0, 0)])
    }

    /// **A click on something inside a container selects the container.**
    ///
    /// The feature, stated as the one substitution it performs.
    #[test]
    fn a_leaf_resolves_to_its_container() {
        let ctx = egui::Context::default();
        let targets = stub();
        assert_eq!(
            scope(&ctx, 0).resolve(&targets, 0, TargetId::Leaf(0)),
            TargetId::Object(0),
            "clicking a line inside a title block selects the title block"
        );
    }

    /// …**and once inside it, the same click selects the line.**
    #[test]
    fn inside_a_container_a_leaf_resolves_to_itself() {
        let ctx = egui::Context::default();
        let targets = stub();
        enter(
            &ctx,
            Entered {
                page: 0,
                form: 0,
                slot: 0,
            },
        );
        assert_eq!(
            scope(&ctx, 0).resolve(&targets, 0, TargetId::Leaf(0)),
            TargetId::Leaf(0)
        );
    }

    /// **The record is scoped to its page and its document**, and does not
    /// have to be cleared by whoever changes either.
    #[test]
    fn the_scope_does_not_follow_you_to_another_page_or_document() {
        let ctx = egui::Context::default();
        let targets = stub();
        enter(
            &ctx,
            Entered {
                page: 0,
                form: 0,
                slot: 0,
            },
        );
        assert!(entered(&ctx, 0, 0).is_some());
        assert!(entered(&ctx, 1, 0).is_none(), "another page");
        assert!(entered(&ctx, 0, 1).is_none(), "another document");

        // And the consequence, which is the part worth asserting: in the
        // OTHER document the same click resolves to the container again. The
        // record being filtered out is the mechanism; this is the behaviour.
        //
        // Asked of page 0 deliberately — the stub answers `containing_form`
        // for its own page only, exactly as the live provider does (it
        // decomposes one page), so asking about page 1 would be testing the
        // stub's page guard rather than this module's scope.
        assert_eq!(
            Scope {
                enabled: true,
                entered: entered(&ctx, 0, 1).map(|e| e.form),
            }
            .resolve(&targets, 0, TargetId::Leaf(0)),
            TargetId::Object(0),
            "the scope belongs to the document it was entered in"
        );
    }

    /// Switched off, nothing is substituted.
    #[test]
    fn with_the_option_off_a_leaf_is_a_leaf() {
        let ctx = egui::Context::default();
        let targets = stub();
        set_enabled(&ctx, false);
        assert_eq!(
            scope(&ctx, 0).resolve(&targets, 0, TargetId::Leaf(0)),
            TargetId::Leaf(0)
        );
    }

    /// **Switching it off leaves the container too.**
    ///
    /// A scope that outlived the mechanism that created it would make the next
    /// click resolve by a rule that is no longer switched on.
    #[test]
    fn switching_it_off_leaves_the_container() {
        let ctx = egui::Context::default();
        enter(
            &ctx,
            Entered {
                page: 0,
                form: 0,
                slot: 0,
            },
        );
        set_enabled(&ctx, false);
        assert!(entered(&ctx, 0, 0).is_none());
    }

    /// `leave` reports whether it did anything, which is what makes Escape a
    /// two-step gesture rather than one that clears everything at once.
    #[test]
    fn leaving_reports_whether_there_was_anything_to_leave() {
        let ctx = egui::Context::default();
        assert!(
            !leave(&ctx),
            "nothing entered, so Escape means something else"
        );
        enter(
            &ctx,
            Entered {
                page: 0,
                form: 0,
                slot: 0,
            },
        );
        assert!(leave(&ctx));
        assert!(!leave(&ctx));
    }

    /// A page object is never substituted, switched on or off.
    #[test]
    fn a_page_object_is_always_itself() {
        let ctx = egui::Context::default();
        let targets = stub();
        assert_eq!(
            scope(&ctx, 0).resolve(&targets, 0, TargetId::Object(0)),
            TargetId::Object(0)
        );
    }
}
