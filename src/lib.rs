//! Detect what kind of project a directory is — Go module, Node app, Rust
//! crate, Xcode project, Terraform stack, and 20+ more — by checking canonical
//! indicator files and directories.
//!
//! This is a Rust port of the Go library
//! [`github.com/richardwooding/projectdetect`](https://github.com/richardwooding/projectdetect).
//!
//! # What it does
//!
//! - [`detect`] — what project type(s) does *this* directory look like? A
//!   directory can match several at once (a Go module that also ships a
//!   `docker-compose.yml` matches both `go` and `docker-compose`).
//! - [`Registry::find`] — recursively walk a tree for project roots.
//! - [`Registry::resolve_for_path`] / [`Resolver`] — which project does a
//!   given file belong to (nearest-ancestor walk-up)?
//! - [`Registry::collect_build_excludes`] — the union of canonical
//!   build-artefact dirs (`vendor`, `node_modules`, `target`, …) under a tree.
//!
//! # Quick start
//!
//! ```no_run
//! for m in projectdetect::detect(".") {
//!     println!("{} (via {})", m.r#type, m.indicator);
//! }
//! ```
//!
//! # Indicators
//!
//! A [`ProjectType`] matches by [`Indicator`]s: an exact filename
//! ([`Indicator::HasFile`], case-insensitive), a file-basename glob
//! ([`Indicator::HasGlob`]), a subdirectory-basename glob
//! ([`Indicator::HasSubdirGlob`], for directory markers like `*.xcodeproj`),
//! or — with the `cel` feature — a CEL expression ([`Indicator::Cel`]) over the
//! directory's `files` / `subdirs`.
//!
//! # Custom types
//!
//! Extra types load from YAML ([`Registry::load_from_file`]) or via layered
//! [discovery](Registry::load_discovered).

mod builtins;
#[cfg(feature = "cel")]
mod cel;
mod config;
mod detect;
mod discovery;
mod error;
mod find;
mod indicator;
mod project_type;
mod registry;
mod resolver;

use std::path::{Path, PathBuf};

pub use config::{Config, ConfigEntry, ConfigIndicator};
pub use discovery::{
    discovery_entries, discovery_paths, DiscoveryEntry, CONFIG_DIR_NAME, CONFIG_FILE_NAME,
    PER_PROJECT_DIR_NAME,
};
pub use error::{Error, Result};
pub use find::{FindOptions, FindResult, FoundProject};
pub use indicator::{Indicator, Match};
pub use project_type::ProjectType;
pub use registry::Registry;
pub use resolver::Resolver;

use registry::DEFAULT;

/// Detects the project type(s) of `dir` using the default registry. See
/// [`Registry::detect`].
pub fn detect(dir: impl AsRef<Path>) -> Vec<Match> {
    DEFAULT.read().unwrap().detect(dir)
}

/// Recursively finds project roots under `root` using the default registry.
/// See [`Registry::find`].
pub fn find(root: impl AsRef<Path>, opts: &FindOptions) -> Result<FindResult> {
    DEFAULT.read().unwrap().find(root, opts)
}

/// Resolves the nearest project root for `file_path` using the default
/// registry. See [`Registry::resolve_for_path`].
pub fn resolve_for_path(file_path: impl AsRef<Path>) -> Option<(PathBuf, Vec<Match>)> {
    DEFAULT.read().unwrap().resolve_for_path(file_path)
}

/// Returns the union of build-artefact dirs under `root` using the default
/// registry. See [`Registry::collect_build_excludes`].
pub fn collect_build_excludes(root: impl AsRef<Path>) -> Result<Vec<String>> {
    DEFAULT.read().unwrap().collect_build_excludes(root)
}

/// Registers a project type into the default registry at runtime.
pub fn register(t: ProjectType) -> Result<()> {
    DEFAULT.write().unwrap().register(t)
}

/// Loads a YAML project-type config into the default registry, returning the
/// number of types registered. See [`Registry::load_from_file`].
pub fn load_from_file(path: impl AsRef<Path>) -> Result<usize> {
    DEFAULT.write().unwrap().load_from_file(path)
}

/// Loads all discovered configs into the default registry. See
/// [`Registry::load_discovered`].
pub fn load_discovered() -> Result<usize> {
    DEFAULT.write().unwrap().load_discovered()
}

/// Returns the names of every type registered in the default registry, sorted.
pub fn type_names() -> Vec<String> {
    DEFAULT
        .read()
        .unwrap()
        .types()
        .into_iter()
        .map(|t| t.name.clone())
        .collect()
}
