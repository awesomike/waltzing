//! Waltzing Language Extension for Zed
//!
//! This extension provides Language Server Protocol (LSP) support for Waltzing
//! template files (.wtz) in the Zed editor.
//!
//! Resolution order for the `waltzing-lsp` binary:
//!   1. User-configured path in Zed settings.
//!   2. Existing install on PATH (cargo/.local/system/homebrew).
//!   3. Auto-downloaded from the project's GitHub releases.

use std::fs;
use zed_extension_api::{
    self as zed, Architecture, Command, DownloadedFileType, GithubReleaseOptions,
    LanguageServerId, LanguageServerInstallationStatus, Os, Result, Worktree, current_platform,
    download_file, latest_github_release, make_file_executable,
    set_language_server_installation_status, settings::LspSettings,
};

/// GitHub repository hosting `waltzing-lsp` release binaries.
const REPO: &str = "awesomike/waltzing";

/// Bare name of the LSP binary on disk.
const BINARY_NAME: &str = "waltzing-lsp";

/// The Waltzing extension for Zed
struct WaltzingExtension {
    /// Cached path to the language server binary
    cached_binary_path: Option<String>,
}

impl WaltzingExtension {
    /// Try to find a pre-existing `waltzing-lsp` binary on disk
    /// (cargo install / local bin / system / Homebrew).
    fn find_lsp_binary_on_path(&self) -> Option<String> {
        if let Some(ref path) = self.cached_binary_path {
            if fs::metadata(path).is_ok() {
                return Some(path.clone());
            }
        }

        let home = std::env::var("HOME").ok()?;
        let candidates = [
            format!("{}/.cargo/bin/{}", home, BINARY_NAME),
            format!("{}/.local/bin/{}", home, BINARY_NAME),
            format!("/usr/local/bin/{}", BINARY_NAME),
            format!("/usr/bin/{}", BINARY_NAME),
            format!("/opt/homebrew/bin/{}", BINARY_NAME),
        ];

        candidates.into_iter().find(|p| fs::metadata(p).is_ok())
    }

    /// Asset name on a release that matches the current host.
    /// Matches the naming used by `scripts/build-release.sh`:
    ///   waltzing-lsp-{darwin|linux}-{aarch64|x86_64}
    fn asset_name_for_current_platform() -> Result<String> {
        let (os, arch) = current_platform();
        let os_str = match os {
            Os::Mac => "darwin",
            Os::Linux => "linux",
            Os::Windows => {
                return Err("waltzing-lsp does not yet ship a Windows binary; \
                    install from source with `cargo install --path lsp` \
                    in the waltzing repository."
                    .into());
            }
        };
        let arch_str = match arch {
            Architecture::Aarch64 => "aarch64",
            Architecture::X8664 => "x86_64",
            Architecture::X86 => {
                return Err("32-bit x86 is not supported by waltzing-lsp.".into());
            }
        };
        Ok(format!("{}-{}-{}", BINARY_NAME, os_str, arch_str))
    }

    /// Download the platform-appropriate `waltzing-lsp` from the latest GitHub
    /// release and return the path to the binary in the extension work dir.
    fn download_lsp(&mut self, language_server_id: &LanguageServerId) -> Result<String> {
        let asset_name = Self::asset_name_for_current_platform()?;

        set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = latest_github_release(
            REPO,
            GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| {
                format!(
                    "No asset named '{}' in waltzing release {}",
                    asset_name, release.version
                )
            })?;

        let version_dir = format!("{}-{}", BINARY_NAME, release.version);
        let binary_path = format!("{}/{}", version_dir, BINARY_NAME);

        let already_present = fs::metadata(&binary_path)
            .map(|m| m.is_file())
            .unwrap_or(false);

        if !already_present {
            set_language_server_installation_status(
                language_server_id,
                &LanguageServerInstallationStatus::Downloading,
            );

            fs::create_dir_all(&version_dir)
                .map_err(|e| format!("Failed to create directory '{}': {}", version_dir, e))?;

            download_file(
                &asset.download_url,
                &binary_path,
                DownloadedFileType::Uncompressed,
            )?;
            make_file_executable(&binary_path)?;

            // Clean up previously-downloaded versions to avoid disk creep.
            if let Ok(entries) = fs::read_dir(".") {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        let is_old_install =
                            name.starts_with(&format!("{}-", BINARY_NAME)) && name != version_dir;
                        if is_old_install {
                            let _ = fs::remove_dir_all(entry.path());
                        }
                    }
                }
            }
        }

        Ok(binary_path)
    }
}

impl zed::Extension for WaltzingExtension {
    fn new() -> Self {
        WaltzingExtension {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        let lsp_settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;

        let (settings_binary_path, args) = if let Some(ref binary_settings) = lsp_settings.binary {
            let path = binary_settings.path.as_ref().map(|p| p.to_string());
            let args = binary_settings
                .arguments
                .as_ref()
                .map(|a| a.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default();
            (path, args)
        } else {
            (None, vec![])
        };

        let command = if let Some(path) = settings_binary_path {
            path
        } else if let Some(path) = self.find_lsp_binary_on_path() {
            self.cached_binary_path = Some(path.clone());
            path
        } else {
            let path = self.download_lsp(language_server_id)?;
            self.cached_binary_path = Some(path.clone());
            path
        };

        Ok(Command {
            command,
            args,
            env: vec![],
        })
    }
}

zed::register_extension!(WaltzingExtension);
