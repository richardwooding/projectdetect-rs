//! Shared test helpers.
//!
//! Each integration-test binary pulls this module in via `mod common;` and
//! uses only a subset of the helpers, so per-binary dead-code warnings are
//! expected and suppressed.
#![allow(dead_code)]

use std::fs;
use std::path::Path;

/// Creates an empty file at `dir/name` (creating parent dirs as needed).
pub fn touch(dir: &Path, name: &str) {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, b"x").unwrap();
}

/// Writes `body` to `dir/name` (creating parent dirs as needed).
pub fn write(dir: &Path, name: &str, body: &str) {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

/// Creates the directory `dir/name` (and parents).
pub fn mkdir(dir: &Path, name: &str) {
    fs::create_dir_all(dir.join(name)).unwrap();
}

/// Returns the sorted list of matched type names for a set of matches.
pub fn type_names(matches: &[projectdetect::Match]) -> Vec<String> {
    let mut v: Vec<String> = matches.iter().map(|m| m.r#type.clone()).collect();
    v.sort();
    v
}
