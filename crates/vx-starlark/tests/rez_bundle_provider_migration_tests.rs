//! Contract tests for the built-in provider migration to the Rez Next SDK.

use std::path::{Path, PathBuf};

fn find_provider_stars(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let entries = std::fs::read_dir(root).expect("provider directory should be readable");
    for entry in entries {
        let path = entry.expect("provider entry should be readable").path();
        if path.is_dir() {
            files.extend(find_provider_stars(&path));
        } else if path.file_name().is_some_and(|name| name == "provider.star") {
            files.push(path);
        }
    }
    files.sort();
    files
}

#[test]
fn every_builtin_provider_declares_a_rez_next_sdk_descriptor() {
    let providers_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate should live under crates")
        .join("vx-providers");
    let provider_stars = find_provider_stars(&providers_dir);

    assert!(
        !provider_stars.is_empty(),
        "built-in providers should exist"
    );

    for provider_star in provider_stars {
        let source =
            std::fs::read_to_string(&provider_star).expect("provider.star should be readable");
        let provider_name = provider_star
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        assert!(
            source.contains("rez_bundle_source"),
            "{provider_name} must declare the Rez Next SDK bundle source"
        );
        assert!(
            source.contains("rez_bundle ="),
            "{provider_name} must expose the Rez Next SDK activation hook"
        );
        assert!(
            source.contains("\"vx-org\"") && source.contains("\"vx-rez-packages\""),
            "{provider_name} must use the shared Rez bundle repository"
        );
    }
}
