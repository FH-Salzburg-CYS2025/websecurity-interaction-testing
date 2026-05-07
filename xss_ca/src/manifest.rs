//! Manifest helpers for tracking which input files have been processed.
//!
//! Each operation (translate, cluster) writes a `.processed` manifest file
//! into its output directory. The canonical absolute path of every input file
//! is appended on success; subsequent runs skip files already listed there.

use crate::error::AppError;
use std::{
    fs::{self, OpenOptions},
    io::Write as _,
    path::Path,
};

/// Name of the manifest file placed inside each output directory.
pub const MANIFEST: &str = ".processed";

/// Returns `true` if `canonical_path` appears as a line in `manifest_path`.
///
/// Returns `false` when the manifest does not yet exist or cannot be read.
pub fn is_already_processed(manifest_path: &Path, canonical_path: &str) -> bool {
    fs::read_to_string(manifest_path).is_ok_and(|contents| contents.lines().any(|line| line == canonical_path))
}

/// Appends `canonical_path` as a new line to `manifest_path`.
///
/// # Errors
///
/// Returns an error if the manifest file cannot be opened or written.
pub fn record_as_processed(manifest_path: &Path, canonical_path: &str) -> Result<(), AppError> {
    let mut manifest = match OpenOptions::new().create(true).append(true).open(manifest_path) {
        Ok(file) => file,
        Err(err) => return Err(AppError(format!("Could not open manifest: {err}"))),
    };
    match writeln!(manifest, "{canonical_path}") {
        Ok(()) => Ok(()),
        Err(err) => Err(AppError(format!("Could not write manifest: {err}"))),
    }
}

/// Resolves `path` to its canonical absolute form for use as a manifest key.
///
/// # Errors
///
/// Returns an error if the path cannot be resolved (e.g. does not exist).
pub fn canonical(path: &str) -> Result<String, AppError> {
    match fs::canonicalize(path) {
        Ok(abs) => Ok(abs.to_string_lossy().into_owned()),
        Err(err) => Err(AppError(format!("Could not resolve {path}: {err}"))),
    }
}
