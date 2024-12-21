/// ui.rs - all logic for ui display - dependent exclusively on app state
/// contains
/// - widget rendering logic
///

///
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::rc::Rc;

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
    // setting up base layout and boxes
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

    let help_text = vec![match app.current_screen {
        CurrentScreen::Start => Line::from(vec![Span::styled(
            "Press 'g' to get server files, 'u' to upload files, 'c' to configure server",
            Style::default().fg(Color::Yellow),
        )]),
        CurrentScreen::Configuring => Line::from(vec![Span::styled(
            "Press 'Enter' to submit your input, 'Esc' to cancel",
            Style::default().fg(Color::Yellow),
        )]),
        CurrentScreen::ServerFiles => Line::from(vec![Span::styled(
            "Press 'd' or 'Enter' to download the current file, 'Esc' to go back",
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

    // basic screen setup done - building app-specific ui now

    let server_files = app
        .server_files
        .items
        .iter()
        .map(|item| ListItem::new(Span::styled(item, Style::default().fg(Color::White))));

    let server_files = List::new(server_files)
        .block(Block::default().title("Server Files").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::LightBlue).fg(Color::Black))
        .highlight_symbol(">>");

    frame.render_stateful_widget(server_files, main_chunks[0], &mut app.server_files.state); // always
    // render server files in background of other screens

    let area = centered_rect(60, 40, frame.area());
    let popup_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50)])
        .split(area);

    // render app based on screen
    match app.current_screen {
        CurrentScreen::Start => {
            render_start_screen(frame, app, popup_chunks);
        }

        CurrentScreen::Configuring => {
            render_config_screen(&app.currently_configuring, frame, app, popup_chunks);
        }
        CurrentScreen::Downloading => {
            render_download_screen(frame, app, popup_chunks);
        }
        CurrentScreen::Uploading => {
            render_upload_screen(frame, app, popup_chunks);
        }
        _ => {}
    };
}

fn render_start_screen(frame: &mut Frame, app: &App, popup_chunks: Rc<[Rect]>) {
    let popup = Block::default()
        .title("Please Choose An Option:")
        .borders(Borders::ALL);
    let serverlocation = &app.client;
    let serverlocation = match serverlocation {
        Some(client) => client.address.clone(),
        None => "No Server Configured".to_string(),
    };
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
        Line::from(vec![Span::styled(
            " 4. Exit (q)",
            Style::default().fg(Color::Red),
        )]),
        Line::from(vec![Span::styled(
            format!("Current Server Location: {}", serverlocation),
            Style::default().fg(Color::White),
        )]),
    ];
    let input: Paragraph = Paragraph::new(Text::from(popup_text))
        .style(Style::default().fg(Color::White))
        .block(popup);
    frame.render_widget(input, popup_chunks[0]);
}

fn render_config_screen(
    config: &Option<CurrentlyConfiguring>,
    frame: &mut Frame,
    app: &App,
    popup_chunks: Rc<[Rect]>,
) {
    if let Some(current_config) = config {
        match current_config {
            CurrentlyConfiguring::DownloadLocation => {
                let popup = Block::default()
                    .title("Please Enter Where You'd Like Downloaded FilesTo Go:")
                    .borders(Borders::ALL);
                let popup_input_window = Paragraph::new(app.input.clone())
                    .style(Style::default().fg(Color::White))
                    .block(popup)
                    .wrap(Wrap { trim: true });
                frame.render_widget(popup_input_window, popup_chunks[0]);
            }
            CurrentlyConfiguring::ServerLocation => {
                let popup = Block::default()
                    .title("Please Enter Server Location:")
                    .borders(Borders::ALL);
                let popup_input_window = Paragraph::new(app.input.clone())
                    .style(Style::default().fg(Color::White))
                    .block(popup)
                    .wrap(Wrap { trim: true });
                frame.render_widget(popup_input_window, popup_chunks[0]);
            }
            CurrentlyConfiguring::UploadLocation => {
                let popup = Block::default()
                    .title("Please Enter The File Path Of What You're Uploading:")
                    .borders(Borders::ALL);

                let popup_input_window = Paragraph::new(app.input.clone())
                    .style(Style::default().fg(Color::White))
                    .block(popup)
                    .wrap(Wrap { trim: true });
                frame.render_widget(popup_input_window, popup_chunks[0]);
            }
        }
    }
}

fn render_download_screen(frame: &mut Frame, app: &App, popup_chunks: Rc<[Rect]>) {
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

fn render_upload_screen(frame: &mut Frame, app: &App, popup_chunks: Rc<[Rect]>) {
    let popup = Block::default()
        .title("Uploading Files")
        .borders(Borders::ALL);
    let popup_text_window = Paragraph::new(app.input.clone())
        .wrap(Wrap { trim: true })
        .block(popup);
    frame.render_widget(popup_text_window, popup_chunks[0]);
}
