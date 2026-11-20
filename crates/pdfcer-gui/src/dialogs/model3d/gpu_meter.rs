//! The light meter's bounce built on the graphics card. The engine packs it
//! (`LightMeter::gpu_new`), the paint traces it in the meter's slot of
//! [`super::gpu_bounce_card`] exactly as it traces the frame's, and the
//! reading's thread waits for the atlas and finishes the meter with it.
//!
//! The processor builds the meter instead when the viewer is not drawing on
//! the card (no paint would advance the job), when the card cannot run the
//! build, when it has not finished within [`WAIT`], or under
//! `PDFCER_3D_BOUNCE_CPU=1`. A reading superseded while it waits gets the
//! error [`SUPERSEDED`], which nobody receives.

use std::sync::Arc;
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::time::Duration;

use pdfcer_3d::RenderOptions as ModelRenderOptions;
use pdfcer_3d::{AssembledModel, Camera, LightMeter, MeterSettings, PreparedModel};

use super::gpu_bounce_card::{CardJob, CardQueue, Slot};

/// The longest a reading waits on the card before building on the processor.
const WAIT: Duration = Duration::from_secs(60);
/// The error of a reading whose card build was dropped for a newer one.
const SUPERSEDED: &str = "superseded"; // ui-text-exempt: never displayed; its receiver is gone

/// The meter for one reading, its bounce built on `card` when given and able.
///
/// # Errors
///
/// The engine's refusal, as text; or [`SUPERSEDED`].
pub(super) fn light_meter<'m>(
    model: &'m AssembledModel,
    prepared: &PreparedModel,
    camera: &Camera,
    options: &ModelRenderOptions,
    settings: &MeterSettings,
    card: Option<&CardQueue>,
) -> Result<LightMeter<'m>, String> {
    let processor =
        || LightMeter::new(model, prepared, camera, options, settings).map_err(|e| e.to_string());
    use pdfcer_gui_base::cardstatus::{Outcome, Path, note};
    let on_processor = |why: String| note(Path::Meter, Outcome::Processor(why));
    let Some(card) = card else {
        on_processor(crate::text::cardstatus::meter_not_drawing().to_owned());
        return processor();
    };
    if !card.wanted() {
        on_processor(crate::text::cardstatus::meter_refused().to_owned());
        return processor();
    }
    let meter = LightMeter::gpu_new(model, prepared, camera, options, settings)
        .map_err(|e| e.to_string())?;
    // `finish` reads the atlas only while `build` is set, so the card gets a
    // copy (reported to the engine: note_gpu_meter_build_ownership).
    let Some(build) = meter.build.clone() else {
        return meter.finish(&[]).map_err(|e| e.to_string());
    };
    let (tx, rx) = channel();
    let alive = Arc::new(());
    let job = CardJob::new(build, tx, Arc::downgrade(&alive), Box::new(|| Ok(None)));
    card.hand(Slot::Meter, job);
    match rx.recv_timeout(WAIT) {
        Ok(Ok(Some(bounce))) => {
            note(Path::Meter, Outcome::InUse);
            meter.finish(&bounce.atlas).map_err(|e| e.to_string())
        }
        // The card could not run it: its fallback sends no bounce.
        Ok(Ok(None) | Err(_)) => {
            on_processor(crate::text::cardstatus::meter_refused().to_owned());
            processor()
        }
        // Returning drops `alive`, so the next paint drops the job.
        Err(RecvTimeoutError::Timeout) => {
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| {
                "model-light-meter-bounce built=processor reason=\"timeout\"".into()
            });
            on_processor(crate::text::cardstatus::meter_timeout(WAIT.as_secs()));
            processor()
        }
        Err(RecvTimeoutError::Disconnected) => Err(SUPERSEDED.to_owned()),
    }
}
