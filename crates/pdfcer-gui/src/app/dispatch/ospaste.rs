//! Pasting what another program copied: a picture becomes page content at
//! the pointer.
//!
//! Contract: [`newer`] decides whether the paste reads the OS clipboard at
//! all (no pdfcer clip, or one older than the clipboard's last write);
//! [`paste`] places what it read. A picture lands at its natural size,
//! centred on the pointer (or the view centre), kept wholly on the page, as
//! one undoable edit.

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::state::Status;
use crate::text::ospaste::OsPasteRefusal;
use pdfcer_gui_base::clippaste::{self, Incoming};

/// What the OS clipboard holds, when it is the content a paste should take.
#[must_use]
pub fn newer(ctx: &egui::Context, has_clip: bool) -> Option<Incoming> {
    if has_clip && !crate::canvas::clipseq::changed(ctx) {
        return None;
    }
    match read() {
        Incoming::Nothing if !has_clip => None,
        incoming => Some(incoming),
    }
}

/// The OS clipboard; under test, what [`tests_fake`] set (nothing by
/// default), so no unit test reads the machine's real clipboard.
fn read() -> Incoming {
    #[cfg(test)]
    return tests_fake::take();
    #[cfg(not(test))]
    clippaste::read()
}

/// Paste `incoming` onto the current page.
pub fn paste(
    app: &mut PdfcerApp,
    ctx: &egui::Context,
    id: &str,
    incoming: Incoming,
    actions: &mut Vec<Action>,
) {
    let Status::Open(doc) = &app.status else {
        return;
    };
    let (image, format) = match incoming {
        Incoming::Image { image, format } => (image, format),
        Incoming::Unreadable(why) => return decline("unreadable", OsPasteRefusal::Unreadable(why)),
        Incoming::Text(_) => return decline("text", OsPasteRefusal::Text),
        Incoming::Nothing => return decline("nothing", OsPasteRefusal::Nothing),
    };
    if !app.capabilities().edit_content {
        crate::diag::trace(|| format!("command-declined id={id} reason=mode-cannot-paste-here"));
        crate::app::status::decline::record_mode_refusal(
            crate::text::clipboard::ModeRefusal::PastePicture,
        );
        return;
    }
    let frame = crate::canvas::zoom::last_frame(ctx);
    let page = frame.as_ref().map_or(doc.view.page_index, |f| f.page);
    let Some(sheet) = doc.pages.get(page) else {
        return;
    };
    let at = frame
        .and_then(|f| {
            let canvas = crate::canvas::zoom::anchor_point(ctx.pointer_latest_pos(), &f);
            crate::viewer::canvas_to_pdf_space(canvas, sheet)
        })
        .map_or_else(
            || {
                let b = sheet.crop_box;
                ((b.llx + b.urx) / 2.0, (b.lly + b.ury) / 2.0)
            },
            |p| (f64::from(p.x), f64::from(p.y)),
        );
    let rect = clippaste::rect_at(at, image.natural_size_pt(), sheet.crop_box);
    crate::diag::trace(|| {
        format!(
            "clip-pasted source=os kind=image format={format} page={page} llx={:.2} lly={:.2} \
             urx={:.2} ury={:.2}",
            rect.llx, rect.lly, rect.urx, rect.ury
        )
    });
    actions.push(Action::InsertImage {
        page,
        rect,
        fit: pdfcer_core::edit::ImageFit::Contain,
        image: std::sync::Arc::new(*image),
    });
}

fn decline(kind: &str, why: OsPasteRefusal) {
    crate::diag::trace(|| format!("clip-paste-declined source=os kind={kind}"));
    crate::app::status::decline::record_os_paste(why);
}

/// The clipboard a unit test sees in place of the OS one.
#[cfg(test)]
pub mod tests_fake {
    use super::Incoming;
    use std::cell::RefCell;

    thread_local!(static FAKE: RefCell<Option<Incoming>> = const { RefCell::new(None) });

    /// Make the next read return `incoming`.
    pub fn set(incoming: Incoming) {
        FAKE.with(|f| *f.borrow_mut() = Some(incoming));
    }

    pub(super) fn take() -> Incoming {
        FAKE.with(|f| f.borrow_mut().take())
            .unwrap_or(Incoming::Nothing)
    }
}
