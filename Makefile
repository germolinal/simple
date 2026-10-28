VALIDATION_FEATURES=--features parallel --release

# None of these targets is a file (`weather` is also a directory, so without
# this, `make weather` would never run)
.PHONY: pre_commit ci build release wasm test neighbours validate cold box convection dc cloud_to_sun solar_irradiance ir_irradiance view_factors ray_tracer light_sources infiltration walls weather io_reference api_docs summary book website site serve_book check_validation documentation


pre_commit: #test validate
	cargo fmt
	cargo clippy --workspace --all-targets

# The checks CI runs on every pull request (see .github/workflows/test.yaml),
# except the tests
ci:
	cargo fmt --all -- --check
	RUSTFLAGS="-D warnings" cargo clippy --workspace --all-targets --locked -- -D warnings
	RUSTFLAGS="-D warnings" cargo clippy --workspace --all-targets --locked --features parallel -- -D warnings
	RUSTFLAGS="-D warnings" cargo check --workspace --all-targets --locked --features float
	RUSTFLAGS="-D warnings" cargo check --workspace --all-targets --locked --features float --features parallel
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked

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
validate: cold box convection dc cloud_to_sun solar_irradiance ir_irradiance view_factors ray_tracer light_sources infiltration walls weather
	echo "Validated $$(git rev-parse --short HEAD)$$(git diff --quiet HEAD -- . ':!docs/validation' || echo ' (with uncommitted changes)') on $$(date -u '+%Y-%m-%d %H:%M:%S UTC')" > docs/validation/.last_validated
	echo Done validating! Commit docs/validation, including .last_validated

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

view_factors:
	cargo test $(VALIDATION_FEATURES) --package light --test validate_view_factors -- validate_view_factors --exact --nocapture

ray_tracer:
	cargo test $(VALIDATION_FEATURES) --package rendering --test validate_ray_tracer -- validate_ray_tracer --exact --nocapture

light_sources:
	cargo test $(VALIDATION_FEATURES) --package rendering --test validate_light_sources -- validate_light_sources --exact --nocapture

infiltration:
	cargo test --package air --test validate_infiltration -- validate --exact --nocapture


walls:
	cargo test  --package heat --test validate_wall_heat_transfer -- validate --exact --nocapture

weather:
	cargo test --release --package weather --test go_through -- test_go_through --exact --nocapture



# DOCUMENTATION
# The published site (https://germolinal.github.io/simple/), built from each
# release tag by .github/workflows/release.yaml, is:
#   /             -> docs/website (Astro landing page: downloads, links to the rest)
#   /validation/  -> docs/validation/*.html (written by `make validate`)
#   /book/        -> docs/book (hand-written chapters + auto-generated IO reference)
#   /api/         -> Rust API reference (`make api_docs`, copied if present)
SITE_DIR=$(CURDIR)/target/site

# Writes docs/book/src/auto-*.md and IO_SUMMARY.md
io_reference:
	cargo test -p model --lib -- model::testing::write_io_doc --exact

# Writes the Rust API reference into target/doc
api_docs:
	cargo doc --workspace --no-deps

summary:
	cat docs/book/SUMMARY_TEMPLATE.md docs/book/src/IO_SUMMARY.md > docs/book/src/SUMMARY.md

# Everything: generates the references, then assembles the site
book: io_reference api_docs site

# Builds the landing page into $(SITE_DIR) (needs Node.js; set SIMPLE_VERSION
# to the release it describes, or it uses the latest `v*` tag)
website:
	cd docs/website && npm ci && npm run build -- --outDir $(SITE_DIR)

# Assembles the site, assuming the IO reference (and API docs) were already generated
site: summary
	rm -rf $(SITE_DIR)
	$(MAKE) website
	mdbook build docs/book --dest-dir $(SITE_DIR)/book
	cp -R docs/validation $(SITE_DIR)/validation
	if [ -d target/doc ]; then cp -R target/doc $(SITE_DIR)/api; fi

serve_book: io_reference summary
	mdbook serve docs/book --open

# Fails if any report in docs/validation is older than the code it validates
check_validation:
	./docs/check_validation.sh

# Rust API docs (not published)
documentation:
	RUSTDOCFLAGS="--html-in-header $(shell pwd)/katex.html" cargo doc --document-private-items --workspace
