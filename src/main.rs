mod config;

use anyhow::Context;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use serde_json::Value;
use service_directory_client::Client;
use std::path::PathBuf;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://dir.kitepass.xyz";

#[derive(Parser)]
#[command(
    name = "kitedir",
    version,
    about = "Kite Service Directory CLI",
    propagate_version = true,
    arg_required_else_help = true
)]
struct Cli {
    /// Service Directory base URL (overrides profile).
    #[arg(long, env = "KITEDIR_BASE_URL", global = true)]
    base_url: Option<url::Url>,

    /// Profile name in ~/.config/kite/directory/credentials.toml.
    #[arg(long, env = "KITEDIR_PROFILE", default_value = "default", global = true)]
    profile: String,

    /// Emit machine-readable JSON.
    #[arg(long, global = true)]
    json: bool,

    /// Disable ANSI color (Phase 5: humans only have a small color budget).
    #[arg(long, global = true)]
    no_color: bool,

    /// Refuse to prompt for input.
    #[arg(long, global = true)]
    non_interactive: bool,

    /// For state-changing commands, validate but don't send.
    #[arg(long, global = true)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print API version.
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
        cmd: CommentCmd,
    },
    /// Ratings.
    Ratings {
        #[command(subcommand)]
        cmd: RatingCmd,
    },
    /// Combined comment + rating.
    Evaluations {
        #[command(subcommand)]
        cmd: EvalCmd,
    },
    /// Authentication.
    Auth {
        #[command(subcommand)]
        cmd: AuthCmd,
    },
    /// Moderation.
    Moderation {
        #[command(subcommand)]
        cmd: ModerationCmd,
    },
    /// Admin operations.
    Admin {
        #[command(subcommand)]
        cmd: AdminCmd,
    },
    /// Generate shell completions.
    Completions { shell: Shell },
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
    /// Submit a provider profile (and optionally a descriptor draft).
    Submit {
        /// Path to a JSON file (provider profile, with optional `descriptor` key).
        path: PathBuf,
        #[arg(long)]
        notes: Option<String>,
    },
    /// List endpoints from the current descriptor.
    Endpoints { provider_id: String },
    /// Show the descriptor's accepted_payment_protocols + pricing_hints.
    Payments { provider_id: String },
}

#[derive(Subcommand)]
enum DescriptorCmd {
    /// Locally validate a descriptor JSON file.
    Validate { path: PathBuf },
    /// Fetch the current descriptor from the API.
    Get {
        provider_id: String,
        #[arg(long)]
        version: Option<i32>,
    },
}

#[derive(Subcommand)]
enum CommentCmd {
    Add {
        provider_id: String,
        #[arg(long)]
        text: String,
        #[arg(long)]
        rating_snapshot: Option<i16>,
    },
    List { provider_id: String },
}

#[derive(Subcommand)]
enum RatingCmd {
    Add {
        provider_id: String,
        #[arg(long)]
        score: i16,
    },
    List { provider_id: String },
}

#[derive(Subcommand)]
enum EvalCmd {
    Add {
        provider_id: String,
        #[arg(long)]
        text: Option<String>,
        #[arg(long)]
        score: Option<i16>,
    },
}

#[derive(Subcommand)]
enum AuthCmd {
    /// Send a magic-link to an email.
    Login {
        #[arg(long)]
        email: String,
    },
    /// Verify a magic-link token and store the session in this profile.
    Verify {
        #[arg(long)]
        token: String,
    },
    /// Print the current authenticated identity.
    Whoami,
    /// Revoke the current session and clear the local profile.
    Logout,
    /// Mint a single-use 5-minute step-up token (cached in profile).
    Stepup,
    /// RFC 8628 Device Authorization Grant.
    DeviceFlow {
        #[command(subcommand)]
        cmd: DeviceFlowCmd,
    },
}

#[derive(Subcommand)]
enum DeviceFlowCmd {
    /// Start a device-flow request and print the user_code + verification URL.
    Start {
        #[arg(long, value_delimiter = ',')]
        scopes: Vec<String>,
    },
    /// Poll a device_code until approval; on approval saves the agent token to this profile.
    Poll {
        device_code: String,
        #[arg(long, default_value_t = 5)]
        interval_secs: u64,
        #[arg(long, default_value_t = 900)]
        max_secs: u64,
    },
    /// Approve or deny a user_code (run in a logged-in browser session normally).
    Approve {
        user_code: String,
        #[arg(long)]
        deny: bool,
    },
}

#[derive(Subcommand)]
enum ModerationCmd {
    /// Recent hides (admin only).
    RecentHides {
        #[arg(long, default_value_t = 30)]
        days: i64,
    },
    /// Hide / restore / block a target.
    Action {
        #[arg(long)]
        target_type: String,
        #[arg(long)]
        target_id: String,
        #[arg(long)]
        action: String,
        #[arg(long)]
        reason: Option<String>,
    },
}

#[derive(Subcommand)]
enum AdminCmd {
    /// Submission queue.
    Submissions {
        #[arg(long)]
        state: Option<String>,
    },
    /// Review a submission.
    Review {
        submission_id: String,
        #[arg(long)]
        action: String,
        #[arg(long)]
        notes: Option<String>,
    },
    /// Audit chain.
    Audit {
        #[arg(long)]
        after_seq: Option<i64>,
        #[arg(long, default_value_t = 50)]
        limit: i64,
    },
    /// Ranking weights.
    RankingWeights,
}

fn build_client(cli: &Cli) -> anyhow::Result<(Client, config::CredentialsFile, config::Profile)> {
    let file = config::load()?;
    let profile_name = if cli.profile.is_empty() {
        file.default_profile.clone()
    } else {
        cli.profile.clone()
    };
    let profile = config::get_profile(&file, &profile_name)
        .cloned()
        .unwrap_or_default();

    let base_url = cli
        .base_url
        .clone()
        .or_else(|| profile.base_url.as_deref().and_then(|s| url::Url::parse(s).ok()))
        .unwrap_or_else(|| url::Url::parse(DEFAULT_BASE_URL).expect("default base url"));

    let mut client = Client::new(base_url)?;
    if let Some(t) = profile.session_token.as_deref() {
        client = client.with_session(t);
    }
    if let Some(t) = profile.agent_token.as_deref() {
        client = client.with_bearer(t);
    }
    if let Some(t) = profile.stepup_token.as_deref() {
        client = client.with_stepup(t);
    }
    Ok((client, file, profile))
}

fn render(json: bool, value: &Value) {
    if json {
        println!("{}", serde_json::to_string(value).unwrap_or_default());
    } else {
        println!("{}", serde_json::to_string_pretty(value).unwrap_or_default());
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
    if let Commands::Completions { shell } = &cli.command {
        let mut cmd = Cli::command();
        clap_complete::generate(*shell, &mut cmd, "kitedir", &mut std::io::stdout());
        return Ok(());
    }

    let json_out = cli.json;
    let dry_run = cli.dry_run;
    let (client, mut file, mut profile) = build_client(&cli)?;
    let profile_name = cli.profile.clone();

    let result: Value = match cli.command {
        Commands::Version => client.version().await?,
        Commands::Providers { cmd } => match cmd {
            ProviderCmd::Search { q, limit, offset } => {
                client.list_providers(q.as_deref(), Some(limit), Some(offset)).await?
            }
            ProviderCmd::Get { provider_id } => client.get_provider(&provider_id).await?,
            ProviderCmd::Submit { path, notes } => {
                let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
                let payload: Value = serde_json::from_str(&raw)?;
                let descriptor = payload.get("descriptor").cloned();
                let mut provider_payload = payload.clone();
                if let Some(obj) = provider_payload.as_object_mut() {
                    obj.remove("descriptor");
                }
                if dry_run {
                    serde_json::json!({"dry_run": true, "provider_payload": provider_payload, "descriptor": descriptor})
                } else {
                    client.submit_provider(&provider_payload, descriptor.as_ref(), notes.as_deref()).await?
                }
            }
            ProviderCmd::Endpoints { provider_id } => client.list_endpoints(&provider_id).await?,
            ProviderCmd::Payments { provider_id } => client.get_payment_capabilities(&provider_id).await?,
        },
        Commands::Descriptors { cmd } => match cmd {
            DescriptorCmd::Validate { path } => validate_local(&path)?,
            DescriptorCmd::Get { provider_id, version } => {
                client.get_descriptor(&provider_id, version).await?
            }
        },
        Commands::Comments { cmd } => match cmd {
            CommentCmd::Add { provider_id, text, rating_snapshot } => {
                if dry_run {
                    serde_json::json!({"dry_run": true, "provider_id": provider_id, "comment_text": text, "rating_snapshot": rating_snapshot})
                } else {
                    client.add_comment(&provider_id, &text, rating_snapshot).await?
                }
            }
            CommentCmd::List { provider_id } => client.list_comments(&provider_id).await?,
        },
        Commands::Ratings { cmd } => match cmd {
            RatingCmd::Add { provider_id, score } => {
                if !(1..=5).contains(&score) {
                    anyhow::bail!("score must be 1..5");
                }
                if dry_run {
                    serde_json::json!({"dry_run": true, "provider_id": provider_id, "overall_score": score})
                } else {
                    client.add_rating(&provider_id, score).await?
                }
            }
            RatingCmd::List { provider_id } => client.list_ratings(&provider_id).await?,
        },
        Commands::Evaluations { cmd } => match cmd {
            EvalCmd::Add { provider_id, text, score } => {
                let mut out = serde_json::json!({});
                if let Some(s) = score {
                    if !(1..=5).contains(&s) {
                        anyhow::bail!("score must be 1..5");
                    }
                    if !dry_run {
                        out["rating"] = client.add_rating(&provider_id, s).await?;
                    }
                }
                if let Some(t) = text {
                    if !dry_run {
                        out["comment"] = client.add_comment(&provider_id, &t, score).await?;
                    }
                }
                if dry_run {
                    serde_json::json!({"dry_run": true, "provider_id": provider_id})
                } else {
                    out
                }
            }
        },
        Commands::Auth { cmd } => match cmd {
            AuthCmd::Login { email } => {
                let r = client.auth_start(&email).await?;
                profile.email = Some(email.clone());
                config::set_profile(&mut file, &profile_name, profile.clone());
                config::save(&file)?;
                serde_json::json!({"ok": true, "started": r, "next": "look up the magic link in API logs, then `kitedir auth verify --token <raw>`"})
            }
            AuthCmd::Verify { token } => {
                let (cookie, body) = client.auth_verify(&token).await?;
                if let Some(c) = cookie {
                    profile.session_token = Some(c);
                }
                profile.account_id = body
                    .get("account")
                    .and_then(|a| a.get("account_id"))
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                profile.email = body
                    .get("account")
                    .and_then(|a| a.get("email"))
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                config::set_profile(&mut file, &profile_name, profile.clone());
                config::save(&file)?;
                serde_json::json!({"ok": true, "saved_profile": profile_name, "account": body.get("account")})
            }
            AuthCmd::Whoami => {
                let r = client.me().await?;
                if r.get("account").is_none() {
                    serde_json::json!({"role": "anonymous"})
                } else {
                    r
                }
            }
            AuthCmd::Logout => {
                let _ = client.auth_logout().await;
                profile.session_token = None;
                profile.agent_token = None;
                profile.stepup_token = None;
                config::set_profile(&mut file, &profile_name, profile.clone());
                config::save(&file)?;
                serde_json::json!({"ok": true, "cleared_profile": profile_name})
            }
            AuthCmd::Stepup => {
                let r = client.auth_stepup().await?;
                profile.stepup_token = r
                    .get("stepup_token")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                config::set_profile(&mut file, &profile_name, profile.clone());
                config::save(&file)?;
                r
            }
            AuthCmd::DeviceFlow { cmd } => match cmd {
                DeviceFlowCmd::Start { scopes } => {
                    let scopes = if scopes.is_empty() {
                        vec!["directory:read".into(), "directory:evaluate:authenticated".into()]
                    } else {
                        scopes
                    };
                    client.device_flow_start(&scopes).await?
                }
                DeviceFlowCmd::Poll { device_code, interval_secs, max_secs } => {
                    let start = std::time::Instant::now();
                    loop {
                        let r = client.device_flow_poll(&device_code).await?;
                        let state = r.get("state").and_then(|v| v.as_str()).unwrap_or("");
                        if state == "approved" {
                            if let Some(t) = r.get("access_token").and_then(|v| v.as_str()) {
                                profile.agent_token = Some(t.to_string());
                                config::set_profile(&mut file, &profile_name, profile.clone());
                                config::save(&file)?;
                            }
                            break r;
                        }
                        if state == "denied" || state == "expired" {
                            break r;
                        }
                        if start.elapsed().as_secs() > max_secs {
                            anyhow::bail!("device-flow poll timed out after {max_secs}s");
                        }
                        tokio::time::sleep(Duration::from_secs(interval_secs)).await;
                    }
                }
                DeviceFlowCmd::Approve { user_code, deny } => {
                    client.device_flow_approve(&user_code, deny).await?
                }
            },
        },
        Commands::Moderation { cmd } => match cmd {
            ModerationCmd::RecentHides { days } => client.admin_recent_hides(days).await?,
            ModerationCmd::Action { target_type, target_id, action, reason } => {
                if dry_run {
                    serde_json::json!({"dry_run": true, "target_type": target_type, "target_id": target_id, "action": action})
                } else {
                    client.moderation_action(&target_type, &target_id, &action, reason.as_deref()).await?
                }
            }
        },
        Commands::Admin { cmd } => match cmd {
            AdminCmd::Submissions { state } => client.list_submissions(false, state.as_deref()).await?,
            AdminCmd::Review { submission_id, action, notes } => {
                if dry_run {
                    serde_json::json!({"dry_run": true, "submission_id": submission_id, "action": action})
                } else {
                    client.review_submission(&submission_id, &action, notes.as_deref()).await?
                }
            }
            AdminCmd::Audit { after_seq, limit } => client.admin_audit(after_seq, limit).await?,
            AdminCmd::RankingWeights => client.admin_ranking_weights().await?,
        },
        Commands::Completions { .. } => unreachable!(),
    };

    render(json_out, &result);
    Ok(())
}

fn validate_local(path: &PathBuf) -> anyhow::Result<Value> {
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
