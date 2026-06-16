//! Print the detected project type(s) for each path argument.
//!
//! ```sh
//! cargo run --example detect -- /path/to/repo [more paths...]
//! ```

fn main() {
    let reg = projectdetect::Registry::with_builtins();
    for arg in std::env::args().skip(1) {
        let mut names: Vec<String> = reg.detect(&arg).into_iter().map(|m| m.r#type).collect();
        names.sort();
        println!("{arg}\t{names:?}");
    }
}
