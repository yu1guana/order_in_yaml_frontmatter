// Copyright (c) 2022 Yuichi Ishida

use crate::key_bind;
use crate::page::{PageList, SwapDirection};
use anyhow::Result;
use std::cmp;
use std::fmt::Write as _;
use std::io;
use std::io::Stdout;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::{AlternateScreen, IntoAlternateScreen};
use tui::backend::{Backend, TermionBackend};
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Modifier, Style};
use tui::terminal::{Frame, Terminal};
use tui::widgets::{Block, Paragraph, Row, Table, TableState};
use unicode_width::UnicodeWidthStr;

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

pub struct App {
    page_list: PageList,
    selected_idx: usize,
    current_status: Status,
    previous_status: Status,
}

impl Tui<TermionBackend<AlternateScreen<RawTerminal<Stdout>>>> {
    pub fn try_new() -> Result<Self> {
        let stdout = io::stdout().into_raw_mode()?.into_alternate_screen()?;
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
    const UP_KEY: Key = Key::Char(key_bind::UP);
    const DOWN_KEY: Key = Key::Char(key_bind::DOWN);
    const PICK_TOGGLE_KEY: Key = Key::Char(key_bind::PICK_TOGGLE);
    const INCLUDE_TOGGLE_KEY: Key = Key::Char(key_bind::INCLUDE_TOGGLE);
    const QUIT_KEY: Key = Key::Char(key_bind::QUIT);
    const SAVE_KEY: Key = Key::Char(key_bind::SAVE);

    pub fn new(page_list: PageList) -> Self {
        Self {
            page_list,
            selected_idx: 0,
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

    fn ui<B: Backend>(&mut self, frame: &mut Frame<B>) {
        match self.current_status {
            Status::Unpicked => {
                self.ui_select(frame, false);
            }
            Status::Picked => {
                self.ui_select(frame, true);
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
            Self::SAVE_KEY => self.update_status(Status::AskSave),
            Self::UP_KEY | Key::Up => {
                self.update_status(Status::Unpicked);
                if self.selected_idx != 0 {
                    self.selected_idx -= 1;
                }
            }
            Self::DOWN_KEY | Key::Down => {
                self.update_status(Status::Unpicked);
                if self.selected_idx != self.page_list.len() - 1 {
                    self.selected_idx += 1;
                }
            }
            Self::INCLUDE_TOGGLE_KEY => {
                self.page_list.toggle_value(self.selected_idx)?;
            }
            Self::PICK_TOGGLE_KEY => {
                self.update_status(Status::Picked);
            }
            _ => (),
        }
        Ok(())
    }

    fn picked(&mut self, key: Key) -> Result<()> {
        match key {
            Self::QUIT_KEY => self.update_status(Status::AskQuit),
            Self::UP_KEY | Key::Up => {
                self.update_status(Status::Picked);
                if self.selected_idx != 0 {
                    self.page_list
                        .swap_with_value(self.selected_idx, SwapDirection::Prev)?;
                    self.selected_idx -= 1;
                }
            }
            Self::DOWN_KEY | Key::Down => {
                self.update_status(Status::Picked);
                if self.selected_idx != self.page_list.len() - 1 {
                    self.page_list
                        .swap_with_value(self.selected_idx, SwapDirection::Next)?;
                    self.selected_idx += 1;
                }
            }
            Self::PICK_TOGGLE_KEY => {
                self.update_status(Status::Unpicked);
            }
            _ => (),
        }
        Ok(())
    }

    fn ask_quit(&mut self, key: Key) {
        match key {
            Key::Char('Y') => self.update_status(Status::Quit),
            _ => self.update_status(self.previous_status),
        }
    }

    fn ask_save(&mut self, key: Key) -> Result<()> {
        match key {
            Key::Char('Y') => {
                self.page_list.substitute_value();
                self.page_list.overwrite_frontmatter()?;
                self.update_status(Status::Quit);
            }
            _ => self.update_status(self.previous_status),
        }
        Ok(())
    }

    fn ui_select<B: Backend>(&self, frame: &mut Frame<B>, picked: bool) {
        let guidance = self.guidance(picked);
        let guidance_height = 1 + (guidance.len() / frame.size().width as usize) as u16;
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(guidance_height),
                    Constraint::Length(1),
                    Constraint::Length(frame.size().height - guidance_height - 1),
                ]
                .as_ref(),
            )
            .split(frame.size());
        frame.render_widget(Paragraph::new(guidance).block(Block::default()), chunks[0]);
        let max_file_name_length = self
            .page_list
            .iter()
            .map(|page| page.path().file_name().unwrap().to_str().unwrap().len())
            .max()
            .unwrap() as u16;
        let max_dir_name_length = self
            .page_list
            .iter()
            .map(|page| page.path().parent().unwrap().to_str().unwrap().len())
            .max()
            .unwrap() as u16;
        let max_title_name_length = self
            .page_list
            .iter()
            .map(|page| {
                if let Some(title) = page.title() {
                    title.width_cjk()
                } else {
                    0
                }
            })
            .max()
            .unwrap() as u16;
        let rows = self.page_list.iter().map(|page| {
            Row::new(vec![
                if let Some(title) = page.title() {
                    title
                } else {
                    ""
                },
                if page.value().is_none() { "x" } else { "" },
                page.path().file_name().unwrap().to_str().unwrap(),
                page.path().parent().unwrap().to_str().unwrap(),
            ])
        });
        let header_list = vec!["Title", "", "File", "Dirctory"];
        let widths = vec![
            Constraint::Length(cmp::max(
                max_title_name_length,
                header_list.get(0).unwrap().len() as u16,
            )),
            Constraint::Length(1),
            Constraint::Length(cmp::max(
                max_file_name_length,
                header_list.get(2).unwrap().len() as u16,
            )),
            Constraint::Length(cmp::max(
                max_dir_name_length,
                header_list.get(3).unwrap().len() as u16,
            )),
        ];
        let table = Table::new(rows)
            .widths(&widths)
            .header(
                Row::new(header_list).style(Style::default().add_modifier(Modifier::UNDERLINED)),
            )
            .column_spacing(2)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(if picked { " >> " } else { " >  " });
        let mut table_state = TableState::default();
        table_state.select(Some(self.selected_idx));
        frame.render_stateful_widget(table.block(Block::default()), chunks[2], &mut table_state);
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
        let chunks = Layout::default()
            .margin(1)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Length(3),
                Constraint::Percentage(50),
            ])
            .split(frame.size());
        let title = Paragraph::new("Save and quit?")
            .alignment(Alignment::Center)
            .block(Block::default());
        frame.render_widget(title, chunks[1]);
        let opening_msg = Paragraph::new("Y / [n]")
            .alignment(Alignment::Center)
            .block(Block::default());
        frame.render_widget(opening_msg, chunks[2]);
    }

    fn guidance(&self, picked: bool) -> String {
        let mut guidance = String::new();
        write!(guidance, " Quit [{}]", key_bind::QUIT).unwrap();
        write!(guidance, ", Up [{}]", key_bind::UP).unwrap();
        write!(guidance, ", Down [{}]", key_bind::DOWN).unwrap();
        if picked {
            write!(guidance, ", Unpick [{}]", key_bind::PICK_TOGGLE).unwrap();
        } else {
            write!(guidance, ", Pick [{}]", key_bind::PICK_TOGGLE).unwrap();
            if self
                .page_list
                .get(self.selected_idx)
                .unwrap()
                .value()
                .is_some()
            {
                write!(guidance, ", Exclude [{}]", key_bind::INCLUDE_TOGGLE).unwrap();
            } else {
                write!(guidance, ", Include [{}]", key_bind::INCLUDE_TOGGLE).unwrap();
            }
        }
        if !picked {
            write!(guidance, ", Save [{}]", key_bind::SAVE).unwrap();
        }
        guidance
    }
}
