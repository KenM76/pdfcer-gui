//! # `text::ocroutput` — the output choice in File ▸ Recognise text…

/// The group's heading.
#[must_use]
pub const fn heading() -> &'static str {
    "Output"
}

/// The default: invisible text over the untouched scan.
#[must_use]
pub const fn searchable() -> &'static str {
    "Searchable image"
}

/// Its tooltip.
#[must_use]
pub const fn searchable_tooltip() -> &'static str {
    "The page looks exactly as it did. The words are added as invisible text over \
     the scan, so the page can be searched, selected and copied."
}

/// Words taken out of the scan and written as text.
#[must_use]
pub const fn editable() -> &'static str {
    "Editable text and images"
}

/// Its tooltip. States the cost: the scan itself is changed.
#[must_use]
pub const fn editable_tooltip() -> &'static str {
    "Each word read with enough confidence is taken out of the scanned picture and \
     written back as text you can edit, like Acrobat's Editable Text and Images. \
     The scan itself is changed, words read with low confidence stay in the picture, \
     and the page can then not be read again or have its OCR text removed. \
     Ctrl+Z puts the scan back."
}

/// Under the editable choice: store the rewritten scan without loss.
#[must_use]
pub const fn lossless() -> &'static str {
    "Keep the scan lossless"
}

/// Its tooltip. States the cost.
#[must_use]
pub const fn lossless_tooltip() -> &'static str {
    "With the words taken out, the scanned picture is stored again. By default a \
     JPEG scan is stored as JPEG, which loses a little detail each time. Ticked, \
     it is stored without any loss, at the cost of a larger file."
}
