//! The graphics-memory upload ledger: the uploads each frame ordered, and
//! whether a raised GL error can be blamed on one of them.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/pressure.md`.

use crate::renderworker::RenderKey;

/// Which surface ordered an upload.
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
    /// `render::raster::texture_from_pixels`.
    Thumbnail,
    /// The print dialog's page preview.
    Preview,
    /// One glyph of the icon sheet.
    Icon,
    /// A text draft's ink, drawn in the run's own font while it is typed.
    TextDraft,
}

/// The page-shaped part of an upload, present only when there is one.
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
    #[must_use]
    pub fn trace_fragment(self) -> String {
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
#[derive(Clone, Default)]
struct Ordered(Vec<Upload>);

/// The `ctx.data` slot. One per process; GL's error flag is global too.
fn slot() -> egui::Id {
    egui::Id::new("render::pressure::ordered")
}

/// Note that an upload of a **page raster** has been ordered this frame.
pub fn record_raster(ctx: &egui::Context, surface: Surface, key: &RenderKey, w: u32, h: u32) {
    record(
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
pub fn record_other(ctx: &egui::Context, surface: Surface, w: u32, h: u32) {
    record(
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
pub fn record(ctx: &egui::Context, upload: Upload) {
    ctx.data_mut(|d| d.get_temp_mut_or_default::<Ordered>(slot()).0.push(upload));
}

/// Take and clear what the previous frame ordered.
#[must_use]
pub fn take(ctx: &egui::Context) -> Vec<Upload> {
    ctx.data_mut(|d| std::mem::take(&mut d.get_temp_mut_or_default::<Ordered>(slot()).0))
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

    /// The only shape that may be blamed: one upload, the canvas's, whole.
    #[test]
    fn one_whole_page_canvas_upload_is_blamed() {
        assert_eq!(
            attribute(&[canvas_whole(3)], OOM),
            Attribution::Blamed(canvas_whole(3))
        );
    }

    /// A thumbnail is NOT the canvas, even though it is a whole page.
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

    /// Two uploads is a refusal, not a guess at the bigger one.
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

    /// A drained set with no OOM in it still produces a verdict.
    #[test]
    fn an_unrelated_error_still_produces_a_verdict() {
        let other = native_gl::Drained {
            out_of_memory: false,
            count: 1,
            first_other: Some(0x0502), // GL_INVALID_OPERATION,
        };
        assert_eq!(
            attribute(&[canvas_whole(0)], other),
            Attribution::Blamed(canvas_whole(0))
        );
    }

    /// The record survives being written and read back through `ctx.data`.
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

        // And is GONE — a record read twice would be attributed to two
        // frames, the second of which uploaded nothing.
        assert!(take(&ctx).is_empty());
    }
}
