[[Prompt]]

Create a rust cli powered by models.dev.

The only purpose is to connect to any model via https://models.dev/api.json and call a 1-time api call (no chatting).

## There will be three commands

- modelcli "Hello World!" --stream --model ollama-cloud/kimi-k2.5
  - args:
    - --stream (no value, just boolean) is optional (when not supplied it generates). Output format is a 'generate' operation by default meaning dont stream the tokens.
    - --format (json | default) - by default, it uses default. Default means it's spitting out the tokens as readable text by the humans
    - --thinking - whether to show thinking tokens or not
    - --reasoning-effort - just whatever is usually the default. Like what do most sdks use cuz idk.
    - --model (string) is optional (it uses the default model set by the user via `modelcli models`, if none, ask the user to 'set a default model w/ `modelcli connect` if you don't want to specify with --models' ). It uses the combination that models.dev uses which is usually `<provider>/<model-id>` like ollama-cloud/kimi-k2.5 -> provider is ollama-cloud and kimi-k2.5 is the model

- modelcli connect
  - Very similar experience to https://github.com/bombshell-dev/clack/tree/main/packages/prompts
  - Opens a convenient UI that lists down providers based on the models.dev/api.json
  - Very similar experience of 'Select a provider'
  - Searching a provider
  - Pressing enter
  - Then get prompted to enter an API Key
  - Note: Make sure to filter the models.dev results for text outputs only.

- modelcli models
  - It prints out a similar experience to 'connect' command, but lists down the models based on the providers selected.
  - You can also search.
  - There should be options on whether to 'copy' (copies the model id) or 'set as default' a model

## faqs

- can you connect to multiple providers at the same time? yes you can connect to multiple providers that also affects what's shown in modelcli models.
- does modelcli connect flow validate with a test call? no just save blindly.

## empty states

- `modelcli models` — if no providers connected, error: "No providers connected. Run `modelcli connect` first."
- `modelcli "<prompt>"` — if no default model and no `--model` flag, error: "No default model set. Run `modelcli connect` and `modelcli models` to get started."
- `modelcli connect` — if no text-output models available (shouldn't happen), error and exit.

## disconnect flow

- When running `modelcli connect` and there are already connected providers, show an option: `[a] Add new provider` | `[d] Disconnect a provider`
- On disconnect: show list of connected providers (single select), confirm "Are you sure you want to disconnect {provider}? [y/N]", save and exit.

## provider ordering

- Sort alphabetically, but hardcode popular providers at top: OpenAI, Anthropic, Ollama, OpenCode Zen, OpenCode Go

## search

- In both `connect` and `models`, you can search by typing to filter the list in real-time.

## persistence

- like ~/.local/share/modelcli/auth.json - for api keys
- and ~/.local/share/modelcli/models-dev.json - for the cache
- Not sure where to store the default model though but yeah, you decide.

-

## Bonus points:

1. Prepare a short and sweet readme with a license and authored by Blankeos https://github.com/blankeos/modelcli

[[AI can write implementation plan below]]

---

## Implementation Plan

### Tech Stack

- **CLI framework**: `clap` (argument parsing)
- **TUI/interactive UI**: `ratatui` (simple single-select with search-as-you-type)
- **HTTP client**: `reqwest` with `tokio` async runtime
- **Serialization**: `serde` + `serde_json`
- **Clipboard**: `arboard` (for `modelcli models` copy action)
- **Streaming**: `reqwest` with streaming response body + `eventsource-stream` for SSE
- **Plain text**: streaming output to stdout is fine (no fancy formatting needed)

---

### models.dev API Schema (relevant fields)

From `https://models.dev/api.json`:

- Top-level: flat object keyed by **provider ID** → provider object
- Each provider: `{ id, name, env: [API_KEY_ENV_VAR], api?: baseUrl, models: { [modelId]: model } }`
  - `api` is only present on OpenAI-compatible providers (use this as base URL for those)
  - `env[0]` is the env var name for the API key (e.g. `ANTHROPIC_API_KEY`)
- Each model:
  - `reasoning: boolean` — whether it's a thinking/reasoning model
  - `interleaved: { field: string } | true | undefined` — if present, reasoning tokens are in this response field (e.g. `reasoning_content`, `reasoning_details`). Only present on reasoning models.
  - `temperature: boolean` — if false, do NOT send temperature param
  - `tool_call: boolean`, `structured_output: boolean`
  - `modalities.output: string[]` — filter for `"text"` to exclude image/audio-only models
  - `cost.reasoning: number` — some models bill reasoning tokens separately
  - `status?: "deprecated" | "beta"` — skip deprecated models

**Text-only filter**: `modalities.output.includes("text")` — applied in both `connect` and `models` commands.

---

### Flag Behavior Based on API Schema

- **`--thinking`**:
  - Only meaningful if the selected model has `interleaved` field
  - When enabled: read the field named by `interleaved.field` (e.g. `reasoning_content`) from the response and print it before the main output
  - If model has no `interleaved`: silently ignore the flag

- **`--reasoning-effort`**:
  - Not tracked in models.dev — it's an OpenAI `o`-series specific API param
  - Only pass through to the API request if the user explicitly provides it
  - Silently omit for all other providers/models

- **`--stream`**:
  - Not in models.dev schema — always a CLI-level option
  - When true: use SSE/streaming endpoint and print tokens as they arrive
  - When false (default): wait for full response and print at once

- **Temperature**:
  - If model has `temperature: false`, do not include temperature in the API request body

---

### Persistence

- `~/.local/share/modelcli/auth.json` — API keys keyed by provider ID
  ```json
  { "anthropic": "sk-...", "openai": "sk-..." }
  ```
- `~/.local/share/modelcli/models-dev.json` — cached response from models.dev, with a timestamp for cache invalidation (e.g. 24h TTL)
- `~/.local/share/modelcli/config.json` — default model and other user preferences
  ```json
  { "default_model": "anthropic/claude-sonnet-4-6" }
  ```

---

### Command Breakdown

#### `modelcli "<prompt>"` (default command)

1. Parse `--model` (or load `default_model` from config.json; if neither, print error and exit: "No default model set. Run `modelcli connect` and `modelcli models` to get started.")
2. Parse provider/model-id from `<provider>/<model-id>` format
3. Look up provider in cached models.dev data → get `api` base URL (if OpenAI-compatible) and model metadata
4. Look up API key from `auth.json` for that provider; if missing, error: "Not connected to provider {provider}. Run `modelcli connect` first."
5. Build request body based on model capabilities:
   - Include temperature only if `model.temperature === true`
   - Include reasoning_effort only if `--reasoning-effort` flag was explicitly set
6. Call the API (stream or generate based on `--stream`)
7. Print response. If `--thinking` and `model.interleaved` exists, print thinking tokens first
8. If `--format json`: print raw JSON response

#### `modelcli connect`

1. Fetch (or use cached) models.dev data
2. Filter providers to those with at least one text-output model
3. If already connected providers exist: show choice `[a] Add new provider` | `[d] Disconnect a provider`
   - Disconnect: show single-select list of connected providers, confirm with "[y/N]", remove from auth.json
4. Add flow: show interactive list of providers (ordered: popular first then alphabetical), with search-as-you-type
5. On select: prompt for API key input (masked input)
6. Save to `auth.json` under provider ID
7. Re-runnable — each run adds/updates one provider

#### `modelcli models`

1. Load `auth.json` — get list of connected provider IDs
2. Filter models.dev data: only providers the user has connected, only text-output models, exclude deprecated
3. Show interactive list with search
4. On select: show two options — `[c] Copy model ID` | `[s] Set as default`
   - Copy: write `<provider>/<model-id>` to clipboard
   - Set default: write to `config.json`

---

### Project Structure

```
src/
  main.rs           # CLI entrypoint, clap setup
  commands/
    prompt.rs       # default modelcli "<prompt>" command
    connect.rs      # modelcli connect
    models.rs       # modelcli models
  api/
    models_dev.rs   # fetch + cache models.dev/api.json
    call.rs         # make the actual LLM API call
  storage.rs        # read/write auth.json, config.json, cache
  ui.rs             # shared TUI select+search component
```
