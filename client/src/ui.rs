use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, CurrentScreen, CurrentlyConfiguring};

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

pub fn ui(frame: &mut Frame, app: &mut App) {
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

    let title = Block::default().borders(Borders::ALL).title(Span::styled(
        app.title,
        Style::default().fg(Color::LightBlue),
    ));
    frame.render_widget(title, chunks[0]);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(50)].as_ref())
        .split(chunks[1]);

    let main_block = Block::default().title("Server Files").borders(Borders::ALL);
    frame.render_widget(main_block, main_chunks[0]);

    let help_text = vec![
        match app.current_screen {
            CurrentScreen::Start => Line::from(vec![Span::styled(
                "Press 'g' to get server files, 'u' to upload files, 'c' to configure server",
                Style::default().fg(Color::Yellow),
            )]),
            CurrentScreen::Configuring => Line::from(vec![Span::styled(
                "Press 'Enter' to submit your input, 'Esc' to cancel",
                Style::default().fg(Color::Yellow),
            )]),
            CurrentScreen::ServerFiles => Line::from(vec![Span::styled(
                "Press 'd' or 'Enter' to download the current file",
                Style::default().fg(Color::Yellow),
            )]),
            _ => Line::from(vec![Span::styled(
                "Press 'g' to get server files, 'u' to upload files, 'c' to configure server",
                Style::default().fg(Color::Yellow),
            )]),
        }];

    let help_box = Paragraph::new(Text::from(help_text))
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("Help"));
    frame.render_widget(help_box, chunks[2]);

    let server_files = app
        .server_files
        .items
        .iter()
        .map(|item| ListItem::new(Span::styled(item, Style::default().fg(Color::White))));

    let server_files = List::new(server_files)
        .block(Block::default().title("Server Files").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::LightBlue).fg(Color::Black))
        .highlight_symbol(">>");

    frame.render_stateful_widget(server_files, main_chunks[0], &mut app.server_files.state);

    let area = centered_rect(100, 60, frame.area());
    let popup_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50)])
        .split(area);
    match app.current_screen {
        CurrentScreen::Start => {
            let popup = Block::default()
                .title("Please Choose An Option:")
                .borders(Borders::ALL);
            let popup_text = vec![
                Line::from(vec![Span::raw("1. Download/View Server Files (g)")]),
                Line::from(vec![Span::styled(
                    " 2. Upload Files (u)",
                    Style::default().fg(Color::Yellow),
                )]),
                Line::from(vec![Span::styled(
                    " 3. Configure Server (c)",
                    Style::default().fg(Color::Yellow),
                )]),
            ];
            let input: Paragraph = Paragraph::new(Text::from(popup_text))
                .style(Style::default().fg(Color::White))
                .block(popup);
            frame.render_widget(input, popup_chunks[0]);
        }

        CurrentScreen::Configuring => {
            if let Some(config) = &app.currently_configuring {
                match config {
                    CurrentlyConfiguring::DownloadLocation => {
                        let popup = Block::default()
                            .title("Please Enter Where You'd Like Downloaded FilesTo Go:")
                            .borders(Borders::ALL);
                        let popup_input_window = Paragraph::new(app.input.clone())
                            .style(Style::default().fg(Color::White))
                            .block(popup);
                        frame.render_widget(popup_input_window, popup_chunks[0]);
                    }
                    CurrentlyConfiguring::ServerLocation => {
                        let popup = Block::default()
                            .title("Please Enter Server Location:")
                            .borders(Borders::ALL);
                        let popup_input_window = Paragraph::new(app.input.clone())
                            .style(Style::default().fg(Color::White))
                            .block(popup);
                        frame.render_widget(popup_input_window, popup_chunks[0]);
                    }
                    CurrentlyConfiguring::UploadLocation => {
                        let popup = Block::default()
                            .title("Please Enter The File Path Of What You're Uploading:")
                            .borders(Borders::ALL);
                        let popup_input_window = Paragraph::new(app.input.clone())
                            .style(Style::default().fg(Color::White))
                            .block(popup);
                        frame.render_widget(popup_input_window, popup_chunks[0]);
                    }
                }

            }
        }
        CurrentScreen::Downloading => {
            let popup = Block::default()
                .title("Downloading Files")
                .borders(Borders::ALL);
            let popup_text = vec![Line::from(vec![Span::raw("Downloading Files")])];
            let popup_text = Text::from(popup_text);
            let popup_text_window = Paragraph::new(popup_text)
                .wrap(Wrap { trim: true })
                .block(popup);
            frame.render_widget(popup_text_window, popup_chunks[0]);
        }
        CurrentScreen::Uploading => {
            let popup = Block::default().title("Uploading Files").borders(Borders::ALL);
            let popup_text_window = Paragraph::new(app.input.clone())
                .wrap(Wrap { trim: true })
                .block(popup);
            frame.render_widget(popup_text_window, popup_chunks[0]);
        }
        _ => {}
    };
}
