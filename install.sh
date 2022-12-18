#!/bin/bash

install_dir="$HOME/.cargo"

function lang_ja() {
  if (locale | grep LANG | grep -q ja_JP);then
    return 0
  else
    return 1
  fi
}

if [ $# -eq 1 ]; then
  install_dir=$1
elif [ $# -gt 1 ]; then
  if lang_ja; then
    echo "エラー: 引数は0個または1個です。"
  else
    echo "Error: The number of arguments must be zero or one."
  fi
  exit 1
fi

if lang_ja; then
  echo "$install_dir/bin にインストールされます。"
  echo -n "よろしいですか？ [Y/n] > "
else
  echo "Program will be installed in $install_dir/bin."
  echo -n "Is it OK? [Y/n] > "
fi
read confirm

if [ $confirm = "Y" ]; then
  cargo install --path . --bin order_in_yaml_frontmatter --root $install_dir
else
  if lang_ja; then
    echo "インストールを中止します。"
  else
    echo "Installation is stopped."
  fi
fi

