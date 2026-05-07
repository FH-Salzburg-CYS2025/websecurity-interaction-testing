//! Encoding and case-transformation utilities.

use core::fmt::Write as _;
use rand::{RngExt as _, rng};

/// Applies the requested encoding transformation to `text`.
///
/// | `encoding`         | Transformation                                    |
/// |--------------------|---------------------------------------------------|
/// | `"url_encoded"`    | Percent-encodes every character (`%XX`)           |
/// | `"html_entity"`    | Escapes `<`, `>`, and `"` as HTML entities        |
/// | `"unicode_escape"` | Replaces every character with `\uXXXX` notation  |
/// | anything else      | Returns `text` unchanged                          |
pub fn apply_encoding(text: &str, encoding: &str) -> String {
    match encoding {
        "url_encoded" => {
            let mut output = String::new();
            for chr in text.chars() {
                write!(output, "%{:02X}", u32::from(chr)).unwrap_or_default();
            }
            output
        },
        "html_entity" => text.replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;"),
        "unicode_escape" => {
            let mut output = String::new();
            for chr in text.chars() {
                write!(output, "\\u{:04x}", u32::from(chr)).unwrap_or_default();
            }
            output
        },
        _ => String::from(text),
    }
}

/// Transforms `tag` according to the requested case style.
///
/// | `case`    | Behaviour                                                    |
/// |-----------|--------------------------------------------------------------|
/// | `"upper"` | Converts the entire tag name to uppercase.                   |
/// | `"mixed"` | Each character's case is decided independently at random.    |
/// | anything  | Returns the tag in lowercase (default).                      |
pub fn apply_case(tag: &str, case: &str) -> String {
    match case {
        "upper" => tag.to_uppercase(),
        "mixed" => apply_random_mixed_case(tag),
        _ => tag.to_lowercase(),
    }
}

/// Applies a randomly-mixed case transformation to `tag`.
///
/// Each character's case is decided independently at random.
/// Only called with ASCII HTML tag names, so case folding never changes byte
/// length.
fn apply_random_mixed_case(tag: &str) -> String {
    let mut rng = rng();
    tag.chars()
        .map(|chr| {
            if rng.random_bool(0.5_f64) {
                chr.to_uppercase().next().unwrap_or(chr)
            } else {
                chr.to_lowercase().next().unwrap_or(chr)
            }
        })
        .collect()
}
