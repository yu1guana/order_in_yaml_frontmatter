// Copyright (c) 2022 Yuichi Ishida

use crate::page::PageList;
use anyhow::Result;
use derive_new::new;
use std::fmt::Write as _;
use std::io;
use std::io::Stdout;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use tui::backend::{Backend, TermionBackend};
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::terminal::Frame;
use tui::terminal::Terminal;
use tui::widgets::{Block, Borders, Clear, Paragraph};

#[derive(Debug)]
pub struct Tui<B: Backend> {
    terminal: Terminal<B>,
}

#[derive(Clone, Copy, Debug, Default)]
enum Status {
    #[default]
    Unpicked,
    Picked,
    AskQuit,
    AskSave,
    Quit,
}

#[derive(Debug)]
pub struct App {
    page_list: PageList,
    guidance: String,
    current_status: Status,
    previous_status: Status,
}

impl Tui<TermionBackend<RawTerminal<Stdout>>> {
    pub fn try_new() -> Result<Self> {
        let stdout = io::stdout().into_raw_mode()?;
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;
        Ok(Self { terminal })
    }
    pub fn run(&mut self, app: &mut App) -> Result<()> {
        self.terminal.draw(|frame| app.ui(frame))?;
        while let Some(Ok(key)) = io::stdin().keys().next() {
            app.transition(key)?;
            if let Status::Quit = app.current_status {
                break;
            } else {
                self.terminal.draw(|frame| app.ui(frame))?;
            }
        }
        Ok(())
    }
}

impl App {
    const UP_STR: char = 'i';
    const DOWN_STR: char = 'k';
    const PICK_STR: char = 'l';
    const UNPICK_STR: char = 'j';
    const INCLUDE_TOGGLE_STR: char = 'x';
    const QUIT_STR: char = 'q';
    const SAVE_STR: char = 's';

    const UP_KEY: Key = Key::Char(Self::UP_STR);
    const DOWN_KEY: Key = Key::Char(Self::DOWN_STR);
    const PICK_KEY: Key = Key::Char(Self::PICK_STR);
    const UNPICK_KEY: Key = Key::Char(Self::UNPICK_STR);
    const INCLUDE_TOGGLE_KEY: Key = Key::Char(Self::INCLUDE_TOGGLE_STR);
    const QUIT_KEY: Key = Key::Char(Self::QUIT_STR);
    const SAVE_KEY: Key = Key::Char(Self::SAVE_STR);

    pub fn new(page_list: PageList) -> Self {
        let mut guidance = String::new();
        write!(guidance, "Up [{}] ", Self::UP_STR).unwrap();
        write!(guidance, "Down [{}] ", Self::DOWN_STR).unwrap();
        write!(guidance, "Pick [{}]", Self::PICK_STR).unwrap();
        write!(guidance, "Unpick [{}]", Self::UNPICK_STR).unwrap();
        write!(guidance, "Include/Exclude [{}]", Self::INCLUDE_TOGGLE_STR).unwrap();
        write!(guidance, "Quit [{}]", Self::QUIT_STR).unwrap();
        write!(guidance, "Save [{}]", Self::SAVE_STR).unwrap();
        Self {
            page_list,
            guidance: Default::default(),
            current_status: Default::default(),
            previous_status: Default::default(),
        }
    }

    fn update_status(&mut self, status: Status) {
        self.previous_status = self.current_status;
        self.current_status = status;
    }

    fn transition(&mut self, key: Key) -> Result<()> {
        match &self.current_status {
            Status::Unpicked => {
                self.unpicked(key)?;
            }
            Status::Picked => {
                self.picked(key)?;
            }
            Status::AskQuit => {
                self.ask_quit(key);
            }
            Status::AskSave => {
                self.ask_save(key)?;
            }
            Status::Quit => {
                unreachable!()
            }
        }
        Ok(())
    }

    fn ui<B: Backend>(&self, frame: &mut Frame<B>) {
        match self.current_status {
            Status::Unpicked => {
                self.ui_unpicked(frame);
            }
            Status::Picked => {
                self.ui_picked(frame);
            }
            Status::AskQuit => {
                self.ui_ask_quit(frame);
            }
            Status::AskSave => {
                self.ui_ask_save(frame);
            }
            Status::Quit => {
                unreachable!()
            }
        }
    }

    fn unpicked(&mut self, key: Key) -> Result<()> {
        match key {
            Self::QUIT_KEY => self.update_status(Status::AskQuit),
            _ => (),
        }
        Ok(())
    }

    fn picked(&mut self, key: Key) -> Result<()> {
        unimplemented!()
    }

    fn ask_quit(&mut self, key: Key) {
        match key {
            Key::Char('Y') => self.update_status(Status::Quit),
            _ => self.update_status(self.current_status),
        }
    }

    fn ask_save(&mut self, key: Key) -> Result<()> {
        unimplemented!()
    }

    fn ui_unpicked<B: Backend>(&self, frame: &mut Frame<B>) {
        let guidance_height = (self.guidance.len() / frame.size().width as usize) as u16;
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(10),
                    Constraint::Percentage(90),
                    // Constraint::Length(guidance_height),
                    // Constraint::Length(frame.size().height - guidance_height),
                ]
                .as_ref(),
            )
            .split(frame.size());
        frame.render_widget(
            Paragraph::new(self.guidance.as_str()).block(Block::default()),
            chunks[0],
        );
        frame.render_widget(
            Paragraph::new("Unimplemented.").block(Block::default()),
            chunks[1],
        );
    }

    fn ui_picked<B: Backend>(&self, frame: &mut Frame<B>) {
        unimplemented!()
    }

    fn ui_ask_quit<B: Backend>(&self, frame: &mut Frame<B>) {
        let chunks = Layout::default()
            .margin(1)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Length(3),
                Constraint::Percentage(50),
            ])
            .split(frame.size());
        let title = Paragraph::new("Quit without save?")
            .alignment(Alignment::Center)
            .block(Block::default());
        frame.render_widget(title, chunks[1]);
        let opening_msg = Paragraph::new("Y / [n]")
            .alignment(Alignment::Center)
            .block(Block::default());
        frame.render_widget(opening_msg, chunks[2]);
    }

    fn ui_ask_save<B: Backend>(&self, frame: &mut Frame<B>) {
        unimplemented!()
    }
}
