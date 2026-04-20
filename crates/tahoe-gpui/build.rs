//! Build script: validates icon asset paths at compile time.
//!
//! Scans `names.rs` for string literals like `"icons/symbols/ant.svg"` and
//! checks each one exists under `assets/`. A typo in a path string will
//! fail `cargo build` immediately instead of being caught only by nextest.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

fn main() {
    let assets_dir = Path::new("assets");
    let names_rs = Path::new("src/foundations/icons/names.rs");

    println!("cargo:rerun-if-changed=assets");
    println!("cargo:rerun-if-changed=src/foundations/icons/names.rs");

    let on_disk = collect_svg_paths(assets_dir);
    let referenced = extract_paths_from_source(names_rs);

    let mut errors = 0;
    for path in &referenced {
        if !on_disk.contains(path) {
            eprintln!(
                "error: icon asset path \"{path}\" referenced in names.rs \
                 does not exist at assets/{path}"
            );
            errors += 1;
        }
    }

    if errors > 0 {
        std::process::exit(1);
    }
}

/// Recursively collect all `.svg` file paths relative to the assets root
/// (e.g., `"icons/symbols/ant.svg"`).
fn collect_svg_paths(dir: &Path) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    collect_svg_paths_recursive(dir, dir, &mut paths);
    paths
}

fn collect_svg_paths_recursive(base: &Path, current: &Path, out: &mut BTreeSet<String>) {
    if let Ok(entries) = fs::read_dir(current) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_svg_paths_recursive(base, &path, out);
            } else if path.extension().is_some_and(|e| e == "svg")
                && let Ok(rel) = path.strip_prefix(base)
            {
                out.insert(rel.to_string_lossy().into_owned());
            }
        }
    }
}

/// Extract all quoted strings matching `icons/...svg` from the source file.
fn extract_paths_from_source(path: &Path) -> BTreeSet<String> {
    let source = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("error: cannot read {}: {e}", path.display());
        std::process::exit(1);
    });

    let mut paths = BTreeSet::new();
    let mut remaining = source.as_str();

    while let Some(start) = remaining.find("\"icons/") {
        let after_quote = start + 1; // skip opening quote
        remaining = &remaining[after_quote..];
        if let Some(end) = remaining.find(".svg\"") {
            let path = &remaining[..end + 4]; // include ".svg"
            paths.insert(path.to_owned());
            remaining = &remaining[end + 5..]; // skip past closing quote
        } else {
            break;
        }
    }

    paths
}
