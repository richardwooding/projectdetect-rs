//! The built-in project types, ported 1:1 from the Go library's `builtins.go`.

use crate::indicator::Indicator::{self, HasFile, HasGlob, HasSubdirGlob};
use crate::project_type::ProjectType;
use crate::registry::Registry;

fn file(s: &str) -> Indicator {
    HasFile(s.to_string())
}
fn glob(s: &str) -> Indicator {
    HasGlob(s.to_string())
}
fn subdir(s: &str) -> Indicator {
    HasSubdirGlob(s.to_string())
}

/// Registers every built-in project type into `r`. Panics on a compile error,
/// which would be a programming bug in a built-in definition (mirrors the Go
/// package-level `Register` panic in `init()`).
pub(crate) fn register_builtins(r: &mut Registry) {
    for t in builtins() {
        r.register(t).expect("built-in project type must compile");
    }
}

fn builtins() -> Vec<ProjectType> {
    vec![
        ProjectType::new("go", "Go module (go.mod)", vec![file("go.mod")], vec!["vendor"]),
        ProjectType::new(
            "node",
            "Node.js / npm / yarn / pnpm (package.json)",
            vec![file("package.json")],
            vec!["node_modules"],
        ),
        ProjectType::new(
            "rust",
            "Rust crate (Cargo.toml)",
            vec![file("Cargo.toml")],
            vec!["target"],
        ),
        ProjectType::new(
            "python",
            "Python project (pyproject.toml / requirements.txt / Pipfile / setup.py / setup.cfg)",
            vec![
                file("pyproject.toml"),
                file("requirements.txt"),
                file("Pipfile"),
                file("setup.py"),
                file("setup.cfg"),
            ],
            vec![
                "__pycache__",
                ".venv",
                "venv",
                ".tox",
                ".pytest_cache",
                ".mypy_cache",
                ".ruff_cache",
            ],
        ),
        ProjectType::new(
            "ruby",
            "Ruby Bundler project (Gemfile)",
            vec![file("Gemfile")],
            vec![".bundle"],
        ),
        ProjectType::new(
            "java-maven",
            "Java Maven project (pom.xml)",
            vec![file("pom.xml")],
            vec!["target"],
        ),
        ProjectType::new(
            "java-gradle",
            "Java/Kotlin Gradle project (build.gradle / build.gradle.kts)",
            vec![
                file("build.gradle"),
                file("build.gradle.kts"),
                file("settings.gradle"),
                file("settings.gradle.kts"),
            ],
            vec!["build", ".gradle"],
        ),
        ProjectType::new(
            "dotnet",
            ".NET project (*.csproj / *.fsproj / *.vbproj / *.sln / *.slnx, MSBuild + SDK markers)",
            vec![
                glob("*.csproj"),
                glob("*.fsproj"),
                glob("*.vbproj"),
                glob("*.sln"),
                glob("*.slnx"),
                glob("*.slnf"),
                file("global.json"),
                file("Directory.Build.props"),
                file("Directory.Packages.props"),
                file("nuget.config"),
            ],
            vec!["bin", "obj"],
        ),
        ProjectType::new(
            "terraform",
            "Terraform / OpenTofu (*.tf)",
            vec![glob("*.tf")],
            vec![".terraform"],
        ),
        ProjectType::new(
            "docker-compose",
            "Docker Compose stack (docker-compose.{yml,yaml} / compose.{yml,yaml})",
            vec![
                file("docker-compose.yml"),
                file("docker-compose.yaml"),
                file("compose.yml"),
                file("compose.yaml"),
            ],
            vec![],
        ),
        ProjectType::new(
            "swift",
            "Swift package (Package.swift) / CocoaPods (*.podspec) / Xcode (*.xcodeproj, *.xcworkspace)",
            vec![
                file("Package.swift"),
                glob("*.podspec"),
                subdir("*.xcodeproj"),
                subdir("*.xcworkspace"),
            ],
            vec![".build", ".swiftpm", "DerivedData"],
        ),
        ProjectType::new(
            "php",
            "PHP Composer project (composer.json)",
            vec![file("composer.json")],
            vec!["vendor"],
        ),
        ProjectType::new(
            "scala-sbt",
            "Scala sbt project (build.sbt)",
            vec![file("build.sbt")],
            vec!["target", ".bsp"],
        ),
        ProjectType::new(
            "scala-mill",
            "Scala Mill project (build.mill / build.sc)",
            vec![file("build.mill"), file("build.sc")],
            vec!["out"],
        ),
        ProjectType::new(
            "cmake",
            "C/C++ CMake project (CMakeLists.txt)",
            vec![file("CMakeLists.txt")],
            vec!["build", "cmake-build-debug", "cmake-build-release", "_build"],
        ),
        ProjectType::new(
            "autotools",
            "GNU Autotools project (configure.ac / configure.in / Makefile.am)",
            vec![file("configure.ac"), file("configure.in"), file("Makefile.am")],
            vec!["autom4te.cache"],
        ),
        ProjectType::new(
            "r",
            "R package / project (DESCRIPTION / *.Rproj)",
            vec![file("DESCRIPTION"), glob("*.Rproj")],
            vec![],
        ),
        ProjectType::new(
            "zig",
            "Zig project (build.zig / build.zig.zon)",
            vec![file("build.zig"), file("build.zig.zon")],
            vec!["zig-out", "zig-cache", ".zig-cache"],
        ),
        ProjectType::new(
            "perl",
            "Perl distribution (Makefile.PL / Build.PL / cpanfile / dist.ini)",
            vec![
                file("Makefile.PL"),
                file("Build.PL"),
                file("cpanfile"),
                file("dist.ini"),
            ],
            vec!["blib", "_build"],
        ),
        ProjectType::new(
            "matlab",
            "MATLAB project / toolbox (*.prj)",
            vec![glob("*.prj")],
            vec![],
        ),
        ProjectType::new(
            "hugo",
            "Hugo static site (hugo.{toml,yaml,yml})",
            vec![file("hugo.toml"), file("hugo.yaml"), file("hugo.yml")],
            vec!["public", "resources"],
        ),
        ProjectType::new(
            "jekyll",
            "Jekyll static site (_config.{yml,yaml})",
            vec![file("_config.yml"), file("_config.yaml")],
            vec!["_site", ".jekyll-cache", ".sass-cache"],
        ),
        ProjectType::new(
            "eleventy",
            "Eleventy static site (.eleventy.js / eleventy.config.*)",
            vec![
                file(".eleventy.js"),
                file("eleventy.config.js"),
                file("eleventy.config.cjs"),
                file("eleventy.config.mjs"),
                file("eleventy.config.ts"),
            ],
            vec!["_site"],
        ),
        ProjectType::new(
            "astro",
            "Astro static site (astro.config.{mjs,cjs,js,ts})",
            vec![
                file("astro.config.mjs"),
                file("astro.config.cjs"),
                file("astro.config.js"),
                file("astro.config.ts"),
            ],
            vec!["dist", ".astro"],
        ),
        ProjectType::new(
            "gatsby",
            "Gatsby static site (gatsby-config.{js,ts,mjs})",
            vec![
                file("gatsby-config.js"),
                file("gatsby-config.ts"),
                file("gatsby-config.mjs"),
            ],
            vec!["public", ".cache", ".gatsby"],
        ),
        ProjectType::new(
            "mkdocs",
            "MkDocs documentation site (mkdocs.{yml,yaml})",
            vec![file("mkdocs.yml"), file("mkdocs.yaml")],
            vec!["site"],
        ),
        ProjectType::new(
            "docusaurus",
            "Docusaurus documentation site (docusaurus.config.{js,ts,mjs})",
            vec![
                file("docusaurus.config.js"),
                file("docusaurus.config.ts"),
                file("docusaurus.config.mjs"),
            ],
            vec!["build", ".docusaurus"],
        ),
        ProjectType::new(
            "pelican",
            "Pelican static site (pelicanconf.py)",
            vec![file("pelicanconf.py")],
            vec!["output"],
        ),
    ]
}
