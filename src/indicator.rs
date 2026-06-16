//! Indicators (match rules) and the [`Match`] result type.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A single match rule evaluated against a directory's listing.
///
/// Indicators are OR'd within a [`crate::ProjectType`]: the first one that
/// fires identifies the directory as that type. The variant determines what
/// the rule sees:
///
/// - [`Indicator::HasFile`] — case-insensitive exact **file** basename match
///   (ASCII fold, e.g. `NuGet.Config` matches `nuget.config`).
/// - [`Indicator::HasGlob`] — glob over **file** basenames only
///   (e.g. `*.tf`). Directories are never matched.
/// - [`Indicator::HasSubdirGlob`] — glob over immediate **subdirectory**
///   basenames only (e.g. `*.xcodeproj`). Files are never matched.
/// - [`Indicator::Cel`] — a CEL expression over the `files` and `subdirs`
///   lists. Requires the `cel` cargo feature; without it, registering a type
///   with this indicator fails with [`crate::Error::CelFeatureDisabled`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Indicator {
    /// Case-insensitive exact file-basename match.
    HasFile(String),
    /// Glob over file basenames.
    HasGlob(String),
    /// Glob over immediate-subdirectory basenames.
    HasSubdirGlob(String),
    /// CEL expression over `files` / `subdirs` (requires the `cel` feature).
    Cel(String),
}

impl fmt::Display for Indicator {
    /// Human-readable form surfaced in [`Match::indicator`] for "why did this
    /// match" debuggability. A `HasSubdirGlob` renders with a trailing slash
    /// (e.g. `MyApp.xcodeproj/`) to signal a directory marker; a `Cel`
    /// indicator renders as `cel:<expr>`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Indicator::HasFile(s) | Indicator::HasGlob(s) => f.write_str(s),
            Indicator::HasSubdirGlob(s) => write!(f, "{s}/"),
            Indicator::Cel(e) => write!(f, "cel:{e}"),
        }
    }
}

/// Couples a matched project type with the indicator that fired. Surfaced by
/// [`crate::Registry::detect`] so consumers can audit detection decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Match {
    /// The [`crate::ProjectType::name`] that matched.
    #[serde(rename = "type")]
    pub r#type: String,
    /// The [`Indicator`]'s [`Display`](std::fmt::Display) form that fired.
    pub indicator: String,
}
