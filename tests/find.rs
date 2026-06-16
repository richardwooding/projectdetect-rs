//! Recursive discovery — ports `projectdetect_test.go`'s Find tests.

mod common;

use common::touch;
use projectdetect::{FindOptions, Registry};
use tempfile::tempdir;

/// Non-nested (default): the walk stops at the first project root and does not
/// descend into a nested project.
#[test]
fn stops_at_project_root() {
    let reg = Registry::with_builtins();
    let root = tempdir().unwrap();
    touch(root.path(), "a/go.mod");
    touch(root.path(), "a/inner/go.mod");

    let res = reg.find(root.path(), &FindOptions::default()).unwrap();
    assert_eq!(res.count, 1, "projects={:?}", res.projects);
    assert_eq!(res.projects[0].path, root.path().join("a"));
}

/// Nested = true surfaces the inner project too.
#[test]
fn nested_true() {
    let reg = Registry::with_builtins();
    let root = tempdir().unwrap();
    touch(root.path(), "a/go.mod");
    touch(root.path(), "a/inner/Cargo.toml");

    let res = reg
        .find(
            root.path(),
            &FindOptions {
                nested: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(res.count, 2, "projects={:?}", res.projects);
}

/// The `types` filter keeps only matching project types.
#[test]
fn types_filter() {
    let reg = Registry::with_builtins();
    let root = tempdir().unwrap();
    touch(root.path(), "go-app/go.mod");
    touch(root.path(), "rust-app/Cargo.toml");
    touch(root.path(), "node-app/package.json");

    let res = reg
        .find(
            root.path(),
            &FindOptions {
                types: vec!["go".into(), "rust".into()],
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(res.count, 2, "projects={:?}", res.projects);
}

/// Excludes prune directories (and their subtrees) during the walk.
#[test]
fn excludes() {
    let reg = Registry::with_builtins();
    let root = tempdir().unwrap();
    touch(root.path(), "real/go.mod");
    touch(root.path(), "node_modules/vendored/go.mod");

    let res = reg
        .find(
            root.path(),
            &FindOptions {
                excludes: vec!["node_modules".into()],
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(res.count, 1, "projects={:?}", res.projects);
    assert_eq!(res.projects[0].path, root.path().join("real"));
}

/// A root `.gitignore` prunes matching directories when `respect_gitignore`
/// is set (this crate implements the option the Go doc promised).
#[test]
fn respects_root_gitignore() {
    let reg = Registry::with_builtins();
    let root = tempdir().unwrap();
    touch(root.path(), "kept/go.mod");
    touch(root.path(), "ignored/go.mod");
    common::write(root.path(), ".gitignore", "ignored/\n");

    let res = reg
        .find(
            root.path(),
            &FindOptions {
                respect_gitignore: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(res.count, 1, "projects={:?}", res.projects);
    assert_eq!(res.projects[0].path, root.path().join("kept"));
}

/// `collect_build_excludes` unions the build dirs of detected types.
#[test]
fn collect_build_excludes_unions() {
    let reg = Registry::with_builtins();
    let root = tempdir().unwrap();
    touch(root.path(), "g/go.mod"); // vendor
    touch(root.path(), "n/package.json"); // node_modules

    let ex = reg.collect_build_excludes(root.path()).unwrap();
    assert!(ex.contains(&"vendor".to_string()), "{ex:?}");
    assert!(ex.contains(&"node_modules".to_string()), "{ex:?}");
}
