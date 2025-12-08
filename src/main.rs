use anyhow::{Context, Result};
use clap::Parser;
use fossil::{cli, config, filters, git, models, reporter, scanner};

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Scan(args) => scan_command(args)?,
    }

    Ok(())
}

fn scan_command(args: cli::ScanArgs) -> Result<()> {
    if args.verbose {
        println!("Fossil - Unearthing technical debt...");
        println!("Scanning: {}", args.path.display());
    }

    // Load configuration
    let config =
        config::load_config(args.config.as_deref()).context("Failed to load configuration")?;

    if args.verbose {
        println!("Using markers: {:?}", config.markers);
    }

    // Scan directory for markers
    let mut markers =
        scanner::scan_directory(&args.path, &config).context("Failed to scan directory")?;

    if args.verbose {
        println!("Found {} markers before filtering", markers.len());
    }

    // Apply type filter first (before git enrichment to reduce work)
    if let Some(ref marker_type) = args.marker_type {
        if args.verbose {
            println!("Filtering by type: {}", marker_type);
        }
        markers = filters::filter_by_type(markers, marker_type);
        if args.verbose {
            println!("Markers after type filter: {}", markers.len());
        }
    }

    // Enrich with git blame information
    let repo = git::get_repository(&args.path)?;

    if args.verbose {
        if repo.is_some() {
            println!("Git repository detected, enriching with blame data...");
        } else {
            println!("No git repository found, skipping blame data");
        }
    }

    git::enrich_markers_batch(&mut markers, repo.as_ref())?;

    // Apply filters that require git data
    if let Some(ref older_than) = args.older_than {
        if args.verbose {
            println!("Filtering by age: {}", older_than);
        }
        markers = filters::filter_by_age(markers, older_than).context("Failed to filter by age")?;
    }

    if let Some(ref author) = args.author {
        if args.verbose {
            println!("Filtering by author: {}", author);
        }
        markers = filters::filter_by_author(markers, author);
    }

    if args.verbose {
        println!("Generating report with {} markers", markers.len());
    }

    // Generate report
    let report = models::DebtReport::new(markers, args.path.clone());

    // Output report
    reporter::generate_report(
        &report,
        args.format,
        args.output.as_deref(),
        args.top,
        args.count_only,
    )
    .context("Failed to generate report")?;

    Ok(())
}
