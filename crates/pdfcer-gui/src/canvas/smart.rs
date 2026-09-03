//! # `canvas::smart` — **click selects the whole thing; double-click goes
//! # inside it**
//!
//! ## The request
//!
//! `OPERATOR_REQUESTS.md` **O70**, 2026-08-31:
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
//! ★ He named the convention, so **the convention is the spec** — this shell's
//! standing rule about never inventing an interaction model. What follows is
//! Inkscape's group context, applied to the only container a PDF page actually
//! has.
//!
//! ## ★★★ What this changes, and it is the opposite of what it sounds like
//!
//! It sounds like *"add a way to go deeper"*. It is not. Before this module a
//! click on a title block selected **one line inside it**, because the deep hit
//! test returns a form XObject's interior and excludes the form itself:
//!
//! ```text
//! provider::hit_test_all → [Leaf(1180), Leaf(1181), …]     // never Object(the form)
//! ```
//!
//! ⇒ So the operator could reach the 10,256 leaves of a wrapped CAD drawing and
//! could not reach the drawing. The container was addressable only through a
//! Format-tab command (`format.select_form`) that they had to know existed.
//!
//! **This module makes a click select the container and a double-click enter
//! it**, which is what every drawing program in the class does and what he
//! described. The interior is not less reachable than it was — it is one
//! double-click away instead of zero — and the container is reachable at all
//! for the first time.
//!
//! ## The chain, and where each rung already lived
//!
//! | rung | reached by | who owns it |
//! |---|---|---|
//! | the **container** (a form XObject) | a click | this module |
//! | an **object inside it** (a leaf) | double-click *into* the container | this module |
//! | a **part** (a subpath, a text run) | double-click again | `SelectionState::descend` |
//! | a **node** (an anchor) | double-click again | `SelectionState::descend` |
//!
//! Only the first two rungs are new. The bottom two are the ladder this shell
//! has had since S4, and they are deliberately untouched: a container is a
//! **scope**, not a fourth `SelectionLevel`. Adding a rung above `Object` would
//! have meant re-reading every `match` on that enum and every assertion that
//! *"Object is the rung a click starts on and Escape returns to"*.
//!
//! ## ★★ Why a scope rather than a rung, in one sentence
//!
//! Because a form XObject **is a page object**. Selecting it is an ordinary
//! Object-rung selection of `TargetId::Object(paint_order)`; being *inside* it
//! is a fact about what the next click will resolve to, not about what is
//! selected. Inkscape models it the same way — the group context is a property
//! of the canvas, and the selection inside a group is an ordinary selection.
//!
//! ## Rule 4 — what is drawn and what is only said
//!
//! Entering a container draws **nothing extra on the page**. The selected
//! object gets the selection outline it would get anywhere, which is a cursor
//! affordance and explicitly allowed. *Which container you are inside* is
//! disclosed **off-canvas**, in the status row, exactly as rule 4 requires and
//! exactly where Inkscape puts it (*"Entered group g1234"*).
//!
//! A screenshot of the canvas while inside a container differs from a
//! screenshot of the same document saved and reopened only by where the pointer
//! is and what is selected. That is the one-line test, and it passes.
//!
//! ## Leaving, and why there are five ways
//!
//! Every one of these leaves the container, and each is somebody's habit:
//!
//! 1. **Escape**, as the last claimant on that key — one press clears the
//!    selection, a second leaves. See `canvas::keys` for why the scope is the
//!    outermost rung of one ladder rather than a competitor for the press.
//! 2. **A click outside the container's bounds** — Inkscape's other one.
//! 3. **Changing page.** The record names a page; carrying it to another sheet
//!    would silently scope clicks to a form that is not there.
//! 4. **Changing document.**
//! 5. **Leaving Edit mode**, because the whole gesture is content selection and
//!    `caps.edit_content` is what grants that.
//!
//! ★ 3–5 are enforced by **the record carrying its own page and document
//! epoch** rather than by five call sites remembering to clear it. That is the
//! `dialogs::placing` lesson applied one module over: a state that five routes
//! must remember to clear is a state one of them will forget.

use crate::panels::objects::provider::TargetId;

/// Whether Smart-Selector is on. Memory key.
///
/// ★ Application-scoped rather than per document, like the armed tool and the
/// pick filter: it is a statement about how this operator works, not about a
/// file, and re-answering it per document would be a question asked again for
/// no new reason.
const ENABLED_KEY: &str = "pdfcer.smart-select.enabled"; // ui-text-exempt: a memory key, never displayed

/// The container the pointer is currently working inside. Memory key.
const ENTERED_KEY: &str = "pdfcer.smart-select.entered"; // ui-text-exempt: a memory key, never displayed

/// **The container a click is currently scoped to.**
///
/// Carries the page and the document epoch with the object index for the reason
/// in the module header: the record invalidates itself rather than relying on
/// five call sites to clear it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entered {
    /// The page the container is on.
    pub page: usize,
    /// The container's **page paint-order index** — a form XObject.
    pub form: u64,
    /// Which open document this was recorded for.
    ///
    /// ★ The tab slot, published every frame by `crate::pagedrag::active`.
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
///
/// # ★★ Why the default is ON, when the operator asked for a checkbox
///
/// Because the checkbox exists so the behaviour can be turned **off**, and the
/// behaviour is what every program in the class does. A default of `false`
/// would ship the convention switched off and leave the complaint that produced
/// this row — *"I'm still not entirely clear how to reliably get to a point
/// where I can edit nodes"* — answered only for an operator who found a
/// checkbox first.
#[must_use]
pub fn enabled(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(id(ENABLED_KEY)))
        .unwrap_or(true)
}

/// Turn it on or off.
pub fn set_enabled(ctx: &egui::Context, on: bool) {
    ctx.data_mut(|d| d.insert_temp(id(ENABLED_KEY), on));
    if !on {
        // ★ Switching it off must also leave whatever container the operator
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
///
/// # ★★ Why this is not [`set_enabled`], which looks like the same function
///
/// `set_enabled` is what a **press** calls, and it also leaves whatever
/// container the operator is inside — because switching the mechanism off while
/// scoped to a container would leave the next click resolving by a rule that is
/// no longer switched on.
///
/// This runs every frame from `app::frame`, changed or not. Leaving on every
/// call would make entering a container impossible; leaving on every *change*
/// would make it identical to `set_enabled` and the two would not need to exist
/// separately. It does neither — it writes the value and nothing else, and the
/// consequence lives on the press path where the operator's intent is.
///
/// ⇒ The direction is strictly `Prefs` → memory. The only writer of the
/// persisted answer is the dispatch arm an operator's press runs, so there is
/// one source of truth and one mirror of it.
pub fn sync(ctx: &egui::Context, on: bool) {
    if enabled(ctx) != on {
        ctx.data_mut(|d| d.insert_temp(id(ENABLED_KEY), on));
    }
}

/// **The container the pointer is scoped to**, if the record is still valid for
/// this page and this document.
///
/// Returns `None` — and clears nothing — when the record names another page or
/// another document. Reading is not the place to write; the record is replaced
/// the next time one is written and is harmless meanwhile.
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
///
/// The `bool` is what makes Escape composable. `canvas::keys` consults this as
/// its **last** claimant — one press clears the selection, a second steps back
/// out of the container — which follows that ladder's own rule of retiring the
/// most transient thing first: a selection inside a title block is remade by
/// every click, while the fact that the operator is working inside it survives
/// all of them.
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
///
/// # ★★★ Why a value rather than reading the context at each call site
///
/// The pick helpers in [`crate::canvas::input`] are pure functions over a
/// `&dyn CanvasTargetProvider`, deliberately: they are the most heavily
/// unit-tested code in this crate and they answer *"what did this click
/// land on?"* without a running application. Handing them an `egui::Context`
/// to consult would make every one of those tests build a context and would
/// put a global read in the middle of a hit test.
///
/// ⇒ So the scope is read **once**, at the surface that has the context, and
/// travels as two facts. Every consumer then resolves identically by
/// construction — which is the property that matters, because a press and the
/// click that follows it must agree about what is under the pointer or a drag
/// starts on one thing and selects another.
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
    ///
    /// The whole rule, in one function so that the click path, the press path
    /// and any future caller cannot each have their own version of it:
    ///
    /// | switched on | target | inside its container | resolves to |
    /// |---|---|---|---|
    /// | no | anything | — | itself — the behaviour before O70, unchanged |
    /// | yes | a page object | — | itself |
    /// | yes | a **leaf** | no | its **outermost container** |
    /// | yes | a **leaf** | yes | itself |
    ///
    /// ★ The third row is the whole feature and the fourth is what stops it
    /// from being a cage: once you have entered a title block, clicking its
    /// lines selects its lines.
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
        // ★★★ **A CONTAINER THAT HOLDS EVERYTHING IS THE PAGE** — 2026-09-01,
        // and this guard repairs a defect this module CAUSED.
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
        // ★★ It shipped on 2026-08-31 and was found on 2026-09-01 by
        // `a_click_inside_a_form_selects_what_is_drawn_there` — a driven check
        // written for the FIRST occurrence, which had SKIPPED on a stale binary
        // through both sweeps in between. The check is why this cost a day
        // rather than a fortnight, and the skip is why it cost a day rather
        // than an hour.
        //
        // ★ Entering such a form is untouched. A double-click descends, the
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
///
/// ★ The document slot comes from `crate::pagedrag::active`, which the frame
/// publishes before any surface draws — the same source the Pages panel uses
/// to know which document it is showing, so *"which document is this?"* has
/// one answer in this crate rather than two.
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
    use crate::canvas::target::StubTargets;

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

    /// ★★★ **A click on something inside a container selects the container.**
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

    /// ★★★ …**and once inside it, the same click selects the line.**
    ///
    /// Without this the feature would be a cage: the operator could select a
    /// container and never anything in it, which is strictly worse than the
    /// behaviour it replaces.
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

    /// ★★ **The record is scoped to its page and its document**, and does not
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

        // ★ And the consequence, which is the part worth asserting: in the
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

    /// ★★ **Switching it off leaves the container too.**
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
