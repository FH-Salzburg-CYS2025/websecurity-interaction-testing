//! IPM detection and payload translation functions.

use crate::encoding::{apply_case, apply_encoding};
use core::fmt::{Display, Formatter, Result as FmtResult, Write as _};
use std::collections::HashMap;

/// Identifies the injection point model (IPM) that a CSV file targets.
#[derive(Debug)]
pub enum IpmType {
    /// Breakout from an existing HTML attribute value into a new element.
    AttributeBreakout,
    /// Event-handler injection via an HTML element attribute (e.g. `onerror`).
    EventHandler,
    /// Injection into an already-open JavaScript string or expression context.
    JSContext,
    /// Bare `<script>…</script>` injection with optional whitespace and
    /// encoding.
    ScriptTag,
}

impl Display for IpmType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let name = match *self {
            Self::ScriptTag => "ScriptTag",
            Self::EventHandler => "EventHandler",
            Self::AttributeBreakout => "AttributeBreakout",
            Self::JSContext => "JSContext",
        };
        f.write_str(name)
    }
}

/// Resolves a logical token name to its concrete string value.
///
/// If `key` is not present in `map` the key itself is returned, allowing
/// literal strings to pass through unchanged.
pub fn sym<'map>(map: &'map HashMap<&str, String>, key: &'map str) -> &'map str {
    map.get(key).map_or(key, |val| val.as_str())
}

/// Infers the [`IpmType`] from the CSV column headers.
///
/// Returns `None` when no known schema matches.
pub fn detect_ipm(headers: &[String]) -> Option<IpmType> {
    let cols: Vec<&str> = headers.iter().map(|header| header.trim()).collect();
    if cols.contains(&"open_tag") && cols.contains(&"close_tag") && cols.contains(&"whitespace") {
        return Some(IpmType::ScriptTag);
    }
    if cols.contains(&"tag") && cols.contains(&"src_attr") && cols.contains(&"quote_style") {
        return Some(IpmType::EventHandler);
    }
    if cols.contains(&"breakout") && cols.contains(&"prefix") && cols.contains(&"suffix") {
        return Some(IpmType::AttributeBreakout);
    }
    if cols.contains(&"breakout") && cols.contains(&"separator") && cols.contains(&"comment") {
        return Some(IpmType::JSContext);
    }
    None
}

/// Returns the trimmed field value from `row`, or `""` if the key is absent.
fn field<'row>(row: &'row HashMap<String, String>, key: &str) -> &'row str {
    row.get(key).map_or("", |val| val.as_str()).trim()
}

/// Generates a script-tag XSS payload from a single CSV row.
pub fn translate_scripttag(row: &HashMap<String, String>, map: &HashMap<&str, String>) -> String {
    let open_tag = sym(map, field(row, "open_tag"));
    let payload = sym(map, field(row, "payload"));
    let close_tag = sym(map, field(row, "close_tag"));
    let whitespace = sym(map, field(row, "whitespace"));
    let safe_ws = if whitespace == "\n" { " " } else { whitespace };
    let encoding = field(row, "encoding");

    apply_encoding(&format!("{open_tag}{safe_ws}{payload}{close_tag}"), encoding)
}

/// Generates an event-handler XSS payload from a single CSV row.
pub fn translate_eventhandler(row: &HashMap<String, String>, map: &HashMap<&str, String>) -> String {
    let tag_raw = sym(map, field(row, "tag"));
    let tag = apply_case(tag_raw, field(row, "tag_case"));
    let src_attr = sym(map, field(row, "src_attr"));
    let event = sym(map, field(row, "event"));
    let payload = sym(map, field(row, "payload"));

    let contains_double_quote = payload.contains('"');
    let contains_single_quote = payload.contains('\'');

    let (quote_char, safe_payload) = match field(row, "quote_style") {
        "no_quote" if !contains_double_quote && !contains_single_quote && !payload.contains(' ') => {
            ("", String::from(payload))
        },
        "squote" if !contains_single_quote => ("'", String::from(payload)),
        "dquote" if !contains_double_quote => ("\"", String::from(payload)),
        _ => ("\"", payload.replace('"', "&quot;")),
    };

    let mut result = format!("<{tag}");
    if !src_attr.is_empty() {
        write!(result, " {src_attr}").unwrap_or_default();
    }
    write!(result, " {event}={quote_char}{safe_payload}{quote_char}>").unwrap_or_default();
    result
}

/// Generates an attribute-breakout XSS payload from a single CSV row.
pub fn translate_attributebreakout(row: &HashMap<String, String>, map: &HashMap<&str, String>) -> String {
    let breakout = sym(map, field(row, "breakout"));
    let prefix = sym(map, field(row, "prefix"));
    let tag = sym(map, field(row, "tag"));
    let event = sym(map, field(row, "event"));
    let payload = sym(map, field(row, "payload"));
    let suffix_key = field(row, "suffix");

    let quote_char = if payload.contains('"') { "'" } else { "\"" };

    match suffix_key {
        "space_slash" => format!("{breakout}{prefix}<{tag} {event}={quote_char}{payload}{quote_char} />"),
        "comment_close" => format!("{breakout}{prefix}<{tag} {event}={quote_char}{payload}{quote_char}>-->"),
        _ => format!("{breakout}{prefix}<{tag} {event}={quote_char}{payload}{quote_char}>"),
    }
}

/// Generates a JavaScript-context XSS payload from a single CSV row.
pub fn translate_jscontext(row: &HashMap<String, String>, map: &HashMap<&str, String>) -> String {
    let breakout = sym(map, field(row, "breakout"));
    let separator = sym(map, field(row, "separator"));
    let payload = sym(map, field(row, "payload"));
    let comment = sym(map, field(row, "comment"));
    let encoding = field(row, "encoding");

    let encoded =
        if encoding == "unicode_escape" { apply_encoding(payload, "unicode_escape") } else { String::from(payload) };

    format!("{breakout}{separator}{encoded}{separator}{comment}")
}
