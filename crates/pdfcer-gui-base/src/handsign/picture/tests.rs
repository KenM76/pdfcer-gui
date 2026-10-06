use super::*;
use pdfcer_core::document::Document;
use pdfcer_core::edit::{EditSession, NewImage};
use pdfcer_core::page_tree::Rect as PdfRect;

/// A 24-bit BMP, `w` x `h`, every pixel `rgb`.
fn bmp(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
    let row = (w * 3).div_ceil(4) * 4;
    let size = 54 + row * h;
    let mut out = Vec::new();
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&size.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&w.to_le_bytes());
    out.extend_from_slice(&h.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(&[0; 24]);
    for _ in 0..h {
        for _ in 0..w {
            out.extend_from_slice(&[rgb[2], rgb[1], rgb[0]]);
        }
        out.resize(out.len() + (row - w * 3) as usize, 0);
    }
    out
}

fn picture(rgb: [u8; 3], clear_white: bool) -> SigPicture {
    SigPicture {
        bytes: Arc::new(bmp(8, 4, rgb)),
        clear_white,
    }
}

#[test]
fn an_rgb_picture_can_have_its_white_cleared() {
    let plain = picture([255, 255, 255], false).image().expect("imports");
    assert!(can_clear_white(&plain));
    assert!(plain.color_key_mask.is_none());
    let keyed = picture([255, 255, 255], true).image().expect("imports");
    assert_eq!(
        keyed.color_key_mask,
        Some(vec![235, 255, 235, 255, 235, 255])
    );
}

#[test]
fn a_picture_with_its_own_transparency_is_left_alone() {
    let rgba = [0u8, 0, 0, 0].repeat(4);
    let mut image = ImportedImage::from_rgba8(2, 2, &rgba).expect("imports");
    if image.soft_mask.is_none() {
        image.color_key_mask = Some(vec![0, 0, 0, 0, 0, 0]);
    }
    assert!(!can_clear_white(&image));
}

#[test]
fn the_kept_form_round_trips_and_refuses_anything_else() {
    for clear in [false, true] {
        let p = picture([10, 20, 30], clear);
        assert_eq!(SigPicture::from_kept(&p.to_kept()), Some(p));
    }
    assert_eq!(SigPicture::from_kept(b"not a kept picture"), None);
    assert_eq!(SigPicture::from_kept(HEADER), None);
}

#[test]
fn the_ink_size_is_the_displayed_pixel_size() {
    let image = picture([0, 0, 0], false).image().expect("imports");
    assert_eq!(ink_size(&image), vec2(8.0, 4.0));
}

/// The page's centre pixel after a red square and then `sig` over it.
fn centre_over_red(sig: &SigPicture) -> [u8; 4] {
    let red = ImportedImage::from_rgba8(1, 1, &[255, 0, 0, 255]).expect("imports");
    let image = sig.image().expect("imports");
    let base = Document::from_bytes(crate::blank::TEMPLATE.to_vec()).expect("template");
    let mut session = EditSession::new(base);
    let rect = PdfRect::from_corners(100.0, 100.0, 300.0, 200.0);
    session
        .add_image(&NewImage::new(0, rect, &red).stretching())
        .expect("red placed");
    session
        .add_image(&NewImage::new(0, rect, &image).stretching())
        .expect("signature placed");
    let (bytes, _) = session
        .to_full_bytes(&pdfcer_core::writer::SaveOptions::identity())
        .expect("writes");
    let doc = Document::from_bytes(bytes).expect("reparses");
    let pages = pdfcer_core::page_tree::pages(&doc).expect("pages");
    let rendered = pdfcer_render::render_page(&doc, &pages[0], 1.0).expect("renders");
    let pix = rendered.pixmap;
    let h = pix.height();
    let (x, y) = (200, h - 150);
    let i = ((y * pix.width() + x) * 4) as usize;
    let d = pix.data();
    [d[i], d[i + 1], d[i + 2], d[i + 3]]
}

#[test]
fn cleared_white_shows_what_is_under_it_and_kept_white_covers_it() {
    let cleared = centre_over_red(&picture([250, 250, 250], true));
    assert!(cleared[0] > 200 && cleared[1] < 60, "{cleared:?}");
    let kept = centre_over_red(&picture([250, 250, 250], false));
    assert!(kept[0] > 200 && kept[1] > 200, "{kept:?}");
}

#[test]
fn the_preview_clears_white_only_when_asked() {
    let alpha_at_centre = |clear: bool| {
        let image = picture([250, 250, 250], clear).image().expect("imports");
        let ([w, h], rgba) =
            preview(&image, &pdfcer_render::RenderOptions::default()).expect("draws");
        assert!(w.max(h) as f32 <= PREVIEW_PX + 1.0, "{w}x{h}");
        rgba[((h / 2) * w + w / 2) * 4 + 3]
    };
    assert_eq!(alpha_at_centre(false), 255);
    assert_eq!(alpha_at_centre(true), 0);
}
