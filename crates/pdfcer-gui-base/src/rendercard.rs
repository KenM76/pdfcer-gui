//! # `rendercard` — a canvas page drawn on the graphics card
//!
//! With the switch on, the worker records the page as a display list and
//! asks the engine to pack it for the card ([`pdfcer_render::GpuScene`]). The
//! UI thread then draws that scene with the installed [`CardDraw`], and the
//! bytes it reads back are the processor's raster, byte for byte (engine
//! decision 223). A whole page is the card's or the processor's, never a mix:
//! a page the engine refuses, or one it cannot record, is drawn by the
//! processor and carries the reason in [`CardNote`] for the status bar.
//!
//! The draw is a [`CardJob`]. [`begin`] uploads the scene and each [`step`]
//! does one slice and returns at once, so the window keeps its frame rate and
//! shows the picture it had until the card's arrives. The render worker steps
//! the job once a frame while it is in flight. Dropping the job (a newer
//! request, a closed document) abandons it on the card.
//!
//! The drawer lives on the UI thread, in a thread-local, because the GL
//! context is current there and nowhere else. A drawer that fails once is
//! retired for the session: every later request goes to the processor, and
//! says why.
//!
//! Traces: `render-card drew=1 w= h= texels= layers= pack_ms= draw_ms= ui_ms=
//! steps=` for a page drawn on the card, where `draw_ms` is from the upload to
//! the picture arriving and `ui_ms` is the part of it spent on the UI thread;
//! `render-card drew=0 reason=` for one that was not.

use std::cell::RefCell;
use std::time::{Duration, Instant};

use pdfcer_render::{GpuRefusal, GpuScene};

use crate::cardstatus::{Outcome, Path, note};
use crate::renderworker::{RenderKey, RenderRequest, RenderedPixels};

/// Something that draws a packed scene into `RGBA8` bytes, row 0 first, a
/// slice at a time. Jobs are named by the number [`CardDraw::begin`] returns.
pub trait CardDraw {
    /// Upload `scene` and start a job; nothing is drawn yet.
    ///
    /// # Errors
    ///
    /// The driver's account of why it could not.
    fn begin(&mut self, scene: &GpuScene) -> Result<u64, String>;

    /// Advance `job` by one slice without waiting: `None` while it is
    /// unfinished, else its bytes, after which the job is gone.
    ///
    /// # Errors
    ///
    /// The driver's account of why it could not; the job is gone.
    fn step(&mut self, job: u64, scene: &GpuScene) -> Result<Option<Vec<u8>>, String>;

    /// Release `job`, finished or not; an unknown job is ignored.
    fn abandon(&mut self, job: u64);
}

/// What a render request asks of the card.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CardAsk {
    /// The switch is off; the processor draws, and nothing is said.
    #[default]
    Off,
    /// Pack the page for the card.
    Try,
    /// The switch is on but the card cannot be used; the reason.
    Unavailable(String),
}

/// Which way a raster was drawn, for the status bar.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CardNote {
    /// The switch is off.
    #[default]
    Off,
    /// Drawn on the graphics card.
    Drew,
    /// The switch is on and the processor drew it; why.
    Processor(String),
}

/// A page packed on the worker, waiting for the UI thread to draw it.
#[derive(Debug)]
pub struct ScenePixels {
    /// The packed page.
    pub scene: GpuScene,
    /// The recording's diagnostics: the same report the processor would give.
    pub diagnostics: pdfcer_render::Diagnostics,
    /// The request's key.
    pub key: RenderKey,
    /// Recording and packing, on the worker.
    pub pack: Duration,
}

thread_local! {
    static CARD: RefCell<Card> = const { RefCell::new(Card::None) };
    static RETIRED_NOW: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

enum Card {
    None,
    Ready(Box<dyn CardDraw>),
    Retired(String),
}

/// Install the UI thread's drawer, once; a later call is ignored.
pub fn install(drawer: Box<dyn CardDraw>) {
    CARD.with_borrow_mut(|card| {
        if matches!(card, Card::None) {
            *card = Card::Ready(drawer);
        }
    });
}

/// Record that no drawer can be made, and why; a later call is ignored.
pub fn refuse(why: String) {
    CARD.with_borrow_mut(|card| {
        if matches!(card, Card::None) {
            *card = Card::Retired(why);
        }
    });
}

/// Whether a drawer failed since the last call. The canvas asks again on the
/// next frame, so the caller requests one.
#[must_use]
pub fn take_retirement() -> bool {
    RETIRED_NOW.replace(false)
}

/// Whether a drawer has been offered, retired or not.
#[must_use]
pub fn installed() -> bool {
    CARD.with_borrow(|card| !matches!(card, Card::None))
}

/// The ask for a request, `wanted` being the operator's switch.
#[must_use]
pub fn ask(wanted: bool) -> CardAsk {
    if !wanted {
        note(Path::Pages, Outcome::Off);
        return CardAsk::Off;
    }
    let ask = CARD.with_borrow(|card| match card {
        Card::Ready(_) => CardAsk::Try,
        Card::Retired(why) => CardAsk::Unavailable(why.clone()),
        Card::None => CardAsk::Unavailable(crate::text::status::card_no_context().to_owned()),
    });
    if let CardAsk::Unavailable(why) = &ask {
        note(Path::Pages, Outcome::Processor(why.clone()));
    }
    ask
}

/// On the worker: the page packed for the card, or the reason the processor
/// must draw it. `None` when the request did not ask.
pub(crate) fn pack(
    view: &pdfcer_render::DocumentView<'_>,
    request: &RenderRequest,
    options: &pdfcer_render::RenderOptions,
) -> Option<Result<ScenePixels, String>> {
    match &request.card {
        CardAsk::Off => None,
        CardAsk::Unavailable(why) => Some(Err(why.clone())),
        CardAsk::Try => Some(pack_page(view, request, options)),
    }
}

fn pack_page(
    view: &pdfcer_render::DocumentView<'_>,
    request: &RenderRequest,
    options: &pdfcer_render::RenderOptions,
) -> Result<ScenePixels, String> {
    let started = Instant::now();
    let list = pdfcer_render::display_list::record_page(
        view,
        &request.page,
        request.raster_scale,
        0,
        options,
    )
    .map_err(|e| match e {
        pdfcer_render::RenderError::PageNotRecordable { .. } => {
            crate::text::status::card_not_recordable().to_owned()
        }
        other => other.to_string(),
    })?;
    let recorded = started.elapsed().as_millis();
    let region = request.region.unwrap_or(request.page.crop_box);
    let scene = list
        .gpu_scene(list.key(), region)
        .map_err(|e| refusal(&e))?;
    let total = started.elapsed().as_millis();
    let cancelled = options.cancel.as_ref().is_some_and(|c| c.is_cancelled());
    // ui-text-exempt: diagnostic trace, never displayed in the UI
    crate::diag::trace(|| {
        format!("render-card-pack record_ms={recorded} total_ms={total} cancelled={cancelled}")
    });
    Ok(ScenePixels {
        scene,
        diagnostics: list.diagnostics().clone(),
        key: RenderKey::of(request),
        pack: started.elapsed(),
    })
}

/// The status bar's words for an engine refusal.
fn refusal(e: &GpuRefusal) -> String {
    use crate::text::status as t;
    match e {
        GpuRefusal::Image => t::card_refused_image().to_owned(),
        GpuRefusal::Shading => t::card_refused_shading().to_owned(),
        GpuRefusal::Transparency => t::card_refused_transparency().to_owned(),
        GpuRefusal::Curves => t::card_refused_curves().to_owned(),
        GpuRefusal::TooMuchData { .. } => t::card_refused_size().to_owned(),
        other => other.to_string(),
    }
}

/// Trace a page the processor drew with the switch on.
pub(crate) fn trace_processor(why: &str) {
    note(Path::Pages, Outcome::Processor(why.to_owned()));
    crate::diag::trace(|| format!("render-card drew=0 reason={why:?}")); // ui-text-exempt: diagnostic trace, never displayed
}

/// A page being drawn on the card; [`step`] it once a frame.
pub(crate) struct CardJob {
    id: u64,
    size: [u32; 2],
    texels: usize,
    layers: u32,
    diagnostics: Option<pdfcer_render::Diagnostics>,
    key: RenderKey,
    pack: Duration,
    started: Instant,
    ui: Duration,
    steps: u32,
    /// The packed page, uploaded a part a step.
    scene: GpuScene,
}

impl Drop for CardJob {
    fn drop(&mut self) {
        let id = self.id;
        let started = Instant::now();
        let _ = CARD.try_with(|card| {
            if let Ok(mut card) = card.try_borrow_mut()
                && let Card::Ready(drawer) = &mut *card
            {
                drawer.abandon(id);
            }
        });
        let ms = started.elapsed().as_millis();
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        crate::diag::trace(|| format!("render-card-hold at=drop ms={ms}"));
    }
}

/// Where a [`step`] left a job.
pub(crate) enum Drawn {
    /// Still on the card.
    Pending,
    /// The page.
    Done(Box<RenderedPixels>),
    /// The drawer failed and is retired; the canvas asks again and the
    /// processor draws.
    Failed,
}

/// On the UI thread: upload `packed` to the installed drawer. `None` when the
/// drawer is missing or fails, which retires it.
pub(crate) fn begin(packed: ScenePixels) -> Option<CardJob> {
    let started = Instant::now();
    let begun = CARD.with_borrow_mut(|card| match card {
        Card::Ready(drawer) => Some(drawer.begin(&packed.scene)),
        Card::None | Card::Retired(_) => None,
    })?;
    let held = started.elapsed().as_millis();
    let texels = packed.scene.data.texels.len() / 4;
    // ui-text-exempt: diagnostic trace, never displayed in the UI
    crate::diag::trace(|| format!("render-card-hold at=begin ms={held} texels={texels}"));
    match begun {
        Ok(id) => Some(CardJob {
            id,
            size: [packed.scene.width, packed.scene.height],
            texels: packed.scene.data.texels.len() / 4,
            layers: packed.scene.data.layers,
            diagnostics: Some(packed.diagnostics),
            key: packed.key,
            pack: packed.pack,
            started,
            ui: started.elapsed(),
            steps: 0,
            scene: packed.scene,
        }),
        Err(why) => {
            retire(&why);
            None
        }
    }
}

/// On the UI thread: one slice of `job`, without waiting on the card.
pub(crate) fn step(job: &mut CardJob) -> Drawn {
    let started = Instant::now();
    let stepped = CARD.with_borrow_mut(|card| match card {
        Card::Ready(drawer) => Some(drawer.step(job.id, &job.scene)),
        Card::None | Card::Retired(_) => None,
    });
    let held = started.elapsed();
    job.ui += held;
    job.steps += 1;
    if held.as_millis() >= 20 {
        let (ms, n) = (held.as_millis(), job.steps);
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        crate::diag::trace(|| format!("render-card-hold at=step ms={ms} step={n}"));
    }
    let [w, h] = job.size;
    let pixmap = match stepped {
        None => return Drawn::Failed,
        Some(Ok(None)) => return Drawn::Pending,
        Some(Ok(Some(bytes))) => pdfcer_render::tiny_skia::IntSize::from_wh(w, h)
            .ok_or_else(|| "empty".to_owned()) // ui-text-exempt: driver-words slot
            .and_then(|size| {
                pdfcer_render::tiny_skia::Pixmap::from_vec(bytes, size)
                    .ok_or_else(|| format!("{w}x{h} read back the wrong size")) // ui-text-exempt: driver-words slot
            }),
        Some(Err(why)) => Err(why),
    };
    match pixmap {
        Ok(pixmap) => Drawn::Done(Box::new(finish(job, pixmap))),
        Err(why) => {
            retire(&why);
            Drawn::Failed
        }
    }
}

fn finish(job: &mut CardJob, pixmap: pdfcer_render::tiny_skia::Pixmap) -> RenderedPixels {
    let drawn = job.started.elapsed();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "render-card drew=1 w={} h={} texels={} layers={} pack_ms={} draw_ms={} ui_ms={} steps={}",
            job.size[0],
            job.size[1],
            job.texels,
            job.layers,
            job.pack.as_millis(),
            drawn.as_millis(),
            job.ui.as_millis(),
            job.steps
        )
    });
    note(Path::Pages, Outcome::InUse);
    RenderedPixels {
        pixmap,
        diagnostics: job.diagnostics.take().unwrap_or_default(),
        key: job.key,
        elapsed: job.pack + drawn,
        card: CardNote::Drew,
    }
}

/// Retire the drawer for the session with the driver's words.
fn retire(why: &str) {
    let why = crate::text::status::card_failed(why);
    trace_processor(&why);
    CARD.with_borrow_mut(|card| *card = Card::Retired(why));
    RETIRED_NOW.set(true);
}
