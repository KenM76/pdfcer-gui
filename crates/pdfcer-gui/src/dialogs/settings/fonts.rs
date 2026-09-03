//! # `dialogs::settings::fonts` — the Settings window's Fonts group
//!
//! One control: the list of folders pdfcer searches when it has to embed a font
//! a document names but does not carry.
//!
//! ## ★★★ Why it is HERE and not on a batch pane
//!
//! `tools.font_folders`' recorded blocker said the list *"needs the pane it
//! lives in"* — the old shell's batch pane, one of the few units
//! `SALVAGE.md` still lists as not carried across. That was an **assumption
//! rather than a finding**, and re-deriving it on 2026-08-28 is what turned it
//! up: nothing about a list of directories needs a batch pane. This window has
//! nine modules and seven groups and is the surface whose whole subject is
//! *"settings that persist across documents"*, which is exactly what a font
//! search path is.
//!
//! ⇒ Recorded because the shape recurs: a blocker naming a **missing host** is
//! weaker than one naming a missing capability, and it goes stale the moment
//! any other host will do. Nobody had asked whether one would.
//!
//! ## ★★ The command still exists, and points here
//!
//! `tools.font_folders` on the Tools tab raises
//! `Action::Command("file.settings")`. That is `format.properties`' precedent
//! and its stated rule: *"a second route to an existing command cannot become
//! a second implementation of it."* An operator who looks on the Tools tab
//! finds the command where it has always been drawn; it opens the window that
//! holds the list.
//!
//! ## ★ What this group deliberately does NOT do
//!
//! **It does not check that a folder exists**, and it does not list the faces
//! in one. The first is [`crate::app::prefs::fonts::add`]'s stated position —
//! an unmounted drive is still where the fonts live. The second is a font
//! census, which is `panels::fonts`' subject and would be a second inventory
//! with a second way of going stale.

use egui::Ui;
use pdfcer_core::settings::StylePolicy;

use crate::app::prefs::{Prefs, fonts};
use crate::text::settings as t;

use super::{Draft, widgets};

/// The Fonts group's rect, for `ui-verify`.
// ui-text-exempt: trace region name, never displayed
pub const REGION: &str = "settings.fonts";
/// The Add button.
// ui-text-exempt: trace region name, never displayed
pub const ADD_REGION: &str = "settings.fonts.add";
/// The "use this computer's fonts" checkbox.
// ui-text-exempt: trace region name, never displayed
pub const OS_REGION: &str = "settings.fonts.use_os";
/// The faking-bold-and-italic radio group.
// ui-text-exempt: trace region name, never displayed
pub const STYLE_POLICY_REGION: &str = "settings.fonts.style_policy";

/// Draw the folder list and its two controls.
///
/// Takes `&mut Prefs` rather than the settings draft, like
/// `appearance::ui_scale`: this is a **shell preference**, not a
/// `pdfcer_core::settings` entry, and the window's own header explains that the
/// two live side by side in one dialog because an operator does not care which
/// file a setting lands in.
pub fn folders(ui: &mut Ui, prefs: &mut Prefs) {
    ui.label(t::font_folders_label());
    ui.small(t::font_folders_hint());
    ui.add_space(4.0);

    // ★ The list is drawn before the buttons, and each row carries its own
    // Remove. A single "remove selected" would need a selection model for a
    // list that is at most sixteen rows — and a row whose delete is on the row
    // is the arrangement every list in this shell already uses.
    let mut remove: Option<usize> = None;
    for (index, folder) in prefs.font_folders.iter().enumerate() {
        ui.horizontal(|ui| {
            if ui
                .small_button(t::font_folder_remove())
                .on_hover_text(t::font_folder_remove_hover())
                .clicked()
            {
                remove = Some(index);
            }
            // ★ `truncate` with the full path on hover — `panels::properties`'
            // row rule, and for its reason: a path can run to any length and a
            // row that grew to three lines would push the buttons under it
            // around as the operator added folders.
            let text = folder.display().to_string();
            ui.add(egui::Label::new(&text).truncate())
                .on_hover_text(&text);
        });
    }
    if let Some(index) = remove {
        prefs.font_folders.remove(index);
    }

    if prefs.font_folders.is_empty() {
        // ★★ The empty state is a SENTENCE, not a blank. An empty list is
        // indistinguishable from a broken control, and this one has a
        // consequence worth stating before the operator meets it at the far end
        // of an embed.
        //
        // ★★★ TWO sentences, because as of the OS-fonts checkbox there are two
        // states and only one of them is a problem. "No folders" no longer
        // means "nothing to embed from" — the box may be ticked — and an empty
        // state that contradicted a control four rows below it would tell an
        // operator who HAS ticked it that their setting does not work.
        ui.small(if prefs.use_os_fonts {
            t::font_folders_none()
        } else {
            t::font_folders_none_at_all()
        });
    }

    ui.add_space(4.0);
    let full = prefs.font_folders.len() >= fonts::MAX_FOLDERS;
    let add = ui.add_enabled(!full, egui::Button::new(t::font_folder_add()));
    crate::diag::ui_rect_visible(ADD_REGION, add.rect, ui.clip_rect());
    let add = if full {
        // R9: greyed for a temporarily unavailable capability, explained on
        // hover. Removing a folder makes it available again, which is what the
        // sentence says.
        add.on_disabled_hover_text(t::font_folders_full(fonts::MAX_FOLDERS))
    } else {
        add.on_hover_text(t::font_folder_add_hover())
    };
    if add.clicked()
        && let crate::app::files::Picked::Path(path) = crate::app::files::pick_font_folder()
    {
        fonts::add(&mut prefs.font_folders, &path);
    }

    ui.add_space(8.0);
    ui.separator();
    // ★★★ **The checkbox the operator asked for — `OPERATOR_REQUESTS.md` O50.**
    //
    // *"just a simple checkbox to include fonts from the OS installed font
    // folders."* Below the list rather than above it, and the order is the
    // argument: the folders **he** curated are the primary answer and this is
    // the fallback, which is also the search order `fonts::search_path`
    // enforces. A control drawn above a list it is subordinate to reads as the
    // main event.
    let os = ui.checkbox(&mut prefs.use_os_fonts, t::use_os_fonts_label());
    crate::diag::ui_rect_visible(OS_REGION, os.rect, ui.clip_rect());
    // ★★ The hint is DRAWN, not put on hover, and it is the one place in this
    // window that argues for itself. Every other hint here describes a control;
    // this one hands the operator a licensing decision, and a decision nobody
    // reads is a decision the program took. Hover text is for the operator who
    // went looking — this is for the one who did not.
    ui.small(t::use_os_fonts_hint());

    // ★★★ The folders the tick resolves to, drawn under it.
    //
    // A checkbox whose effect is invisible is one nobody can verify. The
    // per-user folder in particular — `…\AppData\Local\Microsoft\Windows\Fonts`
    // — is somewhere most operators do not know exists, and it is where a plain
    // double-click on a `.ttf` installs by default on a modern Windows. Listing
    // it is the difference between a setting an operator trusts and one they
    // re-tick to see whether it took.
    //
    // ★ Drawn only when ticked. An unticked box with a list of folders under it
    // states a fact about the machine and implies a promise about the program.
    if prefs.use_os_fonts {
        let found = fonts::os_font_dirs();
        if found.is_empty() {
            ui.small(t::use_os_fonts_none_found());
        } else {
            ui.small(t::use_os_fonts_folders());
            for dir in &found {
                // `weak`, and truncated with the full path on hover, for the
                // rows above's reason: these are a **consequence** of the tick
                // rather than entries the operator manages, and drawing them
                // like the list would invite a Remove button that cannot exist.
                let text = dir.display().to_string();
                ui.add(egui::Label::new(egui::RichText::new(&text).weak().small()).truncate())
                    .on_hover_text(&text);
            }
        }
    }
    crate::diag::ui_rect(REGION, ui.min_rect());
}

/// **May pdfcer fake a bold or an italic that the page has no real face for?**
///
/// # What this setting is, in this shell
///
/// The engine calls it `StylePolicy` and gives it three values. Its own reading
/// of them is narrower than this window's, and the difference is deliberate and
/// worth stating rather than papering over.
///
/// `pdfcer-core`'s gate asks exactly one question: *"could a real face have done
/// this instead?"* — so its `Refuse` refuses a fake **only when a real face was
/// available and would have been passed over**. On a page carrying no bold at
/// all there is nothing to refuse in favour of, and the engine thickens the
/// strokes under all three postures.
///
/// That is the right contract for a crate whose caller might genuinely mean
/// *"fake it"*. It is not what this shell's Bold button means. Here the button
/// means **"make this bold"**, and `crate::app::actions::textstyle` already
/// prefers a real face under every posture — it asks the gate with `Refuse`
/// pinned, purely to learn which real face is on offer, and then takes it.
///
/// ⇒ So the only question left for the operator is the one this control asks:
/// **when no real face will do, may pdfcer fake one?** All three engine values
/// map onto it, and every one of them is observable:
///
/// | choice | what happens |
/// |---|---|
/// | Fake it quietly (`Auto`, the default) | the letters are thickened or slanted, and it is reported with the edit's other disclosures |
/// | Fake it and say so (`Warn`) | the same, plus a sentence of its own on the status bar |
/// | Never fake it (`Refuse`) | nothing changes, and pdfcer says no real face on the page can show that text |
///
/// # ★★ Why it lives in Fonts and not in Text
///
/// The window's rule is *"whichever group matches the SYMPTOM that would send
/// somebody looking"*. The symptom here is **"my bold looks wrong / pdfcer did
/// not use the bold font"** — a statement about faces. The Text group is about
/// what comes out when you copy, which this never touches: a synthesised weight
/// changes how a run is painted, not what it extracts as.
///
/// # ★ Never fake it is NOT the same as an error
///
/// The third option changes nothing and says so, which makes it the only
/// setting in this window that can make a control appear not to work. The note
/// under it says that in advance, because an operator who ticks it in January
/// and presses Bold in March will otherwise file a bug.
pub fn style_policy(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::style_policy_title(),
        t::style_policy_silence(),
        t::style_policy_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.style_policy,
        StylePolicy::Auto,
        t::style_policy_auto_label(),
        Some(t::style_policy_auto_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.style_policy,
        StylePolicy::Warn,
        t::style_policy_warn_label(),
        Some(t::style_policy_warn_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.style_policy,
        StylePolicy::Refuse,
        t::style_policy_refuse_label(),
        Some(t::style_policy_refuse_note()),
    );
    widgets::disclosure(ui, t::style_policy_bound());
    crate::diag::ui_rect(STYLE_POLICY_REGION, ui.min_rect());
}
