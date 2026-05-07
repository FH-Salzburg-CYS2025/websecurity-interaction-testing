//! XSS payload clustering by structural pattern.

use crate::{
    error::AppError,
    manifest::{MANIFEST, canonical, is_already_processed, record_as_processed},
};
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{self, BufWriter, Write as _},
    path::Path,
};

/// Ordered set of cluster keys used as output file name suffixes.
const KEYS: &[&str] =
    &["ScriptTag", "EventHandler", "AttributeBreakout", "JSContext", "StyleCSS", "DataURI", "ProtocolHandler", "Other"];

/// Classifies a single payload line into a cluster group.
fn classify(line: &str) -> &'static str {
    let lower = line.to_lowercase();

    if lower.starts_with("<script") {
        "ScriptTag"
    } else if lower.starts_with("<style") || lower.contains("expression(") {
        "StyleCSS"
    } else if lower.contains("data:text/html") || lower.contains("data:image/svg") {
        "DataURI"
    } else if lower.contains("javascript:") || lower.contains("vbscript:") {
        "ProtocolHandler"
    } else if lower.contains("onerror=")
        || lower.contains("onload=")
        || lower.contains("onclick=")
        || lower.contains("onmouseover=")
        || lower.contains("onfocus=")
        || lower.contains("onstart=")
        || lower.contains("ontoggle=")
    {
        "EventHandler"
    } else if line.starts_with("\">")
        || line.starts_with("'>")
        || line.starts_with("\"<")
        || line.starts_with("'<")
        || line.starts_with("><")
    {
        "AttributeBreakout"
    } else if line.starts_with("'-")
        || line.starts_with("\"-")
        || line.starts_with("';")
        || line.starts_with("\";")
        || line.starts_with("`-")
        || line.starts_with("];")
    {
        "JSContext"
    } else {
        "Other"
    }
}

/// Reads a raw XSS payload list from `path`, clusters each line by structural
/// pattern, and appends each cluster to its file under a `clusters/`
/// subdirectory next to the input file.
///
/// Skips the file silently if it has already been processed according to the
/// manifest at `clusters/.processed`. On success the canonical path of `path`
/// is recorded in that manifest so it will not be processed again.
///
/// # Errors
///
/// Returns an error if the file cannot be read, the output directory cannot be
/// created, any cluster file cannot be written, or the manifest cannot be
/// updated.
pub fn cluster_payloads(path: &str) -> Result<(), AppError> {
    let canonical_path = match canonical(path) {
        Ok(resolved) => resolved,
        Err(err) => return Err(err),
    };

    let input_path = Path::new(path);
    let output_dir = input_path.parent().unwrap_or_else(|| Path::new(".")).join("clusters");

    match fs::create_dir_all(&output_dir) {
        Ok(()) => (),
        Err(err) => return Err(AppError(format!("Could not create clusters dir: {err}"))),
    }

    let manifest_path = output_dir.join(MANIFEST);

    if is_already_processed(&manifest_path, &canonical_path) {
        writeln!(io::stderr(), "SKIP (already processed): {path}").unwrap_or_default();
        return Ok(());
    }

    let content = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => return Err(AppError(format!("Could not read {path}: {err}"))),
    };

    let mut groups: HashMap<&str, Vec<String>> = KEYS.iter().map(|&key| (key, Vec::new())).collect();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let cluster_key = classify(trimmed);
        if let Some(bucket) = groups.get_mut(cluster_key) {
            bucket.push(trimmed.to_owned());
        }
    }

    let mut total: usize = 0;

    for &key in KEYS {
        let payloads = match groups.get(key) {
            Some(list) => list,
            None => continue,
        };
        if payloads.is_empty() {
            continue;
        }

        let out_path = output_dir.join(format!("{key}.txt"));
        let file = match OpenOptions::new().create(true).append(true).open(&out_path) {
            Ok(opened) => opened,
            Err(err) => return Err(AppError(format!("Could not open {}: {err}", out_path.display()))),
        };
        let mut writer = BufWriter::new(file);

        for payload in payloads {
            match writeln!(writer, "{payload}") {
                Ok(()) => (),
                Err(err) => return Err(AppError(format!("Write error: {err}"))),
            }
        }

        writeln!(io::stderr(), "  {:20} {:3} payloads -> {}", key, payloads.len(), out_path.display())
            .unwrap_or_default();

        total = total.saturating_add(payloads.len());
    }

    writeln!(io::stderr(), "Total clustered: {total} payloads from {path}").unwrap_or_default();

    record_as_processed(&manifest_path, &canonical_path)
}
