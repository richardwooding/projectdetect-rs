//! [`ProjectType`] and its internal compiled matchers.

use crate::error::{Error, Result};
use crate::indicator::Indicator;

/// Describes a kind of project and the [`Indicator`]s that identify it.
///
/// Indicators are evaluated against a directory's own listing (basenames
/// only — no recursion). Any single indicator matching is enough to count the
/// directory as this type (OR semantics across the list); the first match
/// wins and is returned for debuggability.
#[derive(Debug, Clone)]
pub struct ProjectType {
    /// Stable identifier, lowercase + dashes (e.g. `go`, `node`, `java-maven`).
    pub name: String,
    /// Short human-readable label.
    pub description: String,
    /// The OR-list of match rules.
    pub indicators: Vec<Indicator>,
    /// Canonical build-artefact basenames typically present in this kind of
    /// project (e.g. `vendor` for Go, `node_modules` for Node, `target` for
    /// Rust). Unioned by [`crate::Registry::collect_build_excludes`]. Empty
    /// for types with no canonical artefact dir.
    pub build_excludes: Vec<String>,

    /// Compiled matchers, one per indicator, built by
    /// [`ProjectType::compile`] at registration time. Same length and order
    /// as `indicators`.
    pub(crate) matchers: Vec<Matcher>,
}

impl ProjectType {
    /// Builds a project type with the given fields and no compiled matchers.
    /// Call [`ProjectType::compile`] (done automatically by
    /// [`crate::Registry::register`]) before using it for detection.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        indicators: Vec<Indicator>,
        build_excludes: Vec<&str>,
    ) -> Self {
        ProjectType {
            name: name.into(),
            description: description.into(),
            indicators,
            build_excludes: build_excludes.into_iter().map(String::from).collect(),
            matchers: Vec::new(),
        }
    }

    /// Compiles every indicator into a [`Matcher`]. Returns an error on a bad
    /// glob, a CEL compile failure, or a CEL indicator when the `cel` feature
    /// is disabled. Idempotent-ish: rebuilds `matchers` from `indicators`.
    pub(crate) fn compile(&mut self) -> Result<()> {
        let mut matchers = Vec::with_capacity(self.indicators.len());
        for (i, ind) in self.indicators.iter().enumerate() {
            matchers.push(Matcher::compile(ind, &self.name, i)?);
        }
        self.matchers = matchers;
        Ok(())
    }

    /// Reports whether any indicator fires against the supplied listing,
    /// returning the matching indicator's [`Display`](std::fmt::Display)
    /// string on success. `files` holds file basenames; `subdirs` holds
    /// immediate subdirectory basenames.
    pub(crate) fn match_listing(&self, files: &[String], subdirs: &[String]) -> Option<String> {
        for (i, m) in self.matchers.iter().enumerate() {
            if m.matches(files, subdirs) {
                return Some(self.indicators[i].to_string());
            }
        }
        None
    }
}

/// A compiled, ready-to-evaluate form of an [`Indicator`].
#[derive(Debug, Clone)]
pub(crate) enum Matcher {
    /// Case-insensitive exact file-basename match (ASCII fold).
    File(String),
    /// Precompiled glob over file basenames.
    Glob(glob::Pattern),
    /// Precompiled glob over subdirectory basenames.
    SubdirGlob(glob::Pattern),
    /// Compiled CEL program over `files` / `subdirs`.
    #[cfg(feature = "cel")]
    Cel(crate::cel::Program),
}

impl Matcher {
    fn compile(ind: &Indicator, type_name: &str, index: usize) -> Result<Self> {
        match ind {
            Indicator::HasFile(s) => Ok(Matcher::File(s.clone())),
            Indicator::HasGlob(s) => Ok(Matcher::Glob(compile_glob(s, type_name, index)?)),
            Indicator::HasSubdirGlob(s) => {
                Ok(Matcher::SubdirGlob(compile_glob(s, type_name, index)?))
            }
            Indicator::Cel(expr) => compile_cel(expr, type_name, index),
        }
    }

    fn matches(&self, files: &[String], subdirs: &[String]) -> bool {
        match self {
            // ASCII case-insensitive exact match — mirrors Go's equalFold.
            Matcher::File(want) => files.iter().any(|n| n.eq_ignore_ascii_case(want)),
            // Glob over files only (directories never matched).
            Matcher::Glob(p) => files.iter().any(|n| p.matches(n)),
            // Glob over subdirectories only (files never matched).
            Matcher::SubdirGlob(p) => subdirs.iter().any(|n| p.matches(n)),
            #[cfg(feature = "cel")]
            Matcher::Cel(prog) => prog.eval(files, subdirs),
        }
    }
}

fn compile_glob(pattern: &str, type_name: &str, index: usize) -> Result<glob::Pattern> {
    glob::Pattern::new(pattern).map_err(|e| Error::Config {
        name: type_name.to_string(),
        reason: format!("indicator[{index}] bad glob {pattern:?}: {e}"),
    })
}

#[cfg(feature = "cel")]
fn compile_cel(expr: &str, type_name: &str, index: usize) -> Result<Matcher> {
    crate::cel::Program::compile(expr)
        .map(Matcher::Cel)
        .map_err(|reason| Error::Cel {
            name: type_name.to_string(),
            index,
            reason,
        })
}

#[cfg(not(feature = "cel"))]
fn compile_cel(_expr: &str, type_name: &str, _index: usize) -> Result<Matcher> {
    Err(Error::CelFeatureDisabled {
        name: type_name.to_string(),
    })
}
