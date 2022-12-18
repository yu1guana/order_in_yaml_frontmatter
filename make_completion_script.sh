#!/bin/bash

if [ $# -ne 1 ]; then
  echo "Error: One argument is necessary. Please input a shell name."
  exit 1
fi

cargo run --bin make_completion_script -- $1
