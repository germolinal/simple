#/usr/bin/bash
rm ./docs/ioreference/src/auto*.md
rm -rf ./docs/ioreference/book
rm -rf ./docs/rustdoc
cargo fmt
cargo clippy 2> clippy.txt
cargo hack check --feature-powerset --verbose     
