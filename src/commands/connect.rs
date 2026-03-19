use anyhow::{Result, bail};
use inquire::{Confirm, Password, Select, Text};

use crate::api::models_dev::{self, text_providers, sorted_provider_ids};
use crate::storage::{AuthStore, CustomConfig};

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
    let other_label = "Other (custom provider)".to_string();

    let mut display_names: Vec<String> = sorted_ids
        .iter()
        .map(|id| {
            let p = &providers[id];
            let connected = if auth.get(id).is_some() { " (connected)" } else { "" };
            format!("{}{connected}", p.name)
        })
        .collect();
    display_names.push(other_label.clone());

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

    // "Other" is the last item
    if display_names[selected_idx] == other_label {
        return custom_provider_flow(&mut auth, &providers).await;
    }

    let provider_id = &sorted_ids[selected_idx];
    let provider = &providers[provider_id];

    let env_hint = provider.env.first().map(|e| e.as_str()).unwrap_or("API_KEY");

    let api_key = Password::new(&format!("Enter API key for {} ({env_hint}):", provider.name))
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
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

async fn custom_provider_flow(
    auth: &mut AuthStore,
    known_providers: &models_dev::ProvidersMap,
) -> Result<()> {
    // Prompt for provider ID
    let provider_id = Text::new("Enter a unique provider ID (e.g. \"myprovider\"):")
        .prompt();

    let provider_id = match provider_id {
        Ok(id) => id.trim().to_string(),
        Err(_) => {
            eprintln!("Cancelled.");
            return Ok(());
        }
    };

    if provider_id.is_empty() {
        eprintln!("Provider ID cannot be empty.");
        return Ok(());
    }

    // Validate: alphanumeric + hyphens only
    let valid = provider_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-');
    if !valid {
        eprintln!("Provider ID must be alphanumeric and hyphens only.");
        return Ok(());
    }

    // Must not collide with a known models.dev provider
    if known_providers.contains_key(&provider_id) {
        eprintln!(
            "Provider ID '{}' is already a known provider. Choose a different ID.",
            provider_id
        );
        return Ok(());
    }

    let config_path = CustomConfig::config_path()?;
    let config_display = config_path.display();

    eprintln!();
    eprintln!("Note: This only stores a credential for \"{}\".", provider_id);
    eprintln!(
        "You'll need to configure it in {}",
        config_display
    );
    eprintln!("(see docs for format).");
    eprintln!();

    // Prompt for API key
    let api_key = Password::new(&format!("Enter API key for \"{}\":", provider_id))
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .without_confirmation()
        .prompt();

    match api_key {
        Ok(key) => {
            if key.trim().is_empty() {
                eprintln!("API key cannot be empty.");
                return Ok(());
            }
            auth.set(&provider_id, key.trim());
            auth.save()?;
            eprintln!("✓ Credential saved for \"{}\".", provider_id);

            // Auto-scaffold config.jsonc if it doesn't exist
            CustomConfig::scaffold(&provider_id)?;

            eprintln!(
                "✓ Next step: add provider config to {}",
                config_display
            );
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
