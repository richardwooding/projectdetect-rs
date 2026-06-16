//! File→project resolution — ports `resolver_test.go`.

mod common;

use common::{touch, type_names};
use projectdetect::{Indicator, ProjectType, Registry, Resolver};
use tempfile::tempdir;

/// `Resolver::resolve` walks up to the nearest project root.
#[test]
fn finds_nearest_project() {
    let reg = Registry::with_builtins();
    let root = tempdir().unwrap();
    touch(root.path(), "proj/go.mod");
    touch(root.path(), "proj/cmd/main.go");
    touch(root.path(), "proj/inner/Cargo.toml");
    touch(root.path(), "proj/inner/src/lib.rs");

    let r = Resolver::new(root.path(), &reg);

    let m = r.resolve(root.path().join("proj/cmd/main.go"));
    assert_eq!(type_names(&m), ["go"]);

    // The nearer Cargo.toml wins over the outer go.mod.
    let m = r.resolve(root.path().join("proj/inner/src/lib.rs"));
    assert_eq!(type_names(&m), ["rust"]);
}

/// A file outside any project resolves to nothing.
#[test]
fn no_project() {
    let reg = Registry::with_builtins();
    let root = tempdir().unwrap();
    touch(root.path(), "loose.txt");
    let r = Resolver::new(root.path(), &reg);
    assert!(r.resolve(root.path().join("loose.txt")).is_empty());
}

/// `resolve_for_path` returns the nearest project root and its types.
#[test]
fn resolve_for_path_finds_nearest() {
    let reg = Registry::with_builtins();
    let root = tempdir().unwrap();
    touch(root.path(), "proj/go.mod");
    touch(root.path(), "proj/cmd/main.go");
    touch(root.path(), "proj/inner/Cargo.toml");
    touch(root.path(), "proj/inner/src/lib.rs");

    let (p, m) = reg
        .resolve_for_path(root.path().join("proj/cmd/main.go"))
        .unwrap();
    assert_eq!(p, root.path().join("proj"));
    assert_eq!(type_names(&m), ["go"]);

    let (p, m) = reg
        .resolve_for_path(root.path().join("proj/inner/src/lib.rs"))
        .unwrap();
    assert_eq!(p, root.path().join("proj/inner"));
    assert_eq!(type_names(&m), ["rust"]);
}

/// `resolve_for_path` returns `None` when no ancestor is a project.
#[test]
fn resolve_for_path_no_project() {
    let reg = Registry::with_builtins();
    let root = tempdir().unwrap();
    touch(root.path(), "loose.txt");
    assert!(reg
        .resolve_for_path(root.path().join("loose.txt"))
        .is_none());
}

/// A polyglot root surfaces all matched types.
#[test]
fn resolve_for_path_polyglot() {
    let reg = Registry::with_builtins();
    let root = tempdir().unwrap();
    touch(root.path(), "go.mod");
    touch(root.path(), "docker-compose.yml");
    touch(root.path(), "cmd/main.go");

    let (p, m) = reg
        .resolve_for_path(root.path().join("cmd/main.go"))
        .unwrap();
    assert_eq!(p, root.path().to_path_buf());
    assert_eq!(type_names(&m), ["docker-compose", "go"]);
}

/// The resolver respects the registry it was constructed with.
#[test]
fn custom_registry() {
    let mut reg = Registry::new();
    reg.register(ProjectType::new(
        "custom",
        "Custom",
        vec![Indicator::HasFile("custom.marker".into())],
        vec![],
    ))
    .unwrap();

    let root = tempdir().unwrap();
    touch(root.path(), "custom.marker");
    touch(root.path(), "go.mod"); // would match built-in `go` in the default registry
    touch(root.path(), "sub/file.txt");

    let r = Resolver::new(root.path(), &reg);
    let m = r.resolve(root.path().join("sub/file.txt"));
    assert_eq!(type_names(&m), ["custom"]);
}
