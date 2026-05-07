//! XSS Covering Array Translator and Payload Clusterer.

mod cluster;
mod encoding;
mod error;
mod manifest;
mod symbols;
mod translate;

use cluster::cluster_payloads;
use csv::ReaderBuilder;
use error::AppError;
use manifest::{MANIFEST, canonical, is_already_processed, record_as_processed};
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{self, BufWriter, Write as _},
    path::Path,
};
use symbols::symbol_map;
use translate::{
    IpmType,
    detect_ipm,
    translate_attributebreakout,
    translate_eventhandler,
    translate_jscontext,
    translate_scripttag,
};

/// Marker string injected into every generated payload so results are
/// attributable to this test group in server-side logs or search output.
pub const GROUP: &str = "rowhammer";

fn main() -> Result<(), AppError> {
    let args: Vec<String> = env::args().collect();

    let flag = if let Some(arg) = args.get(1) {
        arg.as_str()
    } else {
        writeln!(io::stderr(), "Usage:").unwrap_or_default();
        writeln!(io::stderr(), "  xss_ca --translate <ca_file.csv> [...]    translate covering-array CSVs")
            .unwrap_or_default();
        writeln!(io::stderr(), "  xss_ca --cluster  <payload_file> [...]    cluster payload lists").unwrap_or_default();
        return Err(AppError("No arguments provided".into()));
    };

    let paths: Vec<&str> = args.iter().skip(2).map(String::as_str).collect();

    match flag {
        "--translate" => {
            if paths.is_empty() {
                writeln!(io::stderr(), "Usage: xss_ca --translate <ca_file.csv> [...]").unwrap_or_default();
                return Err(AppError("No CA files provided".into()));
            }
            run_for_each(&paths, translate_ca)
        },
        "--cluster" => {
            if paths.is_empty() {
                writeln!(io::stderr(), "Usage: xss_ca --cluster <payload_file> [...]").unwrap_or_default();
                return Err(AppError("No payload files provided".into()));
            }
            run_for_each(&paths, cluster_payloads)
        },
        other => {
            writeln!(io::stderr(), "Unknown flag: {other}").unwrap_or_default();
            writeln!(io::stderr(), "Usage:").unwrap_or_default();
            writeln!(io::stderr(), "  xss_ca --translate <ca_file.csv> [...]    translate covering-array CSVs")
                .unwrap_or_default();
            writeln!(io::stderr(), "  xss_ca --cluster  <payload_file> [...]    cluster payload lists")
                .unwrap_or_default();
            Err(AppError(format!("Unknown flag: {other}")))
        },
    }
}

/// Applies `op` to every path in `paths`, accumulates errors, and returns a
/// single error if any individual call failed.
///
/// # Errors
///
/// Returns an error if one or more `op` calls returned an error.
fn run_for_each(paths: &[&str], op: fn(&str) -> Result<(), AppError>) -> Result<(), AppError> {
    let mut had_error = false;
    for path in paths {
        match op(path) {
            Ok(()) => (),
            Err(err) => {
                writeln!(io::stderr(), "ERROR {path}: {err}").unwrap_or_default();
                had_error = true;
            },
        }
    }
    if had_error { Err(AppError("One or more files failed".into())) } else { Ok(()) }
}

/// Reads a covering-array CSV at `path`, detects its IPM from the headers,
/// translates every row into a concrete XSS payload, and writes results to
/// `<input_dir>/translated_exploits/<stem>.txt`.
///
/// # Errors
///
/// Returns an error if the file cannot be opened, headers cannot be parsed,
/// the IPM cannot be detected, the output directory cannot be created, or
/// the output file cannot be written.
fn translate_ca(path: &str) -> Result<(), AppError> {
    let canonical_path = match canonical(path) {
        Ok(resolved) => resolved,
        Err(err) => return Err(err),
    };

    let input_path = Path::new(path);
    let output_dir = input_path.parent().unwrap_or_else(|| Path::new(".")).join("translated_exploits");

    match fs::create_dir_all(&output_dir) {
        Ok(()) => (),
        Err(err) => return Err(AppError(format!("Could not create output dir: {err}"))),
    }

    let manifest_path = output_dir.join(MANIFEST);

    if is_already_processed(&manifest_path, &canonical_path) {
        writeln!(io::stderr(), "SKIP (already processed): {path}").unwrap_or_default();
        return Ok(());
    }

    let map = symbol_map();

    let mut reader = match ReaderBuilder::new().has_headers(true).trim(csv::Trim::All).from_path(path) {
        Ok(rdr) => rdr,
        Err(err) => return Err(AppError(format!("Could not open {path}: {err}"))),
    };

    let headers: Vec<String> = match reader.headers() {
        Ok(hdrs) => hdrs.iter().map(String::from).collect(),
        Err(err) => return Err(AppError(format!("Could not read headers: {err}"))),
    };

    let ipm = if let Some(detected) = detect_ipm(&headers) {
        detected
    } else {
        let header_list = headers.join(", ");
        return Err(AppError(format!("Could not detect IPM from headers: [{header_list}]")));
    };

    writeln!(io::stderr(), "IPM: {ipm} | Group: {GROUP}").unwrap_or_default();

    let file_stem = match input_path.file_stem().and_then(|stem| stem.to_str()) {
        Some(name) => name,
        None => return Err(AppError("Could not determine input file name".into())),
    };

    let output_path = output_dir.join(format!("{file_stem}.txt"));

    let file = match File::create(&output_path) {
        Ok(opened) => opened,
        Err(err) => return Err(AppError(format!("Could not create {}: {err}", output_path.display()))),
    };

    writeln!(io::stderr(), "Output: {}", output_path.display()).unwrap_or_default();

    let mut out = BufWriter::new(file);

    for (index, result) in reader.deserialize::<HashMap<String, String>>().enumerate() {
        match result {
            Ok(row) => {
                let exploit = match ipm {
                    IpmType::ScriptTag => translate_scripttag(&row, &map),
                    IpmType::EventHandler => translate_eventhandler(&row, &map),
                    IpmType::AttributeBreakout => translate_attributebreakout(&row, &map),
                    IpmType::JSContext => translate_jscontext(&row, &map),
                };
                match writeln!(out, "{exploit}") {
                    Ok(()) => (),
                    Err(err) => return Err(AppError(format!("Write error: {err}"))),
                }
            },
            Err(err) => {
                writeln!(io::stderr(), "ERROR row {}: {err}", index.saturating_add(1)).unwrap_or_default();
            },
        }
    }

    record_as_processed(&manifest_path, &canonical_path)
}
