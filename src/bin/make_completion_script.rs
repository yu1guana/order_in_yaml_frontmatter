// Copyright (c) 2023 Yuichi Ishida
//
// Released under the MIT license.
// see https://opensource.org/licenses/mit-license.php

use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use order_in_yaml_frontmatter::cli::Cli;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

#[derive(Parser)]
#[clap(
    name = "Make completion script",
    author = env!("CARGO_PKG_AUTHORS"),
    version = "",
    about = "Make shellscript to complete arguments."
    )]
struct AppArg {
    shell: Shell,
}

fn main() -> Result<()> {
    let arg = AppArg::parse();
    let mut app = Cli::command();
    let name = app.get_name().to_owned();
    let script_file_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("completion_script")
        .join(concat!(env!("CARGO_PKG_NAME"), "-completion.").to_owned() + &arg.shell.to_string());

    let mut writer = BufWriter::new(File::create(&script_file_path)?);
    generate(arg.shell, &mut app, name, &mut writer);
    println!("Successfully done.");
    println!(
        "A completion script is created (the file path is `{}`).",
        script_file_path.display()
    );
    match arg.shell {
        Shell::Bash => println!("Please read the sciprt using `source` command."),
        Shell::Zsh => println!("Please create a link of the sciprt into a path assigned by `fpath`, which is an environment variable."),
        _ => {}
    }

    Ok(())
}
