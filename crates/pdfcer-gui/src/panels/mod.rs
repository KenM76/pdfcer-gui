//! # `panels` — the dock's panel bodies
//!
//! **Thirteen** panels, each a **function the dock can call**. This module owns the
//! set, the dispatch, the little state the bodies share, and the two layout
//! rules that every one of them has to get right.
//!
//! | Panel | Ribbon command | Salvaged from |
//! |---|---|---|
//! | [`attachments`] | `edit.attachments` — **new**; see that variant for the tab argument | **new** — no old-shell surface existed |
//! | [`bookmarks`] | `view.panel_bookmarks` | `panels_structure.rs` |
//! | [`layers`] | `view.panel_layers` | `panels_structure.rs` |
//! | [`signatures`] | `view.panel_signatures` | `panels_structure.rs` |
//! | [`fonts`] | `file.fonts` | `panels_structure.rs` |
//! | [`objects`] | `view.panel_objects` | `main.rs` + `object_provider.rs` + `object_summary.rs` |
//! | [`properties`] | `file.properties` | **new** — `RIBBON_IA.md` §5.8 |
//! | [`docprops`] | `file.document_properties` — **new 2026-09-05**; see that variant | **new** — was the last section of [`properties`] |
//! | [`forms`] | `view.panel_forms` — moved off Edit so Read can reach it | `panels_forms.rs` |
//! | [`pages`] | `view.panel_pages` — **not registered; see that module** | `main.rs::thumbnail_rail` + `raster::ThumbnailCache` |
//! | [`comments`] | `markup.comments` — **not** a `view.panel_*` id; see that variant | `main.rs::comments_panel` |
//! | [`redact`] | `edit.redact` — the reversible half of redaction; its irreversible twin is [`crate::dialogs::redact`] | `main.rs::redact_panel` |
//!
//! ## ★ These panels once had no way in
//!
//! Recorded here because this is now the file someone reads when they touch
//! them. The old shell's `panels_structure.rs` header:
//!
//! > All three shipped with a `PaneSubject`, a panel body, a rail entry and
//! > a diagnostic step — and no control an operator could click. Their only
//! > callers were the harness step handlers, so every verification passed
//! > while the panels were unreachable in a real build.
//!
//! That is what [`Panel::command_id`] and
//! [`tests::every_panel_is_reachable_from_the_ribbon`] exist for, and the
//! check here is stronger than the one it replaces. The old gate was a
//! **source-text grep** — it read `main.rs` as a string and looked for a
//! `show_pane_subject(…)` call outside the harness function. This one asks
//! the shell manifest whether a real ribbon command names the panel, and
//! asks the command registry whether that command exists, so it is
//! satisfied by the same data the ribbon draws itself from rather than by
//! the presence of a substring.
//!
//! A panel added here without a ribbon command does not compile a warning;
//! it fails a test with the name of the panel in the message.
//!
//! ## Actions, not mutations — and it still has teeth
//!
//! A panel body never touches a document. It is handed `&OpenDoc` — a
//! **shared** reference, so this is a compile-time fact and not a
//! convention — it reads, and it pushes a
//! [`crate::app::actions::Action`]. `PROJECT_PLAN.md` §3 lists this first
//! among the invariants that are *"not up for renegotiation"*, and
//! `crate::app::actions`' own header explains why retrofitting it is
//! expensive.
//!
//! **Three** panels can act on the document, and the count is worth stating
//! plainly rather than discovering. Bookmarks pushes [`Action::GoToPage`].
//! Layers pushes [`Action::SetLayerVisible`] and [`Action::ResetLayers`],
//! which arrived at S4 and are what restored its visibility checkbox.
//! [`comments`] pushes [`Action::GoToPage`] as well, and *only* that: the old
//! shell's Comments panel could also delete an annotation, and that half is
//! deliberately absent here because no [`Action`] variant can carry the
//! intent — see that module's header for what the day it lands needs. Every
//! other panel is still a report — and where the old shell had a control that
//! this build does not (the Fonts unembed and embed buttons), that panel's
//! own module docs say which control is missing and what it is waiting for,
//! because a control with no action behind it is an affordance for something
//! that cannot work (`RIBBON_IA.md` P3, R83).
//!
//! ### ★ A note for whoever restores one of the remaining controls
//!
//! The Layers checkbox is the worked example, and its three preconditions are
//! written up in [`layers`]' own header. In summary: the renderer had to
//! accept an override (it always did), the render worker's cache key had to
//! vary with it (S4: `RenderKey` carries `layers_generation`, and
//! `crate::app::state::OpenDoc` carries the override plus
//! `set_layer_visible`, `set_hidden_layers`, `reset_layers`), and an
//! [`Action`] variant had to carry the intent from the panel to `apply`
//! (S4, last to land). **A control that is missing any one of the three
//! renders nothing at all** rather than shipping and looking broken.
//!
//! It is tempting, seeing an override behind a `RefCell` on `OpenDoc`, to
//! reach for interior mutability and let the panel toggle it through the
//! shared reference. **Do not.** The `RefCell`s there hold *derived caches*,
//! whose filling nothing can observe; layer visibility is state that decides
//! what appears on the page. Mutating it from a widget would make "what can
//! change what is drawn?" un-greppable, which is the fourth of the four
//! properties `crate::app::actions`' header says the funnel buys.
//!
//! Note also that a panel may raise **several** actions for one gesture, and
//! that this is the intended shape rather than a workaround: the Layers
//! panel's `/RBGroups` radio behaviour is one `Action::SetLayerVisible` per
//! layer that moves, applied in order, each recomputing from the state the
//! previous one left. That keeps one greppable action per changed layer
//! instead of a single variant carrying an opaque set.
//!
//! ## Rule 4 lives here
//!
//! `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`'s first
//! non-negotiable, in one clause:
//!
//! > **Disclosure lives off-canvas**: a status line, a results panel, a
//! > report after the command, a properties field. Never blocking, never
//! > requiring acknowledgement, never positioned relative to the document.
//!
//! A panel is the *right home* for everything pdfcer inferred — a substituted
//! font, a best-fit residual, a snapped point, an approximate text extent —
//! and the page view must carry none of it. No badge, no tint, no dashed
//! outline, no "provisional" layer. Nothing in this module draws on the
//! canvas, and nothing in it may start to: the one-line test is *would a
//! screenshot of the editing canvas differ from a screenshot of the same
//! document saved and reopened?*
//!
//! [`objects::summary::ObjectSummary::bounds_are_approximate`] is where that
//! bit: in the old shell it drove a **dashed outline on the page**. It
//! survives as a question, and its answer is now a sentence in
//! [`properties`].
//!
//! ## Two layout rules every panel obeys
//!
//! ### 1. Scrollbars must be visible
//!
//! egui's default `ScrollStyle` is `floating()`: a 2 pt sliver that
//! allocates **zero** space and has `dormant_handle_opacity: 0.0`, i.e. is
//! fully transparent when the pointer is elsewhere. The area scrolls
//! correctly and a screenshot of it is indistinguishable from content
//! clipped at the container edge.
//!
//! `ScrollStyle::solid()` is not enough on its own: it sets
//! `foreground_color: false`, which draws the handle from
//! `visuals.widgets.inactive.bg_fill` — a near-white on a near-white panel
//! under a light preset. Measured in the old shell: the bar was present,
//! opaque, correctly sized, reserving its 10 pt of layout, and invisible in
//! a capture.
//!
//! [`scroll_style`] sets all three, and every panel calls it. See
//! `D:\dev\rag\egui\scrollstyle_solid_draws_the_handle_in_bg_fill_which_is_invisible_on_a_light_panel.md`.
//!
//! ### 2. A fixed-size child inside a scroll area needs the container's
//! width stated
//!
//! `Ui::allocate_ui*` and `add_sized` CLAMP their requested size to the
//! space left in the parent region, so a row wider than the viewport is
//! silently squeezed, the area measures content == viewport, and no bar
//! appears anywhere — the overflow is clipped by the outer container with
//! nothing to say so.
//!
//! [`content_width`] is the fix, and it is a pure function precisely so it
//! can be tested: the container's width is `max(widest row, viewport)`,
//! never a measurement of the laid-out row. See
//! `D:\dev\rag\egui\allocate_ui_clamps_to_remaining_space_so_a_horizontal_scrollarea_squeezes_a_column_instead_of_scrolling.md`.

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::shell::menus::MenuHost;
use egui_shell::HandlerToken;

pub mod attachments;
pub mod bookmarks;
pub mod comments;
pub mod dimension_groups;
/// ★★★ **The document's own properties**, a panel since 2026-09-05 — the
/// operator: *"it needs to get out of there and be in its own document
/// properties tab."* Was `properties::info`; its header carries the move.
pub mod docprops;
pub mod fonts;
pub mod forms;
pub mod layers;
pub mod objects;
pub mod pages;
pub mod properties;
pub mod redact;
pub mod signatures;

/// One dockable panel.
///
/// An enum rather than a trait object, for one reason that matters and one
/// that follows from it. The reason that matters: [`Panel::ALL`] makes the
/// set **enumerable**, which is what lets a test sweep every panel and
/// assert something about each one — the reachability check below is exactly
/// that, and it is the check three panels shipped without. A registry of
/// boxed closures would be extensible and unsweepable.
///
/// The reason that follows: a dock hosting these needs to persist which
/// panels are open, and a `Copy`, `Eq`, `Debug` enum serialises to a token
/// that survives a restart. A closure does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Panel {
    /// The document's outline, as navigation.
    Bookmarks,
    /// The document's optional-content groups.
    Layers,
    /// What each digital signature covers.
    Signatures,
    /// What fonts the document declares, and what they cost.
    Fonts,
    /// Everything drawn on the current page.
    Objects,
    /// The read-only facts about one object — **and about nothing else**.
    ///
    /// ★★ Since 2026-09-05 that clause is the variant's whole scope, where
    /// before it was half of it. `file.properties`' tooltip commissioned two
    /// subjects in one sentence — *"The document's own title, author, subject
    /// and keywords, and the properties of whatever is selected on the page"* —
    /// and the panel drew both, the second permanently. The operator ruled
    /// otherwise; the document half is [`Self::DocumentProperties`].
    Properties,
    /// **The document's own title, author, subject and keywords**, and the
    /// facts pdfcer read about the file — `file.document_properties`.
    ///
    /// ★★★ **The operator, 2026-09-05:** *"the document properties are still
    /// always visible in the properties tab. it needs to get out of there and
    /// be in its own document properties tab."*
    ///
    /// It is the seventh panel whose command is not on View ▸ Panels, and the
    /// placement needed no argument of its own: `RIBBON_IA.md` §5.1's **File ▸
    /// Document** band is *"inspection of what is inside the file"* and already
    /// holds Properties and Fonts. A document's title is inside the file.
    ///
    /// ★★ **A new id rather than a second meaning for `file.properties`.**
    /// [`Self::command_id`] is the single binding between a command and a
    /// panel, and `crate::app::dispatch` resolves toggles through
    /// [`Self::from_command_id`] — so one id cannot open two panels, and a
    /// second spelling of an existing id would have been a second thing to keep
    /// in step. `crate::app::modes::defaults`' own `comments()` and `pages()`
    /// record what that costs when it is got wrong: an id no code has ever
    /// resolved is a guess, and that one was wrong for weeks.
    ///
    /// ★ **A toggle, unlike [`Self::Properties`].** It falls through
    /// `dispatch`'s guard arm to `toggle_panel` because its control asks *"is
    /// this panel open?"*, which is the question `file.fonts` and the whole
    /// `view.panel_*` family ask. `file.properties` is show-only for a reason
    /// that does not apply here: it is offered by the **Objects row context
    /// menu** to describe the row just clicked, and a second invocation that
    /// closed the description would be hostile. Nothing offers this command to
    /// describe anything.
    ///
    /// **Mounted by all three modes.** Reading a document's title is reading,
    /// and Read is shown the `file` tab — so unlike [`Self::Redact`] and
    /// [`Self::Attachments`], a mode that mounts this panel can always reopen
    /// it after closing it, which is the trap [`Self::Forms`] had to move off
    /// the Edit tab to escape.
    DocumentProperties,
    /// The document's form fields, for **filling** — not for authoring.
    ///
    /// The distinction is the panel's whole scope and is worth stating at
    /// the variant rather than only in its module: creating, deleting,
    /// renaming and grouping fields are `Edit ▸ Forms` authoring work
    /// behind a different certification gate, and are deliberately absent.
    Forms,
    /// The document's pages, as pictures — navigate, pick, and act on
    /// sheets.
    ///
    /// The only panel offered by **all three** modes, and the only one whose
    /// body renders anything. Both facts are argued in [`pages`]' own header:
    /// the first from `README.md`'s ruling that page operations do not alter
    /// content, so a reviewer may rotate and extract without leaving the
    /// stance Review takes; the second from `BENCHMARK.md`'s measurement that
    /// a two-pixel render of a dense drawing costs 691 ms — which is why this
    /// panel has a rendering *policy* rather than a loop.
    Pages,
    /// Every annotation on the document — the comment list a reviewer works
    /// through.
    ///
    /// **The only panel whose command is not a `view.panel_*` or a `file.*`
    /// id**, and the reason is worth stating at the variant rather than only
    /// in its module: `RIBBON_IA.md` names Comments in two places, and §7's
    /// migration map — the more specific of the two — sends it to Markup ▸
    /// Comments. See [`Self::command_id`].
    ///
    /// It is a **report with one verb**, like [`Self::Bookmarks`]: it raises
    /// [`Action::GoToPage`] and nothing else. The old shell's panel could also
    /// delete an annotation; that half is deliberately absent here because no
    /// [`Action`] variant can carry the intent, and a control with nothing
    /// behind it is the defect this module's header is about.
    Comments,
    /// Marking content for permanent removal, and reviewing what is marked.
    ///
    /// ★ **The only panel whose command reads as an authoring verb rather than
    /// as a panel name**, and the reason is worth stating at the variant.
    /// `edit.redact`'s shipped tooltip describes an *action* — *"Mark what is
    /// to be permanently removed"* — because marking is what the surface is
    /// for; what it opens is nonetheless somewhere an operator dips in and out
    /// of while working, which is [`crate::dialogs`]' own test for a panel
    /// rather than a dialog.
    ///
    /// It is therefore a **toggle**, like the `view.panel_*` family and unlike
    /// [`Self::Properties`]: pressing Redact with the Redact panel open closes
    /// it, which is what `crate::app::panels` settled for every control whose
    /// question is *"is this panel open?"*. Nothing about that is special-cased
    /// — it falls out of [`Self::from_command_id`] answering for this id, which
    /// is the guard arm the toggle family already goes through.
    ///
    /// The **irreversible** half deliberately does not live here.
    /// `edit.redact_apply` opens [`crate::dialogs::redact`], because applying
    /// is a single transaction with a start and an end, and because a control
    /// that commits an irreversible operation must not sit two rows below one
    /// that merely marks. See [`redact`]'s header for the whole argument,
    /// including why canvas drag-to-mark is not in this landing.
    Redact,
    /// Where ce-dimension groups are made, chosen and configured.
    ///
    /// ★ **The only panel that was built as a window first and moved**, and
    /// the move is the operator's, not a refactor: a window whose content is
    /// taller than the screen can push its own title bar — and its only ✕ —
    /// off the desktop, and he could not close it. See
    /// [`dimension_groups`]'s header for the three findings packed into that
    /// one report and for why a dock column removes the condition rather than
    /// tuning it.
    ///
    /// Its command is `measure.manage_groups`, which is **not** a
    /// `view.panel_*` id, and that is [`Self::Redact`]'s precedent applied
    /// deliberately rather than an omission. A second id for one surface would
    /// put this panel on a tab a mode without measure authoring is shown, and
    /// the point of leaving it on Measure ▸ Scale is that the mode taxonomy
    /// then does the gating with no capability flag of its own: `read` is not
    /// shown the `measure` tab, so `read` cannot reach the panel, and Review
    /// and Edit both can. The same argument, in the same words, is why there
    /// is no `view.panel_redact`.
    DimensionGroups,
    /// The whole files this document carries inside itself (§7.11.4.1).
    ///
    /// ★★ **The sixth panel whose command is not on View ▸ Panels**, and the
    /// only one `RIBBON_IA.md` names nowhere at all — it lists no Attachments
    /// control on any tab, in any group. So the placement is argued rather than
    /// read off, and the argument is [`Self::Redact`]'s, applied to the same
    /// question:
    ///
    /// Read is shown `file` and `view` alone. A `view.panel_attachments` — or a
    /// `file.attachments` beside Fonts and Properties, which is where the
    /// *reading* half of this panel would otherwise belong — would put a
    /// surface that **embeds and removes whole files** in front of a reading
    /// stance. `edit.attachments` on the Edit tab makes the mode taxonomy do
    /// that work with no capability flag and no gate of its own, which is the
    /// property that decided Redact and is the closest defensible precedent
    /// this IA has.
    ///
    /// ★ The cost is stated rather than hidden: Acrobat *Reader* lists
    /// attachments and saves them out, and this build's Read mode cannot. The
    /// day that matters, the fix is a second panel — a listing with no verbs —
    /// and not a second id for this one, because P1 gives a command one tab and
    /// a panel with authoring controls does not belong on a reading stance's
    /// ribbon.
    Attachments,
}

impl Panel {
    /// Every panel.
    ///
    /// Hand-written, because Rust cannot enumerate an enum. That makes it
    /// the classic array that silently stops being exhaustive when a variant
    /// is added — so [`tests::the_panel_catalog_is_complete`] pins its
    /// length against a match that the compiler *does* check, which is the
    /// only way to make a hand-written catalog self-defending.
    pub const ALL: [Self; 13] = [
        Self::Attachments,
        Self::Bookmarks,
        Self::Layers,
        Self::Signatures,
        Self::Fonts,
        Self::Objects,
        Self::Properties,
        Self::DocumentProperties,
        Self::Forms,
        Self::Pages,
        Self::Comments,
        Self::Redact,
        Self::DimensionGroups,
    ];

    /// The ribbon command that shows this panel.
    ///
    /// **This is the reachability contract**, and it is the answer to the
    /// defect in this module's header. Every panel names a command; the test
    /// below asserts every one of those commands is both registered in
    /// `crate::shell::commands` and referenced by
    /// `crate::shell::manifest::built_in`. A panel with no route from the
    /// ribbon cannot get past that.
    ///
    /// Three of the nine are **not** on View ▸ Panels, and every placement is
    /// `RIBBON_IA.md`'s:
    ///
    /// - **Fonts is `file.fonts`.** §7's migration map moves it from View ▸
    ///   Panels to File ▸ Document, because the Fonts panel answers "what is
    ///   inside this file", not "what is on my screen".
    /// - **Properties is `file.properties`.** ★★ Its tooltip used to commission
    ///   both halves of one panel — *"The document's own title, author, subject
    ///   and keywords, and the properties of whatever is selected on the
    ///   page."* Since 2026-09-05 those are two panels and two commands, on the
    ///   operator's instruction, and the tooltip says only what its own panel
    ///   does. See [`Self::DocumentProperties`].
    /// - **Document properties is `file.document_properties`**, beside it in
    ///   File ▸ Document for the reason Fonts is there: it answers *"what is
    ///   inside this file"*.
    #[must_use]
    pub fn command_id(self) -> &'static str {
        match self {
            // ★ **The sixth panel whose command is not on View ▸ Panels**, and
            // the only one `RIBBON_IA.md` places nowhere: §5.2's Panels row
            // names Pages, Objects, Bookmarks, Layers, Signatures, Comments and
            // Forms, and no section of that document mentions attachments at
            // all. The variant's own doc carries the argument for Edit, which
            // is `Self::Redact`'s applied to the same question — a surface that
            // embeds and removes whole files must not be reachable from a
            // reading stance, and Read is shown `file` and `view` alone.
            Self::Attachments => "edit.attachments",
            Self::Bookmarks => "view.panel_bookmarks",
            Self::Layers => "view.panel_layers",
            Self::Signatures => "view.panel_signatures",
            Self::Fonts => "file.fonts",
            Self::Objects => "view.panel_objects",
            Self::Properties => "file.properties",
            // ★ The seventh id that is not a `view.panel_*`, and the one that
            // needed no argument: File ▸ Document is the band for *"what is
            // inside this file"*, and it already holds Properties and Fonts.
            // See the variant for why it is a NEW id rather than a second
            // meaning for the one above it.
            Self::DocumentProperties => "file.document_properties",
            // ★ **On View ▸ Panels since 2026-08-14**, and it was on Edit
            // before that. `RIBBON_IA.md`'s placement was the Edit tab, on
            // the argument that a form panel answers "what can I fill in
            // this file", which is an edit of the document rather than of
            // the view. That argument survives — filling still writes — but
            // it was answering the wrong question. The operator's question
            // was *which modes may fill*, and the answer is all three,
            // because Acrobat Reader fills forms in its default view.
            //
            // Read is shown `file` and `view` alone, so the tab followed
            // the mode. `crate::app::modes` carries the amended taxonomy;
            // `crate::shell::manifest::edit` records what stayed behind.
            //
            // The command was registered and reachable from the ribbon well
            // before this panel existed — it had no dispatch arm, which is
            // precisely the class of half-built surface this module's
            // header is about.
            Self::Forms => "view.panel_forms",
            // ★ **This command is not registered in this build**, and the
            // panel is therefore filtered out of every arrangement by
            // `SHELL_FRAMEWORK.md` §5b — see `pages`' own header, which
            // carries the account and the exact lines needed.
            //
            // The id is `RIBBON_IA.md`'s and `crate::app::modes`' both:
            // `modes::defaults::spec` has named it in all three default arrangements
            // since before this panel existed, and `modes::ABSENT_PANELS`
            // carried the matching "not built yet" entry, which this panel's
            // arrival removes. It is on View ▸ Panels rather than anywhere
            // else because a thumbnail grid answers *"what is on my
            // screen"* — it is a navigator, and navigators live in View.
            Self::Pages => "view.panel_pages",
            // ★ **The third panel whose command is not on View ▸ Panels**, and
            // the only one whose placement had to be *chosen* between two
            // sentences of `RIBBON_IA.md` rather than read off one.
            //
            // §5.2 lists `Comments` among View ▸ Panels. §5.5 gives the Markup
            // tab a `Comments` group containing `Comments panel`. P1 gives a
            // command one tab, so both cannot be honoured — and §7's migration
            // map settles it by naming the control: `Review ▸ Comments ▸
            // Comments` → `Markup ▸ Comments`. A per-control ruling is more
            // specific than a list of panel names, so Markup wins.
            //
            // `crate::shell::manifest::markup` reached the same conclusion in
            // the same words when that tab was built, which is why this
            // command was **already registered and already on the ribbon**
            // before this panel existed — a control with no body, the mirror
            // image of the defect in this module's header, and the reason
            // `crate::panels::comments` is the last panel the taxonomy names
            // to acquire one.
            //
            // The mode taxonomy agrees, which is what makes it safe: Comments
            // is mounted by Review and Edit alone, and both are shown the
            // `markup` tab. Contrast [`Self::Forms`], which had to move off
            // Edit precisely because Read mounts it and Read is shown `file`
            // and `view` only.
            //
            // ★ `crate::app::modes::defaults` still names the panel by the
            // OTHER id — `view.panel_comments`, with a matching
            // `ABSENT_PANELS` entry — so the default arrangements will not
            // mount this panel until both are changed. That file is not this
            // panel's to edit; the two lines it needs are in this work's
            // report to the shell owner.
            Self::Comments => "markup.comments",
            // ★ **The fourth panel whose command is not on View ▸ Panels**, and
            // the only one whose command was written as a *verb*.
            //
            // `RIBBON_IA.md` §5.4 puts Redact on **Edit ▸ Protect**, and
            // `crate::shell::manifest::edit`'s header carries the placement
            // argument: *"a user editing a document looks under Edit for the
            // command that removes content from it. Tools is for jobs that run
            // across other files."* It moved there from Tools ▸ Protect when
            // this shell's ribbon was built, long before either half of the
            // feature existed.
            //
            // There is deliberately no `view.panel_redact`. A second id for the
            // same surface would put the panel on a tab Read is shown, and Read
            // must not be able to reach a marking surface at all: `edit.redact`
            // sitting on the Edit tab is what makes the mode taxonomy do that
            // work, with no capability flag and no gate of its own. Contrast
            // [`Self::Forms`], which had to acquire a `view.` id precisely
            // because Read *should* reach it.
            Self::Redact => "edit.redact",
            // ★ **The fifth panel whose command is not on View ▸ Panels.** It
            // stays where `RIBBON_IA.md` put the control — Measure ▸ Scale —
            // and the variant's own doc carries the argument. The command has
            // been registered and drawn since the Measure tab was built; what
            // changed on 2026-08-19 is that pressing it toggles a panel rather
            // than opening a window.
            Self::DimensionGroups => "measure.manage_groups",
        }
    }

    /// The panel whose [`Self::command_id`] is `id`, if any.
    ///
    /// The dock stores opaque ids, so something has to turn one back into a
    /// panel, and this is deliberately the only thing that does. Written as
    /// a search over [`Self::ALL`] rather than a second `match`: a second
    /// `match` is a second list to keep in step, and the failure when it
    /// drifts is a panel that opens from the ribbon and draws nothing in
    /// the dock — which looks like a rendering bug and is not.
    ///
    /// Returns `None` for an id this build does not have, which is a
    /// reachable state: a saved layout can name a panel whose capability
    /// was compiled out.
    #[must_use]
    pub fn from_command_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.command_id() == id)
    }

    /// Draw this panel.
    ///
    /// The one entry point a dock calls. `doc` is `None` when nothing is
    /// open, and that case is handled **here** rather than nine times: the
    /// answer does not vary by panel, and nine bespoke "open a document to…"
    /// sentences would be nine chances for one of them to drift.
    ///
    /// The bodies below therefore all have the shape
    /// `fn body(ui, doc: &OpenDoc, state: &mut PanelsState, actions: &mut Vec<Action>)`
    /// and never see the empty case.
    ///
    /// # ★ Two routes out, and why context-menu commands take the second
    ///
    /// `actions` carries what a panel decides for itself — the Bookmarks
    /// panel's `GoToPage`, the Layers panel's `SetLayerVisible`. The
    /// **return value** carries `egui_shell::HandlerToken`s: the commands an
    /// operator chose from a panel's context menu.
    ///
    /// A panel must not translate those into `Action`s, for the same reason
    /// the canvas must not. A token is resolved to an id and dispatched by
    /// `PdfcerApp::dispatch_token`, which is the single choke point where a
    /// confirmation gate, an undo entry or a refusal lives; a panel that
    /// translated `file.properties` for itself would be a second
    /// implementation of a command that already has one, and the two would
    /// drift the first time the command grew a precondition.
    ///
    /// `host` is `None` when the application has no validated shell (see
    /// [`MenuHost`]), in which case no panel attaches a menu and a
    /// right-click does nothing.
    ///
    /// **Two panels attach a menu today**, and only one of them can open
    /// one. Objects attaches `objects.row`, which `crate::shell::menus`
    /// defines. [`pages`] attaches `pages.row`, which it does **not** — so
    /// that right-click opens nothing at all, which is the correct behaviour
    /// for a surface with nothing to offer and becomes the intended menu the
    /// day the context is defined, with no edit in the panel. The other five
    /// return an empty `Vec` because no context is defined for them either.
    #[must_use]
    pub fn show(
        self,
        ui: &mut egui::Ui,
        doc: Option<&OpenDoc>,
        state: &mut PanelsState,
        host: Option<&MenuHost<'_>>,
        actions: &mut Vec<Action>,
    ) -> Vec<HandlerToken> {
        scroll_style(ui);
        let Some(doc) = doc else {
            // Nothing open: forget the operator's tree state. Doing this
            // here rather than in each body is what makes it unforgettable —
            // a panel that never draws while the shell is empty would never
            // get the chance. (The document's own caches need no equivalent:
            // they live on `OpenDoc` and were dropped with it.)
            state.forget_document();
            ui.label(crate::text::panels::panel_no_document());
            return Vec::new();
        };
        state.sync(doc);
        match self {
            Self::Attachments => attachments::body(ui, doc, state, actions),
            Self::Bookmarks => bookmarks::body(ui, doc, state, actions),
            Self::Layers => layers::body(ui, doc, state, actions),
            Self::Signatures => signatures::body(ui, doc, state, actions),
            Self::Fonts => fonts::body(ui, doc, state, actions),
            Self::Objects => return objects::body(ui, doc, state, host, actions),
            Self::Properties => properties::body(ui, doc, state, actions),
            Self::DocumentProperties => docprops::body(ui, doc, state.docprops_mut(), actions),
            Self::Forms => forms::body(ui, doc, state, actions),
            // The second panel with a menu, and the second `return` for the
            // same reason: it has tokens to hand back.
            Self::Pages => return pages::body(ui, doc, state, host, actions),
            Self::Comments => comments::body(ui, doc, state, actions),
            Self::Redact => redact::body(ui, doc, state, actions),
            Self::DimensionGroups => dimension_groups::body(ui, doc, state, actions),
        }
        Vec::new()
    }
}

/// The little state the panel bodies own between frames.
///
/// # Why this exists at all, and why it is not on `PdfcerApp`
///
/// Two of the nine panels are not pure functions of the document: the Objects
/// panel remembers which rows are expanded and which row was last picked, and
/// the Properties panel reads that pick. None of it is document state, and
/// none of it is derivable from anything — but all of it has to outlive a
/// frame.
///
/// It lives here rather than as fields on `crate::app::PdfcerApp` because
/// *this* is the module that owns the concepts. A dock hands one `&mut
/// PanelsState` to whichever panel it is drawing, and the app holds it the
/// way it holds any other subsystem's state. Spreading these fields across
/// `PdfcerApp` would put the Objects panel's expansion set next to the render
/// worker.
///
/// # ★ What is NO LONGER here: the two caches, and their identity key
///
/// Until S4 this struct also held the page decomposition and the font
/// inventory, guarded by a `DocKey` assembled from the `Arc<EditSession>`'s
/// **address** plus the path, page count and edit epoch. The header of that
/// type documented its own residual hazard: an address is not an identity,
/// because a dropped `Arc`'s allocation can be reused, so a reopened document
/// could in principle have been served the previous one's decomposition. It
/// also documented why the obvious fix was worse — holding an `Arc` or a
/// `Weak` clone would make it a real identity and would break
/// `crate::app::state::OpenDoc::session`'s `Arc::get_mut` mutation path,
/// disabling document editing to fix a cache.
///
/// Both caches now live on `crate::app::state::OpenDoc`, where the document's
/// own lifetime bounds them and **no identity key is needed at all**:
/// `OpenDoc::new` constructs a whole new document state per open, so a cache
/// inside it can never describe a previous file. `DocKey` was deleted rather
/// than repaired, because an identity key existed only to compensate for a
/// cache outliving the thing it described.
///
/// What is left here genuinely does outlive a document — it hangs off the
/// application — and it is handled by *forgetting* rather than by keying:
/// [`Self::forget_document`] is called from `PdfcerApp::open_path`, the one
/// place a document is ever opened, and from [`Panel::show`] when nothing is
/// open. A single statement at the one moment it is true beats a comparison
/// made sixty times a second.
///
/// Within one document, [`Self::sync`] still drops this state on a page or
/// revision change, keyed on `(page index, edit epoch)` — two plain values,
/// no address, no ABA hazard.
#[derive(Default)]
pub struct PanelsState {
    /// The `(page index, edit epoch)` [`Self::tree`] describes, or `None`
    /// before anything has been drawn.
    ///
    /// The page is in the key because a paint-order index is a position on
    /// **one page**: keeping a focus or an expansion set across a page step
    /// would silently point them at a different object with the same number.
    /// The epoch is in it because an edit renumbers everything after the
    /// object it touched, which does the same thing without moving page.
    tree_key: Option<(usize, u64)>,
    /// What the operator has opened and picked in the Objects tree.
    ///
    /// A struct rather than three loose fields so the grouping says what it
    /// is: the operator's own state, not a cache of the document. Everything
    /// derived from the document now lives on `OpenDoc`, and this is what was
    /// left when it went.
    tree: ObjectTreeUi,
    /// The Pages panel's picked sheets and its thumbnail cache.
    ///
    /// # ★ Why this cache lives here and not on `OpenDoc`
    ///
    /// Every other derived cache moved to `crate::app::state::OpenDoc` at S4,
    /// and the argument for that move — *"the document's own lifetime bounds
    /// them and no identity key is needed at all"* — applies to thumbnails
    /// word for word. It is still right for this one to be here, for a reason
    /// that is about **borrowing** rather than about lifetime.
    ///
    /// A panel body is handed `&OpenDoc`, shared, which is the compile-time
    /// half of actions-not-mutations. Filling a cache on `OpenDoc` would
    /// therefore need interior mutability — which the two existing caches
    /// have (`RefCell`) and which this module's own header permits for
    /// *derived* data. But a thumbnail cache is not only filled: it is
    /// **evicted from, and stopped**, by a control the operator clicks, and
    /// `ThumbnailCache::force_on` is state that decides whether work happens
    /// at all. Putting a `RefCell` around that would put an operator
    /// instruction behind interior mutability, which is precisely the line
    /// this module's header draws for the Layers panel.
    ///
    /// `&mut PanelsState` is already threaded to every body, so the panel's
    /// own state needs no such device — and the forgetting is free, because
    /// [`Self::forget_document`] resets this struct whole.
    pages: pages::PagesUi,
    /// The Redact panel's half-typed search and its match mode.
    ///
    /// Here rather than on `OpenDoc` for the same reason [`Self::pages`] is:
    /// a panel body is handed `&OpenDoc`, shared, and a text field the operator
    /// types into is **their** state rather than a derived cache of the
    /// document's. It is also state that arms a verb — a query plus a mode
    /// decides what a marking click will cover — and this module's header draws
    /// exactly that line for the Layers checkbox: interior mutability is for
    /// derived data whose filling nothing can observe, never for an operator
    /// instruction.
    ///
    /// Reset with the document by [`Self::forget_document`], which resets this
    /// struct whole. That is not tidiness: a search term left over from a
    /// previous file is one an operator could run against a document it was
    /// never meant for, and on this feature a search authors marks over whatever
    /// it hits.
    redact: redact::RedactUi,
    /// **What the operator has typed into the Layers panel's search box.**
    ///
    /// Here rather than on the panel for [`Self::redact`]'s first reason: a
    /// `TextEdit` needs a `&mut String` that survives the frame, and a panel
    /// body is handed `&OpenDoc` — shared, deliberately, so that it cannot
    /// mutate. It is the operator's own typing rather than a derived cache,
    /// which is the line this module's header draws.
    ///
    /// ★ Reset with the document by [`Self::forget_document`], and here that
    /// is a straightforward good rather than a safety property: a query left
    /// over from a previous file would open the next one showing a filtered
    /// layer list with no obvious cause. Unlike `redact`'s, this search
    /// authors nothing — the worst it can do is hide rows — so the reset is
    /// about not confusing the operator rather than about not marking the
    /// wrong document.
    layers_search: String,
    /// The **Document properties** panel's half-typed metadata.
    ///
    /// Here for [`Self::pages`]' reason and one of its own: a `TextEdit` needs
    /// a `&mut String` that survives the frame, and a panel body is handed
    /// `&OpenDoc`, shared. It is also the operator's own typing rather than a
    /// derived cache — this module's header draws exactly that line.
    ///
    /// ★ Reset with the document by [`Self::forget_document`], and that matters
    /// more here than for a search term: a half-typed `/Author` carried into a
    /// second file would be written into **that** file's metadata by the next
    /// focus change, silently, in a field nobody looks at twice.
    ///
    /// ★★ Named `properties` until 2026-09-05, when the section became
    /// [`Panel::DocumentProperties`]. Renamed with it rather than left: a field
    /// named after the panel that no longer draws it is how the next reader
    /// looks in the wrong place, and this struct already holds a `geometry`, a
    /// `text_style` and a `text_object` that ARE the Properties panel's.
    docprops: docprops::InfoDrafts,
    /// The form-field rename draft, and which field it is for. See
    /// [`Self::field_rename_mut`] for why the key is stored beside it.
    field_rename: String,
    /// The fully-qualified name [`Self::field_rename`] was seeded from.
    field_rename_key: Option<String>,
    /// ★ The two TYPED properties of the selected form field — its tooltip and
    /// its maximum length — and the `(name, epoch)` they were read at.
    ///
    /// Only two, and the omission is the design: every other property in
    /// `properties::fieldedit` is a checkbox that reads `field.flags` straight
    /// from the session each frame, so a press the engine refuses leaves the
    /// box where it was. A draft-backed boolean would show the operator's
    /// intent while the document silently disagreed with it.
    ///
    /// Here rather than on `OpenDoc` by this struct's own rule: a half-typed
    /// tooltip is the operator's state, not the document's.
    field_props: properties::fieldedit::FieldPropsDraft,
    /// ★ The selected WIDGET's four typed numbers and its caption, and the
    /// `(name, widget index, epoch)` they were read at.
    ///
    /// Separate from [`Self::field_props`] rather than a field inside it,
    /// because the two have different stamps: a field draft is keyed on the
    /// name and this one has to be keyed on the name **and the placement**. One
    /// field can be drawn in three places with three different boxes, and a
    /// merged struct would need both keys and one reset rule for two lifetimes.
    widget_props: properties::widgetedit::WidgetPropsDraft,
    /// The Properties panel's **geometry** draft — the four typed numbers, and
    /// the `(page, object, epoch)` they were seeded from.
    ///
    /// Separate from `properties` above rather than a field inside it, because
    /// the two have different lifetimes and different reset conditions: the
    /// metadata drafts survive a selection change (they describe the document),
    /// and this one must not (it describes one object). Merging them would make
    /// one struct with two reset rules.
    geometry: properties::geometry::GeometryDraft,
    /// ★ The selected text's face, size and colour, and the size being typed.
    ///
    /// Held here for a stronger reason than its neighbours: the read-back needs
    /// an extraction with provenance on — 392 ms on the operator's benchmark
    /// sheet — so a section that re-read it every frame would take the whole
    /// application to under three frames a second on exactly the drawings this
    /// program is for. The struct carries a `(page, run, epoch)` stamp and
    /// re-reads only when it moves.
    text_style: properties::text::TextStyleDraft,
    /// ★★★ The **clicked text object's** run range and colour — O89's object
    /// route.
    ///
    /// Held here for exactly [`Self::text_style`]'s reason and at exactly its
    /// cost: `properties::textobject` reads the object's runs and their fills
    /// out of one extraction with provenance capture on, which is 392 ms on the
    /// operator's benchmark sheet, and a section that re-read it every frame
    /// would take the application to under three frames a second on the
    /// drawings this program is for. The struct carries a
    /// `(page, object, epoch)` stamp and re-reads only when it moves.
    ///
    /// ★ Separate from [`Self::text_style`] rather than a second case inside
    /// it, and the reason is the stamp: that one is keyed on a **run** and this
    /// on an **object**, and the two selections are different index spaces that
    /// can both be absent, either be present, and — since the Text tool can be
    /// armed in Edit — both be present at once. One struct with two stamps is
    /// one struct with two reset rules.
    text_object: properties::textobject::TextObjectDraft,
    /// ★ The memoised answer to *what would go with deleting the selected
    /// annotation?* — `EditSession::annotation_deletion_preview`.
    ///
    /// Held here for [`Self::text_style`]'s reason at a smaller magnitude, and
    /// the shape of the argument is what matters rather than the milliseconds.
    /// The query is `&self` and side-effect-free, but it walks the page's whole
    /// `/Annots` array looking for `/IRT` referrers — O(annotations) per call —
    /// and the old shell paid that per *row* and gated it on hover for exactly
    /// that reason. This section has one subject rather than a list, so the
    /// worst case is one call per frame; the `(annotation id, edit epoch)` stamp
    /// takes it down to none. See `properties::annotdelete`'s header.
    ///
    /// ★★ Not a document cache smuggled into the operator's own state. What is
    /// stored is the finished **sentence**, which is drawing state, and it is
    /// reset with the document by [`Self::forget_document`] like everything else
    /// here — an object id carried into a second file names a different object
    /// there, which for a cached collateral warning would mean describing one
    /// document's reply thread while the operator looks at another's.
    annot_delete: properties::annotdelete::DeletionPreview,
    /// The Bookmarks panel's half-typed title and its chosen parent.
    ///
    /// Here for [`Self::pages`]' reason: a panel body is handed `&OpenDoc`,
    /// shared, and a text field the operator types into is **their** state.
    ///
    /// ★ Reset with the document by [`Self::forget_document`], and the parent
    /// is why that matters more than for a search term: an `ObjId` carried into
    /// a second file names a different object there, and a bookmark would be
    /// filed under whatever happens to hold that number.
    bookmarks: bookmarks::BookmarksUi,
    /// ★ The Attachments panel's half-typed description.
    ///
    /// Here for [`Self::properties`]' reason, and the hazard is the same one
    /// stated more sharply: `attach_file` takes a description **at attach
    /// time** and no verb edits one afterwards, so a draft carried into a
    /// second document would be written permanently into that file's `/Desc`
    /// by the next attach, describing one operator's spreadsheet with another
    /// document's note. [`Self::forget_document`] resets this struct whole,
    /// which is what makes that unrepresentable rather than merely avoided.
    attachments: attachments::AttachmentsUi,
    /// The dimension-groups panel's selected row, its half-typed names and its
    /// pending *Set scale…* request.
    ///
    /// `pub(crate)` rather than private, and it is the only field here that is:
    /// `crate::app::PdfcerApp::docks` drains
    /// [`dimension_groups::DimensionGroupsUi::take_scale_request`] after the
    /// dock body closes. A panel cannot open a window from inside its own body
    /// — it is handed `&OpenDoc` and `&mut PanelsState` and nothing else — so
    /// the request has to leave through the state it is allowed to touch.
    ///
    /// ★ Reset with the document by [`Self::forget_document`], for the reason
    /// [`Self::bookmarks`] gives and one of its own: a `GroupId` names a
    /// different group in a different file, so a selection carried across would
    /// point the appearance controls at somebody else's group.
    pub(crate) dimension_groups: dimension_groups::DimensionGroupsUi,
    /// **The comment note being typed, and the annotation it belongs to.**
    ///
    /// Here for [`Self::pages`]' reason — a `TextEdit` needs a `&mut String`
    /// that outlives the frame and a panel body is handed `&OpenDoc`, shared —
    /// and the Comments panel is the surface that most recently claimed to have
    /// no such state at all. It has one now, and
    /// [`comments::note::NoteDraft`]'s header carries the argument for why it
    /// is a draft rather than a live binding.
    ///
    /// ★ Reset with the document by [`Self::forget_document`], and here that
    /// matters as much as it does for a half-typed `/Author`: an `ObjId` names
    /// a different annotation in a different file, so a draft carried across
    /// would offer to write one document's comment onto another document's
    /// shape.
    comments: comments::note::CommentsUi,
}

/// What the operator has opened and picked in the Objects tree.
///
/// Every field is cleared when the page or the document revision changes
/// (see [`PanelsState::sync`]), because a paint-order index is a **position
/// on one page of one revision**, not an identity.
#[derive(Default)]
pub struct ObjectTreeUi {
    /// Which object rows are expanded, by paint-order index.
    pub(crate) objects_expanded: std::collections::BTreeSet<usize>,
    /// Which part rows are expanded, by `(object, part)`.
    pub(crate) parts_expanded: std::collections::BTreeSet<(usize, usize)>,
    /// The object the Properties panel describes, by paint-order index.
    ///
    /// **Not a selection**, and the distinction is load-bearing enough to
    /// have its own name. A selection is document-scoped, multi-valued,
    /// survives a page change, drives the contextual Format tab, and is what
    /// an edit acts on. This is one `usize` that says which Objects row the
    /// operator last clicked, so a second panel can describe it.
    ///
    /// # ★ This field is DELETED when the selection model lands, not grown
    ///
    /// The distinction is what stops a shell acquiring two selections. When
    /// the real one exists, [`properties`] reads *it*, the Objects row click
    /// becomes a selection gesture, and this field goes. Growing it instead —
    /// making it a `Vec`, letting it survive a page change, letting an edit
    /// act on it — would produce a second, weaker selection that the canvas
    /// and the panel would have to keep in step, and the drift between them
    /// would be invisible until an edit acted on the wrong object.
    ///
    /// **The S4 status, stated rather than assumed.** The canvas's selection
    /// model is being built in this stage, and this field has deliberately
    /// **not** been extended to meet it half way: it is still one `usize`,
    /// still page-scoped, still cleared by [`PanelsState::sync`] on any page
    /// or revision change, and still read by exactly one panel.
    /// ★★★ **RETIRED 2026-08-26. Nothing in production writes or reads it.**
    ///
    /// This field's own docs used to end: *"It is deleted in the commit that
    /// makes [`properties`] read the canvas's selection, and not before —
    /// deleting it earlier would leave the Properties panel with nothing to
    /// describe."* That commit has happened. The Properties panel reads
    /// `OpenDoc::selection`, the Objects panel's row highlight reads the same,
    /// and a row click raises `Action::SelectObject` rather than writing here.
    ///
    /// # Why the field is still declared
    ///
    /// Because three tests in `app::files` and `app::lifecycle` use it to pin a
    /// property that is still real and still worth pinning: **closing or
    /// re-opening a document must forget the paint-order indices the panels
    /// hold**, because an index names a position in a document that is no
    /// longer open. Those tests reach for the one index-bearing field they can
    /// set from outside, and deleting it would delete the assertion with it.
    ///
    /// ★ That is a poor reason to keep a field and it is stated as one. The
    /// right end is for those tests to assert against the tree's *expansion*
    /// state, which is index-bearing, production-live and cleared by the same
    /// `forget_document`. Until they do, this field is a test fixture wearing a
    /// production field's clothes, and this comment is what stops the next
    /// reader mistaking it for the second selection it used to be.
    focus: Option<usize>,
}

impl ObjectTreeUi {
    /// Which object the Properties panel is describing.
    #[must_use]
    pub fn focus(&self) -> Option<usize> {
        self.focus
    }

    /// Point the Properties panel at an object.
    ///
    /// Clicking the already-focused row clears the focus, so a row click is
    /// its own undo. That is a deliberate consequence of there being no
    /// Escape ladder yet: with no selection model there is no other way back
    /// to "nothing focused", and a panel an operator cannot get out of is
    /// worse than one they cannot get into.
    pub fn set_focus(&mut self, index: usize) {
        self.focus = if self.focus == Some(index) {
            None
        } else {
            Some(index)
        };
    }

    /// Toggle an object row's expansion.
    pub(crate) fn toggle_object(&mut self, index: usize) {
        if !self.objects_expanded.remove(&index) {
            self.objects_expanded.insert(index);
        }
    }

    /// Toggle a part row's expansion.
    pub(crate) fn toggle_part(&mut self, object: usize, part: usize) {
        if !self.parts_expanded.remove(&(object, part)) {
            self.parts_expanded.insert((object, part));
        }
    }
}

impl PanelsState {
    /// Drop anything that no longer describes `doc`'s current page.
    ///
    /// Called once per frame, before any panel body runs, so no two panels
    /// can disagree about which revision they are describing — which is the
    /// whole point of doing it here rather than in each body.
    ///
    /// **A page or revision change clears the focus and the expansion sets.**
    /// Paint-order indices are positions, not identities: deleting one object
    /// renumbers every object after it, so a retained focus would silently
    /// describe a *different* object with the same number, and a retained
    /// expansion set would open the wrong rows. Forgetting is the only honest
    /// response, and it is cheap.
    ///
    /// The key is `(page index, edit epoch)` and nothing else. It does not
    /// need to say *which document* — a different document reaches
    /// [`Self::forget_document`] through `PdfcerApp::open_path` before any
    /// panel draws, so there is nothing left to confuse it with. That is what
    /// let the old four-field `DocKey`, with the `Arc` address in it, be
    /// deleted rather than repaired; see this struct's own header.
    fn sync(&mut self, doc: &OpenDoc) {
        let key = (doc.view.page_index, doc.edit_epoch);
        if self.tree_key != Some(key) {
            self.tree_key = Some(key);
            self.tree = ObjectTreeUi::default();
        }
    }

    /// Forget everything about whatever document was open.
    ///
    /// Called from two places, and both are needed: `PdfcerApp::open_path`,
    /// because a new document makes every paint-order index here meaningless,
    /// and [`Panel::show`] when nothing is open, because a panel that never
    /// draws while the shell is empty would never get the chance.
    ///
    /// `*self = Self::default()` rather than clearing fields one at a time,
    /// so a field added later is forgotten by construction. This is the one
    /// operation that must not need updating when the struct grows.
    pub fn forget_document(&mut self) {
        *self = Self::default();
    }

    /// The operator's state in the Objects tree — what is expanded, and what
    /// is focused.
    ///
    /// Handed out whole rather than through a method per field, because the
    /// Objects panel reads the expansion sets while it draws and writes them
    /// after; splitting that across four accessors would gain nothing and
    /// cost the panel the ability to hold one borrow for the frame.
    ///
    /// Note what it is **not** paired with any more. Until S4 this came back
    /// alongside the page decomposition from one method, because the panel
    /// needed `&provider` and `&mut tree` simultaneously and Rust permits
    /// that only as two disjoint borrows of one struct. The provider now
    /// lives on `OpenDoc`, so the two come from different objects entirely
    /// and the pairing has no reason to exist.
    pub fn tree_mut(&mut self) -> &mut ObjectTreeUi {
        &mut self.tree
    }

    /// Which object the Properties panel is describing.
    ///
    /// Delegates to [`ObjectTreeUi`], which is where the field lives. The
    /// forwarder exists so a panel that only needs to *read* the focus — the
    /// Properties panel — does not have to reach through the grouping.
    #[must_use]
    pub fn focus(&self) -> Option<usize> {
        self.tree.focus()
    }

    /// Point the Properties panel at an object. See
    /// [`ObjectTreeUi::set_focus`].
    pub fn set_focus(&mut self, index: usize) {
        self.tree.set_focus(index);
    }

    /// The Pages panel's own state — its picked sheets and its thumbnails.
    ///
    /// Handed out whole for the same reason [`Self::tree_mut`] is: the body
    /// reads the cache while it lays tiles out and writes the selection while
    /// it reads the clicks, and splitting that into accessors per field would
    /// cost it the ability to hold one borrow for the frame.
    pub fn pages_mut(&mut self) -> &mut pages::PagesUi {
        &mut self.pages
    }

    /// The Redact panel's own state — the search query and the match mode.
    ///
    /// Handed out whole for [`Self::pages_mut`]'s reason: the body reads the
    /// query while it draws the field and writes the mode while it reads the
    /// switch, and splitting that into accessors per field would cost it the
    /// ability to hold one borrow for the frame.
    /// The Layers panel's search box, mutably.
    ///
    /// One accessor for one `String`, matching [`Self::redact_mut`]'s shape:
    /// the panel needs the `&mut` to hand to a `TextEdit` and needs to read
    /// the trimmed value back in the same frame.
    pub fn layers_search_mut(&mut self) -> &mut String {
        &mut self.layers_search
    }

    pub fn redact_mut(&mut self) -> &mut redact::RedactUi {
        &mut self.redact
    }

    /// The **Document properties** panel's metadata drafts.
    ///
    /// Same shape as [`Self::pages_mut`] and [`Self::redact_mut`]: the body is
    /// handed `&mut PanelsState` and reaches its own state through an
    /// accessor, so the field stays private and no other panel can write it.
    ///
    /// ★ Was `properties_mut` until 2026-09-05. Renamed with the panel, and the
    /// rename is what makes the compiler point at every caller — there was one.
    pub fn docprops_mut(&mut self) -> &mut docprops::InfoDrafts {
        &mut self.docprops
    }

    /// **The Comments panel's note draft.**
    ///
    /// Same shape as [`Self::pages_mut`], [`Self::redact_mut`] and
    /// [`Self::properties_mut`]: the body is handed `&mut PanelsState` and
    /// reaches its own state through an accessor, so the field stays private
    /// and no other panel can write it.
    pub fn comments_mut(&mut self) -> &mut comments::note::CommentsUi {
        &mut self.comments
    }

    /// **The rename draft for the selected form field**, re-seeded whenever the
    /// selection moves.
    ///
    /// ★★ The re-seeding is the whole reason this is a method rather than a
    /// bare `&mut String`. Without it, clicking field A, typing a new name, then
    /// clicking field B leaves A's half-typed name in the box — aimed at B. The
    /// operator presses Rename and renames the wrong field to a name they chose
    /// for a different one, and nothing on screen said which field the box
    /// belonged to.
    ///
    /// So the key travels with the draft and a mismatch reseeds. `for_field` is
    /// the FULLY-QUALIFIED name (the identity), and the draft is seeded with the
    /// **last dotted segment** — the partial name, which is what
    /// `rename_field` takes. Seeding it with the qualified name would invite the
    /// operator to press Rename on a string containing a dot, authoring a `/T`
    /// nothing can address.
    pub fn field_rename_mut(&mut self, for_field: &str) -> &mut String {
        if self.field_rename_key.as_deref() != Some(for_field) {
            self.field_rename_key = Some(for_field.to_owned());
            self.field_rename = for_field.rsplit('.').next().unwrap_or(for_field).to_owned();
        }
        &mut self.field_rename
    }

    /// The Properties panel's geometry draft.
    /// The selected text's style draft, for `properties::text`.
    ///
    /// No re-seed argument, unlike [`Self::field_rename_mut`]: the draft owns
    /// its own stamp and decides for itself when what it holds is stale, which
    /// is right here because the staleness condition includes the edit epoch
    /// and a caller would have to be handed that as well.
    pub fn text_style_mut(&mut self) -> &mut properties::text::TextStyleDraft {
        &mut self.text_style
    }

    /// The clicked text object's draft, for `properties::textobject`.
    ///
    /// No re-seed argument, like [`Self::text_style_mut`] and for its reason:
    /// the draft owns its `(page, object, epoch)` stamp and decides for itself
    /// when what it holds is stale.
    pub fn text_object_mut(&mut self) -> &mut properties::textobject::TextObjectDraft {
        &mut self.text_object
    }

    /// The selected annotation's memoised deletion collateral, for
    /// `properties::annotdelete`.
    ///
    /// No re-seed argument, like [`Self::text_style_mut`]: the memo owns its own
    /// `(id, epoch)` stamp and decides for itself when what it holds is stale,
    /// which is right here because the staleness condition includes the edit
    /// epoch and a caller would have to be handed that as well.
    pub fn annot_delete_mut(&mut self) -> &mut properties::annotdelete::DeletionPreview {
        &mut self.annot_delete
    }

    /// The selected form field's typed-property draft.
    ///
    /// No re-seed argument, like [`Self::text_style_mut`] and unlike
    /// [`Self::field_rename_mut`]: the draft owns its own `(name, epoch)` stamp
    /// and decides for itself when what it holds is stale, which is right here
    /// because the staleness condition includes the edit epoch and a caller
    /// would have to be handed that as well.
    pub fn field_props_mut(&mut self) -> &mut properties::fieldedit::FieldPropsDraft {
        &mut self.field_props
    }

    /// The selected widget's typed-property draft. See
    /// [`Self::field_props_mut`]; this one's stamp carries the placement too.
    pub fn widget_props_mut(&mut self) -> &mut properties::widgetedit::WidgetPropsDraft {
        &mut self.widget_props
    }

    pub fn geometry_mut(&mut self) -> &mut properties::geometry::GeometryDraft {
        &mut self.geometry
    }

    /// The Bookmarks panel's authoring state.
    pub fn bookmarks_mut(&mut self) -> &mut bookmarks::BookmarksUi {
        &mut self.bookmarks
    }

    /// The Attachments panel's authoring state — the optional description.
    ///
    /// Same shape as [`Self::bookmarks_mut`]: the body is handed
    /// `&mut PanelsState` and reaches its own state through an accessor, so the
    /// field stays private and no other panel can write it.
    pub fn attachments_mut(&mut self) -> &mut attachments::AttachmentsUi {
        &mut self.attachments
    }

    /// **The pages the operator has picked in the Pages panel.**
    ///
    /// ★ Read-only, and this is the accessor a `pages.*` dispatch arm must
    /// use when the first one lands. The ribbon's Pages tab already promises
    /// this set in every one of its tooltips — `pages.delete` is *"Remove
    /// **the selected pages** from this document"* — and
    /// `crate::shell::commands`' comment on that band says those verbs
    /// *"respect the thumbnail rail's selection when there is one"*.
    ///
    /// It is exposed *before* anything reads it, deliberately, because the
    /// alternative is that the first arm to arrive invents a second page
    /// selection of its own — the exact drift [`ObjectTreeUi::focus`]'s docs
    /// refuse for objects. Empty is a defined answer, not a missing one: with
    /// nothing picked those commands act on the current page.
    pub fn selected_pages(&self) -> &std::collections::BTreeSet<usize> {
        self.pages.selection.pages()
    }
}

/// Apply this project's scroll-bar style to `ui`.
///
/// Scoped to the `Ui` that owns the scroll area rather than to the app
/// style, because "always show a solid bar" is right for a narrow panel
/// column and not obviously right for every surface in the application.
///
/// Three settings, and all three are needed — see this module's header for
/// the measurement behind each:
///
/// 1. `solid()` over the `floating()` default, so the bar allocates layout
///    and is drawn when the pointer is elsewhere.
/// 2. `foreground_color = true`, so the handle is drawn in the visuals' TEXT
///    colour rather than `widgets.inactive.bg_fill` — which on a light
///    preset is a near-white handle on a near-white panel. This is also the
///    theme-respecting form: the handle inherits whatever contrast the
///    active theme gives its text, so it stays correct across light and dark
///    without a hard-coded colour.
/// 3. `bar_width = 10.0`, wide enough to grab with a mouse.
pub fn scroll_style(ui: &mut egui::Ui) {
    let mut scroll = egui::style::ScrollStyle::solid();
    scroll.foreground_color = true;
    scroll.bar_width = 10.0;
    ui.style_mut().spacing.scroll = scroll;
}

/// The width a scrolling container must declare so its rows are not
/// squeezed.
///
/// # The defect this prevents
///
/// `Ui::allocate_ui_with_layout_dyn` fits its requested size into the space
/// **remaining in the parent region**, so a row that asks for 600 pt inside
/// a 370 pt viewport receives 370. The row's `min_rect` therefore measures
/// exactly the viewport width, `ScrollArea` compares content against
/// viewport, finds them equal, and draws no bar. The visible symptom is a
/// label cut off at the panel's edge with no way to reach the rest of it,
/// and nothing errors or warns.
///
/// `auto_shrink([false, false])` does not help — it stops the area shrinking
/// *below* the viewport, it does not let content exceed it. `max_width` does
/// not help either; it bounds the viewport, which was already right.
///
/// The fix is to state the content's own width on the container:
/// `Ui::set_width` calls `set_max_width`, and `Placer::set_max_width` GROWS
/// `max_rect` rather than only shrinking it, which is what gives the rows
/// their real width and lets the area measure content > viewport.
///
/// # Why `.max(viewport)`
///
/// So a wide panel still fills rather than leaving a dead strip to the right
/// of the rows.
///
/// # Why this is a function and not three lines at the call site
///
/// So it can be tested. The RAG note this comes from is explicit that the
/// value must not be a measurement of the *laid-out* row — measuring is what
/// produced the squeezed number in the first place — and the difference
/// between "the intrinsic width of this text" and "the width this row ended
/// up with" is invisible at a call site and obvious in a test.
#[must_use]
pub fn content_width(row_widths: impl IntoIterator<Item = f32>, viewport: f32) -> f32 {
    row_widths
        .into_iter()
        .filter(|w| w.is_finite())
        .fold(viewport, f32::max)
}

/// The character this crate ends a shortened row with.
///
/// One code point, not three periods. Three periods measure wider, and at the
/// width where a row is being shortened at all, three periods is another
/// character and a half of the operator's text spent on the punctuation that
/// says text was spent.
pub const ELLIPSIS: char = '\u{2026}';

/// **Shorten `label` until it fits `available`, or say that it already does** —
/// `OPERATOR_REQUESTS.md` **O123**: *"rows that ellipsise with a tooltip
/// instead of hard-clipping mid-character."*
///
/// Returns `None` when the whole label fits, and `Some(shortened)` when it does
/// not. The caller draws whichever it got and attaches the **full** text on
/// hover in the `Some` case.
///
/// # ★★★ This reverses a recorded ruling, and the operator reversed it
///
/// `REVIEW_TRIAGE.md` §4 lists *"A7 — Objects rows should ellipsize"* under
/// *already decided against*, citing `SALVAGE.md:44` — *"Row text must not
/// clip; the old panel truncated with no horizontal scroll."* That ruling was
/// right about the OLD behaviour and it is being overturned on the operator's
/// own instruction, not quietly.
///
/// ★ And the requirement `SALVAGE.md` states is still met, by a different
/// route. What it forbade was **silent** loss: the old panel cut a row at the
/// pane's edge with no bar, no mark and no recovery. This shortens the row, says
/// so with a character the eye reads as *there is more*, and puts the whole
/// string one hover away. The thing that must not happen — an operator seeing
/// `AAAAAA+SpaceGrotesk-Bold 1` and having no idea a `2` was cut off — cannot
/// happen either way round.
///
/// # ★★ Why a `measure` closure rather than a `&Ui`
///
/// So the decision is a pure function and can be tested against a synthetic
/// font. Every earlier attempt at this in this crate ended as three lines
/// inside a draw closure, and [`content_width`]'s own doc records what that
/// costs: *"the difference between 'the intrinsic width of this text' and 'the
/// width this row ended up with' is invisible at a call site and obvious in a
/// test."*
///
/// # The search
///
/// Binary search over **character** counts, never bytes: slicing a UTF-8 string
/// at a byte offset panics mid-code-point, and this crate's rows carry the
/// middle dot, the em dash, the multiplication sign and font names with
/// accents. The predicate is monotone — a longer prefix is never narrower — so
/// the search is sound, and it costs `log2(len)` measurements on the rows that
/// need it and one on the rows that do not.
///
/// Returns `Some` of the bare ellipsis when not even one character plus the
/// ellipsis fits. That is a legitimate state at a very narrow dock and it is
/// **still better than a clipped row**: a lone ellipsis says *this is a row,
/// and it has content you cannot see here*, and it still carries the hover.
#[must_use]
pub fn elide_to_width(
    label: &str,
    available: f32,
    measure: impl Fn(&str) -> f32,
) -> Option<String> {
    if !available.is_finite() || available <= 0.0 {
        // No width to fit into. Nothing sensible to shorten to, and returning
        // `None` here is deliberate: the caller draws the whole label, `egui`
        // clips it, and the frame in which a pane has no width is not one worth
        // making a layout decision inside.
        return None;
    }
    if measure(label) <= available {
        return None;
    }
    let chars: Vec<char> = label.chars().collect();
    // `lo` always fits, `hi` never does. `lo` starts at zero because the bare
    // ellipsis is the floor of what this function will return.
    let (mut lo, mut hi) = (0usize, chars.len());
    while lo + 1 < hi {
        let mid = lo + (hi - lo) / 2;
        let mut candidate: String = chars[..mid].iter().collect();
        candidate.push(ELLIPSIS);
        if measure(&candidate) <= available {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let mut out: String = chars[..lo].iter().collect();
    out.push(ELLIPSIS);
    Some(out)
}

/// Measure the intrinsic width of a row's text, in points.
///
/// The *intrinsic* width — what the text would occupy with no wrapping and
/// no container — which is the number [`content_width`] needs and the one a
/// laid-out row cannot give (a laid-out row has already been clamped).
///
/// `layout_no_wrap` is what makes it intrinsic — the same call the widget
/// itself will make, so the number is the width the row would want rather
/// than an estimate of it.
///
/// The colour is [`egui::Color32::PLACEHOLDER`] because a galley's *width*
/// does not depend on its colour, and naming a real one here would tie a
/// measurement to a theme decision.
#[must_use]
pub fn text_width(ui: &egui::Ui, text: &str) -> f32 {
    let font_id = egui::TextStyle::Body.resolve(ui.style());
    ui.painter()
        .layout_no_wrap(text.to_owned(), font_id, egui::Color32::PLACEHOLDER)
        .rect
        .width()
}

mod tests;
