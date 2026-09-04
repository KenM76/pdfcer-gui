//! # `dialogs::settings` — where a spec ambiguity becomes a choice
//!
//! Ported from `D:\Dev\pdfce\crates\pdfce-gui\src\settings_panel.rs` (914
//! lines). `SALVAGE.md` classes it Class B — *"salvaged but unbuilt"* — and
//! `shell::commands::reach` recorded `file.settings` as blocked on exactly
//! this module.
//!
//! ## What this surface is for
//!
//! The standing rule behind it, an operator directive of 2026-08-08:
//!
//! > Where standards are ambiguous those should become settings that the user
//! > can choose direction one, with the initial installed default as the best
//! > guess of what is usually followed.
//!
//! `pdfcer_core::settings` is the store that makes such a choice survive a
//! restart. This module is the only place an operator can *make* one without
//! opening a text editor — and the no-affordance-without-capability rule cuts
//! both ways here: **a setting the program honours but offers nowhere is just
//! as much a gap as a control that does nothing.**
//!
//! That is the whole reason this shipped late and the reason it had to ship: the
//! engine has honoured thirteen of these since before this project began, and
//! the operator's report was that features had been added with *"no surface for
//! changing or editing the settings for them."*
//!
//! ## Why a window and not a dock panel
//!
//! The dock's own test is **"selection state is watched, workflows are
//! entered."** Watched things — the page rail, the object tree, the armed
//! tool's options — earn a permanent compartment because they are consulted
//! continuously *while doing something else*. Settings are the opposite:
//! consulted in bursts, deliberately, when there is something to change. The
//! old shell moved Properties out of the dock on 2026-08-06 for precisely this
//! reason, after the operator used it and found the pairing wrong; putting
//! Settings *in* would be the same mistake with a different noun.
//!
//! So it is a window, opened from the File tab's pdfcer group, in the shape an
//! operator arriving from Office or PDF-XChange already expects of *File →
//! Options*.
//!
//! ## ★ Seven groups, not six — and one of them is new
//!
//! Settings are collapsed into subject groups, and this is navigation rather
//! than tidiness: **an operator opens this window with a *symptom*** — "my
//! black lines look grey", "copied text has no spaces", "my dimension came out
//! as an angle" — and the group headings are how a symptom finds its setting. A
//! flat list of thirteen makes the reader scan every one.
//!
//! Which means a setting filed under the wrong heading is not untidy, it is
//! **unreachable**. The source had one: `parallel_epsilon_degrees`, which
//! governs whether two lines are dimensioned as a distance or an angle, sat
//! under *Copying and extracting text*, where it has nothing to do with either.
//! It was there because it happened to be a slider like the word-gap one beside
//! it. It now has its own group, [`measuring`].
//!
//! | # | Group | Settings |
//! |---|---|---|
//! | 1 | Appearance | theme |
//! | 2 | **Colour** *(open)* | CMYK intent · CMYK JPEG polarity |
//! | 3 | Images and transparency | mask resampling · minification |
//! | 4 | Copying and extracting text | word gap · unmappable codes · replacement text |
//! | 5 | **Measuring and dimensioning** *(new)* | parallel tolerance |
//! | 6 | Pages and printing | separations · missing appearance state |
//! | 7 | Saving files | index line endings · trailing newline |
//!
//! ### Which group starts expanded — a contradiction in the source, resolved
//!
//! `settings_panel.rs`'s header states, with a reason:
//!
//! > The **Colour** group starts expanded because it holds the setting most
//! > likely to have brought someone here — and the only one whose default
//! > knowingly differs from other PDF viewers.
//!
//! Its code opens **Appearance**. Appearance was added later and took the
//! expanded slot without the prose moving. The stated reasoning is still
//! right — Colour holds the knowingly-divergent default and answers the "my
//! black lines look grey" symptom — so **Colour is expanded here** and the
//! prose and the code now agree.
//!
//! ## The three obligations, enforced by a function signature
//!
//! A settings screen that listed keys and radio buttons would satisfy nobody
//! here. Three things must be visible that a conventional one omits:
//!
//! 1. **What the default rests on.** Most of these defaults are *reasoned
//!    inference* — a guess — and a guess must say it is a guess. Exactly one
//!    is well-sourced and says that too.
//! 2. **That a choice was made at all.** These are settings *because the
//!    standard declines to have an opinion*. An operator who does not know
//!    that reads a difference between pdfcer and Acrobat as a pdfcer bug.
//! 3. **Which way costs what.** A setting whose blast radius is the SAVED
//!    BYTES is a different kind of decision from one that only changes the
//!    preview.
//!
//! Obligations 2 and 3 are not left to discipline: [`widgets::header`] takes
//! `title`, `silence` and `radius` as **required arguments**, so a setting
//! cannot be added without answering all three. Obligation 1 lives in the
//! option notes and is pinned by tests in [`crate::text::settings`].
//!
//! ## Cancel is real
//!
//! The window edits a **working copy**. Nothing reaches the live configuration
//! or the disk until *Save*, and *Cancel* discards the lot. This is not
//! ceremony: four of the thirteen change **saved bytes**, so a radio click that
//! took effect immediately would be an edit the operator never intended and
//! cannot see.
//!
//! **Theme is the single exception**, and it is deliberate — see [`Draft`].

/// ★★★ O122 — where Acrobat is. The one group in this window that is about
/// **another program on this machine**, and the only one whose whole purpose
/// is to be reachable while the control it governs is absent from the ribbon.
mod acrobat;
pub mod appearance;
pub mod colour;
mod comments;
mod preset;

/// ★ The eighth group, and the only one not about the PDF standard: how pdfcer
/// draws, as distinct from what it draws. Two settings out of seven that were
/// commissioned — its header says which five had nothing behind them.
pub mod display;
/// The Fonts group — where pdfcer may take a font from when it has to embed
/// one. See its header for why the list lives here rather than on the batch
/// pane its blocker named.
mod fonts;
pub mod images;
pub mod measuring;
pub mod pages;
pub mod saving;
pub mod text;
pub mod widgets;

use egui::RichText;
use pdfcer_core::settings::{Settings, StoreLocation};

use crate::text::settings as t;

/// The region the window publishes for its whole body.
///
/// ★ **This is what makes `ui-verify`'s `settings_headings_legible` a LIVE
/// check.** That check is the regression test for `DEFECTS.md` D2 — every
/// collapsible heading in the old shell's Settings dialog rendering near-white
/// on light grey, at around **1.1:1** against a 3:1 floor — and it has been
/// running in offline mode against a dated screenshot for the whole project,
/// because the sentence in its own header was true: *"the new application has
/// no Settings dialog."*
///
/// It does now, and a check that measures the running program is worth more
/// than one that measures a photograph of a program that no longer exists.
/// Renaming this constant un-aims that check.
pub const REGION_BODY: &str = "dialog:settings"; // ui-text-exempt: trace region name, never displayed

/// The Cancel button's rect, so a driven check can press the abort path itself
/// rather than pressing Escape and assuming the two agree.
///
/// ★ The two DO agree — `app::settings_window` documents both as dropping the
/// draft — but a check that can only reach one of them proves nothing about the
/// other, and the whole point of the check that wanted this is that the
/// coupling it guards fails **silently**.
pub const REGION_CANCEL: &str = "dialog:settings.cancel"; // ui-text-exempt: trace region name, never displayed

/// The region each group heading publishes, suffixed with the group's key.
///
/// One per collapsible header, so the contrast check can measure **each**
/// heading against its own background rather than sampling the window and
/// hoping. D2's defect was a foreground/background *pairing*, and a pairing
/// only exists once something is drawn — so the check needs the rectangle the
/// application actually laid the text into, not a rectangle derived from a
/// palette.
pub const REGION_HEADING_PREFIX: &str = "settings.heading."; // ui-text-exempt: trace region name, never displayed

/// The region each theme radio publishes, suffixed with the preset's key.
///
/// Exists so a check can **click** one. Proving that a theme picker is on
/// screen is not the property anybody cares about; the property is that
/// choosing Dark makes the window dark, which needs a rect to aim at and a
/// capture afterwards. See `DEFECTS.md` D10.
pub const REGION_THEME_PREFIX: &str = "settings.theme."; // ui-text-exempt: trace region name, never displayed

/// How far the working copy has drifted from what the window opened on.
///
/// # ★ Theme is the one setting that breaks the draft contract, on purpose
///
/// Every other setting here is draft-until-Save. A theme cannot be judged from
/// a radio label — you choose it by *seeing* it — so the selection takes effect
/// on the next frame. The draft still governs what is **saved**; it just no
/// longer governs what is **shown**.
///
/// The mechanism is one line in the application's per-frame `ui()`, before any
/// widget is built: the theme token is read from the draft when a draft exists
/// and from the live settings otherwise. Cancel drops the draft, so the look
/// reverts with it — no separate undo path, and nothing that can get out of
/// step. The window says so in the theme setting's own radius line rather than
/// leaving it to be discovered.
#[derive(Debug, Clone, PartialEq)]
pub struct Draft {
    /// ★★★ **Which group to open and scroll to, once, on the first frame.**
    ///
    /// `None` for File ▸ pdfcer ▸ Settings, which is the operator saying *"show
    /// me the settings"* and has no one group in mind. `Some` for a command
    /// that routes here **because of** one group — today that is Tools ▸ Font
    /// folders, and the reason it needed this is `OPERATOR_REQUESTS.md` **O50**.
    ///
    /// He asked for a font-folder setting that had shipped the day before. The
    /// interesting half of that is not that he missed a row: it is that the
    /// route built for exactly his question — a command called *Font folders* —
    /// dropped him at the top of ten **collapsed** headings and left the
    /// finding to him. ⇒ **A route that exists because of one setting must land
    /// on that setting.** Opening the right window is not the same as answering
    /// the question the command's own name asks.
    ///
    /// ★ Taken by [`Self::take_focus`] rather than read, so it fires once. A
    /// group forced open on every frame is a group the operator cannot collapse.
    pub focus: Option<&'static str>,
    /// The edits in progress.
    pub working: Settings,
    /// What the settings were when the window opened.
    pub original: Settings,
    /// The shell's own preferences, in progress.
    ///
    /// ★ **A second pair, not a second draft.** The window edits two stores —
    /// `pdfcer_core::settings` and `crate::app::prefs` — and the operator must
    /// not be able to tell: one Cancel discards both, one Save writes both, and
    /// `is_dirty` is true if *either* moved. A separate draft per store would
    /// give the window two Save buttons or one that lied.
    ///
    /// They are two stores because they answer different questions — see
    /// `crate::app::prefs`' header — and that is an implementation fact the
    /// operator has no business meeting.
    /// ★★★ **Which preset the operator CHOSE**, as distinct from which one
    /// their settings happen to match.
    ///
    /// Added 2026-08-26, and it fixes a defect the operator reported in exactly
    /// the right words: *"in the settings for the standards compatibility I can
    /// only select (ISO15930-1, -4)"*.
    ///
    /// The control used to derive its selection from the **values** —
    /// `preset::matching` returns the first choice whose settings equal the
    /// current ones. Measured on 2026-08-26: **all eight of the PDF/X and PDF/A
    /// presets apply byte-identical render settings**, differing only in PDF/UA
    /// which leaves `image_minify` alone. So clicking PDF/X-4 applied PDF/X-4's
    /// answers, `matching` then found PDF/X-1a first, and the dot jumped back
    /// to the top of the list. Every one of the eight was unselectable and the
    /// program looked broken while behaving exactly as written.
    ///
    /// ★★ A choice is not a value, and a control that derives one from the
    /// other can only ever show as many states as there are distinct values.
    /// That is fine while the presets differ and silently wrong the moment two
    /// agree — which is not a rare accident here but the **normal** case, since
    /// the standards genuinely make the same demands of a renderer.
    ///
    /// ★★★ **It IS persisted, as of 2026-08-26** — in
    /// [`crate::app::prefs::Prefs::chosen_standard`], whose docs carry the
    /// operator report and the full argument. This field is the in-window
    /// working copy of it.
    ///
    /// The comment that stood here said it was *deliberately not* persisted,
    /// because the derived reading *"cannot claim an intent nobody expressed in
    /// this sitting."* That reasoning is sound about not **inventing** an
    /// intent and was being applied to the opposite case. Its two consequences,
    /// both reported by the operator or found while fixing what he reported:
    ///
    /// * **Save was greyed out** after choosing a standard, because all eight
    ///   PDF/X and PDF/A presets apply byte-identical render settings
    ///   (`preset::identical_siblings` measures it), so a second choice moved
    ///   no value and [`Self::is_dirty`] was correctly false about a draft that
    ///   really did equal what was saved.
    /// * **The choice was discarded**, so reopening showed whichever standard
    ///   `preset::matching` found first — choose PDF/X-4, come back, read
    ///   PDF/X-1a.
    ///
    /// Persisting it makes a choice a change, which makes Save live, which is
    /// what the operator expected; and it makes the window able to tell him
    /// next week what he asked for.
    pub chosen_preset: Option<&'static str>,
    pub working_prefs: crate::app::prefs::Prefs,
    /// What the preferences were when the window opened.
    pub original_prefs: crate::app::prefs::Prefs,
}

impl Draft {
    /// Start editing from the **live** configuration.
    ///
    /// Live rather than a re-read of the file, and the difference matters in
    /// exactly one case: if a previous save failed, the session is honouring a
    /// choice the disk does not have. The window must show what pdfcer is
    /// actually doing, not what it wished it had written.
    #[must_use]
    pub fn new(current: &Settings, prefs: &crate::app::prefs::Prefs) -> Self {
        Self::focused_on(current, prefs, None)
    }

    /// [`Self::new`], opened at one group. See [`Self::focus`].
    #[must_use]
    pub fn focused_on(
        current: &Settings,
        prefs: &crate::app::prefs::Prefs,
        focus: Option<&'static str>,
    ) -> Self {
        Self {
            focus,
            working: current.clone(),
            original: current.clone(),
            // ★ Seeded from the operator's PERSISTED choice, so the window
            // opens saying what they asked for rather than what their values
            // happen to resemble. `preset::live_choice` still filters it
            // through `still_holds`, so a stored id whose settings have since
            // been changed by hand is not shown — the fallback reading takes
            // over exactly where the claim stops being true.
            //
            // Resolved to the `&'static str` the choice list owns rather than
            // kept as the stored `String`: an id this build does not know is
            // not a choice it can offer, and dropping it here means every later
            // reader is comparing against the real list instead of a string
            // that might match nothing.
            chosen_preset: crate::dialogs::settings::preset::resolve_id(
                prefs.chosen_standard.as_deref(),
            ),
            working_prefs: prefs.clone(),
            original_prefs: prefs.clone(),
        }
    }

    /// Whether anything has actually changed since the window opened.
    ///
    /// Drives whether *Save* is offered at all. A Save button that is always
    /// live cannot tell the operator whether they have unsaved changes, and
    /// this is a window someone may open just to read.
    ///
    /// **Not latched.** Click a radio and click it back, and the draft is clean
    /// again — because it is, and a dirty flag that only ever went one way
    /// would make Save mean "you visited this window".
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.working != self.original || self.working_prefs != self.original_prefs
    }

    /// Whether every value is still pdfcer's own answer.
    ///
    /// # Why this is not the same question as [`Self::is_dirty`]
    ///
    /// A draft opened from non-default settings is **clean but not
    /// all-default**: loading is not editing. Collapsing the two predicates
    /// would disable *Restore defaults* for exactly the operator who most needs
    /// it — the one who changed something in a previous session and wants it
    /// back.
    #[must_use]
    pub fn is_all_default(&self) -> bool {
        self.working == Settings::default()
            && self.working_prefs == crate::app::prefs::Prefs::default()
    }
}

/// What the window is asking the application to do.
///
/// Returned rather than performed. This module renders and does not own
/// application state — the split every other dialog and panel in this shell
/// uses — and the three verbs have consequences (adopting a configuration,
/// writing a file, invalidating every cached raster) that belong in the
/// dispatcher where they can be seen together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing was pressed this frame.
    Idle,
    /// Adopt the working copy and write it.
    Save,
    /// Discard the working copy.
    Cancel,
    /// Replace the working copy with the defaults.
    ///
    /// **Does not save.** The operator still has to confirm, and still has
    /// Cancel. "Restore defaults" is not the kind of button that should be able
    /// to discard a configuration in one click with no way back.
    RestoreDefaults,
}

/// Draw one frame of the window. Returns what the operator asked for.
///
/// # Geometry, and why every number has a reason
///
/// | aspect | value | why |
/// |---|---|---|
/// | screen source | `content_rect` | not `viewport_rect`: it subtracts safe-area insets, so centring uses *usable* space |
/// | width | `620` clamped to `[420, screen − 40]` | wide enough for a full sentence at the body's text size |
/// | height | `82 %` of screen, clamped `[420, 900]` | a fixed 620 left half the height unused *while still scrolling* — the worst combination |
/// | position | `default_pos`, centred, `−20` vertically | see below |
///
/// **`default_pos` rather than `anchor`**, so the operator can drag it aside.
/// A window pinned in the middle of the screen is a window in the way of the
/// document it is about. And egui's own default position put it top-left, over
/// the quick-access toolbar and the ribbon tabs — so *opening Settings hid the
/// control that opened it*. The `−20` keeps the button row on a short screen.
///
/// # The scroll area's height is computed, not fixed
///
/// `available − 96`, floored at 180. The source used a fixed 460 and it clipped
/// the last group's heading in half, so *"Saving files"* read as a rendering
/// fault rather than as a group. The reserved 96 is the intro, the store line,
/// two separators and the button row — everything that is not the list.
pub fn show(
    ctx: &egui::Context,
    draft: &mut Draft,
    store: &StoreLocation,
    open: &mut bool,
    acrobat_viewer: Option<&crate::acrobat::Viewer>,
) -> Outcome {
    // ★★ TAKEN, not read. See `Draft::focus`: the group is forced open and
    // scrolled to on the first frame and left alone afterwards, so the operator
    // can collapse it again. A focus that survived the frame would be a heading
    // that reopened itself every time they closed it.
    let focus = draft.focus.take();
    let screen = ctx.input(egui::InputState::content_rect);
    let width = 620.0_f32.min(screen.width() - 40.0).max(420.0);
    let height = (screen.height() * 0.82).clamp(420.0, 900.0);
    let pos = egui::pos2(
        ((screen.width() - width).max(0.0) / 2.0).max(0.0),
        (((screen.height() - height).max(0.0) / 2.0) - 20.0).max(0.0),
    );

    // ★ ITS OWN OS WINDOW as of 2026-08-21. Settings is the tallest dialog in
    // the program — the height above is 82 % of the screen — so it was the one
    // most obviously squeezed by living inside the application frame.
    //
    // ★ `open` stays an `&mut bool` here, unlike the other twelve, because this
    // is a free function whose caller owns the flag. `frame.closed` is written
    // into it rather than returned.
    let _ = pos;
    let mut outcome = Outcome::Idle;
    let (frame, ()) = crate::dialogs::host::Host::new(
        "settings", // ui-text-exempt: a viewport key, never displayed.
        t::window_title(),
        egui::vec2(width, height),
        egui::vec2(420.0, 420.0),
    )
    .show(ctx, |ui| {
        crate::diag::ui_rect(REGION_BODY, ui.max_rect());
        ui.label(t::intro());
        ui.add_space(4.0);
        // Always shown, never a control. An operator who does not know
        // which of the two homes is live cannot follow the update
        // instructions, and those instructions are the one place a wrong
        // guess costs them their configuration.
        ui.label(RichText::new(t::store_location(store)).small().weak());
        ui.separator();

        let reserved = 96.0;
        let available = (ui.available_height() - reserved).max(180.0);
        egui::ScrollArea::vertical()
            .max_height(available)
            .show(ui, |ui| {
                // The order below is the operator-facing order and is the
                // contract. It is deliberately NOT the order the fields
                // appear in `Settings`, nor the order they are written to
                // the file — three orders that the source let drift apart,
                // which is how `theme` ended up emitted between the two
                // image settings and splitting them.
                // ★ The presets row, above every group because it sets all of
                // them. Operator request O38: *"a preset setting for rendering
                // things to what the [conformance] page needs"* — and, in the
                // same message, the plainer need underneath it: he had changed
                // several settings while investigating and wanted a way back.
                // See `preset`'s header for why PDF/X-4 is not among the
                // entries yet and why its absence is the design rather than a
                // gap.
                //
                // ★★★ **IN A COLLAPSIBLE GROUP, AND CLOSED — 2026-08-26.** It
                // was a bare row until then, and it had grown to **ten radios
                // plus a detail block: about 730 points**, in a scroll area
                // roughly 620 points tall. The consequence was not subtle and
                // was not caught by any test:
                //
                // > **Opening Settings showed nothing but a list of standards.
                // > Every actual setting, and every group heading, was below
                // > the fold.**
                //
                // Found by `ui-verify settings_theme_takes_effect` reporting
                // *"the Settings window is open but declared no
                // `settings.heading.appearance`. Headings declared: none"* —
                // which was **exactly right** and had been read as a tracing
                // bug. `widgets::group` publishes through `ui_rect_visible`,
                // which correctly declines to publish a heading nobody can see;
                // the check was reporting the real defect and the region
                // mechanism was working perfectly.
                //
                // ★ Arithmetic, so it is checkable: the dialog is 82 % of
                // screen height clamped to 900, less ~104 for the intro and the
                // store line and less the 96 reserved for the buttons. On a
                // 1440-point screen that is ~700 available against ~730 of
                // presets — so this was true on the operator's own display too,
                // not only in the harness's smaller offscreen viewport.
                //
                // Closed by default, like every group except Colour. It is
                // still FIRST, which is the part of the original decision that
                // was about meaning rather than about space: a preset sets all
                // the groups, so it belongs above them.
                //
                // ★★ It is a `widgets::group` rather than a `ComboBox`, and
                // that is deliberate restraint rather than the better answer. A
                // dropdown is what most applications use for a preset and it
                // would cost one row instead of one click — but the radio list
                // is what the operator has just been given, reported on and
                // accepted, and swapping the control would be improvising an
                // interaction change while fixing a layout defect. It is worth
                // proposing separately; it is not worth bundling here.
                widgets::group(ui, "presets", t::preset_title(), false, |ui| {
                    preset::row(ui, draft);
                });
                ui.add_space(10.0);
                ui.separator();
                widgets::group(ui, "appearance", t::group_appearance(), false, |ui| {
                    appearance::theme(ui, draft);
                    // ★ The group's second member, and the two belong
                    // together: they are the only settings in this window
                    // that change the PROGRAM's appearance rather than the
                    // document's, and the only two that take effect before
                    // Save. Grouping them makes that exception legible in
                    // one place instead of scattered across the window.
                    ui.add_space(10.0);
                    appearance::ui_scale(ui, &mut draft.working_prefs);
                });
                // ★ Colour is the expanded one. See the module header for
                // the contradiction in the source this resolves.
                widgets::group(ui, "colour", t::group_colour(), true, |ui| {
                    colour::intent(ui, draft);
                    ui.add_space(10.0);
                    colour::polarity(ui, draft);
                    ui.add_space(10.0);
                    colour::page_blend_space(ui, draft);
                    ui.add_space(10.0);
                    // ★ Immediately under `page_blend_space`, because it is
                    // meaningless above it: that setting decides whether a page
                    // is composited in ink, and this one decides which source
                    // colours the ink rules then reach. Reversed, the window
                    // would ask about grey-over-spot before saying whether
                    // overprint is simulated at all.
                    colour::zero_tint(ui, draft);
                    // ★ Immediately after its sibling: that control is what
                    // OVERPRINTS a spot colour, this one is what a spot colour
                    // IS. Both are reached by the same symptom.
                    colour::spot_model(ui, draft);
                    ui.add_space(10.0);
                    // ★ Directly after `page_blend_space`, and the order is an
                    // argument rather than a preference: that setting decides
                    // WHETHER a page is blended in ink, and this one decides how
                    // far up the zoom range it stays that way. An operator who
                    // has just read the first has the context for the second,
                    // and the reverse order would present a ceiling before the
                    // thing it is a ceiling on.
                    colour::cmyk_ceiling(ui, draft);
                    ui.add_space(10.0);
                    colour::mesh_patch_padding(ui, draft);
                });
                // ★ Between Appearance/Colour and Images, and the placement is
                // the window's own ordering rule rather than a gap-filling:
                // the groups run from what the PROGRAM looks like, through what
                // the DOCUMENT is made of, to what pdfcer does with it. Fonts is
                // a thing documents are made of, and it sits with Images and
                // Text rather than with the rendering knobs.
                // ★★★ The one group a command routes here FOR. See
                // `Draft::focus`: Tools ▸ Font folders opens this window
                // because the folders live in it, and until 2026-08-28 it left
                // the operator to find a collapsed heading among ten.
                widgets::group_focused(
                    ui,
                    "fonts", // ui-text-exempt: a group key, never displayed.
                    t::group_fonts(),
                    false,
                    focus == Some("fonts"), // ui-text-exempt: a group key.
                    |ui| {
                        fonts::folders(ui, &mut draft.working_prefs);
                        ui.add_space(10.0);
                        ui.separator();
                        // ★ Under the folder list, and after a separator,
                        // because the two are about different halves of the
                        // same subject: the list is where pdfcer LOOKS for a
                        // face, and this is what it does when looking has
                        // failed. Drawn in the order an operator meets them.
                        fonts::style_policy(ui, draft);
                    },
                );
                widgets::group(ui, "images", t::group_images(), false, |ui| {
                    images::mask_resample(ui, draft);
                    ui.add_space(10.0);
                    images::minify(ui, draft);
                });
                widgets::group(ui, "text", t::group_text(), false, |ui| {
                    text::word_gap(ui, draft);
                    // The source omitted this one space, so the word-gap
                    // slider and the setting under it ran together. Every
                    // other adjacent pair in every group has it.
                    ui.add_space(10.0);
                    text::unmappable(ui, draft);
                    ui.add_space(10.0);
                    text::actual_text(ui, draft);
                });
                widgets::group(ui, "measuring", t::group_measuring(), false, |ui| {
                    measuring::parallel(ui, draft);
                });
                // ★ Beside Measuring, the other group about AUTHORING rather
                // than about reading, and before Pages. `comments`' own header
                // carries why a preference is filed among the document groups
                // instead of with the other preferences at the end.
                widgets::group(ui, "comments", t::group_comments(), false, |ui| {
                    comments::author_name(ui, &mut draft.working_prefs);
                });
                widgets::group(ui, "pages", t::group_pages(), false, |ui| {
                    pages::separations(ui, draft);
                    ui.add_space(10.0);
                    pages::missing_as(ui, draft);
                });
                // ★ The shell's own preferences, LAST — after the twelve
                // that are about the document and before nothing. They are
                // the only group here whose values live in a different
                // file, and putting them at the end keeps the window
                // reading as "everything about your documents, then
                // everything about the program".
                widgets::group(ui, "display", t::group_display(), false, |ui| {
                    display::render_quality(ui, &mut draft.working_prefs);
                    ui.add_space(10.0);
                    display::zoom_settle(ui, &mut draft.working_prefs);
                    // ★ Third, after the two an operator adjusts while
                    // looking at a page and before the two that apply to
                    // the NEXT document. See `display::page_cache`.
                    ui.add_space(10.0);
                    display::page_cache(ui, &mut draft.working_prefs);
                    // ★ The two "when a document opens" settings come after
                    // the two "how a frame is drawn" ones, and the order is
                    // the group's argument rather than the order they were
                    // built in. A reader scanning the group meets the
                    // settings that affect what they are looking at now,
                    // then the ones that affect the next thing they open —
                    // and the second pair's radius lines both say so, so a
                    // reader who stops after the first two has not been
                    // misled about what the ones below do.
                    ui.add_space(10.0);
                    display::opening_fit(ui, &mut draft.working_prefs);
                    display::wheel_paging(ui, &mut draft.working_prefs);
                    display::paste_chords(ui, &mut draft.working_prefs);
                    ui.add_space(10.0);
                    display::field_shade(ui, &mut draft.working_prefs);
                    display::page_chrome(ui, &mut draft.working_prefs);
                });
                // ★★★ WHERE ACROBAT IS — O122, and LAST of all.
                //
                // Every group above changes something about a PDF; this one
                // changes nothing at all except which program a single button
                // starts, so it is the setting furthest from the document.
                //
                // ★ It is drawn whether or not an Acrobat was found, which is
                // the load-bearing half of O122's escape hatch: the ribbon
                // control is ABSENT on a machine where discovery failed, so
                // this group is the only place a person in that position can
                // be told the feature exists. `acrobat`'s own header carries
                // the argument.
                widgets::group(
                    ui,
                    "acrobat",
                    &crate::text::acrobat::group_acrobat(),
                    false,
                    |ui| {
                        acrobat::path(ui, &mut draft.working_prefs, acrobat_viewer);
                    },
                );
                widgets::group(ui, "saving", t::group_saving(), false, |ui| {
                    saving::xref_entry_eol(ui, draft);
                    ui.add_space(10.0);
                    saving::trailing_eol(ui, draft);
                    // ★ Third, and last, because it is the one an operator
                    // is least likely to have come for. The two above are
                    // about every file pdfcer writes; this one is about
                    // files that carry text markup, which not every
                    // document does.
                    ui.add_space(10.0);
                    saving::quad_point_order(ui, draft);
                });
            });

        ui.separator();
        ui.horizontal(|ui| {
            let dirty = draft.is_dirty();
            let save = ui.add_enabled(dirty, egui::Button::new(t::save()));
            if save.clicked() {
                // ★★ Retire a stored choice the settings no longer express,
                // **before** it is written.
                //
                // `preset::live_choice` already declines to SHOW a chosen
                // standard once a control has been changed by hand — that is
                // `still_holds`, asked against the preset rather than
                // remembered as a flag. Without this line the window would stop
                // claiming it and the file would go on carrying it, so the next
                // session would open showing a standard the values contradict.
                //
                // The display rule and the stored value have to retire
                // together, and this is the one place a stored value is
                // committed.
                if !preset::still_chosen(draft) {
                    draft.chosen_preset = None;
                    draft.working_prefs.chosen_standard = None;
                }
                outcome = Outcome::Save;
            }
            if !dirty {
                save.on_disabled_hover_text(t::save_disabled_tooltip());
            }
            // ★★★ **Cancel publishes its own rect, 2026-09-04, and the reason
            // is a check that could not be written without it.**
            //
            // `settings_theme_takes_effect` drives a live theme change and then
            // proves it is put back — the one-line coupling whose failure is
            // silent, and the behaviour the outside review praised while noting
            // nothing guarded it. To press Cancel, a driven check has to know
            // where Cancel is.
            //
            // Without this it could only press **Escape**, which reaches the
            // same `settings_draft = None` and is documented as contractually
            // identical — but "identical today" is not the property under test.
            // An edit that made `Outcome::Cancel` behave differently from the
            // Escape path would leave the check green.
            //
            // ★ And it could not simply aim at the button beside it: that is
            // **Save**, which writes the operator's real `settings.txt`. A
            // harness that mis-aims by one control does not fail — it silently
            // rewrites his preferences, which is the class of harness accident
            // this project has already paid for once.
            //
            // ★★ `ui_rect`, not `ui_rect_visible`: this row is in the dialog's
            // fixed footer, outside the body's scroll area, so it is never
            // partly clipped. The visible-fraction filter exists for content
            // that can scroll out from under the pointer and would report a
            // sliver a driven click cannot hit.
            let cancel = ui.button(t::cancel()).on_hover_text(t::cancel_tooltip());
            crate::diag::ui_rect(REGION_CANCEL, cancel.rect);
            if cancel.clicked() {
                outcome = Outcome::Cancel;
            }
            // Separated from the two commit/abort controls, because it is
            // the destructive one and a mis-click on it costs the operator
            // every choice they have ever made here.
            ui.add_space(12.0);
            let all_default = draft.is_all_default();
            let restore = ui.add_enabled(!all_default, egui::Button::new(t::restore_defaults()));
            if restore.clicked() {
                outcome = Outcome::RestoreDefaults;
            }
            if all_default {
                restore.on_disabled_hover_text(t::restore_defaults_disabled_tooltip());
            } else {
                restore.on_hover_text(t::restore_defaults_tooltip());
            }
        });
    });

    // The caller's flag, written rather than returned. See the note above the
    // host for why this dialog differs from the other twelve.
    if frame.closed {
        *open = false;
    }
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::pageops::SeparationPolicy;
    use pdfcer_core::settings::CmykIntent;

    /// ★★ **Every setting `pdfcer-core` carries has a control in this window.**
    ///
    /// # What this replaces, and why it had to be rebuilt rather than moved
    ///
    /// The old shell carried
    /// `settings_panel::tests::every_setting_the_store_carries_can_be_reached_from_this_window`,
    /// which asserted exactly this. It fired on 2026-08-19 on a real gap — the
    /// engine added `Settings::quad_point_order` and the window had no control
    /// for it — and the `pdfcer` session's note of that day warned that **the
    /// gate disappears when the old crate is deleted**, taking the property with
    /// it:
    ///
    /// > *"`NO_SURFACE.md` is a kept inventory; this is a build failure. They
    /// > are not substitutes — one tells you what is missing when someone looks,
    /// > the other refuses to compile when someone adds it."*
    ///
    /// It could not be copied across. The old test read the store's **source**
    /// with `include_str!("../../pdfcer-core/src/settings/mod.rs")` — a relative
    /// path that only exists because both crates lived in one repository. In
    /// this project `pdfcer-core` is a **git dependency** and its source is not
    /// on any path this crate can name. So the enumeration had to come from
    /// somewhere else.
    ///
    /// # Where the list of settings comes from now, and why it is better
    ///
    /// From [`Settings::write_to_string`] — the engine's own settings-file
    /// writer, at **runtime**, against the shipped default. Every setting the
    /// store round-trips appears there as a `key = value` line, because that is
    /// what the file is; a setting missing from it could not be persisted at
    /// all, which is a different and larger defect the engine's own tests own.
    ///
    /// That is a stronger instrument than parsing a struct definition, and the
    /// difference is not cosmetic:
    ///
    /// * it reads the **compiled dependency**, so it is answering about the
    ///   engine this build actually links, not about a file on disk that may be
    ///   from a different revision;
    /// * it cannot be fooled by a field that is `pub` and not persisted, or by
    ///   one persisted under a key that differs from its field name;
    /// * it needs no path into another repository, which is what makes it
    ///   survive the fold-in in either direction.
    ///
    /// # The crude half, kept crude deliberately
    ///
    /// Coverage is asserted by reading **this directory's own source** and
    /// looking for `working.<key>`. That is a text search and it can be
    /// defeated — by a control that names the field in a comment and never
    /// binds it, say. It is kept anyway, because the alternative is a
    /// hand-maintained list of which settings have controls, and a
    /// hand-maintained list is exactly the thing that goes stale silently. A
    /// crude check that fails when a setting is added beats an exact one that
    /// nobody updates.
    ///
    /// **When this fails, the fix is a control, not an edit to this test.** The
    /// failure message says which setting, and the group it belongs in is
    /// decided by the symptom that brings an operator looking for it — see this
    /// module's header.
    /// Every file in this directory, paired with its module name.
    ///
    /// ★★★ HAND-WRITTEN, AND THEREFORE AUDITED — see
    /// [`every_source_in_this_directory_is_listed`].
    ///
    /// `fonts.rs` was missing from here on 2026-08-30 and the consequence was
    /// precise and misleading: the completeness test below reported
    /// `Settings::style_policy` as having no control **after the control had
    /// been written**, because the control was in the one file the search did
    /// not read. A check that cannot see a file reports its contents as absent,
    /// which is indistinguishable from the defect it exists to find.
    const SOURCES: &[(&str, &str)] = &[
        ("mod", include_str!("mod.rs")),
        ("appearance", include_str!("appearance.rs")),
        ("colour", include_str!("colour.rs")),
        ("acrobat", include_str!("acrobat.rs")),
        ("comments", include_str!("comments.rs")),
        ("display", include_str!("display.rs")),
        ("fonts", include_str!("fonts.rs")),
        ("images", include_str!("images.rs")),
        ("measuring", include_str!("measuring.rs")),
        ("pages", include_str!("pages.rs")),
        ("preset", include_str!("preset.rs")),
        ("saving", include_str!("saving.rs")),
        ("text", include_str!("text.rs")),
        ("widgets", include_str!("widgets.rs")),
    ];

    /// ★★★ EVERY MODULE THIS DIRECTORY DECLARES IS IN [`SOURCES`].
    ///
    /// Unlike the catalog's own version of this guard, there is **no
    /// exclusion list**: this search is for *"is there a control bound to this
    /// setting anywhere in the window"*, and a control could legitimately be
    /// written in any file here — including `widgets.rs`, if a helper ever
    /// bound a field directly. Every file counts, so every file is listed.
    ///
    /// The failure this prevents is the quiet one. A missing file does not
    /// make the completeness test fail loudly; it makes it **report the
    /// missing file's settings as uncontrolled**, sending the next session to
    /// write a control that already exists a few lines away.
    #[test]
    fn every_source_in_this_directory_is_listed() {
        let file = syn::parse_file(include_str!("mod.rs")).expect("mod.rs did not parse");
        let declared: Vec<String> = file
            .items
            .iter()
            .filter_map(|item| match item {
                syn::Item::Mod(m) if m.content.is_none() => Some(m.ident.to_string()),
                _ => None,
            })
            .collect();
        assert!(
            declared.len() >= 8,
            "parsed {} module declaration(s) — the PARSER is stale, not the list.",
            declared.len()
        );
        let listed: Vec<&str> = SOURCES.iter().map(|(name, _)| *name).collect();
        for name in declared {
            assert!(
                listed.contains(&name.as_str()),
                "`{name}.rs` is not in SOURCES, so any control it binds is invisible to \
                 `every_setting_the_store_carries_has_a_control_in_this_window` — which will \
                 then report that setting as having no control at all, whether or not it has \
                 one. Add it."
            );
        }
    }

    #[test]
    fn every_setting_the_store_carries_has_a_control_in_this_window() {
        let file = Settings::default().write_to_string();
        let keys: Vec<&str> = file
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with('#') && !line.is_empty())
            .filter_map(|line| line.split_once('='))
            .map(|(key, _)| key.trim())
            .collect();

        // A floor, so a change to the file format that stopped this parser
        // finding anything reports ITSELF rather than reporting that every
        // setting is covered. An instrument that can only return one answer
        // cannot detect the thing it was added to detect.
        assert!(
            keys.len() >= 10,
            "parsed {} key(s) out of the settings file — the PARSER is stale, not the window. \
             `Settings::write_to_string` no longer emits `key = value` lines this test can read.",
            keys.len()
        );

        for key in keys {
            let needle = format!("working.{key}");
            assert!(
                SOURCES.iter().any(|(_, src)| src.contains(&needle)),
                "`Settings::{key}` is honoured by the engine and has NO control in this \
                 window, so an operator can only change it by hand-editing settings.txt — \
                 which is not a user interface. Add one to whichever group matches the \
                 SYMPTOM that would send somebody looking for it (see the module header), \
                 rather than to whichever group is shortest."
            );
        }
    }

    /// The shell's own preferences are **not** covered by the sweep above, and
    /// this records why rather than leaving the gap implied.
    ///
    /// [`crate::app::prefs::Prefs`] lives in a different file, is written by the
    /// shell rather than by the engine, and reaches the window through
    /// `draft.working_prefs` rather than `draft.working`. The sweep is
    /// deliberately scoped to the ENGINE's store, because that is where the
    /// asymmetry it exists to catch comes from: `pdfcer-core` gains settings on
    /// its own schedule and this project finds out by reading a note.
    ///
    /// A preference added to `Prefs` is added by this project, in the same
    /// session that would add its control, so the failure mode is not the same
    /// one. If that ever stops being true — if the shell's preferences start
    /// arriving from elsewhere — this is the test to widen.
    #[test]
    fn the_sweep_is_scoped_to_the_engines_store_on_purpose() {
        let draft = Draft::new(&Settings::default(), &crate::app::prefs::Prefs::default());
        // Both halves exist and are distinct fields; the assertion is that this
        // test's premise is still true, so its doc comment is not describing a
        // structure that has since changed.
        let _ = &draft.working;
        let _ = &draft.working_prefs;
    }

    /// Opening the window is not an edit.
    #[test]
    fn a_fresh_draft_is_not_dirty() {
        let draft = Draft::new(&Settings::default(), &crate::app::prefs::Prefs::default());
        assert!(!draft.is_dirty());
        assert!(draft.is_all_default());
    }

    /// Changing a value makes Save live and Restore live.
    #[test]
    fn changing_a_value_makes_the_draft_dirty() {
        let mut draft = Draft::new(&Settings::default(), &crate::app::prefs::Prefs::default());
        // ★★ `NeutralBlack`, and it was `Calibrated` until 2026-08-28. The
        // engine's `Pass 153.0` made `Calibrated` the DEFAULT, so assigning it
        // to a default-seeded draft changed nothing and this test failed — on a
        // build where the dirty flag works perfectly.
        //
        // ⇒ **A test that names a specific value to prove "something changed"
        // is coupled to what the default is.** The assertion below is the fix:
        // it states the premise, so the next default move fails with *"the test
        // needs a CHANGE"* rather than with *"the dirty flag is broken"*.
        draft.working.cmyk_intent = CmykIntent::NeutralBlack;
        assert_ne!(
            draft.working.cmyk_intent,
            Settings::default().cmyk_intent,
            "the test needs a CHANGE"
        );
        assert!(draft.is_dirty());
        assert!(!draft.is_all_default());
    }

    /// ★ The dirty flag does not latch.
    ///
    /// A radio click and a click back is not an edit, and a Save button that
    /// stayed live afterwards would be telling the operator they have unsaved
    /// changes when they have none — which is the same lie as a Save button
    /// that is always live, arrived at by a different route.
    #[test]
    fn changing_a_value_back_makes_it_clean_again() {
        let mut draft = Draft::new(&Settings::default(), &crate::app::prefs::Prefs::default());
        let was = draft.working.cmyk_intent;
        // ★ `NeutralBlack` because `Naive` was DELETED by the engine's
        // `Pass 153.0` on 2026-08-28, hours after O52 asked for it. Any value
        // other than the default works here — the test is about the flag not
        // latching, not about colour — and the one requirement is that it
        // differs from `was`, which the assertion two lines down enforces.
        draft.working.cmyk_intent = CmykIntent::NeutralBlack;
        assert_ne!(draft.working.cmyk_intent, was, "the test needs a CHANGE");
        assert!(draft.is_dirty());
        draft.working.cmyk_intent = was;
        assert!(!draft.is_dirty(), "the dirty flag latched");
    }

    /// ★ The two predicates must not collapse into one.
    ///
    /// A draft opened from non-default settings is **clean** (nothing has been
    /// edited) and **not all-default** (something is not pdfcer's answer).
    /// Collapsing them would grey *Restore defaults* for the operator who
    /// changed something last week and wants it back — which is most of the
    /// people who will ever press it.
    #[test]
    fn a_draft_started_from_non_default_settings_is_clean_but_not_all_default() {
        // `Settings` is `#[non_exhaustive]`, so a struct expression is illegal
        // out of crate: the only shape available is start-from-default and
        // assign, which is what this seeds through.
        let mut seed = Settings::default();
        seed.separations = SeparationPolicy::Refuse;
        let draft = Draft::new(&seed, &crate::app::prefs::Prefs::default());
        assert!(!draft.is_dirty(), "loading is not editing");
        assert!(
            !draft.is_all_default(),
            "Restore defaults must stay available"
        );
    }

    /// Restore defaults reaches all-default without going through Save.
    #[test]
    fn restoring_defaults_is_a_draft_edit_and_nothing_else() {
        let mut seed = Settings::default();
        seed.separations = SeparationPolicy::Refuse;
        let mut draft = Draft::new(&seed, &crate::app::prefs::Prefs::default());
        draft.working = Settings::default();
        assert!(draft.is_all_default());
        // …and it is an *edit*, so Save is offered. The operator must confirm.
        assert!(
            draft.is_dirty(),
            "restoring defaults must leave something to save"
        );
    }
}
