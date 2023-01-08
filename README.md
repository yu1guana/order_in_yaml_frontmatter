# OrderInYamlFrontmatter

Assign sequential variables for yaml frontmatters.
This is useful when using static site generators such as Jekyll.

## Installation

Compilation requires the `cargo` command, so if you do not have it, refer to [this page](https://www.rust-lang.org/ja/tools/install).

If bash is running, execute the following command.

```sh
./install.sh
```

When using the above script, the installation directory can be specified as an argument (if no argument is specified, the installation directory is ~/.cargo/bin).

If you do not have bash, execute the following command (please refer to [this site](https://doc.rust-lang.org/cargo/commands/cargo-install.html)).

```sh
cargo install --path .
```

## Completion script

Executing `make_completion_script.sh`, a completion script is created in [completion\_script](completion_script).

## Usage

```
Usage: order_in_yaml_frontmatter [OPTIONS] --key <KEY>

Options:
      --key <KEY>            Variables in frontmatters to assign order
  -t, --target <TARGET_DIR>  Specify a target directory [default: .]
  -r, --recursive            Handles all files under a target directory
  -h, --help                 Print help information
  -V, --version              Print version information
```

## License
Copyright (c) 2023 Yuichi Ishida
Released under the MIT license
[https://opensource.org/licenses/mit-license.php](https://opensource.org/licenses/mit-license.php)
