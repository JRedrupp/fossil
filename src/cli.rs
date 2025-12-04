use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "fossil")]
#[command(version, about = "Unearth your technical debt", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Debug)]
pub enum Commands {
    /// Scan a directory for technical debt markers
    Scan(ScanArgs),
}

#[derive(Parser, Debug)]
pub struct ScanArgs {
    /// Directory to scan (defaults to current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Output format
    #[arg(short, long, value_enum, default_value = "terminal")]
    pub format: OutputFormat,

    /// Output file (if not specified, writes to stdout)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Filter: only show markers older than specified age (e.g., "30d", "6m", "1y")
    #[arg(long)]
    pub older_than: Option<String>,

    /// Filter: only show markers by specific author
    #[arg(long)]
    pub author: Option<String>,

    /// Filter: only show markers of specific type (TODO, FIXME, etc.)
    #[arg(short = 't', long = "type")]
    pub marker_type: Option<String>,

    /// Path to custom config file
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Show only the top N oldest markers
    #[arg(long, default_value = "10")]
    pub top: usize,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    /// Formatted table output for terminal
    Terminal,
    /// Markdown format
    Markdown,
    /// JSON format
    Json,
}
