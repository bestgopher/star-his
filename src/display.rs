use crate::github::Data;
use std::io::{self, Write};
use tui::Terminal;
use tui::backend::CrosstermBackend;
use tui::widgets::{Widget, Block, Borders, *};
use tui::layout::{Layout, Constraint, Direction, *};
use tui::style::{Style, Color, *};
use tui::symbols::{self, *};
use tui::text::*;

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

        let datasets = vec![
            Dataset::default()
                .name("data1")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Scatter)
                .style(Style::default().fg(Color::Cyan))
                .data(&[(0.0, 5.0), (1.0, 6.0), (1.5, 6.434)]),
            Dataset::default()
                .name("data2")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Magenta))
                .data(&[(4.0, 5.0), (5.0, 8.0), (7.66, 13.5)]),
        ];

        let c = Chart::new(datasets)
            .block(Block::default().title("Chart"))
            .x_axis(Axis::default()
                .title(Span::styled("X Axis", Style::default().fg(Color::Red)))
                .style(Style::default().fg(Color::White))
                .bounds([0.0, 10.0])
                .labels(["0.0", "5.0", "10.0"].iter().cloned().map(Span::from).collect()))
            .y_axis(Axis::default()
                .title(Span::styled("Y Axis", Style::default().fg(Color::Red)))
                .style(Style::default().fg(Color::White))
                .bounds([0.0, 10.0])
                .labels(["0.0", "5.0", "10.0"].iter().cloned().map(Span::from).collect()));

        f.render_widget(c, chunk);
    })?;
    Ok(())
}