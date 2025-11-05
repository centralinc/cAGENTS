use anyhow::Result;
use clap::{Parser, Subcommand};
use cagents_telemetry::{TelemetryClient, CommandEvent};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "cagents", version, about = "cAGENTS CLI")]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scaffold config + example templates
    Init {
        #[arg(long, default_value = "basic")]
        preset: String,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        backup: bool,
    },
    /// Build AGENTS.md (and optional exports) for all targets
    Build {
        #[arg(long)] out: Option<String>,
        #[arg(long)] dry_run: bool,
    },
    /// Validate configuration and rules
    Lint,
    /// Preview build output with rendered content for all files
    Preview {
        #[arg(default_value = ".")]
        path: String
    },
    /// Migrate from other formats (.cursorrules, .cursor/rules, AGENTS.md, CLAUDE.md)
    Migrate {
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        backup: bool,
    },
    /// Git integration commands
    Git {
        #[command(subcommand)]
        action: GitAction,
    },
    /// Setup package manager integration
    Setup {
        #[arg(value_name = "MANAGER")]
        manager: String,
    },
    /// Show project status and statistics
    Status,
    /// Render AGENTS.md for a specific file
    Render {
        /// File path to render context for
        file: String,
        /// Variables in key=value format (can be specified multiple times)
        #[arg(long = "var", value_name = "KEY=VALUE")]
        vars: Vec<String>,
    },
    /// Show comprehensive context and metadata for a file
    Context {
        /// File path to get context for
        file: String,
        /// Variables in key=value format (can be specified multiple times)
        #[arg(long = "var", value_name = "KEY=VALUE")]
        vars: Vec<String>,
        /// Output in JSON format instead of Markdown
        #[arg(long)]
        json: bool,
    },
    /// Manage telemetry settings
    Telemetry {
        #[command(subcommand)]
        action: TelemetryAction,
    },
}

#[derive(Subcommand)]
enum TelemetryAction {
    /// Enable telemetry
    Enable,
    /// Disable telemetry
    Disable,
    /// Show telemetry status
    Status,
}

#[derive(Subcommand)]
enum GitAction {
    /// Add AGENTS.md files to .gitignore
    IgnoreOutputs,
    /// Remove AGENTS.md from .gitignore
    UnignoreOutputs,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize telemetry client
    let mut telemetry = TelemetryClient::new().unwrap_or_default();

    // Get command name for tracking
    let command_name = get_command_name(&cli.cmd);

    // Start timing
    let start = Instant::now();

    // Execute command
    let result = match cli.cmd {
        Command::Init{preset, force, dry_run, backup} => cagents_core::cmd_init(&preset, force, dry_run, backup),
        Command::Build{out,dry_run} => cagents_core::cmd_build(out, dry_run),
        Command::Lint => cagents_core::cmd_lint(),
        Command::Preview{path} => cagents_core::cmd_preview(&path),
        Command::Migrate{from, backup} => cagents_core::cmd_migrate(from.as_deref(), backup),
        Command::Git{action} => {
            match action {
                GitAction::IgnoreOutputs => cagents_core::helpers::git::ignore_outputs(),
                GitAction::UnignoreOutputs => cagents_core::helpers::git::unignore_outputs(),
            }
        }
        Command::Setup{manager} => cagents_core::cmd_setup(&manager),
        Command::Status => cagents_core::cmd_status(),
        Command::Render{file, vars} => cagents_core::cmd_render(&file, vars),
        Command::Context{file, vars, json} => cagents_core::cmd_context(&file, vars, json),
        Command::Telemetry{action} => handle_telemetry_command(action, &telemetry),
    };

    // Track command execution (non-blocking)
    let duration_ms = start.elapsed().as_millis() as u64;
    let success = result.is_ok();
    let error_type = result.as_ref().err().map(|e| format!("{:?}", e));

    let event = CommandEvent::new(
        telemetry.machine_id().to_string(),
        uuid::Uuid::new_v4().to_string(),
        command_name,
        env!("CARGO_PKG_VERSION").to_string(),
    );

    let mut event = event;
    event.duration_ms = duration_ms;
    event.success = success;
    event.error_type = error_type;

    telemetry.track_command(event);

    result
}

fn get_command_name(cmd: &Command) -> String {
    match cmd {
        Command::Init{..} => "init".to_string(),
        Command::Build{..} => "build".to_string(),
        Command::Lint => "lint".to_string(),
        Command::Preview{..} => "preview".to_string(),
        Command::Migrate{..} => "migrate".to_string(),
        Command::Git{..} => "git".to_string(),
        Command::Setup{..} => "setup".to_string(),
        Command::Status => "status".to_string(),
        Command::Render{..} => "render".to_string(),
        Command::Context{..} => "context".to_string(),
        Command::Telemetry{..} => "telemetry".to_string(),
    }
}

fn handle_telemetry_command(action: TelemetryAction, client: &TelemetryClient) -> Result<()> {
    use std::fs;

    match action {
        TelemetryAction::Enable => {
            // Update user config
            if let Some(home_dir) = dirs::home_dir() {
                let config_dir = home_dir.join(".cagents");
                let config_path = config_dir.join("config.toml");

                fs::create_dir_all(&config_dir)?;

                let content = if config_path.exists() {
                    fs::read_to_string(&config_path)?
                } else {
                    String::new()
                };

                // Simple TOML update (replace [telemetry] section)
                let new_content = if content.contains("[telemetry]") {
                    content.replace("enabled = false", "enabled = true")
                } else {
                    format!("{}\n[telemetry]\nenabled = true\n", content)
                };

                fs::write(&config_path, new_content)?;
                println!("✓ Telemetry enabled");
                println!("  Machine ID: {}...", &client.machine_id()[..8]);
                println!("  Learn more: https://github.com/centralinc/cagents/blob/main/docs/TELEMETRY.md");
            }
        }
        TelemetryAction::Disable => {
            // Update user config
            if let Some(home_dir) = dirs::home_dir() {
                let config_dir = home_dir.join(".cagents");
                let config_path = config_dir.join("config.toml");

                fs::create_dir_all(&config_dir)?;

                let content = if config_path.exists() {
                    fs::read_to_string(&config_path)?
                } else {
                    String::new()
                };

                // Simple TOML update
                let new_content = if content.contains("[telemetry]") {
                    content.replace("enabled = true", "enabled = false")
                } else {
                    format!("{}\n[telemetry]\nenabled = false\n", content)
                };

                fs::write(&config_path, new_content)?;
                println!("✓ Telemetry disabled");
                println!("  You can re-enable with: cagents telemetry enable");
            }
        }
        TelemetryAction::Status => {
            println!("Telemetry Status");
            println!("================");
            println!("Enabled: {}", if client.is_enabled() { "yes" } else { "no" });
            println!("Debug Mode: {}", if client.is_debug() { "yes" } else { "no" });
            println!("Machine ID: {}...", &client.machine_id()[..16]);
            println!();
            println!("Opt-out: cagents telemetry disable");
            println!("Learn more: https://github.com/centralinc/cagents/blob/main/docs/TELEMETRY.md");
        }
    }

    Ok(())
}
