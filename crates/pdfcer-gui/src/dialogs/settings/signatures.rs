//! # `dialogs::settings::signatures` — may pdfcer read Acrobat's trust list,
//! and where is it
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/settings/signatures.md`.

use egui::Ui;

use pdfcer_core::settings::AcrobatTrustStore;

use crate::text::trust as t;

/// The region the resolved-state line publishes.
///
/// Named for [`super::acrobat::REGION_RESOLVED`]'s reason: the whole value of
/// that line is that it is **on screen and legible**, and `ui-verify` can only
/// assert that about a rect the application published. A driven check that read
/// the trace would learn what pdfcer resolved and nothing about whether the
/// operator can see it.
pub const REGION_RESOLVED: &str = "settings:signatures.resolved"; // ui-text-exempt: trace region name, never displayed

/// The Browse button's region.
pub const REGION_BROWSE: &str = "settings:signatures.browse"; // ui-text-exempt: trace region name, never displayed

/// The inspect button's region.
///
/// **Its absence is the assertion.** R9 says an unavailable capability
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
/// The at-own-risk disclosure is a [`super::widgets::disclosure`] rather
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
/// `text_value` with an identity parse, exactly as [`super::acrobat::path`]
/// uses it and for its stated reason: the helper exists to hold a half-typed
/// *number* apart from a parsed value, and a path has no invalid intermediate
/// state. Every keystroke reaches the draft, so Save writes exactly what is on
/// screen.
///
/// **No validation as you type and no red field.** A path that does not
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
    // Located from the DRAFT, not from the live preferences. The window
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

    /// **`Configured` and `Discovered` deliberately say the same thing.**
    #[test]
    fn a_configured_store_and_a_discovered_one_read_alike() {
        let p = PathBuf::from(r"D:\nowhere\addressbook.acrodata");
        assert_eq!(
            resolved_note(&Located::Configured(p.clone())),
            resolved_note(&Located::Discovered(p))
        );
    }
}
