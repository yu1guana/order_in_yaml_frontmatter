// Copyright (c) 2023 Yuichi Ishida
//
// Released under the MIT license.
// see https://opensource.org/licenses/mit-license.php

use crate::app::{App, Tui};
use crate::page::PageList;
use anyhow::Result;
use clap::{Parser, ValueHint};
use std::path::PathBuf;

impl Cli {
    pub fn run() -> Result<()> {
        let arg = Cli::parse();
        let mut app = App::new(PageList::try_new(&arg.key, &arg.target_dir, arg.recursive)?);
        let mut tui = Tui::try_new()?;
        tui.run(&mut app)?;
        Ok(())
    }
}

#[derive(Parser)]
#[clap(author, version, about, after_help = concat!("Repository: ", env!("CARGO_PKG_REPOSITORY")))]
pub struct Cli {
    #[clap(long, help = "Variables in frontmatters to assign order")]
    key: String,

    #[clap(
        short = 't',
        long = "target",
        value_hint(ValueHint::FilePath),
        default_value = ".",
        help = "Specify a target directory"
    )]
    target_dir: PathBuf,

    #[clap(short, long, help = "Handles all files under a target directory")]
    recursive: bool,
}
