use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

fn data_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let dir = home.join(".local").join("share").join("modelcli");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn custom_config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".config"))
}

// ── Auth (API keys per provider) ──

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AuthStore(pub HashMap<String, String>);

impl AuthStore {
    pub fn load() -> Result<Self> {
        let path = data_dir()?.join("auth.json");
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn save(&self) -> Result<()> {
        let path = data_dir()?.join("auth.json");
        let data = serde_json::to_string_pretty(&self.0)?;
        std::fs::write(&path, data)?;
        Ok(())
    }

    pub fn set(&mut self, provider: &str, key: &str) {
        self.0.insert(provider.to_string(), key.to_string());
    }

    pub fn remove(&mut self, provider: &str) {
        self.0.remove(provider);
    }

    pub fn get(&self, provider: &str) -> Option<&String> {
        self.0.get(provider)
    }

    pub fn connected_providers(&self) -> Vec<&String> {
        self.0.keys().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// ── Config (default model, etc.) ──

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = data_dir()?.join("config.json");
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn save(&self) -> Result<()> {
        let path = data_dir()?.join("config.json");
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, data)?;
        Ok(())
    }
}

// ── Cache (models.dev data) ──

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedModelsData {
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    pub data: serde_json::Value,
}

impl CachedModelsData {
    pub fn load() -> Result<Option<Self>> {
        let path = data_dir()?.join("models-dev.json");
        if !path.exists() {
            return Ok(None);
        }
        let data = std::fs::read_to_string(&path)?;
        let cached: Self = match serde_json::from_str(&data) {
            Ok(c) => c,
            Err(_) => return Ok(None), // invalid cache, refetch
        };

        // 24h TTL
        let age = chrono::Utc::now() - cached.fetched_at;
        if age.num_hours() >= 24 {
            return Ok(None);
        }
        Ok(Some(cached))
    }

    pub fn save(data: &serde_json::Value) -> Result<()> {
        let path = data_dir()?.join("models-dev.json");
        let cached = Self {
            fetched_at: chrono::Utc::now(),
            data: data.clone(),
        };
        let json = serde_json::to_string(&cached)?;
        std::fs::write(&path, json)?;
        Ok(())
    }
}

// ── Custom provider config (~/.config/modelcli/config.jsonc) ──

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

impl CustomConfig {
    /// Load custom config from ~/.config/modelcli/config.jsonc (or .json).
    /// Errors if both files exist. Returns default if neither exists.
    pub fn load() -> Result<Self> {
        let dir = custom_config_dir()?;
        let jsonc_path = dir.join("modelcli.jsonc");
        let json_path = dir.join("modelcli.json");
        let jsonc_exists = jsonc_path.exists();
        let json_exists = json_path.exists();

        if jsonc_exists && json_exists {
            bail!(
                "Found both ~/.config/modelcli.jsonc and ~/.config/modelcli.json. Please keep only one."
            );
        }

        let path = if jsonc_exists {
            jsonc_path
        } else if json_exists {
            json_path
        } else {
            return Ok(Self::default());
        };

        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        // Strip JSONC comments before parsing
        let stripped = json_comments::StripComments::new(raw.as_bytes());
        let config: CustomConfig = serde_json::from_reader(stripped)
            .with_context(|| format!("Failed to parse {}", path.display()))?;

        Ok(config)
    }

    /// Returns the path to the config file (prefers .jsonc).
    pub fn config_path() -> Result<PathBuf> {
        let dir = custom_config_dir()?;
        Ok(dir.join("modelcli.jsonc"))
    }

    /// Auto-scaffold ~/.config/modelcli.jsonc with a template for the given provider ID.
    pub fn scaffold(provider_id: &str) -> Result<()> {
        let dir = custom_config_dir()?;
        let path = dir.join("modelcli.jsonc");

        if path.exists() {
            return Ok(());
        }

        std::fs::create_dir_all(&dir)?;

        let template = format!(
            r#"{{
  "provider": {{
    // "{provider_id}": {{
    //   "name": "My Provider Display Name",
    //   "baseURL": "https://api.example.com/v1",
    //   "models": {{
    //     "model-id": {{
    //       "name": "Model Display Name",
    //       "reasoning": false,
    //       "context": 200000,
    //       "output": 65536
    //     }}
    //   }}
    // }}
  }}
}}"#
        );

        std::fs::write(&path, template)?;
        Ok(())
    }
}
