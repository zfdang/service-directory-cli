use anyhow::Context;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "kitedir",
    version,
    about = "Kite Service Directory CLI",
    propagate_version = true
)]
struct Cli {
    /// Service Directory base URL (e.g. https://dir.kitepass.xyz)
    #[arg(
        long,
        env = "KITEDIR_BASE_URL",
        default_value = "https://dir.kitepass.xyz"
    )]
    base_url: url::Url,

    /// Profile name in ~/.config/kite/directory/credentials.toml
    #[arg(long, env = "KITEDIR_PROFILE", default_value = "default")]
    profile: String,

    /// Emit machine-readable JSON.
    #[arg(long, global = true)]
    json: bool,

    /// Disable ANSI color (currently unused).
    #[arg(long, global = true)]
    no_color: bool,

    /// Refuse to prompt for input (currently unused; CLI is non-interactive in Phase 1).
    #[arg(long, global = true)]
    non_interactive: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print API version and connectivity status.
    Version,
    /// Provider operations.
    Providers {
        #[command(subcommand)]
        cmd: ProviderCmd,
    },
    /// Descriptor operations.
    Descriptors {
        #[command(subcommand)]
        cmd: DescriptorCmd,
    },
}

#[derive(Subcommand)]
enum ProviderCmd {
    /// List or full-text search providers.
    Search {
        #[arg(long)]
        q: Option<String>,
        #[arg(long, default_value_t = 25)]
        limit: i64,
        #[arg(long, default_value_t = 0)]
        offset: i64,
    },
    /// Show one provider.
    Get { provider_id: String },
}

#[derive(Subcommand)]
enum DescriptorCmd {
    /// Locally validate a descriptor JSON file (signature, slug rules, pricing_ref hash format).
    Validate { path: PathBuf },
    /// Fetch the current descriptor for a provider from the directory API.
    Get {
        provider_id: String,
        #[arg(long)]
        version: Option<i32>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let cli = Cli::parse();
    let client = service_directory_client::Client::new(cli.base_url.clone())?;

    let value: serde_json::Value = match cli.command {
        Commands::Version => client.version().await?,
        Commands::Providers { cmd } => match cmd {
            ProviderCmd::Search { q, limit, offset } => {
                client.list_providers(q.as_deref(), Some(limit), Some(offset)).await?
            }
            ProviderCmd::Get { provider_id } => client.get_provider(&provider_id).await?,
        },
        Commands::Descriptors { cmd } => match cmd {
            DescriptorCmd::Validate { path } => validate_local(&path)?,
            DescriptorCmd::Get { provider_id, version } => {
                client.get_descriptor(&provider_id, version).await?
            }
        },
    };

    if cli.json {
        println!("{}", serde_json::to_string(&value)?);
    } else {
        println!("{}", serde_json::to_string_pretty(&value)?);
    }
    Ok(())
}

fn validate_local(path: &PathBuf) -> anyhow::Result<serde_json::Value> {
    use service_directory_descriptor_validator::{validate_signed, SignatureError, ValidationError};
    use service_directory_schemas::ServiceDescriptor;

    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let descriptor: ServiceDescriptor = serde_json::from_str(&raw)?;

    match validate_signed(&descriptor) {
        Ok(out) => {
            let hex = out
                .payload_hash
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            Ok(serde_json::json!({
                "ok": true,
                "service_id": descriptor.service_id,
                "descriptor_version": descriptor.descriptor_version,
                "payload_hash_sha256": hex,
                "warnings": out.warnings,
            }))
        }
        Err(ValidationError::Signature(SignatureError::Missing)) => Ok(serde_json::json!({
            "ok": false,
            "error": "descriptor is unsigned (provider_signature missing); use seed-fixtures or sign offline",
        })),
        Err(e) => Ok(serde_json::json!({
            "ok": false,
            "error": format!("{e}"),
        })),
    }
}
