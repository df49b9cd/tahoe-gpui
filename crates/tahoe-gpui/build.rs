//! Build script: validates icon asset paths at compile time.
//!
//! Checks that:
//! 1. Every path referenced in `names.rs` exists on disk.
//! 2. Every path registered in `ICON_ENTRIES` in `assets.rs` exists on disk.
//! 3. Every path referenced in `names.rs` is registered in `ICON_ENTRIES`.
//!
//! A typo in any path string will fail `cargo build` immediately instead of
//! being caught only by nextest.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

fn main() {
    let assets_dir = Path::new("assets");
    let names_rs = Path::new("src/foundations/icons/names.rs");
    let assets_rs = Path::new("src/foundations/icons/assets.rs");

    println!("cargo:rerun-if-changed=assets");
    println!("cargo:rerun-if-changed=src/foundations/icons/names.rs");
    println!("cargo:rerun-if-changed=src/foundations/icons/assets.rs");

    let on_disk = collect_svg_paths(assets_dir);
    let from_names = extract_paths_from_source(names_rs, "names.rs");
    let from_assets = extract_paths_from_source(assets_rs, "assets.rs");

    let mut errors = 0;

    // 1. names.rs paths must resolve to files on disk.
    for asset_path in &from_names {
        if !on_disk.contains(asset_path) {
            eprintln!(
                "error: icon path \"{asset_path}\" referenced in names.rs \
                 does not exist at assets/{asset_path}"
            );
            errors += 1;
        }
    }

    // 2. assets.rs ICON_ENTRIES paths must resolve to files on disk.
    for asset_path in &from_assets {
        if !on_disk.contains(asset_path) {
            eprintln!(
                "error: icon path \"{asset_path}\" registered in assets.rs \
                 does not exist at assets/{asset_path}"
            );
            errors += 1;
        }
    }

    // 3. names.rs paths must be registered in assets.rs ICON_ENTRIES.
    for asset_path in &from_names {
        if !from_assets.contains(asset_path) {
            eprintln!(
                "error: icon path \"{asset_path}\" referenced in names.rs \
                 but not registered in assets.rs ICON_ENTRIES"
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
            let entry_path = entry.path();
            if entry_path.is_dir() {
                collect_svg_paths_recursive(base, &entry_path, out);
            } else if entry_path.extension().is_some_and(|e| e == "svg")
                && let Ok(rel) = entry_path.strip_prefix(base)
            {
                out.insert(rel.to_string_lossy().into_owned());
            }
        }
    }
}

/// Extract all quoted strings matching `icons/...svg` from the source file.
fn extract_paths_from_source(file_path: &Path, label: &str) -> BTreeSet<String> {
    let source = fs::read_to_string(file_path).unwrap_or_else(|e| {
        eprintln!("error: cannot read {label} ({}): {e}", file_path.display());
        std::process::exit(1);
    });

    let mut paths = BTreeSet::new();
    let mut remaining = source.as_str();

    while let Some(start) = remaining.find("\"icons/") {
        let after_quote = start + 1; // skip opening quote
        remaining = &remaining[after_quote..];
        if let Some(end) = remaining.find(".svg\"") {
            let asset_path = &remaining[..end + 4]; // include ".svg"
            paths.insert(asset_path.to_owned());
            remaining = &remaining[end + 5..]; // skip past closing quote
        } else {
            break;
        }
    }

    paths
}
