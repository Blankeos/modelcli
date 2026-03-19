use anyhow::{Result, bail};
use inquire::Select;

use crate::api::models_dev;
use crate::storage::{AuthStore, Config};

pub async fn run() -> Result<()> {
    let auth = AuthStore::load()?;

    if auth.is_empty() {
        bail!("No providers connected. Run `modelcli connect` first.");
    }

    let all_providers = models_dev::fetch_providers().await?;

    // Collect models from connected providers only
    let mut model_entries: Vec<(String, String, String)> = Vec::new(); // (display, provider/model, provider_id)

    let connected: Vec<String> = auth.connected_providers().iter().map(|s| s.to_string()).collect();

    for provider_id in &connected {
        if let Some(provider) = all_providers.get(provider_id) {
            let mut text_models: Vec<_> = provider.text_models();
            text_models.sort_by(|(_, a), (_, b)| a.name.cmp(&b.name));

            for (model_id, model) in text_models {
                let full_id = format!("{provider_id}/{model_id}");
                let reasoning_tag = if model.reasoning { " [reasoning]" } else { "" };
                let display = format!("{} ({}){reasoning_tag}", model.name, full_id);
                model_entries.push((display, full_id, provider_id.clone()));
            }
        }
    }

    if model_entries.is_empty() {
        bail!("No text-output models found for your connected providers.");
    }

    let display_list: Vec<String> = model_entries.iter().map(|(d, _, _)| d.clone()).collect();

    let selection = Select::new("Select a model:", display_list.clone())
        .with_page_size(15)
        .prompt();

    let selected_idx = match selection {
        Ok(ref selected) => display_list.iter().position(|d| d == selected).unwrap(),
        Err(_) => {
            eprintln!("Cancelled.");
            return Ok(());
        }
    };

    let (_, full_model_id, _) = &model_entries[selected_idx];

    // Action: copy or set as default
    let actions = vec![
        "Set as default model".to_string(),
        "Copy model ID to clipboard".to_string(),
    ];

    let action = Select::new("What would you like to do?", actions).prompt();

    match action {
        Ok(ref a) if a.starts_with("Set") => {
            let mut config = Config::load()?;
            config.default_model = Some(full_model_id.clone());
            config.save()?;
            eprintln!("✓ Default model set to {full_model_id}");
        }
        Ok(ref a) if a.starts_with("Copy") => {
            match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(full_model_id.clone())) {
                Ok(_) => eprintln!("✓ Copied '{full_model_id}' to clipboard."),
                Err(e) => eprintln!("Failed to copy to clipboard: {e}"),
            }
        }
        _ => {
            eprintln!("Cancelled.");
        }
    }

    Ok(())
}
