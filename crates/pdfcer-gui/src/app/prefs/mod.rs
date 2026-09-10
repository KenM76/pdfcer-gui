//! # `app::prefs` — the shell's own preferences, as distinct from the engine's settings
//!
//! ## ★ Why this is not `pdfcer_core::settings`
//!
//! That store has a stated purpose and this is not it. Its own window says so
//! in its first paragraph: *"The PDF standard leaves some things genuinely
//! undefined … Where that happens, pdfcer asks you rather than deciding
//! quietly."* Every one of its thirteen entries exists because a **standard
//! declines to have an opinion**, and each one states what clause is silent.
//!
//! How sharp a page is rasterised is not that. Nothing in ISO 32000-1 is silent
//! about it; it is a **preference**, a trade of sharpness against time that
//! depends on the operator's machine and on how big their drawings are. Filing
//! it beside the CMYK conversion intent would make the settings file's own
//! framing dishonest — and would put a value with no clause number in a file
//! whose every entry cites one.
//!
//! `canvas::markup::pen`'s header already named this module before it existed:
//!
//! > Persisting it belongs with the ribbon layout and the keymap, under the
//! > same `userdata/` roof and in their own file.
//!
//! ## Same roof, same shape, same fail-soft contract
//!
//! It sits in the directory `pdfcer_core::settings::resolve_store` resolves,
//! beside `settings.txt` and `layout.ron`, so the update instructions —
//! *"replace the program files, keep your `userdata` folder"* — cover it
//! without being reworded.
//!
//! The format is the engine's: flat `key = value`, `#` comments, and **per-key
//! recovery**. That last point is the one worth copying rather than a
//! convenience: an unknown key is left in the file and reported, a bad value
//! falls back for that key alone, and one bad line never discards the rest.
//! The file is meant to be hand-editable, and a parser that fails a whole
//! document over one typo punishes the operator for using it.
//!
//! ## ★ Why the RENDER settings were two and not seven
//!
//! `RIBBON_IA.md` §5.2 commissioned a View ▸ Render group of five, plus two
//! behaviour settings, and `shell::manifest`'s `DIRECTED` list carried all
//! seven as *"named individually, with their value sets and their defaults,
//! when this shell was commissioned"*. They were registered, drawn on the
//! ribbon, and inert.
//!
//! Checked against the engine on 2026-08-17, only two can be honoured:
//!
//! | commissioned | verdict |
//! |---|---|
//! | **Render quality** | ✅ [`RenderQuality`] — a raster-scale multiplier. `viewer::raster_scale` was `zoom × pixels_per_point` exactly, with no multiplier at all, so this is new capability rather than an exposed constant |
//! | **Zoom settle delay** | ✅ [`Prefs::zoom_settle_ms`] — `render::settle::ZOOM_SETTLE` was a compiled-in 150 ms |
//! | Render strategy (whole page · tiled progressive) | ❌ there is no tiled-progressive path in this shell. `pdfcer_render::render_page_region` exists, so it is buildable — but it is a rendering **architecture**, not a setting, and a radio offering it would be an affordance for a code path that does not exist |
//! | Thin lines | ⚠ **❌ then, ✅ now.** It had no such field — verified by reading `RenderOptions`' eleven public fields — and the engine shipped a twelfth on 2026-09-05 (`stroke_display`, `Pass 254.0`), the day this shell asked, because the operator asked for the control back by name (`OPERATOR_REQUESTS.md` O137). It is a **View toggle** rather than a preference — `view.line_weights`, per document, on `crate::viewer::ViewState` — for the reason the row below this table gives about what a ribbon tab picks: it is flipped several times while reading one sheet. The argument for where a persisted default would go, if he asks for one, is at `crate::text::commands::view_line_weights` |
//! | Antialiasing | ❌ `interpret.rs` sets `anti_alias: true` as a literal at two call sites and `RenderOptions` exposes no knob. (`shading.rs`'s `anti_alias` is the *document's* `/AntiAlias` key — a property of the shading pattern, not a viewer preference, and honouring it is correct.) |
//! | Floating panels (Off · Allowed) | ❌ `egui-shell`'s dock has no floating mode. Its only `floating` is `egui`'s scroll-bar style |
//! | App initiative (Never · Ask · Allowed) | ❌ **the setting has nothing to gate.** Nothing in this build opens a surface unasked — which is the specified default, *Never*, already true by construction. A control whose only value is the one already in force is a control that does nothing |
//!
//! The last row is the interesting one, and it is why this table is here
//! rather than in a commit message: `app_initiative`'s absence is not a gap.
//! It is a setting that would exist to switch off a behaviour pdfcer does not
//! have. Building it would mean building the behaviour first.
//!
//! `DIRECTED`'s own doc comment anticipated exactly this outcome — *"if it
//! turns out to be wrong, the fix is deleting eight rows from one list rather
//! than re-deriving which entries were deliberate"* — and that is what
//! happened.
//!
//! ## ★ …and then two more arrived, from the opposite direction
//!
//! [`opening`]'s two preferences — how the first page is fitted, and which
//! overlays are already on — were **not** commissioned by `RIBBON_IA.md`. They
//! came out of the `NO_SURFACE.md` sweep, which is the inventory of *every
//! tunable an operator would plausibly want to change and cannot*, and they are
//! the two rows in it that cost an operator something on **every document they
//! ever open** rather than once.
//!
//! That contrast is worth carrying, because it says where the next preference
//! will come from. The commissioned list was written from the outside, before
//! the shell existed, and five of its seven turned out to name nothing. The
//! sweep was written from the inside, by reading the constants the code
//! actually holds, and both of its candidates were real. **An inventory of what
//! the program does beats a wishlist of what it might.**
//!
//! ## The two stores are two files, and the operator never finds out
//!
//! `dialogs::settings::Draft` edits both and the window has one Save and one
//! Cancel. See its `working_prefs` field, which states the rule: *"one Cancel
//! discards both, one Save writes both, and `is_dirty` is true if either
//! moved."*

/// How big the **program's own controls** are drawn — the one accessibility
/// preference, and the only one here that changes nothing about the document.
/// ★★ How much memory pdfcer may spend so a page it has already drawn does not
/// have to be drawn again.
///
/// Its header carries the defect it exists for and the part of that defect a
/// reader would otherwise carry out wrongly: the cache pruned itself to the
/// VISIBLE SET on every frame, so the budget had never bitten, and raising the
/// number alone would have changed nothing at all.
/// Where pdfcer looks for a font it has to embed — the input `tools.embed_fonts`
/// has been waiting for, and an unrecorded dependency found by re-deriving that
/// command's blocker. See its header.
pub mod fonts;

pub mod cache;
pub mod chrome;
/// What an operator is shown when a page **first appears** — read once per
/// document open, never on the hot path.
pub mod opening;
// What a plain wheel does when the document is not one long scroll -- O30.
/// ★ Which chord means which form-field paste — O58. Its own file because
/// neither order is obviously right and the argument for each is worth keeping.
pub mod pastechords;
/// ★★ **What the Print window opens with** — `OPERATOR_REQUESTS.md` **O166**,
/// 2026-09-10. Its own file because deciding *which* of that window's twenty
/// controls may be remembered is a judgement worth keeping, and because it
/// carries this group's whole file format — parser and writer together. See
/// its header.
pub(crate) mod printing;
/// How sharply a page is drawn, and how long zoom waits before drawing it.
/// The two preferences that change what a **frame costs**.
pub mod quality;
pub mod wheel;

use std::path::PathBuf;

pub use cache::PageCache;
pub use chrome::{DEFAULT_UI_SCALE, MAX_UI_SCALE, MIN_UI_SCALE, UI_SCALE_STEP};
pub use opening::{OpeningFit, PageChrome};
pub use pastechords::PasteChords;
// `pub(crate)`, not `pub`: `PrintPrefs` carries `dialogs::print::spooler`'s own
// `pub(crate)` types, so it can be no more visible than they are. `Prefs` stays
// `pub` and holds it on a `pub(crate)` field, which is legal and is what keeps
// `private_interfaces` quiet.
pub(crate) use printing::PrintPrefs;
pub use quality::{DEFAULT_SETTLE_MS, MAX_SETTLE_MS, MIN_SETTLE_MS, RenderQuality};
pub use wheel::WheelPaging;

/// The shipped maximum zoom, as a percentage.
///
/// ★★ **The maximum, on the operator's instruction of 2026-08-22** — *"Also
/// set the default to be able to hit the maximum zoom."*
///
/// It was 800 % for one build, chosen so a fresh install behaved exactly as
/// the shell had before the setting existed. That was the cautious call and he
/// overruled it, consistently with his earlier one: *"it is up to the user to
/// determine how much of a performance hit they want to take."* A capability
/// he has to find a preferences file to switch on is a capability most of its
/// users never have.
///
/// ★ What this does NOT change is the behaviour he cares about. The ceiling is
/// permission, not policy: `viewer::zoom_ceiling` still lets the whole-page
/// raster bind wherever it can, so **panning stays instant at every zoom that
/// could render whole-page before** — the region path engages only above it,
/// where the alternative is not a slower zoom but no zoom at all.
pub const DEFAULT_MAX_ZOOM_PERCENT: f32 = MAX_MAX_ZOOM_PERCENT;

/// The lowest a maximum-zoom setting may be. Below this the operator could
/// configure a document they cannot magnify at all.
pub const MIN_MAX_ZOOM_PERCENT: f32 = 10.0;

/// The highest a maximum-zoom setting may be — **a hundred billion percent**,
/// which is the deepest zoom the page has been confirmed to actually DRAW at.
///
/// ★★ The operator named a trillion, and a trillion very nearly works: driving
/// to it renders cleanly with no failed rasters. What it does not do is put a
/// page on screen. The limit there is no longer the scroll offset — tier 3's
/// `f64` anchor fixed that — but the **strip's own extent**, which is still
/// `page × zoom` in `f32` and reaches 6×10^12 points at a trillion percent on
/// US Letter. Measured by driving: drawn at 8.6×10^9× (859 billion percent),
/// not drawn at 1×10^10×.
///
/// ★ So this is set an order of magnitude inside the confirmed-working range
/// rather than at the edge of it. Offering a rung that renders without error
/// and shows a blank page would be the same defect this feature has refused
/// throughout: a control that accepts a number and then misbehaves.
///
/// Removing this needs the strip to stop being built in `page × zoom` space at
/// deep zoom — the same move tier 3 made for the offset, one layer out.
///
/// ★ It is not a judgement about what is sensible. He was explicit that the
/// performance trade is his to make; this is about what the shell can put on
/// the screen.
pub const MAX_MAX_ZOOM_PERCENT: f32 = 1e12;

/// Format a percentage for the preferences file without an exponent or a
/// trailing `.0`.
///
/// ★ `1e12` is what `f32::to_string` produces for a trillion, and a file the
/// operator opens in a text editor should say `1000000000000`. The file is
/// his to read and edit; a machine-shaped number there is a small rudeness
/// with a real cost, because he cannot tell at a glance what he set.
fn format_percent(value: f32) -> String {
    format!("{value:.0}")
}

/// The file this store is written to, beside `settings.txt`.
// ui-text-exempt: a file name, never displayed.
pub const PREFS_FILE: &str = "preferences.txt";

/// **A stored `bool` as the shell's own auto-hide setting.**
///
/// One conversion, named, rather than an `if … { OnHover } else { Off }` at
/// each of the two call sites. The two settings are stored as `bool` because
/// `settings.txt` is a `key = true | false` file an operator edits by hand and
/// a third spelling of the same fact would be a third thing to get wrong; the
/// shell's [`egui_shell::peek::AutoHide`] is an enum because it has room to
/// grow a third position (Office has three). This function is the seam where
/// that difference is absorbed, and it is the place a third position would be
/// mapped.
#[must_use]
pub fn auto_hide(on: bool) -> egui_shell::peek::AutoHide {
    if on {
        egui_shell::peek::AutoHide::OnHover
    } else {
        egui_shell::peek::AutoHide::Off
    }
}

/// The shell's own preferences.
///
/// ## `PartialEq` but not `Eq` — and it was `Eq` until [`Self::ui_scale`] landed
///
/// A scale is a continuous quantity and `f32` has no total equality, so the
/// derive cannot be kept. Nothing is lost: the only thing that compares two
/// `Prefs` is `dialogs::settings::Draft::is_dirty`, which asks *"has the
/// operator changed anything?"* — and `PartialEq` answers that exactly. `Eq`
/// would additionally promise reflexivity, which the one field that could
/// break it (a `NaN` scale) cannot reach, because [`chrome::normalise_ui_scale`]
/// clamps every value that enters the struct.
#[derive(Debug, Clone, PartialEq)]
pub struct Prefs {
    /// How sharply a page is rasterised.
    pub render_quality: RenderQuality,
    /// ★★ **How much memory the page cache may hold**, so a page already drawn
    /// is not drawn again.
    ///
    /// Read every frame by `crate::render::settle::fill_strip`, which hands it
    /// to `StripRasters::retain` — the one place it is spent. Read live rather
    /// than at open, unlike [`Self::opening_fit`]: shrinking it must take effect
    /// at once, because an operator reaching for a smaller value is an operator
    /// whose machine is already struggling.
    pub page_cache: PageCache,
    /// How long a zoom must stop changing before it is committed to a real
    /// rasterisation, in milliseconds.
    ///
    /// Stored as a number rather than as a `Duration` because that is what the
    /// file holds and what the control edits; `render::settle` converts once,
    /// at the one place it is read.
    pub zoom_settle_ms: u64,
    /// ★★ **The highest zoom the operator wants to be able to reach**, as a
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
    /// ★ It is also the control he asked for to **compare the two rendering
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
    /// ★★ Empty by default, and the emptiness is honest rather than a gap:
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
    /// ★★★ **`false` by default, and that is the whole of the licensing
    /// argument surviving intact.** The field above explains why pdfcer must not
    /// go looking on its own; this does not overrule that, it satisfies it. The
    /// objection was never to *using* system fonts — it was to pdfcer deciding
    /// silently, and an explicit, persistent, off-by-default switch is the
    /// operator making that decision once, visibly, where they can find it
    /// again.
    ///
    /// ★ It is a **separate preference** rather than the two OS folders being
    /// appended to [`Self::font_folders`] when the box is ticked, and the
    /// difference shows up the day the machine changes: a stored *intent*
    /// ("use this computer's fonts") still means the right thing on a new
    /// machine with a different `%WINDIR%`, while two stored *paths* would name
    /// folders that no longer exist. See [`fonts::os_font_dirs`].
    pub use_os_fonts: bool,
    /// How the first page of a newly opened document is sized to the window.
    ///
    /// ★ Read **once**, by [`Self::seed_view`], in the one place a document is
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
    /// **Wash the fillable fields, so you can see what can be typed into** —
    /// `OPERATOR_REQUESTS.md` O96.
    ///
    /// Ken, 2026-09-02: *"in our display section we should have an option to
    /// shade the form fields like acrobat does."*
    ///
    /// # ★★★ Why this is not the thing rule 4 forbids, and the distinction is
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
    /// ★★ The property that keeps it honest is where it is drawn: it is painted
    /// by the canvas **overlay**, over the finished page texture, and reaches
    /// no rasterizer. It cannot appear in a print, an export, a Save, or a
    /// `render-page`. The one-line test rule 4 is judged by — *would a
    /// screenshot of the canvas differ from a screenshot of the same document
    /// saved and reopened?* — answers **yes and that is correct here**, because
    /// what differs is a control's affordance rather than pdfcer marking its own
    /// uncertainty.
    ///
    /// ★ **On by default, which is Acrobat's answer** and the useful one: an
    /// operator who does not know a form is fillable is the person this exists
    /// for, and they will not go looking for a setting to reveal it. Somebody
    /// who wants the page clean turns it off once.
    pub shade_form_fields: bool,
    /// ★★★ **How a document the program has never seen is laid out** —
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
    /// # ★★ Why `Option`, and why collapsing it would be a regression
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
    /// # ★ This overturns a written decision, and the header it overturns has
    /// been rewritten rather than left contradicting the code
    ///
    /// `crate::app::prefs::opening`'s header said a global default for page
    /// display was *"a second axis colliding with the per-document one … and
    /// is deliberately unbuilt"*. The collision is real and the answer is
    /// **precedence**, not absence: per document beats global beats per mode.
    /// Three tiers with a stated order is one axis, not two.
    pub default_page_display: Option<crate::viewer::PageDisplay>,
    /// **Whether a click selects a whole container or one line inside it** —
    /// `OPERATOR_REQUESTS.md` **O70**, 2026-08-31.
    ///
    /// The persisted half of [`crate::canvas::smart`]. The live value lives in
    /// `egui::Memory`, because the canvas reads it from places that have a
    /// context and nothing else; this is where it survives a restart.
    ///
    /// ★ **`true` by default**, which is the same argument that module makes:
    /// the checkbox exists so the behaviour can be turned OFF, and the
    /// behaviour is what every drawing program in the class does.
    pub smart_select: bool,
    /// **Is going to a search hit allowed to change the zoom?** —
    /// `OPERATOR_REQUESTS.md` **O163**, 2026-09-09, his words: *"add a
    /// checkbox option to our search bar called zoom - when I unchecked just
    /// jump to the page and highlight the found item as before but don't
    /// change the zoom."*
    ///
    /// The persisted half of [`crate::find::FindState::zoom_on_jump`]. The
    /// live value lives on the find state because the bar reads and writes it
    /// every frame and `Prefs` is not reachable from a widget; this is where
    /// it survives a restart. Written by `Action::SetFindZoom`, read back into
    /// the find state once at startup.
    ///
    /// ★ **`true` by default**, which is the behaviour every build before
    /// 2026-09-09 had: going to a hit re-applies Fit page or Fit width to
    /// whatever page the hit is on, so on a set whose sheets are different
    /// sizes the zoom changes. The checkbox exists to turn that OFF, so its
    /// default is ON — the same argument [`Self::smart_select`] makes above.
    ///
    /// ★★ **This is not a search option.** It changes nothing about which
    /// text matches, so it is deliberately not a `FindOptions` field: those
    /// re-run the search when they change, and re-running a search because a
    /// view preference moved would throw away the operator's place in the
    /// result list for no reason.
    pub find_zoom_on_jump: bool,
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
    /// ★ **Off by default.** An operator who has never heard of the feature
    /// gets the ribbon they have always had, and the two ways to it are the
    /// Settings window and `view.ribbon_autohide` on View ▸ Window. The tab
    /// strip never hides with it — see [`egui_shell::peek`] on why Office's
    /// full *Auto-hide Ribbon* is the setting people get stuck in.
    pub ribbon_auto_hide: bool,
    /// **Does the left rail hide itself until the pointer reaches its edge?** —
    /// the same instruction, *"left rail should also have the option to auto
    /// hide as well."*
    ///
    /// ★ Off by default, for [`Self::ribbon_auto_hide`]'s reason. When it is on
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
    /// # ★ A multiplier, not a size, and the distinction is the whole design
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
    /// # ★ Live-previewed, like the theme, and for the identical reason
    ///
    /// `app::frame`'s step 0 reads this from the **draft** while the settings
    /// window is open. A scale cannot be judged from a number — you choose it
    /// by seeing whether you can read the ribbon — so it is the second of the
    /// two settings in this window that take effect before Save. Cancel drops
    /// the draft and the size reverts with it; there is no separate preview
    /// state that could get out of step with what will be written.
    pub ui_scale: f32,
    /// ★★★ **Which rendering standard the operator chose**, by its engine id —
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
    /// ★ And the worse half he had not seen yet: **his choice was discarded.**
    /// Nothing recorded it, so on reopening the window `preset::matching`
    /// supplied the derived reading — *"your settings look like this one"* —
    /// which returns the FIRST of the eight. Choose PDF/X-4, come back, and the
    /// window says PDF/X-1a.
    ///
    /// # ★★ Why persisting it is not the thing the old comment refused
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
    /// ★ Retired on save when it no longer holds. If a control is changed by
    /// hand afterwards the settings stop being that standard's, and
    /// `preset::live_choice` already declines to show it —
    /// [`crate::dialogs::settings::commit`] clears the stored value to match,
    /// so the file never carries a claim the settings contradict.
    pub chosen_standard: Option<String>,
    /// ★★★ **The operator's name, written into every comment they author** —
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
    /// # ★★ Why a PREFERENCE and not a setting
    ///
    /// `Settings` is the engine's store and describes how to read and write
    /// PDFs. This describes **the person at the keyboard**. It is the same
    /// distinction [`Self::chosen_standard`] makes one field up, and it is
    /// sharper here: two operators sharing one machine's settings file would
    /// still want two names.
    ///
    /// # ★ Empty means anonymous, and that is a real choice
    ///
    /// The default is empty and an empty value writes **no `/T` at all**,
    /// which is legal and is what every reviewer UI shows as an anonymous
    /// note. It is not a placeholder to be filled with a guess: pdfcer does not
    /// know the operator's name, and reading one out of the OS user account
    /// would put a Windows login into a document that leaves the building.
    pub author_name: String,

    /// ★★★ **Where Acrobat is, when the operator has had to say** —
    /// `OPERATOR_REQUESTS.md` **O122**: *"have a setting where people can
    /// change it."*
    ///
    /// # ★★ Empty is the normal value, and it means "find it yourself"
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
    /// # ★★★ Why this exists at all, given discovery works
    ///
    /// Because discovery reads registrations, and a registration is a thing an
    /// installer writes. A portable copy, a second version kept for a client,
    /// an install on a volume Windows was never told about, a build where the
    /// registration was written with an environment variable in it that pdfcer
    /// does not expand — every one of those is an Acrobat that exists and that
    /// discovery cannot see.
    ///
    /// ★ And it is visible in Settings **whether or not discovery succeeded**,
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
    /// `ENGINE_BACKLOG.md`'s trust-store rows, 2026-09-05. The list Acrobat
    /// downloads (AATL + the EU Trusted Lists) lands in
    /// `%APPDATA%\Adobe\Acrobat\<track>\Security\addressbook.acrodata`, and
    /// `crate::trust::candidate_paths` tries four tracks in the same order
    /// `pdfcer-cli` does — so the window and the command line find the same
    /// file, which is the one support conversation nobody can win when they
    /// disagree.
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
    /// ★ **R9's escape hatch, and it is the load-bearing half.** The control
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

    /// ★★★ **What the Print window opens with** — `OPERATOR_REQUESTS.md`
    /// **O166**, his words of 2026-09-10: *"the printer dialogue box needs to
    /// remember our last settings."*
    ///
    /// Read once by `crate::dialogs::print::PrintDialog::open`, written back
    /// when a job is committed. See [`PrintPrefs`] for the rule that decides
    /// which of that window's controls are in here and which are deliberately
    /// not — the short version being that a setting is remembered only if it
    /// would still be the right answer for a **different document**.
    ///
    /// ★ `pub(crate)` where every field above it is `pub`, because it carries
    /// the print dialog's own crate-private types. That is deliberate and is
    /// argued in [`printing`]'s header: storing the real types rather than a
    /// mirrored set is what makes a new variant a compile error here instead of
    /// a silent round-trip to the default.
    pub(crate) print: PrintPrefs,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            render_quality: RenderQuality::default(),
            // O58: the operator's own ruling, not Acrobat's. He was told about
            // the divergence and asked for a setting rather than a swap.
            paste_chords: PasteChords::default(),
            page_cache: PageCache::default(),
            zoom_settle_ms: DEFAULT_SETTLE_MS,
            // ★ The shipped default is today's ceiling, so a fresh install
            // behaves exactly as the shell behaved before this existed.
            // Raising it is the operator's decision, which is the whole
            // point of the setting.
            max_zoom_percent: DEFAULT_MAX_ZOOM_PERCENT,
            // ★ Empty, deliberately. See the field's own note: guessing a
            // system font directory would embed whatever that machine holds
            // into the operator's document, which is a licensing decision.
            font_folders: Vec::new(),
            use_os_fonts: false,
            opening_fit: OpeningFit::default(),
            wheel_paging: WheelPaging::default(),
            // ★ True, which is Acrobat's answer — see the field's ★ on why the
            // default is the useful one rather than the unobtrusive one.
            shade_form_fields: true,
            // ★ `None` — "he has not said" — so a fresh profile keeps
            // `MODES_AND_PANELS.md`'s per-mode rule. See the field.
            default_page_display: None,
            smart_select: true,
            // ★ True = the behaviour every build before 2026-09-09 had,
            // deliberately. See the field's ★ on why a checkbox that exists
            // to disable something defaults to enabled.
            find_zoom_on_jump: true,
            ribbon_auto_hide: false,
            rail_auto_hide: false,
            chrome: PageChrome::default(),
            ui_scale: DEFAULT_UI_SCALE,
            chosen_standard: None,
            // ★ Empty = anonymous, deliberately. See the field's own note on
            // why the OS user name is not a defensible guess.
            author_name: String::new(),
            // ★ Empty = "ask Windows", deliberately. See the field's own note
            // on why a cleared field must not mean "no Acrobat".
            acrobat_path: String::new(),
            // ★ Empty = "look in the usual places", deliberately, and for the
            // reason spelled out on the field: a cleared box is how a person
            // un-sets a path, so it cannot also mean "there is no store".
            acrobat_trust_store_path: String::new(),
            // ★ Exactly what `PrintDialog::open` hard-coded before O166, so a
            // fresh `userdata` folder opens the Print window in the state every
            // previous build of pdfcer opened it in. Asserted, not assumed —
            // see `printing::tests::the_default_is_what_the_dialog_used_to_hard_code`.
            print: PrintPrefs::default(),
        }
    }
}

/// Why a preference was not applied as written.
///
/// The same shape as `pdfcer_core::settings::SettingNote` and for the same
/// reason: the file is hand-editable, so a mistake in it must be findable, and
/// **at its line number**. A message saying only "something was wrong" sends
/// the operator to read the whole file.
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
    ///
    /// Derived from the same `pdfcer_core::settings::resolve_store()` the
    /// settings and the layout use, so the three cannot drift apart — which is
    /// the failure this project already found once, when two callers in one
    /// process disagreed about which home was live and put two files that
    /// belong together in two places.
    #[must_use]
    pub fn path() -> Option<PathBuf> {
        pdfcer_core::settings::resolve_store()
            .directory()
            .map(|dir| dir.join(PREFS_FILE))
    }

    /// Load, never failing.
    ///
    /// A missing file, an unreadable one, a broken line or a value out of range
    /// all yield usable preferences with a reason in the returned notes. **A
    /// missing file produces no note**, deliberately: a first run is the
    /// expected state, not a fault, and reporting it would train the operator
    /// to ignore the channel that carries the real problems.
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
    ///
    /// Unlike loading, saving fails **loudly**: the operator asked for
    /// something to be remembered and is owed the truth if it was not. Same
    /// asymmetry the engine's store holds itself to.
    ///
    /// # Errors
    ///
    /// The path could not be resolved, its directory could not be created, or
    /// the write was refused. Carried as a `String` because the caller's only
    /// use for it is a trace line — the operator-facing half is a fixed
    /// sentence, for the reason `text::status::settings_not_saved` documents.
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
    ///
    /// Called once per document, from `PdfcerApp::adopt`, and from nowhere else.
    ///
    /// # ★ Why this is a method here rather than a field read in `ViewState::default`
    ///
    /// Because `ViewState::default()` cannot see the application. `OpenDoc::assemble`
    /// builds a document without a `PdfcerApp` in reach — its own comment says
    /// so — which is the same constraint that put `adopt_settings` in the open
    /// path rather than in the constructor. Seeding here keeps `ViewState`'s
    /// `Default` the **conservative** answer, which is what every test that
    /// builds one without a configuration relies on.
    ///
    /// # ★ The remembered-guides override still wins, and that is not a
    /// coincidence of ordering
    ///
    /// `OpenDoc::assemble` may already have set `view.guides = true`, because
    /// `canvas::guides::opening` turns the layer on for a document that has
    /// guides saved against it — *"the presence of the work is the
    /// preference"*. This function therefore **ORs** rather than assigns for
    /// that one field:
    ///
    /// | remembered guides | preference | result |
    /// |---|---|---|
    /// | yes | on | shown |
    /// | yes | off | **shown** — the work outranks the default |
    /// | no | on | shown, and empty until the first is placed |
    /// | no | off | hidden |
    ///
    /// Row two is the one that matters and it is the reason this is not three
    /// plain assignments. A preference is a statement about documents in
    /// general; a document that carries guides is a statement about *that*
    /// document, and the specific beats the general. Assigning would hide work
    /// the operator did, on the document they did it on, because of a switch
    /// they set weeks earlier about something else.
    ///
    /// Rulers and grid have no per-document memory at all, so they assign.
    ///
    /// # What it deliberately does not touch
    ///
    /// [`crate::viewer::ViewState::display`] — the single/continuous/facing
    /// arrangement. That has its own per-document store and its own operator
    /// requirement; see [`opening`]'s header for why a global default for it
    /// would be a second axis colliding with the one that was asked for.
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
/// Split out under **R2** on 2026-09-09 when the *Zoom* control's key took
/// this file two lines over the ceiling. See that file's own header for why
/// the seam falls between the *model* and its *transcription* rather than
/// between reading and writing.
mod file;

/// The preference file's own tests — the round trips, the notes a bad value
/// produces, and the seeding of a fresh view.
///
/// Split out under **R2** on 2026-08-28, when `author_name` took this file past
/// 1,500 lines. The seam is the one this project has used seven times now:
/// tests are the largest single block in a mature module and the one whose
/// removal leaves the subject intact.
#[cfg(test)]
mod tests;
