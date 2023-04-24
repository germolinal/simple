
cargo test --verbose --workspace 
cargo test --features parallel --verbose --workspace 

cargo check --features float --verbose --workspace 
cargo check --features float --features parallel --verbose --workspace 