use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use http_body_util::Empty;
use hyper::body::Bytes;
use futures::executor::block_on;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, ListState, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use color_eyre::{
    eyre::{WrapErr},
    Result,
};
use std::io;
use crate::httpclient::CustomHTTPClient;

#[derive(Debug, Default)]
pub struct App<'a> {
    pub title: &'a str,
    pub server_files: StatefulList<&'a str>,
    pub client: CustomHTTPClient,
    pub exit: bool,
}

#[derive(Debug, Default)]
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<'a> App<'a> {
    pub fn new(client: CustomHTTPClient) -> Self {
        Self {
            title: "tuifs",
            server_files: StatefulList::default(),
            client,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().wrap_err("handle events failed").unwrap();
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)?
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('u') => self.upload_file(),
            KeyCode::Char('d') => self.download_file(),
            KeyCode::Char('s') => self.set_server_location(),
            KeyCode::Char('g') => self.get_server_files(),
            _ => {}
        }
        Ok(())
    }

    fn upload_file(&mut self) {
        println!("uploading file");
    }

    fn download_file(&mut self) {
        println!("downloading file");
    }

    fn set_server_location(&mut self) {
        println!("setting server location");
    }

    fn get_server_files(&mut self) {
        let uri = format!("http://{}/getfiles", self.client.address);
        let req = hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Empty::<Bytes>::new()).unwrap();

        let mut res = block_on(self.client.send_request(req));
        println!("getting server files");
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl<'a> Widget for &App<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" This will eventually be a tuifs B)".bold());
        let instructions = Line::from(vec![
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        Paragraph::new("Welcome to tuifs")
            .centered()
            .block(block)
            .render(area, buf);
    }
}

