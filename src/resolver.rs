//! File→project resolution: walk up from a path to its nearest project root.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::indicator::Match;
use crate::registry::Registry;

/// Lexically normalizes a path (collapses `.` and redundant separators)
/// without touching the filesystem — mirrors Go's `filepath.Clean` closely
/// enough for ancestor comparisons. `..` components are preserved.
fn normalize(p: &Path) -> PathBuf {
    p.components().collect()
}

impl Registry {
    /// One-shot, uncached walk-up: finds the nearest ancestor directory of
    /// `file_path` (starting at its parent) that detects as a project, walking
    /// all the way to the filesystem root.
    ///
    /// Returns the project root and its matched types, or `None` if no
    /// ancestor matches. For "which project does this file belong to?" queries.
    pub fn resolve_for_path(&self, file_path: impl AsRef<Path>) -> Option<(PathBuf, Vec<Match>)> {
        let mut dir = normalize(file_path.as_ref().parent()?);
        loop {
            let m = self.detect(&dir);
            if !m.is_empty() {
                return Some((dir, m));
            }
            match dir.parent() {
                Some(parent) => dir = parent.to_path_buf(),
                None => return None,
            }
        }
    }
}

/// A caching file→project resolver bound to a registry and a walk-up root.
///
/// Best for batch operations: each unique directory is detected at most once
/// (negative results cached too). Unlike [`Registry::resolve_for_path`], the
/// walk-up stops at the resolver's `root` (in addition to the filesystem root
/// and the first match).
pub struct Resolver<'a> {
    registry: &'a Registry,
    root: PathBuf,
    cache: Mutex<HashMap<PathBuf, Vec<Match>>>,
}

impl<'a> Resolver<'a> {
    /// Creates a resolver rooted at `root`, resolving against `registry`.
    pub fn new(root: impl AsRef<Path>, registry: &'a Registry) -> Self {
        Resolver {
            registry,
            root: normalize(root.as_ref()),
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Walks up from `file_path`'s parent to the nearest project root, stopping
    /// at the resolver root, the filesystem root, or the first match. Returns
    /// the matched types (empty if none).
    pub fn resolve(&self, file_path: impl AsRef<Path>) -> Vec<Match> {
        let mut dir = match file_path.as_ref().parent() {
            Some(p) => normalize(p),
            None => return Vec::new(),
        };
        loop {
            let m = self.detect_cached(&dir);
            if !m.is_empty() {
                return m;
            }
            if dir == self.root {
                return Vec::new();
            }
            match dir.parent() {
                Some(parent) => dir = parent.to_path_buf(),
                None => return Vec::new(),
            }
        }
    }

    fn detect_cached(&self, dir: &Path) -> Vec<Match> {
        if let Some(cached) = self.cache.lock().unwrap().get(dir) {
            return cached.clone();
        }
        let m = self.registry.detect(dir);
        self.cache
            .lock()
            .unwrap()
            .insert(dir.to_path_buf(), m.clone());
        m
    }
}
