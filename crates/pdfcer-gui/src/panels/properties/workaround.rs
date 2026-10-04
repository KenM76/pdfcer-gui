//! # `panels::properties::workaround` — the offer to make a refused text edit
//! another way
//!
//! Contract: when the engine refuses a committed text edit and names a
//! workaround (`EditError::workaround`), [`record`] keeps what was typed and
//! the workaround; [`section`] shows the offer until the document changes, and
//! its button re-raises the same edit with `workarounds: true`, so the engine
//! applies the workaround only on the operator's press, never by default. If
//! the retry is refused too ([`record_failed`]), the section says so instead.
//!
//! The applied workaround is disclosed by the engine's own report, which the
//! status line shows; nothing is marked on the page.

use std::cell::RefCell;

use pdfcer_core::text_edit::Workaround;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::textedit::Committing;
use crate::text::workaround as t;

/// The section, published on the frames it draws.
pub const REGION: &str = "properties.workaround"; // ui-text-exempt: trace region name, never displayed
/// Its button.
pub const APPLY_REGION: &str = "properties.workaround.apply"; // ui-text-exempt: trace region name, never displayed

#[derive(Clone, Debug)]
enum Offer {
    /// The workaround on offer for this edit.
    Open(Committing, Workaround),
    /// The workaround was tried and refused; the engine's reason.
    Failed(String),
}

#[derive(Clone, Debug)]
struct Shown {
    offer: Offer,
    /// `OpenDoc::edit_epoch` when it was raised; any edit retires it.
    epoch: u64,
}

thread_local! {
    static PENDING: RefCell<Option<Offer>> = const { RefCell::new(None) };
    static SHOWN: RefCell<Option<Shown>> = const { RefCell::new(None) };
}

/// Offer `workaround` for the refused edit `typed`.
pub(crate) fn record(typed: Committing, workaround: Workaround) {
    PENDING.with_borrow_mut(|slot| *slot = Some(Offer::Open(typed, workaround)));
}

/// The workaround was tried and refused for `why`.
pub(crate) fn record_failed(why: String) {
    PENDING.with_borrow_mut(|slot| *slot = Some(Offer::Failed(why)));
}

/// Drop any offer: a `(page, run)` names different text in another document.
pub(crate) fn forget_document() {
    PENDING.with_borrow_mut(|slot| *slot = None);
    SHOWN.with_borrow_mut(|slot| *slot = None);
}

/// The offer as it stands this frame, or `None` once an edit has retired it.
fn current(epoch: u64) -> Option<Offer> {
    if let Some(offer) = PENDING.with_borrow_mut(Option::take) {
        SHOWN.with_borrow_mut(|slot| *slot = Some(Shown { offer, epoch }));
    }
    SHOWN.with_borrow_mut(|slot| {
        if slot.as_ref().is_some_and(|s| s.epoch != epoch) {
            *slot = None;
        }
        slot.as_ref().map(|s| s.offer.clone())
    })
}

/// Draw the offer; `true` when it drew.
pub(super) fn section(ui: &mut egui::Ui, doc: &OpenDoc, actions: &mut Vec<Action>) -> bool {
    let Some(offer) = current(doc.edit_epoch) else {
        return false;
    };
    ui.label(t::heading());
    match offer {
        Offer::Open(typed, workaround) => {
            ui.label(t::offer(workaround));
            ui.label(egui::RichText::new(t::typed(&typed.replacement)).small());
            let button = ui.button(t::apply_button()).on_hover_text(t::apply_hover());
            crate::diag::ui_rect(APPLY_REGION, button.rect);
            crate::diag::trace(|| {
                format!(
                    "workaround-offer page={} run={} workaround={}",
                    typed.page,
                    typed.run,
                    workaround.label()
                )
            });
            if button.clicked() {
                SHOWN.with_borrow_mut(|slot| *slot = None);
                actions.push(Action::CommitTextEdit {
                    page: typed.page,
                    run: typed.run,
                    original: typed.original,
                    replacement: typed.replacement,
                    reface: None,
                    workarounds: true,
                });
            }
        }
        Offer::Failed(why) => {
            ui.label(t::failed(&why));
            crate::diag::trace(|| "workaround-offer state=failed".to_owned());
        }
    }
    crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
    ui.separator();
    true
}
