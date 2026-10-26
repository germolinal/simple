#/usr/bin/bash
rm -f ../docs/book/src/auto-*.md ../docs/book/src/IO_SUMMARY.md ../docs/book/src/SUMMARY.md
rm -rf ../target/site
rm -rf ./docs/rustdoc
cargo fmt
cargo clippy 2> clippy.txt
cargo hack check --feature-powerset --verbose     
