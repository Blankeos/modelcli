# PLAN-2: Custom Providers ("Other")

## Goal

Allow users to add any OpenAI-compatible provider not listed in models.dev, via the `modelcli connect` command and a config file.

---

## Config File

**Location**: `~/.config/modelcli/config.jsonc` **or** `config.json` — but **not both**. If both exist, error out: `"Found both config.jsonc and config.json in ~/.config/modelcli/. Please keep only one."`

Prefer `.jsonc` when auto-scaffolding.

This is separate from the existing `~/.local/share/modelcli/config.json` (which stores `default_model`). The `.config/` path follows XDG conventions for user-editable configuration, while `.local/share/` is for app-managed data. Keep them separate.

**Schema**:

```jsonc
{
  "provider": {
    // key = provider ID (user-chosen, used in --model flag)
    "myprovider": {
      "name": "My AI Provider",       // display name in UI
      "baseURL": "https://api.myprovider.com/v1",  // OpenAI-compatible endpoint
      "models": {
        // key = model ID (used in --model flag as myprovider/my-model)
        "my-model": {
          "name": "My Model Display Name",  // optional display name
          "reasoning": false,               // optional, default false
          "context": 200000,                // optional context window
          "output": 65536                   // optional max output tokens
        }
      }
    }
  }
}
```

**Differences from the opencode example** (simplified for our use case):
- No `npm` field (we always use rig's OpenAI-compatible client)
- No `options.headers` for now (keep it simple, can add later)
- No `options.apiKey` / `{env:VAR}` syntax for now — API keys live in auth.json like all other providers
- Flat `baseURL` instead of nested `options.baseURL`
- Model metadata is minimal: just `name`, `reasoning`, `context`, `output`

---

## The "Other" Flow in `modelcli connect`

### Current flow (already exists)

```
modelcli connect
→ [if providers connected] Add new provider / Disconnect a provider
→ Select a provider: (list from models.dev)
→ Enter API key
→ Saved
```

### Updated flow

The provider select list gets **one extra item at the bottom**: `"Other (custom provider)"`.

```
◆ Select a provider:
│  OpenAI
│  Anthropic
│  Google
│  ...
│  ──────────────
│  ● Other (custom provider)
```

When user selects "Other":

```
◇ Enter a unique provider ID (e.g. "myprovider"):
│  myprovider
│
▲ Note: This only stores a credential for "myprovider".
│ You'll need to configure it in ~/.config/modelcli/config.jsonc
│ (see docs for format).
│
◇ Enter your API key:
│  sk-...
│
✓ Credential saved for "myprovider".
✓ Next step: add provider config to ~/.config/modelcli/config.jsonc
```

**Validation**:
- Provider ID must be non-empty, alphanumeric + hyphens only
- Provider ID must not collide with an existing models.dev provider ID
- API key must be non-empty

**After saving**:
1. Auto-scaffold `~/.config/modelcli/config.jsonc` if it doesn't exist (write a template with the new provider pre-filled as a commented-out example).
2. If the file already exists, don't touch it — just print instructions.
3. Always print: `"Next: edit ~/.config/modelcli/config.jsonc and configure the 'myprovider' provider (baseURL, models)."`

---

## Config Loading: `CustomProviders`

New struct in `storage.rs`:

```rust
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CustomConfig {
    #[serde(default)]
    pub provider: HashMap<String, CustomProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomProvider {
    pub name: String,
    #[serde(rename = "baseURL")]
    pub base_url: String,
    #[serde(default)]
    pub models: HashMap<String, CustomModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomModel {
    pub name: Option<String>,
    #[serde(default)]
    pub reasoning: bool,
    pub context: Option<u64>,
    pub output: Option<u64>,
}
```

**Loading logic**:
1. Check for both `~/.config/modelcli/config.jsonc` and `config.json`
2. If **both** exist → error: `"Found both config.jsonc and config.json in ~/.config/modelcli/. Please keep only one."`
3. If one exists → load it. Strip JSONC comments before parsing (always, regardless of extension).
4. If neither exists → return `CustomConfig::default()` (not an error)

**JSONC parsing**: Use a crate (e.g. `json_comments` or `serde_jsonc`) to handle comment stripping properly — avoids edge cases with `//` inside strings. Always strip comments regardless of file extension.

---

## Merging Custom Providers into the Pipeline

Custom providers need to appear alongside models.dev providers in two places:

### 1. `modelcli models`

After loading models.dev providers, also load `CustomConfig`. Convert each `CustomProvider` into a `models_dev::Provider` (or a unified type) so the rest of the code works uniformly.

**Conversion**:
```rust
impl CustomProvider -> Provider {
    id: key from config map
    name: custom_provider.name
    env: vec![]  // not used for custom
    api: Some(custom_provider.base_url)
    models: convert CustomModel -> Model for each
}
```

This way the existing `models.rs` command code doesn't need to know about custom vs. models.dev — it just sees a `ProvidersMap`.

### 2. `modelcli "<prompt>"` (prompt command)

When resolving `--model myprovider/my-model`:
1. Look up `myprovider` in models.dev providers first
2. If not found, look up in custom config
3. If found in custom config, convert to `Provider`/`Model` and proceed with `call_model()` as usual (it'll hit the OpenAI-compatible path since `api` is set)

---

## Affected Files

| File | Changes |
|------|---------|
| `src/storage.rs` | Add `CustomConfig`, `CustomProvider`, `CustomModel` structs + load/save |
| `src/commands/connect.rs` | Add "Other" option, custom provider ID + API key flow |
| `src/commands/models.rs` | Merge custom providers into model list |
| `src/commands/prompt.rs` | Fall back to custom providers when resolving model |
| `src/api/models_dev.rs` | Add `From<CustomProvider>` conversion (or a merge helper) |

---

## Edge Cases

- **Custom provider ID collides with models.dev ID**: Reject at connect time with a message like "Provider ID 'openai' is already a known provider. Choose a different ID."
- **Custom provider has no models in config yet**: Show the provider in `modelcli models` but with an empty list + a hint: "No models configured for 'myprovider'. Add models to ~/.config/modelcli/config.jsonc"
- **User disconnects a custom provider**: Same flow as regular disconnect (removes from auth.json). The config.jsonc entry stays — it's the user's file to manage.
- **Config file doesn't exist**: Not an error. Custom providers just don't show up.
- **Prompt references a custom provider not in config**: Error: "Provider 'myprovider' not found in models.dev or custom config."

---

## Resolved Decisions

1. **Auto-scaffold**: Yes. Create `~/.config/modelcli/config.jsonc` with a template when user first adds a custom provider. Pre-fill the provider they just added as a commented-out example. Print a message telling them to go edit it.

2. **Two config files**: Keep separate. `~/.local/share/modelcli/config.json` = app-managed (default_model). `~/.config/modelcli/config.jsonc` = user-edited (custom providers). Error if both `.jsonc` and `.json` exist in `~/.config/modelcli/`.

3. **JSONC**: Yes, required. Users want to comment and temporarily disable things. Always strip comments before parsing regardless of file extension. Use a crate for correctness.
