//! Single-directory detection and the build-exclude collector.

use std::collections::BTreeSet;
use std::path::Path;

use crate::find::FindOptions;
use crate::indicator::Match;
use crate::registry::Registry;

impl Registry {
    /// Inspects a single directory and returns the project types it matches.
    ///
    /// A directory can match multiple types simultaneously (e.g. a Go module
    /// with a `docker-compose.yml`). Returns an empty vec when no type fires
    /// or the directory cannot be read (mirrors the Go library — detection is
    /// infallible). Results are sorted by type name.
    ///
    /// Reads the directory listing once (non-recursive): `HasFile` / `HasGlob`
    /// indicators run against file basenames, `HasSubdirGlob` against
    /// immediate-subdirectory basenames, and `Cel` indicators see both.
    pub fn detect(&self, dir: impl AsRef<Path>) -> Vec<Match> {
        let (files, subdirs) = match read_listing(dir.as_ref()) {
            Ok(v) => v,
            Err(_) => return Vec::new(),
        };
        self.match_dir(&files, &subdirs)
    }

    /// Runs every registered type against an already-read directory listing
    /// and returns the sorted matches. Used by both [`Registry::detect`] and
    /// the [`Registry::find`] walk (which reads each directory once).
    pub(crate) fn match_dir(&self, files: &[String], subdirs: &[String]) -> Vec<Match> {
        let mut matches: Vec<Match> = self
            .types
            .iter()
            .filter_map(|t| {
                t.match_listing(files, subdirs).map(|ind| Match {
                    r#type: t.name.clone(),
                    indicator: ind,
                })
            })
            .collect();
        matches.sort_by(|a, b| a.r#type.cmp(&b.r#type));
        matches
    }

    /// Walks `root` and returns the sorted, deduped union of `build_excludes`
    /// from every project type detected at or below `root` (nested-style —
    /// every project, not just outer roots). Used to pre-populate a search
    /// walker's exclude set so dependency caches (`vendor/`, `node_modules/`,
    /// `target/`, …) are pruned by default.
    pub fn collect_build_excludes(
        &self,
        root: impl AsRef<Path>,
    ) -> crate::error::Result<Vec<String>> {
        let res = self.find(
            root,
            &FindOptions {
                nested: true,
                ..Default::default()
            },
        )?;
        let by_name: std::collections::HashMap<&str, &[String]> = self
            .types
            .iter()
            .map(|t| (t.name.as_str(), t.build_excludes.as_slice()))
            .collect();
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for p in &res.projects {
            for m in &p.types {
                if let Some(excludes) = by_name.get(m.r#type.as_str()) {
                    for ex in *excludes {
                        seen.insert(ex.clone());
                    }
                }
            }
        }
        Ok(seen.into_iter().collect())
    }
}

/// Returns the basenames of the immediate children of `dir`, split into files
/// and subdirectories. Symlinks are classified by their target's type via
/// [`std::fs::DirEntry::file_type`] (no traversal), matching Go's `os.ReadDir`
/// + `DirEntry.IsDir`.
pub(crate) fn read_listing(dir: &Path) -> std::io::Result<(Vec<String>, Vec<String>)> {
    let mut files = Vec::new();
    let mut subdirs = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        if is_dir {
            subdirs.push(name);
        } else {
            files.push(name);
        }
    }
    Ok((files, subdirs))
}
