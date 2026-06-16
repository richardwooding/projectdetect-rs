//! Recursive project discovery ([`Registry::find`]).

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::detect::read_listing;
use crate::error::Result;
use crate::indicator::Match;
use crate::registry::Registry;

/// Configures a recursive search for project roots under a starting directory.
#[derive(Debug, Clone, Default)]
pub struct FindOptions {
    /// When non-empty, restricts results to projects matching at least one of
    /// the named types. Empty means accept all.
    pub types: Vec<String>,
    /// Basename globs that prune directories during the walk (literal names
    /// also work).
    pub excludes: Vec<String>,
    /// When true, keeps walking inside a matched project root so nested
    /// sub-projects (monorepo workspaces, vendored deps) are also reported.
    /// Default (false) stops at the first match — the common "find all my Go
    /// repos" shape.
    pub nested: bool,
    /// Parses a `.gitignore` at the walk root and prunes matching paths.
    /// Nested `.gitignore` files are not consulted.
    pub respect_gitignore: bool,
    /// Bounds the walk. `None` means no timeout. On expiry,
    /// [`FindResult::cancelled`] is set with the partial result.
    pub timeout: Option<Duration>,
    /// Optional cooperative cancellation flag. When it becomes `true` the walk
    /// stops and returns the partial result with reason `client_cancel`.
    pub cancel: Option<Arc<AtomicBool>>,
}

/// The structured output of [`Registry::find`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FindResult {
    /// The project roots found.
    pub projects: Vec<FoundProject>,
    /// Number of projects found (`projects.len()`).
    pub count: usize,
    /// True if a timeout or cancellation cut the walk short.
    #[serde(default, skip_serializing_if = "is_false")]
    pub cancelled: bool,
    /// `"timeout"` or `"client_cancel"` when [`Self::cancelled`] is set.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub cancellation_reason: String,
    /// Wall-clock duration of the walk, in seconds.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub elapsed_seconds: f64,
}

/// One project root found during a [`Registry::find`] walk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundProject {
    /// Absolute or root-relative path of the project directory.
    pub path: PathBuf,
    /// The types matched at this directory.
    pub types: Vec<Match>,
}

fn is_false(b: &bool) -> bool {
    !*b
}
fn is_zero(f: &f64) -> bool {
    *f == 0.0
}

impl Registry {
    /// Walks `root` recursively and returns every directory matching at least
    /// one project type (subject to [`FindOptions::types`]).
    ///
    /// By default the walker does **not** descend into matched roots
    /// ([`FindOptions::nested`] = false) — set it true to also surface
    /// sub-projects. Honours [`FindOptions::timeout`] and
    /// [`FindOptions::cancel`]; on expiry the partial result is returned with
    /// [`FindResult::cancelled`] set (cancellation is not an error).
    pub fn find(&self, root: impl AsRef<Path>, opts: &FindOptions) -> Result<FindResult> {
        let root = root.as_ref();
        let start = Instant::now();

        let want: Option<HashSet<String>> = if opts.types.is_empty() {
            None
        } else {
            Some(opts.types.iter().cloned().collect())
        };

        let gitignore = if opts.respect_gitignore {
            build_root_gitignore(root)
        } else {
            None
        };

        let mut walk = Walk {
            reg: self,
            opts,
            excluder: Excluder::new(&opts.excludes),
            gitignore,
            want,
            deadline: opts.timeout.map(|d| start + d),
            out: FindResult::default(),
        };
        walk.walk(root, true);

        let mut out = walk.out;
        out.count = out.projects.len();
        out.elapsed_seconds = start.elapsed().as_secs_f64();
        Ok(out)
    }
}

struct Walk<'a> {
    reg: &'a Registry,
    opts: &'a FindOptions,
    excluder: Excluder,
    gitignore: Option<ignore::gitignore::Gitignore>,
    want: Option<HashSet<String>>,
    deadline: Option<Instant>,
    out: FindResult,
}

impl Walk<'_> {
    /// Recursively visits `dir`. Returns `false` if the whole walk was
    /// cancelled (so callers unwind); `true` to continue with siblings.
    fn walk(&mut self, dir: &Path, is_root: bool) -> bool {
        if let Some(reason) = self.check_cancel() {
            self.out.cancelled = true;
            self.out.cancellation_reason = reason.to_string();
            return false;
        }

        // Excludes / gitignore prune a directory (and its subtree) before we
        // read it — applied to every dir except the walk root.
        if !is_root {
            let base = dir
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            if self.excluder.skip(&base) {
                return true;
            }
            if let Some(gi) = &self.gitignore {
                if gi.matched(dir, true).is_ignore() {
                    return true;
                }
            }
        }

        // Permission errors / vanished entries: skip this dir, keep walking
        // (mirrors Go's WalkDir error handling).
        let (files, subdirs) = match read_listing(dir) {
            Ok(v) => v,
            Err(_) => return true,
        };

        let mut matches = self.reg.match_dir(&files, &subdirs);
        if !matches.is_empty() {
            if let Some(want) = &self.want {
                matches.retain(|m| want.contains(&m.r#type));
            }
            if !matches.is_empty() {
                self.out.projects.push(FoundProject {
                    path: dir.to_path_buf(),
                    types: matches,
                });
                if !self.opts.nested {
                    // Matched and not nested: don't descend (Go's SkipDir).
                    return true;
                }
            }
        }

        // Descend in lexical order for deterministic results.
        let mut subs: Vec<&String> = subdirs.iter().collect();
        subs.sort();
        for sd in subs {
            if !self.walk(&dir.join(sd), false) {
                return false;
            }
        }
        true
    }

    fn check_cancel(&self) -> Option<&'static str> {
        if let Some(flag) = &self.opts.cancel {
            if flag.load(Ordering::Relaxed) {
                return Some("client_cancel");
            }
        }
        if let Some(dl) = self.deadline {
            if Instant::now() >= dl {
                return Some("timeout");
            }
        }
        None
    }
}

/// Builds a [`Gitignore`](ignore::gitignore::Gitignore) from `root/.gitignore`
/// only (nested gitignores are not consulted). Returns `None` if there is no
/// readable root `.gitignore`.
fn build_root_gitignore(root: &Path) -> Option<ignore::gitignore::Gitignore> {
    let gi_path = root.join(".gitignore");
    if !gi_path.is_file() {
        return None;
    }
    let mut builder = ignore::gitignore::GitignoreBuilder::new(root);
    builder.add(&gi_path);
    builder.build().ok()
}

/// Basename matcher for [`FindOptions::excludes`]. Each pattern is compiled as
/// a glob; if it isn't a valid glob it falls back to literal equality.
struct Excluder {
    patterns: Vec<(String, Option<glob::Pattern>)>,
}

impl Excluder {
    fn new(patterns: &[String]) -> Self {
        Excluder {
            patterns: patterns
                .iter()
                .map(|p| (p.clone(), glob::Pattern::new(p).ok()))
                .collect(),
        }
    }

    fn skip(&self, name: &str) -> bool {
        self.patterns.iter().any(|(lit, pat)| match pat {
            Some(p) => p.matches(name),
            None => lit == name,
        })
    }
}
