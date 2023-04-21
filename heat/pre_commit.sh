#/usr/bin/bash
cargo fmt
cargo clippy 2> clippy.txt
cargo hack test --feature-powerset --verbose     
