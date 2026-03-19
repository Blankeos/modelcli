```
                       _         _           _
                      ( )       (_ )        (_ )  _
  ___ ___     _      _| |   __   | |    ___  | | (_)
/' _ ` _ `\ /'_`\  /'_` | /'__`\ | |  /'___) | | | |
| ( ) ( ) |( (_) )( (_| |(  ___/ | | ( (___  | | | |
(_) (_) (_)`\___/'`\__,_)`\____)(___)`\____)(___)(_)
```

# modelcli

Call any LLM from the command line via [models.dev](https://models.dev).

## Install

```sh
cargo install --path .
```

## Quick Start

```sh
# 1. Connect to a provider
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

| Command   | Description                          |
|-----------|--------------------------------------|
| `connect` | Connect to a provider (add API key)  |
| `models`  | Browse and manage models             |

### Options

| Flag                          | Description                                    |
|-------------------------------|------------------------------------------------|
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

## Data Storage

Config and credentials are stored in `~/.local/share/modelcli/`.
