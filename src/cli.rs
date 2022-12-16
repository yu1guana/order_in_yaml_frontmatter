// Copyright (c) 2022 Yuichi Ishida

use crate::page::{PageList, SwapDirection};
use anyhow::Result;
use clap::{Parser, ValueHint};
use std::path::PathBuf;
use yaml_rust::YamlEmitter;

impl Cli {
    pub fn run() -> Result<()> {
        let arg = Cli::parse();
        let mut page_list = PageList::try_new(&arg.key, &arg.target_dir, arg.recursive)?;
        for page in page_list.iter() {
            println!("{:?}", page);
        }
        println!();
        page_list.unset_value(0)?;
        page_list.set_value(5)?;
        for page in page_list.iter() {
            println!("{:?}", page);
        }
        println!();
        page_list.swap_with_value(1, SwapDirection::Prev)?;
        for page in page_list.iter() {
            println!("{:?}", page);
        }
        println!();
        let mut out_str = String::new();
        let mut emitter = YamlEmitter::new(&mut out_str);
        page_list.substitute_value();
        emitter.dump(page_list.first().unwrap().yaml())?;
        println!("{}", out_str);
        Ok(())
    }
}

#[derive(Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    about = "Assign sequential variables in Jekyll frontmatter."
)]
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
