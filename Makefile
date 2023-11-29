VALIDATION_FEATURES=--features parallel --release


pre_commit: #test validate
	cargo fmt
	cargo clippy


validate: 
	cargo test $(VALIDATION_FEATURES) --workspace 
	cargo test $(VALIDATION_FEATURES) -p simple -- --ignored 

test:
	cargo test --features parallel  --workspace 
	cargo test --release --features parallel  -p simple -- --ignored 

v_box: 
	cargo test $(VALIDATION_FEATURES) --package simple --test box -- box_sim --exact --nocapture

v_cold:
	cargo test $(VALIDATION_FEATURES) --package simple --test cold_apartment -- apartment_sim --exact --nocapture --ignored

v_versailles:
	cargo test $(VALIDATION_FEATURES) --package simple --test versailles -- versailles_sim --exact --nocapture --ignored

v_walls: 
	cargo test $(VALIDATION_FEATURES) --package heat --test validate_wall_heat_transfer -- validate --exact --nocapture

neighbours:
	cargo test --features parallel --release --package simple --test neighbours -- neighbours_sim --exact --nocapture --ignored


# DOCUMENTATION
documentation:
	RUSTDOCFLAGS="--html-in-header $(shell pwd)/katex.html" cargo doc --document-private-items --workspace