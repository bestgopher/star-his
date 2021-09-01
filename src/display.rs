use crate::github::Data;
use std::io::{self, Write};
use tui::Terminal;
use tui::backend::CrosstermBackend;
use tui::widgets::{Widget, Block, Borders, *};
use tui::layout::{Layout, Constraint, Direction, *};
use tui::style::{Style, Color, *};
use tui::symbols::{self, *};
use tui::text::*;
use chrono::{DateTime, Utc, Datelike};

pub(crate) fn display(data: Vec<Data>) -> Result<(), io::Error> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                vec![Constraint::Percentage(100)].as_ref()
            )
            .split(f.size());

        let datasets = get_datasets(&data);
        let datas = get_datasets_date(&data);

        let datasets: Vec<Dataset> = datasets
            .into_iter()
            .zip(datas.iter())
            .map(|(x, y)| x.data(y))
            .collect();

        let c = Chart::new(datasets)
            .block(Block::default().title("star-history"))
            .x_axis(Axis::default()
                .title(Span::styled("year", Style::default().fg(Color::White)))
                .style(Style::default().fg(Color::White))
                .bounds([0.0, 10.0])
                .labels(get_x_label(&data).iter().cloned().map(Span::from).collect()))
            .y_axis(Axis::default()
                .title(Span::styled("stars", Style::default().fg(Color::White)))
                .style(Style::default().fg(Color::White))
                .bounds([0.0, 10.0])
                .labels(get_y_label(&data).iter().cloned().map(Span::from).collect()));

        f.render_widget(c, chunks[0]);
    })?;
    Ok(())
}

fn get_datasets(data: &[Data]) -> Vec<Dataset> {
    data
        .into_iter()
        .enumerate()
        .map(|(index, x)| {
            Dataset::default()
                .name(&x.repo)
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(get_datasets_color(index)))
        })
        .collect()
}

fn get_datasets_color(index: usize) -> Color {
    match index {
        0 => Color::Red,
        1 => Color::Green,
        2 => Color::Yellow,
        3 => Color::Blue,
        4 => Color::Magenta,
        _ => panic!("to many repos")
    }
}

fn get_datasets_date(datas: &[Data]) -> Vec<Vec<(f64, f64)>> {
    datas
        .into_iter()
        .map(|data| {
            let mut d = data.data
                .iter()
                .map(|x| {
                    let mut index = x.data.len() - 1;
                    for i in 1..x.data.len() {
                        if x.data[i].starred_at.date().day() != x.data[i - 1].starred_at.date().day() {
                            index = i - 1;
                            break;
                        }
                    }

                    (time_to_year(&x.data[index].starred_at), ((x.page - 1 * 100) as f64 + index as f64))
                })
                .collect::<Vec<(f64, f64)>>();

            d.push((time_to_year(&Utc::now()), data.current_num as f64));
            d
        })
        .collect::<Vec<Vec<(f64, f64)>>>()
}

fn is_leap_year(year: i32) -> bool {
    match (year % 4, year % 100, year % 400) {
        (0, 0, 0) => true,
        (0, 0, _) => false,
        (0, _, _) => true,
        (_, _, _) => false,
    }
}

fn time_to_year(t: &DateTime<Utc>) -> f64 {
    let is_leap = is_leap_year(t.date().year());

    t.date().year() as f64 +
        (month_day(is_leap, t.date().month())
            + t.date().day()) as f64 / if is_leap { 366f64 } else { 365f64 }
}

fn month_day(is_leap: bool, month: u32) -> u32 {
    match month {
        1 => 0,
        2 => month_day(is_leap, 1) + 31,
        3 => month_day(is_leap, 2) + if is_leap { 29 } else { 28 },
        4 => month_day(is_leap, 3) + 31,
        5 => month_day(is_leap, 4) + 30,
        6 => month_day(is_leap, 5) + 31,
        7 => month_day(is_leap, 6) + 30,
        8 => month_day(is_leap, 7) + 31,
        9 => month_day(is_leap, 8) + 31,
        10 => month_day(is_leap, 9) + 30,
        11 => month_day(is_leap, 10) + 31,
        12 => month_day(is_leap, 11) + 30,
        _ => panic!("invalid month")
    }
}

fn get_x_label(datas: &[Data]) -> Vec<String> {
    let now_year = Utc::now().date().year();

    let min_year = datas
        .iter()
        .map(|x| x.data[0].data[0].starred_at.date().year())
        .min()
        .unwrap_or(now_year);

    (min_year..=now_year).into_iter().map(|x| x.to_string()).collect()
}

fn get_y_label(datas: &[Data]) -> Vec<String> {
    let min_star = 0;

    let max_star = datas
        .iter()
        .map(|x| x.current_num)
        .max()
        .unwrap_or(0);

    (min_star..=max_star).into_iter().map(|x| x.to_string()).collect()
}
