use chrono::{Datelike, Duration, NaiveDate, Utc, Weekday};
use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    DefaultTerminal,
};
use std::io;

const WEEK_DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

// Submit information for each day
#[derive(Debug)]
struct SubmitRecord {
    date: NaiveDate,
    count: u32,
}

// 生成过去 6 个月的日期和随机提交数据
fn generate_sumbit_data() -> Vec<SubmitRecord> {
    let end_date = Utc::now().date_naive();
    let start_date = end_date - Duration::days(180);

    let mut sumbit_data = Vec::new();
    let mut date = start_date;
    while date <= end_date {
        let sumbit_count = date.day() % 10;
        sumbit_data.push(SubmitRecord {
            date,
            count: sumbit_count,
        });
        date = date.succ_opt().unwrap();
    }

    sumbit_data
}

// determine color by q1-q5
fn determine_color_thresholds(data: &[u32]) -> Vec<u32> {
    let mut sorted_data = data.to_vec();
    sorted_data.sort_unstable();

    // quintiles
    let q1 = sorted_data[sorted_data.len() / 5];
    let q2 = sorted_data[2 * sorted_data.len() / 5];
    let q3 = sorted_data[3 * sorted_data.len() / 5];
    let q4 = sorted_data[4 * sorted_data.len() / 5];

    vec![q1, q2, q3, q4]
}

// get color for count with five categories
fn get_color_for_count(count: u32, thresholds: &[u32]) -> Color {
    match count {
        0 => Color::DarkGray,
        c if c <= thresholds[0] => Color::Yellow,
        c if c <= thresholds[1] => Color::LightGreen,
        c if c <= thresholds[2] => Color::Green,
        c if c <= thresholds[3] => Color::Blue,
        _ => Color::Red,
    }
}

pub fn lunch() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(terminal);
    ratatui::restore();
    app_result
}

fn run(mut terminal: DefaultTerminal) -> io::Result<()> {
    let submits = generate_sumbit_data();
    let submit_counts: Vec<u32> = submits.iter().map(|c| c.count).collect();

    // calculate color thresholds
    let color_thresholds = determine_color_thresholds(&submit_counts);

    // find all mondays in the date range
    let mut mondays = vec![];
    let today = Utc::now().date_naive();
    // from 6 months ago to today
    let mut date = today - Duration::days(180);
    while date <= today {
        if date.weekday() == Weekday::Mon {
            mondays.push(date);
        }
        date = date.succ_opt().unwrap();
    }

    loop {
        // get title from years
        let start_year = submits.first().map(|c| c.date.year()).unwrap_or(0);
        let end_year = submits.last().map(|c| c.date.year()).unwrap_or(0);
        let title = if start_year == end_year {
            format!("{} sumbit Heatmap", start_year)
        } else {
            format!("{} - {} sumbit Heatmap", start_year, end_year)
        };
        terminal.draw(|f| {
            // layout vertical
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(90), // table
                    Constraint::Percentage(10), // legend
                ])
                .split(f.area());

            // 5-90-5 in horizontal
            let centered_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(5),  // blank
                    Constraint::Percentage(90), // main table
                    Constraint::Percentage(5),  // blank
                ])
                .split(chunks[0]);

            // legend chunks
            let legend_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(5),  // blank
                    Constraint::Percentage(90), // main table
                    Constraint::Percentage(5),  // blank
                ])
                .split(chunks[1]);

            let rows: Vec<Row> = (0..7)
                .flat_map(|day| {
                    let mut cells: Vec<Cell> = vec![Cell::from(WEEK_DAYS[day].to_string())];
                    cells.extend(mondays.iter().map(|&monday| {
                        let date = monday + Duration::days(day.try_into().unwrap());
                        let sumbit_count = submits
                            .iter()
                            .find(|c| c.date == date)
                            .map_or(0, |c| c.count);

                        let color = get_color_for_count(sumbit_count, &color_thresholds);
                        Cell::from(" ").style(Style::default().bg(color))
                    }));

                    vec![Row::new(cells), Row::new(vec![Cell::from(" ")])]
                })
                .collect();

            let headers = get_header(&mondays, true);

            // set width for each column
            // 5 for weekday text , 10 for each week
            let widths = vec![Constraint::Length(5)]
                .into_iter()
                .chain(mondays.iter().map(|_| Constraint::Length(10)))
                .collect::<Vec<_>>();

            // keep top border for title
            let block = Block::default()
                .title(title)
                .title_alignment(Alignment::Center)
                .style(Style::default().add_modifier(Modifier::BOLD))
                .borders(Borders::TOP);
            // generate table
            let table = Table::new(rows, widths)
                .header(Row::new(headers))
                .block(block);
            // render table
            f.render_widget(table, centered_chunks[1]);

            // render legend
            let legend_table = get_legend(&color_thresholds);
            f.render_widget(legend_table, legend_chunks[1]);
        })?;

        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(());
            }
        }
    }
}

fn get_header(mondays: &Vec<NaiveDate>, full: bool) -> Vec<Cell<'static>> {
    if full {
        let headers: Vec<Cell> = vec![Cell::from(" ")]
            .into_iter()
            .chain(mondays.iter().map(|monday| {
                let month = monday.format("%b").to_string();
                let day = monday.format("%d").to_string();
                Cell::from(format!("{}{}", month, day))
            }))
            .collect();
        return headers;
    }
    let mut headers: Vec<Cell> = vec![Cell::from(" ")];
    let mut last_month = 0;

    for monday in mondays {
        let month = monday.format("%b").to_string();
        let current_month = monday.month();

        // 只显示每月的第一个周一
        if current_month != last_month {
            headers.push(Cell::from(month));
            last_month = current_month;
        } else {
            headers.push(Cell::from(" ")); // 留空
        }
    }
    headers
}

fn get_legend(color_thresholds: &[u32]) -> Table<'static> {
    // add legend
    let legend_text = Row::new(vec![
        Cell::from("    Less    "),
        Cell::from("= 0".to_string()).style(Style::default().bg(Color::DarkGray)),
        Cell::from(format!("<= {}", color_thresholds[0])).style(Style::default().bg(Color::Yellow)),
        Cell::from(format!("<= {}", color_thresholds[1]))
            .style(Style::default().bg(Color::LightGreen)),
        Cell::from(format!("<= {}", color_thresholds[2])).style(Style::default().bg(Color::Green)),
        Cell::from(format!("<= {}", color_thresholds[3])).style(Style::default().bg(Color::Blue)),
        Cell::from(format!("> {}", color_thresholds[3])).style(Style::default().bg(Color::Red)),
        Cell::from("    More    ").style(Style::default().add_modifier(Modifier::BOLD)),
    ]);

    let legend_table = Table::new(
        vec![legend_text],
        vec![
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
        ],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center)
            .title("SUBMIT COUNT"),
    );
    legend_table
}
