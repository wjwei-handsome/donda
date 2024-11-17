use chrono::NaiveDate;
use clap::{Parser, ValueEnum};

const MAX_DAYS: i64 = 180;

/// Command-line interface for configuring Slurm submit record analysis and display
#[derive(Parser, Debug)]
#[command(name = "donda")]
#[command(author = "Wenjie Wei")]
#[command(version = "0.1")]
#[command(about = "Show Slurm submit heatmap in a TUI", long_about = None)]
pub struct Cli {
    /// Start date in the format YYYY-MM-DD (optional, defaults to 180 days ago from today)
    #[arg(long, short, value_parser = parse_date)]
    pub start: Option<NaiveDate>,

    /// End date in the format YYYY-MM-DD (optional, defaults to today)
    #[arg(long, short, value_parser = parse_date)]
    pub end: Option<NaiveDate>,

    /// Display full header in TUI (default: false)
    #[arg(long, short, action = clap::ArgAction::SetTrue)]
    pub full_header: bool,

    /// Color scheme to use for TUI (not yet implemented)
    #[arg(long, short, value_enum)]
    pub color_scheme: Option<ColorScheme>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum ColorScheme {
    Default,
    // Future schemes can be added here
}

fn parse_date(date_str: &str) -> Result<NaiveDate, String> {
    date_str
        .parse()
        .map_err(|_| "Invalid date format. Please Use `YYYY-MM-DD`.".to_string())
}

impl Cli {
    /// Validates that the date range is within 180 days if both `start` and `end` are specified
    pub fn validate_date_range(&self) -> Result<(), String> {
        if let (Some(start), Some(end)) = (self.start, self.end) {
            let duration = end.signed_duration_since(start).num_days();
            if !(0..=MAX_DAYS).contains(&duration) {
                return Err(
                    "Date range must be within 180 days and start must be before end.".into(),
                );
            }
        }
        Ok(())
    }
}

pub fn get_cli_args() -> Cli {
    let cli = Cli::parse();
    if let Err(e) = cli.validate_date_range() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
    cli
}
