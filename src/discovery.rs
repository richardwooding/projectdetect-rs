//! Layered discovery of project-type config files.
//!
//! Two layers are searched, in precedence order (later overrides earlier):
//! a **user-wide** config under the platform config dir, then a
//! **per-project** config under the current working directory.

use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::registry::Registry;

/// The config file basename searched in each layer.
pub const CONFIG_FILE_NAME: &str = "project-types.yaml";
/// The user-wide config subdirectory (under the platform config dir).
pub const CONFIG_DIR_NAME: &str = "file-search-on";
/// The per-project config subdirectory (under the current directory).
pub const PER_PROJECT_DIR_NAME: &str = ".file-search-on";

/// One config search location with its scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryEntry {
    /// `"user-wide"` or `"per-project"`.
    pub scope: &'static str,
    /// The candidate config file path (may not exist).
    pub path: PathBuf,
}

/// Returns the ordered config search locations: user-wide first, then
/// per-project. Entries whose anchor cannot be resolved (no platform config
/// dir, or no current dir) are omitted.
pub fn discovery_entries() -> Vec<DiscoveryEntry> {
    entries_from(dirs::config_dir(), std::env::current_dir().ok())
}

/// Returns just the candidate paths from [`discovery_entries`].
pub fn discovery_paths() -> Vec<PathBuf> {
    discovery_entries().into_iter().map(|e| e.path).collect()
}

/// Builds the entry list from explicit anchors (the platform config dir and
/// the current directory). Split out so tests can inject anchors without
/// mutating process-wide environment.
fn entries_from(user_config_dir: Option<PathBuf>, cwd: Option<PathBuf>) -> Vec<DiscoveryEntry> {
    let mut out = Vec::with_capacity(2);
    if let Some(base) = user_config_dir {
        out.push(DiscoveryEntry {
            scope: "user-wide",
            path: base.join(CONFIG_DIR_NAME).join(CONFIG_FILE_NAME),
        });
    }
    if let Some(cwd) = cwd {
        out.push(DiscoveryEntry {
            scope: "per-project",
            path: cwd.join(PER_PROJECT_DIR_NAME).join(CONFIG_FILE_NAME),
        });
    }
    out
}

impl Registry {
    /// Loads every config found via [`discovery_paths`] into this registry, in
    /// precedence order. Missing files are not errors; a real error (YAML
    /// parse, bad CEL/glob, missing indicators) halts loading and is returned.
    /// Returns the total number of types registered.
    pub fn load_discovered(&mut self) -> Result<usize> {
        self.load_paths(&discovery_paths())
    }

    /// Loads each existing path in order, summing the registered counts.
    /// Non-existent paths are skipped.
    fn load_paths(&mut self, paths: &[PathBuf]) -> Result<usize> {
        let mut total = 0;
        for path in paths {
            if Path::new(path).is_file() {
                total += self.load_from_file(path)?;
            }
        }
        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_order_user_wide_then_cwd() {
        let entries = entries_from(
            Some(PathBuf::from("/home/u/.config")),
            Some(PathBuf::from("/work/proj")),
        );
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].scope, "user-wide");
        assert!(entries[0]
            .path
            .ends_with(format!("{CONFIG_DIR_NAME}/{CONFIG_FILE_NAME}")));
        assert_eq!(entries[1].scope, "per-project");
        assert!(entries[1]
            .path
            .ends_with(format!("{PER_PROJECT_DIR_NAME}/{CONFIG_FILE_NAME}")));
    }

    #[test]
    fn entries_omit_unresolvable_anchors() {
        assert!(entries_from(None, None).is_empty());
        assert_eq!(entries_from(Some(PathBuf::from("/x")), None).len(), 1);
        assert_eq!(entries_from(None, Some(PathBuf::from("/y"))).len(), 1);
    }

    fn write(path: &Path, body: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    #[test]
    fn load_paths_layers_both_configs() {
        let tmp = tempfile::tempdir().unwrap();
        let user = tmp.path().join("user/project-types.yaml");
        let proj = tmp.path().join("proj/project-types.yaml");
        write(
            &user,
            "project_types:\n  - name: user-wide-app\n    indicators:\n      - has_file: user.marker\n",
        );
        write(
            &proj,
            "project_types:\n  - name: project-local-app\n    indicators:\n      - has_file: local.marker\n",
        );

        let mut reg = Registry::new();
        let n = reg.load_paths(&[user, proj]).unwrap();
        assert_eq!(n, 2);

        let names: Vec<&str> = reg.types().iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"user-wide-app"));
        assert!(names.contains(&"project-local-app"));
    }

    #[test]
    fn load_paths_no_configs_is_ok() {
        let mut reg = Registry::new();
        let missing = PathBuf::from("/definitely/not/here/project-types.yaml");
        assert_eq!(reg.load_paths(&[missing]).unwrap(), 0);
    }

    #[test]
    fn load_paths_bad_config_surfaces() {
        let tmp = tempfile::tempdir().unwrap();
        let bad = tmp.path().join("bad/project-types.yaml");
        // Missing indicators → validation error.
        write(&bad, "project_types:\n  - name: broken\n");
        let mut reg = Registry::new();
        assert!(reg.load_paths(&[bad]).is_err());
    }
}
