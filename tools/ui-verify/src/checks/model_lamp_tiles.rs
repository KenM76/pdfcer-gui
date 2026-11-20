//! `checks::model_lamp_tiles` — **listing lamps per tile draws the picture
//! every lamp draws**
//!
//! Places `fixtures/floor-lamp-grid.prc` (64 dim lamps just above a floor,
//! each reaching about half a unit) and opens it with *View…*, once as
//! shipped and once with `PDFCER_3D_LAMP_TILES=0`, which leaves the shader
//! visiting every lamp.
//!
//! Oracles, from the `model-view-rendered` line: the shipped picture is
//! drawn on the card with `lamp-tiles` above 0 and no tile listing every
//! lamp (`lamp-tile-most` below 64), so tiles are in use and do cut the
//! work; the forced picture has `lamp-tiles=0`; both light something
//! (`luma` above 0); and the two `hash`es are equal, the engine's
//! byte-for-byte contract. A tile table bound to the wrong unit, read from
//! the wrong scale or carrying the wrong count drops lamps from some tiles
//! and changes the picture.
//!
//! What this cannot see: a lamp's reach ends where its light falls below
//! half an 8-bit level, so its visible spot is far inside its listed reach,
//! and an origin off by a tile or two drops nothing visible. A wrong tile
//! size, an unbound table or a wrong count does.

use crate::checks::driving::repo_fixture;
use crate::checks::model_shadow_sun::{open, rendered};
use crate::checks::model_view_window::launch_with_model_env;
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;

const MODEL: &str = "floor-lamp-grid.prc";
const METHOD: &str = "Rebuild it with `python fixtures/floor-box.PROVENANCE.py <pdfcer.exe> grid`.";
const LAMPS: usize = 64;
const FIELDS: [&str; 6] = [
    "gpu",
    "gpu-declined",
    "lamp-tiles",
    "lamp-tile-most",
    "luma",
    "hash",
];

/// See the module documentation.
pub struct LampTilesDrawEveryLamp;

impl Check for LampTilesDrawEveryLamp {
    fn name(&self) -> &'static str {
        "lamp_tiles_draw_every_lamp"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer's per-tile lamp lists change the picture, or are never used on a model \
         with many short-reach lamps"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let tiled = match picture(ctx, report, "lamp-tiles", &[])? {
        Err(failure) => return Ok(Some(failure)),
        Ok(fields) => fields,
    };
    report.note(format!("as shipped {FIELDS:?}: {tiled:?}"));
    let count = |s: &str| s.parse::<usize>().unwrap_or(0);
    if tiled[..2] != ["1", "-"] {
        return Ok(Some(format!(
            "the viewer drew gpu={} declined={}; expected the card.",
            tiled[0], tiled[1]
        )));
    }
    if count(&tiled[2]) == 0 || count(&tiled[3]) >= LAMPS {
        return Ok(Some(format!(
            "★★ {LAMPS} short-reach lamps drew with lamp-tiles={} lamp-tile-most={}: the \
             lamps were not listed per tile.",
            tiled[2], tiled[3]
        )));
    }
    let forced = [("PDFCER_3D_LAMP_TILES", "0".to_owned())];
    let every = match picture(ctx, report, "lamp-tiles-off", &forced)? {
        Err(failure) => return Ok(Some(failure)),
        Ok(fields) => fields,
    };
    report.note(format!("tiles off {FIELDS:?}: {every:?}"));
    if every[..3] != ["1", "-", "0"] {
        return Ok(Some(format!(
            "`PDFCER_3D_LAMP_TILES=0` drew gpu={} declined={} lamp-tiles={}.",
            every[0], every[1], every[2]
        )));
    }
    let lit = |s: &str| s.parse::<f64>().unwrap_or(0.0) > 0.0;
    if !lit(&every[4]) || !lit(&tiled[4]) {
        return Ok(Some(format!(
            "a picture is black (luma {} tiled, {} every lamp): the lamps light nothing to \
             compare.",
            tiled[4], every[4]
        )));
    }
    Ok((tiled[5] != every[5]).then(|| {
        format!(
            "★★★ the tiled picture (hash {}, luma {}) is not the picture every lamp draws \
             (hash {}, luma {}).",
            tiled[5], tiled[4], every[5], every[4]
        )
    }))
}

/// One launch's picture fields, or why there is none.
fn picture(
    ctx: &CheckContext,
    report: &mut CheckReport,
    stem: &str,
    env: &[(&str, String)],
) -> Result<std::result::Result<Vec<String>, String>> {
    let model = repo_fixture(MODEL, METHOD)?;
    let (session, pointer) =
        launch_with_model_env(ctx, report, stem, &model.to_string_lossy(), env)?;
    if let Some(failure) = open(ctx, &session, &pointer)? {
        return Ok(Err(failure));
    }
    Ok(match rendered(&session, FIELDS)? {
        Some(fields) => Ok(fields.to_vec()),
        None => Err("the viewer drew no picture line.".to_owned()),
    })
}
