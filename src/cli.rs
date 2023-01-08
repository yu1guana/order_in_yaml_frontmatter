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
    /// ディレクトリを指定
    #[clap(
        short = 't',
        long = "target",
        value_hint(ValueHint::FilePath),
        default_value = "."
    )]
    target_dir: PathBuf,

    /// ディレクトリを再起的に探る
    #[clap(short, long)]
    recursive: bool,

    /// 順序を割り当てるFronMatterの変数
    #[clap(long)]
    key: String,
}
