//! YAML configuration for user-defined project types.

use std::path::Path;

use serde::Deserialize;

use crate::error::{Error, Result};
use crate::indicator::Indicator;
use crate::project_type::ProjectType;
use crate::registry::Registry;

/// The YAML schema accepted by [`Registry::load_from_file`].
///
/// ```yaml
/// project_types:
///   - name: my-app
///     description: Internal Foo app
///     indicators:
///       - cel: '"services" in subdirs && "foo.yaml" in files'
///   - name: helm-chart
///     indicators:
///       - has_file: Chart.yaml
///       - has_file: values.yaml
/// ```
#[derive(Debug, Default, Deserialize)]
pub struct Config {
    /// The project types declared in the file.
    #[serde(default)]
    pub project_types: Vec<ConfigEntry>,
}

/// One project type declared in a config file.
#[derive(Debug, Deserialize)]
pub struct ConfigEntry {
    /// Required, non-empty stable identifier.
    #[serde(default)]
    pub name: String,
    /// Optional human-readable label.
    #[serde(default)]
    pub description: String,
    /// Required, non-empty list of indicators.
    #[serde(default)]
    pub indicators: Vec<ConfigIndicator>,
}

/// The YAML representation of an [`Indicator`]. Exactly one field should be
/// set; if several are, precedence is `has_file` > `has_glob` >
/// `has_subdir_glob` > `cel` (matching the Go loader).
#[derive(Debug, Default, Deserialize)]
pub struct ConfigIndicator {
    /// Case-insensitive exact file-basename match.
    #[serde(default)]
    pub has_file: Option<String>,
    /// Glob over file basenames.
    #[serde(default)]
    pub has_glob: Option<String>,
    /// Glob over subdirectory basenames.
    #[serde(default)]
    pub has_subdir_glob: Option<String>,
    /// CEL expression over `files` / `subdirs` (requires the `cel` feature).
    #[serde(default)]
    pub cel: Option<String>,
}

impl Registry {
    /// Parses `path` as YAML and registers every project type it declares.
    ///
    /// Validates each entry (non-empty name + at least one indicator) and
    /// surfaces CEL/glob compile errors. Returns the number of types
    /// registered. Not idempotent: loading the same config twice registers
    /// the types twice.
    pub fn load_from_file(&mut self, path: impl AsRef<Path>) -> Result<usize> {
        let path = path.as_ref();
        let data = std::fs::read_to_string(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let cfg: Config = serde_norway::from_str(&data).map_err(|source| Error::Yaml {
            path: path.to_path_buf(),
            source,
        })?;
        let mut registered = 0;
        for entry in cfg.project_types {
            let pt = build_project_type(entry)?;
            self.register(pt)?;
            registered += 1;
        }
        Ok(registered)
    }
}

/// Validates a [`ConfigEntry`] and converts it to a [`ProjectType`] (not yet
/// compiled — [`Registry::register`] compiles it).
fn build_project_type(entry: ConfigEntry) -> Result<ProjectType> {
    if entry.name.is_empty() {
        return Err(Error::Config {
            name: entry.name,
            reason: "name is required".to_string(),
        });
    }
    if entry.indicators.is_empty() {
        return Err(Error::Config {
            name: entry.name,
            reason: "at least one indicator is required".to_string(),
        });
    }
    let mut indicators = Vec::with_capacity(entry.indicators.len());
    for (j, ci) in entry.indicators.into_iter().enumerate() {
        let ind = if let Some(s) = ci.has_file {
            Indicator::HasFile(s)
        } else if let Some(s) = ci.has_glob {
            Indicator::HasGlob(s)
        } else if let Some(s) = ci.has_subdir_glob {
            Indicator::HasSubdirGlob(s)
        } else if let Some(s) = ci.cel {
            Indicator::Cel(s)
        } else {
            return Err(Error::Config {
                name: entry.name,
                reason: format!(
                    "indicator[{j}]: must set has_file / has_glob / has_subdir_glob / cel"
                ),
            });
        };
        indicators.push(ind);
    }
    Ok(ProjectType::new(
        entry.name,
        entry.description,
        indicators,
        Vec::new(),
    ))
}
