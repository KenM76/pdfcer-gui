//! Pasting what another program copied: a picture becomes page content at
//! the pointer, and text becomes a text box whose top-left corner is the
//! pointer.
//!
//! Contract: [`newer`] decides whether the paste reads the OS clipboard at
//! all (no pdfcer clip, or one older than the clipboard's last write);
//! [`paste`] places what it read as one undoable edit. A picture lands at its
//! natural size, centred on the pointer (or the view centre), kept wholly on
//! the page; a mode that only authors markup places it as a stamp, as does
//! [`paste_stamp`] in any mode that authors markup. A drawing (a PDF another
//! copy placed) lands as a stamp its crop box's size in any mode that authors
//! markup, Edit included: the engine places a PDF page only as stamp artwork.
//! Text becomes page text in
//! a mode that edits content and a
//! `/FreeText` comment in one that only authors markup;
//! `clippaste::textbox` holds the geometry.

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::state::Status;
use crate::text::clipboard::ModeRefusal;
use crate::text::ospaste::OsPasteRefusal;
use pdfcer_core::image_import::ImportedImage;
use pdfcer_core::page_tree::Rect;
use pdfcer_gui_base::clippaste::{self, Incoming, textbox};

/// New PDF from Clipboard and Insert Pages from Clipboard.
pub mod pages;

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

/// Paste `incoming` onto the page under the pointer, or the viewed page.
pub fn paste(
    app: &mut PdfcerApp,
    ctx: &egui::Context,
    id: &str,
    incoming: Incoming,
    actions: &mut Vec<Action>,
) {
    match incoming {
        Incoming::Image { image, format } => picture(app, ctx, id, *image, format, actions),
        Incoming::Pdf { bytes, size_pt } => drawing(app, ctx, id, &bytes, size_pt, actions),
        Incoming::Text(text) => words(app, ctx, id, &text, actions),
        Incoming::Unreadable(why) => decline("unreadable", OsPasteRefusal::Unreadable(why)),
        Incoming::Nothing => decline("nothing", OsPasteRefusal::Nothing),
    }
}

fn target(app: &PdfcerApp, ctx: &egui::Context) -> Option<(usize, (f64, f64), Rect)> {
    target_at(app, ctx, ctx.pointer_latest_pos())
}

/// The page to place on, the point on it, and its crop box, for window point
/// `at`: its page and position, else the viewed page's centre.
pub(crate) fn target_at(
    app: &PdfcerApp,
    ctx: &egui::Context,
    at: Option<egui::Pos2>,
) -> Option<(usize, (f64, f64), Rect)> {
    let Status::Open(doc) = &app.status else {
        return None;
    };
    let frame = crate::canvas::zoom::last_frame(ctx);
    let page = frame.as_ref().map_or(doc.view.page_index, |f| f.page);
    let sheet = doc.pages.get(page)?;
    let at = frame
        .and_then(|f| {
            let canvas = crate::canvas::zoom::anchor_point(at, &f);
            crate::viewer::canvas_to_pdf_space(canvas, sheet)
        })
        .map_or_else(
            || {
                let b = sheet.crop_box;
                ((b.llx + b.urx) / 2.0, (b.lly + b.ury) / 2.0)
            },
            |p| (f64::from(p.x), f64::from(p.y)),
        );
    Some((page, at, sheet.crop_box))
}

fn picture(
    app: &PdfcerApp,
    ctx: &egui::Context,
    id: &str,
    image: ImportedImage,
    format: &str,
    actions: &mut Vec<Action>,
) {
    let caps = app.capabilities();
    if !caps.edit_content && !caps.author_markup {
        return refuse(id, ModeRefusal::PastePicture);
    }
    let Some((page, at, crop)) = target(app, ctx) else {
        return;
    };
    let rect = clippaste::rect_at(at, image.natural_size_pt(), crop);
    if !caps.edit_content {
        return stamp(page, rect, &image, format, actions);
    }
    pasted(
        &format!("kind=image format={format} as=content"), // ui-text-exempt: diagnostic trace
        page,
        rect,
    );
    actions.push(Action::InsertImage {
        page,
        rect,
        fit: pdfcer_core::edit::ImageFit::Contain,
        image: std::sync::Arc::new(image),
    });
}

/// `markup.paste_image_stamp`: the clipboard's picture as a stamp at the
/// pointer, or at the view centre when the pointer is off the page.
pub fn paste_stamp(app: &PdfcerApp, ctx: &egui::Context, id: &str, actions: &mut Vec<Action>) {
    let (image, format) = match read() {
        Incoming::Image { image, format } => (image, format),
        Incoming::Pdf { bytes, size_pt } => {
            return drawing(app, ctx, id, &bytes, size_pt, actions);
        }
        Incoming::Unreadable(why) => return decline("unreadable", OsPasteRefusal::Unreadable(why)),
        Incoming::Text(_) | Incoming::Nothing => {
            return decline("no-picture", OsPasteRefusal::NoPicture);
        }
    };
    if !app.capabilities().author_markup {
        return refuse(id, ModeRefusal::PastePicture);
    }
    let Some((page, at, crop)) = target(app, ctx) else {
        return;
    };
    let rect = clippaste::rect_at(at, image.natural_size_pt(), crop);
    stamp(page, rect, &image, format, actions);
}

/// `image` as custom-stamp artwork filling `rect` on `page`.
fn stamp(page: usize, rect: Rect, image: &ImportedImage, format: &str, actions: &mut Vec<Action>) {
    let file = match stamp_file(image) {
        Ok(file) => file,
        Err(why) => return decline("unplaceable", OsPasteRefusal::Unplaceable(why)),
    };
    // ui-text-exempt: diagnostic trace, never displayed in the UI
    pasted(&format!("kind=image format={format} as=stamp"), page, rect);
    let label = crate::text::ospaste::pasted_picture();
    actions.push(stamp_action(page, rect, file, label));
}

/// A drawing as a stamp its own size, centred on the pointer or the view.
fn drawing(
    app: &PdfcerApp,
    ctx: &egui::Context,
    id: &str,
    bytes: &[u8],
    size_pt: (f64, f64),
    actions: &mut Vec<Action>,
) {
    if !app.capabilities().author_markup {
        return refuse(id, ModeRefusal::PasteDrawing);
    }
    let Some((page, at, crop)) = target(app, ctx) else {
        return;
    };
    let rect = clippaste::rect_at(at, size_pt, crop);
    // ui-text-exempt: a file name, never displayed
    let name = format!("pasted-drawing-{}.pdf", clippaste::sequence());
    let file = match clippaste::page::scratch(&name, bytes) {
        Ok(file) => file,
        Err(why) => return decline("unplaceable", OsPasteRefusal::Unplaceable(why)),
    };
    // ui-text-exempt: diagnostic trace, never displayed in the UI
    pasted("kind=pdf as=stamp", page, rect);
    let label = crate::text::ospaste::pasted_drawing();
    actions.push(stamp_action(page, rect, file, label));
}

/// Page 1 of the PDF at `file` as a custom stamp named `label` filling `rect`.
fn stamp_action(page: usize, rect: Rect, file: std::path::PathBuf, label: &str) -> Action {
    Action::CommitTextAnnot {
        page,
        kind: crate::canvas::textannot::TextAnnotKind::Stamp,
        rect,
        text: String::new(),
        stamp: crate::canvas::textannot::DEFAULT_STAMP,
        stamp_size: crate::canvas::textannot::DEFAULT_STAMP_SIZE,
        icon: crate::canvas::textannot::DEFAULT_STICKY_ICON,
        custom: Some(crate::stamps::library::CustomStamp {
            label: label.to_owned(),
            category: String::new(),
            file,
            page_index: 0,
            dynamic: false,
        }),
    }
}

/// `image`'s one-page PDF, written to the temporary folder for the stamp
/// verb to read; the file is named by the clipboard's change counter, so one
/// copy is written once.
fn stamp_file(image: &ImportedImage) -> Result<std::path::PathBuf, String> {
    let bytes = pdfcer_gui_base::blank::picture_page(image)?;
    // ui-text-exempt: a file name, never displayed
    let name = format!("pasted-picture-{}.pdf", clippaste::sequence());
    clippaste::page::scratch(&name, &bytes)
}

/// Text as page text where content can change, else as a text-box comment.
fn words(app: &PdfcerApp, ctx: &egui::Context, id: &str, text: &str, actions: &mut Vec<Action>) {
    let caps = app.capabilities();
    if !caps.edit_content && !caps.author_markup {
        return refuse(id, ModeRefusal::PasteText);
    }
    let Some((page, at, crop)) = target(app, ctx) else {
        return;
    };
    let text = textbox::normalise(text);
    if caps.edit_content {
        let rect = textbox::content_box(at, crop);
        // ui-text-exempt: diagnostic trace fields, never displayed
        pasted("kind=text as=content", page, rect);
        actions.push(Action::CommitAddText {
            page,
            origin: (rect.llx, rect.lly),
            text,
            pen: crate::canvas::textedit::pen::read(ctx),
            wrap: Some((rect.llx, rect.lly, rect.urx, rect.ury)),
        });
        return;
    }
    let size = pdfcer_gui_base::wordmarkup::TEXT_SIZE_PT;
    let rect = textbox::comment_box(at, &text, size, crop);
    // ui-text-exempt: diagnostic trace fields, never displayed
    pasted("kind=text as=comment", page, rect);
    actions.push(Action::CommitTextAnnot {
        page,
        kind: crate::canvas::textannot::TextAnnotKind::TextBox,
        rect,
        text,
        stamp: pdfcer_core::annot_author::StampName::default(),
        stamp_size: crate::canvas::textannot::DEFAULT_STAMP_SIZE,
        icon: pdfcer_core::annot_author::StickyIcon::default(),
        custom: None,
    });
}

fn pasted(what: &str, page: usize, rect: Rect) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "clip-pasted source=os {what} page={page} llx={:.2} lly={:.2} urx={:.2} ury={:.2}",
            rect.llx, rect.lly, rect.urx, rect.ury
        )
    });
}

fn refuse(id: &str, why: ModeRefusal) {
    // ui-text-exempt: diagnostic trace, never displayed in the UI
    crate::diag::trace(|| format!("command-declined id={id} reason=mode-cannot-paste-here"));
    crate::app::status::decline::record_mode_refusal(why);
}

fn decline(kind: &str, why: OsPasteRefusal) {
    // ui-text-exempt: diagnostic trace, never displayed in the UI
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
