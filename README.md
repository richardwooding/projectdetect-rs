# projectdetect

[![CI](https://github.com/richardwooding/projectdetect-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/richardwooding/projectdetect-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/projectdetect.svg)](https://crates.io/crates/projectdetect)
[![docs.rs](https://img.shields.io/docsrs/projectdetect)](https://docs.rs/projectdetect)

Detect what kind of project a directory is — pure Rust, no unsafe.

A Rust port of the Go library [`github.com/richardwooding/projectdetect`](https://github.com/richardwooding/projectdetect).

`projectdetect` answers a few questions over a filesystem:

- **`detect(dir)`** — what project type(s) does *this* directory look like? (a directory can match several at once — a Go module that also ships a `docker-compose.yml` matches both)
- **`Registry::find(root, opts)`** — walk a tree and report every project root under it.
- **`Registry::resolve_for_path(file)`** / **`Resolver`** — which project does a given file belong to (nearest-ancestor walk-up)?
- **`Registry::collect_build_excludes(root)`** — the union of canonical build-artefact dirs under a tree.

A type matches by **indicators**: an exact filename (`HasFile`, case-insensitive), a file-basename glob (`HasGlob`), a subdirectory-basename glob (`HasSubdirGlob`, for directory markers like `*.xcodeproj`), or — with the `cel` feature — a CEL expression over the directory's `files` / `subdirs`.

## Built-in types

`go`, `node`, `rust`, `python`, `ruby`, `java-maven`, `java-gradle`, `dotnet`, `terraform`, `docker-compose`, the language / build-tool ecosystems `swift`, `php`, `scala-sbt`, `scala-mill`, `cmake`, `autotools`, `r`, `zig`, `perl`, `matlab`, and the static-site generators `hugo`, `jekyll`, `eleventy`, `astro`, `gatsby`, `mkdocs`, `docusaurus`, `pelican` (28 total). The `dotnet` type covers `*.csproj` / `*.fsproj` / `*.vbproj` / `*.sln` / `*.slnx` plus `global.json` / `Directory.Build.props` / `Directory.Packages.props` / `nuget.config`. `swift` matches `Package.swift` (SwiftPM), `*.podspec` (CocoaPods), and the `*.xcodeproj` / `*.xcworkspace` bundles (Xcode); `cmake` matches `CMakeLists.txt` (C/C++).

Each type also declares its canonical build-artefact dirs (`bin`/`obj`, `node_modules`, `target`, …) — see `Registry::collect_build_excludes`.

## Install

```sh
cargo add projectdetect
```

## Usage

```no_run
// What is this directory?
for m in projectdetect::detect(".") {
    println!("{} (via {})", m.r#type, m.indicator);
}
```

Recursively find project roots under a tree:

```no_run
use projectdetect::{Registry, FindOptions};

let reg = Registry::with_builtins();
let result = reg.find("/path/to/code", &FindOptions::default())?;
for project in result.projects {
    println!("{}: {:?}", project.path.display(), project.types);
}
# Ok::<(), projectdetect::Error>(())
```

Resolve which project a file belongs to:

```no_run
let reg = projectdetect::Registry::with_builtins();
if let Some((root, types)) = reg.resolve_for_path("/path/to/code/src/main.rs") {
    println!("{} is part of {:?}", "main.rs", types);
    let _ = root;
}
```

## Custom types (YAML)

Load extra project types from YAML — `has_file`, `has_glob`, `has_subdir_glob`, or `cel`:

```yaml
project_types:
  - name: my-stack
    indicators:
      - has_file: "my.config"
      - has_glob: "*.mytool"
      - has_subdir_glob: "*.bundle"
      - cel: '"services" in subdirs && "compose.yaml" in files'
```

```no_run
let mut reg = projectdetect::Registry::with_builtins();
let n = reg.load_from_file("project-types.yaml")?; // returns the count registered
# let _ = n;
# Ok::<(), projectdetect::Error>(())
```

Configs are also discovered automatically across two layers (user-wide under the
platform config dir, then per-project under `./.file-search-on/`) via
`Registry::load_discovered`.

## CEL indicators are opt-in

The crate has **no CEL dependency** by default. `cel:` indicators are compiled by
the optional `cel` cargo feature:

```toml
[dependencies]
projectdetect = { version = "0.1", features = ["cel"] }
```

Without it, `HasFile` / `HasGlob` / `HasSubdirGlob` indicators and all built-ins
work as normal; registering a type that uses a `cel:` indicator returns a clear
error telling you to enable the feature. This keeps the CEL interpreter (and its
transitive deps) out of the build for consumers that only need filename/glob
matching.

## MSRV

The default-feature crate builds on **Rust 1.79+**. The optional `cel` feature
pulls a dependency that requires `edition2024`, so enabling it needs **Rust
1.85+**.

## Relationship to the Go library

This is a faithful 1:1 port of the Go
[`projectdetect`](https://github.com/richardwooding/projectdetect): same built-in
types, same indicator semantics (files vs. subdirs split, ASCII-case-insensitive
`HasFile`), same nested/excludes/timeout `find` behaviour. The `respect_gitignore`
option (root-only `.gitignore`) is implemented here. The API is idiomatic Rust:
`Indicator` is an enum, `Resolver` uses interior-mutable caching, and timeouts use
`Option<Duration>` plus an optional `Arc<AtomicBool>` cancellation flag.

## License

MIT © Richard Wooding
