//! # `dialogs::settings::nav` — the page list, the search box and the page pane
//!
//! The window shows one page at a time: a list of pages on the left, grouped
//! under section labels, and the selected page on the right in its own scroll
//! area. A search box above both filters the list to pages holding an option
//! whose name matches, and marks the matching names on the page.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/settings/nav.md`.

use std::sync::Arc;

use egui::{RichText, Ui};

use super::{Draft, widgets};
use crate::text::settings as t;

/// The search box, published so a driven check can type into it.
pub const REGION_SEARCH: &str = "settings.search"; // ui-text-exempt: trace region name, never displayed

/// The pane showing the selected page.
pub const REGION_PAGE: &str = "settings.page"; // ui-text-exempt: trace region name, never displayed

/// Width of the page list, in points.
const NAV_WIDTH: f32 = 170.0;

/// The label a run of pages sits under in the list.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Section {
    /// No label: the two pages that are about everything else.
    Start,
    Program,
    Document,
    Authoring,
    Output,
}

/// Every page, in the order the list shows them. The order is the contract:
/// the program first, then what a document is made of, then adding to it, then
/// writing it.
const PAGES: &[(Section, &str)] = &[
    (Section::Start, "general"),
    (Section::Start, "presets"),
    (Section::Program, "appearance"),
    (Section::Program, "display"),
    (Section::Program, "shortcuts"),
    (Section::Program, "acrobat"),
    (Section::Program, "remote"),
    (Section::Document, "colour"),
    (Section::Document, "fonts"),
    (Section::Document, "images"),
    (Section::Document, "text"),
    (Section::Document, "pages"),
    (Section::Document, "signatures"),
    (Section::Authoring, "measuring"),
    (Section::Authoring, "comments"),
    (Section::Authoring, "forms"),
    (Section::Output, "saving"),
    (Section::Output, "redaction"),
];

/// Whether the window has a page keyed `key`, for a route that selects one.
pub(crate) fn has_page(key: &str) -> bool {
    PAGES.iter().any(|(_, k)| *k == key)
}

/// The page shown when nothing asked for another.
const FIRST: &str = "general";

fn section_label(section: Section) -> Option<&'static str> {
    match section {
        Section::Start => None,
        Section::Program => Some(t::nav_program()),
        Section::Document => Some(t::nav_document()),
        Section::Authoring => Some(t::nav_authoring()),
        Section::Output => Some(t::nav_output()),
    }
}

/// A page's heading, as the list and the pane show it.
fn heading(key: &str) -> String {
    match key {
        "general" => t::group_general().to_owned(),
        "presets" => t::preset_title().to_owned(),
        "appearance" => t::group_appearance().to_owned(),
        "display" => t::group_display().to_owned(),
        "shortcuts" => t::keys::title().to_owned(),
        "acrobat" => crate::text::acrobat::group_acrobat(),
        "remote" => t::group_remote().to_owned(),
        "colour" => t::group_colour().to_owned(),
        "fonts" => t::group_fonts().to_owned(),
        "images" => t::group_images().to_owned(),
        "text" => t::group_text().to_owned(),
        "pages" => t::group_pages().to_owned(),
        "signatures" => crate::text::trust::group_signatures().to_owned(),
        "measuring" => t::group_measuring().to_owned(),
        "comments" => t::group_comments().to_owned(),
        "forms" => t::group_forms().to_owned(),
        "saving" => t::group_saving().to_owned(),
        "redaction" => t::group_redaction().to_owned(),
        _ => key.to_owned(),
    }
}

/// The names each page's options carry, lower-cased, for the search.
type Index = Arc<Vec<(&'static str, Vec<String>)>>;

/// Draw the search box, the page list and the selected page into `height`
/// points. `focus`, when set, selects that page.
pub fn show(
    ui: &mut Ui,
    height: f32,
    draft: &mut Draft,
    focus: Option<&'static str>,
    acrobat_viewer: Option<&crate::acrobat::Viewer>,
) {
    let query_id = egui::Id::new("settings.search-query");
    let page_id = egui::Id::new("settings.page-key");
    let mut query: String = ui.data(|d| d.get_temp(query_id)).unwrap_or_default();
    // A focus naming no page is ignored rather than showing an empty pane.
    let mut selected: &'static str = focus
        .filter(|key| has_page(key))
        .or_else(|| ui.data(|d| d.get_temp(page_id)))
        .unwrap_or(FIRST);

    let top = ui.cursor().top();
    // escape-disposition: dialog-cancels — `dialogs::host` owns the key for
    // every field in this window.
    let search = ui.add(
        egui::TextEdit::singleline(&mut query)
            .hint_text(t::search_hint())
            .desired_width(f32::INFINITY),
    );
    crate::diag::ui_rect(REGION_SEARCH, search.rect);
    let search = search.on_hover_text(t::search_hover());
    ui.data_mut(|d| d.insert_temp(query_id, query.clone()));
    widgets::set_query(&query);

    let needle = query.trim().to_lowercase();
    let shown: Vec<(Section, &'static str)> = if needle.is_empty() {
        PAGES.to_vec()
    } else {
        let index = index(ui, draft, acrobat_viewer);
        PAGES
            .iter()
            .copied()
            .filter(|(_, key)| {
                heading(key).to_lowercase().contains(&needle)
                    || index
                        .iter()
                        .find(|(k, _)| k == key)
                        .is_some_and(|(_, names)| names.iter().any(|n| n.contains(&needle)))
            })
            .collect()
    };
    // A search that hides the page on show moves to the first page it kept,
    // so the pane never shows a page the list does not.
    if search.changed()
        && let Some((_, first)) = shown.first()
        && !shown.iter().any(|(_, k)| *k == selected)
    {
        selected = first;
    }

    ui.add_space(4.0);
    let rest = (height - (ui.cursor().top() - top)).max(120.0);
    ui.horizontal_top(|ui| {
        // The vertical separator below spans the row's AVAILABLE height, which
        // uncapped is the whole window: the row then pushes the footer past
        // the bottom edge, and the host's fit-to-content grows the window
        // after it until its budget runs out with Save off-screen.
        ui.set_max_height(rest);
        ui.allocate_ui_with_layout(
            egui::vec2(NAV_WIDTH, rest),
            egui::Layout::top_down_justified(egui::Align::LEFT),
            |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("settings-nav")
                    .max_height(rest)
                    .auto_shrink([false, false])
                    .show(ui, |ui| list(ui, &shown, &mut selected));
            },
        );
        ui.separator();
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), rest),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                crate::diag::ui_rect(REGION_PAGE, ui.max_rect());
                if shown.iter().any(|(_, k)| *k == selected) {
                    ui.label(RichText::new(heading(selected)).heading());
                    ui.separator();
                    egui::ScrollArea::vertical()
                        // Per page, so each opens at its top.
                        .id_salt(("settings-page", selected))
                        .max_height(ui.available_height())
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            super::page_body(ui, selected, draft, acrobat_viewer)
                        });
                } else {
                    ui.label(RichText::new(t::search_none(query.trim())).weak());
                }
            },
        );
    });

    ui.data_mut(|d| d.insert_temp(page_id, selected));
    crate::diag::trace_changed("settings-page", || format!("settings-page key={selected}"));
}

/// The page list. A click selects the page; it never toggles it.
fn list(ui: &mut Ui, shown: &[(Section, &'static str)], selected: &mut &'static str) {
    let mut last = None;
    for &(section, key) in shown {
        if last != Some(section) {
            if let Some(label) = section_label(section) {
                ui.add_space(6.0);
                ui.label(RichText::new(label).small().weak());
            }
            last = Some(section);
        }
        let entry = ui.selectable_label(*selected == key, heading(key));
        // The heading region a driven check clicks to reach a page. See
        // `super::REGION_HEADING_PREFIX`.
        crate::diag::ui_rect_visible(
            &format!("{}{key}", super::REGION_HEADING_PREFIX),
            entry.rect,
            ui.clip_rect(),
        );
        if entry.clicked() {
            *selected = key;
        }
    }
}

/// Every page's option names, read by drawing each page once, invisibly and
/// with the trace muted, while [`widgets`] collects the names it is handed.
/// Built on the first search of the process: the names are fixed copy.
fn index(ui: &mut Ui, draft: &mut Draft, acrobat_viewer: Option<&crate::acrobat::Viewer>) -> Index {
    let id = egui::Id::new("settings.search-index");
    if let Some(index) = ui.data(|d| d.get_temp::<Index>(id)) {
        return index;
    }
    let rect = ui.max_rect();
    let built: Vec<(&'static str, Vec<String>)> = PAGES
        .iter()
        .map(|&(_, key)| {
            let names = crate::diag::muted(|| {
                widgets::collect(|| {
                    let mut hidden = ui.new_child(
                        egui::UiBuilder::new()
                            .id_salt(("settings-index", key))
                            .max_rect(rect)
                            .invisible(),
                    );
                    hidden.set_clip_rect(egui::Rect::NOTHING);
                    super::page_body(&mut hidden, key, draft, acrobat_viewer);
                })
            });
            (key, names)
        })
        .collect();
    let index: Index = Arc::new(built);
    ui.data_mut(|d| d.insert_temp(id, index.clone()));
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_page_has_a_heading_of_its_own() {
        for (_, key) in PAGES {
            assert_ne!(heading(key), *key, "page `{key}` has no heading");
        }
    }

    #[test]
    fn every_page_is_listed_once_and_the_first_page_exists() {
        for (i, (_, key)) in PAGES.iter().enumerate() {
            assert!(
                !PAGES[i + 1..].iter().any(|(_, k)| k == key),
                "`{key}` listed twice"
            );
        }
        assert!(PAGES.iter().any(|(_, k)| *k == FIRST));
    }

    /// Every page draws options the search can find, which is also the proof
    /// that `super::page_body` knows every key this list names.
    #[test]
    fn every_page_offers_the_search_a_name() {
        let ctx = egui::Context::default();
        let mut draft = Draft::new(
            &pdfcer_core::settings::Settings::default(),
            &crate::app::prefs::Prefs::default(),
        );
        let mut found = Vec::new();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            for (_, key) in PAGES {
                let names = widgets::collect(|| super::super::page_body(ui, key, &mut draft, None));
                found.push((*key, names.len()));
            }
        });
        let empty: Vec<&str> = found
            .iter()
            .filter(|(key, n)| *n == 0 && *key != "general")
            .map(|(key, _)| *key)
            .collect();
        assert!(
            empty.is_empty(),
            "pages the search cannot find by option: {empty:?}"
        );
    }
}
