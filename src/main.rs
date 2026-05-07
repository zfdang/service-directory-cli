use clap::{Parser, Subcommand};

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
        default_value = "http://localhost:8080"
    )]
    base_url: url::Url,

    /// Profile name in ~/.config/kite/directory/credentials.toml
    #[arg(long, env = "KITEDIR_PROFILE", default_value = "default")]
    profile: String,

    /// Emit machine-readable JSON
    #[arg(long, global = true)]
    json: bool,

    /// Disable ANSI color
    #[arg(long, global = true)]
    no_color: bool,

    /// Refuse to prompt for input
    #[arg(long, global = true)]
    non_interactive: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print API version (Phase 0 smoke).
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
    /// Comments.
    Comments {
        #[command(subcommand)]
        cmd: CrudCmd,
    },
    /// Ratings.
    Ratings {
        #[command(subcommand)]
        cmd: CrudCmd,
    },
    /// Evaluations (combined comment+rating).
    Evaluations {
        #[command(subcommand)]
        cmd: CrudCmd,
    },
    /// Authentication.
    Auth {
        #[command(subcommand)]
        cmd: AuthCmd,
    },
    /// Moderation operations.
    Moderation {
        #[command(subcommand)]
        cmd: ModerationCmd,
    },
    /// Admin operations.
    Admin {
        #[command(subcommand)]
        cmd: AdminCmd,
    },
    /// Shell completions.
    Completions {
        #[arg(value_enum)]
        shell: clap_complete_shell::Shell,
    },
}

#[derive(Subcommand)]
enum ProviderCmd {
    Search {
        #[arg(long)]
        q: Option<String>,
    },
    Get {
        provider_id: String,
    },
    Submit {
        path_or_url: String,
    },
}

#[derive(Subcommand)]
enum DescriptorCmd {
    Validate { path_or_url: String },
}

#[derive(Subcommand)]
enum CrudCmd {
    Add { provider_id: String },
    List { provider_id: String },
}

#[derive(Subcommand)]
enum AuthCmd {
    DeviceFlow {
        #[command(subcommand)]
        cmd: DeviceFlowCmd,
    },
}

#[derive(Subcommand)]
enum DeviceFlowCmd {
    Start,
    Poll { device_code: String },
}

#[derive(Subcommand)]
enum ModerationCmd {
    List,
    Hide {
        target_type: String,
        target_id: String,
    },
}

#[derive(Subcommand)]
enum AdminCmd {
    Managers {
        #[command(subcommand)]
        cmd: ManagerCmd,
    },
}

#[derive(Subcommand)]
enum ManagerCmd {
    Invite { email_or_account: String },
}

mod clap_complete_shell {
    #[derive(clap::ValueEnum, Clone, Copy, Debug)]
    pub enum Shell {
        Bash,
        Zsh,
        Fish,
    }
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

    match cli.command {
        Commands::Version => {
            let v = client.version().await?;
            if cli.json {
                println!("{}", serde_json::to_string(&v)?);
            } else {
                println!("{}", serde_json::to_string_pretty(&v)?);
            }
        }
        other => {
            let stub = stub_name(&other);
            anyhow::bail!(
                "`{stub}` not yet implemented in Phase 0; profile={}",
                cli.profile
            );
        }
    }
    Ok(())
}

fn stub_name(c: &Commands) -> &'static str {
    match c {
        Commands::Version => "version",
        Commands::Providers { .. } => "providers",
        Commands::Descriptors { .. } => "descriptors",
        Commands::Comments { .. } => "comments",
        Commands::Ratings { .. } => "ratings",
        Commands::Evaluations { .. } => "evaluations",
        Commands::Auth { .. } => "auth",
        Commands::Moderation { .. } => "moderation",
        Commands::Admin { .. } => "admin",
        Commands::Completions { .. } => "completions",
    }
}
