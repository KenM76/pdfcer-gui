//! **Graphics-memory pressure, made observable** — the blank page that
//! nothing reports.
//!
//! # The defect
//!
//! O219: *"sometimes before this happens the view goes blank and when I zoom
//! in a little more I get the error."* A blank page with no message is the
//! signature of a texture upload that failed for want of graphics memory:
//! `GL_OUT_OF_MEMORY` is raised on a flag, nothing traps, the texture object
//! stays bound with no storage, and the canvas draws an empty rectangle at
//! full frame rate. `egui_glow` reads that flag only under
//! `debug_assertions`, so in the build the operator runs, **nothing in the
//! render stack looks at it at all**.
//!
//! [`native_gl::drain`] is the instrument. This module is the bookkeeping that
//! makes a reading of it mean something.
//!
//! # ★★★ The frame boundary this module is built around
//!
//! `eframe` runs the whole of [`eframe::App::ui`] and *then* calls
//! `paint_and_update_textures`, which is where `ctx.load_texture`'s queued
//! delta actually becomes a `glTexImage2D`. So:
//!
//! * an upload **ordered** during frame *N* is **performed** at the end of
//!   frame *N*, and
//! * the error it raised is first readable at the **top of frame *N+1***.
//!
//! ⇒ [`poll`] runs at the top of the frame and is told about the *previous*
//! frame's uploads. Draining anywhere inside the frame's own work would read
//! a flag the frame's own uploads have not reached yet, and attribute every
//! failure to whatever was on screen one frame too early.
//!
//! # ★★ A code carries no provenance, so attribution is the hard part
//!
//! GL's error flag records *that* something failed, never *what*. [`attribute`]
//! is therefore deliberately unwilling: it blames the canvas's whole-page
//! raster only when that raster was the frame's **only** upload. Everything
//! else is traced with the reason it could not be pinned, which is the
//! measurement that says whether the rule is too strict to ever fire.
//!
//! ★ That refusal is only worth anything if the census is COMPLETE. Every
//! `ctx.load_texture` in this crate records here — the canvas raster, the page
//! thumbnails, the print preview and the icon sheet — because a route that
//! uploads without recording does not merely go unseen: it makes a frame that
//! uploaded two things look like a frame that uploaded one, and the one left
//! standing gets blamed for the other's failure.
//!
//! ⚠ [`Unattributed::NoUploads`] therefore means *"nothing this crate knows
//! about uploaded"*, not *"nothing uploaded"*. `egui` grows its own font atlas
//! and `egui_tiles` its own decorations, and neither passes through here.
//!
//! # ⚠ A debug run is not evidence
//!
//! In a debug build `egui_glow`'s own `check_for_gl_error!` runs immediately
//! after each GL call and **clears** the flag. By the time [`poll`] looks,
//! there is nothing on it. That is not a defect in either party — it is why
//! any measurement taken with this module must be taken from a release build.

use crate::render::worker::RenderKey;

/// Which surface ordered an upload.
///
/// ★ Not derivable from the pixels: a page thumbnail and the canvas raster are
/// built from the same [`RenderKey`] type, by the same function, and differ
/// only in what they are *for*. Passed in at the call site so that adding a
/// fourth surface is a compile error rather than a silent miscount.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Surface {
    /// The page the operator is looking at. **The only blamable surface**,
    /// because it is the only one whose size follows the zoom.
    Canvas,
    /// A page thumbnail in the Pages panel.
    ///
    /// Rasterized at a fixed small size regardless of zoom, so it can no more
    /// be the cause of a zoom-dependent failure than the icon sheet can. It is
    /// recorded to stop it being mistaken for the canvas — the two share
    /// [`crate::render::raster::texture_from_pixels`].
    Thumbnail,
    /// The print dialog's page preview.
    Preview,
    /// One glyph of the icon sheet.
    Icon,
}

/// The page-shaped part of an upload, present only when there is one.
///
/// An icon has no page and no zoom; giving it a page number would be a right
/// value in the wrong role, and the trace would read as though the operator
/// were at scale 1.0 on page 0.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Raster {
    /// Which page the raster is a picture of (0-based).
    pub page: usize,
    /// Device pixels per PDF user-space unit — the operator's zoom already
    /// multiplied by `pixels_per_point`, exactly as [`RenderKey`] stores it.
    pub raster_scale: f32,
    /// Whether this was a picture of the **whole page** rather than a region.
    ///
    /// The distinction is the whole of why a ceiling might move: the region
    /// tier is scale-invariant — its rasters are a fixed multiple of the
    /// viewport at any zoom — so a region upload that ran out of memory says
    /// something about the machine, not about the zoom. Only the whole-page
    /// tier grows without bound as the operator zooms in.
    pub whole_page: bool,
}

/// One texture upload, as it was ordered.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Upload {
    /// Who ordered it.
    pub surface: Surface,
    /// How many pixels were handed to the driver.
    ///
    /// Four bytes each, RGBA8. Kept because it is the number the failure is
    /// actually about, and because a trace line carrying only a zoom leaves
    /// the reader to multiply page size by scale squared to find out whether
    /// the figure was plausible.
    pub pixels: u64,
    /// The page and scale, for the surfaces that have one.
    pub raster: Option<Raster>,
}

impl Upload {
    /// The upload as a trace fragment.
    fn trace_fragment(self) -> String {
        let mpx = self.pixels as f64 / 1_000_000.0;
        match self.raster {
            Some(r) => format!(
                // ui-text-exempt: a fragment of the `gl-pressure` trace line,
                // written to stderr and never rendered by any widget.
                "surface={:?} page={} scale={:.2} mpx={mpx:.1} whole={}",
                self.surface, r.page, r.raster_scale, r.whole_page
            ),
            None => format!(
                // ui-text-exempt: a fragment of the `gl-pressure` trace line,
                // written to stderr and never rendered by any widget.
                "surface={:?} mpx={mpx:.1}",
                self.surface
            ),
        }
    }
}

/// What a drained error set can and cannot be pinned on.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Attribution {
    /// The flag was clean. Nothing happened and nothing is owed.
    Clean,
    /// Something was drained, and it can be blamed on this upload.
    ///
    /// The one case in which a caller would be entitled to act — lower a
    /// ceiling, drop a cached raster. Nothing acts on it yet; the design
    /// requires the measurement with one, three and six documents open first.
    Blamed(Upload),
    /// Something was drained and cannot be blamed on the canvas's raster.
    Unattributed(Unattributed),
}

/// Why a drained error could not be pinned on the canvas's page raster.
///
/// Each variant is a distinct thing to learn from a trace, which is why this
/// is not a `bool`: *"the failure had nothing to do with the canvas"* and
/// *"the failure was one of four uploads and we cannot say which"* call for
/// different next moves, and collapsing them would hide that.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unattributed {
    /// Nothing this crate knows about uploaded last frame.
    ///
    /// ⚠ Not *"nothing uploaded"* — `egui`'s font atlas does not pass through
    /// here. This is the expected verdict for a failure raised by the
    /// framework's own texture growth.
    NoUploads,
    /// More than one upload was ordered. Carries how many.
    Several(usize),
    /// The frame's one upload was not the canvas's.
    NotTheCanvas(Surface),
    /// The canvas's one upload was a **region** raster.
    ///
    /// Scale-invariant by construction, so this is evidence about the machine
    /// — most likely accumulation across open documents, which is O221 — and
    /// not about the zoom the operator had reached.
    RegionOnly,
}

/// Decide what a drained error set can be blamed on. Pure; the whole rule.
///
/// Split out from [`poll`] because the rule is the part that can be wrong and
/// the GL call is the part that cannot be tested. A caller acting on a
/// [`Attribution::Blamed`] is acting on this function alone.
#[must_use]
pub fn attribute(uploads: &[Upload], drained: native_gl::Drained) -> Attribution {
    if drained.is_clean() {
        return Attribution::Clean;
    }
    match uploads {
        [] => Attribution::Unattributed(Unattributed::NoUploads),
        [one] => match (one.surface, one.raster) {
            (Surface::Canvas, Some(r)) if r.whole_page => Attribution::Blamed(*one),
            (Surface::Canvas, _) => Attribution::Unattributed(Unattributed::RegionOnly),
            (other, _) => Attribution::Unattributed(Unattributed::NotTheCanvas(other)),
        },
        many => Attribution::Unattributed(Unattributed::Several(many.len())),
    }
}

/// Where the frame's ordered uploads are stashed between being ordered and
/// being judged.
///
/// `ctx.data` rather than a field on `PdfcerApp`, for the reason every other
/// cross-cutting per-frame value in this crate travels that way (the theme,
/// the selected tool, the text draft, the dialog owner): the producers are an
/// `OpenDoc` method, a panel, a dialog and an icon cache, and threading a
/// field to all four would put a graphics-memory concern into each of them.
#[derive(Clone, Default)]
struct Ordered(Vec<Upload>);

/// The `ctx.data` slot. One per process; GL's error flag is global too.
fn slot() -> egui::Id {
    egui::Id::new("render::pressure::ordered")
}

/// Note that an upload of a **page raster** has been ordered this frame.
///
/// Called from [`crate::render::raster::texture_from_pixels`], which both the
/// canvas and the thumbnails go through — hence `surface`, which that function
/// cannot work out for itself.
pub fn record_raster(ctx: &egui::Context, surface: Surface, key: &RenderKey, w: u32, h: u32) {
    push(
        ctx,
        Upload {
            surface,
            pixels: u64::from(w) * u64::from(h),
            raster: Some(Raster {
                page: key.page(),
                raster_scale: key.raster_scale(),
                whole_page: key.region().is_none(),
            }),
        },
    );
}

/// Note that an upload with no page behind it has been ordered this frame.
///
/// The icon sheet and the print preview. Neither can be blamed for a
/// zoom-dependent failure, and that is exactly why they are recorded: an
/// unrecorded upload makes a two-upload frame look like a one-upload frame,
/// and the one left standing is blamed for the other's failure.
pub fn record_other(ctx: &egui::Context, surface: Surface, w: u32, h: u32) {
    push(
        ctx,
        Upload {
            surface,
            pixels: u64::from(w) * u64::from(h),
            raster: None,
        },
    );
}

/// Cheap: a push onto a vector that is emptied every frame and, on the
/// overwhelming majority of frames, never grows at all.
fn push(ctx: &egui::Context, upload: Upload) {
    ctx.data_mut(|d| d.get_temp_mut_or_default::<Ordered>(slot()).0.push(upload));
}

/// Take and clear what the previous frame ordered.
fn take(ctx: &egui::Context) -> Vec<Upload> {
    ctx.data_mut(|d| std::mem::take(&mut d.get_temp_mut_or_default::<Ordered>(slot()).0))
}

/// Read the error flag at the top of a frame and trace what it held.
///
/// `gl` is `None` when the backend is not glow or the context has gone away
/// during shutdown; the previous frame's record is still cleared in that case,
/// because a record kept across frames would be attributed to the wrong one
/// the moment a context came back.
///
/// **Traces only when something was drained.** A line per frame would be sixty
/// a second of "nothing happened", which is how the one line that matters
/// becomes unfindable.
///
/// This does not change any ceiling. Per `DESIGNS.md`, the measurement with
/// one, three and six documents open is owed before anything acts on it.
pub fn poll(ctx: &egui::Context, gl: Option<&eframe::glow::Context>) {
    let uploads = take(ctx);
    let Some(gl) = gl else { return };

    let drained = native_gl::drain(gl);
    if drained.is_clean() {
        return;
    }

    // The verdict is assembled inside the closure, not before it: `diag::trace`
    // takes a thunk precisely so that a build with tracing off pays nothing,
    // and a string formatted at the call site is paid for either way.
    crate::diag::trace(move || {
        let verdict = match attribute(&uploads, drained) {
            // `is_clean` was false, so `attribute` cannot return this. Spelled
            // out rather than unreachable-panicking: a trace is not worth a
            // crash.
            Attribution::Clean => "clean".to_owned(),
            Attribution::Blamed(u) => format!("blamed {}", u.trace_fragment()),
            Attribution::Unattributed(u) => format!("unattributed={u:?}"),
        };
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        format!(
            "gl-pressure oom={} count={} other={:?} truncated={} {verdict}",
            drained.out_of_memory,
            drained.count,
            drained.first_other,
            drained.truncated(),
        )
    });
}

/// Trace the device's real single-axis texture limit whenever it changes.
///
/// ⚠ **Read every frame, never cached.** `egui` defaults
/// `InputState::max_texture_side` to 2048 and only learns the true figure once
/// the backend has supplied it through `RawInput` — so a value captured at
/// startup is a plausible-looking number that belongs to no device. Reading it
/// every frame costs one field access and cannot go stale.
///
/// Trace-only. The whole-page tier's budget is an edge count that admits
/// 16383², which is over the limit on some devices and under it on others; the
/// guard that uses this figure is a separate change, and on the operator's own
/// card it is expected to be inert.
pub fn trace_texture_limit(ctx: &egui::Context) {
    let side = ctx.input(|i| i.max_texture_side);
    crate::diag::trace_on_change("gl-max-texture-side", move || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        format!("side={side}")
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canvas_whole(page: usize) -> Upload {
        Upload {
            surface: Surface::Canvas,
            pixels: 64_000_000,
            raster: Some(Raster {
                page,
                raster_scale: 8.0,
                whole_page: true,
            }),
        }
    }

    const OOM: native_gl::Drained = native_gl::Drained {
        out_of_memory: true,
        count: 1,
        first_other: None,
    };

    /// A clean flag is clean no matter what was uploaded.
    ///
    /// The case that runs every frame of every session, and the one where a
    /// mistake would blame a page raster for nothing at all.
    #[test]
    fn a_clean_flag_blames_nothing_even_after_a_huge_upload() {
        assert_eq!(
            attribute(&[canvas_whole(0)], native_gl::Drained::CLEAN),
            Attribution::Clean
        );
    }

    /// ★ The only shape that may be blamed: one upload, the canvas's, whole.
    #[test]
    fn one_whole_page_canvas_upload_is_blamed() {
        assert_eq!(
            attribute(&[canvas_whole(3)], OOM),
            Attribution::Blamed(canvas_whole(3))
        );
    }

    /// ★★★ A thumbnail is NOT the canvas, even though it is a whole page.
    ///
    /// The two share [`crate::render::raster::texture_from_pixels`] and are
    /// built from the same key type, so a rule keyed on the pixels alone
    /// cannot tell them apart — and would blame a 40 kB thumbnail, at whatever
    /// page happened to scroll into the Pages panel, for a failure raised by
    /// the font atlas. This test is what makes that a build failure.
    #[test]
    fn a_thumbnail_is_never_blamed_though_it_is_a_whole_page_raster() {
        let thumb = Upload {
            surface: Surface::Thumbnail,
            pixels: 10_000,
            ..canvas_whole(5)
        };
        assert_eq!(
            attribute(&[thumb], OOM),
            Attribution::Unattributed(Unattributed::NotTheCanvas(Surface::Thumbnail))
        );
    }

    /// ★★ Two uploads is a refusal, not a guess at the bigger one.
    ///
    /// The tempting rule — blame whichever was largest — is how a failed icon
    /// sheet becomes a lowered zoom ceiling on a page that was fine.
    #[test]
    fn several_uploads_are_not_attributed_even_when_one_is_far_bigger() {
        let big = Upload {
            pixels: 250_000_000,
            ..canvas_whole(1)
        };
        let small = Upload {
            surface: Surface::Icon,
            pixels: 1_024,
            raster: None,
        };
        assert_eq!(
            attribute(&[big, small], OOM),
            Attribution::Unattributed(Unattributed::Several(2))
        );
    }

    /// A lone region upload is reported as such, not blamed.
    ///
    /// Region rasters are a fixed multiple of the viewport at every zoom, so
    /// one failing is evidence about the machine — O221 — and lowering a zoom
    /// ceiling in response would take zoom away for a reason that had nothing
    /// to do with zoom.
    #[test]
    fn a_lone_region_upload_is_reported_but_not_blamed() {
        let region = Upload {
            raster: Some(Raster {
                page: 4,
                raster_scale: 8.0,
                whole_page: false,
            }),
            ..canvas_whole(4)
        };
        assert_eq!(
            attribute(&[region], OOM),
            Attribution::Unattributed(Unattributed::RegionOnly)
        );
    }

    /// An error with no upload behind it belongs to something else entirely —
    /// the font atlas being the likeliest, since it does not pass through here.
    #[test]
    fn no_uploads_means_the_error_came_from_elsewhere() {
        assert_eq!(
            attribute(&[], OOM),
            Attribution::Unattributed(Unattributed::NoUploads)
        );
    }

    /// ★ A drained set with no OOM in it still produces a verdict.
    ///
    /// `attribute` keys on `is_clean`, deliberately: a frame that raised an
    /// `INVALID_OPERATION` had something happen, and a trace reporting nothing
    /// would hide it. Acting on it is `out_of_memory`'s job, at the call site.
    #[test]
    fn an_unrelated_error_still_produces_a_verdict() {
        let other = native_gl::Drained {
            out_of_memory: false,
            count: 1,
            first_other: Some(eframe::glow::INVALID_OPERATION),
        };
        assert_eq!(
            attribute(&[canvas_whole(0)], other),
            Attribution::Blamed(canvas_whole(0))
        );
    }

    /// The record survives being written and read back through `ctx.data`.
    ///
    /// Pins the two halves together: a `record_*` that stashed under one id and
    /// a `take` that read another would silently report `NoUploads` forever,
    /// which is indistinguishable from a healthy session.
    #[test]
    fn a_recorded_upload_comes_back_out_once() {
        let ctx = egui::Context::default();
        let key = RenderKey::new(7, 4.0, true, 0, pdfcer_render::font::StrokeDisplay::Actual);
        record_raster(&ctx, Surface::Canvas, &key, 1_000, 2_000);
        record_other(&ctx, Surface::Icon, 32, 32);

        let first = take(&ctx);
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].surface, Surface::Canvas);
        assert_eq!(first[0].pixels, 2_000_000);
        assert_eq!(first[0].raster.map(|r| r.page), Some(7));
        assert!(first[0].raster.is_some_and(|r| r.whole_page));
        assert_eq!(first[1].raster, None);

        // ★ And is GONE — a record read twice would be attributed to two
        // frames, the second of which uploaded nothing.
        assert!(take(&ctx).is_empty());
    }
}
