mod api;
mod commands;
mod storage;
mod ui;

use clap::{Parser, Subcommand};

const LOGO: &str = r#"
███╗░░░███╗░█████╗░██████╗░███████╗██╗░░░░░░█████╗░██╗░░░░░██╗
████╗░████║██╔══██╗██╔══██╗██╔════╝██║░░░░░██╔══██╗██║░░░░░██║
██╔████╔██║██║░░██║██║░░██║█████╗░░██║░░░░░██║░░╚═╝██║░░░░░██║
██║╚██╔╝██║██║░░██║██║░░██║██╔══╝░░██║░░░░░██║░░██╗██║░░░░░██║
██║░╚═╝░██║╚█████╔╝██████╔╝███████╗███████╗╚█████╔╝███████╗██║
╚═╝░░░░░╚═╝░╚════╝░╚═════╝░╚══════╝╚══════╝░╚════╝░╚══════╝╚═╝
"#;

#[derive(Parser)]
#[command(
    name = "modelcli",
    version,
    about = "Call any LLM via models.dev",
    before_help = LOGO,
)]
struct Cli {
    /// The prompt to send to the model (default command)
    prompt: Option<String>,

    /// Stream tokens as they arrive
    #[arg(long, default_value_t = false)]
    stream: bool,

    /// Output format: "default" (human-readable) or "json"
    #[arg(long, default_value = "default")]
    format: String,

    /// Show thinking/reasoning tokens
    #[arg(long, default_value_t = false)]
    thinking: bool,

    /// Reasoning effort level (e.g. "low", "medium", "high")
    #[arg(long)]
    reasoning_effort: Option<String>,

    /// Model to use as <provider>/<model-id>
    #[arg(long)]
    model: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Connect to a provider (add API key)
    Connect,
    /// Browse and manage models
    Models,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Connect) => commands::connect::run().await,
        Some(Commands::Models) => commands::models::run().await,
        None => {
            if let Some(prompt_text) = &cli.prompt {
                commands::prompt::run(
                    prompt_text,
                    cli.model.as_deref(),
                    cli.stream,
                    cli.thinking,
                    cli.reasoning_effort.as_deref(),
                    cli.format == "json",
                )
                .await
            } else {
                Cli::parse_from(["modelcli", "--help"]);
                unreachable!()
            }
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
