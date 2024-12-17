use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, CurrentScreen, CurrentlyConfiguring};

pub fn ui(frame : &mut Frame, app: &mut App){
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(frame.area());

    let title = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            app.title,
            Style::default().fg(Color::LightBlue),
        ));
    frame.render_widget(title, chunks[0]);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Min(50)].as_ref())
        .split(chunks[1]);

    let left = Block::default()
        .title("Server Files")
        .borders(Borders::ALL);
    frame.render_widget(left, main_chunks[0]);

    let right = Block::default()
        .title("Client Files")
        .borders(Borders::ALL);
    frame.render_widget(right, main_chunks[1]);

    let server_files = app.server_files.items.iter().map(|item| {
        ListItem::new(Span::styled(
            item,
            Style::default().fg(Color::White),
        ))
    });

    let server_files = List::new(server_files)
        .block(Block::default().title("Server Files").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::LightBlue).fg(Color::Black))
        .highlight_symbol(">>");

    frame.render_stateful_widget(server_files, main_chunks[0],  &mut app.server_files.state);


}
