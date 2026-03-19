use anyhow::{Result, bail};

use crate::api::{call, models_dev};
use crate::storage::{AuthStore, Config};

pub async fn run(
    prompt_text: &str,
    model_flag: Option<&str>,
    stream: bool,
    show_thinking: bool,
    reasoning_effort: Option<&str>,
    format_json: bool,
) -> Result<()> {
    // Resolve model
    let model_str = if let Some(m) = model_flag {
        m.to_string()
    } else {
        let config = Config::load()?;
        match config.default_model {
            Some(m) => m,
            None => bail!(
                "No default model set. Run `modelcli connect` and `modelcli models` to get started."
            ),
        }
    };

    // Parse provider/model-id
    let (provider_id, model_id) = model_str
        .split_once('/')
        .ok_or_else(|| anyhow::anyhow!(
            "Invalid model format '{}'. Expected '<provider>/<model-id>'.",
            model_str
        ))?;

    // Load auth
    let auth = AuthStore::load()?;
    let api_key = auth
        .get(provider_id)
        .ok_or_else(|| anyhow::anyhow!(
            "Not connected to provider '{provider_id}'. Run `modelcli connect` first."
        ))?;

    // Load models.dev data
    let providers = models_dev::fetch_providers().await?;
    let provider = providers
        .get(provider_id)
        .ok_or_else(|| anyhow::anyhow!("Unknown provider '{provider_id}' in models.dev data."))?;

    let model_meta = provider
        .models
        .get(model_id)
        .ok_or_else(|| anyhow::anyhow!(
            "Model '{model_id}' not found for provider '{provider_id}'."
        ))?;

    call::call_model(
        provider,
        model_meta,
        api_key,
        prompt_text,
        stream,
        show_thinking,
        reasoning_effort,
        format_json,
    )
    .await
}
