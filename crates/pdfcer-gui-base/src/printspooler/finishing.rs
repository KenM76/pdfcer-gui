//! # `printspooler::finishing` — staple, punch and output bin through the driver
//!
//! What a printer offers is read from its driver's PrintCapabilities on a
//! worker, once per printer per dialog ([`OfferCache`]). At commit the
//! request is merged into the job's driver settings by the driver itself
//! ([`with_finishing`]); a driver that refuses or substitutes any part is
//! an `Err` naming it, and the job is not sent. Fold and hole counts are
//! IPP-only in the engine and never offered here.

use std::sync::mpsc::{Receiver, Sender, channel};

pub use pdfcer_print::finishing::{Edge, Finishing, Staple};

use super::DriverConfig;

/// The finishing one printer's driver offers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Offer {
    pub staples: Vec<Staple>,
    pub punch: Vec<Edge>,
    /// Output bins, by the driver's own names.
    pub bins: Vec<String>,
}

impl Offer {
    /// Whether the driver offers nothing this module can ask for.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.staples.is_empty() && self.punch.is_empty() && self.bins.is_empty()
    }
}

/// What [`OfferCache::get`] knows about one printer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OfferState {
    Pending,
    Ready(Offer),
    /// The driver's PrintTicket provider would not answer; the engine's words.
    Failed(String),
}

/// `printer_finishing`, answered on a worker and remembered for the life of
/// one dialog.
pub struct OfferCache {
    known: Vec<(String, OfferState)>,
    tx: Sender<(String, OfferState)>,
    rx: Receiver<(String, OfferState)>,
}

impl std::fmt::Debug for OfferCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OfferCache")
            .field("known", &self.known.len())
            .finish_non_exhaustive()
    }
}

impl Default for OfferCache {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            known: Vec::new(),
            tx,
            rx,
        }
    }
}

impl OfferCache {
    /// The offer for `printer`, starting the read on first ask.
    pub fn get(&mut self, printer: &str) -> OfferState {
        while let Ok((name, state)) = self.rx.try_recv() {
            trace_offer(&name, &state);
            if let Some(slot) = self.known.iter_mut().find(|(k, _)| *k == name) {
                slot.1 = state;
            }
        }
        if let Some((_, state)) = self.known.iter().find(|(k, _)| k == printer) {
            return state.clone();
        }
        self.known.push((printer.to_owned(), OfferState::Pending));
        let tx = self.tx.clone();
        let name = printer.to_owned();
        let spawned = std::thread::Builder::new()
            .name(format!("printer-finishing {printer}")) // ui-text-exempt: thread name, never displayed
            .spawn(move || {
                let state = match pdfcer_print::finishing::printer_finishing(&name) {
                    Ok(caps) => OfferState::Ready(Offer {
                        staples: caps.staples,
                        punch: caps.punch,
                        bins: caps.bins.into_iter().map(|b| b.name).collect(),
                    }),
                    Err(e) => OfferState::Failed(e.to_string()),
                };
                drop(tx.send((name, state)));
            });
        if let Err(e) = spawned {
            let failed = OfferState::Failed(e.to_string());
            if let Some(last) = self.known.last_mut() {
                last.1 = failed.clone();
            }
            return failed;
        }
        OfferState::Pending
    }
}

fn trace_offer(printer: &str, state: &OfferState) {
    crate::diag::trace(|| match state {
        // ui-text-exempt: diagnostic trace, never displayed
        OfferState::Ready(o) => format!(
            "print-finishing printer={printer} staples={} punch={} bins={}",
            o.staples.len(),
            o.punch.len(),
            o.bins.len()
        ),
        // ui-text-exempt: diagnostic trace, never displayed
        OfferState::Failed(why) => format!("print-finishing-unread printer={printer} why={why}"),
        // ui-text-exempt: diagnostic trace, never displayed
        OfferState::Pending => format!("print-finishing-pending printer={printer}"),
    });
}

/// The finishing request the dialog's choices make.
#[must_use]
pub fn request(staple: Option<Staple>, punch: Option<Edge>, bin: Option<&str>) -> Finishing {
    let mut f = Finishing::default();
    f.staple = staple;
    f.punch = punch;
    f.output_bin = bin.map(str::to_owned);
    f
}

/// The driver settings to spool with once `f` is merged in: `config` itself
/// when nothing is asked for, otherwise the driver's merge of `f` onto
/// `config` (or onto the driver's own current settings when there is none).
///
/// # Errors
///
/// The engine's sentence when the driver refuses or substitutes any part of
/// `f`, or its PrintTicket provider fails. Nothing is sent.
pub fn with_finishing(
    printer: &str,
    config: Option<&DriverConfig>,
    f: &Finishing,
) -> Result<Option<DriverConfig>, String> {
    if f.is_empty() {
        return Ok(config.cloned());
    }
    let base = match config {
        Some(c) => c.engine().clone(),
        None => pdfcer_print::printer_configuration(printer).map_err(|e| e.to_string())?,
    };
    let merged =
        pdfcer_print::finishing::apply_finishing(printer, &base, f).map_err(|e| e.to_string())?;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "print-finishing-applied printer={printer} staple={:?} punch={:?} bin={:?}",
            f.staple, f.punch, f.output_bin
        )
    });
    Ok(Some(DriverConfig::from_engine(merged)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// No choice made is an empty request, which leaves the job's settings alone.
    #[test]
    fn nothing_chosen_asks_for_nothing() {
        assert!(request(None, None, None).is_empty());
        assert!(
            with_finishing("no such printer", None, &request(None, None, None))
                .unwrap()
                .is_none()
        );
    }

    /// Each choice reaches its own field of the request.
    #[test]
    fn choices_reach_the_request() {
        let f = request(Some(Staple::TopLeft), Some(Edge::Left), Some("Tray 2"));
        assert_eq!(f.staple, Some(Staple::TopLeft));
        assert_eq!(f.punch, Some(Edge::Left));
        assert_eq!(f.output_bin.as_deref(), Some("Tray 2"));
        assert!(f.fold.is_none() && f.holes.is_none());
    }

    /// An empty offer is one the dialog draws nothing for.
    #[test]
    fn an_empty_offer_is_empty() {
        assert!(Offer::default().is_empty());
        let staples = Offer {
            staples: vec![Staple::TopLeft],
            ..Offer::default()
        };
        assert!(!staples.is_empty());
    }
}
