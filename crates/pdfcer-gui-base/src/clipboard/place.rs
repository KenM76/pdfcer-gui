//! # `clipboard::place` — producing the payload from a document, and placing it
//!
//! [`super`] builds the bytes and states the order. This module produces the
//! payload from a real document and hands the ordered set to
//! `native_clipboard::place`, which is the crate that owns the `unsafe`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/clipboard/place.md`.

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
pub fn copy_out(doc: &crate::opendoc::OpenDoc) -> Result<Placed, Refusal> {
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
pub(super) fn render_options(doc: &crate::opendoc::OpenDoc) -> pdfcer_render::RenderOptions {
    use crate::settings::SettingsExt;
    let mut options = doc
        .settings
        .render_options()
        .with_backdrop(pdfcer_render::PageBackdrop::Transparent);
    options.annotations = doc.annotations_visible();
    options.layers = doc.layer_visibility();
    options
}

/// The SVG writer's clipboard options at `dpi`: outlines, because Word's
/// importer ignores embedded fonts, and no background.
pub(super) fn svg_options(dpi: f32) -> pdfcer_render::svg::SvgOptions {
    pdfcer_render::svg::SvgOptions::default()
        .with_raster_dpi(dpi)
        .with_background(None)
        .with_text(pdfcer_render::svg::SvgText::Outlines)
}

/// The EMF writer's clipboard options at `dpi`: outlines, because an EMF
/// cannot carry the font, and no background.
pub(super) fn emf_options(dpi: f32) -> pdfcer_render::emf::EmfOptions {
    pdfcer_render::emf::EmfOptions::default()
        .with_raster_dpi(dpi)
        .with_background(None)
        .with_text(pdfcer_render::emf::EmfText::Outlines)
}

/// The whole current page, in every format [`ORDER`] names.
fn page_payload(
    doc: &crate::opendoc::OpenDoc,
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

    let svg = pdfcer_render::svg::export_svg_view(&view, page, options, &svg_options(COPY_DPI))
        .map_err(|error| Refusal::Render(error.to_string()))?;
    let emf = pdfcer_render::emf::export_emf_view(&view, page, options, &emf_options(COPY_DPI))
        .map_err(|error| Refusal::Render(error.to_string()))?;
    let rendered = pdfcer_render::render_page_with_view(
        &view,
        page,
        crate::imageexport::scale_for(COPY_DPI),
        options,
    )
    .map_err(|error| Refusal::Render(error.to_string()))?;

    raster_into(svg.svg, emf.emf, rendered.pixmap)
}

/// The **selected page objects**, if any are selected.
fn selection_payload(
    doc: &crate::opendoc::OpenDoc,
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
    doc: &crate::opendoc::OpenDoc,
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
    let svg = pdfcer_render::svg::export_svg(&clipped, page, options, &svg_options(COPY_DPI))
        .map_err(|error| Refusal::Render(error.to_string()))?;
    let emf = pdfcer_render::emf::export_emf(&clipped, page, options, &emf_options(COPY_DPI))
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
        crate::imageexport::scale_for(COPY_DPI),
        options,
    )
    .map_err(|error| Refusal::Render(error.to_string()))?;

    raster_into(svg.svg, emf.emf, rendered.pixmap)
}

/// Assemble the three products into a [`CopyPayload`] at [`COPY_DPI`].
fn raster_into(
    svg: String,
    emf: Vec<u8>,
    pixmap: pdfcer_render::tiny_skia::Pixmap,
) -> Result<CopyPayload, Refusal> {
    raster_at(svg, emf, pixmap, COPY_DPI)
}

/// Assemble the three products into a [`CopyPayload`], encoding the PNG and
/// stamping both rasters with `dpi`.
pub(super) fn raster_at(
    svg: String,
    emf: Vec<u8>,
    pixmap: pdfcer_render::tiny_skia::Pixmap,
    dpi: f32,
) -> Result<CopyPayload, Refusal> {
    let png = pdfcer_render::export::encode_png(&pixmap, Some(dpi))
        .map_err(|error| Refusal::Render(error.to_string()))?;
    Ok(CopyPayload {
        svg: Some(svg),
        emf: Some(emf),
        png: Some(png),
        pixmap: Some(pixmap),
        pixels_per_metre: pixels_per_metre(dpi),
        pdf: None,
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
    Ok(frame(payload))
}

/// Frame every format the payload has, in [`ORDER`].
fn frame(payload: &CopyPayload) -> Vec<Staged> {
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
            ClipFormat::Pdf => payload.pdf.clone(),
        };
        if let Some(bytes) = bytes {
            out.push(Staged { format, bytes });
        }
    }
    out
}

/// Place a payload, returning the format names that landed.
pub(super) fn place(payload: &CopyPayload) -> Result<Vec<&'static str>, Refusal> {
    put(&staged(payload)?)
}

/// Place a payload the caller has deliberately reduced to its picture, with
/// the reason disclosed by the caller: [`staged`]'s would-degrade gate is
/// for a copy that lost its vectors by accident.
pub(super) fn place_withheld(payload: &CopyPayload) -> Result<Vec<&'static str>, Refusal> {
    if payload.is_empty() {
        return Err(Refusal::NoPage);
    }
    put(&frame(payload))
}

/// The environment variable that sends a copy-out to a folder instead of the
/// clipboard, so a driven check can read the payload without replacing the
/// operator's clipboard.
pub const DIAG_CLIPBOARD_DIR: &str = "PDFCER_DIAG_CLIPBOARD_DIR"; // ui-text-exempt: an environment variable name, never displayed

/// Hand framed entries to the clipboard.
fn put(staged: &[Staged]) -> Result<Vec<&'static str>, Refusal> {
    put_entries(&entries(staged))
}

/// The clipboard entries for framed payloads, in the same order.
fn entries(staged: &[Staged]) -> Vec<native_clipboard::Entry<'_>> {
    staged
        .iter()
        .map(|item| native_clipboard::Entry {
            name: item.format.name(),
            slot: slot_for(item.format),
            bytes: &item.bytes,
        })
        .collect()
}

/// Hand `entries` to the operating system's clipboard in one transaction, or
/// to [`DIAG_CLIPBOARD_DIR`] when it is set. Returns the names that landed.
///
/// # Errors
///
/// The clipboard's [`native_clipboard::PlaceError`], or its `Stage` refusal
/// for a capture file that cannot be written.
pub fn put_entries(entries: &[native_clipboard::Entry<'_>]) -> Result<Vec<&'static str>, Refusal> {
    if let Some(dir) = std::env::var_os(DIAG_CLIPBOARD_DIR) {
        return capture(std::path::Path::new(&dir), entries);
    }
    native_clipboard::place(entries).map_err(Refusal::Clipboard)
}

/// Under [`DIAG_CLIPBOARD_DIR`], the bytes the last capture wrote for the
/// format `name`; `None` when the variable is unset or no such file exists.
/// The reading half of a capture, so a driven check can paste what another
/// window copied without either touching the operator's clipboard.
#[must_use]
pub fn captured(name: &str) -> Option<Vec<u8>> {
    let dir = std::env::var_os(DIAG_CLIPBOARD_DIR)?;
    let suffix = format!("-{}.bin", file_stem(name));
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .find(|e| e.file_name().to_string_lossy().ends_with(&suffix))
        .and_then(|e| std::fs::read(e.path()).ok())
}

/// Write each entry to `<dir>/<n>-<format>.bin`, `n` counting from 1 in
/// placement order, and return the names as a placement would. The OS
/// clipboard is not opened. A failed write is the clipboard's `Stage` refusal
/// for that format, the same one a handle that cannot be made gives.
fn capture(
    dir: &std::path::Path,
    entries: &[native_clipboard::Entry<'_>],
) -> Result<Vec<&'static str>, Refusal> {
    let Some(first) = entries.first() else {
        return Err(Refusal::Clipboard(native_clipboard::PlaceError::Nothing));
    };
    let stage = |name| Refusal::Clipboard(native_clipboard::PlaceError::Stage(name));
    std::fs::create_dir_all(dir).map_err(|_| stage(first.name))?;
    let mut names = Vec::with_capacity(entries.len());
    for (n, item) in entries.iter().enumerate() {
        let file = dir.join(format!("{}-{}.bin", n + 1, file_stem(item.name)));
        std::fs::write(&file, item.bytes).map_err(|_| stage(item.name))?;
        names.push(item.name);
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "clipboard-captured dir={} formats={}",
            dir.display(),
            names.len()
        )
    });
    Ok(names)
}

/// A format name with the characters a file name cannot hold replaced:
/// `image/svg+xml` becomes `image_svg_xml`.
fn file_stem(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

/// **How each format's bytes become a clipboard handle.**
fn slot_for(format: ClipFormat) -> native_clipboard::Slot {
    match format {
        // The three whose names do not exist until `RegisterClipboardFormat` has
        // been called. `ClipFormat::is_registered` says the same thing from the
        // other side, and the tests assert the two against each other.
        ClipFormat::Svg | ClipFormat::Png | ClipFormat::Pdf => native_clipboard::Slot::Registered,
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
            pdf: None,
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

    /// **The registered slots are exactly the formats `ClipFormat` says
    /// are registered**, so the shell's vocabulary and Win32's cannot drift.
    #[test]
    fn a_capture_writes_each_entry_in_placement_order() {
        let dir = std::env::temp_dir().join(format!("pdfcer-capture-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let framed = frame(&full());
        let names = capture(&dir, &entries(&framed)).expect("captured");
        assert_eq!(
            names,
            framed.iter().map(|e| e.format.name()).collect::<Vec<_>>()
        );
        let mut files: Vec<String> = std::fs::read_dir(&dir)
            .expect("dir")
            .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
            .collect();
        files.sort();
        assert_eq!(
            files.first().map(String::as_str),
            Some("1-image_svg_xml.bin")
        );
        assert_eq!(files.len(), names.len());
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(
            capture(&dir, &[]),
            Err(Refusal::Clipboard(native_clipboard::PlaceError::Nothing))
        );
    }

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
