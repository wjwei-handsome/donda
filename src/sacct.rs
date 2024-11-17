use anyhow::Result;
use chrono::NaiveDate;
use std::process::{Command, Output};
use std::str;

// Submit information for each day
#[derive(Debug)]
pub struct SubmitRecord {
    pub date: NaiveDate,
    pub count: u32,
}

fn check_sacct_exists() -> Result<()> {
    match Command::new("sacct").arg("-V").output() {
        Ok(_) => Ok(()),
        Err(_) => Err(anyhow::anyhow!("R U kidding me? sacct command not found")),
    }
}

pub fn fetch_submit_records(
    start: NaiveDate,
    end: NaiveDate,
    username: Option<String>,
) -> Result<Vec<SubmitRecord>> {
    // check if sacct command exists
    check_sacct_exists()?;
    // get username
    let username = if let Some(username) = username {
        username
    } else {
        whoami::username()
    };
    // execute sacct command and get output
    let output = Command::new("sacct")
        .args([
            "--user",
            &username,
            "--starttime",
            &start.format("%Y-%m-%d").to_string(),
            "--endtime",
            &end.format("%Y-%m-%d").to_string(),
            "--format=Submit",
            "--noheader",
        ])
        .output()?;
    // some output example:
    //2024-11-05T18:07:54
    // 2024-11-05T18:08:00
    // 2024-11-05T18:08:00
    // parse the output
    parse_sacct_output(&output)
}

fn parse_sacct_output(output: &Output) -> Result<Vec<SubmitRecord>> {
    let stdout = str::from_utf8(&output.stdout)?;
    let mut date_counts = std::collections::HashMap::<NaiveDate, u32>::new();

    for line in stdout.lines() {
        if let Ok(submit_date) = NaiveDate::parse_from_str(line.trim(), "%Y-%m-%dT%H:%M:%S") {
            let date_only = submit_date;
            *date_counts.entry(date_only).or_insert(0) += 1;
        }
    }

    // convert to SubmitRecord
    let records: Vec<SubmitRecord> = date_counts
        .into_iter()
        .map(|(date, count)| SubmitRecord { date, count })
        .collect();

    Ok(records)
}
