use anyhow::Result;
use clap::{Parser, Subcommand};

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
}

#[derive(Subcommand)]
enum GitAction {
    /// Add AGENTS.md files to .gitignore
    IgnoreOutputs,
    /// Remove AGENTS.md from .gitignore
    UnignoreOutputs,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Command::Init{preset, force, dry_run, backup} => cagents_core::cmd_init(&preset, force, dry_run, backup)?,
        Command::Build{out,dry_run} => cagents_core::cmd_build(out, dry_run)?,
        Command::Lint => cagents_core::cmd_lint()?,
        Command::Preview{path} => cagents_core::cmd_preview(&path)?,
        Command::Migrate{from, backup} => cagents_core::cmd_migrate(from.as_deref(), backup)?,
        Command::Git{action} => {
            match action {
                GitAction::IgnoreOutputs => cagents_core::helpers::git::ignore_outputs()?,
                GitAction::UnignoreOutputs => cagents_core::helpers::git::unignore_outputs()?,
            }
        }
        Command::Setup{manager} => cagents_core::cmd_setup(&manager)?,
        Command::Status => cagents_core::cmd_status()?,
        Command::Render{file, vars} => cagents_core::cmd_render(&file, vars)?,
        Command::Context{file, vars, json} => cagents_core::cmd_context(&file, vars, json)?,
    }
    Ok(())
}
