use anyhow::{Result, bail};
use inquire::{Confirm, Password, Select};

use crate::api::models_dev::{self, text_providers, sorted_provider_ids};
use crate::storage::AuthStore;

pub async fn run() -> Result<()> {
    let providers = models_dev::fetch_providers().await?;
    let providers = text_providers(&providers);

    if providers.is_empty() {
        bail!("No text-output models available from models.dev.");
    }

    let mut auth = AuthStore::load()?;

    // If already connected to some providers, offer add/disconnect choice
    if !auth.is_empty() {
        let connected: Vec<String> = auth.connected_providers().iter().map(|s| s.to_string()).collect();
        let choices = vec![
            "Add new provider".to_string(),
            "Disconnect a provider".to_string(),
        ];

        let action = Select::new(
            &format!("Connected to {} provider(s). What would you like to do?", connected.len()),
            choices,
        )
        .prompt();

        match action {
            Ok(ref a) if a == "Disconnect a provider" => {
                return disconnect_flow(&mut auth, &providers);
            }
            Ok(_) => { /* continue to add flow */ }
            Err(_) => {
                eprintln!("Cancelled.");
                return Ok(());
            }
        }
    }

    // Add provider flow
    let sorted_ids = sorted_provider_ids(&providers);
    let display_names: Vec<String> = sorted_ids
        .iter()
        .map(|id| {
            let p = &providers[id];
            let connected = if auth.get(id).is_some() { " (connected)" } else { "" };
            format!("{}{connected}", p.name)
        })
        .collect();

    let selection = Select::new("Select a provider:", display_names.clone())
        .with_page_size(15)
        .prompt();

    let selected_idx = match selection {
        Ok(ref selected) => display_names.iter().position(|d| d == selected).unwrap(),
        Err(_) => {
            eprintln!("Cancelled.");
            return Ok(());
        }
    };

    let provider_id = &sorted_ids[selected_idx];
    let provider = &providers[provider_id];

    let env_hint = provider.env.first().map(|e| e.as_str()).unwrap_or("API_KEY");

    let api_key = Password::new(&format!("Enter API key for {} ({env_hint}):", provider.name))
        .without_confirmation()
        .prompt();

    match api_key {
        Ok(key) => {
            if key.trim().is_empty() {
                eprintln!("API key cannot be empty.");
                return Ok(());
            }
            auth.set(provider_id, key.trim());
            auth.save()?;
            eprintln!("✓ Connected to {}.", provider.name);
        }
        Err(_) => {
            eprintln!("Cancelled.");
        }
    }

    Ok(())
}

fn disconnect_flow(
    auth: &mut AuthStore,
    providers: &models_dev::ProvidersMap,
) -> Result<()> {
    let connected: Vec<String> = auth
        .connected_providers()
        .iter()
        .map(|s| s.to_string())
        .collect();

    let display: Vec<String> = connected
        .iter()
        .map(|id| {
            providers
                .get(id)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| id.clone())
        })
        .collect();

    let selection = Select::new("Select provider to disconnect:", display.clone())
        .prompt();

    let selected_idx = match selection {
        Ok(ref selected) => display.iter().position(|d| d == selected).unwrap(),
        Err(_) => {
            eprintln!("Cancelled.");
            return Ok(());
        }
    };

    let provider_id = &connected[selected_idx];
    let provider_name = &display[selected_idx];

    let confirm = Confirm::new(&format!("Are you sure you want to disconnect {provider_name}?"))
        .with_default(false)
        .prompt();

    match confirm {
        Ok(true) => {
            auth.remove(provider_id);
            auth.save()?;
            eprintln!("✓ Disconnected from {provider_name}.");
        }
        _ => {
            eprintln!("Cancelled.");
        }
    }

    Ok(())
}
