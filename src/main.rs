// Copyright (c) 2023 Yuichi Ishida
//
// Released under the MIT license.
// see https://opensource.org/licenses/mit-license.php

use anyhow::Result;
use order_in_yaml_frontmatter::cli::Cli;

fn main() -> Result<()> {
    Cli::run()
}
