# Installing Waltzing Extension for Zed

## Step 1: Install the Language Server

From the waltzing repository root:

```bash
cargo install --path lsp
```

This installs `waltzing-lsp` to `~/.cargo/bin/waltzing-lsp`.

## Step 2: Install the Extension in Zed

1. Open Zed
2. Press `Cmd+Shift+P` (macOS) or `Ctrl+Shift+P` (Linux)
3. Type "install dev extension" and select **zed: install dev extension**
4. Navigate to and select the `extensions/zed` directory

## Step 3: Verify Installation

1. Open any `.wtz` file
2. The language should show as "Waltzing" in the status bar
3. Syntax errors should appear as diagnostics

## Optional: Custom LSP Path

If the LSP binary is in a non-standard location, add to `~/.config/zed/settings.json`:

```json
{
  "lsp": {
    "waltzing-lsp": {
      "binary": {
        "path": "/custom/path/to/waltzing-lsp"
      }
    }
  }
}
```

## Troubleshooting

### LSP not starting

Check that the binary is installed:
```bash
which waltzing-lsp
```

### No syntax highlighting

Ensure the file has a `.wtz`, `.html.wtz`, `.css.wtz`, or `.js.wtz` extension.

### View LSP logs

In Zed: `Cmd+Shift+P` → "zed: open log"
