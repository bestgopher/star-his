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
    return Ok(());
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

        let mut datasets = get_datasets(&data);
        let datas = get_datasets_date(&data);

        let datasets: Vec<Dataset> = datasets
            .into_iter()
            .zip(datas.iter())
            .map(|(mut x, y)| x.data(y))
            .collect();


        let c = Chart::new(datasets)
            .block(Block::default().title("star-history"))
            .x_axis(Axis::default()
                .title(Span::styled("year", Style::default().fg(Color::White)))
                .style(Style::default().fg(Color::White))
                .bounds([0.0, 10.0])
                .labels(["0.0", "5.0", "10.0"].iter().cloned().map(Span::from).collect()))
            .y_axis(Axis::default()
                .title(Span::styled("stars", Style::default().fg(Color::White)))
                .style(Style::default().fg(Color::White))
                .bounds([0.0, 10.0])
                .labels(["0.0", "5.0", "10.0"].iter().cloned().map(Span::from).collect()));

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

/// todo
fn get_datasets_date(data: &[Data]) -> Vec<Vec<(f64, f64)>> {
    vec![]
}