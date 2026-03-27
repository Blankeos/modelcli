use anyhow::{Context, Result, bail};
use futures::StreamExt;
use rig::client::completion::CompletionClient;
use rig::completion::Prompt;
use rig::providers::{anthropic, cohere, gemini, groq, mistral, openai, perplexity, together, xai};
use rig::streaming::{StreamedAssistantContent, StreamingPrompt};
use rig::agent::MultiTurnStreamItem;
use std::io::Write;

use crate::api::models_dev::{Model, Provider};

/// Make a completion call using rig
pub async fn call_model(
    provider: &Provider,
    model_meta: &Model,
    api_key: &str,
    prompt_text: &str,
    stream: bool,
    show_thinking: bool,
    _reasoning_effort: Option<&str>,
    format_json: bool,
) -> Result<()> {
    let model_id = &model_meta.id;

    // Providers with an explicit api base URL always use the OpenAI-compatible path.
    // Anthropic is the only provider with a native non-OpenAI rig client.
    // All other providers without an api field are dispatched via their npm package name
    // to the appropriate native rig client (no hardcoded URLs needed).
    if provider.id == "anthropic" {
        call_anthropic(api_key, model_id, model_meta, prompt_text, stream, show_thinking, format_json).await
    } else if let Some(base_url) = &provider.api {
        call_openai_compatible(base_url, api_key, model_id, model_meta, prompt_text, stream, show_thinking, format_json).await
    } else {
        match provider.npm.as_deref() {
            Some("@ai-sdk/openai") =>
                call_openai(api_key, model_id, model_meta, prompt_text, stream, show_thinking, format_json).await,
            Some("@ai-sdk/google") =>
                call_gemini(api_key, model_id, model_meta, prompt_text, stream, show_thinking, format_json).await,
            Some("@ai-sdk/groq") =>
                call_groq(api_key, model_id, model_meta, prompt_text, stream, show_thinking, format_json).await,
            Some("@ai-sdk/mistral") =>
                call_mistral(api_key, model_id, model_meta, prompt_text, stream, show_thinking, format_json).await,
            Some("@ai-sdk/xai") =>
                call_xai(api_key, model_id, model_meta, prompt_text, stream, show_thinking, format_json).await,
            Some("@ai-sdk/togetherai") =>
                call_together(api_key, model_id, model_meta, prompt_text, stream, show_thinking, format_json).await,
            Some("@ai-sdk/cohere") =>
                call_cohere(api_key, model_id, model_meta, prompt_text, stream, show_thinking, format_json).await,
            Some("@ai-sdk/perplexity") =>
                call_perplexity(api_key, model_id, model_meta, prompt_text, stream, show_thinking, format_json).await,
            _ => bail!(
                "Provider '{}' is not supported. No API base URL and no known native client.",
                provider.name
            ),
        }
    }
}

async fn call_anthropic(
    api_key: &str,
    model_id: &str,
    model_meta: &Model,
    prompt_text: &str,
    stream: bool,
    show_thinking: bool,
    format_json: bool,
) -> Result<()> {
    let client = anthropic::Client::builder()
        .api_key(api_key)
        .build()
        .context("Failed to create Anthropic client")?;

    let mut agent_builder = client.agent(model_id).max_tokens(4096);
    if model_meta.temperature != Some(false) {
        agent_builder = agent_builder.temperature(0.7);
    }
    let agent = agent_builder.build();

    if stream { do_stream(&agent, prompt_text, show_thinking).await } else { do_prompt(&agent, prompt_text, format_json, model_id).await }
}

async fn call_openai_compatible(
    base_url: &str,
    api_key: &str,
    model_id: &str,
    model_meta: &Model,
    prompt_text: &str,
    stream: bool,
    show_thinking: bool,
    format_json: bool,
) -> Result<()> {
    let client = openai::CompletionsClient::builder()
        .api_key(api_key)
        .base_url(base_url)
        .build()
        .context("Failed to create OpenAI-compatible client")?;

    let mut agent_builder = client.agent(model_id).max_tokens(4096);
    if model_meta.temperature != Some(false) {
        agent_builder = agent_builder.temperature(0.7);
    }
    let agent = agent_builder.build();

    if stream { do_stream(&agent, prompt_text, show_thinking).await } else { do_prompt(&agent, prompt_text, format_json, model_id).await }
}

async fn call_openai(
    api_key: &str,
    model_id: &str,
    model_meta: &Model,
    prompt_text: &str,
    stream: bool,
    show_thinking: bool,
    format_json: bool,
) -> Result<()> {
    let client = openai::CompletionsClient::builder()
        .api_key(api_key)
        .build()
        .context("Failed to create OpenAI client")?;

    let mut agent_builder = client.agent(model_id).max_tokens(4096);
    if model_meta.temperature != Some(false) {
        agent_builder = agent_builder.temperature(0.7);
    }
    let agent = agent_builder.build();

    if stream { do_stream(&agent, prompt_text, show_thinking).await } else { do_prompt(&agent, prompt_text, format_json, model_id).await }
}

async fn call_gemini(
    api_key: &str,
    model_id: &str,
    model_meta: &Model,
    prompt_text: &str,
    stream: bool,
    show_thinking: bool,
    format_json: bool,
) -> Result<()> {
    let client = gemini::Client::new(api_key)
        .context("Failed to create Gemini client")?;

    let mut agent_builder = client.agent(model_id).max_tokens(4096);
    if model_meta.temperature != Some(false) {
        agent_builder = agent_builder.temperature(0.7);
    }
    let agent = agent_builder.build();

    if stream { do_stream(&agent, prompt_text, show_thinking).await } else { do_prompt(&agent, prompt_text, format_json, model_id).await }
}

async fn call_groq(
    api_key: &str,
    model_id: &str,
    model_meta: &Model,
    prompt_text: &str,
    stream: bool,
    show_thinking: bool,
    format_json: bool,
) -> Result<()> {
    let client = groq::Client::new(api_key)
        .context("Failed to create Groq client")?;

    let mut agent_builder = client.agent(model_id).max_tokens(4096);
    if model_meta.temperature != Some(false) {
        agent_builder = agent_builder.temperature(0.7);
    }
    let agent = agent_builder.build();

    if stream { do_stream(&agent, prompt_text, show_thinking).await } else { do_prompt(&agent, prompt_text, format_json, model_id).await }
}

async fn call_mistral(
    api_key: &str,
    model_id: &str,
    model_meta: &Model,
    prompt_text: &str,
    stream: bool,
    show_thinking: bool,
    format_json: bool,
) -> Result<()> {
    let client = mistral::Client::new(api_key)
        .context("Failed to create Mistral client")?;

    let mut agent_builder = client.agent(model_id).max_tokens(4096);
    if model_meta.temperature != Some(false) {
        agent_builder = agent_builder.temperature(0.7);
    }
    let agent = agent_builder.build();

    if stream { do_stream(&agent, prompt_text, show_thinking).await } else { do_prompt(&agent, prompt_text, format_json, model_id).await }
}

async fn call_xai(
    api_key: &str,
    model_id: &str,
    model_meta: &Model,
    prompt_text: &str,
    stream: bool,
    show_thinking: bool,
    format_json: bool,
) -> Result<()> {
    let client = xai::Client::new(api_key)
        .context("Failed to create xAI client")?;

    let mut agent_builder = client.agent(model_id).max_tokens(4096);
    if model_meta.temperature != Some(false) {
        agent_builder = agent_builder.temperature(0.7);
    }
    let agent = agent_builder.build();

    if stream { do_stream(&agent, prompt_text, show_thinking).await } else { do_prompt(&agent, prompt_text, format_json, model_id).await }
}

async fn call_together(
    api_key: &str,
    model_id: &str,
    model_meta: &Model,
    prompt_text: &str,
    stream: bool,
    show_thinking: bool,
    format_json: bool,
) -> Result<()> {
    let client = together::Client::new(api_key)
        .context("Failed to create Together AI client")?;

    let mut agent_builder = client.agent(model_id).max_tokens(4096);
    if model_meta.temperature != Some(false) {
        agent_builder = agent_builder.temperature(0.7);
    }
    let agent = agent_builder.build();

    if stream { do_stream(&agent, prompt_text, show_thinking).await } else { do_prompt(&agent, prompt_text, format_json, model_id).await }
}

async fn call_cohere(
    api_key: &str,
    model_id: &str,
    model_meta: &Model,
    prompt_text: &str,
    stream: bool,
    show_thinking: bool,
    format_json: bool,
) -> Result<()> {
    let client = cohere::Client::new(api_key)
        .context("Failed to create Cohere client")?;

    let mut agent_builder = client.agent(model_id).max_tokens(4096);
    if model_meta.temperature != Some(false) {
        agent_builder = agent_builder.temperature(0.7);
    }
    let agent = agent_builder.build();

    if stream { do_stream(&agent, prompt_text, show_thinking).await } else { do_prompt(&agent, prompt_text, format_json, model_id).await }
}

async fn call_perplexity(
    api_key: &str,
    model_id: &str,
    model_meta: &Model,
    prompt_text: &str,
    stream: bool,
    show_thinking: bool,
    format_json: bool,
) -> Result<()> {
    let client = perplexity::Client::new(api_key)
        .context("Failed to create Perplexity client")?;

    let mut agent_builder = client.agent(model_id).max_tokens(4096);
    if model_meta.temperature != Some(false) {
        agent_builder = agent_builder.temperature(0.7);
    }
    let agent = agent_builder.build();

    if stream { do_stream(&agent, prompt_text, show_thinking).await } else { do_prompt(&agent, prompt_text, format_json, model_id).await }
}

async fn do_prompt<M>(agent: &rig::agent::Agent<M>, prompt_text: &str, format_json: bool, model_id: &str) -> Result<()>
where
    M: rig::completion::CompletionModel,
{
    let response: String = agent
        .prompt(prompt_text)
        .await
        .context("Completion request failed")?;

    if format_json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "model": model_id,
            "response": response,
        }))?);
    } else {
        println!("{response}");
    }
    Ok(())
}

async fn do_stream<M>(agent: &rig::agent::Agent<M>, prompt_text: &str, show_thinking: bool) -> Result<()>
where
    M: rig::completion::CompletionModel + 'static,
    M::StreamingResponse: Send + Clone + Unpin + rig::completion::GetTokenUsage,
{
    let mut stream = agent
        .stream_prompt(prompt_text)
        .await;

    let mut is_reasoning = false;
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text))) => {
                if is_reasoning {
                    is_reasoning = false;
                    if show_thinking {
                        eprintln!("\n---");
                    }
                }
                print!("{}", text.text);
                std::io::stdout().flush()?;
            }
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Reasoning(reasoning))) => {
                if show_thinking {
                    if !is_reasoning {
                        is_reasoning = true;
                        eprint!("Thinking:\n");
                    }
                    eprint!("{}", reasoning.display_text());
                    std::io::stderr().flush()?;
                }
            }
            Ok(MultiTurnStreamItem::FinalResponse(_)) => {}
            Err(e) => {
                bail!("Stream error: {e}");
            }
            _ => {}
        }
    }
    println!();
    Ok(())
}
