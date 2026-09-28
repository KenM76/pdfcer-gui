//! # `app::actions::export` — writing part of the document out as something
//! else
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/export.md`.

use crate::app::state::OpenDoc;

/// Write one page's vector geometry as an ASCII DXF.
pub(super) fn dxf(doc: &mut OpenDoc, page: usize, options: &pdfcer_core::export::dxf::DxfOptions) {
    // The decomposition, from the cache the canvas and the Objects panel share.
    // `None` is reachable — a page still being read, or one whose content
    // streams could not be resolved — and is a decline rather than a failure:
    // nothing was written and the sentence says which.
    let Some(dxf) = doc
        .page_objects()
        .map(|provider| pdfcer_core::export::dxf::write_dxf(provider.page_objects(), options))
    else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("export-dxf-declined page={page} reason=no-decomposition")
        });
        super::record_note(
            doc.edit_epoch,
            crate::text::export_dxf::no_geometry().to_owned(),
        );
        return;
    };
    let (text, outcome) = dxf;

    // The picker AFTER the write, not before.
    //
    // The write is pure and cannot fail — `write_dxf` returns no `Result`, and
    // its doc says why: *"the writer cannot fail on well-formed input, and
    // malformed input is skipped and counted rather than refused."* So doing it
    // first costs nothing and buys the property that matters: the operator is
    // never asked where to put a file that turns out to be empty. If a future
    // slice gives the writer a refusal, this ordering is what lets the refusal
    // be reported before a save dialog has been opened.
    let suggested = suggested_path(doc);
    let crate::app::files::Picked::Path(target) =
        crate::app::files::pick_save_path(&suggested, crate::text::export_dxf::save_dialog_title())
    else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("export-dxf-cancelled page={page}")
        });
        return;
    };

    match std::fs::write(&target, text.as_bytes()) {
        Ok(()) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "export-dxf page={page} bytes={} polylines={} circles={} arcs={} \
                     splines={} skipped_text={} skipped_images={} unreadable_text={} \
                     version={} splines_flattened={} units_undeclared={}",
                    text.len(),
                    outcome.polylines,
                    outcome.circles,
                    outcome.arcs,
                    outcome.splines,
                    outcome.skipped_text,
                    outcome.skipped_images,
                    outcome.unreadable_text,
                    options.version.acadver(),
                    outcome.splines_flattened,
                    u8::from(outcome.units_undeclared)
                )
            });
            // Recorded through `record_note` rather than returned from a
            // `vector_edit` closure, because there is no edit to ride in on —
            // the same case `canvas::interact` records for a caret that cannot
            // be placed. Stamped with the CURRENT epoch, so the sentences stand
            // until the next real edit moves past them.
            //
            // The list is joined rather than recorded one at a time: the slot
            // holds one disclosure, and the last writer would win.
            let notes = crate::text::export_dxf::exported(
                &target.display().to_string(),
                &outcome,
                options.units,
            );
            super::record_edit_disclosure(Some(super::EditDisclosure {
                epoch: doc.edit_epoch,
                notes,
            }));
        }
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("export-dxf-failed page={page} detail={error}")
            });
            super::record_note(
                doc.edit_epoch,
                crate::text::export_dxf::export_failed(&error.to_string()),
            );
        }
    }
}

/// **Write the form's values out as FDF, XFDF or CSV.**
pub(super) fn form_data(doc: &mut OpenDoc) {
    // The data first, the picker second — `dxf`'s ordering and its reason:
    // the operator is never asked where to put a file that turns out to be
    // empty. A document with no `/AcroForm` has nothing to export, and that is
    // a decline with a sentence rather than a dialog followed by a zero-byte
    // file.
    let Some(data) = doc.session.export_form_data() else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            "export-form-data-declined reason=no-acroform".to_owned()
        });
        super::record_note(
            doc.edit_epoch,
            crate::text::export_form::no_form().to_owned(),
        );
        return;
    };
    if data.fields.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            "export-form-data-declined reason=no-fields".to_owned()
        });
        super::record_note(
            doc.edit_epoch,
            crate::text::export_form::no_fields().to_owned(),
        );
        return;
    }

    let suggested = suggested_form_path(doc);
    let crate::app::files::Picked::Path(target) = crate::app::files::pick_save_path(
        &suggested,
        crate::text::export_form::save_dialog_title(),
    ) else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            "export-form-data-cancelled".to_owned()
        });
        return;
    };

    // The document's own path as the FDF `/F` source, so a reader importing
    // the data knows which file it came from. `to_fdf`'s parameter is exactly
    // that, and passing `None` would produce a valid file that has forgotten
    // its subject.
    let source = doc.path.to_string_lossy().into_owned();
    let extension = target
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    let (bytes, mut notes) = match extension.as_str() {
        // ui-text-exempt: file extensions, matched not displayed.
        "xfdf" => (
            data.to_xfdf(Some(&source)),
            vec![crate::text::export_form::wrote_xfdf(data.fields.len())],
        ),
        "csv" => {
            let export = pdfcer_core::formcsv::to_csv(&data);
            let mut notes = vec![crate::text::export_form::wrote_csv(data.fields.len())];
            // The neutralisation disclosure — see the header. Reported only
            // when it fired, because a form with no formula-shaped values owes
            // no sentence, and a bar that narrates non-events stops being read.
            if export.neutralised > 0 {
                notes.push(crate::text::export_form::neutralised(
                    export.neutralised,
                    &export.neutralised_fields,
                ));
            }
            (export.csv, notes)
        }
        _ => (
            data.to_fdf(Some(&source)),
            vec![crate::text::export_form::wrote_fdf(data.fields.len())],
        ),
    };

    match std::fs::write(&target, &bytes) {
        Ok(()) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "export-form-data format={extension} fields={} bytes={}",
                    data.fields.len(),
                    bytes.len()
                )
            });
            notes.push(crate::text::export_form::written_to(
                &target.display().to_string(),
            ));
            super::record_edit_disclosure(Some(super::EditDisclosure {
                epoch: doc.edit_epoch,
                notes,
            }));
        }
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("export-form-data-failed detail={error}")
            });
            super::record_note(
                doc.edit_epoch,
                crate::text::export_form::export_failed(&error.to_string()),
            );
        }
    }
}

/// Where the form-data save dialog opens, and what it calls the file.
fn suggested_form_path(doc: &OpenDoc) -> std::path::PathBuf {
    let mut path = doc.path.clone();
    let stem = path
        .file_stem()
        .map_or_else(|| "form".to_owned(), |s| s.to_string_lossy().into_owned());
    // `set_file_name`, NOT `set_extension` — see [`suggested_path`] below,
    // which carries the whole argument. `set_extension` replaces everything
    // after the LAST dot, so a `plan.rev2.pdf` loses its revision here too.
    path.set_file_name(format!("{stem}.fdf")); // ui-text-exempt: a file extension, never displayed as prose
    path
}

/// Where the save dialog opens, and what it calls the file.
fn suggested_path(doc: &OpenDoc) -> std::path::PathBuf {
    let mut path = doc.path.clone();
    let stem = path
        .file_stem()
        .map_or_else(|| "export".to_owned(), |s| s.to_string_lossy().into_owned());
    // **NOT `set_extension`, AND THE DIFFERENCE IS DATA LOSS.**
    //
    // The reasoning that makes `set_extension` look safe — *"a document called
    // `plan.rev2.pdf` has a stem of `plan.rev2`, and appending would produce
    // `plan.rev2.dxf` either way"* — is true in its first clause and does not
    // reach its conclusion. `Path::set_extension` replaces everything after
    // the **last** dot, and `plan.rev2` has one, so that call produces
    // **`plan.dxf`** and the revision is dropped.
    //
    // ⇒ Why that is a data-loss defect rather than a cosmetic one:
    // `plan.rev2.pdf` and `plan.rev3.pdf` would both suggest `plan.dxf`, so
    // exporting the second **overwrites the first**, in a save dialog whose
    // only warning is the operating system's generic "a file with that name
    // already exists". `.rev2` / `.rev3` is an ordinary CAD naming shape, and
    // the two files that collide are the two the operator is most likely to
    // want side by side. **A claim in a comment is not a test** — the test is
    // `imageexport::tests::a_dotted_document_name_keeps_its_revision`.
    //
    // `set_file_name` with the stem interpolated appends unconditionally. A
    // document with no extension at all still gains one, because the stem of
    // `plan` is `plan`.
    path.set_file_name(format!("{stem}.dxf")); // ui-text-exempt: a file extension, never displayed as prose
    path
}

/// **Write one or more pages out as PNG, JPEG or SVG** —
/// `OPERATOR_REQUESTS.md` **O120**, and the third member of this module's
/// family.
pub(super) fn image(doc: &mut OpenDoc, plan: &crate::app::actions::imageexport::ImagePlan) {
    use crate::app::actions::imageexport;
    use crate::app::settings::SettingsExt;
    use crate::text::export_image as t;

    // First, before the picker and before the render. See the header.
    if let Some(why) = plan.impossible() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("export-image-refused reason={why:?}")
        });
        super::record_note(doc.edit_epoch, t::refused(why).to_owned());
        return;
    }
    if plan.pages.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            "export-image-declined reason=no-pages".to_owned()
        });
        super::record_note(doc.edit_epoch, t::no_pages().to_owned());
        return;
    }

    let suggested = imageexport::suggested_path(&doc.path, plan.format);
    let crate::app::files::Picked::Path(chosen) =
        crate::app::files::pick_save_path(&suggested, t::save_dialog_title())
    else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            "export-image-cancelled".to_owned()
        });
        return;
    };

    // Through the settings funnel, never `RenderOptions::default()`.
    //
    // `crate::app::settings::SettingsExt` is the one place that turns the
    // operator's configuration into render options, and a `syn` check in that
    // module fails the build if any other file constructs these itself. The
    // five settings it applies — CMYK intent, mask resampling, minification,
    // JPEG polarity, missing appearance state — are exactly the ones that
    // decide what the exported picture LOOKS like, so an export that skipped
    // the funnel would disagree with the canvas the operator was looking at.
    //
    // The annotation stance and the layer overrides come from the DOCUMENT,
    // which is what makes this export *a picture of what you can see*. An
    // operator who has hidden a layer and turned annotations off is looking at
    // a drawing, and the file they asked for is a picture of that drawing
    // rather than of the one underneath it. Neither is a control this window
    // offers, deliberately: both are already offered on the ribbon, against a
    // canvas that shows the answer immediately.
    let mut options = doc
        .settings
        .render_options()
        .with_backdrop(if plan.transparent {
            pdfcer_render::PageBackdrop::Transparent
        } else {
            pdfcer_render::PageBackdrop::White
        });
    options.annotations = doc.annotations_visible();
    options.layers = doc.layer_visibility();

    let multi = plan.is_multi_file();
    let scale = imageexport::scale_for(plan.dpi);
    // `session.view()`, NOT `session.document()` — the view composes the
    // overlay and the staging buffer, so unsaved edits are what gets exported.
    // The print preview states the same rule for the same reason.
    let view = doc.session.view();

    let mut written: Vec<std::path::PathBuf> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let mut first_line: Option<String> = None;
    for &page_index in &plan.pages {
        let Some(page) = doc.pages.get(page_index) else {
            continue;
        };
        let target = imageexport::output_path(&chosen, plan.format, page_index, multi);
        let number = page_index.saturating_add(1);

        // A `match` on the format, NOT `if plan.format.is_vector()`.
        //
        // It was the `if` until EMF arrived, and the `if` would have written
        // every EMF export as an SVG — a file with the right extension, the
        // wrong bytes, and nothing anywhere to say so. `is_vector` answers
        // *"does the resolution mean a recording scale"*, which is a question
        // about the hint text; it stopped being a synonym for *"which writer"*
        // the moment there were two vector writers.
        //
        // ⇒ Matching means a fifth format is a compile error here rather than
        // a silent routing into whichever branch a predicate happens to pick.
        let produced = match plan.format {
            imageexport::ImageFormat::Svg => svg_bytes(&view, page, &options, plan),
            imageexport::ImageFormat::Emf => emf_bytes(&view, page, &options, plan),
            imageexport::ImageFormat::Png | imageexport::ImageFormat::Jpeg => {
                raster_bytes(&view, page, scale, &options, plan)
            }
        };
        let produced = match produced {
            Ok(produced) => produced,
            // One page's failure STOPS the run rather than skipping on.
            //
            // The alternative — carry on and summarise at the end — leaves the
            // operator with a directory of files and a sentence about a gap
            // somewhere in it. A run that stops names the page it stopped on,
            // and every file written before it is on disk and named in the same
            // disclosure.
            //
            // The two failures are told APART, and that is not decoration.
            // *"Could not be drawn"* is about the page and will happen again
            // whatever format is chosen; *"could not be written as this
            // format"* is about the encoder, and its commonest cause —
            // `ExportError::TooLargeForJpeg`, a raster over 65,535 pixels on a
            // side, which is JPEG's 16-bit dimension field (ITU-T T.81 §B.2.2)
            // — is fixed by choosing PNG or lowering the resolution. Rolling
            // the two together would send an operator whose only problem is a
            // format limit off to investigate their drawing.
            Err(Failed::Render(detail)) => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!("export-image-render-failed page={page_index} detail={detail}")
                });
                notes.push(t::render_failed(number, &detail));
                break;
            }
            Err(Failed::Encode(detail)) => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!("export-image-encode-failed page={page_index} detail={detail}")
                });
                notes.push(t::encode_failed(number, &detail));
                break;
            }
        };

        if let Err(error) = std::fs::write(&target, &produced.bytes) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("export-image-write-failed page={page_index} detail={error}")
            });
            notes.push(t::write_failed(&error.to_string()));
            break;
        }
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "export-image page={page_index} format={} bytes={} dpi={} transparent={}",
                // The TOKEN, not `{:?}`, and for the reason the window
                // states beside its own trace: this line and
                // `export-image-requested` describe ONE format, forty lines
                // apart in one export, and `{:?}` would spell it `Emf`
                // here and `emf` there. A check that cross-checks the plan
                // against the file — the obvious next one to write here, and
                // `export_image_emf` already reads a field from each — would
                // read those two as unequal and report that the window asked
                // for one format and the writer produced another. All three
                // readers (this, the window, `preferences.txt`) share one
                // vocabulary: `prefs::exporting::image_format_key`.
                crate::app::prefs::exporting::image_format_key(plan.format),
                produced.bytes.len(),
                plan.dpi,
                u8::from(plan.transparent)
            )
        });

        // Only the FIRST page's fidelity notes are kept.
        //
        // The alternative is fifty copies of *"text is written as outlines"* in
        // one status line, which is a disclosure nobody reads — and Rule 4's
        // whole value is in being read. What differs between pages is the
        // counters; what does not is the standing truth (outlines, the
        // resolution, the background), and that is the half an operator acts
        // on. A per-page report belongs in a panel, and there is not one.
        if written.is_empty() {
            first_line = Some(match produced.kind {
                Produced::Raster { width, height } => t::wrote_raster(
                    &target.display().to_string(),
                    number,
                    width,
                    height,
                    plan.dpi,
                ),
                Produced::Vector { ops } => {
                    t::wrote_svg(&target.display().to_string(), number, ops)
                }
                Produced::Metafile { ops, rasters } => {
                    t::wrote_emf(&target.display().to_string(), number, ops, rasters)
                }
            });
            notes.extend(produced.notes);
        }
        written.push(target);
    }

    if written.is_empty() {
        // Nothing landed. Whatever went wrong has already pushed its sentence;
        // the fallback covers a plan whose every page index was out of range,
        // which the window cannot produce and a restored plan could.
        if notes.is_empty() {
            notes.push(t::no_pages().to_owned());
        }
        super::record_notes(doc.edit_epoch, notes);
        return;
    }

    // The lead-in goes FIRST — `record_notes`' own rule: *"the first sentence
    // is the one an operator reads if they read only one."* For a single file
    // that is the file's own line; for many it is the count and the range of
    // names, because fifty paths in a status bar is not a sentence.
    let lead = if written.len() > 1 {
        t::wrote_many(
            written.len(),
            &written[0].display().to_string(),
            &written[written.len() - 1].display().to_string(),
        )
    } else {
        first_line.unwrap_or_else(|| t::no_pages().to_owned())
    };
    notes.insert(0, lead);
    super::record_notes(doc.edit_epoch, notes);
}

/// What one page's writer produced: the bytes, the shape of its receipt line,
/// and whatever it has to disclose.
struct Output {
    bytes: Vec<u8>,
    kind: Produced,
    notes: Vec<String>,
}

/// Why a page's writer failed, kept apart because the two failures ask
/// different things of the operator.
enum Failed {
    /// The page could not be rasterised or recorded.
    Render(String),
    /// The bytes existed and the encoder would not take them.
    Encode(String),
}

/// Which receipt line a page's output earns.
enum Produced {
    /// Pixels — the line names the pixel count and the resolution recorded in
    /// the file.
    Raster { width: u32, height: u32 },
    /// Geometry — the line names the drawing operations, which is the only
    /// honest size measure a vector file has.
    Vector { ops: usize },
    /// Geometry **and** pictures, which is what a metafile always is.
    ///
    /// A separate variant rather than reusing [`Self::Vector`], because the
    /// receipt has a second number to give: `ops` is only the part that
    /// stayed lines, and a metafile that is half `EMR_ALPHABLEND` would
    /// otherwise be described in exactly the same words as one that is all
    /// geometry. See `crate::text::export_image::wrote_emf`.
    Metafile { ops: usize, rasters: usize },
}

/// One page, encoded as PNG or JPEG.
///
fn raster_bytes(
    view: &pdfcer_render::DocumentView<'_>,
    page: &pdfcer_core::page_tree::Page,
    scale: f32,
    options: &pdfcer_render::RenderOptions,
    plan: &crate::app::actions::imageexport::ImagePlan,
) -> Result<Output, Failed> {
    use crate::app::actions::imageexport::ImageFormat;

    let rendered = pdfcer_render::render_page_with_view(view, page, scale, options)
        .map_err(|error| Failed::Render(error.to_string()))?;
    let pixmap = &rendered.pixmap;
    let (width, height) = (pixmap.width(), pixmap.height());

    // `Some(dpi)`, never `None`. The engine's note is unambiguous about
    // what leaving it out costs: *"without `pHYs` Word places a 300 DPI page
    // four times too large."* That is not a metadata nicety — it is the
    // difference between a paste the size of the page and one four times it,
    // and there is no case in which pdfcer knows the resolution and should
    // decline to write it down.
    let bytes = match plan.format {
        ImageFormat::Png => pdfcer_render::export::encode_png(pixmap, Some(plan.dpi)),
        ImageFormat::Jpeg => {
            // `#[non_exhaustive]`: default, then assign. A struct literal will
            // not compile from outside the crate, and that is the engine
            // reserving the right to add a field — which this call site should
            // inherit rather than have to be told about.
            let mut jpeg = pdfcer_render::export::JpegOptions::default();
            jpeg.quality = plan.quality;
            jpeg.dpi = Some(plan.dpi);
            // Reachable only when the operator did NOT ask for transparency: a
            // transparent JPEG was refused before the picker opened, so the
            // render above already came back opaque over white and this
            // composites nothing. Set anyway, because a default that happens to
            // agree is not the same as a decision.
            jpeg.background = pdfcer_render::export::Rgb::WHITE;
            pdfcer_render::export::encode_jpeg(pixmap, &jpeg)
        }
        // Unreachable — the caller matches on the format and sends these two
        // to `svg_bytes` and `emf_bytes`. Written as arms returning the
        // lossless format rather than as `unreachable!`, because a panic
        // inside an export is never the right answer to a fifth variant
        // arriving, and because a PNG on disk under a wrong extension is a
        // file the operator can still open.
        //
        // Note that these arms cost nothing in safety: the caller's own
        // `match` is exhaustive, so a fifth format is a compile error THERE —
        // at the routing decision, which is where it can be answered — and
        // never silently lands here.
        ImageFormat::Svg | ImageFormat::Emf => {
            pdfcer_render::export::encode_png(pixmap, Some(plan.dpi))
        }
    }
    .map_err(|error| Failed::Encode(error.to_string()))?;

    let notes = vec![if plan.transparent {
        crate::text::export_image::transparency_kept().to_owned()
    } else {
        crate::text::export_image::flattened_to_white().to_owned()
    }];
    Ok(Output {
        bytes,
        kind: Produced::Raster { width, height },
        notes,
    })
}

/// One page, recorded as SVG.
fn svg_bytes(
    view: &pdfcer_render::DocumentView<'_>,
    page: &pdfcer_core::page_tree::Page,
    options: &pdfcer_render::RenderOptions,
    plan: &crate::app::actions::imageexport::ImagePlan,
) -> Result<Output, Failed> {
    // The BACKGROUND, not the backdrop. `export_svg_view`'s own doc: *"The
    // backdrop field of `render` is ignored: an SVG's background is
    // `SvgOptions::background`."* Setting one and expecting the other is
    // precisely the shape of mistake that ships a window promising transparency
    // and a file that is opaque — so both are set, from the same flag, and this
    // comment is why the apparent duplication is not one.
    let svg_options = pdfcer_render::svg::SvgOptions::default()
        .with_raster_dpi(plan.dpi)
        .with_background(if plan.transparent {
            None
        } else {
            Some(pdfcer_render::export::Rgb::WHITE)
        })
        .with_text(if plan.keep_text {
            pdfcer_render::svg::SvgText::KeepText
        } else {
            pdfcer_render::svg::SvgText::Outlines
        });
    let export = pdfcer_render::svg::export_svg_view(view, page, options, &svg_options)
        .map_err(|error| Failed::Render(error.to_string()))?;

    let mut notes = vec![if plan.transparent {
        crate::text::export_image::transparency_kept().to_owned()
    } else {
        crate::text::export_image::flattened_to_white().to_owned()
    }];
    // Rule 4's content. The text sentences lead — outlines, or what keep-text
    // kept and why the rest fell back — then every counter the recording had
    // to raise. See `crate::text::export_image` and `export_keeptext`.
    let text = if plan.keep_text {
        trace_svg_text(&export.outcome.text);
        crate::text::export_keeptext::svg_kept(&export.outcome.text)
    } else {
        vec![crate::text::export_image::svg_text_is_outlines().to_owned()]
    };
    notes.extend(crate::text::export_image::svg_fidelity_with(
        text,
        &export.outcome.tally,
        export.outcome.dashed_strokes_pre_applied,
        export.outcome.blend_modes_used,
    ));
    let ops = export.outcome.ops;
    Ok(Output {
        bytes: export.svg.into_bytes(),
        kind: Produced::Vector { ops },
        notes,
    })
}

/// One page, recorded as a Windows Enhanced Metafile ([MS-EMF]).
fn emf_bytes(
    view: &pdfcer_render::DocumentView<'_>,
    page: &pdfcer_core::page_tree::Page,
    options: &pdfcer_render::RenderOptions,
    plan: &crate::app::actions::imageexport::ImagePlan,
) -> Result<Output, Failed> {
    let emf_options = pdfcer_render::emf::EmfOptions::default()
        .with_raster_dpi(plan.dpi)
        .with_background(if plan.transparent {
            None
        } else {
            Some(pdfcer_render::export::Rgb::WHITE)
        })
        .with_text(if plan.keep_text {
            pdfcer_render::emf::EmfText::KeepText
        } else {
            pdfcer_render::emf::EmfText::Outlines
        });
    let export = pdfcer_render::emf::export_emf_view(view, page, options, &emf_options)
        .map_err(|error| Failed::Render(error.to_string()))?;

    let mut notes = vec![if plan.transparent {
        crate::text::export_image::transparency_kept().to_owned()
    } else {
        crate::text::export_image::flattened_to_white().to_owned()
    }];
    // Rule 4's content, through the shell's own `EmfCounts` rather than
    // the engine's `EmfOutcome`. The reason is testability and it is argued in
    // full on `EmfCounts` itself: `EmfOutcome` is `#[non_exhaustive]` with no
    // `Default`, so no test in this crate could ever build one, and the
    // counters-to-sentences mapping is the part of this path most worth
    // testing.
    let counts = crate::app::actions::imageexport::EmfCounts::from(&export.outcome);
    let text = if plan.keep_text {
        trace_emf_text(&export.outcome.text);
        crate::text::export_keeptext::emf_kept(&export.outcome.text)
    } else {
        vec![crate::text::export_image::emf_text_is_outlines().to_owned()]
    };
    notes.extend(crate::text::export_image::emf_fidelity_with(text, &counts));
    Ok(Output {
        bytes: export.emf,
        kind: Produced::Metafile {
            ops: counts.ops,
            rasters: counts.rasters_embedded,
        },
        notes,
    })
}

/// Trace an SVG keep-text outcome, in the engine CLI's `svg-text:` vocabulary.
fn trace_svg_text(o: &pdfcer_render::svg::SvgTextOutcome) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "export-image-svg-text kept={} outlines={} fonts={} not_sfnt={} paint={} \
             unmapped={} conflict={} geometry={} font_build={} restricted={}",
            o.runs_as_text,
            o.runs_as_outlines(),
            o.fonts_embedded,
            o.fallback_not_sfnt,
            o.fallback_paint,
            o.fallback_unmapped,
            o.fallback_conflict,
            o.fallback_geometry,
            o.fallback_font_build,
            o.fallback_restricted,
        )
    });
}

/// Trace an EMF keep-text outcome, in the engine CLI's `emf-text:` vocabulary.
fn trace_emf_text(o: &pdfcer_render::emf::EmfTextOutcome) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "export-image-emf-text kept={} outlines={} paint={} unmapped={} \
             geometry={} symbol_face={}",
            o.runs_as_text,
            o.runs_as_outlines(),
            o.fallback_paint,
            o.fallback_unmapped,
            o.fallback_geometry,
            o.fallback_symbol_face,
        )
    });
}

/// **Write the words on one or more pages out as a plain text file** — the
/// operator's ask, and the fourth member of this module's family.
pub(super) fn text(doc: &mut OpenDoc, plan: &super::exporttext::TextExportPlan) {
    use crate::app::settings::SettingsExt;
    use crate::text::export_text as t;

    if plan.pages.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            "export-text-declined reason=no-pages".to_owned()
        });
        super::record_note(doc.edit_epoch, t::no_pages().to_owned());
        return;
    }

    // The funnel, never `ExtractOptions::default()` — `app::settings`'
    // `syn` check fails the build on a bare constructor outside that module,
    // and the reason binds here hardest of anywhere: the operator's word-gap
    // and `/ActualText` settings decide what the exported string SAYS, and a
    // file that disagreed with the clipboard would be two answers again.
    let options = doc.settings.extract_options();
    let extracted = match pdfcer_core::text_extract::extract_pages_view(
        &doc.session.view(),
        &plan.pages,
        &options,
    ) {
        Ok(extracted) => extracted,
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("export-text-failed reason=extract detail={error}")
            });
            super::record_note(doc.edit_epoch, t::extract_failed(&error.to_string()));
            return;
        }
    };

    // One-based page numbers from here on: the only consumers are operator
    // sentences and the marker lines, and a conversion done at the point of
    // display is a conversion that gets forgotten at one of several points of
    // display.
    let pages: Vec<(usize, String)> = extracted
        .pages
        .iter()
        .map(|page| (page.page_index.saturating_add(1), page.plain_text()))
        .collect();
    let assembled = super::exporttext::assemble(&pages, plan.separator);

    // THE REFUSAL. Before the picker. See the header.
    if assembled.characters == 0 {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "export-text-refused reason=no-text pages={} unreadable={}",
                pages.len(),
                extracted.diagnostics.pages_unreadable
            )
        });
        let mut notes = vec![t::no_text_at_all(pages.len())];
        // The two counters that change what "no text" MEANS, appended to the
        // refusal rather than replacing it: a document of scans and a document
        // of Identity-H-without-ToUnicode both come out empty, and they need
        // different things done to them.
        notes.extend(honesty_notes(&extracted.diagnostics));
        super::record_notes(doc.edit_epoch, notes);
        return;
    }

    let suggested = super::exporttext::suggested_path(&doc.path);
    let crate::app::files::Picked::Path(target) =
        crate::app::files::pick_save_path(&suggested, t::save_dialog_title())
    else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            "export-text-cancelled".to_owned()
        });
        return;
    };

    let bytes = super::exporttext::encode(&assembled.text, plan);
    match std::fs::write(&target, &bytes) {
        Ok(()) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "export-text pages={} chars={} bytes={} empty={} separator={:?} \
                     bom={} crlf={:?}",
                    pages.len(),
                    assembled.characters,
                    bytes.len(),
                    assembled.empty_pages.len(),
                    plan.separator,
                    u8::from(plan.byte_order_mark),
                    plan.line_endings
                )
            });
            // The receipt goes FIRST — `record_notes`' own rule: *"the first
            // sentence is the one an operator reads if they read only one."*
            let mut notes = vec![t::wrote(
                &target.display().to_string(),
                pages.len(),
                assembled.characters,
            )];
            // Departures from the clipboard's own bytes, only when they
            // happened. A bar that narrates non-events stops being read.
            notes.extend(t::wrote_with(
                plan.byte_order_mark,
                matches!(plan.line_endings, super::exporttext::LineEndings::Windows),
            ));
            if assembled.markers_added > 0 {
                notes.push(t::marker_lines_added(assembled.markers_added));
            }
            if !assembled.empty_pages.is_empty() {
                notes.push(t::pages_without_text(&assembled.empty_pages));
            }
            notes.extend(honesty_notes(&extracted.diagnostics));
            super::record_notes(doc.edit_epoch, notes);
        }
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("export-text-failed reason=write detail={error}")
            });
            super::record_note(doc.edit_epoch, t::export_failed(&error.to_string()));
        }
    }
}

/// The four counters from `TextDiagnostics` that change what an operator
/// should do next, worded — or nothing, when all four are zero.
fn honesty_notes(diagnostics: &pdfcer_core::text_extract::TextDiagnostics) -> Vec<String> {
    use crate::text::export_text as t;

    let mut notes = Vec::new();
    let unreadable_fonts = diagnostics
        .identity_fonts_without_to_unicode
        .saturating_add(diagnostics.type3_fonts_without_to_unicode);
    if unreadable_fonts > 0 {
        notes.push(t::unreadable_fonts(
            diagnostics.identity_fonts_without_to_unicode,
            diagnostics.type3_fonts_without_to_unicode,
        ));
    }
    if diagnostics.ladder_failures > 0 {
        notes.push(t::undecodable_characters(
            diagnostics.ladder_failures,
            diagnostics.codes_total,
        ));
    }
    // The fourth counter — read this helper's header before adding a fifth.
    // It earns its place on the same test the other three pass: a Type 3 font
    // may omit its own `/Resources` and inherit the page's (§7.8.3), so a page
    // pdfcer had to assume an empty set for is a candidate explanation for text
    // that came out short. Without it, that export is indistinguishable from a
    // scan.
    if diagnostics.pages_resources_defaulted > 0 {
        notes.push(t::pages_resources_defaulted(
            usize::try_from(diagnostics.pages_resources_defaulted).unwrap_or(usize::MAX),
        ));
    }
    if diagnostics.pages_unreadable > 0 {
        // `usize` for the sentence: the counter is a `u64` because a document
        // may be arbitrarily long, and a page count that does not fit a `usize`
        // is a document that could not have been opened.
        notes.push(t::pages_unreadable(
            usize::try_from(diagnostics.pages_unreadable).unwrap_or(usize::MAX),
        ));
    }
    notes
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    /// The two suggested-name helpers, reduced to the part under test.
    fn named(document: &str, extension: &str) -> PathBuf {
        let mut path = Path::new(document).to_path_buf();
        let stem = path
            .file_stem()
            .map_or_else(|| "export".to_owned(), |s| s.to_string_lossy().into_owned());
        path.set_file_name(format!("{stem}.{extension}"));
        path
    }

    /// **A revision in the document's name survives the export.**
    #[test]
    fn a_dotted_document_name_keeps_its_revision() {
        assert_eq!(
            named("C:/d/plan.rev2.pdf", "dxf"),
            PathBuf::from("C:/d/plan.rev2.dxf"),
            "the revision must survive — `plan.rev2.pdf` and `plan.rev3.pdf` both suggesting \
             `plan.dxf` means the second export silently overwrites the first"
        );
        assert_eq!(
            named("C:/d/plan.rev2.pdf", "fdf"),
            PathBuf::from("C:/d/plan.rev2.fdf"),
            "the form-data export shares the defect and the fix"
        );
    }

    /// The ordinary case, and the one a document with no extension produces.
    #[test]
    fn an_ordinary_name_and_a_bare_one_both_gain_the_extension() {
        assert_eq!(
            named("C:/d/drawing.pdf", "dxf"),
            PathBuf::from("C:/d/drawing.dxf")
        );
        assert_eq!(named("C:/d/plan", "dxf"), PathBuf::from("C:/d/plan.dxf"));
    }
}
