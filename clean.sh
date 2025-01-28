#!/bin/zsh

# skip README.md
rm *.jsonl
find . -type f -name "*.md" ! -name "README.md" -exec rm -f {} +
