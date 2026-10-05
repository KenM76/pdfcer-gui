//! File ▸ Security ▸ Add archive time-stamp…: a document time-stamp over the
//! whole file, through `EditSession::add_document_timestamp`.
//!
//! The engine returns the stamped file's bytes and the open document stays as
//! it was, so this writes a second file the operator names, as Sign does, and
//! never the open one. The server is asked over the network, bounded by
//! `sign::timestamp::TIMEOUT`.

use pdfcer_core::sign::timestamp::DocTimestampRequest;

use crate::app::files::{self, Picked};
use crate::app::state::OpenDoc;
use crate::text::archive as t;

/// Pick the target, ask `server` for a stamp, and write the copy.
pub(super) fn stamp(doc: &mut OpenDoc, server: &str) {
    let suggested = crate::sign::suggested_path(&doc.path, t::SUFFIX);
    let Picked::Path(target) = files::pick_save_path(&suggested, t::window_title()) else {
        crate::diag::trace(|| "archive-cancelled".to_owned()); // ui-text-exempt: diagnostic trace, never displayed
        return;
    };
    let epoch = doc.edit_epoch;
    let note = |reason: &str, sentence: String| {
        crate::diag::trace(|| format!("archive-refused reason={reason}")); // ui-text-exempt: diagnostic trace, never displayed
        crate::app::actions::record_note(epoch, sentence);
    };
    if crate::app::files::same_file(&target, &doc.path) {
        return note("source", t::not_the_source().to_owned());
    }
    let Some(authority) = crate::sign::timestamp::authority_for(server) else {
        return note("unavailable", t::unavailable().to_owned());
    };
    doc.render_worker.cancel_and_wait();
    let options = crate::app::settings::SettingsExt::save_options(&doc.settings);
    let Some(session) = std::sync::Arc::get_mut(&mut doc.session) else {
        return note("session-held", t::busy().to_owned());
    };
    let (bytes, report) = match session.add_document_timestamp(
        &*authority,
        &DocTimestampRequest::default(),
        &options,
    ) {
        Ok(done) => done,
        Err(error) => return note("engine", t::refused(&error.to_string())),
    };
    let temporary = target.with_extension("pdfcer-tmp");
    let written =
        std::fs::write(&temporary, &bytes).and_then(|()| std::fs::rename(&temporary, &target));
    if let Err(error) = written {
        let _ = std::fs::remove_file(&temporary);
        return note("write", t::write_failed(&error.to_string()));
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "archive-written path={target:?} bytes={} prior={} dss={} level={} notes={}",
            bytes.len(),
            report.prior_signatures,
            u8::from(report.dss_present),
            report.pades_level.unwrap_or("none"), // ui-text-exempt: trace token
            report.notes.len()
        )
    });
    let written = t::written(
        &target.display().to_string(),
        &report.timestamp.gen_time,
        &report.timestamp.tsa_subject,
        &t::level(report.prior_signatures, report.dss_present),
    );
    let notes: Vec<String> = std::iter::once(written)
        .chain(crate::app::rc4::after_save(report.rc4_keystream_reused))
        .collect();
    crate::app::actions::record_notes(epoch, notes);
}
