//! Error type for the crate.

use std::path::PathBuf;

/// Errors surfaced by registration, config loading, and CEL compilation.
///
/// Detection itself ([`crate::Registry::detect`]) is infallible — an
/// unreadable directory yields an empty match list rather than an error,
/// mirroring the Go library's `Detect`.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An I/O error reading a config file or walking the tree.
    #[error("{path}: {source}")]
    Io {
        /// The path being operated on when the error occurred.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// A YAML parse error while loading a project-type config.
    #[error("{path}: YAML: {source}")]
    Yaml {
        /// The config file that failed to parse.
        path: PathBuf,
        /// The underlying YAML error.
        source: serde_norway::Error,
    },

    /// A project-type config entry failed validation (e.g. missing name,
    /// no indicators, or an indicator with no rule set).
    #[error("project type {name:?}: {reason}")]
    Config {
        /// The offending entry's name (may be empty if the name was the problem).
        name: String,
        /// Human-readable description of the validation failure.
        reason: String,
    },

    /// A CEL indicator failed to compile.
    #[error("project type {name:?}: indicator[{index}] CEL compile: {reason}")]
    Cel {
        /// The project type whose indicator failed.
        name: String,
        /// The zero-based indicator index within the type.
        index: usize,
        /// The compiler's error message.
        reason: String,
    },

    /// A project type uses a CEL indicator but the `cel` cargo feature is
    /// not enabled. Mirrors the Go "no CEL compiler installed" error.
    #[error(
        "project type {name:?} uses a CEL indicator but the `cel` feature is not enabled — \
         build with `--features cel`"
    )]
    CelFeatureDisabled {
        /// The project type that requested CEL.
        name: String,
    },
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;
