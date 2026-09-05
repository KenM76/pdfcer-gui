//! # `dialogs::settings::signatures` — may pdfcer read Acrobat's trust list,
//! and where is it
//!
//! Two settings, and the group exists because of what they have to explain
//! rather than because of how many they are.
//!
//! `ENGINE_BACKLOG.md`'s third trust row asked for exactly this and named the
//! two properties it had to have:
//!
//! > it is a preference with a privacy shape — it reads a file belonging to
//! > another vendor's product. Belongs in Settings beside the other opt-ins,
//! > off by default, with the store's path and its **mtime** disclosed, because
//! > an anchor set that silently went stale is worse than one that was never
//! > imported.
//!
//! Both are here: [`use_store`] draws the engine's own `Off`/`AtOwnRisk`
//! setting, and [`store_path`] draws the location, the resolved state and the
//! **date**.
//!
//! ## ★★★ Why this is not filed under the *Where Acrobat is* group
//!
//! It is the obvious place — both settings are about Acrobat — and it is wrong,
//! for the reason this window's own header gives: **a setting filed under the
//! wrong heading is not untidy, it is unreachable**, because an operator opens
//! this window with a *symptom* and the headings are how a symptom finds its
//! setting.
//!
//! The symptom that brings somebody here is *"the Signatures panel says it did
//! not check who signed this"*. Nobody carrying that symptom looks under a
//! group whose own header says it *"changes nothing at all except which program
//! a single button starts"* — and putting this there would make that sentence
//! false as well as making the setting hard to find.
//!
//! ## Placement: last of the document groups, before the program ones
//!
//! The window's ordering rule runs from what the **program** looks like,
//! through what the **document** is made of, to what pdfcer **does with it**.
//! Reading a document's signatures is squarely the third, so this sits after
//! *Pages and printing* and before *Drawing the page*, which is where the
//! shell's own preferences begin.
//!
//! ## ★★ Two headers, not one, and the reason is the two stores
//!
//! [`super::widgets::toggle`]'s own note says the sub-parts of ONE setting
//! share a header. These are two settings: a **permission**, persisted in
//! `pdfcer_core::settings` where the CLI reads the same answer, and a
//! **location**, persisted in `crate::app::prefs` because the engine has no
//! field for it and deliberately does not — its module header says *"locating
//! the file is the shell's job"*.
//!
//! An operator has no business meeting that split, and does not: the two are
//! adjacent, one Cancel discards both, one Save writes both. But they are two
//! questions with two different blast radii, and a single `radius` line
//! covering both would have to be vague about the one that matters.
//!
//! ## ★★★ R9, and which control is absent
//!
//! [`inspect`] — the button that reads the store and reports what is in it — is
//! drawn **only when a store was actually found**. An unavailable capability
//! renders nothing; greying is reserved for something *temporarily*
//! unavailable, and a person with no Acrobat store is not one press away from
//! having one.
//!
//! The path field above it is drawn **always**, and that is R9's other half:
//! the remedy for an absent capability must be reachable, and an absent
//! capability whose remedy is also absent is a dead end. Somebody whose store
//! is on a redirected profile sees no inspect button, and the field that fixes
//! it is right there with a resolved-state line telling them what pdfcer
//! currently finds.
//!
//! ## The resolved line IS live, unlike the Acrobat group's
//!
//! [`super::acrobat`]'s state line cannot update as the operator types, because
//! resolving an Acrobat spawns processes and a settings pane redraws every
//! frame. Locating a trust store does not: it is `Path::is_file` plus one
//! `metadata`, which is a stat rather than a read. So this line updates
//! keystroke by keystroke and a typo is visible at the place it was made,
//! rather than after a Save.
//!
//! ★ The *anchors* are a different matter — parsing 3 MB of COS and decoding
//! ~1,800 certificates is not a per-frame act — which is exactly why reading
//! them is behind a button and the button caches its answer.

use egui::Ui;

use pdfcer_core::settings::AcrobatTrustStore;

use crate::text::trust as t;

/// The region the resolved-state line publishes.
///
/// ★ Named for [`super::acrobat::REGION_RESOLVED`]'s reason: the whole value of
/// that line is that it is **on screen and legible**, and `ui-verify` can only
/// assert that about a rect the application published. A driven check that read
/// the trace would learn what pdfcer resolved and nothing about whether the
/// operator can see it.
pub const REGION_RESOLVED: &str = "settings:signatures.resolved"; // ui-text-exempt: trace region name, never displayed

/// The Browse button's region.
pub const REGION_BROWSE: &str = "settings:signatures.browse"; // ui-text-exempt: trace region name, never displayed

/// The inspect button's region.
///
/// ★★★ **Its absence is the assertion.** R9 says an unavailable capability
/// renders nothing, and "renders nothing" is only checkable if the thing that
/// would have rendered has a name. A driven check on a machine with no trust
/// store asserts this region is **not** published; on a machine with one it
/// asserts it is, and presses it.
pub const REGION_INSPECT: &str = "settings:signatures.inspect"; // ui-text-exempt: trace region name, never displayed

/// The region the inspect button's answer publishes.
pub const REGION_STORE_LINE: &str = "settings:signatures.store"; // ui-text-exempt: trace region name, never displayed

/// Setting 1 — whether pdfcer may read Acrobat's downloaded trust list.
///
/// The engine's own `Off` / `AtOwnRisk`, bound directly to the draft. Two named
/// alternatives, so [`super::widgets::option`] rather than
/// [`super::widgets::toggle`]: the labels carry the content of the choice, and
/// *"at my own risk"* is a phrase the operator is agreeing to rather than a
/// state they are switching.
///
/// ★★ The at-own-risk disclosure is a [`super::widgets::disclosure`] rather
/// than an option note, and that is the widget's own documented distinction:
/// it belongs to the **setting**, not to either option, and greying it — which
/// an option note does — would be the quiet version of not saying it. It is the
/// sentence somebody would quote back at us.
pub fn use_store(ui: &mut Ui, draft: &mut super::Draft) {
    super::widgets::header(
        ui,
        t::use_store_title(),
        t::use_store_silence(),
        t::use_store_radius(),
    );
    super::widgets::option(
        ui,
        &mut draft.working.acrobat_trust_store,
        AcrobatTrustStore::Off,
        t::use_store_off_label(),
        Some(t::use_store_off_note()),
    );
    super::widgets::option(
        ui,
        &mut draft.working.acrobat_trust_store,
        AcrobatTrustStore::AtOwnRisk,
        t::use_store_on_label(),
        Some(t::use_store_on_note()),
    );
    super::widgets::disclosure(ui, t::at_own_risk());
}

/// Setting 2 — where the trust list is, what pdfcer currently resolves, and
/// (when there is one to read) what is in it.
///
/// ★ `text_value` with an identity parse, exactly as [`super::acrobat::path`]
/// uses it and for its stated reason: the helper exists to hold a half-typed
/// *number* apart from a parsed value, and a path has no invalid intermediate
/// state. Every keystroke reaches the draft, so Save writes exactly what is on
/// screen.
///
/// ★★ **No validation as you type and no red field.** A path that does not
/// exist is not a typing error — it is a path to something not there yet, or on
/// a drive that is not mounted, or typed from memory and about to be corrected.
/// Marking it wrong mid-word would be the field arguing with somebody who has
/// not finished. The resolved line below says what actually happened, and
/// because locating is a stat rather than a process launch it says it live.
pub fn store_path(ui: &mut Ui, draft: &mut super::Draft) {
    super::widgets::header(
        ui,
        t::store_path_title(),
        t::store_path_silence(),
        t::store_path_radius(),
    );
    super::widgets::text_value(
        ui,
        // ui-text-exempt: an egui control id, never displayed.
        "settings-trust-store-path",
        &mut draft.working_prefs.acrobat_trust_store_path,
        t::store_path_label(),
        Some(t::store_path_note()),
        Clone::clone,
        |typed| Some(typed.to_owned()),
    );

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let browse = ui.button(t::store_path_browse());
        crate::diag::ui_rect_visible(REGION_BROWSE, browse.rect, ui.clip_rect());
        let browse = browse.on_hover_text(t::store_path_browse_hover());
        if browse.clicked()
            && let crate::app::files::Picked::Path(picked) = crate::app::files::pick_trust_store()
        {
            draft.working_prefs.acrobat_trust_store_path = picked.display().to_string();
        }
    });

    ui.add_space(6.0);
    // ★★ Located from the DRAFT, not from the live preferences. The window
    // edits a working copy and nothing reaches the configuration until Save —
    // so a resolved line read from the live value would answer a question about
    // the path the operator has just replaced, and would keep answering it
    // until they pressed a button. This is the one place in this window where
    // "the draft is what you are looking at" has to be true of a *derived*
    // reading as well as of a control.
    let located = crate::trust::locate(&draft.working_prefs.acrobat_trust_store_path);
    let line = ui.label(
        egui::RichText::new(resolved_note(&located))
            .color(egui_shell::theme::Theme::of(ui.ctx()).palette.notice),
    );
    crate::diag::ui_rect_visible(REGION_RESOLVED, line.rect, ui.clip_rect());

    if let Some(path) = located.usable() {
        ui.add_space(6.0);
        inspect(ui, path);
    }
}

/// The live resolved-state sentence for a [`crate::trust::Located`].
///
/// Split out so the mapping from state to sentence is a pure function over a
/// value and can be asserted without a frame. Three of the four states get
/// their own sentence; `Configured` and `Discovered` share one, because from
/// the operator's side they are the same fact — *pdfcer will read this file* —
/// and the difference between "you told me" and "I found it" is visible in the
/// field two lines above.
fn resolved_note(located: &crate::trust::Located) -> String {
    match located {
        crate::trust::Located::Configured(path) | crate::trust::Located::Discovered(path) => {
            let date = std::fs::metadata(path)
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(crate::trust::modified_date);
            t::resolved_found(&path.display().to_string(), date.as_deref())
        }
        crate::trust::Located::ConfiguredMissing(path) => {
            t::resolved_configured_missing(&path.display().to_string())
        }
        crate::trust::Located::None { looked_in } => t::resolved_none(looked_in.len()),
    }
}

/// The **import** control: read the store now and report what is in it.
///
/// ## ★★★ Why "import" is a read and not a copy
///
/// Because pdfcer keeps no anchor file of its own. Nothing is copied out of
/// Acrobat's store into pdfcer's configuration, at any point, and this button
/// does not create one — it reads the operator's own file and prints its
/// contents. `ENGINE_BACKLOG.md`'s argument for the whole feature is why:
///
/// > an anchor set that silently went stale is worse than one that was never
/// > imported.
///
/// A copy has no way to say how old it is that anybody will ever read. A live
/// read has one that costs nothing — the file's modification time — so
/// [`crate::text::trust::store_line`] prints the count and the date as one
/// sentence, and there is no function in the catalog that produces one without
/// the other.
///
/// ## The answer is cached, and the cache key is the button press
///
/// Deliberately the simplest possible: the result is held in `egui` memory
/// under this control's own id, and pressing the button replaces it. Parsing
/// 3 MB of COS and decoding ~1,800 certificates is not a per-frame act, and a
/// key over the file's stat would make the button look inert to somebody who
/// pressed it twice — where here the second press is a genuine re-read, which
/// is what a person who has just told Acrobat to update its list wants.
fn inspect(ui: &mut Ui, path: &std::path::Path) {
    // ui-text-exempt: an egui memory key, never displayed.
    let id = egui::Id::new("settings-trust-store-inspect");
    let button = ui.button(t::inspect_button());
    crate::diag::ui_rect_visible(REGION_INSPECT, button.rect, ui.clip_rect());
    if button.on_hover_text(t::inspect_hover()).clicked() {
        let answer = match crate::trust::load(path) {
            Ok(store) => {
                let date = store.modified.and_then(crate::trust::modified_date);
                let mut said = t::store_line(
                    &store.path.display().to_string(),
                    date.as_deref(),
                    &store.counts,
                );
                // Only when non-zero. An operator whose signer happens to be
                // one of the refused entries would otherwise meet an
                // inexplicable "does not chain" with nothing to look at.
                if store.undecodable > 0 {
                    said.push(' ');
                    said.push_str(&t::store_undecodable(store.undecodable));
                }
                crate::diag::trace(|| {
                    format!(
                        "trust-store-inspect path={:?} total={} aatl={} eutl={} adbe={} other={} undecodable={}",
                        store.path,
                        store.counts.total,
                        store.counts.aatl,
                        store.counts.eutl,
                        store.counts.adbe,
                        store.counts.other,
                        store.undecodable
                    )
                });
                said
            }
            Err(reason) => {
                crate::diag::trace(|| format!("trust-store-inspect path={path:?} failed={reason}"));
                t::inspect_failed(&reason)
            }
        };
        ui.ctx().data_mut(|d| d.insert_temp(id, answer));
    }
    if let Some(said) = ui.ctx().data(|d| d.get_temp::<String>(id)) {
        ui.add_space(4.0);
        let line = ui.label(egui::RichText::new(said).small());
        crate::diag::ui_rect_visible(REGION_STORE_LINE, line.rect, ui.clip_rect());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trust::Located;
    use std::path::PathBuf;

    /// **The four resolved states produce four different sentences.**
    ///
    /// The failure this refuses is the cheap one: a single *"no trust list
    /// found"* line under every state, which would tell a person who made a
    /// typo that their machine has no Acrobat.
    #[test]
    fn every_located_state_says_something_different() {
        let p = PathBuf::from(r"D:\nowhere\addressbook.acrodata");
        let lines = [
            resolved_note(&Located::Configured(p.clone())),
            resolved_note(&Located::ConfiguredMissing(p.clone())),
            resolved_note(&Located::None {
                looked_in: vec![p.clone()],
            }),
        ];
        for (i, a) in lines.iter().enumerate() {
            for b in lines.iter().skip(i + 1) {
                assert_ne!(a, b, "two resolved states share one sentence");
            }
        }
        // The configured-but-missing sentence must name the path the operator
        // typed, because the fix is in the field above it.
        assert!(
            lines[1].contains(r"D:\nowhere\addressbook.acrodata"),
            "{}",
            lines[1]
        );
    }

    /// ★★ **`Configured` and `Discovered` deliberately say the same thing.**
    ///
    /// Recorded as an assertion rather than as a comment, because it is the one
    /// place in this module where two states SHARE a sentence and a reader
    /// would otherwise reasonably suspect an oversight. From the operator's
    /// side they are one fact — *pdfcer will read this file* — and the
    /// difference between "you told me" and "I found it" is already visible in
    /// the field two lines above.
    #[test]
    fn a_configured_store_and_a_discovered_one_read_alike() {
        let p = PathBuf::from(r"D:\nowhere\addressbook.acrodata");
        assert_eq!(
            resolved_note(&Located::Configured(p.clone())),
            resolved_note(&Located::Discovered(p))
        );
    }
}
