use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize)]
struct Manifest {
    components: BTreeMap<String, Entry>,
    layouts: BTreeMap<String, Entry>,
    #[serde(default)]
    blocks: BTreeMap<String, Entry>,
}

#[derive(Debug, Deserialize)]
struct Entry {
    path: String,
    #[serde(default)]
    deps: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Registry {
    version: String,
    components: Vec<RegistryEntry>,
    layouts: Vec<RegistryEntry>,
    #[serde(default)]
    blocks: Vec<RegistryEntry>,
}

#[derive(Debug, Deserialize)]
struct RegistryEntry {
    name: String,
    path: String,
    #[serde(default)]
    dependencies: Vec<String>,
}

fn library_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("libraries/waltzing-ui")
}

fn manifest() -> Manifest {
    let source = std::fs::read_to_string(library_dir().join("waltzing-ui.toml")).unwrap();
    toml::from_str(&source).unwrap()
}

fn registry() -> Registry {
    let source = std::fs::read_to_string(library_dir().join("registry.json")).unwrap();
    serde_json::from_str(&source).unwrap()
}

fn names(entries: &BTreeMap<String, Entry>) -> BTreeSet<String> {
    entries.keys().cloned().collect()
}

fn registry_names(entries: &[RegistryEntry]) -> BTreeSet<String> {
    entries.iter().map(|entry| entry.name.clone()).collect()
}

#[test]
fn manifest_paths_and_dependencies_are_valid() {
    let manifest = manifest();
    let root = library_dir();

    let mut known = BTreeSet::from(["lib/utils".to_string()]);
    known.extend(names(&manifest.components));
    known.extend(names(&manifest.layouts));
    known.extend(names(&manifest.blocks));

    for (kind, entries) in [
        ("component", &manifest.components),
        ("layout", &manifest.layouts),
        ("block", &manifest.blocks),
    ] {
        for (name, entry) in entries {
            assert!(
                root.join(&entry.path).exists(),
                "{kind} {name} points at missing path {}",
                entry.path
            );

            for dep in &entry.deps {
                assert!(
                    known.contains(dep) || dep.starts_with("lib/"),
                    "{kind} {name} depends on unknown entry {dep}"
                );
            }
        }
    }
}

#[test]
fn registry_matches_manifest_surface() {
    let manifest = manifest();
    let registry = registry();

    assert_eq!(registry.version, "0.2.0");
    assert_eq!(
        registry_names(&registry.components),
        names(&manifest.components)
    );
    assert_eq!(registry_names(&registry.layouts), names(&manifest.layouts));
    assert_eq!(registry_names(&registry.blocks), names(&manifest.blocks));

    for entry in registry
        .components
        .iter()
        .chain(registry.layouts.iter())
        .chain(registry.blocks.iter())
    {
        assert!(
            library_dir().join(&entry.path).exists(),
            "registry entry {} points at missing path {}",
            entry.name,
            entry.path
        );

        for dep in &entry.dependencies {
            assert!(
                manifest.components.contains_key(dep)
                    || manifest.layouts.contains_key(dep)
                    || manifest.blocks.contains_key(dep)
                    || dep.starts_with("lib/"),
                "registry entry {} depends on unknown entry {}",
                entry.name,
                dep
            );
        }
    }
}

#[test]
fn library_keeps_real_breadth() {
    let manifest = manifest();

    assert!(
        manifest.components.len() >= 50,
        "component count regressed to {}",
        manifest.components.len()
    );
    assert!(
        manifest.layouts.len() >= 3,
        "layout count regressed to {}",
        manifest.layouts.len()
    );
    assert!(
        manifest.blocks.len() >= 4,
        "block count regressed to {}",
        manifest.blocks.len()
    );
}

#[test]
fn waltzing_ui_compiles_as_importable_library() {
    let waltzing = std::env::var("WALTZING_BIN").unwrap_or_else(|_| "waltzing".to_string());
    let output_dir = std::env::temp_dir().join(format!("waltzing-ui-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&output_dir);

    let output = Command::new(&waltzing)
        .args([
            "-i",
            library_dir().to_str().unwrap(),
            "--with-axum",
            "-o",
            output_dir.to_str().unwrap(),
        ])
        .output()
        .unwrap_or_else(|err| panic!("failed to run {waltzing}: {err}"));

    assert!(
        output.status.success(),
        "waltzing-ui compile failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn accessibility_smoke_checks_cover_blocks_and_dialogs() {
    let root = library_dir();
    for block in [
        "blocks/auth/login-card.wtz",
        "blocks/auth/signup-card.wtz",
        "blocks/forms/contact-card.wtz",
    ] {
        let source = read(&root.join(block));
        assert!(source.contains("<form"), "{block} should render a form");
        assert!(source.contains("<label"), "{block} should label inputs");
        assert!(
            source.contains("role=\"alert\""),
            "{block} should expose validation errors"
        );
    }

    let stats = read(&root.join("blocks/dashboard/stats-grid.wtz"));
    assert!(
        stats.contains("aria-label=\"Dashboard statistics\""),
        "stats grid should name the statistics region"
    );

    for dialog in ["components/dialog.wtz", "components/dialogs/dialog.wtz"] {
        let source = read(&root.join(dialog));
        assert!(
            source.contains("role=\"dialog\"") || source.contains("role=\"alertdialog\""),
            "{dialog} should expose a dialog role"
        );
        assert!(
            source.contains("aria-modal=\"true\""),
            "{dialog} should mark modal surfaces"
        );
    }

    let sheet = read(&root.join("components/dialogs/sheet.wtz"));
    assert!(
        sheet.contains("role=\"complementary\"") && sheet.contains("aria-label=@title"),
        "non-modal sheets should expose a named complementary landmark"
    );
}

#[test]
fn parser_unsafe_quoted_interpolation_stays_out_of_templates() {
    for path in wtz_files(&library_dir()) {
        let source = read(&path);
        assert!(
            !source.contains("class=\"@"),
            "{} contains quoted class interpolation",
            path.display()
        );
        for line in source.lines() {
            if let Some(start) = line.find("x-data=\"{") {
                let value = &line[start..];
                assert!(
                    !value.contains('@'),
                    "{} likely embeds Waltzing interpolation in quoted Alpine state: {line}",
                    path.display()
                );
            }
        }
    }
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn wtz_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_wtz(root, &mut files);
    files
}

fn collect_wtz(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_wtz(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("wtz") {
            files.push(path);
        }
    }
}
