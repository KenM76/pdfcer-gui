//! # `app::modes::defaults` — what a mode's arrangement *is*
//!
//! One subject: the **default arrangement per mode**. Which panels Read,
//! Review and Edit mount, on which side, stacked with which others, and how
//! wide those docks start. It is a pure function from a mode id to a
//! [`DockLayout`], and nothing in it can reach a document, a dock, a layout
//! file or a [`super::Modes`].
//!
//! [`super`] owns the other half of the same feature — how an arrangement is
//! **remembered** once the operator has rearranged it.
//!
//! ## Why this is its own file
//!
//! Split from `app/modes.rs` when that file reached 1,512 lines against the
//! 1,500-line gate (R2). `app/mod.rs` has been split twice under the same
//! rule, into `crate::app::dispatch` and `crate::app::conditions`, and the
//! standing instruction for this gate is the one written into the gate
//! itself: *"the right response to this gate firing is to SPLIT THE MODULE,
//! not to shrink the prose."* This project's documentation is the logic, so
//! trimming it to fit is the one response that would make the file smaller
//! and the program less well specified.
//!
//! **The seam is a real one rather than arithmetic**, and the test for that
//! is whether the two halves change for different reasons. They demonstrably
//! do:
//!
//! * This file changes when the **information architecture** changes — when
//!   `MODES_AND_PANELS.md`'s table is amended, when a panel is invented, when
//!   the operator answers a taxonomy question. Its most recent change was
//!   exactly that: Read gained Forms on 2026-08-14 because the operator
//!   answered the question the module had been carrying.
//! * `super` changes when **persistence** changes — a new workspace naming
//!   rule, a new upgrade-reconciliation case, a different start-up order.
//!   Its most recent change was exactly that: the `Unseen` stamp, so a panel
//!   added in a new release is not born invisible.
//!
//! The two are also readable at different times. Someone asking *"why does
//! Read have no Objects panel"* never needs to know how a workspace is
//! named; someone asking *"why did my arrangement come back wrong after an
//! upgrade"* never needs the taxonomy. And the dependency runs one way only:
//! `super` calls [`layout_for_build`], while this file calls nothing in
//! `super` at all.
//!
//! ## ★ What `egui-shell` cannot supply
//!
//! `SHELL_FRAMEWORK.md` §4 and `egui-shell`'s workspace store between them
//! make Read/Review/Edit a *configuration* rather than a built-in — see
//! [`super`]'s header for that rule and for how [`super::Modes`] honours it.
//!
//! What *is* pdfcer's business, and therefore is here, is the **default
//! arrangement per mode** — see [`layout_for`]. `egui-shell` cannot supply
//! that: it does not know what a panel is for, so a default arrangement
//! invented there would be the framework inventing an application's
//! information architecture.
//!
//! ## The three defaults, and where they come from
//!
//! `MODES_AND_PANELS.md` Part 1's table, reduced to what the *dock* does:
//!
//! | Mode | Left | Right |
//! |---|---|---|
//! | **Read** | Pages, Bookmarks | Forms |
//! | **Review** | Pages, Bookmarks | Comments, Properties |
//! | **Edit** | Pages, Bookmarks / Layers, Signatures, Fonts | Objects / Properties, Comments |
//!
//! Read is the point of the whole feature — *"a PDF viewer, with pdfcer's
//! inspection panels available but nothing that authors anything"* — so its
//! default mounts the two surfaces that answer *where am I* and nothing that
//! merely describes an object you are not allowed to edit. **Forms on its
//! right is the one amendment to that sentence**, made on 2026-08-14 when
//! the operator answered the open question; the arms of `spec` carry the
//! full reasoning and it is not repeated here. Review adds the two surfaces
//! markup work needs. Edit is everything, with **Objects on the right**,
//! opposite the navigators, because an inspector and a navigator are
//! consulted in different directions.
//!
//! A mode this module has never heard of gets the **full** arrangement: a
//! mode with no opinion recorded about it should not have panels taken
//! away, because removing is the opinionated act.
//!
//! ## ★ Panels this build does not have
//!
//! **As of 2026-08-14, none — and [`ABSENT_PANELS`] is empty.** Both entries
//! that lived there have now landed, and the pair is worth keeping in view
//! because they failed in *opposite* directions and the same mechanism
//! caught both.
//!
//! **Comments** (`markup.comments`) was declared absent with the reason
//! *"annotation authoring does not exist yet, so neither does the panel that
//! lists comments"*. That was wrong on its merits: listing what a document
//! already carries needs no authoring, and the panel shipped against
//! `pdfcer_core::annot` while this shell still cannot place a single markup.
//! A blocker recorded from the wrong end held back a surface that was never
//! blocked. Note also that the id changed — the defaults named
//! `view.panel_comments` for the whole time nothing implemented it, and
//! `RIBBON_IA.md` §7's migration map puts the control on Markup ▸ Comments.
//! An id no code has ever resolved is a guess, and this one was wrong.
//!
//! **Pages** went the other way: the body existed and the *command* did not,
//! so it was correctly not an absent panel and still not reachable.
//!
//! Both are the `SHELL_FRAMEWORK.md` §5b mechanism rather than an oversight:
//! [`layout_for_build`] filters every default through the live
//! [`PanelCatalog`], so an id nothing registers is simply not mounted —
//! whether what is missing is the body or the command.
//!
//! Writing the *intended* arrangement and filtering it is strictly better
//! than writing only what exists today, because the alternative is that the
//! intent lives in a document nobody re-reads when the panel lands. The
//! Pages panel is the worked proof of that: it was built long after these
//! defaults were written, and the day its command is registered it appears
//! in all three with **no edit in this file at all**.
//! `every_default_panel_is_registered_or_declared_absent` is what keeps
//! [`ABSENT_PANELS`] honest in both directions.

use egui_shell::dock::{Column, DockLayout, PanelCatalog, PanelId, SideLayout, Stack};

use crate::panels::Panel;

/// **Panel ids the defaults name that this build does not register, and
/// why.**
///
/// `(id, reason)`, in the shape and for the reasons
/// `crate::shell::manifest::PLANNED` uses for absent *commands*: an
/// omission that is data can be tested, enumerated and grepped, whereas an
/// omission that is a comment becomes stale the day it stops being true.
///
/// Tested in both directions by
/// `every_default_panel_is_registered_or_declared_absent`: nothing in a
/// default layout may be missing from both `Panel::ALL` and this list, and
/// nothing in this list may already exist as a panel. So the day either
/// panel lands, the suite fails until this entry is removed — which is the
/// same commit in which the default starts mounting it.
pub const ABSENT_PANELS: &[(&str, &str)] = &[
    // ★ `view.panel_pages` WAS here, and its removal is what
    // `every_default_panel_is_registered_or_declared_absent` predicted:
    // *"the day either panel lands, the suite fails until this entry is
    // removed — which is the same commit in which the default starts
    // mounting it."* `crate::panels::Panel::Pages` now implements it, so the
    // entry had to go or that test would fail from the other direction.
    //
    // **The panel is still not reachable**, and the distinction is exactly
    // the one this list is for. It is no longer an *absent panel* — the body
    // exists, `Panel::ALL` enumerates it, and `layout_for` mounts it. What is
    // absent is the **command** `view.panel_pages`, which lives in
    // `crate::shell::manifest::PLANNED` and must be registered in
    // `crate::shell::commands` and referenced by a `View ▸ Panels` control
    // before `crate::app::PdfcerApp::new`'s panel registry will accept it. See
    // `crate::panels::pages`' header for the exact lines.
    //
    // Nothing here changes when that happens: `layout_for_build` filters
    // through the live catalog, so the panel appears in all three defaults on
    // the frame the command is registered and this file is untouched — which
    // is what this whole mechanism was for.
    // ★ `view.panel_comments` was the last entry, and it is gone for the
    // reason this list predicted of itself. It read:
    //
    //   "N — annotation authoring does not exist yet, so neither does the
    //    panel that lists comments. It is what Review's right dock is FOR,
    //    per Part 1's table, so the arrangement names it and mounts nothing
    //    until it is real."
    //
    // `crate::panels::Panel::Comments` landed 2026-08-14 and the panel is
    // reachable, so the entry had to go or
    // `every_default_panel_is_registered_or_declared_absent` would fail from
    // the other direction — exactly as it did for `view.panel_pages` before
    // it, and exactly as the doc comment above promised.
    //
    // Note the reason was ALSO wrong on its merits, and that is worth more
    // than the entry was. It said the panel waits on annotation *authoring*.
    // It did not: listing what a document already carries needs no authoring
    // at all, and the panel shipped against `pdfcer_core::annot` while this
    // shell still cannot place a single markup. A blocker recorded from the
    // wrong end delayed a surface that was never blocked.
    //
    // **The list is now empty, and empty is a valid state** — it means every
    // panel the defaults name exists. Do not delete the list: it is the
    // discipline, not the entries, and the next intended-but-unbuilt panel
    // belongs here rather than in a document nobody re-reads.
];

/// One side's default arrangement: a list of stacks, each a list of tabs.
///
/// A single column per side, deliberately. Multiple columns are what the
/// dock is *for* — a narrow navigator beside a wide inspector — but they
/// are an arrangement the operator reaches by widening and splitting, not
/// one to hand somebody on their first launch. The model expresses them;
/// the defaults do not use them.
///
/// **Owned rather than a `&'static` table**, for one reason worth stating
/// because the static form is the obvious first attempt and does not
/// compile: [`Panel::command_id`] is an ordinary function, so its result
/// cannot be promoted into a `'static` slice literal. The alternative is a
/// table of string literals plus a test asserting each one still matches
/// its panel — a second spelling of every id, kept in step by a test rather
/// than by construction. Two `Vec`s built on a mode change are cheaper than
/// that, in every sense.
type SideSpec = Vec<Vec<&'static str>>;

/// Both sides of one mode's default arrangement.
struct ModeSpec {
    /// The leading-edge dock's stacks, top to bottom.
    left: SideSpec,
    /// The trailing-edge dock's stacks, top to bottom.
    right: SideSpec,
    /// The left dock's width in points.
    left_width: f32,
    /// The right dock's width in points.
    right_width: f32,
}

/// The Comments panel's id.
///
/// ★ **Was `const COMMENTS: &str = "view.panel_comments"` until 2026-08-14**,
/// when the panel landed — and both halves of that line were wrong by then,
/// which is why this is a function like [`pages`] rather than a corrected
/// constant.
///
/// The *value* was wrong: the panel's command is `markup.comments`, because
/// `RIBBON_IA.md` §7's migration map sends the control to Markup ▸ Comments
/// by name, and a ruling about one control beats §5.2's list that merely
/// contains its name. The *form* was wrong for the reason [`pages`] records:
/// a literal here is a second spelling of an id that
/// [`Panel::command_id`] already owns, kept in step by a test instead of by
/// construction. Asking the panel is how the two cannot drift.
fn comments() -> &'static str {
    Panel::Comments.command_id()
}

/// The Pages panel's id.
///
/// A function rather than the `const` it used to be, because the panel now
/// exists and its id must come from [`Panel::command_id`] like every other
/// one — a second spelling of the same string is a second thing to keep in
/// step, and [`SideSpec`]'s own doc comment explains why that matters here.
///
/// It is a function and not an inline call only so the three arms below read
/// the same way they did, and so this doc comment has somewhere to live.
///
/// `pub(super)` rather than private: it stayed with the arrangements when
/// `app/modes.rs` was split, and `super`'s upgrade-reconciliation tests name
/// the Pages panel the same way its own arms do. Deliberately not `pub` —
/// outside this module the id comes from [`Panel::command_id`] directly.
pub(super) fn pages() -> &'static str {
    Panel::Pages.command_id()
}

/// The default width of a navigator dock, in points.
///
/// Wide enough for two columns of page thumbnails, which is the measurement
/// that decides this number: a thumbnail rail one column wide wastes the
/// dock, and three columns makes each too small to recognise a drawing by.
const NAVIGATOR_WIDTH: f32 = 280.0;

/// The default width of an inspector dock, in points.
///
/// Wider than a navigator because its rows are `label: value` pairs whose
/// values are paths, font names and coordinate triples — content that wraps
/// badly and reads terribly when it does.
///
/// This is Read's and Review's width. Edit's is [`EDIT_INSPECTOR_WIDTH`], and
/// the two being different constants is the whole of what *"remembered per
/// mode"* needs from this file — see that constant's ★★ section.
const INSPECTOR_WIDTH: f32 = 320.0;

/// The default width of **Edit's** inspector dock, in points —
/// `OPERATOR_REQUESTS.md` **O123**: *"Default dock width 360 px in Edit,
/// remembered per mode."*
///
/// ## ★★ "Remembered per mode" is already built, and this is the other half
///
/// A width is stored on [`egui_shell::dock::SideLayout::width_pts`], which is
/// per side, of a [`egui_shell::dock::DockLayout`], which is saved **per mode**
/// as a named workspace by `super::Modes::record_layout` every time the dock
/// reports `layout_changed`. So a splitter drag in Edit has never been able to
/// move Read's dock, and this change adds no mechanism — it changes what the
/// *unremembered* case starts from.
///
/// ⇒ Which is why it is a second constant and not a runtime branch: an
/// operator who has dragged Edit's dock is unaffected by either number, because
/// their saved workspace wins. This is only the first frame of a fresh profile.
///
/// ## ★★★ Why 360 rather than "as wide as the widest row"
///
/// Because no width fits every row and a dock that tried would be one nobody
/// wants. `SHELL_LAYOUT_PROPOSAL.md` §0.2 measured the real complaint: our
/// object rows already carry paint style, colour hex, line width, node count,
/// text preview, font name and size, image pixels **and a trailing diagnostic
/// note the mockup has no equivalent for**. They were never missing content;
/// they were being cut mid-character at 320 pt.
///
/// 360 stops the *common* row being cut. The uncommon one is elided with a
/// tooltip — see `crate::panels::objects`' row work, which is the other half of
/// O123 and the half that actually closes the defect.
///
/// ⚠ **Widening this is a harness re-baseline.** The canvas rect moves when the
/// right dock widens, so every canvas-relative click coordinate in
/// `tools/ui-verify` shifts. `SHELL_LAYOUT_PROPOSAL.md` §2.4 calls it *"the
/// single most under-estimated line in this document"*, and it is a one-line
/// constant change that is a suite-wide event.
const EDIT_INSPECTOR_WIDTH: f32 = 360.0;

/// The default arrangement for `mode_id`, **before** this build's panels
/// are taken into account.
///
/// The intended arrangement, naming every panel the mode is specified to
/// offer whether or not this build has it. Almost every caller wants
/// [`layout_for_build`] instead; this exists so the intent is expressible,
/// testable and readable on its own.
///
/// An unrecognised `mode_id` gets the full arrangement — see the module
/// header on why removing is the opinionated act.
#[must_use]
pub fn layout_for(mode_id: &str) -> DockLayout {
    build(&spec(mode_id), None)
}

/// The default arrangement for `mode_id`, with panels this build does not
/// register dropped and whatever they emptied pruned.
///
/// This is the one an application calls. `SHELL_FRAMEWORK.md` §5b: a
/// capability's presence is expressed by registering it and by nothing
/// else, so a default that mounts a panel nothing registers must mount
/// nothing rather than produce a tab whose body cannot be drawn.
///
/// The filter runs over the same [`PanelCatalog`] the dock and the layout
/// loader use, so "what a fresh profile starts with" and "what a saved
/// layout is allowed to contain" cannot disagree.
#[must_use]
pub fn layout_for_build(mode_id: &str, catalog: &dyn PanelCatalog) -> DockLayout {
    build(&spec(mode_id), Some(catalog))
}

/// The specification for one mode.
///
/// The `match` is the one place in this crate that knows what "read" means
/// as an *arrangement*. Note what it is not: it is not a list of the modes
/// that exist. [`super::Modes`] takes that from the manifest, so a mode with
/// no arm here still works — it simply starts from the full arrangement.
fn spec(mode_id: &str) -> ModeSpec {
    match mode_id {
        // Read — a reader. The two surfaces that answer "where am I", and
        // nothing that describes an object the mode does not let you touch.
        // No Objects, no Properties: Part 1's table gives Read neither, and
        // an inspector in a mode with no edit verbs is a panel whose every
        // row is a fact you cannot act on.
        //
        // ★ **Forms is here as of 2026-08-14, and it is the one exception
        // to the sentence above.** The operator answered the question this
        // module had been carrying: pdfcer should fill forms without leaving
        // Read, because Acrobat Reader does and replacing it is the stated
        // goal. So the taxonomy is amended openly, as the old note asked.
        //
        // It does not contradict the "no panel whose rows you cannot act
        // on" rule — it satisfies it. The rule keeps out surfaces that
        // *describe* what a mode gives you no verb for; Forms carries its
        // own verb in its own rows, and the fill verb moved to a tab Read
        // is shown (`view.panel_forms`, View ▸ Panels) so the panel can be
        // reopened after it is closed. Objects and Properties are still
        // out, and for the unchanged reason: their verbs live on tabs Read
        // does not have, and mounting them would put the operator in front
        // of a list of facts with nothing to do about any of them.
        //
        // It is mounted **closed-side**, in the right dock Read otherwise
        // does not have, rather than in the left navigator: a form is
        // something you work *on*, beside the page, and putting it left
        // would push Pages and Bookmarks — the two surfaces that answer
        // "where am I" — down a tab bar in the mode that needs them most.
        "read" => ModeSpec {
            left: vec![vec![pages(), Panel::Bookmarks.command_id()]],
            // ★★★ **COMMENTS IS MOUNTED IN READ, and it is the fix for the
            // report he made on 2026-09-05:**
            //
            // > *"I could add a yellow sticky note but even in read mode I
            // > don't think I could figure out how to read it."*
            //
            // He was right, and it was an absence rather than a
            // discoverability problem. Until this line, Read's dock held
            // Pages, Bookmarks and Forms — **no comment list at all** — and
            // the panel's only command sits on the Markup tab, which
            // `crate::shell::manifest`'s mode table does not show to Read.
            // Two independent barriers, so neither one alone was the bug and
            // fixing either alone would have left him exactly where he was.
            // The other half is `manifest::view`'s Panels group, which now
            // carries the toggle.
            //
            // ★★ **The argument is the Forms argument, and it is STRONGER
            // here.** Forms is mounted in Read on the operator's 2026-08-14
            // ruling, and that one had to overcome a real objection: filling
            // a field *writes to the file*. Reading a comment writes nothing.
            // If a mode whose stance is *the document is not yours to alter*
            // may nonetheless set `/V` and regenerate an appearance, it may
            // certainly read a `/Contents` string somebody else already
            // wrote.
            //
            // ⇒ The stance Read takes is about **authorship**, not about
            // information. Withholding the comment list confused *reading a
            // comment* with *writing one* — and Acrobat **Reader**, a
            // read-only product, is built around exactly this surface.
            //
            // Comments FIRST and Forms second, reversing the order Review
            // uses for the same two panels for the same reason it uses:
            // a tabbed stack draws only its active tab, so the panel the mode
            // is *for* goes at the front and the one that writes goes behind
            // it. In Review the front is the reviewer's work list; in Read the
            // front is the thing being read.
            right: vec![vec![comments(), Panel::Forms.command_id()]],
            left_width: NAVIGATOR_WIDTH,
            right_width: INSPECTOR_WIDTH,
        },
        // Review — the markup stance. Read's navigators, plus the two
        // surfaces markup work needs: the comment list you are working
        // through, and the properties of the markup you are placing.
        // Properties is scoped to markup in this mode by the *mode*, not by
        // the dock; the dock mounts one panel either way.
        // Forms is mounted here, and the argument is the same one that put
        // Pages in Review. Filling a field DOES write to the file — it sets
        // `/V` and regenerates an appearance — so this is not the "changes
        // nothing" case. It is the case where **the change is the one the
        // document's author invited**: a field exists in order to be filled,
        // and writing a value into it is using the file as designed rather
        // than altering what it says. That is the same stance markup takes,
        // which is why the two share a mode.
        //
        // ★ **It used to say "deliberately NOT in Read", and that question
        // was put to the operator and answered on 2026-08-14: Read fills
        // forms.** The note that stood here said what the change would cost
        // if the answer came back yes — *this line plus a fill verb on
        // Read's ribbon, amended openly rather than quietly bent* — and
        // that is exactly what it cost. The verb is `view.panel_forms`, on
        // View ▸ Panels, moved off the Edit tab because P1 gives a command
        // one tab and Read is shown `file` and `view` alone.
        //
        // Review's own argument for mounting Forms is untouched by that. It
        // was never "only Review may fill"; it was "filling is the change
        // the author invited, which is the Review stance". A stance Read
        // now shares is not a stance Review has lost.
        "review" => ModeSpec {
            left: vec![vec![pages(), Panel::Bookmarks.command_id()]],
            right: vec![
                // ★★★ **The Tool panel is gone** — `OPERATOR_REQUESTS.md` O123.
                //
                // It used to hold a stack of its own here, at the top, and the
                // argument was recorded at length: *"Its entire purpose is
                // being OFFERED rather than asked for … A tab that is
                // invisible until clicked cannot fix a discoverability
                // defect."* That argument was about a **tab**, and it is not
                // what replaced the panel: `crate::app::toolstatus` is a
                // permanent strip the dock reserves above these columns, which
                // is offered harder than a stack was — it cannot be closed at
                // all.
                //
                // The panel's live controls did not go with it. They are in
                // `crate::panels::properties::tool`, one stack down, which is
                // the whole of the operator's *"everything can be in object
                // and properties."*
                vec![
                    comments(),
                    Panel::Properties.command_id(),
                    Panel::Forms.command_id(),
                    // ★ Dimension groups, and Review gets it for the reason the
                    // mode taxonomy already settled: Review is shown the `measure`
                    // tab, so it may author a ce dimension, and a mode that can
                    // author one must be able to say which group it joins. The
                    // panel's command lives on that tab (`measure.manage_groups`),
                    // so this mode can reopen it after closing it — the trap
                    // `Panel::Forms` had to move off the Edit tab to escape.
                    //
                    // Last in the stack, like Redact in Edit's: a tabbed stack
                    // draws only its active tab, so a panel at the end is
                    // reachable in one click and invisible until asked for. Group
                    // setup is not what a reviewer opens Review to do.
                    Panel::DimensionGroups.command_id(),
                ],
            ],
            left_width: NAVIGATOR_WIDTH,
            right_width: INSPECTOR_WIDTH,
        },
        // Edit — everything. Two stacks per side rather than one long tab
        // bar, because the previous implementation's reasoning still holds:
        // *"reaching one surface must not hide another you are using AT THE
        // SAME TIME"*. Navigating pages while reading the layer list is one
        // such pair; picking an object while reading its properties is the
        // other, and it is why Objects and Properties are separate stacks
        // rather than two tabs of one.
        "edit" => ModeSpec {
            // ★★★ **ONE stack, five tabs** — `OPERATOR_REQUESTS.md` O123:
            // *"Layers, Signatures and Fonts join Pages and Bookmarks as tabs
            // in one dock instead of a second dock with a fixed split."*
            //
            // It used to be two stacks with a splitter between them, on the
            // rule *"reaching one surface must not hide another you are using
            // AT THE SAME TIME"* — navigating pages while reading the layer
            // list being the named pair. The operator has ruled the other way,
            // and the cost he is buying is real: a second stack is a second
            // tab bar plus `plan::MIN_STACK_HEIGHT` of floor, and it spent
            // that on a pair nobody had reported using together.
            //
            // ★ Five tabs is where `plan`'s overflow affordance starts to
            // matter at a 280 pt navigator, and that is the same three-rung
            // ladder `RIBBON_SCALING.md` documents for the band. It is the
            // dock's own mechanism, already built and already checked by
            // failure mode #8 — not a second answer invented here.
            left: vec![vec![
                pages(),
                Panel::Bookmarks.command_id(),
                Panel::Layers.command_id(),
                Panel::Signatures.command_id(),
                Panel::Fonts.command_id(),
            ]],
            right: vec![
                // ★★★ **Objects over Properties is the master–detail pair, and
                // it now has the whole side** — `OPERATOR_REQUESTS.md` O123.
                //
                // > *"Objects and Properties become master–detail in one panel
                // > with a draggable split … I'd also like those one to appear
                // > in the space where the tool dock currently shown."*
                //
                // Two adjacent stacks in ONE column is exactly that shape, and
                // it is what this dock already builds: `SHELL_LAYOUT_PROPOSAL`
                // §2.1 — *"We already ship a master–detail, and it is already a
                // vertical pair with a draggable split."* The split is
                // `egui_shell::dock`'s own stack splitter, dragged at
                // `dock/mod.rs`, floored at `plan::MIN_STACK_HEIGHT`.
                //
                // ★ So what O123 changes here is **room, not linkage**. The
                // Tool panel's stack was taking a third of the side; deleting
                // it hands that third to these two. A row click already raises
                // `Action::SelectObject` and Properties already reads the same
                // canvas selection — since 2026-08-26, and neither end is
                // touched by this change.
                vec![Panel::Objects.command_id()],
                vec![
                    Panel::Properties.command_id(),
                    comments(),
                    Panel::Forms.command_id(),
                    // ★ Redact, and it is deliberately the LAST tab of this
                    // stack rather than a stack of its own or the first of
                    // this one — 2026-08-15.
                    //
                    // "Edit is everything" is this module's rule and
                    // `the_three_defaults_are_the_specified_arrangements`
                    // enforces it, so the panel has to be mounted somewhere in
                    // this arrangement. *Where* is the decision, and it is
                    // about what an operator sees on the frame Edit opens: a
                    // tabbed stack draws only its active tab, so a panel at the
                    // end of one is **reachable in a click and invisible until
                    // asked for**. A stack of its own would put a surface whose
                    // whole subject is permanent removal on screen, unasked,
                    // every time anybody entered Edit mode.
                    //
                    // Edit is also the only arrangement it appears in, and that
                    // is not a placement decision but a consequence: the toggle
                    // is `edit.redact`, on the Edit tab, and Read and Review are
                    // not shown that tab. A mode that mounted this panel could
                    // not reopen it after closing it — the trap
                    // `crate::panels::Panel::Forms` had to be moved off Edit to
                    // escape, arriving here from the other direction.
                    Panel::Redact.command_id(),
                    // Dimension groups, after Redact, and the ordering is not
                    // a ranking — it is arrival order within a stack whose
                    // tabs are all "asked for, not offered". See Review's arm
                    // for why this panel is mounted at all.
                    Panel::DimensionGroups.command_id(),
                    // ★ Attachments, after Dimension groups, and in **Edit
                    // alone**. Both facts follow from the same rule the two
                    // above it follow rather than from a new judgment:
                    //
                    // *Where* — the end of the "asked for, not offered" stack,
                    // because a tabbed stack draws only its active tab, and a
                    // document's embedded files are not what anybody opens Edit
                    // mode to look at.
                    //
                    // *Only here* — its toggle is `edit.attachments`, on the
                    // Edit tab, and Read and Review are not shown that tab. A
                    // mode that mounted this panel could not reopen it after
                    // closing it, which is the trap `crate::panels::Panel::Forms`
                    // had to move off Edit to escape.
                    Panel::Attachments.command_id(),
                ],
            ],
            left_width: NAVIGATOR_WIDTH,
            right_width: EDIT_INSPECTOR_WIDTH,
        },
        // A mode this module has no opinion about. The full arrangement,
        // for the reason in the module header — and it is reachable: an
        // operator's customized manifest may declare a fourth mode, and a
        // fourth mode with an empty dock would look like a broken build.
        _ => spec("edit"),
    }
}

/// Turn a specification into a layout, optionally filtered by a catalog.
fn build(spec: &ModeSpec, catalog: Option<&dyn PanelCatalog>) -> DockLayout {
    let mut layout = DockLayout::new(
        side(&spec.left, spec.left_width, catalog),
        side(&spec.right, spec.right_width, catalog),
    );
    // Cheap, and it is what lets `layout_for` be asserted `is_normalized`
    // rather than relying on `DockState::new` to repair a compiled-in
    // constant — the posture the dock's own docs ask an application to
    // take towards its defaults.
    layout.normalize();
    layout
}

/// Build one side, dropping unregistered panels and pruning what they
/// empty.
fn side(
    stacks: &[Vec<&'static str>],
    width: f32,
    catalog: Option<&dyn PanelCatalog>,
) -> SideLayout {
    let kept: Vec<Stack> = stacks
        .iter()
        .filter_map(|tabs| {
            let tabs: Vec<PanelId> = tabs
                .iter()
                .copied()
                .filter(|id| catalog.is_none_or(|c| c.contains(id)))
                .map(PanelId::new)
                .collect();
            (!tabs.is_empty()).then(|| Stack::tabbed(tabs))
        })
        .collect();

    if kept.is_empty() {
        // Not an empty visible side: a side with no columns that is still
        // marked visible is how an application ends up with a permanent
        // grey stripe nobody can remove.
        return SideLayout::none();
    }
    SideLayout::new([Column::new(kept)]).with_width(width)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_shell::dock::{DockSide, PanelInfo, PanelRegistry};

    /// The panel registry a full build would have: every panel this crate
    /// actually implements.
    ///
    /// Duplicated in `super`'s own test module rather than shared, because
    /// a `#[cfg(test)]` helper reachable across module boundaries has to be
    /// made visible in the non-test build too. Six lines of fixture is the
    /// cheaper of the two costs.
    fn registry() -> PanelRegistry {
        let mut r = PanelRegistry::new();
        for panel in Panel::ALL {
            let id = panel.command_id();
            r.register(PanelInfo::new(id, id));
        }
        r
    }

    /// Every panel id in a mode's default layout.
    fn ids(layout: &DockLayout) -> Vec<String> {
        layout.panels().map(|p| p.as_str().to_owned()).collect()
    }

    /// ★★★ **Edit's inspector starts at 360 pt and the reading stances start
    /// at 320** — `OPERATOR_REQUESTS.md` O123, part 6.
    ///
    /// Asserted per mode rather than as one constant, because *"remembered per
    /// mode"* is the operator's phrase and the failure it guards against is the
    /// tempting one-line version: bumping `INSPECTOR_WIDTH` alone, which would
    /// widen Read's and Review's docks too and take that room from the page in
    /// the two modes whose whole subject is the page.
    ///
    /// ★ The left widths are asserted in the same test on purpose. Edit's left
    /// side became ONE stack of five tabs in this change, and a five-tab bar in
    /// a 280 pt navigator is where the dock's overflow affordance starts to
    /// matter — so a future widening of the navigator is a decision somebody
    /// should have to change a test to make.
    #[test]
    fn the_inspector_is_wider_in_edit_than_in_the_reading_stances() {
        let edit = layout_for("edit");
        assert!(
            (edit.right.width_pts - 360.0).abs() < f32::EPSILON,
            "Edit's inspector is {} pt, not the 360 O123 asks for",
            edit.right.width_pts
        );
        for reading in ["read", "review"] {
            let layout = layout_for(reading);
            assert!(
                (layout.right.width_pts - 320.0).abs() < f32::EPSILON,
                "{reading}'s inspector moved to {} pt; O123 widened Edit alone",
                layout.right.width_pts
            );
        }
        for mode in ["read", "review", "edit"] {
            let layout = layout_for(mode);
            assert!(
                (layout.left.width_pts - 280.0).abs() < f32::EPSILON,
                "{mode}'s navigator is {} pt",
                layout.left.width_pts
            );
        }
    }

    /// ★★★ **Edit's left side is ONE stack, and it holds all five navigators**
    /// — `OPERATOR_REQUESTS.md` O123, part 5.
    ///
    /// > *"Layers, Signatures and Fonts join Pages and Bookmarks as tabs in one
    /// > dock instead of a second dock with a fixed split."*
    ///
    /// The count is asserted as well as the membership, and the count is the
    /// half that matters: a build that put all five panels back into two stacks
    /// would satisfy a membership assertion exactly, and would be the fixed
    /// split he asked to be rid of.
    #[test]
    fn edits_navigators_share_one_stack() {
        let edit = layout_for("edit");
        let stacks: usize = edit.left.columns.iter().map(|c| c.stacks.len()).sum();
        assert_eq!(stacks, 1, "Edit's left side is not one stack");
        assert_eq!(
            edit.left.panels().map(PanelId::as_str).collect::<Vec<_>>(),
            [
                pages(),
                Panel::Bookmarks.command_id(),
                Panel::Layers.command_id(),
                Panel::Signatures.command_id(),
                Panel::Fonts.command_id(),
            ],
            "the five navigators, in one stack, in reading order"
        );
    }

    /// ★★★ **Objects sits directly above Properties on Edit's right side, and
    /// nothing sits above them** — `OPERATOR_REQUESTS.md` O123, parts 3 and 4.
    ///
    /// > *"Objects and Properties become master–detail in one panel with a
    /// > draggable split … I'd also like those one to appear in the space where
    /// > the tool dock currently shown."*
    ///
    /// Two adjacent stacks in one column is the master–detail shape, and the
    /// split between them is the dock's own draggable stack splitter. What this
    /// test pins is the part a refactor could undo without anybody noticing:
    /// that **Objects is the first stack**, which is only true because the Tool
    /// panel's stack was removed rather than merely emptied.
    #[test]
    fn edits_right_side_is_objects_over_properties() {
        let edit = layout_for("edit");
        assert_eq!(edit.right.columns.len(), 1, "one column, two stacks");
        let stacks = &edit.right.columns[0].stacks;
        assert_eq!(stacks.len(), 2, "master over detail, and nothing else");
        assert_eq!(
            stacks[0].tabs.first().map(PanelId::as_str),
            Some(Panel::Objects.command_id()),
            "Objects must be the master, at the top of the side"
        );
        assert_eq!(
            stacks[1].tabs.first().map(PanelId::as_str),
            Some(Panel::Properties.command_id()),
            "Properties must be the detail, directly under it"
        );
    }

    /// ★ **Each mode's default is the arrangement `MODES_AND_PANELS.md`
    /// specifies.**
    ///
    /// Asserted on the *unfiltered* defaults, because that is where the
    /// intent lives: filtering is what this build's panel set does to it,
    /// and asserting the filtered form would make the test say less every
    /// time a panel is missing.
    #[test]
    fn the_three_defaults_are_the_specified_arrangements() {
        let read = layout_for("read");
        assert_eq!(
            read.left.panels().map(PanelId::as_str).collect::<Vec<_>>(),
            [pages(), Panel::Bookmarks.command_id()],
            "Read's navigators, unchanged by the forms amendment"
        );
        assert_eq!(
            read.right.panels().map(PanelId::as_str).collect::<Vec<_>>(),
            [comments(), Panel::Forms.command_id()],
            "Read's inspector side is the comment list, then Forms"
        );
        for absent in [Panel::Objects, Panel::Properties] {
            assert!(
                !read.contains(&PanelId::new(absent.command_id())),
                "{absent:?} must not be in Read's default"
            );
        }

        let review = layout_for("review");
        assert!(review.left.panels().any(|p| p.as_str() == pages()));
        assert_eq!(
            review
                .right
                .panels()
                .map(PanelId::as_str)
                .collect::<Vec<_>>(),
            [
                comments(),
                Panel::Properties.command_id(),
                Panel::Forms.command_id(),
                Panel::DimensionGroups.command_id(),
            ],
            "Review's inspector side, in order, and nothing else"
        );
        // Objects stays out, and that is the line Forms had to be argued
        // across rather than waved across: Review mounts what the document
        // INVITES you to add — a comment, a field value — and not an
        // inspector for content the mode gives you no verb to change.
        assert!(!review.contains(&PanelId::new(Panel::Objects.command_id())));
        // ★ **Read has Forms and still has no Objects**, and asserting the
        // pair together is the point. The rule was never "Read mounts
        // nothing that writes" — it is "Read mounts nothing whose rows it
        // gives you no verb for". Forms carries its verb in its own rows
        // and its toggle is on a tab Read is shown; Objects' verbs are on
        // the Edit tab, which Read is not.
        //
        // The assertion that would be wrong here is the tempting one — that
        // Read's right side is empty — because it passes on a build that
        // dropped Forms for the old reason, which the operator overruled.
        assert!(
            !layout_for("read").contains(&PanelId::new(Panel::Objects.command_id())),
            "Read has no verb for an object, so an inspector for one lists facts nobody can act on"
        );

        let edit = layout_for("edit");
        for panel in Panel::ALL {
            assert!(
                edit.contains(&PanelId::new(panel.command_id())),
                "Edit is everything, and {panel:?} is missing"
            );
        }
        let objects = edit
            .find(&PanelId::new(Panel::Objects.command_id()))
            .expect("Objects is mounted");
        assert_eq!(objects.side, DockSide::Right, "Objects is on the right");
    }

    /// ★★★ **Read can read the comments — both halves of it.**
    ///
    /// # The report this exists for
    ///
    /// Ken, 2026-09-05: *"I could add a yellow sticky note but even in read
    /// mode I don't think I could figure out how to read it."*
    ///
    /// He was right, and it was an **absence**, not a discoverability problem.
    /// Two independent barriers stood between him and a comment he had just
    /// written, and each one alone was sufficient:
    ///
    /// 1. Read's default dock held Pages, Bookmarks and Forms. **No comment
    ///    list was mounted at all.**
    /// 2. The panel's only command, `markup.comments`, sits on the **Markup**
    ///    tab, and the mode table shows Read `["file", "view"]`. **So the
    ///    toggle could not be reached to fix (1) by hand.**
    ///
    /// ⇒ ★★ **This test asserts BOTH**, deliberately in one place, because
    /// that is the property that was violated. Two separate tests, each
    /// passing, would each have been green on a build where he still could
    /// not read his note — a barrier removed while another remains is
    /// indistinguishable, from his chair, from nothing having been done. This
    /// project has a standing lesson for that shape: *an absence claim is a
    /// claim about EVERY route.*
    ///
    /// # What it is NOT
    ///
    /// It is not a claim that the popup on the canvas works, or that the
    /// panel renders the words. It says the surface is **mounted and
    /// reachable in Read**, which is the barrier this pair of lines removed.
    /// A rendered screenshot is still the only oracle for the rest.
    #[test]
    fn read_mode_can_reach_the_comment_list_by_both_routes() {
        let read = layout_for("read");
        assert!(
            read.contains(&PanelId::new(comments())),
            "Read's default dock does not mount the comment list, so a note \
             written in Review cannot be read in Read"
        );

        // ★★ **Route two is the RAIL, and the reason it is not the View tab
        // is a rule that bites much harder than it looks.**
        //
        // The obvious placement — `markup.comments` beside the other panel
        // toggles in View ▸ Panels — is a **manifest validation failure**.
        // `RIBBON_IA.md` P1 says one command appears on at most one tab, and
        // `Shell::validate` enforces it. Worse, the failure is not local:
        // `Capabilities::for_mode` returns `FULL` when the shell is absent, so
        // an invalid manifest silently grants every authoring capability to
        // every mode. It was tried on 2026-09-05 and eight mode-gating tests
        // went red at once, one of them reading *"the pen is never picked up
        // in Read"*.
        //
        // The rail is not a tab, so P1 does not reach it — the same permission
        // four `view.panel_*` toggles and `file.fonts` already rely on there.
        // It is also present in **every** mode, which a tab is not, and that
        // is the property this needs.
        //
        // Asked of the rail's own definition rather than restated here: a
        // hand-written copy of the group's contents would be a second spelling
        // that can drift from the one the shell builds.
        let on_the_rail = crate::shell::manifest::rail::groups()
            .into_iter()
            .flat_map(|g| g.items)
            .any(|item| {
                matches!(item, egui_shell::manifest::Item::Command { ref id, .. } if id == comments())
            });
        assert!(
            on_the_rail,
            "{} is on no rail group, so in Read — which is shown `file` and \
             `view` alone — the panel cannot be reopened once it is closed",
            comments()
        );
    }

    /// A mode with no arm gets the full arrangement rather than an empty
    /// dock — a customized manifest's fourth mode must not look broken.
    #[test]
    fn an_unrecognised_mode_gets_the_full_arrangement() {
        assert_eq!(ids(&layout_for("proofing")), ids(&layout_for("edit")));
        assert_eq!(ids(&layout_for("")), ids(&layout_for("edit")));
    }

    /// Every default is already normalized, so a defect in a compiled-in
    /// constant fails here rather than being quietly patched on every
    /// machine that runs it.
    #[test]
    fn every_default_is_already_normalized() {
        for mode in ["read", "review", "edit", "something-else"] {
            let layout = layout_for(mode);
            assert!(layout.is_normalized(), "{mode} needed repair: {layout:?}");
            assert!(
                layout_for_build(mode, &registry()).is_normalized(),
                "{mode}, filtered, needed repair"
            );
        }
    }

    /// ★ **A panel this build does not have is not mounted, and takes
    /// nothing else with it.**
    ///
    /// `SHELL_FRAMEWORK.md` §5b applied to the *defaults* rather than to a
    /// saved file: the intended arrangement names Pages and Comments, this
    /// build registers neither, and what the operator gets is the rest of
    /// the arrangement — never a tab whose body cannot be drawn, and never
    /// an empty compartment where one used to be.
    #[test]
    fn a_default_drops_panels_this_build_does_not_register() {
        let registry = registry();
        // `registry()` is *"the panel registry a full build would have"* —
        // every panel this crate implements, which now includes Pages. The
        // live build's registry is smaller, because it drops any panel whose
        // command is unregistered; that filtering is the same code path, and
        // asserting it here would test `PdfcerApp::new`'s registry rather than
        // this module's arrangement.
        let read = layout_for_build("read", &registry);
        assert_eq!(
            ids(&read),
            [
                Panel::Pages.command_id(),
                Panel::Bookmarks.command_id(),
                // ★ Comments joined Read's default on 2026-09-05 — his report
                // that a sticky note could not be read in Read. Listed here
                // because this assertion is a *literal transcript* of the
                // arrangement, which is the point of it: the arrangement is
                // the spec, so a change to it must be restated here by hand
                // and cannot slip through as an incidental.
                Panel::Comments.command_id(),
                Panel::Forms.command_id()
            ]
        );

        // …and a registry WITHOUT Pages still drops it, which is the property
        // the live build depends on today.
        let mut without_pages = PanelRegistry::new();
        for panel in Panel::ALL.into_iter().filter(|p| *p != Panel::Pages) {
            let id = panel.command_id();
            without_pages.register(PanelInfo::new(id, id));
        }
        assert_eq!(
            ids(&layout_for_build("read", &without_pages)),
            [
                Panel::Bookmarks.command_id(),
                Panel::Comments.command_id(),
                Panel::Forms.command_id()
            ],
            "a panel the catalog does not hold must not be mounted"
        );

        // ★ This assertion used to read "Comments went; Properties and Forms
        // stayed", and Comments going was the *point* of it — the panel was
        // declared in `ABSENT_PANELS`, so a real absent panel demonstrated
        // the filtering against the real registry.
        //
        // The Comments panel landed 2026-08-14 and `ABSENT_PANELS` is now
        // empty, so **nothing real is filtered here any more**. That is a
        // better state to be in and a weaker test, and both halves are worth
        // saying: the property is still proven above, by the constructed
        // `without_pages` registry, which is the form it must keep now that
        // there is no absent panel to borrow. If a future panel is declared
        // absent, this is the assertion that will notice.
        let review = layout_for_build("review", &registry);
        assert_eq!(
            review
                .right
                .panels()
                .map(PanelId::as_str)
                .collect::<Vec<_>>(),
            [
                comments(),
                Panel::Properties.command_id(),
                Panel::Forms.command_id(),
                Panel::DimensionGroups.command_id(),
            ],
            "every panel Review's default names now exists, so none is filtered"
        );

        // Read's whole left side is Pages + Bookmarks. A build with neither
        // must produce a side that draws NOTHING rather than an empty
        // bordered stripe nobody can remove.
        let empty = PanelRegistry::new();
        let bare = layout_for_build("read", &empty);
        assert!(bare.left.is_empty() && bare.right.is_empty());
        assert!(!bare.left.visible, "an empty side must not be visible");
    }

    /// ★ **Every panel a default names either exists or is declared
    /// absent.**
    ///
    /// [`ABSENT_PANELS`] is the `PLANNED` discipline applied to panels, and
    /// this is what keeps it honest in both directions: a default may not
    /// name an id that is neither implemented nor declared absent, and an
    /// id declared absent may not already exist. The second half is the one
    /// that matters over time — it makes the day a Pages panel lands a
    /// failing test rather than a stale comment.
    #[test]
    fn every_default_panel_is_registered_or_declared_absent() {
        let implemented: Vec<&str> = Panel::ALL.iter().map(|p| p.command_id()).collect();

        for mode in ["read", "review", "edit"] {
            for id in ids(&layout_for(mode)) {
                assert!(
                    implemented.contains(&id.as_str())
                        || ABSENT_PANELS.iter().any(|(absent, _)| *absent == id),
                    "`{id}` is mounted by {mode}'s default, is not a panel this build \
                     implements, and is not declared in ABSENT_PANELS. Implement it, \
                     remove it from the default, or declare it absent with a reason."
                );
            }
        }

        for (id, reason) in ABSENT_PANELS {
            assert!(
                !implemented.contains(id),
                "`{id}` is declared absent and yet `Panel::ALL` implements it. Remove \
                 the ABSENT_PANELS entry — the defaults will start mounting it."
            );
            assert!(
                !reason.is_empty(),
                "`{id}` is declared absent with no reason, which is the stale comment \
                 this list exists to replace"
            );
        }
    }
}
