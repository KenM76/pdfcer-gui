//! # `app::actions::export` — writing part of the document out as something
//! else
//!
//! ## Why this is its own file
//!
//! The sixth sibling of [`super::apply`], drawn along the same seam the other
//! five are: *what class of thing does this verb act on?* — pages there,
//! annotations in `annots`, the dimensioning model in `dimensions`, page
//! content in `apply`, redaction marks in `redact`. This is **what leaves the
//! document**.
//!
//! It is a real subject rather than a size-driven cut, and the evidence is the
//! property every verb here shares and no verb elsewhere does: **none of them
//! changes the document at all.** No `vector_edit`, no undo entry, no epoch
//! bump, no cache invalidation. They read the open file and write a different
//! one, which makes every rule the mutation funnel enforces irrelevant to them
//! and every rule about *file* handling — a save picker, an overwrite, a
//! partial write — apply instead.
//!
//! `super::pages::extract` is the same shape and stayed in `pages` because its
//! subject is a page set. If a third export lands, that is the moment to move
//! it here.
//!
//! ## ★ The third export landed on 2026-09-04, and the trigger above FIRED
//!
//! [`image`] — `OPERATOR_REQUESTS.md` **O120**, PNG / JPEG / SVG — is the third,
//! and it is written here rather than in a module of its own, which is the
//! easier half of what the sentence above asks for. The harder half is
//! **`pages::extract` has not moved**, and that is recorded rather than quietly
//! not done:
//!
//! * The condition is met. Three exports now exist and the family is real.
//! * Moving `extract` is a change to `pages`, to `apply`'s dispatch and to
//!   whatever names it, made in the same pass as a new feature — and this
//!   project's own record of what that produces is `RIBBON_IA.md`'s repeated
//!   lesson that a taxonomy move and a capability arriving together make a diff
//!   nobody can review as either.
//!
//! ⇒ So the trigger is left **armed and stated** rather than silently reset. The
//! next reader of this header is looking at a condition that has fired, with
//! the reason it was not acted on written beside it, which is the shape this
//! project uses everywhere else for a decision deferred on purpose. What must
//! not happen is the sentence above being read as still-waiting: it is not.
//!
//! ## ★ Why an export is an `Action` at all
//!
//! `super::apply`'s header answers it for `SaveCopy` and the answer is the same
//! here: **a native file dialog must not open inside a layout pass.** It is a
//! modal OS window that blocks the thread, and opening one from a widget's
//! `clicked()` branch means egui is part-way through building a frame that will
//! not finish until the operator has answered.
//!
//! Nothing about the document is being ordered — there is nothing to order —
//! so the funnel's *invariant* does not apply. Its **reason** does.

use crate::app::state::OpenDoc;

/// Write one page's vector geometry as an ASCII DXF.
///
/// ## What this owes the operator, and why it is not optional
///
/// `DxfOutcome` is the disclosure half, and two of its counts are the reason
/// this feature is worth having over any generic converter:
///
/// - **`skipped_images`** — DXF has no raster entity, so a picture on the page
///   is simply not in the file. The engine's own words for why that must be
///   said: *"an operator whose drawing was half annotation gets a DXF that
///   looks like the geometry went missing, and 'the labels are not in this
///   file' is a sentence they need **before** they open it in SOLIDWORKS, not
///   after."*
/// - **`unreadable_text`** — text pdfcer could not decode, kept apart from
///   `skipped_text` (which the operator asked for) because one is a choice and
///   the other is a fact about the source PDF. Rolling them together would let
///   the second hide inside the first.
///
/// ## ★ Why the geometry is fetched here and not carried in the action
///
/// `PageObjects` is a decomposition of a whole page — every path, every text
/// run, every image placement — and the shell already holds one, cached, keyed
/// on `(page, epoch)`. Carrying it through the action queue would clone it for
/// a value the apply phase can borrow, and a **stale** clone at that: the queue
/// drains after the frame, and an edit raised earlier in the same frame would
/// leave the export describing the page as it was.
///
/// Fetching it here means the export sees the document as it stands when the
/// export runs, which is the only reading that can be defended.
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

    // ★ The picker AFTER the write, not before.
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
                     splines={} skipped_text={} skipped_images={} unreadable_text={}",
                    text.len(),
                    outcome.polylines,
                    outcome.circles,
                    outcome.arcs,
                    outcome.splines,
                    outcome.skipped_text,
                    outcome.skipped_images,
                    outcome.unreadable_text
                )
            });
            // ★ Recorded through `record_note` rather than returned from a
            // `vector_edit` closure, because there is no edit to ride in on —
            // the same case `canvas::interact` records for a caret that cannot
            // be placed. Stamped with the CURRENT epoch, so the sentences stand
            // until the next real edit moves past them.
            //
            // The list is joined rather than recorded one at a time: the slot
            // holds one disclosure, and the last writer would win.
            let notes = crate::text::export_dxf::exported(&target.display().to_string(), &outcome);
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
///
/// `file.export_form_data`, wired 2026-08-27 — a command that had been
/// registered, drawn on File ▸ Export, and **inert for the whole life of the
/// project**.
///
/// # ★★★ The recorded reason was false, and it is the sixth of these
///
/// Its `SCAFFOLDED` entry read:
///
/// > ~~Blocked on a writer that does not exist. `FEATURES.md`'s Forms row:
/// > "fill ✅ …; create field, flatten and FDF/XFDF/CSV still ⬜" — and this
/// > command IS the FDF/XFDF/CSV half.~~
///
/// Three writers exist and two of them have since `Pass 7.1`:
/// `fdf::FormData::to_fdf`, `to_xfdf`, and `formcsv::to_csv`, reached through
/// `EditSession::export_form_data`. The `FEATURES.md` row it cites was itself
/// stale, so — like `edit.form_flatten` two hours earlier — the entry was a
/// **citation of a citation** and nothing had re-read either.
///
/// ⇒ The rule now written on the allow-list's own assertion: *when you touch
/// that list for any purpose, re-derive the reason of the entry beside the one
/// you came for.* This one was found by doing exactly that.
///
/// # ★★ The format is chosen by the EXTENSION, not by a third dialog
///
/// One picker, three formats, decided by what the operator types or picks in
/// the *Save as type* box — which is how every application on this desktop
/// does it, and which `crate::text::tool`'s rule about conventions makes the
/// default answer rather than a shortcut.
///
/// The alternative — a format dialog, then a picker — is two modal windows for
/// one act, and it puts the choice **before** the operator has thought about
/// where the file goes, which is the order they think in reversed.
///
/// An unrecognised extension is **FDF**, and that is a decision rather than a
/// fallback: FDF is the format the standard defines for this data (§12.7.8),
/// it is what Acrobat writes, and it is the only one of the three that a
/// reader can import without being told what it is.
///
/// # ★★★ The CSV disclosure is not optional, and it is about a spreadsheet
/// rather than about a PDF
///
/// `formcsv::to_csv` **neutralises** values that would otherwise be executed as
/// formulas when the file is opened in a spreadsheet — a leading `=`, `+`, `-`
/// or `@`. That is a real and well-known injection route, and pdfcer doing the
/// right thing silently would leave an operator believing their exported data
/// is byte-identical to what the form holds.
///
/// It is not. `neutralised` counts how many, `neutralised_fields` names them,
/// and both are reported. Rule 4's *"the half that survives is the point"*: an
/// inference the operator cannot see still owes them an off-canvas sentence.
///
/// # Nothing about the document changes
///
/// No `vector_edit`, no epoch bump, no cache invalidation — this module's
/// header explains why that is what makes these verbs a family. The
/// disclosures ride `record_edit_disclosure` at the current epoch, so they
/// stand until the next real edit moves past them.
pub(super) fn form_data(doc: &mut OpenDoc) {
    // ★ The data first, the picker second — `dxf`'s ordering and its reason:
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

    // ★ The document's own path as the FDF `/F` source, so a reader importing
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
            // ★★ The neutralisation disclosure — see the header. Reported only
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
///
/// Beside the document and named after it, with `.fdf` — [`suggested_path`]'s
/// rule and its reason. The extension is the **default format** as well as a
/// suggestion, which is why it is FDF: see [`form_data`]'s header.
fn suggested_form_path(doc: &OpenDoc) -> std::path::PathBuf {
    let mut path = doc.path.clone();
    let stem = path
        .file_stem()
        .map_or_else(|| "form".to_owned(), |s| s.to_string_lossy().into_owned());
    // ★★★ `set_file_name`, NOT `set_extension` — see [`suggested_path`] below,
    // which carries the whole argument. `set_extension` replaces everything
    // after the LAST dot, so a `plan.rev2.pdf` loses its revision here too.
    path.set_file_name(format!("{stem}.fdf")); // ui-text-exempt: a file extension, never displayed as prose
    path
}

/// Where the save dialog opens, and what it calls the file.
///
/// Beside the document, named after it, with a `.dxf` extension. The same rule
/// `super::pages::suggested_path` follows for an extract, and for its reason: a
/// picker that opens in the last-used directory of some other application is a
/// picker that makes the operator navigate back to their own project every
/// time.
fn suggested_path(doc: &OpenDoc) -> std::path::PathBuf {
    let mut path = doc.path.clone();
    let stem = path
        .file_stem()
        .map_or_else(|| "export".to_owned(), |s| s.to_string_lossy().into_owned());
    // ★★★ **THE COMMENT THAT USED TO BE HERE WAS WRONG, AND THE CODE IT
    // DEFENDED SILENTLY OVERWROTE THE OPERATOR'S EXPORTS.**
    //
    // It read: *"`set_extension` rather than pushing a string: a document
    // called `plan.rev2.pdf` has a stem of `plan.rev2`, and appending would
    // produce `plan.rev2.dxf` either way."*
    //
    // The first clause is true and the conclusion does not follow.
    // `Path::set_extension` replaces everything after the **last** dot, and
    // `plan.rev2` has one — so the call produced **`plan.dxf`**, not
    // `plan.rev2.dxf`. The revision was dropped.
    //
    // ⇒ Why that is a data-loss defect rather than a cosmetic one:
    // `plan.rev2.pdf` and `plan.rev3.pdf` both suggested `plan.dxf`, so
    // exporting the second **overwrote the first**, in a save dialog whose only
    // warning is the operating system's generic "a file with that name already
    // exists". `.rev2` / `.rev3` is an ordinary CAD naming shape, and the two
    // files that collide are the two the operator is most likely to want side
    // by side.
    //
    // ★ Found on 2026-09-04 by the image export, which wrote the same helper,
    // tested it against `plan.rev2.pdf` on its first run, and watched it fail.
    // The DXF path had shipped for weeks with a comment asserting the
    // behaviour it did not have — **a claim in a comment is not a test**, and
    // this one was load-bearing enough that its author wrote it down to explain
    // why the safer-looking alternative was unnecessary.
    //
    // `set_file_name` with the stem interpolated appends unconditionally, which
    // is what the old comment believed `set_extension` did. A document with no
    // extension at all still gains one, because the stem of `plan` is `plan`.
    path.set_file_name(format!("{stem}.dxf")); // ui-text-exempt: a file extension, never displayed as prose
    path
}

/// ★★★ **Write one or more pages out as PNG, JPEG or SVG** —
/// `OPERATOR_REQUESTS.md` **O120**, and the third member of this module's
/// family.
///
/// The operator, 2026-09-03, verbatim:
///
/// > *"can you add the ability to export page(es) to png, jpg, svg. note that
/// > there had better be full support (including transparency where
/// > supported!)."*
///
/// This module's header says the third export is the one that decides whether
/// the family is real. It is, and it is: nothing here changes the document, no
/// `vector_edit` runs, no epoch moves, no cache is dropped. What it shares with
/// its two siblings is the whole of what the module is for — **it reads the
/// open file and writes a different one.**
///
/// # ★★★ The refusal comes FIRST, before the picker and before the render
///
/// [`crate::app::actions::imageexport::ImagePlan::impossible`] is asked before
/// anything else happens, and the reason is the engine's own instruction:
///
/// > **refuse a "transparent" JPEG by name in your UI, never flatten silently**
///
/// The window already prevents the combination — its checkbox goes dead when
/// JPEG is selected and says why — so reaching this branch means the window was
/// bypassed. That it is *unreachable today* is exactly why it is here: the
/// property that must hold is **pdfcer never puts a page on a white background
/// without saying so**, and a guard that lives only in a window makes that a
/// property of the window rather than of the program. A keymap, a restored
/// plan, or a later window with a different layout each walk past a window and
/// none of them walks past this.
///
/// ⇒ And it *refuses*. Flattening would produce a file that opens, looks nearly
/// right, and carries a white rectangle the operator meets when the drawing is
/// already inside somebody else's document.
///
/// # ★★ Why there is no call to `pdfcer_render::export::flatten_over`
///
/// The engine offers it, this function does not use it, and that is worth
/// stating rather than leaving as an apparent omission.
///
/// Transparency is declined **at the source**, by rendering with
/// [`pdfcer_render::PageBackdrop::White`]. ISO 32000-1 §11.4.7 already makes
/// the page an isolated group composited over white, so the renderer's own
/// composite *is* the standard's; `flatten_over` is a second, later composite
/// over a buffer that has already been premultiplied. The two agree for
/// ordinary content and only one of them is the specification's, so that is
/// the one used. `flatten_over` earns its place in a caller holding a pixmap
/// it did not render — a clipboard paste, a region grab — and this is not one.
///
/// # ★ The order is REFUSE, ASK, RENDER — and it differs from [`dxf`]'s
///
/// [`dxf`] does the whole write before opening the picker, on the rule *"the
/// operator is never asked where to put a file that turns out to be empty"*,
/// and it can afford to because a DXF write is pure and cannot fail.
///
/// A page render is neither pure nor cheap. It is the most expensive thing this
/// program does, it takes seconds on a dense CAD sheet, and fifty of them
/// before a picker would mean an operator who presses Cancel has waited for
/// nothing. So the picker comes second — and the property `dxf`'s ordering was
/// protecting is preserved by a different mechanism: **everything that could
/// make this export empty has already been said in the window**, beside the
/// control that causes it. The pixel count, the `MAX_PIXMAP_EDGE` ceiling, a
/// range naming no page, and the transparent-JPEG refusal are all on screen
/// before the button is pressable.
///
/// # ★★ Rule 4 — the disclosure, off-canvas and afterwards
///
/// Nothing is marked on the page or on the canvas. Every sentence goes to
/// [`super::record_notes`], the same slot [`dxf`] and [`form_data`] use,
/// stamped with the current epoch so it stands until the next real edit moves
/// past it.
///
/// What it carries, and why each is owed:
///
/// * **the resolution written into the file.** The engine's note: *"without
///   `pHYs` Word places a 300 DPI page four times too large."* That the number
///   was *chosen* is not the claim; that it *travelled* is.
/// * **whether transparency survived**, in either direction — the operator
///   asked for it by name, so both answers are answers.
/// * **`ExportTally`, for SVG** — shadings rasterised, soft masks kept,
///   overprint and non-separable blends drawn as their `Normal` approximation,
///   dashed strokes pre-applied, blend modes Word's importer ignores.
/// * **that SVG text is glyph outlines**, which nothing counts, which no
///   inspection of the file by an operator would reveal, and which is the
///   single largest surprise the format holds.
pub(super) fn image(doc: &mut OpenDoc, plan: &crate::app::actions::imageexport::ImagePlan) {
    use crate::app::actions::imageexport;
    use crate::app::settings::SettingsExt;
    use crate::text::export_image as t;

    // ★★★ First, before the picker and before the render. See the header.
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

    // ★ Through the settings funnel, never `RenderOptions::default()`.
    //
    // `crate::app::settings::SettingsExt` is the one place that turns the
    // operator's configuration into render options, and a `syn` check in that
    // module fails the build if any other file constructs these itself. The
    // five settings it applies — CMYK intent, mask resampling, minification,
    // JPEG polarity, missing appearance state — are exactly the ones that
    // decide what the exported picture LOOKS like, so an export that skipped
    // the funnel would disagree with the canvas the operator was looking at.
    //
    // ★★ The annotation stance and the layer overrides come from the DOCUMENT,
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

        // ★★ A `match` on the format, NOT `if plan.format.is_vector()`.
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
            // ★ One page's failure STOPS the run rather than skipping on.
            //
            // The alternative — carry on and summarise at the end — leaves the
            // operator with a directory of files and a sentence about a gap
            // somewhere in it. A run that stops names the page it stopped on,
            // and every file written before it is on disk and named in the same
            // disclosure.
            //
            // ★★ The two failures are told APART, and that is not decoration.
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
                "export-image page={page_index} format={:?} bytes={} dpi={} transparent={}",
                plan.format,
                produced.bytes.len(),
                plan.dpi,
                u8::from(plan.transparent)
            )
        });

        // ★★ Only the FIRST page's fidelity notes are kept.
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

    // ★ The lead-in goes FIRST — `record_notes`' own rule: *"the first sentence
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
///
/// A struct rather than a tuple because the caller has to pick a *different
/// sentence* per kind, and a `(Vec<u8>, u32, u32, usize, Vec<String>)` would
/// carry two fields that are meaningless for one of the two branches.
struct Output {
    bytes: Vec<u8>,
    kind: Produced,
    notes: Vec<String>,
}

/// ★★ Why a page's writer failed, kept apart because the two failures ask
/// different things of the operator.
///
/// [`Self::Render`] is about the **page** — it will happen again whatever
/// format is chosen, and the resolution or the document is the thing to look
/// at. [`Self::Encode`] is about the **format**, and its commonest cause is
/// `ExportError::TooLargeForJpeg`: JPEG stores its dimensions in sixteen bits
/// (ITU-T T.81 §B.2.2), so a raster over 65,535 pixels on a side has no JPEG
/// form at all while having a perfectly good PNG one.
///
/// ⇒ A single "export failed" would send an operator whose only problem is a
/// format's arithmetic limit off to investigate their drawing. Each carries
/// the engine's own message, which names the numbers.
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
    /// ★ Geometry **and** pictures, which is what a metafile always is.
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
/// Split out of [`image`] so the loop reads as *produce, write, say* rather
/// than as one branch nested in another. `Err` carries the engine's own
/// message; the caller wraps it in a sentence that names the page.
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

    // ★★★ `Some(dpi)`, never `None`. The engine's note is unambiguous about
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
        // ★ Note that these arms cost nothing in safety: the caller's own
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
    // ★★ The BACKGROUND, not the backdrop. `export_svg_view`'s own doc: *"The
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
        });
    let export = pdfcer_render::svg::export_svg_view(view, page, options, &svg_options)
        .map_err(|error| Failed::Render(error.to_string()))?;

    let mut notes = vec![if plan.transparent {
        crate::text::export_image::transparency_kept().to_owned()
    } else {
        crate::text::export_image::flattened_to_white().to_owned()
    }];
    // ★★★ Rule 4's content. `svg_fidelity` always leads with the fact nothing
    // counts — that text is glyph outlines — and then names every counter the
    // recording had to raise. See `crate::text::export_image`.
    notes.extend(crate::text::export_image::svg_fidelity(
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
///
/// # Why this is a third function and not a flag on [`svg_bytes`]
///
/// The two share a *recording* — `pdfcer_render::emf` walks the same export
/// display list `pdfcer_render::svg` does — and share nothing else. Different
/// options type, different outcome type, different disclosure, different
/// receipt line, and a different answer for every one of the five things EMF
/// cannot express. A `if metafile { … } else { … }` inside one function would
/// be two functions sharing a brace.
///
/// # ★★ The background follows [`svg_bytes`]'s rule, for the same reason
///
/// `EmfOptions::background` is `Option<Rgb>` exactly as `SvgOptions`' is, and
/// `None` is the transparent state — the engine's CLI calls it *"EMF's
/// natural state (nothing is drawn where nothing was painted)"*. So the same
/// flag drives both, and the `RenderOptions` backdrop the caller set is again
/// **not** what decides it.
///
/// ⇒ This is the shape of mistake that ships a window promising a clear
/// background and a file with a white rectangle at the bottom of it, and it
/// is worth the second comment because the two option structs are the two
/// places in this file where the backdrop is a decoy.
///
/// # ★★★ Nothing here validates the metafile, and that is a decision
///
/// `pdfcer_render::emf::walk_records` exists precisely so a consumer can
/// check a metafile's record structure before handing it to
/// `SetEnhMetaFileBits`, and it is deliberately not called on this path. A
/// **file** export hands the bytes to `std::fs::write`, which cannot be made
/// to misbehave by a malformed record; the reader that would choke on one is
/// somebody else's program, tomorrow. Walking every record to produce a
/// sentence nobody could act on would cost a second pass over the whole
/// metafile on every export.
///
/// ⚠ The **clipboard** path is the one that owes this check, because there
/// `SetEnhMetaFileBits` is handed a raw buffer and a bad one is a GDI failure
/// rather than a refusal. See `crate::clipboard`, which is where that call
/// will live when the placement half is buildable.
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
        });
    let export = pdfcer_render::emf::export_emf_view(view, page, options, &emf_options)
        .map_err(|error| Failed::Render(error.to_string()))?;

    let mut notes = vec![if plan.transparent {
        crate::text::export_image::transparency_kept().to_owned()
    } else {
        crate::text::export_image::flattened_to_white().to_owned()
    }];
    // ★★★ Rule 4's content, through the shell's own `EmfCounts` rather than
    // the engine's `EmfOutcome`. The reason is testability and it is argued in
    // full on `EmfCounts` itself: `EmfOutcome` is `#[non_exhaustive]` with no
    // `Default`, so no test in this crate could ever build one, and the
    // counters-to-sentences mapping is the part of this path most worth
    // testing.
    let counts = crate::app::actions::imageexport::EmfCounts::from(&export.outcome);
    notes.extend(crate::text::export_image::emf_fidelity(&counts));
    Ok(Output {
        bytes: export.emf,
        kind: Produced::Metafile {
            ops: counts.ops,
            rasters: counts.rasters_embedded,
        },
        notes,
    })
}

/// ★★★ **Write the words on one or more pages out as a plain text file** — the
/// operator's ask of 2026-09-04, and the fourth member of this module's family.
///
/// > *"also the engine can export PDFs as text. we should have export/import
/// > for that."*
///
/// Half of that sentence shipped. `super::exporttext`'s header carries the full
/// finding on the other half — **`pdfcer-core` has no route from a text file
/// back into a PDF**, in any of the three senses "import text" could mean, and a
/// request has been filed rather than a round trip faked. Nothing here mentions
/// one.
///
/// # ★★ It writes the CLIPBOARD's own string
///
/// At the plan's defaults, the bytes this writes are exactly what
/// `file.copy_document_text` puts on the clipboard: the settings funnel's
/// `ExtractOptions`, `plain_text()`, U+000C between pages, no BOM, no
/// line-ending rewrite. `app::dispatch::textcopy`'s header makes the argument
/// for its two verbs sharing one extraction; this is the same argument with a
/// file on the end of it. **Two answers to "what is the text of this document"
/// inside one program is worse than either**, because both of them look like
/// text and nothing on screen would say which one you have.
///
/// # ★★★ The order is EXTRACT, REFUSE, ASK, WRITE — and the refusal is the
/// whole feature
///
/// [`dxf`]'s ordering, not [`image`]'s, and for [`dxf`]'s stated rule: *"the
/// operator is never asked where to put a file that turns out to be empty."*
///
/// Here that rule stops being a nicety and becomes the point of the feature.
/// **A scanned drawing has no text layer**, so extracting it succeeds, returns
/// nothing, and would write a zero-byte `.txt` — which is indistinguishable
/// from a successful export of a blank page. The operator finds out when they
/// open it, or worse, when whoever they sent it to does.
///
/// So a zero character count refuses **before the picker opens**, names why (the
/// page is a picture of its words rather than words — a fact about their file,
/// not a pdfcer failure) and names the remedy by its ribbon label,
/// `File ▸ Recognise text`. See `crate::text::export_text::no_text_at_all`.
///
/// ⇒ [`image`] can afford the opposite ordering because a render is expensive
/// and everything that could make *it* empty is already stated in its window.
/// An extraction is 331–449 ms on this project's fixtures — `crate::find`'s own
/// measurement — and the thing that makes this one empty is a property of the
/// document that no window could have known in advance.
///
/// # ★ `extract_pages_view`, for every scope, including "every page"
///
/// One entry point rather than two. `resolve_pages` already turns *every page*
/// into `0..count`, so branching to `extract_document_view` for that case would
/// buy nothing and would introduce the one thing this feature cannot afford: a
/// second path to the same string, differing in its failure semantics.
/// (`extract_document_view` swallows a bad index; `extract_pages_view` reports
/// `NoSuchPage`, which is the honest answer when a page vanished between the
/// press and the drain.)
///
/// `session.view()`, never `session.document()` — the operator is exporting the
/// document they are looking at, unsaved edits included (decision 018), which
/// is the same rule the two clipboard verbs and the print preview follow.
///
/// # ★★ Rule 4 — the disclosure, off-canvas and afterwards
///
/// Nothing is marked on the page. Every sentence goes to [`super::record_notes`]
/// at the current epoch, and the set is chosen so each one tells the operator
/// something they could **act** on:
///
/// * **the file, the page count and the character count** — the receipt;
/// * **which pages came out empty**, by number, because an empty page 4 in a
///   six-page set is a scanned insert they can go and look at;
/// * **pages that could not be read at all**, kept apart from the above: an
///   empty page is one pdfcer read and found nothing on, this is one pdfcer
///   could not read, and rolling them together would let damage present as a
///   scan;
/// * **fonts publishing no route to Unicode** — Type 3 without `/ToUnicode`,
///   Identity-H without `/ToUnicode`. The text renders perfectly and is missing
///   from the file, which is the standard's own answer (§9.10.2) and is exactly
///   why it has to be said. Acrobat's answer to this case is silence;
/// * **characters that fell through the decoding ladder**, as a fraction, so
///   40-in-200 and 40-in-400,000 do not read as the same event;
/// * **what pdfcer itself added**, when page markers were asked for.
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

    // ★ The funnel, never `ExtractOptions::default()` — `app::settings`'
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

    // ★★★ THE REFUSAL. Before the picker. See the header.
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
            // ★ The receipt goes FIRST — `record_notes`' own rule: *"the first
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

/// The three counters from `TextDiagnostics` that change what an operator
/// should do next, worded — or nothing, when all three are zero.
///
/// ★ A helper rather than three inline `if`s at two call sites, because **both**
/// the refusal path and the success path owe exactly this set. A document whose
/// every page is Identity-H-without-`/ToUnicode` refuses with a zero character
/// count that looks identical to a scan, and the counter is the only thing that
/// tells them apart.
///
/// `TextDiagnostics` carries roughly thirty counters and this takes three. The
/// other twenty-seven are true and are not **actionable**: `spaces_derived` and
/// `lines_derived` are facts about every extraction ever run, and the window
/// already said so in `loses_breaks` where an operator can read it before
/// deciding. A status bar that lists everything measured is one nobody reads,
/// and rule 4's whole value is in being read.
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
    ///
    /// `suggested_path` and `suggested_form_path` take an `OpenDoc`, which
    /// carries a session and cannot be built in a unit test. Their arithmetic
    /// is one line each and it is that line that was wrong, so it is
    /// reproduced here **from the same expression** rather than re-derived —
    /// if either site changes shape, this stops describing it and the comment
    /// below is the instruction to whoever notices.
    ///
    /// ★ Not a seam worth extracting: a shared helper would be a third place
    /// the rule lives, and the rule is `format!("{stem}.{ext}")`.
    fn named(document: &str, extension: &str) -> PathBuf {
        let mut path = Path::new(document).to_path_buf();
        let stem = path
            .file_stem()
            .map_or_else(|| "export".to_owned(), |s| s.to_string_lossy().into_owned());
        path.set_file_name(format!("{stem}.{extension}"));
        path
    }

    /// ★★★ **A revision in the document's name survives the export, and until
    /// 2026-09-04 it did not.**
    ///
    /// `Path::set_extension` replaces everything after the **last** dot, so
    /// `plan.rev2` became `plan` and the suggested name was `plan.dxf`.
    ///
    /// ⇒ The consequence is data loss, not untidiness: `plan.rev2.pdf` and
    /// `plan.rev3.pdf` both suggested `plan.dxf`, so exporting the second
    /// **overwrote the first** — behind nothing but the operating system's
    /// generic "a file with that name already exists". `.rev2` / `.rev3` is an
    /// ordinary CAD naming shape, and the two files that collided are the two
    /// an operator is most likely to want side by side.
    ///
    /// ★★ The site carried a comment asserting the behaviour it did not have,
    /// written to explain why the safer-looking alternative was unnecessary.
    /// **A claim in a comment is not a test.** This is that comment, executed.
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
    ///
    /// ★ The old comment's one true claim was that a document called `plan`
    /// with no extension *"would gain one only through this call"*. It still
    /// does: the stem of `plan` is `plan`, and the format string appends
    /// unconditionally.
    #[test]
    fn an_ordinary_name_and_a_bare_one_both_gain_the_extension() {
        assert_eq!(
            named("C:/d/drawing.pdf", "dxf"),
            PathBuf::from("C:/d/drawing.dxf")
        );
        assert_eq!(named("C:/d/plan", "dxf"), PathBuf::from("C:/d/plan.dxf"));
    }
}
