// Copyright (c) 2022 Yuichi Ishida

use anyhow::Result;
use order_in_yaml_frontmatter::cli::Cli;

fn main() -> Result<()> {
    Cli::run()
}
