//! The [`Registry`] of project types and the process-wide default registry.

use once_cell::sync::Lazy;
use std::sync::RwLock;

use crate::builtins;
use crate::error::Result;
use crate::project_type::ProjectType;

/// Holds the registered [`ProjectType`]s used for detection.
///
/// The ergonomic entry point is [`Registry::with_builtins`], which returns a
/// registry preloaded with all built-in types. The crate also maintains a
/// process-wide [default registry](crate::default_registry) that backs the
/// free functions ([`crate::detect`], [`crate::find`], …) for parity with the
/// Go library's package-level API.
#[derive(Debug, Default, Clone)]
pub struct Registry {
    pub(crate) types: Vec<ProjectType>,
}

impl Registry {
    /// Returns an empty registry — useful for tests that want isolation from
    /// the built-ins.
    pub fn new() -> Self {
        Registry { types: Vec::new() }
    }

    /// Returns a registry preloaded with every built-in project type.
    pub fn with_builtins() -> Self {
        let mut r = Registry::new();
        builtins::register_builtins(&mut r);
        r
    }

    /// Registers a project type, compiling its indicators (validates globs and
    /// CEL, and errors if a CEL indicator is present without the `cel`
    /// feature). Returns an error without mutating the registry on failure.
    pub fn register(&mut self, mut t: ProjectType) -> Result<()> {
        t.compile()?;
        self.types.push(t);
        Ok(())
    }

    /// Returns a snapshot of every registered type, sorted by name.
    pub fn types(&self) -> Vec<&ProjectType> {
        let mut out: Vec<&ProjectType> = self.types.iter().collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }
}

/// The process-wide default registry, preloaded with the built-ins. Backs the
/// crate-level free functions and supports runtime [`crate::register`].
pub(crate) static DEFAULT: Lazy<RwLock<Registry>> =
    Lazy::new(|| RwLock::new(Registry::with_builtins()));
