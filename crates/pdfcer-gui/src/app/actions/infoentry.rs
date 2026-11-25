//! # `app::actions::infoentry` — setting or removing any `/Info` entry by key
//!
//! The engine call behind `Action::SetInfoEntry`: one undo step through the
//! funnel, the XMP copy kept in step by the engine's session default, and three
//! things the operator is owed off-canvas — a skipped XMP copy, a replaced
//! value on an Add, and a worded decline for a key the engine will not write.

use pdfcer_core::edit::{EditError, EditSession, XmpSync};

use crate::app::state::OpenDoc;
use crate::text::panels::docprops as t;

/// Set (`Some`) or remove (`None`) the entry `key`.
pub(super) fn apply(doc: &mut OpenDoc, key: &[u8], value: Option<&str>, adding: bool) {
    let name = String::from_utf8_lossy(key).into_owned();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed. The key is the
        // operator's field name; the value goes as a length only.
        format!(
            "info-entry-commit key={name} clearing={} chars={} adding={}",
            u8::from(value.is_none()),
            value.map_or(0, |v| v.chars().count()),
            u8::from(adding),
        )
    });
    super::apply::vector_edit(doc, "set-info-entry", 0, 1, |session| {
        set(session, key, value, adding, &name)
    });
}

/// The engine call and its disclosures; a name refusal is worded first.
fn set(
    session: &mut EditSession,
    key: &[u8],
    value: Option<&str>,
    adding: bool,
    name: &str,
) -> Result<Vec<String>, EditError> {
    let replacing = adding && session.info_entries().iter().any(|e| e.key == key);
    let outcome = session.set_info_entry(key, value).inspect_err(|error| {
        if let Some(why) = t::InfoKeyRefusal::of(error) {
            crate::app::status::decline::record_info_key(why);
        }
    })?;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "info-entry-set key={name} info_changed={} xmp={}",
            u8::from(outcome.info_changed),
            xmp_token(&outcome.xmp),
        )
    });
    let mut notes = Vec::new();
    if replacing && outcome.info_changed {
        notes.push(t::custom_replaced(name));
    }
    if let XmpSync::Skipped { reason, .. } = outcome.xmp {
        notes.push(t::xmp_skipped(name, reason));
    }
    Ok(notes)
}

/// The trace token for what happened to the XMP copy.
fn xmp_token(xmp: &XmpSync) -> String {
    // ui-text-exempt: diagnostic trace tokens, never displayed in the UI
    match xmp {
        XmpSync::NoPacket => "no-packet".to_owned(),
        XmpSync::Updated { property } => format!("updated:{property}"),
        XmpSync::InStep { property } => format!("in-step:{property}"),
        XmpSync::Skipped { reason, .. } => format!("skipped:{}", reason.token()),
        _ => "other".to_owned(),
    }
}
