use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::storage::CachedModelsData;

const MODELS_DEV_URL: &str = "https://models.dev/api.json";

/// Top-level: provider_id -> Provider
pub type ProvidersMap = HashMap<String, Provider>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub env: Vec<String>,
    /// OpenAI-compatible base URL (if present)
    pub api: Option<String>,
    #[serde(default)]
    pub models: HashMap<String, Model>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub reasoning: bool,
    pub interleaved: Option<Interleaved>,
    pub temperature: Option<bool>,
    #[serde(default)]
    pub modalities: Option<Modalities>,
    pub status: Option<String>,
    #[serde(default)]
    pub tool_call: bool,
    #[serde(default)]
    pub structured_output: bool,
    pub cost: Option<Cost>,
    pub limit: Option<Limits>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Interleaved {
    Bool(bool),
    Config { field: String },
}

impl Interleaved {
    pub fn field_name(&self) -> Option<&str> {
        match self {
            Interleaved::Config { field } => Some(field.as_str()),
            Interleaved::Bool(true) => Some("reasoning_content"),
            Interleaved::Bool(false) => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modalities {
    #[serde(default)]
    pub input: Vec<String>,
    #[serde(default)]
    pub output: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cost {
    pub input: Option<f64>,
    pub output: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limits {
    pub context: Option<u64>,
    pub output: Option<u64>,
}

impl Model {
    /// Whether this model supports text output
    pub fn is_text_output(&self) -> bool {
        match &self.modalities {
            Some(m) => m.output.contains(&"text".to_string()),
            None => true, // assume text if not specified
        }
    }

    pub fn is_deprecated(&self) -> bool {
        self.status.as_deref() == Some("deprecated")
    }
}

impl Provider {
    /// Returns models filtered to text-output, non-deprecated
    pub fn text_models(&self) -> Vec<(&String, &Model)> {
        self.models
            .iter()
            .filter(|(_, m)| m.is_text_output() && !m.is_deprecated())
            .collect()
    }

    pub fn has_text_models(&self) -> bool {
        self.models
            .values()
            .any(|m| m.is_text_output() && !m.is_deprecated())
    }
}

/// Popular providers to show first (in order)
const POPULAR_PROVIDERS: &[&str] = &[
    "openai",
    "anthropic",
    "google",
    "ollama-cloud",
    "opencode",
    "opencode-go",
    "zai",
    "minimax",
    "github-copilot",
];

/// Fetch providers from models.dev (with caching)
pub async fn fetch_providers() -> Result<ProvidersMap> {
    // Try cache first
    if let Some(cached) = CachedModelsData::load()? {
        let providers: ProvidersMap = serde_json::from_value(cached.data)
            .context("Failed to parse cached models.dev data")?;
        return Ok(providers);
    }

    // Fetch fresh
    eprintln!("Fetching models from models.dev...");
    let resp = reqwest::get(MODELS_DEV_URL)
        .await
        .context("Failed to fetch models.dev")?;
    let data: serde_json::Value = resp
        .json()
        .await
        .context("Failed to parse models.dev JSON")?;

    // Cache it
    CachedModelsData::save(&data)?;

    let providers: ProvidersMap =
        serde_json::from_value(data).context("Failed to parse models.dev data")?;
    Ok(providers)
}

/// Sort providers: popular first, then alphabetical
pub fn sorted_provider_ids(providers: &ProvidersMap) -> Vec<String> {
    let mut popular: Vec<String> = POPULAR_PROVIDERS
        .iter()
        .filter(|id| providers.contains_key(**id))
        .map(|id| id.to_string())
        .collect();

    let mut rest: Vec<String> = providers
        .keys()
        .filter(|id| !POPULAR_PROVIDERS.contains(&id.as_str()))
        .cloned()
        .collect();
    rest.sort();

    popular.extend(rest);
    popular
}

/// Filter to only providers that have text-output models
pub fn text_providers(providers: &ProvidersMap) -> ProvidersMap {
    providers
        .iter()
        .filter(|(_, p)| p.has_text_models())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// Convert a CustomProvider into a models_dev Provider for uniform handling.
pub fn provider_from_custom(
    id: &str,
    custom: &crate::storage::CustomProvider,
) -> Provider {
    let models: HashMap<String, Model> = custom
        .models
        .iter()
        .map(|(model_id, cm)| {
            let model = Model {
                id: model_id.clone(),
                name: cm.name.clone().unwrap_or_else(|| model_id.clone()),
                reasoning: cm.reasoning,
                interleaved: None,
                temperature: None,
                modalities: Some(Modalities {
                    input: vec!["text".to_string()],
                    output: vec!["text".to_string()],
                }),
                status: None,
                tool_call: false,
                structured_output: false,
                cost: None,
                limit: match (cm.context, cm.output) {
                    (None, None) => None,
                    _ => Some(Limits {
                        context: cm.context,
                        output: cm.output,
                    }),
                },
            };
            (model_id.clone(), model)
        })
        .collect();

    Provider {
        id: id.to_string(),
        name: custom.name.clone(),
        env: vec![],
        api: Some(custom.base_url.clone()),
        models,
    }
}
