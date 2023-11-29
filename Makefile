VALIDATION_FEATURES=--features parallel # --release


# VALIDATION
validate: v_box v_cold v_versailles v_walls
	echo "Done!"

v_box: 
	cargo test $(VALIDATION_FEATURES) --package simple --test box -- box_sim --exact --nocapture

v_cold:
	cargo test $(VALIDATION_FEATURES) --package simple --test cold_apartment -- apartment_sim --exact --nocapture --ignored

v_versailles:
	cargo test $(VALIDATION_FEATURES) --package simple --test versailles -- versailles_sim --exact --nocapture --ignored

v_walls: 
	cargo test $(VALIDATION_FEATURES) --package heat --test validate_wall_heat_transfer -- validate --exact --nocapture



# DOCUMENTATION
documentation:
	RUSTDOCFLAGS="--html-in-header $(shell pwd)/katex.html" cargo doc --document-private-items --workspace