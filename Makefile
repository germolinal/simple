VALIDATION_FEATURES=--features parallel --release


pre_commit: #test validate
	cargo fmt
	cargo clippy



test:
	cargo test --features parallel  --workspace 
	cargo test --release --features parallel  -p simple -- --ignored 

neighbours:
	cargo test --features parallel --release --package simple --test neighbours -- neighbours_sim --exact --nocapture --ignored



validate: cold box convection dc cloud_to_sun solar_irradiance infiltration
	echo Done validating!

cold:
	cargo test $(VALIDATION_FEATURES) --package simple --test cold_apartment -- apartment_sim --exact --nocapture --ignored

box: 
	cargo test $(VALIDATION_FEATURES) --package simple --test box -- box_sim --exact --nocapture	

convection:
	cargo test $(VALIDATION_FEATURES) --package heat --test validate_convection -- validate --exact --nocapture

dc:
	cargo test $(VALIDATION_FEATURES) --features parallel --package rendering --test validate_dc -- validate_dc --exact --nocapture

cloud_to_sun:
	cargo test $(VALIDATION_FEATURES) --package weather --lib -- solar::tests::test_cloud_cover_to_global_rad_generic --exact --nocapture 

solar_irradiance:
	cargo test $(VALIDATION_FEATURES) --package light --test validate_solar_radiation -- validate_solar_radiation --exact --nocapture

infiltration:
	cargo test $(VALIDATION_FEATURES) --package air --test validate_infiltration -- validate --exact --nocapture

versailles:
	cargo test $(VALIDATION_FEATURES) --package simple --test versailles -- versailles_sim --exact --nocapture --ignored

walls: 
	cargo test  --package heat --test validate_wall_heat_transfer -- validate --exact --nocapture	

weather:
	cargo test $(VALIDATION_FEATURES) --package weather --test go_through -- test_go_through --exact --nocapture



# DOCUMENTATION
documentation:
	RUSTDOCFLAGS="--html-in-header $(shell pwd)/katex.html" cargo doc --document-private-items --workspace