# modelcli (npm)

This is the npm wrapper for the `modelcli` Rust binary. When you install via npm, this package downloads the precompiled binary for your platform and provides it as a command-line tool.

## Installation

```bash
npm install -g modelcli
```

## Usage

```bash
modelcli "Your prompt here"
modelcli --model openai/gpt-4o "Ask something"
modelcli --stream "Stream the response"
```

For full documentation, see the main [README.md](../README.md).

## How it works

1. On `npm install`, the `postinstall` script runs `install.js`
2. `install.js` detects your platform and downloads the corresponding precompiled binary
3. When you run `modelcli`, it executes the binary through the `bin.js` wrapper

## Building from source

If you want to build from source instead of downloading precompiled binaries:

```bash
cargo install --path ..
```
