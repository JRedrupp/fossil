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
        println!("Found {} markers before git enrichment", markers.len());
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

    for marker in &mut markers {
        marker.git_info =
            git::enrich_with_git_info(repo.as_ref(), &marker.file_path, marker.line_number);
    }

    // Apply filters
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

    if let Some(ref marker_type) = args.marker_type {
        if args.verbose {
            println!("Filtering by type: {}", marker_type);
        }
        markers = filters::filter_by_type(markers, marker_type);
    }

    if args.verbose {
        println!("Generating report with {} markers", markers.len());
    }

    // Generate report
    let report = models::DebtReport::new(markers, args.path.clone());

    // Output report
    reporter::generate_report(&report, args.format, args.output.as_deref(), args.top)
        .context("Failed to generate report")?;

    Ok(())
}
