//! # `canvas::cursor` — pdfcer's own crosshair, because the platform's is invisible
//!
//! One public function, [`crosshair`], returning an RGBA bitmap for
//! `egui::Context::set_cursor_image`. It exists because of an operator report
//! on 2026-08-18:
//!
//! > *"The crosshairs when over the canvas are white making it hard to see
//! > them."*
//!
//! ## What was actually happening
//!
//! Nothing in this crate draws that crosshair. [`crate::canvas::tool`] asks for
//! `egui::CursorIcon::Crosshair`, egui-winit maps it to the platform's stock
//! crosshair, and on Windows that is `IDC_CROSS` — a **monochrome** cursor
//! whose colour is decided by the operator's mouse-pointer scheme and by the
//! accessibility pointer-colour setting, neither of which this application can
//! read or influence. On a white or inverted scheme it is a white cross, and a
//! white cross on white paper is not a cursor.
//!
//! So the fix is not a colour change. It is to **stop asking the platform for a
//! crosshair and supply one**, which egui 0.35 supports directly:
//! `Context::set_cursor_image` hands an RGBA bitmap to
//! `winit::window::CustomCursor`, and it is a real OS cursor — composited by
//! the window manager, so it does **not** lag the pointer and is **not**
//! clipped by our window, which are the two failures of drawing a cursor with
//! `egui::Painter` instead.
//!
//! ## ★ Why two tones rather than the inversion the operator expected
//!
//! The report guessed at the mechanism — *"I assume they change based on if
//! they are over a black or white or grey object"* — and that guess describes
//! how this used to work and no longer does anywhere.
//!
//! XOR/inverting cursors were a real facility: a monochrome cursor with an AND
//! mask and an XOR mask, where the XOR bits inverted whatever was underneath.
//! Windows still *accepts* such cursors and X11 had the same idea. What killed
//! them is compositing: a desktop compositor draws the cursor into a separate
//! layer and blends it, and there is no blend mode that means "invert the
//! contents of the layer beneath me". A `CustomCursor::from_rgba` bitmap is
//! straight RGBA and has no way to express inversion at all.
//!
//! What every application that needs a precise cursor does instead is a
//! **two-tone glyph**: a dark core with a light outline, or the reverse.
//! Photoshop's *Precise* cursor, Illustrator, GIMP, Inkscape and AutoCAD's
//! crosshair are all this. It is strictly better than inversion for the case
//! that motivates both — mid-grey, where an inverted cursor becomes *another
//! mid-grey* and disappears, while a black-cored white-haloed cross stays
//! legible.
//!
//! So: **black core, white halo, on every background.**
//!
//! ## ★ Why these two colours are not theme colours, and the gate agrees
//!
//! Every other colour in this application comes from `egui_shell::theme` and
//! `tools/gates/check-theme-colors.sh` enforces it. This one must not, and the
//! reason is specific rather than an exemption of convenience:
//!
//! **A theme colour is chosen to contrast with the application's own surfaces.
//! This cursor has to contrast with the operator's document**, which is
//! whatever a CAD exporter drew — including, on any given drawing, a region of
//! exactly the accent colour. A themed cursor would be invisible on the one
//! page that happened to match it, and there is no palette entry that can be
//! right about content pdfcer does not control.
//!
//! Black and white are the only pair with that property, which is why every
//! reference application converged on them. Nothing here constructs a
//! `Color32`; the bitmap is bytes, so the gate has nothing to say either way,
//! and this paragraph is the argument it would want if it did.
//!
//! ## ★ The centre gap is not decoration
//!
//! The arms stop short of the centre, leaving the target pixel and its
//! neighbours unobscured. A crosshair whose arms meet hides the very point it
//! is pointing at, which matters on a dimension pick or a snap — the operator
//! is aiming at a line one pixel wide. Same reasoning, same solution, as every
//! application listed above.
//!
//! ## Scaling, and why the bitmap is cached per size
//!
//! The bitmap is device pixels; the operator's UI scale and display DPI decide
//! how many of them a cursor should be. So it is generated at
//! `32 * pixels_per_point` and **cached by that pixel size**, because
//! egui-winit dedupes the upload to the OS by `Arc::as_ptr` — returning the
//! same `Arc` across frames means the cursor is converted to a platform handle
//! **once**, and returning a fresh one every frame would re-upload a bitmap at
//! sixty hertz.
//!
//! ## ★ The trap: `cursor_image` is STICKY between frames
//!
//! `egui::PlatformOutput::take` explicitly keeps both `cursor_icon` and
//! `cursor_image` across frames — *"sticky between frames"*, in its own
//! comment. And `egui-winit`'s `apply_cursor` prefers the **image** whenever
//! one is present, so a bitmap set once outlives every later `set_cursor_icon`
//! from anywhere in the application.
//!
//! Set it and never clear it and the crosshair follows the pointer onto the
//! ribbon, into the panels, over the scrollbars, and stays there after the
//! document is closed. That is why [`crate::app::frame`] clears it once per
//! frame **before** anything draws, and the canvas re-asserts it if it wants
//! it: one place resets, one place asks, and a frame in which the canvas does
//! not run cannot leave a stale cursor behind.

use std::sync::{Arc, Mutex, OnceLock};

use egui::CustomCursorImage;

/// The crosshair's logical size, in egui points, before UI scale.
///
/// 32 is the size of a standard Windows cursor and of the stock crosshair this
/// replaces, so an operator who has used the application before this change
/// sees the same-sized pointer with a different treatment rather than a
/// different pointer.
const LOGICAL_SIZE_PTS: f32 = 32.0;

/// The largest bitmap `winit::window::CustomCursor` will accept, per its own
/// `MAX_CURSOR_SIZE`. Named rather than inlined because exceeding it is not a
/// panic — it is a silent fall back to the platform crosshair, i.e. to the
/// defect this module exists to fix — so the clamp has to be deliberate.
const MAX_CURSOR_PX: u32 = 2048;

/// Arm length from the centre, in points: where the drawn part of each arm
/// ends.
const ARM_PTS: f32 = 12.0;

/// Gap radius, in points: nothing is drawn within this of the centre.
const GAP_PTS: f32 = 3.0;

/// The generated bitmaps, keyed by their pixel size.
///
/// A `Mutex<Vec<…>>` rather than a map: there are at most a handful of distinct
/// UI scales in a session and usually exactly one, so a linear scan of a
/// two-element vector is cheaper than hashing and much easier to read.
/// Contention is nil — this is touched once per frame from the UI thread.
static CACHE: OnceLock<Mutex<Vec<(u32, CustomCursorImage)>>> = OnceLock::new();

/// The crosshair cursor bitmap for this scale factor.
///
/// `pixels_per_point` is `egui::Context::pixels_per_point` — the product of the
/// display's scale factor and the operator's UI-scale preference.
///
/// Returns the **same `Arc`** for repeated calls at the same size, which is
/// what makes egui-winit's `Arc::as_ptr` dedupe work and keeps the cursor from
/// being re-uploaded to the OS every frame.
///
/// # Why it cannot fail
///
/// The size is clamped into `1..=MAX_CURSOR_PX` and the buffer is allocated
/// from it, so the length invariant `CustomCursorImage` requires
/// (`size[0] * size[1] * 4`) holds by construction. A non-finite or absurd
/// `pixels_per_point` — which egui does not produce, but which a preference
/// file could once have contained — lands on the clamp rather than on an
/// allocation the size of a display.
#[must_use]
pub fn crosshair(pixels_per_point: f32) -> CustomCursorImage {
    let scale = if pixels_per_point.is_finite() && pixels_per_point > 0.0 {
        pixels_per_point
    } else {
        1.0
    };
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the value is clamped into 1..=MAX_CURSOR_PX on the line below, so neither can occur" // ui-text-exempt: lint justification, never displayed
    )]
    let size = ((LOGICAL_SIZE_PTS * scale).round() as u32).clamp(1, MAX_CURSOR_PX);

    let cache = CACHE.get_or_init(|| Mutex::new(Vec::new()));
    // `unwrap_or_else(PoisonError::into_inner)` rather than `unwrap`: a panic
    // while this lock was held would otherwise cost the operator their cursor
    // for the rest of the session, and the worst a possibly-stale cache can do
    // is hand back a correct bitmap that was generated by the thread that
    // panicked.
    let mut cache = cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some((_, image)) = cache.iter().find(|(key, _)| *key == size) {
        return image.clone();
    }
    let image = render(size, scale);
    cache.push((size, image.clone()));
    image
}

/// **How far the I-beam is turned from upright**, in whole degrees, folded into
/// `0..180` and quantised.
///
/// # ★★ Why the cursor has an angle at all
///
/// The operator, 2026-08-26, on a vertical stamp in a title block:
///
/// > *"In Adobe when I hover over it the I cursor re-orients itself to match
/// > the text orientation […] as it is now the I cursor doesn't reorient."*
///
/// Acrobat is right and this is the convention, not an embellishment: the
/// I-beam's whole job is to say *"text flows this way and the caret will land
/// between two glyphs"*, and over a 90° stamp an upright beam says it about the
/// wrong axis.
///
/// # ★ Why pdfcer can do this at all, where most applications cannot
///
/// `egui::CursorIcon` has no rotated I-beam and neither does Win32 —
/// `IDC_IBEAM` is one fixed monochrome bitmap. Acrobat ships its own artwork
/// for each orientation, and so, as it happens, does this shell: the module
/// already generates its I-beam as an RGBA bitmap for
/// `Context::set_cursor_image`, because the platform's was invisible on white
/// paper. **The rotation is free on top of a mechanism that had to exist
/// anyway.** Had this application still been asking the platform for
/// `CursorIcon::Text`, the operator's request would have been unbuildable.
///
/// # The quantisation, and the two reasons for it
///
/// Five degrees. Each distinct angle is a separate generated bitmap held in a
/// cache, and each is uploaded to the OS as a platform cursor handle the first
/// time it is used, so an unquantised angle would mean a new handle every time
/// the pointer crossed a slightly different line. Five degrees is also below
/// what anyone can see in a 32-pixel glyph: at that size one degree moves the
/// beam's tip by a quarter of a pixel.
///
/// Folded into `0..180` because the glyph is symmetric under a half turn — a
/// beam at 200° and one at 20° are the same pixels — which halves the cache for
/// nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tilt(u16);

impl Tilt {
    /// The default: an upright beam, for text that runs along the page.
    pub const UPRIGHT: Self = Self(0);

    /// The quantisation step, in degrees. See the type's docs.
    const STEP: u16 = 5;

    /// The tilt nearest `degrees`, folded and quantised.
    ///
    /// Takes any real angle — including negative ones, which is what an
    /// `atan2` in a Y-down space produces for text running up the page — so no
    /// caller has to normalise before asking.
    ///
    /// A non-finite angle answers [`Self::UPRIGHT`] rather than panicking: it
    /// can only arise from a degenerate page transform, and an upright cursor
    /// on a broken page is a far better outcome than a crash in the middle of a
    /// pointer move.
    #[must_use]
    pub fn nearest(degrees: f32) -> Self {
        if !degrees.is_finite() {
            return Self::UPRIGHT;
        }
        let folded = degrees.rem_euclid(180.0);
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "folded is in 0..180 and the quotient is in 0..36" // ui-text-exempt: lint justification, never displayed
        )]
        let steps = (folded / f32::from(Self::STEP)).round() as u16;
        // 180° rounds up to step 36, which is 0° again — the fold, applied once
        // more after rounding rather than trusted to have survived it.
        Self((steps * Self::STEP) % 180)
    }

    /// The angle in degrees.
    #[must_use]
    pub const fn degrees(self) -> u16 {
        self.0
    }

    /// Whether this is the ordinary upright beam.
    #[must_use]
    pub const fn is_upright(self) -> bool {
        self.0 == 0
    }
}

/// The two-tone I-beam at `tilt`, cached per `(pixel size, tilt)` exactly as
/// [`crosshair`] is cached per pixel size.
///
/// # ★ Why an I-beam and not simply a smaller crosshair
///
/// Because the two answer different questions and the shape IS the answer. A
/// crosshair says *"the point under the intersection is what you are picking"*;
/// an I-beam says *"text flows this way, and the caret will land between two
/// glyphs"*. Its serifs are not decoration — they are what makes a one-pixel
/// bar findable on a page of one-pixel strokes, which a CAD drawing is entirely
/// made of.
///
/// And it is exactly *because* the shape carries that meaning that it has to
/// turn: see [`Tilt`].
#[must_use]
pub fn ibeam(pixels_per_point: f32, tilt: Tilt) -> CustomCursorImage {
    let scale = if pixels_per_point.is_finite() && pixels_per_point > 0.0 {
        pixels_per_point
    } else {
        1.0
    };
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "clamped into 1..=MAX_CURSOR_PX on the same line" // ui-text-exempt: lint justification, never displayed
    )]
    let size = ((LOGICAL_SIZE_PTS * scale).round() as u32).clamp(1, MAX_CURSOR_PX);

    let cache = IBEAM_CACHE.get_or_init(|| Mutex::new(Vec::new()));
    let mut cache = cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some((_, _, img)) = cache
        .iter()
        .find(|(key, angle, _)| *key == size && *angle == tilt)
    {
        return img.clone();
    }
    let img = render_ibeam(size, scale, tilt);
    cache.push((size, tilt, img.clone()));
    img
}

/// Cache for [`ibeam`], separate from the crosshair's.
///
/// Two caches rather than one keyed by shape: each holds at most a handful of
/// entries, and a shared one would need a compound key for no saving. The cost
/// of getting a compound key wrong is handing back the wrong glyph.
///
/// ★ Keyed by `(size, tilt)` since 2026-08-26. **A cache keyed only by size
/// would be worse than no cache at all**: the first angle asked for would be
/// stored and every later angle would silently receive it, so the cursor would
/// appear to reorient once and then never again — a defect that looks like the
/// feature half-working rather than like a cache bug.
///
/// Unbounded, and that is safe rather than lucky: [`Tilt`] quantises to five
/// degrees and folds into a half turn, so there are at most 36 angles, and in
/// practice a document has one or two.
static IBEAM_CACHE: OnceLock<Mutex<Vec<(u32, Tilt, CustomCursorImage)>>> = OnceLock::new();

/// Draw the I-beam: a dark bar with serifs, haloed in light, turned by `tilt`.
///
/// Halo first then core, for the reason [`render`] gives — drawing them the
/// other way round leaves a light glyph with a dark outline, which is thinner
/// in its dark part than its light one and reads as blurry.
///
/// # ★ Drawn by inverse rotation, not by rotating a drawn bitmap
///
/// Each destination pixel is mapped **back** into the beam's own upright frame
/// and tested for membership there. Rotating an already-drawn bitmap forward
/// would leave unpainted pixels wherever two source pixels landed on the same
/// destination — a beam full of holes at every angle that is not a multiple of
/// 90° — and closing them would mean resampling, which on a two-tone glyph
/// whose entire value is a crisp one-pixel core is precisely the wrong tool.
///
/// The membership test is the same shape the upright version drew directly: a
/// bar `width` across and `2 × half_height` along, plus a serif slab `width`
/// deep at each end. Substituting `tilt = 0` gives back the original glyph
/// pixel for pixel, which is the check that this generalises rather than
/// replaces it.
fn render_ibeam(size: u32, scale: f32, tilt: Tilt) -> CustomCursorImage {
    let mut rgba = vec![0u8; (size as usize) * (size as usize) * 4];
    let px = |pts: f32| (pts * scale).round().max(1.0) as i32;
    #[allow(
        clippy::cast_possible_wrap,
        reason = "size is clamped to MAX_CURSOR_PX, far inside i32" // ui-text-exempt: lint justification, never displayed
    )]
    let n = size as i32;
    let centre = n / 2;
    let half_height = px(IBEAM_HEIGHT_PTS / 2.0).min(centre);
    let serif = px(IBEAM_SERIF_PTS).min(centre - 1).max(1);
    // Same asymmetry the crosshair uses and for the same reason: the dark core
    // stays a hairline so the operator can see exactly which glyph boundary the
    // caret will land on, and the light halo takes the extra pixels because its
    // whole job is to be seen against whatever is underneath.
    let core = px(IBEAM_CORE_PTS).max(1);
    let halo = core + 2;

    let radians = f32::from(tilt.degrees()).to_radians();
    let (sin, cos) = radians.sin_cos();

    let mut put = |x: i32, y: i32, white: bool| {
        if x < 0 || y < 0 || x >= n || y >= n {
            return;
        }
        let at = ((y * n + x) as usize) * 4;
        let value = if white { 0xFF } else { 0x00 };
        rgba[at] = value;
        rgba[at + 1] = value;
        rgba[at + 2] = value;
        rgba[at + 3] = 0xFF;
    };
    // ★★ How wide a pixel is, measured across the beam's own axes.
    //
    // **Without it a tilted hairline comes out dotted.** The core is one device
    // pixel wide at ordinary scales (deliberately — see below), so a bare test
    // of `|across| <= 0` selects at most one pixel per scanline; where the beam
    // is not axis-aligned, consecutive scanlines can then step two columns
    // apart and the line breaks into dashes.
    //
    // ★ The value is derived rather than fudged, and the derivation is what
    // stops it being too generous. Walking one step along the beam's dominant
    // axis changes `across` by `max(|cos|, |sin|)`, so a tolerance of **half
    // that** guarantees at least one pixel per scanline and nothing more:
    //
    // | tilt | `max(\|cos\|,\|sin\|)` | spread | a one-pixel core comes out as |
    // |---|---|---|---|
    // | 0° / 90° | 1.0 | 0.5 | one pixel — exactly the columns the upright version drew |
    // | 45° | 0.707 | 0.354 | one pixel per anti-diagonal: a clean 1-px staircase |
    //
    // The first draft used `0.5 × (|cos| + |sin|)` — the pixel's full projected
    // width — and it is 41 % too big at 45°: the ASCII preview showed a
    // three-pixel-thick core where the upright glyph has a hairline, which
    // defeats the whole reason the core is a hairline (see below). That is why
    // `preview::ibeam_ascii` exists.
    let spread = 0.5 * cos.abs().max(sin.abs());
    let mut bar = |width: i32, white: bool| {
        let half = (width / 2) as f32;
        // Where the serif slab begins, as a distance from the centre: the
        // outermost `width` rows at each end, which is what the upright version
        // drew by counting `dy in 0..width` inwards from the tip.
        let serif_from = f32::from(i16::try_from((half_height - width + 1).max(0)).unwrap_or(0));
        for y in 0..n {
            for x in 0..n {
                // Into the beam's own upright frame. Offsets are whole pixels
                // from the hotspot, exactly as the upright version indexed
                // them, so the glyph stays centred on the hotspot at every
                // angle rather than drifting half a pixel.
                let dx = (x - centre) as f32;
                let dy = (y - centre) as f32;
                let across = dx.mul_add(cos, dy * sin);
                let along = dx.mul_add(-sin, dy * cos);
                let within_length = along.abs() <= half_height as f32 + spread;
                let in_bar = across.abs() <= half + spread && within_length;
                let in_serif = across.abs() <= serif as f32 + spread
                    && within_length
                    && along.abs() >= serif_from - spread;
                if in_bar || in_serif {
                    put(x, y, white);
                }
            }
        }
    };
    bar(halo, true);
    bar(core, false);

    CustomCursorImage {
        rgba: Arc::from(rgba),
        #[allow(
            clippy::cast_possible_truncation,
            reason = "size is clamped to MAX_CURSOR_PX = 2048, inside u16" // ui-text-exempt: lint justification, never displayed
        )]
        size: [size as u16, size as u16],
        #[allow(
            clippy::cast_possible_truncation,
            reason = "centre is half of a value clamped to 2048" // ui-text-exempt: lint justification, never displayed
        )]
        hotspot: [centre as u16, centre as u16],
    }
}

/// Total height of the I-beam, in logical points.
const IBEAM_HEIGHT_PTS: f32 = 15.0;
/// How far each serif reaches from the centre line.
const IBEAM_SERIF_PTS: f32 = 3.0;
/// Width of the dark core bar.
const IBEAM_CORE_PTS: f32 = 1.0;

/// Draw the glyph into a fresh buffer.
///
/// # The order is load-bearing: halo first, then core
///
/// The halo is drawn as a *wider* arm and the core is drawn over the middle of
/// it. Drawing them the other way round would put the halo's own pixels over
/// the core and leave a white cross with a black outline — legible, but the
/// opposite of the convention every reference application uses, and thinner in
/// its dark part than in its light one, which reads as blurry.
fn render(size: u32, scale: f32) -> CustomCursorImage {
    // Straight (non-premultiplied) RGBA, four bytes per pixel, fully
    // transparent everywhere the glyph is not. `CustomCursorImage` documents
    // the encoding and `winit::window::CustomCursor::from_rgba` requires it.
    //
    // NOT A THEME COLOUR: black and white, deliberately and permanently — see
    // the module header. These contrast with the operator's DOCUMENT, which is
    // content pdfcer does not control and no palette can be right about.
    let mut rgba = vec![0_u8; (size as usize) * (size as usize) * 4];

    // The centre is the hotspot, and it must be a whole pixel: a hotspot half a
    // pixel away from the crossing point puts every click one pixel from where
    // the operator aimed, which is invisible until somebody dimensions with it.
    let centre = (size / 2) as i32;

    let px = |points: f32| -> i32 { (points * scale).round().max(1.0) as i32 };

    // ★ The core stays a HAIRLINE and the halo grows. That asymmetry is the
    // design, not an oversight in the scaling.
    //
    // A crosshair is an aiming device: its value is that the operator can see
    // exactly which pixel they are about to pick, and every extra pixel of core
    // takes that away. So the core is one device pixel at ordinary scales and
    // reaches three only past 200 %, where one device pixel has become too fine
    // to see at all. Both are ODD widths, which is required rather than
    // preferred — an even-width line has no centre pixel, so the hotspot would
    // sit half a pixel off the line it is meant to be the centre of.
    //
    // The halo has the opposite job: it exists to be seen against whatever is
    // underneath, so it takes the pixels the core does not.
    let core_half = i32::from(scale >= 2.0);
    let halo_half = core_half + (scale * 0.75).round().max(1.0) as i32;
    let gap = px(GAP_PTS);
    let arm = px(ARM_PTS).min(centre);

    let mut put = |x: i32, y: i32, white: bool| {
        if x < 0 || y < 0 || x >= size as i32 || y >= size as i32 {
            return;
        }
        let at = ((y as usize) * (size as usize) + (x as usize)) * 4;
        let value = if white { 0xFF } else { 0x00 };
        rgba[at] = value;
        rgba[at + 1] = value;
        rgba[at + 2] = value;
        rgba[at + 3] = 0xFF;
    };

    // Each of the four arms, twice: the halo runs one pixel further at both
    // ends than the core so the core's tip is capped rather than left bare.
    for (white, half, from, to) in [
        (true, halo_half, gap - px(1.0), arm + px(1.0)),
        (false, core_half, gap, arm),
    ] {
        for along in from.max(0)..=to {
            for across in -half..=half {
                // Horizontal arms, left and right.
                put(centre - along, centre + across, white);
                put(centre + along, centre + across, white);
                // Vertical arms, up and down.
                put(centre + across, centre - along, white);
                put(centre + across, centre + along, white);
            }
        }
    }

    CustomCursorImage {
        rgba: Arc::from(rgba),
        #[allow(
            clippy::cast_possible_truncation,
            reason = "size is clamped to MAX_CURSOR_PX = 2048, which fits u16" // ui-text-exempt: lint justification, never displayed
        )]
        size: [size as u16, size as u16],
        #[allow(
            clippy::cast_possible_truncation,
            reason = "centre is size/2 and size fits u16" // ui-text-exempt: lint justification, never displayed
        )]
        hotspot: [centre as u16, centre as u16],
    }
}

/// Whether the crosshair was wanted on the last frame that asked.
///
/// `u32`: `0` means "not wanted", anything else is the pixel size that was
/// applied. Only [`apply`] touches it.
static LAST_APPLIED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Ask for the crosshair, or don't, and **trace the transition**.
///
/// Called once per frame from [`crate::canvas::interact`], which is the one
/// place that knows whether the cursor's answer is a crosshair.
///
/// # ★ Why this is traced at all, when nothing else about a cursor is
///
/// **A cursor cannot be verified by screenshot.** Windows composites the
/// pointer separately from window contents, so `BitBlt` and `PrintWindow` — the
/// two ways `ui-verify` captures a window — return an image with **no cursor
/// in it**. There is no pixel oracle available here at any price, which is
/// unusual for this project: R1's normal answer is "drive it and look at the
/// picture", and for this one feature the picture cannot contain the answer.
///
/// So the trace is the only machine-readable evidence that the wiring works,
/// and the wiring has two failure modes worth naming:
///
/// | failure | what the operator sees |
/// |---|---|
/// | never applied | the platform's crosshair, i.e. the reported defect, unchanged |
/// | never cleared | pdfcer's crosshair over the ribbon, the panels and the scrollbars, and still there after the document closes |
///
/// The second is the one this exists for. `cursor_image` is **sticky between
/// frames** and `egui-winit` prefers it over every later `set_cursor_icon`, so
/// forgetting the clear is not a small bug — see the module header.
///
/// # On change only
///
/// A line per frame at sixty hertz is not a diagnostic, it is a denial of
/// service on the reader. This emits when the answer *changes*, which is what
/// a reader is looking for: `cursor-crosshair on px=32` when a tool is armed,
/// `cursor-crosshair off` when it is retired.
///
/// # The one gap, stated
///
/// A frame in which the canvas does not run at all — no document open — does
/// not reach here, so a transition to "off" caused by *closing the document*
/// is not traced. The cursor is still cleared: [`crate::app::frame`] does that
/// unconditionally and earlier, which is exactly why the clear lives there and
/// not here.
/// Which of pdfcer's own cursors a frame wants, if any.
///
/// # ★★ The I-beam is here for the SAME reason the crosshair is, reported
/// the same way, three weeks apart
///
/// 2026-08-18: *"the crosshairs when over the canvas are white making it hard
/// to see them."* 2026-08-19: *"the I cursor turns white for text selection so
/// I cant see it on a white background."*
///
/// One cause. `IDC_IBEAM` is a **monochrome** stock cursor exactly as
/// `IDC_CROSS` is, coloured by the operator's pointer scheme, and a white
/// I-beam over white paper is not a cursor. The fix that worked for the
/// crosshair works here unchanged: stop asking the platform, supply a two-tone
/// glyph with a dark core and a light halo.
///
/// ★ It is filed as a defect in this module rather than a new feature
/// because the first fix should have been made here. The header already argued
/// that the platform's monochrome cursors are unusable over a document, and
/// then fixed exactly one of them — the one that had been reported. Every
/// other `CursorIcon` this application asks for over the canvas has the same
/// exposure, and the two that matter over *paper* are these two.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// The picking crosshair, for an armed tool.
    Crosshair,
    /// The text I-beam, for selecting or editing text on the page, **turned to
    /// match the text under the pointer**.
    ///
    /// The angle rides on the variant rather than beside it because a shape and
    /// its orientation are one answer to one question: [`apply`] keys its cache
    /// and its change detection on the whole `Shape`, and a tilt stored
    /// separately would let the two disagree — the classic form of which is a
    /// cursor that turns once and then never again. See [`Tilt`].
    Ibeam(Tilt),
}

impl Shape {
    /// The `egui` icon this replaces, so one call site can decide.
    #[must_use]
    pub const fn of(icon: egui::CursorIcon) -> Option<Self> {
        match icon {
            egui::CursorIcon::Crosshair => Some(Self::Crosshair),
            // Upright, always: this function knows only which *stock* icon the
            // frame asked for, and nothing about what is under the pointer. The
            // tilt is applied afterwards by the one caller that has the page's
            // text — `canvas::interact` — because turning the beam requires an
            // extraction and a pure icon mapping must not reach for one.
            egui::CursorIcon::Text => Some(Self::Ibeam(Tilt::UPRIGHT)),
            _ => None,
        }
    }

    /// The trace word, so a reader can tell the two apart.
    const fn label(self) -> &'static str {
        match self {
            // ui-text-exempt: diagnostic trace values, never displayed
            Self::Crosshair => "crosshair",
            Self::Ibeam(_) => "ibeam",
        }
    }

    /// How far this shape is turned from upright, in degrees.
    ///
    /// Zero for the crosshair, which is rotationally symmetric to within its
    /// own arms and would say nothing by turning.
    const fn degrees(self) -> u16 {
        match self {
            Self::Crosshair => 0,
            Self::Ibeam(tilt) => tilt.degrees(),
        }
    }
}

/// The bitmap for a shape at this scale, cached per `(shape, size)`.
#[must_use]
pub fn image(shape: Shape, pixels_per_point: f32) -> CustomCursorImage {
    match shape {
        Shape::Crosshair => crosshair(pixels_per_point),
        Shape::Ibeam(tilt) => ibeam(pixels_per_point, tilt),
    }
}

pub fn apply(ctx: &egui::Context, wanted: Option<Shape>) {
    use std::sync::atomic::Ordering;

    let size = if let Some(shape) = wanted {
        let image = image(shape, ctx.pixels_per_point());
        let size = u32::from(image.size[0]);
        ctx.set_cursor_image(Some(image));
        // The shape AND its angle ride in the low bits beside the size, so a
        // change of either at one scale is still a change. Without the shape
        // bit, switching from crosshair to I-beam at the same scale would trace
        // nothing; without the angle, an I-beam turning from upright to 90°
        // would trace nothing — and the angle is the half a screenshot cannot
        // check, because Windows composites the pointer separately and
        // `ui-verify`'s window capture contains no cursor at any price.
        //
        // `+ 1` on the shape bit rather than a bare boolean, so the whole
        // packed value stays non-zero for any real cursor: zero is reserved for
        // "none wanted", and an upright crosshair at a zero-width scale must
        // not collide with it.
        (size * 512 + u32::from(shape.degrees()) * 2) + u32::from(matches!(shape, Shape::Ibeam(_)))
    } else {
        // Deliberately does NOT clear: `crate::app::frame` has already done it
        // for this frame, before anything drew. Clearing again here would be a
        // second owner of the same state, and the frame-level one is the only
        // one that covers a frame this function never reaches.
        0
    };

    if LAST_APPLIED.swap(size, Ordering::Relaxed) != size {
        crate::diag::trace(move || match wanted {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            None => "cursor-custom off".to_owned(),
            Some(shape) => format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "cursor-custom on shape={} deg={} px={}",
                shape.label(),
                shape.degrees(),
                size / 512
            ),
        });
    }
}

#[cfg(test)]
mod tests {

    /// ★★ The I-beam has a DARK core, which is the whole of the operator's
    /// report.
    ///
    /// *"The I cursor turns white for text selection so I cant see it on a
    /// white background."* Same defect as the crosshair's, three weeks apart,
    /// same cause: `IDC_IBEAM` is a monochrome stock cursor coloured by the
    /// operator's pointer scheme.
    ///
    /// So the assertion is not "it renders" — it is that the **centre pixel is
    /// black**, because a light-cored glyph would satisfy every other test here
    /// and reproduce the bug exactly.
    #[test]
    fn the_ibeam_core_is_dark_and_its_halo_is_light() {
        let img = ibeam(1.0, Tilt::UPRIGHT);
        let n = usize::from(img.size[0]);
        let at = |x: usize, y: usize| {
            let i = (y * n + x) * 4;
            (img.rgba[i], img.rgba[i + 3])
        };
        let c = n / 2;
        assert_eq!(at(c, c), (0x00, 0xFF), "the core must be opaque black");
        // One pixel out along the bar's width is halo — the core is a
        // HAIRLINE by design, exactly as the crosshair's is, so the operator
        // can see which glyph boundary the caret will land on.
        assert_eq!(
            at(c + 1, c),
            (0xFF, 0xFF),
            "the halo must be opaque white so the core is legible on dark ink too"
        );
        assert_eq!(
            at(c + 2, c).1,
            0x00,
            "and the glyph must end there rather than smearing"
        );
    }

    /// The glyph is taller than it is wide, which is what makes it an I-beam.
    ///
    /// A square two-tone blob would pass the colour test and be a worse
    /// crosshair. The shape carries the meaning: text flows this way, and the
    /// caret lands between two glyphs.
    #[test]
    fn the_ibeam_is_a_bar_and_not_a_blob() {
        let img = ibeam(1.0, Tilt::UPRIGHT);
        let n = usize::from(img.size[0]);
        let opaque = |x: usize, y: usize| img.rgba[(y * n + x) * 4 + 3] == 0xFF;
        let c = n / 2;
        let tall = (0..n).filter(|&y| opaque(c, y)).count();
        let wide = (0..n).filter(|&x| opaque(x, c)).count();
        assert!(
            tall > wide * 2,
            "an I-beam is a vertical bar: {tall} tall against {wide} wide"
        );
    }

    /// The two shapes map from the two `egui` icons and nothing else does.
    ///
    /// ★ The negative half matters: every other `CursorIcon` this application
    /// asks for is over CHROME, where the platform's stock cursor is correct
    /// and a custom one would be wrong. Only the two drawn over the operator's
    /// document need replacing.
    #[test]
    fn only_the_two_cursors_drawn_over_paper_are_replaced() {
        assert_eq!(
            Shape::of(egui::CursorIcon::Crosshair),
            Some(Shape::Crosshair)
        );
        assert_eq!(
            Shape::of(egui::CursorIcon::Text),
            Some(Shape::Ibeam(Tilt::UPRIGHT)),
            "the stock icon maps to an UPRIGHT beam; the tilt is applied later, \
             by the caller that knows what the pointer is over"
        );
        for stock in [
            egui::CursorIcon::Default,
            egui::CursorIcon::PointingHand,
            egui::CursorIcon::ResizeVertical,
            egui::CursorIcon::Grab,
        ] {
            assert_eq!(Shape::of(stock), None, "{stock:?} is chrome, not paper");
        }
    }
    use super::*;

    /// The four values `CustomCursorImage` promises about itself.
    ///
    /// The length invariant is the one that matters: `CustomCursor::from_rgba`
    /// **rejects** a buffer whose length is not `w * h * 4`, and egui-winit's
    /// response to a rejection is to log a warning and fall back to the
    /// platform cursor — i.e. silently back to the defect this module exists to
    /// fix. A wrong length would therefore look exactly like the module not
    /// being wired up.
    #[test]
    fn the_bitmap_matches_the_size_it_declares() {
        for ppp in [1.0_f32, 1.25, 1.5, 2.0, 3.0] {
            let image = crosshair(ppp);
            let (w, h) = (usize::from(image.size[0]), usize::from(image.size[1]));
            assert_eq!(
                image.rgba.len(),
                w * h * 4,
                "the buffer must be exactly w * h * 4 at {ppp}"
            );
            assert!(w > 0 && h > 0, "a zero-sized cursor at {ppp}");
            assert!(
                u32::from(image.size[0]) <= MAX_CURSOR_PX,
                "winit rejects anything over {MAX_CURSOR_PX} and falls back to the platform \
                 cursor, which is the defect"
            );
        }
    }

    /// ★ The hotspot is the crossing point, and it is not painted.
    ///
    /// Two properties in one test because they are the same claim from two
    /// sides: the hotspot pixel is the geometric centre, and the centre gap
    /// means the operator can see what they are aiming at. A regression in
    /// either — an off-by-one hotspot, or a gap of zero — is invisible on
    /// screen and shows up as dimensions that are consistently one pixel out.
    #[test]
    fn the_hotspot_is_the_centre_and_the_centre_is_clear() {
        let image = crosshair(1.0);
        let size = usize::from(image.size[0]);
        let (hx, hy) = (usize::from(image.hotspot[0]), usize::from(image.hotspot[1]));
        assert_eq!(hx, size / 2, "the hotspot must be the centre column");
        assert_eq!(hy, size / 2, "the hotspot must be the centre row");

        let alpha_at = |x: usize, y: usize| image.rgba[(y * size + x) * 4 + 3];
        assert_eq!(
            alpha_at(hx, hy),
            0,
            "the pixel under the hotspot must be clear — a crosshair that paints its own \
             target hides the line the operator is aiming at"
        );
    }

    /// ★ Both tones are present, and the dark one is surrounded by the light.
    ///
    /// This is the whole feature: a cursor of one tone is exactly the defect
    /// reported. Sampling the arm rather than counting pixels, because what
    /// matters is the *arrangement* — a bitmap that happened to contain both
    /// colours somewhere would satisfy a count and could still be illegible.
    #[test]
    fn an_arm_is_a_dark_core_inside_a_light_halo() {
        let image = crosshair(1.0);
        let size = usize::from(image.size[0]);
        let centre = size / 2;
        let sample = |x: usize, y: usize| {
            let at = (y * size + x) * 4;
            (image.rgba[at], image.rgba[at + 3])
        };

        // Along the upward arm, a few pixels clear of the gap.
        let y = centre - 8;
        assert_eq!(
            sample(centre, y),
            (0x00, 0xFF),
            "the core of an arm must be opaque black"
        );
        assert_eq!(
            sample(centre - 1, y),
            (0xFF, 0xFF),
            "the pixel beside the core must be opaque white — without the halo the cursor is \
             invisible on a dark page, which is the operator's report"
        );
        assert_eq!(
            sample(centre + 1, y),
            (0xFF, 0xFF),
            "…and on the other side too"
        );
    }

    /// ★ Repeated calls at one scale return the SAME allocation.
    ///
    /// `egui-winit` dedupes its upload to the OS by `Arc::as_ptr`. A fresh
    /// `Arc` per frame would convert a bitmap to a platform cursor handle sixty
    /// times a second — and it would still *work*, which is why this is worth a
    /// test: the symptom is a performance cost nobody would attribute to the
    /// cursor.
    #[test]
    fn the_same_scale_returns_the_same_allocation() {
        let a = crosshair(1.0);
        let b = crosshair(1.0);
        assert!(
            Arc::ptr_eq(&a.rgba, &b.rgba),
            "two calls at one scale must share the cached buffer, or egui-winit re-uploads the \
             cursor to the OS on every frame"
        );
        let c = crosshair(2.0);
        assert!(
            !Arc::ptr_eq(&a.rgba, &c.rgba),
            "a different scale is a different bitmap"
        );
    }

    // =======================================================================
    // ★★ The tilt — the operator's 2026-08-26 report
    // =======================================================================

    /// The quantiser folds, rounds and refuses nonsense.
    ///
    /// Each row is a case that would produce a visible defect on its own: an
    /// unfolded angle doubles the cache and uploads a duplicate cursor to the
    /// OS; a 180° that survived rounding would be a distinct entry drawing
    /// identical pixels; and a non-finite angle — reachable from a degenerate
    /// page transform — must not panic in the middle of a pointer move.
    #[test]
    fn the_tilt_quantises_and_folds() {
        for (given, expect) in [
            (0.0_f32, 0),
            (2.0, 0),
            (3.0, 5),
            (89.0, 90),
            (90.0, 90),
            (92.4, 90),
            // A half turn is the same glyph, so it folds to upright.
            (180.0, 0),
            (185.0, 5),
            // Negative, which is what an `atan2` in a Y-down space gives for
            // text running UP the page — the operator's own case.
            (-90.0, 90),
            (-5.0, 175),
            (f32::NAN, 0),
            (f32::INFINITY, 0),
        ] {
            assert_eq!(
                Tilt::nearest(given).degrees(),
                expect,
                "Tilt::nearest({given}) should be {expect}°"
            );
            assert!(
                Tilt::nearest(given).degrees() < 180,
                "every tilt must fold into 0..180"
            );
        }
    }

    /// ★★★ **A 90° I-beam is a HORIZONTAL bar**, which is the whole of what the
    /// operator asked for.
    ///
    /// The upright test above this one asserts `tall > wide * 2`; this asserts
    /// the exact reverse on the same glyph at 90°. Asserting both is what makes
    /// the pair evidence: a renderer that ignored the tilt would pass the first
    /// and fail this, and one that rotated everything unconditionally would do
    /// the opposite.
    #[test]
    fn a_quarter_turned_ibeam_is_a_horizontal_bar() {
        let img = ibeam(1.0, Tilt::nearest(90.0));
        let n = usize::from(img.size[0]);
        let opaque = |x: usize, y: usize| img.rgba[(y * n + x) * 4 + 3] == 0xFF;
        let c = n / 2;
        let tall = (0..n).filter(|&y| opaque(c, y)).count();
        let wide = (0..n).filter(|&x| opaque(x, c)).count();
        assert!(
            wide > tall * 2,
            "a 90° I-beam should lie on its side: {tall} tall against {wide} wide"
        );
    }

    /// ★★ **And it keeps its dark core**, at every angle.
    ///
    /// The operator's *other* cursor report — *"the I cursor turns white for
    /// text selection so I cant see it on a white background"* — is a property
    /// of the glyph, not of its orientation, and a rotation implemented by
    /// resampling would soften exactly this pixel into grey. Checked at four
    /// angles including one that is not a multiple of 90°, because that is
    /// where a resampling implementation would first show.
    #[test]
    fn the_core_stays_dark_at_every_angle() {
        for degrees in [0.0_f32, 30.0, 90.0, 135.0] {
            let img = ibeam(1.0, Tilt::nearest(degrees));
            let n = usize::from(img.size[0]);
            let c = n / 2;
            let i = (c * n + c) * 4;
            assert_eq!(
                (img.rgba[i], img.rgba[i + 3]),
                (0x00, 0xFF),
                "the core must be opaque black at {degrees}°"
            );
        }
    }

    /// ★★★ **The cache tells two angles apart.**
    ///
    /// This is the test for the failure the cache's own header names: keyed by
    /// size alone, the first angle asked for would be stored and every later
    /// angle would silently receive it — so the cursor would appear to reorient
    /// **once** and then never again. That reads as the feature half-working
    /// rather than as a cache bug, which is exactly the kind of defect that
    /// survives a manual look.
    ///
    /// The positive half is the same claim `the_same_scale_returns_the_same_
    /// allocation` makes for the crosshair: `egui-winit` dedupes its upload to
    /// the OS by `Arc::as_ptr`, so a fresh `Arc` per frame would convert a
    /// bitmap to a platform cursor handle sixty times a second.
    #[test]
    fn the_ibeam_cache_is_keyed_by_angle_as_well_as_size() {
        let upright = ibeam(1.0, Tilt::UPRIGHT);
        let again = ibeam(1.0, Tilt::UPRIGHT);
        assert!(
            Arc::ptr_eq(&upright.rgba, &again.rgba),
            "one angle at one scale must share its cached buffer"
        );
        let turned = ibeam(1.0, Tilt::nearest(90.0));
        assert!(
            !Arc::ptr_eq(&upright.rgba, &turned.rgba),
            "a different angle must be a different bitmap, or the cursor turns once and never again"
        );
        assert_ne!(
            &*upright.rgba, &*turned.rgba,
            "and the pixels must actually differ, not merely the allocation"
        );
    }

    /// A turned I-beam is still a valid cursor bitmap.
    ///
    /// The length invariant again, at an angle: `CustomCursor::from_rgba`
    /// rejects a buffer whose length is not `w * h * 4` and egui-winit's
    /// response to a rejection is to fall back to the platform cursor — i.e.
    /// silently back to the white-on-white defect this module exists to fix. A
    /// rotation that resized the buffer would look exactly like the tilt not
    /// being wired up.
    #[test]
    fn a_turned_bitmap_still_matches_the_size_it_declares() {
        for degrees in [0.0_f32, 45.0, 90.0, 175.0] {
            for ppp in [1.0_f32, 2.0] {
                let image = ibeam(ppp, Tilt::nearest(degrees));
                let (w, h) = (usize::from(image.size[0]), usize::from(image.size[1]));
                assert_eq!(
                    image.rgba.len(),
                    w * h * 4,
                    "the buffer must be exactly w * h * 4 at {degrees}° and {ppp}x"
                );
            }
        }
    }

    /// A nonsense scale lands on the clamp rather than on an allocation.
    ///
    /// Not defensive programming for its own sake: `pixels_per_point` is
    /// derived from a preference the operator can edit, and this crate has
    /// already shipped one preference that reached a layout pass unvalidated.
    #[test]
    fn a_nonsense_scale_is_clamped_rather_than_allocated() {
        for ppp in [0.0_f32, -3.0, f32::NAN, f32::INFINITY, 1.0e9] {
            let image = crosshair(ppp);
            assert!(
                u32::from(image.size[0]) <= MAX_CURSOR_PX && image.size[0] > 0,
                "a scale of {ppp} produced a {}px cursor",
                image.size[0]
            );
        }
    }
}

#[cfg(test)]
mod preview {
    use super::Tilt;

    /// Dump the raw bitmap so a human can look at it. `--ignored`.
    #[test]
    #[ignore]
    fn dump() {
        for ppp in [1.0_f32, 2.0] {
            let image = super::crosshair(ppp);
            let name = std::env::temp_dir().join(format!("crosshair-{}.rgba", image.size[0]));
            std::fs::write(&name, &*image.rgba).expect("write");
            println!("{} {}x{}", name.display(), image.size[0], image.size[1]);
        }
    }

    /// ★ **Print the I-beam as ASCII at several angles**, so a human can check
    /// the shape with their eyes. `--ignored`.
    ///
    /// This exists because of the constraint `apply`'s docs set out: **a cursor
    /// cannot be verified by screenshot.** Windows composites the pointer
    /// separately from window contents, so `BitBlt` and `PrintWindow` — the two
    /// ways `ui-verify` captures a window — return an image with no cursor in
    /// it at any price. R1's usual answer, *drive it and look at the picture*,
    /// has no picture to look at here.
    ///
    /// The unit tests assert the properties that can be stated as numbers — the
    /// core is dark, a 90° beam is wider than tall, the cache tells angles
    /// apart. What they cannot assert is whether the glyph *looks like an
    /// I-beam* at 30°, and this is how that is checked: by eye, deliberately,
    /// on demand.
    ///
    /// `cargo test -p pdfcer-gui --lib canvas::cursor::preview::ibeam_ascii -- \
    ///  --ignored --nocapture`
    #[test]
    #[ignore]
    fn ibeam_ascii() {
        for degrees in [0.0_f32, 30.0, 45.0, 90.0, 135.0] {
            let img = super::ibeam(1.0, Tilt::nearest(degrees));
            let n = usize::from(img.size[0]);
            println!(
                "\n--- {degrees}° ({}°) ---",
                Tilt::nearest(degrees).degrees()
            );
            for y in 0..n {
                let row: String = (0..n)
                    .map(|x| {
                        let i = (y * n + x) * 4;
                        match (img.rgba[i + 3], img.rgba[i]) {
                            (0, _) => ' ',
                            (_, 0) => '#',
                            _ => '.',
                        }
                    })
                    .collect();
                println!("{row}");
            }
        }
    }
}
