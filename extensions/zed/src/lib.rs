//! Waltzing Language Extension for Zed
//!
//! This extension provides Language Server Protocol (LSP) support for Waltzing
//! template files (.wtz) in the Zed editor, plus a Model Context Protocol
//! (MCP) server that exposes Waltzing grammar/validation tools to Zed's
//! Agent Panel.
//!
//! Both binaries (`waltzing-lsp`, `waltzing-mcp`) resolve in the same order:
//!   1. User-configured path in Zed settings.
//!   2. Existing install on PATH (cargo/.local/system/homebrew).
//!   3. Auto-downloaded from the project's GitHub releases.

use std::fs;
use zed_extension_api::{
    self as zed, Architecture, Command, ContextServerId, DownloadedFileType,
    GithubReleaseOptions, LanguageServerId, LanguageServerInstallationStatus, Os, Project, Result,
    Worktree, current_platform, download_file, latest_github_release, make_file_executable,
    set_language_server_installation_status, settings::LspSettings,
};

/// GitHub repository hosting release binaries.
const REPO: &str = "awesomike/waltzing";

const LSP_BINARY: &str = "waltzing-lsp";
const MCP_BINARY: &str = "waltzing-mcp";

/// The Waltzing extension for Zed.
struct WaltzingExtension {
    cached_lsp_path: Option<String>,
    cached_mcp_path: Option<String>,
}

impl WaltzingExtension {
    /// Search common install locations for `binary_name`.
    fn find_binary_on_path(binary_name: &str, cached: &Option<String>) -> Option<String> {
        if let Some(path) = cached {
            if fs::metadata(path).is_ok() {
                return Some(path.clone());
            }
        }

        let home = std::env::var("HOME").ok()?;
        let candidates = [
            format!("{}/.cargo/bin/{}", home, binary_name),
            format!("{}/.local/bin/{}", home, binary_name),
            format!("/usr/local/bin/{}", binary_name),
            format!("/usr/bin/{}", binary_name),
            format!("/opt/homebrew/bin/{}", binary_name),
        ];

        candidates.into_iter().find(|p| fs::metadata(p).is_ok())
    }

    /// Asset name on a release that matches the current host.
    /// Matches the naming used by `scripts/build-release.sh`:
    ///   {binary}-{darwin|linux}-{aarch64|x86_64}
    fn asset_name_for_current_platform(binary_name: &str) -> Result<String> {
        let (os, arch) = current_platform();
        let os_str = match os {
            Os::Mac => "darwin",
            Os::Linux => "linux",
            Os::Windows => {
                return Err(format!(
                    "{} does not yet ship a Windows binary; install from source with \
                     `cargo install --path lsp` (or `--path mcp`) in the waltzing repository.",
                    binary_name,
                ));
            }
        };
        let arch_str = match arch {
            Architecture::Aarch64 => "aarch64",
            Architecture::X8664 => "x86_64",
            Architecture::X86 => {
                return Err(format!("32-bit x86 is not supported by {}.", binary_name));
            }
        };
        Ok(format!("{}-{}-{}", binary_name, os_str, arch_str))
    }

    /// Download the platform-appropriate binary from the latest GitHub release.
    /// `progress` is invoked with installation-status updates; for binaries that
    /// don't have a Zed status channel (e.g. context servers) it can be a no-op.
    fn download_binary(
        binary_name: &str,
        mut progress: impl FnMut(&LanguageServerInstallationStatus),
    ) -> Result<String> {
        let asset_name = Self::asset_name_for_current_platform(binary_name)?;

        progress(&LanguageServerInstallationStatus::CheckingForUpdate);

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

        let version_dir = format!("{}-{}", binary_name, release.version);
        let binary_path = format!("{}/{}", version_dir, binary_name);

        let already_present = fs::metadata(&binary_path)
            .map(|m| m.is_file())
            .unwrap_or(false);

        if !already_present {
            progress(&LanguageServerInstallationStatus::Downloading);

            fs::create_dir_all(&version_dir)
                .map_err(|e| format!("Failed to create directory '{}': {}", version_dir, e))?;

            download_file(
                &asset.download_url,
                &binary_path,
                DownloadedFileType::Uncompressed,
            )?;
            make_file_executable(&binary_path)?;

            // Clean up previously-downloaded versions of this binary.
            let prefix = format!("{}-", binary_name);
            if let Ok(entries) = fs::read_dir(".") {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.starts_with(&prefix) && name != version_dir {
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
            cached_lsp_path: None,
            cached_mcp_path: None,
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
        } else if let Some(path) =
            Self::find_binary_on_path(LSP_BINARY, &self.cached_lsp_path)
        {
            self.cached_lsp_path = Some(path.clone());
            path
        } else {
            let id = language_server_id.clone();
            let path = Self::download_binary(LSP_BINARY, |status| {
                set_language_server_installation_status(&id, status);
            })?;
            self.cached_lsp_path = Some(path.clone());
            path
        };

        Ok(Command {
            command,
            args,
            env: vec![],
        })
    }

    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &Project,
    ) -> Result<Command> {
        // MCP servers don't have a settings.binary surface like LSPs do, so we
        // only support the local-install fallback and the auto-download path.
        let command = if let Some(path) =
            Self::find_binary_on_path(MCP_BINARY, &self.cached_mcp_path)
        {
            self.cached_mcp_path = Some(path.clone());
            path
        } else {
            let path = Self::download_binary(MCP_BINARY, |_| {})?;
            self.cached_mcp_path = Some(path.clone());
            path
        };

        Ok(Command {
            command,
            args: vec![],
            env: vec![],
        })
    }
}

zed::register_extension!(WaltzingExtension);
