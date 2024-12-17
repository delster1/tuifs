use crate::httpclient::CustomHTTPClient;
use crate::statefullist::StatefulList;
use crate::ui::ui;
use color_eyre::{eyre::WrapErr, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use futures::executor::block_on;
use http_body_util::Empty;
use hyper::body::Bytes;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, ListState, Paragraph, StatefulWidget, Widget},
    DefaultTerminal, Frame,
};
use serde_json as serrde_json;
use std::io;
use std::{borrow::BorrowMut, fmt};

pub enum CurrentScreen {
    Start,
    ServerFiles,
    Uploading,
    Downloading,
    Configuring,
}

pub enum CurrentlyConfiguring {
    DownloadLocation,
    ServerLocation,
    UploadLocation,
}

impl Default for CurrentScreen {
    fn default() -> Self {
        CurrentScreen::Start
    }
}

impl fmt::Debug for CurrentScreen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for CurrentlyConfiguring {
    fn default() -> Self {
        CurrentlyConfiguring::ServerLocation
    }
}

impl fmt::Debug for CurrentlyConfiguring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
#[derive(Debug, Default)]
pub struct App<'a> {
    pub title: &'a str,
    pub input: String,
    pub server_files: StatefulList<String>,
    pub client: Option<CustomHTTPClient>,
    pub exit: bool,
    pub current_screen: CurrentScreen,
    pub currently_configuring: Option<CurrentlyConfiguring>,
    download_location: String,
}

impl<'a> App<'a> {
    pub fn new(client: Option<CustomHTTPClient>) -> Self {
        Self {
            title: "tuifs",
            input: String::new(),
            server_files: StatefulList::default(),
            client,
            exit: false,
            current_screen: CurrentScreen::Start,
            currently_configuring: None,
            download_location: String::new(),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        if self.client.is_some() {
            self.get_server_files();
        } else {
            self.current_screen = CurrentScreen::Configuring;
            self.currently_configuring = Some(CurrentlyConfiguring::ServerLocation);
        }
        while !self.exit {
            terminal.draw(|frame| ui(frame, self.borrow_mut()))?;
            if let Event::Key(key_event) = event::read()? {
                self.handle_key_event(key_event).unwrap();
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
        match self.current_screen {
            CurrentScreen::Start => self.handle_start_screen(key_event)?,
            CurrentScreen::ServerFiles => self.handle_server_files_screen(key_event)?,
            CurrentScreen::Uploading => self.handle_uploading_screen(key_event)?,
            CurrentScreen::Downloading => self.handle_downloading_screen(key_event)?,
            CurrentScreen::Configuring => self.handle_configuring_screen(key_event)?,
        }

        Ok(())
    }

    fn handle_uploading_screen(&mut self, key_event: KeyEvent) -> Result<()> {
        
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Esc => {
                self.current_screen = CurrentScreen::Start;
            }
            KeyCode::Char('c') => {
                self.current_screen = CurrentScreen::Configuring;
                self.set_server_location();
            }
            _ => {}
        }
        Ok(())
    }

    fn change_download_location(&mut self, location: String) {
        self.download_location = location;
    }
    fn handle_configuring_screen(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Esc => {
                self.input = String::new();
                self.current_screen = CurrentScreen::Start;
            }
            KeyCode::Char('\n') | KeyCode::Enter => {
                if let Some(editing) = &self.currently_configuring {
                    match editing {
                        CurrentlyConfiguring::DownloadLocation => {
                            self.change_download_location(self.input.clone());
                            self.currently_configuring = None;
                            self.input = String::new();
                            self.current_screen = CurrentScreen::ServerFiles;
                        }
                        CurrentlyConfiguring::ServerLocation => {
                            self.client = Some(block_on(CustomHTTPClient::new(&self.input)).unwrap());
                            self.currently_configuring = None;
                            self.input = String::new();
                            self.get_server_files();
                            self.current_screen = CurrentScreen::ServerFiles;
                        }
                        CurrentlyConfiguring::UploadLocation => {
                            self.currently_configuring = None;
                            self.input = String::new();
                            self.current_screen = CurrentScreen::ServerFiles;
                        }
                    }
                } else {
                    self.current_screen = CurrentScreen::ServerFiles;
                }
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Char(c) => {
                self.input.push(c);
            }
           _ => {}
        }
        Ok(())
    }

    fn handle_downloading_screen(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Esc => {
                self.current_screen = CurrentScreen::Start;
            }
            KeyCode::Char('c') => {
                self.current_screen = CurrentScreen::Configuring;
                self.set_server_location();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_server_files_screen(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('d') => {
                self.current_screen = CurrentScreen::Configuring;
                self.currently_configuring = Some(CurrentlyConfiguring::DownloadLocation);
            }
            KeyCode::Char('u') => {
                self.current_screen = CurrentScreen::Configuring;
                self.currently_configuring = Some(CurrentlyConfiguring::DownloadLocation);
                self.upload_file();
            }
            KeyCode::Char('c') => {
                self.current_screen = CurrentScreen::Configuring;
                self.currently_configuring = Some(CurrentlyConfiguring::ServerLocation);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.server_files.previous();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.server_files.next();
            }
            KeyCode::Esc => {
                self.currently_configuring = None;
                self.current_screen = CurrentScreen::Start;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_start_screen(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('u') => {
                self.current_screen = CurrentScreen::Configuring;
                self.currently_configuring = Some(CurrentlyConfiguring::UploadLocation);
            }
            KeyCode::Char('d') => {
                self.current_screen = CurrentScreen::Configuring;
                self.currently_configuring = Some(CurrentlyConfiguring::DownloadLocation);
            }
            KeyCode::Char('c') => {
                self.current_screen = CurrentScreen::Configuring;
                self.currently_configuring = Some(CurrentlyConfiguring::ServerLocation);
            }
            KeyCode::Char('g') => {
                self.current_screen = CurrentScreen::ServerFiles;
                self.get_server_files();
            }
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

        let uri = format!("http://{}/getfiles", self.client.as_ref().unwrap().address);
        let req = hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Empty::<Bytes>::new())
            .unwrap();

        let response = block_on(self.client.as_mut().unwrap().send_request(req)).unwrap();
        let server_files: Vec<String> = serrde_json::from_slice(&response).unwrap();
        self.server_files.items = server_files;
        self.server_files.state.select(Some(0));
        *self.server_files.state.offset_mut() = 0;

        println!("getting server files");
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
