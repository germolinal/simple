VALIDATION_FEATURES=--features parallel --release


pre_commit: #test validate
	cargo fmt
	cargo clippy

build:
	cargo build --features parallel --release

# Tags main as v<version in Cargo.toml> and pushes the tag (see release.sh)
release:
	./release.sh

wasm:
	cargo build --features wasm --release --target wasm32-unknown-unknown

test:
	cargo test --features parallel  --workspace
	cargo test --release --features parallel  -p simple -- --ignored

neighbours:
	cargo test --features parallel --release --package simple --test neighbours -- neighbours_sim --exact --nocapture --ignored



# Regenerates every report in docs/validation (see `make check_validation`)
validate: cold box convection dc cloud_to_sun solar_irradiance ir_irradiance ray_tracer infiltration versailles walls weather
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
	cargo test --package weather --lib -- solar::tests::test_cloud_cover_to_global_rad_generic --exact --nocapture

solar_irradiance:
	cargo test $(VALIDATION_FEATURES) --package light --test validate_solar_radiation -- validate_solar_radiation --exact --nocapture

ir_irradiance:
	cargo test $(VALIDATION_FEATURES) --package light --test validate_ir_radiation -- validate_ir_radiation --exact --nocapture

ray_tracer:
	cargo test $(VALIDATION_FEATURES) --package rendering --test validate_ray_tracer -- validate_ray_tracer --exact --nocapture

infiltration:
	cargo test --package air --test validate_infiltration -- validate --exact --nocapture


versailles:
	cargo test $(VALIDATION_FEATURES) --package simple --test versailles -- versailles_sim --exact --nocapture --ignored

walls:
	cargo test  --package heat --test validate_wall_heat_transfer -- validate --exact --nocapture

weather:
	cargo test $(VALIDATION_FEATURES) --package weather --test go_through -- test_go_through --exact --nocapture



# DOCUMENTATION
# The published site (https://germolinal.github.io/simple/) is:
#   /             -> docs/index.html (landing page + validation reports)
#   /validation/  -> docs/validation/*.html (written by `make validate`)
#   /book/        -> docs/book (hand-written chapters + auto-generated IO reference)
SITE_DIR=$(CURDIR)/target/site

# Writes docs/book/src/auto-*.md and IO_SUMMARY.md
io_reference:
	cargo test -p model --lib -- model::testing::write_io_doc --exact

summary:
	cat docs/book/SUMMARY_TEMPLATE.md docs/book/src/IO_SUMMARY.md > docs/book/src/SUMMARY.md

book: io_reference site

# Assembles the site, assuming the IO reference was already generated
site: summary
	rm -rf $(SITE_DIR)
	mdbook build docs/book --dest-dir $(SITE_DIR)/book
	cp docs/index.html $(SITE_DIR)/index.html
	cp -R docs/validation $(SITE_DIR)/validation

serve_book: io_reference summary
	mdbook serve docs/book --open

# Fails if any report in docs/validation is older than the code it validates
check_validation:
	./docs/check_validation.sh

# Rust API docs (not published)
documentation:
	RUSTDOCFLAGS="--html-in-header $(shell pwd)/katex.html" cargo doc --document-private-items --workspace
