//! # `clipboard::place` — **the half that was missing, and the transaction it
//! # makes**
//!
//! [`super`] builds the bytes and states the order. This module produces the
//! payload from a real document and hands the ordered set to
//! `native_clipboard::place`, which is the crate that owns the `unsafe`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/clipboard/place.md`.

use super::{ClipFormat, CopyPayload, ORDER, dib_v5, pixels_per_metre, svg_payload};

/// **The resolution a copy-out renders at.**
const COPY_DPI: f32 = 150.0;

/// What a copy-out put on the clipboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placed {
    /// The format names, in placement order — the names
    /// [`ClipFormat::name`] gives, which are the ones an operator can search
    /// for and the ones a pasting application matches on.
    pub formats: Vec<&'static str>,
    /// Whether the *selection* was copied rather than the whole page.
    pub selection: bool,
}

/// Why a copy-out did not happen.
///
/// Four variants rather than one, because the operator's next move differs
/// for every one of them and a single "copy failed" would tell them nothing
/// about which. The sentences live in [`crate::text::clipboard`]; this type
/// carries only the distinction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The document has no page at the current index. Nothing to copy.
    NoPage,
    /// A writer refused the page or the selection. Carries the engine's own
    /// message, which names the reason in the engine's vocabulary.
    Render(String),
    /// **The payload would have degraded Word's paste to a flat picture**
    /// — a raster with no vector in front of it. Refused rather than placed.
    ///
    /// Unreachable while both vector writers succeed, which is why it is a
    /// variant and not an `assert`: if `export_svg` and `export_emf` ever both
    /// fail on a page whose raster renders, the operator gets a sentence saying
    /// the vector form could not be made — not a picture they will discover is
    /// flat when they try to edit it a week later.
    WouldDegrade,
    /// The clipboard itself refused. Carries `native-clipboard`'s own error,
    /// whose commonest member is *another process is holding the clipboard*,
    /// which is transient and worth retrying.
    Clipboard(native_clipboard::PlaceError),
}

/// **Copy the current page — or the selection on it — as vectors.**
///
/// The whole command, in the order the operator experiences it: work out what
/// the operand is, produce every format for it, refuse if the set would
/// degrade, then place it in one transaction.
///
/// # Why the payload is built BEFORE the clipboard is opened
///
/// A CAD sheet can take seconds to record. Windows serialises clipboard access
/// across every process on the desktop, so holding it open while rendering
/// would stall every other application's copy and paste for that long. The
/// engine's CLI states the same rule for the same reason, and
/// `native-clipboard` enforces it structurally: nothing there opens the
/// clipboard until every handle has already been created.
///
/// # Errors
///
/// [`Refusal`] — see the type. Every variant leaves the operator's existing
/// clipboard contents intact except [`Refusal::Clipboard`] carrying
/// `PlaceError::Set`, which is the one failure Win32 offers no rollback for.
pub fn copy_out(doc: &crate::app::state::OpenDoc) -> Result<Placed, Refusal> {
    let options = render_options(doc);
    let selection = selection_payload(doc, &options);
    // Recorded before the `?`, because the disclosure has to say WHICH
    // operand was copied and the `Result` is consumed on the next line. An
    // operator who selected three objects and got the whole page has been told
    // something untrue about their own document.
    let from_selection = selection.is_some();
    let payload = match selection {
        Some(payload) => payload?,
        None => page_payload(doc, &options)?,
    };
    let formats = place(&payload)?;
    Ok(Placed {
        formats,
        selection: from_selection,
    })
}

/// **The render options both routes use.**
fn render_options(doc: &crate::app::state::OpenDoc) -> pdfcer_render::RenderOptions {
    use crate::app::settings::SettingsExt;
    let mut options = doc
        .settings
        .render_options()
        .with_backdrop(pdfcer_render::PageBackdrop::Transparent);
    options.annotations = doc.annotations_visible();
    options.layers = doc.layer_visibility();
    options
}

/// The whole current page, in every format [`ORDER`] names.
fn page_payload(
    doc: &crate::app::state::OpenDoc,
    options: &pdfcer_render::RenderOptions,
) -> Result<CopyPayload, Refusal> {
    let Some(page) = doc.current_page() else {
        return Err(Refusal::NoPage);
    };
    // `session.view()`, NOT `session.document()` — the view composes the
    // overlay and the staging buffer, so **unsaved edits are what gets
    // copied**. The exporter and the print preview state the same rule for the
    // same reason, and a clipboard that disagreed with both would be the one
    // surface showing the file as it was on disk.
    let view = doc.session.view();

    let svg = pdfcer_render::svg::export_svg_view(
        &view,
        page,
        options,
        // Outlines on the clipboard: Word's importer ignores embedded fonts.
        &pdfcer_render::svg::SvgOptions::default()
            .with_raster_dpi(COPY_DPI)
            .with_background(None)
            .with_text(pdfcer_render::svg::SvgText::Outlines),
    )
    .map_err(|error| Refusal::Render(error.to_string()))?;
    let emf = pdfcer_render::emf::export_emf_view(
        &view,
        page,
        options,
        // Outlines on the clipboard: an EMF cannot carry the font.
        &pdfcer_render::emf::EmfOptions::default()
            .with_raster_dpi(COPY_DPI)
            .with_background(None)
            .with_text(pdfcer_render::emf::EmfText::Outlines),
    )
    .map_err(|error| Refusal::Render(error.to_string()))?;
    let rendered = pdfcer_render::render_page_with_view(
        &view,
        page,
        crate::app::actions::imageexport::scale_for(COPY_DPI),
        options,
    )
    .map_err(|error| Refusal::Render(error.to_string()))?;

    raster_into(svg.svg, emf.emf, rendered.pixmap)
}

/// The **selected page objects**, if any are selected.
fn selection_payload(
    doc: &crate::app::state::OpenDoc,
    options: &pdfcer_render::RenderOptions,
) -> Option<Result<CopyPayload, Refusal>> {
    let page = doc.view.page_index;
    let objects = doc.selection.object_indices_on(page);
    if objects.is_empty() {
        return None;
    }
    Some(selection_bytes(doc, page, &objects, options))
}

/// The selection route's body, split out so [`selection_payload`] is the
/// three-line question *"is there a selection?"* and this is the answer.
fn selection_bytes(
    doc: &crate::app::state::OpenDoc,
    page: usize,
    objects: &[usize],
    options: &pdfcer_render::RenderOptions,
) -> Result<CopyPayload, Refusal> {
    // `&self`, and it commits nothing — the same call `canvas::clipboard`'s
    // content copy makes, for the same reason: a copy is not an edit.
    let clip = doc
        .session
        .copy_objects(page, objects)
        .map_err(|error| Refusal::Render(error.to_string()))?;

    // `ObjectClip::to_pdf` returns a **standalone one-page document whose
    // `/MediaBox` is the selection's bounds**, which is the whole reason this
    // route is worth having: what lands in Word is the selected line-work at
    // its own size, not the selection floating inside a page-sized rectangle
    // of empty space. `canvas::clipimage` documents the same property from the
    // raster side.
    let pdf = clip.to_pdf();
    let clipped = pdfcer_core::document::Document::from_bytes(pdf.bytes)
        .map_err(|error| Refusal::Render(error.to_string()))?;
    let pages = pdfcer_core::page_tree::pages(&clipped)
        .map_err(|error| Refusal::Render(error.to_string()))?;
    let Some(page) = pages.first() else {
        return Err(Refusal::NoPage);
    };

    // The SAME options the page route uses — see [`render_options`] for why
    // the clip is not exempt from the settings funnel even though it carries no
    // annotations and no layers for two of those fields to describe.
    let svg = pdfcer_render::svg::export_svg(
        &clipped,
        page,
        options,
        // Outlines on the clipboard: Word's importer ignores embedded fonts.
        &pdfcer_render::svg::SvgOptions::default()
            .with_raster_dpi(COPY_DPI)
            .with_background(None)
            .with_text(pdfcer_render::svg::SvgText::Outlines),
    )
    .map_err(|error| Refusal::Render(error.to_string()))?;
    let emf = pdfcer_render::emf::export_emf(
        &clipped,
        page,
        options,
        // Outlines on the clipboard: an EMF cannot carry the font.
        &pdfcer_render::emf::EmfOptions::default()
            .with_raster_dpi(COPY_DPI)
            .with_background(None)
            .with_text(pdfcer_render::emf::EmfText::Outlines),
    )
    .map_err(|error| Refusal::Render(error.to_string()))?;
    // `render_page_with`, the four-argument form, NOT the three-argument
    // `render_page` that `canvas::clipimage` uses on the same clip. That one
    // makes a thumbnail for an internal paste, where the operator's colour
    // settings genuinely do not matter; this raster is what a foreign
    // application pastes when it cannot read either vector entry, and it must
    // match the SVG placed above it.
    let rendered = pdfcer_render::render_page_with(
        &clipped,
        page,
        crate::app::actions::imageexport::scale_for(COPY_DPI),
        options,
    )
    .map_err(|error| Refusal::Render(error.to_string()))?;

    raster_into(svg.svg, emf.emf, rendered.pixmap)
}

/// Assemble the three products into a [`CopyPayload`], encoding the PNG.
fn raster_into(
    svg: String,
    emf: Vec<u8>,
    pixmap: pdfcer_render::tiny_skia::Pixmap,
) -> Result<CopyPayload, Refusal> {
    let png = pdfcer_render::export::encode_png(&pixmap, Some(COPY_DPI))
        .map_err(|error| Refusal::Render(error.to_string()))?;
    Ok(CopyPayload {
        svg: Some(svg),
        emf: Some(emf),
        png: Some(png),
        pixmap: Some(pixmap),
        pixels_per_metre: pixels_per_metre(COPY_DPI),
    })
}

/// One entry, framed and ready to place.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Staged {
    /// Which format these bytes are for.
    format: ClipFormat,
    /// The bytes exactly as they should reach the clipboard — terminator,
    /// header and all.
    bytes: Vec<u8>,
}

/// **Frame the payload into ordered entries, or refuse the whole thing.**
fn staged(payload: &CopyPayload) -> Result<Vec<Staged>, Refusal> {
    // THE GATE, and it is asked FIRST — before a single byte is framed.
    //
    // `degrades_word_to_a_picture` is a property of what was PRODUCED, and this
    // is the last point at which the answer can still be "then place nothing".
    if payload.degrades_word_to_a_picture() {
        return Err(Refusal::WouldDegrade);
    }
    if payload.is_empty() {
        return Err(Refusal::NoPage);
    }

    // Driven by `ORDER`, never by the struct's field order. `CopyPayload`'s
    // own `formats()` makes the same choice and states the reason: a second
    // list is a second answer, and the one that goes stale is whichever is not
    // the one being read at the time.
    let mut out = Vec::with_capacity(ORDER.len());
    for format in ORDER {
        let bytes = match format {
            ClipFormat::Svg => payload.svg.as_deref().map(svg_payload),
            ClipFormat::Emf => payload.emf.clone(),
            ClipFormat::Png => payload.png.clone(),
            ClipFormat::DibV5 => payload
                .pixmap
                .as_ref()
                .map(|pixmap| dib_v5(pixmap, payload.pixels_per_metre)),
        };
        if let Some(bytes) = bytes {
            out.push(Staged { format, bytes });
        }
    }
    Ok(out)
}

/// Place a payload, returning the format names that landed.
fn place(payload: &CopyPayload) -> Result<Vec<&'static str>, Refusal> {
    let staged = staged(payload)?;
    let entries: Vec<native_clipboard::Entry<'_>> = staged
        .iter()
        .map(|item| native_clipboard::Entry {
            name: item.format.name(),
            slot: slot_for(item.format),
            bytes: &item.bytes,
        })
        .collect();
    native_clipboard::place(&entries).map_err(Refusal::Clipboard)
}

/// **How each format's bytes become a clipboard handle.**
fn slot_for(format: ClipFormat) -> native_clipboard::Slot {
    match format {
        // The two whose names do not exist until `RegisterClipboardFormat` has
        // been called. `ClipFormat::is_registered` says the same thing from the
        // other side, and the tests assert the two against each other.
        ClipFormat::Svg | ClipFormat::Png => native_clipboard::Slot::Registered,
        // NOT `Predefined(CF_ENHMETAFILE)`. A metafile is a GDI handle, and
        // handing the clipboard an `HGLOBAL` under that id is undefined rather
        // than refused — see `Slot::EnhMetaFile`.
        ClipFormat::Emf => native_clipboard::Slot::EnhMetaFile,
        ClipFormat::DibV5 => native_clipboard::Slot::Predefined(native_clipboard::CF_DIBV5),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_render::tiny_skia::Pixmap;

    /// A payload with all four formats present, for the ordering assertions.
    fn full() -> CopyPayload {
        let mut pixmap = Pixmap::new(2, 1).expect("2x1 is a valid pixmap");
        pixmap.fill(pdfcer_render::tiny_skia::Color::from_rgba8(255, 0, 0, 255));
        CopyPayload {
            svg: Some("<svg/>".to_owned()),
            emf: Some(vec![1, 2, 3, 4]),
            png: Some(vec![5, 6, 7, 8]),
            pixmap: Some(pixmap),
            pixels_per_metre: 5906,
        }
    }

    /// **The entries come out in the measured order, whatever order the
    /// payload's fields were filled in.**
    #[test]
    fn the_entries_are_framed_in_the_measured_order() {
        let staged = staged(&full()).expect("a full payload places");
        assert_eq!(
            staged.iter().map(|s| s.format).collect::<Vec<_>>(),
            vec![
                ClipFormat::Svg,
                ClipFormat::Emf,
                ClipFormat::Png,
                ClipFormat::DibV5
            ],
            "SVG first is what makes Word store an svgBlip; EMF second is \
             LibreOffice 24.x's only vector route. The order is MEASURED."
        );
    }

    /// **A raster-only payload is refused, and nothing is framed.**
    #[test]
    fn a_raster_only_payload_is_refused_rather_than_half_placed() {
        let raster_only = CopyPayload {
            png: Some(vec![1, 2, 3]),
            ..CopyPayload::default()
        };
        assert_eq!(
            staged(&raster_only),
            Err(Refusal::WouldDegrade),
            "placing only the raster formats degrades Word's paste to a plain \
             picture — refusing keeps whatever the operator had copied"
        );
        // …and an empty payload is refused too, rather than reported as a
        // successful copy of nothing.
        assert_eq!(staged(&CopyPayload::default()), Err(Refusal::NoPage));
    }

    /// **The SVG entry carries its trailing NUL and the DIB its 124-byte
    /// header** — the framing is applied at this boundary and nowhere else.
    #[test]
    fn the_framing_is_applied_where_the_bytes_are_staged() {
        let staged = staged(&full()).expect("a full payload places");
        let svg = &staged[0];
        assert_eq!(svg.format, ClipFormat::Svg);
        assert_eq!(svg.bytes, b"<svg/>\0", "one trailing NUL, added here");

        let emf = &staged[1];
        assert_eq!(emf.bytes, vec![1, 2, 3, 4], "the metafile goes on verbatim");

        let dib = &staged[3];
        assert_eq!(dib.format, ClipFormat::DibV5);
        assert_eq!(dib.bytes.len(), 124 + 2 * 4, "header plus two BGRA pixels");
        assert_eq!(
            u32::from_le_bytes([dib.bytes[0], dib.bytes[1], dib.bytes[2], dib.bytes[3]]),
            124,
            "the DIB is assembled at this boundary, not carried in the payload"
        );
        assert_eq!(
            i32::from_le_bytes([dib.bytes[24], dib.bytes[25], dib.bytes[26], dib.bytes[27]]),
            5906,
            "the payload's pixels-per-metre reaches the header"
        );
    }

    /// **A payload missing one format still places the rest, in order** —
    /// as long as a vector survives.
    #[test]
    fn a_missing_format_is_skipped_and_the_rest_keep_their_order() {
        let mut payload = full();
        payload.emf = None;
        let staged = staged(&payload).expect("SVG still leads");
        assert_eq!(
            staged.iter().map(|s| s.format).collect::<Vec<_>>(),
            vec![ClipFormat::Svg, ClipFormat::Png, ClipFormat::DibV5],
            "the EMF's absence must not reorder the three that remain"
        );
    }

    /// **The two registered formats are exactly the two `ClipFormat` says
    /// are registered**, so the shell's vocabulary and Win32's cannot drift.
    #[test]
    fn the_registered_slots_match_the_registered_formats() {
        for format in ORDER {
            let registered = matches!(slot_for(format), native_clipboard::Slot::Registered);
            assert_eq!(
                registered,
                format.is_registered(),
                "{format:?}: the slot `slot_for` actually chooses and \
                 `is_registered` must agree, or the format goes on under the \
                 wrong id \u{2014} which looks like a successful copy from in here"
            );
        }
        // And the metafile takes the HANDLE slot, not a predefined id. It is
        // the one entry whose bytes are converted before placement, and
        // `Predefined(CF_ENHMETAFILE)` would hand GDI an `HGLOBAL`.
        assert!(matches!(
            slot_for(ClipFormat::Emf),
            native_clipboard::Slot::EnhMetaFile
        ));
        assert_eq!(
            slot_for(ClipFormat::DibV5),
            native_clipboard::Slot::Predefined(native_clipboard::CF_DIBV5)
        );
    }
}
