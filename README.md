```
███╗░░░███╗░█████╗░██████╗░███████╗██╗░░░░░░█████╗░██╗░░░░░██╗
████╗░████║██╔══██╗██╔══██╗██╔════╝██║░░░░░██╔══██╗██║░░░░░██║
██╔████╔██║██║░░██║██║░░██║█████╗░░██║░░░░░██║░░╚═╝██║░░░░░██║
██║╚██╔╝██║██║░░██║██║░░██║██╔══╝░░██║░░░░░██║░░██╗██║░░░░░██║
██║░╚═╝░██║╚█████╔╝██████╔╝███████╗███████╗╚█████╔╝███████╗██║
╚═╝░░░░░╚═╝░╚════╝░╚═════╝░╚══════╝╚══════╝░╚════╝░╚══════╝╚═╝
```

# modelcli

Call any LLM from the command line via [models.dev](https://models.dev).

## Install

```sh
npm install -g modelcli # npm
bun install -g modelcli # or bun
cargo install --path . # or cargo
```

## Quick Start

```sh
# 1. Connect to a provider (Any known provider thanks to models.dev)
modelcli connect

# 2. Browse models and set a default
modelcli models

# 3. Send a prompt
modelcli "What is the meaning of life?"
```

## Usage

```
modelcli [OPTIONS] [PROMPT]
```

### Commands

| Command   | Description                         |
| --------- | ----------------------------------- |
| `connect` | Connect to a provider (add API key) |
| `models`  | Browse and manage models            |

### Options

| Flag                          | Description                                    |
| ----------------------------- | ---------------------------------------------- |
| `--model <provider/model-id>` | Model to use (overrides default)               |
| `--stream`                    | Stream tokens as they arrive                   |
| `--thinking`                  | Show thinking/reasoning tokens                 |
| `--reasoning-effort <level>`  | Reasoning effort: `low`, `medium`, or `high`   |
| `--format json`               | Output raw JSON instead of human-readable text |

### Examples

```sh
# Use a specific model
modelcli --model openai/gpt-4o "Explain quicksort"

# Stream the response
modelcli --stream "Write a haiku about Rust"

# Enable reasoning
modelcli --thinking --reasoning-effort high "Prove that √2 is irrational"

# JSON output
modelcli --format json "Hello"
```

## Custom Providers

You can add any OpenAI-compatible provider not listed on models.dev.

**1. Add a credential:**

```sh
modelcli connect
# Select "Other (custom provider)" → enter a provider ID and API key
```

**2. Configure the provider** in `~/.config/modelcli.jsonc`:

```jsonc
{
  "provider": {
    "myprovider": {
      "name": "My AI Provider",
      "baseURL": "https://api.myprovider.com/v1",
      "models": {
        "my-model": {
          "name": "My Model", // optional display name
          "reasoning": false, // optional, default false
          "context": 200000, // optional context window
          "output": 65536, // optional max output tokens
        },
      },
    },
  },
}
```

Then use it like any other model:

```sh
modelcli --model myprovider/my-model "Hello!"
```

> The config file is auto-created the first time you add a custom provider. Both `.jsonc` and `.json` are supported, but not both at the same time.

## Data Storage

- Credentials and app data: `~/.local/share/modelcli/`
- Custom provider config: `~/.config/modelcli.jsonc`

## Motivation

`modelcli` enables piping LLM calls directly from your terminal—perfect for generating commit messages in [lazygit](https://github.com/jesseduffield/lazygit) (see [PR #5389](https://github.com/jesseduffield/lazygit/pull/5389)), or powering any other CLI app with AI capabilities. Quickly ask questions or pipe stdout from other tools to get instant AI-powered responses.

Inspired by [OpenCode's](https://opencode.xyz) seamless multi-provider experience and built on [models.dev](https://models.dev)'s unified LLM API.

🦀 **Made w/ Rust.** A fast, minimal but intuitive CLI made with Rust.
