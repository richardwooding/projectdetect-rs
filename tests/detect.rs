//! Single-directory detection — ports `projectdetect_test.go`'s detect tests.

mod common;

use common::{mkdir, touch, type_names};
use projectdetect::Registry;
use tempfile::tempdir;

/// `HasFile` single-type detection: one indicator file → exactly one match,
/// and the reported indicator string equals the filename.
#[test]
fn single_type_has_file() {
    let cases = [
        ("go.mod", "go"),
        ("package.json", "node"),
        ("Cargo.toml", "rust"),
        ("pyproject.toml", "python"),
        ("requirements.txt", "python"),
        ("Pipfile", "python"),
        ("setup.py", "python"),
        ("setup.cfg", "python"),
        ("Gemfile", "ruby"),
        ("pom.xml", "java-maven"),
        ("build.gradle", "java-gradle"),
        ("build.gradle.kts", "java-gradle"),
        ("settings.gradle", "java-gradle"),
        ("compose.yaml", "docker-compose"),
        ("docker-compose.yml", "docker-compose"),
        ("Package.swift", "swift"),
        ("composer.json", "php"),
        ("build.sbt", "scala-sbt"),
        ("build.mill", "scala-mill"),
        ("build.sc", "scala-mill"),
        ("CMakeLists.txt", "cmake"),
        ("configure.ac", "autotools"),
        ("Makefile.am", "autotools"),
        ("DESCRIPTION", "r"),
        ("build.zig", "zig"),
        ("build.zig.zon", "zig"),
        ("Makefile.PL", "perl"),
        ("cpanfile", "perl"),
        ("dist.ini", "perl"),
        ("hugo.toml", "hugo"),
        ("hugo.yaml", "hugo"),
        ("_config.yml", "jekyll"),
        (".eleventy.js", "eleventy"),
        ("eleventy.config.mjs", "eleventy"),
        ("astro.config.mjs", "astro"),
        ("astro.config.ts", "astro"),
        ("gatsby-config.js", "gatsby"),
        ("mkdocs.yml", "mkdocs"),
        ("docusaurus.config.js", "docusaurus"),
        ("pelicanconf.py", "pelican"),
    ];
    let reg = Registry::with_builtins();
    for (file, want) in cases {
        let dir = tempdir().unwrap();
        touch(dir.path(), file);
        let matches = reg.detect(dir.path());
        assert_eq!(
            matches.len(),
            1,
            "{file}: expected single match, got {matches:?}"
        );
        assert_eq!(matches[0].r#type, want, "{file}");
        assert_eq!(matches[0].indicator, file, "{file}: indicator string");
    }
}

/// Glob (file) indicators — only the type is asserted (the indicator string is
/// the pattern, not the filename). Includes `dotnet`'s many markers and the
/// case-insensitive `NuGet.Config`.
#[test]
fn glob_indicators() {
    let cases = [
        ("main.tf", "terraform"),
        ("providers.tf", "terraform"),
        ("MyApp.csproj", "dotnet"),
        ("MyApp.fsproj", "dotnet"),
        ("MyApp.sln", "dotnet"),
        ("MyApp.slnx", "dotnet"),
        ("MyApp.slnf", "dotnet"),
        ("global.json", "dotnet"),
        ("Directory.Build.props", "dotnet"),
        ("Directory.Packages.props", "dotnet"),
        ("nuget.config", "dotnet"),
        ("NuGet.Config", "dotnet"),
        ("Alamofire.podspec", "swift"),
        ("myanalysis.Rproj", "r"),
        ("Export_fig.prj", "matlab"),
    ];
    let reg = Registry::with_builtins();
    for (file, want) in cases {
        let dir = tempdir().unwrap();
        touch(dir.path(), file);
        let matches = reg.detect(dir.path());
        assert_eq!(matches.len(), 1, "{file}: got {matches:?}");
        assert_eq!(matches[0].r#type, want, "{file}");
    }
}

/// Xcode subdir-only: a `*.xcodeproj` / `*.xcworkspace` **directory** detects
/// as swift via `HasSubdirGlob`, with a trailing-slash indicator string.
#[test]
fn xcode_subdir_only() {
    let reg = Registry::with_builtins();
    for (bundle, indicator) in [
        ("MyApp.xcodeproj", "*.xcodeproj/"),
        ("MyApp.xcworkspace", "*.xcworkspace/"),
    ] {
        let dir = tempdir().unwrap();
        mkdir(dir.path(), bundle);
        touch(dir.path(), "README.md");
        let matches = reg.detect(dir.path());
        assert_eq!(matches.len(), 1, "{bundle}: got {matches:?}");
        assert_eq!(matches[0].r#type, "swift");
        assert_eq!(matches[0].indicator, indicator);
    }
}

/// A `HasSubdirGlob` must not match a same-named regular FILE.
#[test]
fn subdir_glob_ignores_files() {
    let reg = Registry::with_builtins();
    let dir = tempdir().unwrap();
    touch(dir.path(), "x.xcodeproj"); // a FILE, not a bundle dir
    assert!(
        reg.detect(dir.path()).is_empty(),
        "a *.xcodeproj file must not match the subdir glob"
    );
}

/// A directory can match several types at once.
#[test]
fn multiple_types() {
    let reg = Registry::with_builtins();
    let dir = tempdir().unwrap();
    touch(dir.path(), "go.mod");
    touch(dir.path(), "docker-compose.yml");
    assert_eq!(
        type_names(&reg.detect(dir.path())),
        ["docker-compose", "go"]
    );
}

/// A modern .NET root with only a `.slnx` + `Directory.*.props` detects as
/// dotnet (the `.slnx` must not be swallowed by the `*.sln` glob).
#[test]
fn dotnet_slnx_root() {
    let reg = Registry::with_builtins();
    let dir = tempdir().unwrap();
    for f in [
        "Cel2Sql.slnx",
        "Directory.Build.props",
        "Directory.Packages.props",
    ] {
        touch(dir.path(), f);
    }
    let matches = reg.detect(dir.path());
    assert_eq!(matches.len(), 1, "got {matches:?}");
    assert_eq!(matches[0].r#type, "dotnet");
}

/// No indicator → no match.
#[test]
fn no_match() {
    let reg = Registry::with_builtins();
    let dir = tempdir().unwrap();
    touch(dir.path(), "random.txt");
    assert!(reg.detect(dir.path()).is_empty());
}

/// The registry ships at least 28 built-ins, sorted by name.
#[test]
fn registry_types_sorted_and_complete() {
    let reg = Registry::with_builtins();
    let types = reg.types();
    assert!(types.len() >= 28, "got {} types", types.len());
    for w in types.windows(2) {
        assert!(w[0].name <= w[1].name, "types not sorted");
    }
}
