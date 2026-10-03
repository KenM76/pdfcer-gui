//! # `prefs` — the shell's own preferences, as distinct from the engine's settings
//!
//! ## Why this is not `pdfcer_core::settings`
//!
//! That store has a stated purpose and this is not it. Its own window says so
//! in its first paragraph: *"The PDF standard leaves some things genuinely
//! undefined … Where that happens, pdfcer asks you rather than deciding
//! quietly."* Every one of its entries exists because a **standard declines to
//! have an opinion**, and each one states what clause is silent.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/prefs/mod.md`.

/// Where pdfcer looks for a font it has to embed — the input
/// `tools.embed_fonts` needs, and an unrecorded dependency of that command's
/// blocker. See its header.
pub use crate::fontsearch as fonts;

/// How much memory pdfcer may spend so a page it has already drawn does not
/// have to be drawn again.
pub use crate::prefscache as cache;

/// How big the **program's own controls** are drawn — the one accessibility
/// preference, and the only one here that changes nothing about the document.
pub use crate::chromescale as chrome;

/// What an operator is shown when a page **first appears** — read once per
/// document open, never on the hot path.
pub use crate::openingfit as opening;

/// **What the three Export windows open with** — `OPERATOR_REQUESTS.md`
/// **O196**. Its own file for the reason [`printing`] has one: it carries this
/// group's whole file format, parser and writer together, and the judgement
/// about *which* of those windows' settings may be remembered is worth keeping.
/// The DXF scale is the interesting omission; see its header.
pub mod exporting;

mod families;
/// **Whether the canvas grows to show what sits off the sheet — one answer
/// per ribbon mode.** Its own file because *why the answer is per mode at all*
/// is the whole of the operator's request, and because it carries this group's
/// file format — parser and writer together, on [`printing`]'s precedent. See
/// its header.
pub mod offpage;
pub mod shortcuts;
pub mod snapshot;

/// Which chord means which form-field paste — O58. Its own file because
/// neither order is obviously right and the argument for each is worth keeping.
pub use crate::pastechords;
/// **What the Print window opens with** — `OPERATOR_REQUESTS.md` **O166**.
/// Its own file because deciding *which* of that window's controls may be
/// remembered is a judgement worth keeping, and because it carries this group's
/// whole file format — parser and writer together. See its header.
pub mod printing;

/// How sharply a page is drawn, and how long zoom waits before drawing it.
/// The two preferences that change what a **frame costs**.
pub use crate::renderquality as quality;

/// What a plain wheel does when the document is not one long scroll — O30.
pub use crate::wheelpaging as wheel;

/// What colour the recognised text is drawn in over a scan — O229. Its own
/// file because the preferences file's notation for a colour, and the refusal
/// rule behind it, are worth keeping together with their tests.
pub use crate::ocrlayerpref as ocrlayer;

use std::path::PathBuf;

pub use cache::PageCache;
pub use chrome::{DEFAULT_UI_SCALE, MAX_UI_SCALE, MIN_UI_SCALE, UI_SCALE_STEP};
// `pub`, unlike [`PrintPrefs`] below: every type these three groups hold --
// `ImageFormat`, `PageScope`, `PageSeparator`, `LineEndings` and the engine's
// own `DxfUnits` and `DxfText` -- is already `pub`, so no visibility chain
// forces them down. Same storage decision, different constraint.
pub use exporting::{
    ExportDxfPrefs, ExportImagePrefs, ExportPrefs, ExportTablePrefs, ExportTextPrefs,
    MAX_EXPORT_DPI, MAX_JPEG_QUALITY, MIN_EXPORT_DPI, MIN_JPEG_QUALITY,
};
pub use offpage::OffPagePrefs;
pub use opening::{OpeningFit, PageChrome};
pub use pastechords::PasteChords;
pub use shortcuts::ShortcutPrefs;
pub use snapshot::SnapshotPrefs;
// `pub(crate)`, not `pub`: `PrintPrefs` carries `dialogs::print::spooler`'s own
// `pub(crate)` types, so it can be no more visible than they are. `Prefs` stays
// `pub` and holds it on a `pub(crate)` field, which is legal and is what keeps
// `private_interfaces` quiet.
pub use crate::redact::RedactionReach;
pub use crate::remotecontrol::RemoteControl;
pub use printing::PrintPrefs;
pub use quality::{DEFAULT_SETTLE_MS, MAX_SETTLE_MS, MIN_SETTLE_MS, RenderQuality};
pub use wheel::WheelPaging;

pub use crate::viewer::ceiling::{
    DEFAULT_MAX_ZOOM_PERCENT, MAX_MAX_ZOOM_PERCENT, MIN_MAX_ZOOM_PERCENT,
};

/// Format a percentage for the preferences file without an exponent or a
/// trailing `.0`.
fn format_percent(value: f32) -> String {
    format!("{value:.0}")
}

/// The file this store is written to, beside `settings.txt`.
// ui-text-exempt: a file name, never displayed.
pub const PREFS_FILE: &str = "preferences.txt";

/// **A stored `bool` as the shell's own auto-hide setting.**
#[must_use]
pub fn auto_hide(on: bool) -> egui_shell::peek::AutoHide {
    if on {
        egui_shell::peek::AutoHide::OnHover
    } else {
        egui_shell::peek::AutoHide::Off
    }
}

/// The shell's own preferences.
#[derive(Debug, Clone, PartialEq)]
pub struct Prefs {
    /// How sharply a page is rasterised.
    pub render_quality: RenderQuality,
    /// **How far a redaction may reach beyond the regions the operator
    /// marked** — O211, and the one preference in this struct whose wrong
    /// value destroys content.
    ///
    /// Read at the moment a removal is prepared or staged, never held on the
    /// session across a settings change: `crate::redact`'s three entry points
    /// each take it as an argument so the value in force is the one on screen
    /// when the operator pressed the control, not the one that happened to be
    /// set when the document opened.
    pub redaction_reach: RedactionReach,
    /// **How much memory the page cache may hold**, so a page already drawn
    /// is not drawn again.
    ///
    /// Read every frame by `crate::app::settle::fill_strip`, which hands it
    /// to `StripRasters::retain` — the one place it is spent. Read live rather
    /// than at open, unlike [`Self::opening_fit`]: shrinking it must take effect
    /// at once, because an operator reaching for a smaller value is an operator
    /// whose machine is already struggling.
    pub page_cache: PageCache,
    /// How long a zoom must stop changing before it is committed to a real
    /// rasterisation, in milliseconds.
    ///
    /// Stored as a number rather than as a `Duration` because that is what the
    /// file holds and what the control edits; `app::settle` converts once,
    /// at the one place it is read.
    pub zoom_settle_ms: u64,
    /// **The highest zoom the operator wants to be able to reach**, as a
    /// percentage — `OPERATOR_REQUESTS.md` O24.
    ///
    /// > *"add a setting so the user can set the maximum zoom … I'm not
    /// > concerned about the practicality of offering such a high zoom. it is
    /// > up to the user to determine how much of a performance hit they want
    /// > to take."*
    ///
    /// That last sentence is why this has no guard, no warning and no
    /// preflight. The trade is explicitly his; the setting's whole job is to
    /// be honest about what it does and to actually do it.
    ///
    /// It is also the control he asked for to **compare the two rendering
    /// paths**: the shell rasterizes the whole page while it can and switches
    /// to the visible region only when it cannot. Set this low and he never
    /// leaves the whole-page path; set it high and he exercises the region
    /// path. A threshold rather than a mode, which explains itself where a
    /// checkbox would have to be explained.
    ///
    /// Stored as a percentage because that is what the status bar shows and
    /// what he said — *"1,000,000,000,000%"*. `f32` is exact to 2^24, so a
    /// percentage stays whole to 16.7 million; beyond that the stored value
    /// rounds, which is immaterial at zooms where one screen pixel is a
    /// millionth of a point.
    pub max_zoom_percent: f32,
    /// **Folders pdfcer searches when it has to embed a font a document names
    /// but does not carry**, in search order.
    ///
    /// Empty by default, and the emptiness is honest rather than a gap:
    /// `pdfcer`'s own note is that **"the source fonts come from
    /// `--font-dir`; pdfcer never goes looking"**, so a shell that guessed
    /// `C:\Windows\Fonts` would be embedding whatever that machine happened
    /// to hold into an operator's document — a licensing decision made on
    /// their behalf, silently.
    ///
    /// See [`fonts`] for the list rules and for why this is a preference rather
    /// than a `pdfcer_core::settings` entry.
    pub font_folders: Vec<std::path::PathBuf>,
    /// **Whether to search the fonts installed on this computer as well.**
    ///
    /// `OPERATOR_REQUESTS.md` **O50**, in his words: *"just a simple checkbox
    /// to include fonts from the OS installed font folders."*
    ///
    /// **`false` by default, and that is the whole of the licensing
    /// argument surviving intact.** The field above explains why pdfcer must not
    /// go looking on its own; this does not overrule that, it satisfies it. The
    /// objection was never to *using* system fonts — it was to pdfcer deciding
    /// silently, and an explicit, persistent, off-by-default switch is the
    /// operator making that decision once, visibly, where they can find it
    /// again.
    ///
    /// It is a **separate preference** rather than the two OS folders being
    /// appended to [`Self::font_folders`] when the box is ticked, and the
    /// difference shows up the day the machine changes: a stored *intent*
    /// ("use this computer's fonts") still means the right thing on a new
    /// machine with a different `%WINDIR%`, while two stored *paths* would name
    /// folders that no longer exist. See [`fonts::os_font_dirs`].
    pub use_os_fonts: bool,
    /// How the first page of a newly opened document is sized to the window.
    ///
    /// Read **once**, by [`Self::seed_view`], in the one place a document is
    /// adopted. Unlike the two above it is not consulted again — changing it
    /// while a document is open must not resize the page the operator is
    /// looking at, because they may have zoomed it deliberately since.
    pub opening_fit: OpeningFit,
    /// Which of the three View ▸ Display overlays are already on when a
    /// document opens.
    ///
    /// Read once, with [`Self::opening_fit`], and for the same reason.
    pub chrome: PageChrome,
    /// **What a plain mouse wheel does under a one-page-at-a-time display
    /// mode** -- `OPERATOR_REQUESTS.md` O30.
    ///
    /// Unlike the two above this one is consulted **every frame**, not once at
    /// open: it is a live preference about an input gesture, and an operator
    /// who changes it from the status bar expects the very next notch to obey.
    /// See [`WheelPaging`] for why the choice exists only under
    /// `PageDisplay::Single` and `Facing`.
    pub wheel_paging: WheelPaging,
    /// Whether another program may drive the window through the live link.
    pub remote_control: RemoteControl,
    /// **Wash the fillable fields, so you can see what can be typed into** —
    /// `OPERATOR_REQUESTS.md` O96.
    ///
    /// Ken, 2026-09-02: *"in our display section we should have an option to
    /// shade the form fields like acrobat does."*
    ///
    /// # Why this is not the thing rule 4 forbids, and the distinction is
    /// exact
    ///
    /// The standing rule is *applied content renders exactly as saved content
    /// will render* — no badge, tint or provisional styling drawn into the page.
    /// This looks like a tint over part of the page and is **not** one, for a
    /// reason that has to be stated rather than assumed:
    ///
    /// **A field is a control, not content.** The wash is the affordance that
    /// says *this box accepts typing*, which is the same class as the pointing
    /// hand `canvas::forms` already puts over a widget and the same class as a
    /// snap indicator. It says nothing about pdfcer's confidence in anything and
    /// marks no inference.
    ///
    /// The property that keeps it honest is where it is drawn: it is painted
    /// by the canvas **overlay**, over the finished page texture, and reaches
    /// no rasterizer. It cannot appear in a print, an export, a Save, or a
    /// `render-page`. The one-line test rule 4 is judged by — *would a
    /// screenshot of the canvas differ from a screenshot of the same document
    /// saved and reopened?* — answers **yes and that is correct here**, because
    /// what differs is a control's affordance rather than pdfcer marking its own
    /// uncertainty.
    ///
    /// **On by default, which is Acrobat's answer** and the useful one: an
    /// operator who does not know a form is fillable is the person this exists
    /// for, and they will not go looking for a setting to reveal it. Somebody
    /// who wants the page clean turns it off once.
    pub shade_form_fields: bool,
    /// **What colour the recognised text is drawn in over a scan** — O229, in
    /// his words: *"we should be able to change the editing colour of the ocr
    /// text layer just for editing, and remember the user's setting."*
    ///
    /// *"Just for editing"* is the whole of the rule, and it is why this is
    /// a preference at all rather than anything the document carries. The
    /// layer is **invisible text**: mode-3 glyphs that render as nothing, in
    /// this file and in every other reader. Choosing a colour for it changes
    /// what pdfcer paints over the page while the operator is comparing the
    /// two, and cannot reach a save, an export or a print — there is nothing
    /// for it to reach, because the colour is never written anywhere near the
    /// content stream.
    ///
    /// It is therefore not the thing R8b forbids either, for
    /// [`Self::shade_form_fields`]'s reason stated once more: the overlay is a
    /// deliberate X-ray the operator switched on, drawn by the canvas overlay,
    /// showing content the page genuinely holds. It marks no inference and
    /// says nothing about pdfcer's confidence.
    ///
    /// The live copy the painter reads is in [`egui::Context`] memory —
    /// `crate::canvas::ocrlayer::colour` — and `crate::app::frame` mirrors
    /// this into it once a frame, in that direction only. Two homes for the
    /// reason `crate::canvas::chunks` has two: the painter is handed a context
    /// and nothing else.
    pub ocr_layer_colour: [u8; 3],
    /// The recogniser Recognise text last ran with; `None` until one has run.
    ///
    /// Kept when this build lacks the engine named: the dialog then offers
    /// its first available engine without erasing the choice, so a build that
    /// has it again picks it back up.
    pub ocr_engine: Option<crate::ocr::EngineId>,
    /// The time-stamping server the Sign window last signed with; `None` for
    /// none. Never filled in on the operator's behalf: naming a server is
    /// consent to contact it.
    pub sign_timestamp_server: Option<String>,
    /// **How a document the program has never seen is laid out** —
    /// `OPERATOR_REQUESTS.md` O80.
    ///
    /// The operator: *"it should remember my page display preferences from my
    /// last closing of the program. Example if I press show one page at a time
    /// and enable flip pages."*
    ///
    /// # Why the answer was "it already does" and he was still right
    ///
    /// A page display **is** remembered — by
    /// [`crate::viewer::remembered`], keyed on the **document's path**, and
    /// written synchronously the moment he presses the control. That store is
    /// correct and is not changing.
    ///
    /// But it can only answer for a document it has seen. Open a *different*
    /// drawing and the shell fell straight through to
    /// [`crate::viewer::PageDisplay::default_for_mode`] — continuous in Read,
    /// single everywhere else — so the choice he made on sheet A meant nothing
    /// on sheet B. From his chair that is the program forgetting.
    ///
    /// ⇒ **Three tiers, and the order is the whole design:**
    ///
    /// | tier | says | wins over |
    /// |---|---|---|
    /// | `viewer::remembered` (per document) | *"this drawing is read facing"* | everything |
    /// | this (global) | *"I read drawings one page at a time"* | the per-mode default |
    /// | `default_for_mode` | *"Read is for reading"* | nothing |
    ///
    /// # Why `Option`, and why collapsing it would be a regression
    ///
    /// `None` means *"fall through to the per-mode default"*, and it has to
    /// stay expressible. `MODES_AND_PANELS.md`'s per-mode rule — Read is
    /// continuous — is a deliberate decision from 2026-08-13, and an operator
    /// who has never stated a preference must keep getting it. A plain
    /// `PageDisplay` here would have to pick one, and picking one silently
    /// overrides that rule for everybody.
    ///
    /// This is the same refusal [`crate::viewer::remembered::recall`] already
    /// makes for its own layer: *nothing recorded* and *recorded as single*
    /// are different states and must not be collapsed.
    ///
    /// # This overturns a written decision, and the header it overturns has
    /// been rewritten rather than left contradicting the code
    ///
    /// `crate::prefs::opening`'s header said a global default for page
    /// display was *"a second axis colliding with the per-document one … and
    /// is deliberately unbuilt"*. The collision is real and the answer is
    /// **precedence**, not absence: per document beats global beats per mode.
    /// Three tiers with a stated order is one axis, not two.
    pub default_page_display: Option<crate::viewer::PageDisplay>,
    /// **Whether the canvas shows, and reaches, what sits off the
    /// sheet — one remembered answer per ribbon mode**, 2026-09-11.
    ///
    /// Not a `bool`, and that is the whole design: Read opens without the
    /// band of pasteboard that off-page material needs, Review and Edit
    /// open with it, and each mode remembers the operator's own last word
    /// independently. [`offpage`] carries the argument, the defaults and
    /// this group's file keys.
    ///
    /// Read at exactly two moments — document open
    /// (`crate::app::lifecycle`) and ribbon-mode change
    /// (`crate::app::surfaces`) — and written at one, the
    /// `ToggleViewChrome` arm in `crate::app::actions`. Everything after
    /// that reads [`crate::viewer::ViewState::off_page`], which is where a
    /// per-document answer legitimately diverges from the remembered one.
    pub off_page: OffPagePrefs,
    /// The operator's own keyboard shortcuts. See [`shortcuts`].
    pub shortcuts: ShortcutPrefs,
    /// The resolution View ▸ Snapshot copies at. See [`snapshot`].
    pub snapshot: SnapshotPrefs,
    /// **Whether a click selects a whole container or one line inside it** —
    /// `OPERATOR_REQUESTS.md` **O70**, 2026-08-31.
    ///
    /// The persisted half of [`crate::canvas::smart`]. The live value lives in
    /// `egui::Memory`, because the canvas reads it from places that have a
    /// context and nothing else; this is where it survives a restart.
    ///
    /// **`true` by default**, which is the same argument that module makes:
    /// the checkbox exists so the behaviour can be turned OFF, and the
    /// behaviour is what every drawing program in the class does.
    pub smart_select: bool,
    /// **Are the chunk boxes drawn on a selected text block?** —
    /// `OPERATOR_REQUESTS.md` **O215**.
    ///
    /// The persisted half of [`crate::canvas::chunks`]; the live value lives in
    /// `egui::Memory` for [`Self::smart_select`]'s reason, and this is where it
    /// survives a restart.
    ///
    /// **`true` by default**, on the same argument: the switch exists so the
    /// boxes can be turned OFF, and an operator who has to find a checkbox
    /// before the thing he asked for appears has not been given it.
    pub text_chunks: bool,
    /// **Is going to a search hit allowed to change the zoom?** —
    /// `OPERATOR_REQUESTS.md` **O163**, 2026-09-09, his words: *"add a
    /// checkbox option to our search bar called zoom - when I unchecked just
    /// jump to the page and highlight the found item as before but don't
    /// change the zoom."*
    ///
    /// The persisted half of [`crate::find::FindState::zoom_on_jump`]. The
    /// live value lives on the find state because the bar reads and writes it
    /// every frame and `Prefs` is not reachable from a widget; this is where
    /// it survives a restart. Written by
    /// [`PrefAction::FindZoom`](crate::app::actions::prefs::PrefAction::FindZoom), read back into
    /// the find state once at startup.
    ///
    ///
    /// **This is not a search option.** It changes nothing about which
    /// text matches, so it is deliberately not a `FindOptions` field: those
    /// re-run the search when they change, and re-running a search because a
    /// view preference moved would throw away the operator's place in the
    /// result list for no reason.
    pub find_zoom_on_jump: bool,
    /// **Does a search ignore whitespace at either end of the query?** —
    /// `OPERATOR_REQUESTS.md` **O180**, 2026-09-12. His words: *“trailing
    /// spaces/tabs/etc stops a search from finding text on the page that
    /// doesn't have these symbols … copy pasting from excel seems to give
    /// a trailing space that I have to remove to search.”*
    ///
    /// The persisted half of [`crate::find::FindState::trim_query`]; the
    /// live value lives on the find state for the same reason
    /// [`Self::find_zoom_on_jump`]'s does, and this is where it survives a
    /// restart. Written by `Action::SetFindTrim`, read back into the find
    /// state once at startup.
    ///
    ///
    /// **This one DOES change what matches**, unlike its neighbour
    /// above, so changing it makes a standing result set wrong rather than
    /// merely stale. `FindState::set_trim_query` clears the results for
    /// that reason. It is still not a `FindOptions` field: those are the
    /// bar's own menu, and he asked for this one in Settings.
    pub find_trim_query: bool,
    /// **Is the pages panel allowed to draw page pictures?** —
    /// `OPERATOR_REQUESTS.md` **O187**, 2026-09-12, and it closes the half
    /// **O151** left open on 2026-09-08.
    ///
    /// O151 stopped pdfcer turning the tick off behind the operator's back.
    /// It did not make the tick *survive a restart*, so an operator who
    /// cleared it because previews were slow on their sheet set met them
    /// again on the next launch — which is the same complaint wearing a
    /// longer fuse. His words: *“the draw page previews timeout needs to be
    /// remembered”*, and a timeout remembered without the tick beside it
    /// would be half an answer.
    ///
    /// The persisted half of
    /// [`crate::pagebudget::ThumbnailCache::previews_on`].
    /// The live value lives on the cache because the checkbox reads and
    /// writes it every frame and `Prefs` is not reachable from a panel
    /// widget; this is where it survives a restart. Written by
    /// [`PrefAction::PagePreviews`](crate::app::actions::prefs::PrefAction::PagePreviews), read
    /// back into the cache once at startup.
    ///
    /// **`true` by default** — the behaviour every build has had, and
    /// [`Self::smart_select`]'s argument exactly: a checkbox that exists so
    /// something can be turned OFF defaults to ON.
    ///
    pub page_previews: bool,
    /// **How long one page picture may take, in milliseconds — and
    /// `0` means never give up** — `OPERATOR_REQUESTS.md` **O187**,
    /// 2026-09-12: *“setting it to 0 should set it to infinity (never time
    /// out)”*.
    ///
    /// The persisted half of
    /// [`crate::pagebudget::ThumbnailCache::budget`], which
    /// holds the live value as an `Option<Duration>` — `None` being the
    /// same fact in the type system rather than in a sentinel.
    ///
    /// # Why a `u64` of milliseconds and not an `Option`
    ///
    /// Because the preferences **file** is the operator's, and they type
    /// into it. `0` is the number he asked to be able to type, it is what
    /// the box beside the checkbox shows, and it is what the file records;
    /// a file that said `page_preview_budget_ms = none` would be a third
    /// spelling of the same answer for a reader to get wrong. The sentinel
    /// is converted to `None` at exactly one place —
    /// [`crate::pagebudget::budget_from_millis`] — and never
    /// re-derived.
    ///
    /// # `0` is also the shipped default — O225
    ///
    /// *"draw page previews should be set to 'no limit' by default."* A
    /// budget that trips leaves a tile with no picture, and nothing on
    /// screen distinguishes that from a page that could not be drawn at
    /// all; the operator is looking for a sheet, and a blank where a sheet
    /// should be is the one outcome the panel exists to prevent.
    /// `crate::pagebudget::PAGE_BUDGET_DEFAULT` carries the
    /// per-page measurements the decision was made against.
    ///
    /// # ⚠ What `0` actually costs, stated because the operator is entitled
    /// to know
    ///
    /// The watchdog is not armed at all, so a page that would have taken a
    /// minute takes a minute, on the UI thread, with the application
    /// unresponsive for the duration. The box says **never** in words when
    /// it is set and the tooltip says what never means, so the state is
    /// legible rather than merely true.
    ///
    /// **Out-of-range values are clamped, `0` is not** — see
    /// `budget_from_millis`. 1 ms would be an off switch wearing a number,
    /// so it becomes the 100 ms floor; `0` is a deliberate instruction and
    /// survives untouched.
    pub page_preview_budget_ms: u64,
    /// **Does the ribbon band hide itself until the pointer reaches the tab
    /// strip?** — his instruction of 2026-09-05, *"we should also add the
    /// capability to auto hide the ribbon until we hover over top of it."*
    ///
    /// The persisted half of [`egui_shell::peek`]. Pushed into
    /// `RibbonState::sync_auto_hide` once per frame; the *revealed* half is
    /// per-frame state the shell owns and is deliberately not stored — a
    /// restart that reopened with the band stuck open would be a setting that
    /// had silently changed itself.
    ///
    /// **Off by default.** An operator who has never heard of the feature
    /// gets the ribbon they have always had, and the two ways to it are the
    /// Settings window and `view.ribbon_autohide` on View ▸ Window. The tab
    /// strip never hides with it — see [`egui_shell::peek`] on why Office's
    /// full *Auto-hide Ribbon* is the setting people get stuck in.
    pub ribbon_auto_hide: bool,
    /// **Does the left rail hide itself until the pointer reaches its edge?** —
    /// the same instruction, *"left rail should also have the option to auto
    /// hide as well."*
    ///
    /// Off by default, for [`Self::ribbon_auto_hide`]'s reason. When it is on
    /// the rail still reserves `egui_shell::dock::rail::PEEK_WIDTH_PTS` of
    /// permanent, chevron-marked edge, so the panels that are reachable ONLY
    /// from the rail — `markup.comments` in Read, which is on no tab that mode
    /// shows — stay reachable in every state of this setting.
    pub rail_auto_hide: bool,
    /// **Which chord means which form-field paste** — `OPERATOR_REQUESTS.md`
    /// **O58**, operator ruling 2026-08-29.
    ///
    /// See [`PasteChords`]. Read when the shell's keymap is assembled and on
    /// every change to it, never per keystroke: it does not decide what a
    /// command *does*, it decides which key *reaches* it.
    pub paste_chords: PasteChords,
    /// **How big the program's own controls are drawn**, as a multiplier on
    /// whatever the operating system already asked for.
    ///
    /// # A multiplier, not a size, and the distinction is the whole design
    ///
    /// `egui`'s `Context::set_zoom_factor` multiplies the *native* pixels per
    /// point — the value the window system reports, which on Windows is the
    /// display-scaling percentage the operator set for every application on the
    /// machine. So `1.0` here does not mean *"draw at 96 dpi"*; it means
    /// **"whatever you already decided"**, and this preference expresses only
    /// the delta pdfcer needs on top of it.
    ///
    /// That is the correct relationship and it is easy to get backwards.
    /// Storing an absolute point size would make pdfcer the one application on
    /// the machine that ignores the display setting — so an operator who moved
    /// a 4K laptop to a 1080p monitor would fix every program but this one.
    ///
    /// # It is not stored as a `Duration`-style integer, unlike its neighbours
    ///
    /// [`Self::zoom_settle_ms`] is a `u64` because the file holds a whole
    /// number of milliseconds. A scale has no such natural unit, and rounding
    /// it to, say, whole percent in the struct would put the rounding rule in
    /// two places — the parser and the control. [`chrome::normalise_ui_scale`]
    /// is the one place instead, applied on the way in.
    ///
    /// # Live-previewed, like the theme, and for the identical reason
    ///
    /// `app::frame`'s step 0 reads this from the **draft** while the settings
    /// window is open. A scale cannot be judged from a number — you choose it
    /// by seeing whether you can read the ribbon — so it is the second of the
    /// two settings in this window that take effect before Save. Cancel drops
    /// the draft and the size reverts with it; there is no separate preview
    /// state that could get out of step with what will be written.
    pub ui_scale: f32,
    /// Whether toolbar icons draw one part in a muted accent colour, as
    /// Office and SOLIDWORKS do — `OPERATOR_REQUESTS.md` O232. `true` by
    /// default (O265); `false` is the plain single-colour set. Previews live, like
    /// [`Self::ui_scale`], because it too changes only the program's own
    /// appearance. See [`crate::icons::accent`].
    pub colour_icons: bool,
    /// **Which rendering standard the operator chose**, by its engine id —
    /// or `None` if they have never chosen one in any sitting.
    ///
    /// # The defect this exists for
    ///
    /// The operator, 2026-08-26: *"When I go to settings and select some of the
    /// standards the save button is greyed out and I can't save the change."*
    ///
    /// Both halves of that are literally true, and the second explains the
    /// first. A preset's *values* were the only thing recorded, and
    /// `identical_siblings` measures that **all eight PDF/X and PDF/A presets
    /// apply byte-identical render settings** — they genuinely make the same
    /// demands of a renderer and differ in what they demand of a *file*. So
    /// selecting a second standard changed no value, `Draft::is_dirty` was
    /// therefore false, and Save was correctly greyed for a draft that really
    /// did equal what was already saved.
    ///
    /// And the worse half he had not seen yet: **his choice was discarded.**
    /// Nothing recorded it, so on reopening the window `preset::matching`
    /// supplied the derived reading — *"your settings look like this one"* —
    /// which returns the FIRST of the eight. Choose PDF/X-4, come back, and the
    /// window says PDF/X-1a.
    ///
    /// # Why persisting it is not the thing the old comment refused
    ///
    /// `Draft::chosen_preset` carried a deliberate argument for *not* storing
    /// this: the derived reading *"cannot claim an intent nobody expressed in
    /// this sitting."* That reasoning is right about not **inventing** an
    /// intent and was applied to the opposite case. An operator clicking
    /// PDF/X-4 has expressed an intent; discarding it and substituting a guess
    /// is the invention the argument was written against.
    ///
    /// # It is a preference, not a setting, and that is the correct home
    ///
    /// `pdfcer_core::settings::Settings` is the **engine's** store and describes
    /// what to render. This is a record of what the operator *asked for*, which
    /// changes no render — the values it implies are already in `Settings`. It
    /// is also this shell's to keep: the engine has no concept of the window
    /// having been used.
    ///
    /// Retired on save when it no longer holds. If a control is changed by
    /// hand afterwards the settings stop being that standard's, and
    /// `preset::live_choice` already declines to show it —
    /// [`crate::dialogs::settings::commit`] clears the stored value to match,
    /// so the file never carries a claim the settings contradict.
    pub chosen_standard: Option<String>,
    /// **The operator's name, written into every comment they author** —
    /// `/T`, which §12.5.6.4 Table 170 defines as *"the name of the person who
    /// created the annotation"*.
    ///
    /// # Why this exists at all
    ///
    /// Because `pdfcer-core` gained `MarkupNote` in `Pass 150.0` and every
    /// annotation this shell authored before that was **anonymous**. Any
    /// reviewer UI — Acrobat's comment list, pdfcer's own Comments panel —
    /// shows an author column, and pdfcer's rows were blank in it. A comment
    /// nobody signed is a comment nobody can answer.
    ///
    /// # Why a PREFERENCE and not a setting
    ///
    /// `Settings` is the engine's store and describes how to read and write
    /// PDFs. This describes **the person at the keyboard**. It is the same
    /// distinction [`Self::chosen_standard`] makes one field up, and it is
    /// sharper here: two operators sharing one machine's settings file would
    /// still want two names.
    ///
    /// # Empty means anonymous, and that is a real choice
    ///
    /// The default is empty and an empty value writes **no `/T` at all**,
    /// which is legal and is what every reviewer UI shows as an anonymous
    /// note. It is not a placeholder to be filled with a guess: pdfcer does not
    /// know the operator's name, and reading one out of the OS user account
    /// would put a Windows login into a document that leaves the building.
    pub author_name: String,

    /// **Whether pdfcer may still offer to become the default PDF program**
    /// on startup - `OPERATOR_REQUESTS.md` **O173**, his words of 2026-09-10:
    /// *"Ask once with a don't show me again check box option."*
    ///
    /// `true` (the default) means the offer may be made; ticking the dialog's
    /// *Don't ask me again* writes `false` here and it is never made again.
    ///
    /// # Named for the thing it permits, not the thing it suppresses
    ///
    /// A `suppress_default_app_prompt` would default to `false`, and a
    /// hand-editable file whose every off-switch defaults to off is a file
    /// where "on" and "off" stop meaning anything. The same choice
    /// [`Self::find_zoom_on_jump`] makes.
    ///
    /// WARNING: **this governs the QUESTION, never the capability.** The button
    /// lives at the top of Settings whether this is `true` or `false` - O173
    /// asks for both, and a checkbox that removed the feature along with the
    /// prompt would be a checkbox nobody could undo.
    /// `crate::dialogs::settings` does not read this field at all, which is the
    /// mechanical form of that promise.
    ///
    /// # It is also **not** a record of whether pdfcer is the default
    ///
    /// That question has exactly one honest answer and it is
    /// [`crate::app::assoc::is_default`], which asks Windows. A remembered
    /// answer would go stale the moment the operator installed anything else
    /// that opens PDFs, and would then be a settings line asserting something
    /// false about the machine.
    pub ask_default_app: bool,

    /// **Where Acrobat is, when the operator has had to say** —
    /// `OPERATOR_REQUESTS.md` **O122**: *"have a setting where people can
    /// change it."*
    ///
    /// # Empty is the normal value, and it means "find it yourself"
    ///
    /// Not "there is no Acrobat". `crate::acrobat::resolve` reads an empty or
    /// whitespace-only value as *unset* and goes and asks Windows — the
    /// `App Paths` registrations first, the registered `.pdf` handler second.
    /// Nearly every machine will never write anything here.
    ///
    /// The distinction matters because clearing a text field is how a person
    /// un-sets it. A cleared field read as *"configured to nothing"* would
    /// permanently suppress the button with no way back except editing this
    /// file by hand, which is the trap version of an escape hatch.
    ///
    /// # Why this exists at all, given discovery works
    ///
    /// Because discovery reads registrations, and a registration is a thing an
    /// installer writes. A portable copy, a second version kept for a client,
    /// an install on a volume Windows was never told about, a build where the
    /// registration was written with an environment variable in it that pdfcer
    /// does not expand — every one of those is an Acrobat that exists and that
    /// discovery cannot see.
    ///
    /// And it is visible in Settings **whether or not discovery succeeded**,
    /// which is O122's decision and is the load-bearing half: somebody in that
    /// position arrives having seen no button at all, so the only place they
    /// can be told the feature exists is the field that fixes it. See
    /// `crate::dialogs::settings::acrobat`.
    ///
    /// # A path, held as a `String` rather than a `PathBuf`
    ///
    /// Because it is a value the operator TYPES, and a half-typed path is not
    /// a path. `PathBuf` would claim more than is known about the contents of
    /// a text field, and every consumer converts at the point of use anyway —
    /// where the existence check happens.
    pub acrobat_path: String,
    /// **Where Acrobat's downloaded trust list is**, when this machine does not
    /// keep it where pdfcer looks. Empty means *"look in the usual places"*.
    ///
    ///
    /// # Why a preference exists when discovery works
    ///
    /// The same reason [`Self::acrobat_path`] has one, arriving from a
    /// different direction. That field exists because discovery reads
    /// *registrations*; this one exists because discovery reads a **list of
    /// conventional locations**, and a convention is wrong the first time a
    /// profile is redirected to a network share, an Acrobat track this build
    /// does not name is installed, or an administrator hands somebody a store
    /// to use.
    ///
    /// **R9's escape hatch, and it is the load-bearing half.** The control
    /// that inspects a store is ABSENT when there is no store — an unavailable
    /// capability renders nothing. Somebody in that position therefore sees no
    /// evidence the feature exists, so the path field is drawn in Settings
    /// whether or not discovery succeeded. It is the only thing that can fix
    /// the case where discovery failed.
    ///
    /// # A cleared field means "ask this machine", never "there is no store"
    ///
    /// Clearing a text box is how a person un-sets it. Reading an empty value
    /// as a positive choice would suppress the feature permanently with no way
    /// back except hand-editing this file, which is the trap version of an
    /// escape hatch — [`Self::acrobat_path`]'s own note, and it applies here
    /// unchanged.
    ///
    /// # Held as a `String`, like its neighbour
    ///
    /// Because it is a value the operator TYPES, and a half-typed path is not a
    /// path. `crate::trust::locate` converts at the point of use, which is
    /// where the existence check happens and where the answer is knowable.
    ///
    /// ⚠ **This is a location, not a permission.** Whether pdfcer may read the
    /// file at all is `pdfcer_core::settings::AcrobatTrustStore`, which lives in
    /// the engine's own store because the same choice governs
    /// `pdfcer verify-signatures`. Filling this in while that is `Off` changes
    /// nothing, and the Settings group says so where both controls are drawn.
    pub acrobat_trust_store_path: String,

    /// **What the Print window opens with** — `OPERATOR_REQUESTS.md`
    /// **O166**, his words of 2026-09-10: *"the printer dialogue box needs to
    /// remember our last settings."*
    ///
    /// Read once by `crate::dialogs::print::PrintDialog::open`, written back
    /// when a job is committed. See [`PrintPrefs`] for the rule that decides
    /// which of that window's controls are in here and which are deliberately
    /// not — the short version being that a setting is remembered only if it
    /// would still be the right answer for a **different document**.
    ///
    /// `pub(crate)` where every field above it is `pub`, because it carries
    /// the print dialog's own crate-private types. That is deliberate and is
    /// argued in [`printing`]'s header: storing the real types rather than a
    /// mirrored set is what makes a new variant a compile error here instead of
    /// a silent round-trip to the default.
    /// **What the three Export windows open with** -- O196.
    ///
    /// One field holding three groups, because a window reads only its own and
    /// a fourth export window should add a struct rather than widen one. Read
    /// by each dialogue's `open`, written back when Export is committed.
    ///
    /// `pub` where [`print`](Self::print) below is `pub(crate)`: these groups
    /// hold only types that are already `pub`, so nothing forces them down.
    ///
    /// ⚠ The DXF **scale** is deliberately not in here, and that omission is
    /// the most consequential decision in [`exporting`]. A remembered scale
    /// would put yesterday's drawing's number in the box, looking exactly as
    /// right as today's -- which is the defect that window exists to prevent.
    pub export: ExportPrefs,

    pub print: PrintPrefs,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            render_quality: RenderQuality::default(),
            // O211: the engine's own default, and the shell does not
            // second-guess it. `HiddenCarriers` is the value that fixed his
            // report, and a shell that shipped a different one would be
            // answering a question the engine already answered.
            redaction_reach: RedactionReach::default(),
            // O58: the operator's own ruling, not Acrobat's. He was told about
            // the divergence and asked for a setting rather than a swap.
            paste_chords: PasteChords::default(),
            // Empty — nobody has answered yet, so every mode gets
            // `OffPagePrefs::default_for_mode`. See that function on why
            // *unanswered* is stored as absence rather than as `false`.
            off_page: OffPagePrefs::default(),
            shortcuts: ShortcutPrefs::default(),
            snapshot: SnapshotPrefs::default(),
            page_cache: PageCache::default(),
            zoom_settle_ms: DEFAULT_SETTLE_MS,
            // The shipped default is today's ceiling, so a fresh install
            // behaves exactly as the shell behaved before this existed.
            // Raising it is the operator's decision, which is the whole
            // point of the setting.
            max_zoom_percent: DEFAULT_MAX_ZOOM_PERCENT,
            // Empty, deliberately. See the field's own note: guessing a
            // system font directory would embed whatever that machine holds
            // into the operator's document, which is a licensing decision.
            font_folders: Vec::new(),
            use_os_fonts: false,
            opening_fit: OpeningFit::default(),
            wheel_paging: WheelPaging::default(),
            remote_control: RemoteControl::default(),
            // True, which is Acrobat's answer — see the field's on why the
            // default is the useful one rather than the unobtrusive one.
            shade_form_fields: true,
            // Named, not a literal: the value and the argument for it live
            // on the painter's constant, and a second copy here is a number
            // that drifts away from the sentence justifying it.
            ocr_layer_colour: crate::ocrlayerpref::DEFAULT_COLOUR,
            ocr_engine: None,
            sign_timestamp_server: None,
            // `None` — "he has not said" — so a fresh profile keeps
            // `MODES_AND_PANELS.md`'s per-mode rule. See the field.
            default_page_display: None,
            smart_select: true,
            text_chunks: true,
            find_zoom_on_jump: true,
            find_trim_query: true,
            // True = what every build has done, deliberately. See the
            // field's on why a checkbox that exists to disable something
            // defaults to enabled.
            page_previews: true,
            // No limit, and it is NOT written as a literal: `0` is the
            // sentinel, `PAGE_BUDGET_DEFAULT` is where the decision and its
            // measurements live, and `millis_from_budget` is the one place
            // that knows how the two spell each other. Three constants for
            // one default is how a default drifts from its own justification.
            page_preview_budget_ms: crate::pagebudget::millis_from_budget(
                crate::pagebudget::PAGE_BUDGET_DEFAULT,
            ),
            ribbon_auto_hide: false,
            rail_auto_hide: false,
            chrome: PageChrome::default(),
            ui_scale: DEFAULT_UI_SCALE,
            colour_icons: true,
            chosen_standard: None,
            // Empty = anonymous, deliberately. See the field's own note on
            // why the OS user name is not a defensible guess.
            author_name: String::new(),
            // Empty = "ask Windows", deliberately. See the field's own note
            // on why a cleared field must not mean "no Acrobat".
            // True: the offer may be made. See the field's note on why the
            // key is named for what it permits rather than what it suppresses.
            ask_default_app: true,
            acrobat_path: String::new(),
            // Empty = "look in the usual places", deliberately, and for the
            // reason spelled out on the field: a cleared box is how a person
            // un-sets a path, so it cannot also mean "there is no store".
            acrobat_trust_store_path: String::new(),
            // Exactly what `PrintDialog::open` hard-coded before O166, so a
            // fresh `userdata` folder opens the Print window in the state every
            // previous build of pdfcer opened it in. Asserted, not assumed —
            // see `printing::tests::the_default_is_what_the_dialog_used_to_hard_code`.
            // Exactly what the three Export windows hard-coded before O196,
            // so a fresh `userdata` folder opens each of them in the state every
            // previous build of pdfcer opened it in. Asserted per window -- see
            // `exporting::tests::the_image_default_is_what_the_dialog_used_to_hard_code`
            // and its two siblings.
            export: ExportPrefs::default(),
            print: PrintPrefs::default(),
        }
    }
}

/// Why a preference was not applied as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefNote {
    /// A key this build does not know. Left in the file, never deleted — it
    /// may belong to a newer pdfcer the operator also runs from this folder.
    UnknownKey {
        /// The key as written.
        key: String,
        /// Its 1-based line.
        line: usize,
    },
    /// A value this build could not read. That key alone falls back.
    BadValue {
        /// The key.
        key: String,
        /// What was written.
        value: String,
        /// Its 1-based line.
        line: usize,
    },
    /// A value outside the accepted range, clamped.
    Clamped {
        /// The key.
        key: String,
        /// What was written.
        value: String,
        /// Its 1-based line.
        line: usize,
    },
    /// A line that is not `name = value`. Skipped.
    Malformed {
        /// Its 1-based line.
        line: usize,
    },
}

impl Prefs {
    /// Where the preferences file lives, or `None` if there is nowhere
    /// writable.
    #[must_use]
    pub fn path() -> Option<PathBuf> {
        pdfcer_core::settings::resolve_store()
            .directory()
            .map(|dir| dir.join(PREFS_FILE))
    }

    /// Load, never failing.
    #[must_use]
    pub fn load() -> (Self, Vec<PrefNote>) {
        let Some(path) = Self::path() else {
            return (Self::default(), Vec::new());
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            // Unreadable and absent are collapsed here, unlike in the engine's
            // store, and the reason is proportion: that store holds thirteen
            // choices whose blast radius includes saved bytes, so it owes the
            // operator a distinct sentence. This holds four display
            // preferences, and the honest cost of an unreadable file is that
            // the page is drawn at the shipped sharpness.
            return (Self::default(), Vec::new());
        };
        Self::parse(&text)
    }

    /// Write, reporting failure.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::path().ok_or_else(|| {
            // ui-text-exempt: a trace/diagnostic string, never displayed.
            "no writable location for preferences".to_owned()
        })?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&path, self.write_to_string()).map_err(|e| e.to_string())
    }

    /// **Apply the opening preferences to a freshly assembled view.**
    pub fn seed_view(&self, view: &mut crate::viewer::ViewState) {
        let (fit, zoom) = self.opening_fit.to_view();
        view.fit = fit;
        view.zoom = zoom;
        view.rulers = self.chrome.rulers;
        view.grid = self.chrome.grid;
        // OR, not assign — see the table in this function's docs.
        view.guides = view.guides || self.chrome.guides;
    }
}

/// The on-disk format — the parser and the writer, which are two spellings of
/// one vocabulary and must be read together.
///
mod file;

/// The preference file's own tests — the round trips, the notes a bad value
/// produces, and the seeding of a fresh view.
///
#[cfg(test)]
mod tests;
