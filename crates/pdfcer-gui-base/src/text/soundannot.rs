//! # `text::soundannot` — the words for attaching a sound to a page

use pdfcer_core::annot_author::SoundIcon;
use pdfcer_core::sound::{SoundConversion, SoundData};

/// The label over the icon chooser.
#[must_use]
pub const fn icon_heading() -> &'static str {
    "Icon"
}

/// One sound icon's name, as an operator reads it.
#[must_use]
pub const fn icon_label(icon: &SoundIcon) -> &'static str {
    match icon {
        SoundIcon::Speaker => "Speaker",
        SoundIcon::Mic => "Microphone",
        // Never offered; a file can carry one, a chooser never authors one.
        SoundIcon::Other(_) => "Another icon",
    }
}

/// The line naming the chosen recording.
#[must_use]
pub fn file_line(name: &str, bytes: u64) -> String {
    format!(
        "Recording: {name} ({})",
        super::panels::byte_size(usize::try_from(bytes).unwrap_or(usize::MAX))
    )
}

/// The resample choice's label.
#[must_use]
pub const fn resample_label() -> &'static str {
    "Convert to a standard rate (11,025 or 22,050 Hz)"
}

/// The resample choice's hover: why it is a choice at all.
#[must_use]
pub const fn resample_hover() -> &'static str {
    "The PDF standard asks for these rates and also tells readers to convert \
     any other rate as they play. Left off, the recording keeps its own rate \
     and quality."
}

/// The downmix choice's label.
#[must_use]
pub const fn downmix_label() -> &'static str {
    "Mix more than two channels down to mono"
}

/// The downmix choice's hover.
#[must_use]
pub const fn downmix_hover() -> &'static str {
    "A PDF sound holds at most two channels. Left off, a recording with more \
     is refused rather than changed."
}

/// One conversion the import made, as a clause.
#[must_use]
pub fn conversion(c: &SoundConversion) -> String {
    match c {
        SoundConversion::FloatToSigned16 { from_bits } => {
            format!("{from_bits}-bit floating-point samples became 16-bit")
        }
        SoundConversion::Resampled { from, to } => format!("resampled from {from} Hz to {to} Hz"),
        SoundConversion::Downmixed { from_channels } => {
            format!("{from_channels} channels mixed down to mono")
        }
        _ => "converted".to_owned(),
    }
}

/// The disclosure after a sound is attached to a page, naming every
/// conversion the import made.
#[must_use]
pub fn placed(
    name: &str,
    page: usize,
    sound: &SoundData,
    conversions: &[SoundConversion],
) -> String {
    let mut line = format!(
        "{name} is now stored in this PDF, on page {} ({} Hz, {} channel{}, {} bytes of sound).",
        page + 1,
        sound.rate,
        sound.channels,
        if sound.channels == 1 { "" } else { "s" },
        sound.samples.len()
    );
    if !conversions.is_empty() {
        let list: Vec<String> = conversions.iter().map(conversion).collect();
        line.push_str(&format!(" Changed on the way in: {}.", list.join("; ")));
    }
    line
}

/// The refusal when the recording cannot be imported.
#[must_use]
pub fn refused(detail: &str) -> String {
    format!("pdfcer could not use that recording, so no sound was attached: {detail}")
}

/// The refusal when the picked file cannot be read.
#[must_use]
pub fn unreadable(detail: &str) -> String {
    format!("pdfcer could not read that file, so no sound was attached: {detail}")
}

/// The picker's title.
#[must_use]
pub const fn pick_title() -> &'static str {
    "Choose a recording to attach"
}

/// The picker's filter name.
#[must_use]
pub const fn filter_wav() -> &'static str {
    "WAV recordings"
}
