//! Waltzing Language Extension for Zed
//!
//! This extension provides Language Server Protocol (LSP) support for Waltzing
//! template files (.wtz) in the Zed editor.

use std::fs;
use zed_extension_api::{self as zed, settings::LspSettings, Command, LanguageServerId, Result, Worktree};

/// The Waltzing extension for Zed
struct WaltzingExtension {
    /// Cached path to the language server binary
    cached_binary_path: Option<String>,
}

impl WaltzingExtension {
    /// Try to find the waltzing-lsp binary in common locations
    fn find_lsp_binary(&self) -> Option<String> {
        // Check if there's a cached path that still exists
        if let Some(ref path) = self.cached_binary_path {
            if fs::metadata(path).is_ok() {
                return Some(path.clone());
            }
        }

        // Common locations to search for the binary
        let home = std::env::var("HOME").ok()?;
        let candidates = [
            // Cargo install location
            format!("{}/.cargo/bin/waltzing-lsp", home),
            // Local bin
            format!("{}/.local/bin/waltzing-lsp", home),
            // System-wide
            "/usr/local/bin/waltzing-lsp".to_string(),
            "/usr/bin/waltzing-lsp".to_string(),
            // macOS Homebrew
            "/opt/homebrew/bin/waltzing-lsp".to_string(),
        ];

        for candidate in candidates {
            if fs::metadata(&candidate).is_ok() {
                return Some(candidate);
            }
        }

        None
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
        // Check for user-configured binary path in settings
        let lsp_settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;

        let (binary_path, args) = if let Some(ref binary_settings) = lsp_settings.binary {
            let path = binary_settings.path.as_ref().map(|p| p.to_string());
            let args = binary_settings
                .arguments
                .as_ref()
                .map(|args| args.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default();
            (path, args)
        } else {
            (None, vec![])
        };

        // Use configured path, or try to find the binary
        let command = if let Some(path) = binary_path {
            path
        } else if let Some(path) = self.find_lsp_binary() {
            self.cached_binary_path = Some(path.clone());
            path
        } else {
            return Err(
                "Could not find waltzing-lsp binary. \
                 Please install it with `cargo install --path lsp` from the waltzing repository, \
                 or specify the path in your Zed settings:\n\n\
                 \"lsp\": {\n  \
                   \"waltzing-lsp\": {\n    \
                     \"binary\": {\n      \
                       \"path\": \"/path/to/waltzing-lsp\"\n    \
                     }\n  \
                   }\n\
                 }"
                .into(),
            );
        };

        Ok(Command {
            command,
            args,
            env: vec![],
        })
    }
}

zed::register_extension!(WaltzingExtension);
