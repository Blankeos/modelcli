use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

fn data_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let dir = home.join(".local").join("share").join("modelcli");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
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
