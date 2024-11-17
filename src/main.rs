use anyhow::Result;
use donda::{
    cli::{get_cli_args, MAX_DAYS},
    sacct::fetch_submit_records,
    tui::tui_lunch,
};

fn main() -> Result<()> {
    let args = get_cli_args();
    // if start and end are not specified, use 180 days ago from today
    // if only start or end is specified, raise error

    let start = args.start.unwrap_or_else(|| {
        let today = chrono::Utc::now().date_naive();
        today - chrono::Duration::days(MAX_DAYS)
    });
    let end = args.end.unwrap_or_else(|| chrono::Utc::now().date_naive());
    println!("Start: {}, End: {}", start, end);

    let submit_data = fetch_submit_records(start, end)?;
    tui_lunch(submit_data, args.full_header, args.color_scheme)?;
    Ok(())
}
