//! Build script that auto-discovers and compiles Waltzing template libraries.
//!
//! Libraries are discovered by scanning `libraries/` for directories containing
//! a `waltzing-*.toml` or `library.toml` manifest file.
//!
//! Uses the `waltzing` CLI binary (from PATH or ~/.local/bin) for compilation.

use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// Path separator differs between platforms
#[cfg(windows)]
const PATH_SEPARATOR: char = ';';
#[cfg(not(windows))]
const PATH_SEPARATOR: char = ':';

// Executable name differs on Windows
#[cfg(windows)]
const WALTZING_EXE: &str = "waltzing.exe";
#[cfg(not(windows))]
const WALTZING_EXE: &str = "waltzing";

#[derive(Debug, Deserialize)]
struct LibraryManifest {
    library: LibraryInfo,
    #[serde(default)]
    components: HashMap<String, ComponentEntry>,
    #[serde(default)]
    layouts: HashMap<String, ComponentEntry>,
    #[serde(default)]
    blocks: HashMap<String, ComponentEntry>,
}

#[derive(Debug, Deserialize)]
struct LibraryInfo {
    name: String,
    version: String,
    description: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ComponentEntry {
    path: String,
    #[serde(default)]
    deps: Vec<String>,
}

#[derive(Debug)]
struct DiscoveredLibrary {
    dir_name: String,
    manifest: LibraryManifest,
}

/// Find the waltzing binary in PATH or ~/.local/bin
fn find_waltzing_binary() -> Option<PathBuf> {
    // Check if waltzing is on the PATH
    let path_var = env::var("PATH").ok()?;
    let waltzing_path = path_var
        .split(PATH_SEPARATOR)
        .map(PathBuf::from)
        .find(|path| path.join(WALTZING_EXE).exists())
        .map(|path| path.join(WALTZING_EXE));

    if let Some(path) = waltzing_path {
        return Some(path);
    }

    // Fall back to ~/.local/bin/waltzing
    let home_dir = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .ok()?;

    let local_bin_path = PathBuf::from(home_dir)
        .join(".local")
        .join("bin")
        .join(WALTZING_EXE);

    if local_bin_path.exists() {
        Some(local_bin_path)
    } else {
        None
    }
}

fn discover_libraries() -> Vec<DiscoveredLibrary> {
    let libraries_dir = PathBuf::from("libraries");
    let mut libraries = Vec::new();

    if !libraries_dir.exists() {
        println!("cargo:warning=No libraries/ directory found");
        return libraries;
    }

    for entry in fs::read_dir(&libraries_dir).expect("Failed to read libraries/") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let dir_name = path.file_name().unwrap().to_string_lossy().to_string();

        // Look for library.toml or {dir_name}.toml
        let manifest_path = if path.join("library.toml").exists() {
            path.join("library.toml")
        } else if path.join(format!("{}.toml", dir_name)).exists() {
            path.join(format!("{}.toml", dir_name))
        } else {
            println!(
                "cargo:warning=Skipping {}: no library.toml or {}.toml found",
                dir_name, dir_name
            );
            continue;
        };

        let manifest_content =
            fs::read_to_string(&manifest_path).expect("Failed to read manifest");

        match toml::from_str::<LibraryManifest>(&manifest_content) {
            Ok(manifest) => {
                println!("cargo:rerun-if-changed={}", manifest_path.display());
                libraries.push(DiscoveredLibrary { dir_name, manifest });
            }
            Err(e) => {
                println!(
                    "cargo:warning=Failed to parse {}: {}",
                    manifest_path.display(),
                    e
                );
            }
        }
    }

    libraries
}

fn compile_library(library: &DiscoveredLibrary, waltzing_bin: &PathBuf, out_dir: &str) -> bool {
    let lib_dir = PathBuf::from("libraries").join(&library.dir_name);
    let lib_out_dir = format!("{}/{}", out_dir, library.dir_name.replace("-", "_"));

    println!(
        "cargo:warning=Compiling library: {} -> {}",
        library.manifest.library.name, lib_out_dir
    );

    // Create output directory
    fs::create_dir_all(&lib_out_dir).expect("Failed to create output directory");

    // Run waltzing compiler
    let output = Command::new(waltzing_bin)
        .args([
            "-i",
            lib_dir.to_str().unwrap(),
            "--with-axum",
            "-o",
            &lib_out_dir,
        ])
        .output()
        .expect("Failed to run waltzing compiler");

    let success = output.status.success();

    if !success {
        println!(
            "cargo:warning=Waltzing compiler failed for {} (some templates may have syntax errors)",
            library.manifest.library.name
        );
        // Print stderr for debugging
        let stderr = String::from_utf8_lossy(&output.stderr);
        for line in stderr.lines().take(10) {
            println!("cargo:warning=  {}", line);
        }
    }

    // Watch for changes in the library directory
    println!("cargo:rerun-if-changed={}", lib_dir.display());

    // Watch all .wtz files
    for entry in walkdir::WalkDir::new(&lib_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "wtz") {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    success
}

fn generate_libraries_module(libraries: &[DiscoveredLibrary], out_dir: &str) {
    let gen_dir = PathBuf::from("src/generated");
    fs::create_dir_all(&gen_dir).expect("Failed to create src/generated");

    let mut code = String::new();
    code.push_str("//! Auto-generated module containing discovered libraries.\n");
    code.push_str("//! DO NOT EDIT - regenerated by build.rs\n\n");
    code.push_str("#![allow(dead_code)]\n\n");

    code.push_str("use serde::Serialize;\n\n");

    code.push_str("#[derive(Debug, Clone, Serialize)]\n");
    code.push_str("pub struct Library {\n");
    code.push_str("    pub id: &'static str,\n");
    code.push_str("    pub name: &'static str,\n");
    code.push_str("    pub version: &'static str,\n");
    code.push_str("    pub description: &'static str,\n");
    code.push_str("    pub component_count: usize,\n");
    code.push_str("}\n\n");

    code.push_str("pub const LIBRARIES: &[Library] = &[\n");

    for lib in libraries {
        let component_count = lib.manifest.components.len() + lib.manifest.layouts.len();
        code.push_str(&format!(
            "    Library {{\n        id: \"{}\",\n        name: \"{}\",\n        version: \"{}\",\n        description: \"{}\",\n        component_count: {},\n    }},\n",
            lib.dir_name,
            lib.manifest.library.name,
            lib.manifest.library.version,
            lib.manifest.library.description.replace("\"", "\\\""),
            component_count,
        ));
    }

    code.push_str("];\n\n");

    // Include compiled templates from OUT_DIR
    code.push_str(&format!(
        "/// Path to compiled templates (set by build.rs)\n"
    ));
    code.push_str(&format!(
        "pub const TEMPLATES_DIR: &str = \"{}\";\n\n",
        out_dir.replace("\\", "\\\\")
    ));

    // Generate submodule info for each library
    for lib in libraries {
        let module_name = lib.dir_name.replace("-", "_");
        code.push_str(&format!("pub mod {};\n", module_name));
    }

    fs::write(gen_dir.join("mod.rs"), code).expect("Failed to write mod.rs");

    // Generate info module for each library
    for lib in libraries {
        let module_name = lib.dir_name.replace("-", "_");
        let module_path = gen_dir.join(format!("{}.rs", module_name));

        let mut components: Vec<&String> = lib.manifest.components.keys().collect();
        let mut layouts: Vec<&String> = lib.manifest.layouts.keys().collect();
        let mut blocks: Vec<&String> = lib.manifest.blocks.keys().collect();

        // Sort for consistent output
        components.sort();
        layouts.sort();
        blocks.sort();

        let mut lib_code = String::new();
        lib_code.push_str(&format!(
            "//! Generated module for {} library.\n",
            lib.manifest.library.name
        ));
        lib_code.push_str("//! DO NOT EDIT - regenerated by build.rs\n\n");
        lib_code.push_str("#![allow(dead_code)]\n\n");

        lib_code.push_str("/// List of components in this library\n");
        lib_code.push_str("pub const COMPONENTS: &[&str] = &[\n");
        for comp in &components {
            lib_code.push_str(&format!("    \"{}\",\n", comp));
        }
        lib_code.push_str("];\n\n");

        lib_code.push_str("/// List of layouts in this library\n");
        lib_code.push_str("pub const LAYOUTS: &[&str] = &[\n");
        for layout in &layouts {
            lib_code.push_str(&format!("    \"{}\",\n", layout));
        }
        lib_code.push_str("];\n\n");

        lib_code.push_str("/// List of blocks in this library\n");
        lib_code.push_str("pub const BLOCKS: &[&str] = &[\n");
        for block in &blocks {
            lib_code.push_str(&format!("    \"{}\",\n", block));
        }
        lib_code.push_str("];\n\n");

        // Include the compiled templates
        lib_code.push_str(&format!(
            "// Compiled templates are in: {}/{}\n",
            out_dir, module_name
        ));

        fs::write(module_path, lib_code).expect("Failed to write library module");
    }
}

fn main() {
    println!("cargo:rerun-if-changed=libraries");
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let templates_out = format!("{}/templates", out_dir);

    let libraries = discover_libraries();

    println!("cargo:warning=Discovered {} libraries", libraries.len());

    // Find waltzing binary
    match find_waltzing_binary() {
        Some(waltzing_bin) => {
            println!(
                "cargo:warning=Using waltzing at: {}",
                waltzing_bin.display()
            );

            let mut compiled = 0;
            let mut failed = 0;

            for library in &libraries {
                if compile_library(library, &waltzing_bin, &templates_out) {
                    compiled += 1;
                } else {
                    failed += 1;
                }
            }

            if failed > 0 {
                println!(
                    "cargo:warning=Compiled {}/{} libraries ({} had errors)",
                    compiled,
                    libraries.len(),
                    failed
                );
            }
        }
        None => {
            println!("cargo:warning=waltzing binary not found in PATH or ~/.local/bin");
            println!("cargo:warning=Skipping template compilation - install waltzing to enable");
        }
    }

    generate_libraries_module(&libraries, &templates_out);
}
