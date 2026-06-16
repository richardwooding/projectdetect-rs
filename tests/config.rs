//! YAML config loading — ports `config_test.go`.

mod common;

use common::{touch, write};
use projectdetect::Registry;
use tempfile::tempdir;

fn detects(reg: &Registry, dir: &std::path::Path, want_type: &str) -> bool {
    reg.detect(dir).iter().any(|m| m.r#type == want_type)
}

/// Mixed `has_file` / `has_glob` indicators load and detect.
#[test]
fn mixed_indicators() {
    let tmp = tempdir().unwrap();
    write(
        tmp.path(),
        "types.yaml",
        r#"
project_types:
  - name: helm-chart
    indicators:
      - has_file: Chart.yaml
  - name: tf-stack
    indicators:
      - has_glob: "*.tf"
"#,
    );
    let mut reg = Registry::new();
    let n = reg.load_from_file(tmp.path().join("types.yaml")).unwrap();
    assert_eq!(n, 2);

    let chart = tempdir().unwrap();
    touch(chart.path(), "Chart.yaml");
    assert!(detects(&reg, chart.path(), "helm-chart"));

    let tf = tempdir().unwrap();
    touch(tf.path(), "main.tf");
    assert!(detects(&reg, tf.path(), "tf-stack"));
}

/// `has_subdir_glob` matches a subdirectory bundle but not a same-named file.
#[test]
fn subdir_glob_indicator() {
    let tmp = tempdir().unwrap();
    write(
        tmp.path(),
        "types.yaml",
        "project_types:\n  - name: xcode-app\n    indicators:\n      - has_subdir_glob: \"*.xcodeproj\"\n",
    );
    let mut reg = Registry::new();
    assert_eq!(
        reg.load_from_file(tmp.path().join("types.yaml")).unwrap(),
        1
    );

    let bundle = tempdir().unwrap();
    common::mkdir(bundle.path(), "MyApp.xcodeproj");
    assert!(detects(&reg, bundle.path(), "xcode-app"));

    let as_file = tempdir().unwrap();
    touch(as_file.path(), "MyApp.xcodeproj");
    assert!(!detects(&reg, as_file.path(), "xcode-app"));
}

/// An entry with no indicators is a load error.
#[test]
fn missing_indicators_errors() {
    let tmp = tempdir().unwrap();
    write(
        tmp.path(),
        "types.yaml",
        "project_types:\n  - name: missing\n",
    );
    let mut reg = Registry::new();
    assert!(reg.load_from_file(tmp.path().join("types.yaml")).is_err());
}

/// Without the `cel` feature, a `cel:` indicator fails to register.
#[cfg(not(feature = "cel"))]
#[test]
fn cel_disabled_errors() {
    let tmp = tempdir().unwrap();
    write(
        tmp.path(),
        "types.yaml",
        "project_types:\n  - name: c\n    indicators:\n      - cel: '\"x\" in files'\n",
    );
    let mut reg = Registry::new();
    assert!(reg.load_from_file(tmp.path().join("types.yaml")).is_err());
}

/// A CEL indicator with AND-semantics across `files` and `subdirs`.
#[cfg(feature = "cel")]
#[test]
fn cel_indicator_and_semantics() {
    let tmp = tempdir().unwrap();
    write(
        tmp.path(),
        "types.yaml",
        "project_types:\n  - name: my-app\n    indicators:\n      - cel: '\"services\" in subdirs && \"foo.yaml\" in files'\n",
    );
    let mut reg = Registry::new();
    assert_eq!(
        reg.load_from_file(tmp.path().join("types.yaml")).unwrap(),
        1
    );

    // Has both → matches.
    let yes = tempdir().unwrap();
    common::mkdir(yes.path(), "services");
    touch(yes.path(), "foo.yaml");
    assert!(detects(&reg, yes.path(), "my-app"));

    // Missing the `services` subdir → no match.
    let no = tempdir().unwrap();
    touch(no.path(), "foo.yaml");
    assert!(!detects(&reg, no.path(), "my-app"));
}

/// A malformed CEL expression is a compile error at load time.
#[cfg(feature = "cel")]
#[test]
fn bad_cel_errors() {
    let tmp = tempdir().unwrap();
    write(
        tmp.path(),
        "types.yaml",
        "project_types:\n  - name: broken\n    indicators:\n      - cel: 'this is not valid cel ((('\n",
    );
    let mut reg = Registry::new();
    assert!(reg.load_from_file(tmp.path().join("types.yaml")).is_err());
}
