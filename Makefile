uninstall:
	cargo uninstall

install-lc:
	cargo install --path . --example srcstats

check-syntax:
	cargo build
	cargo test --no-run --workspace
	cargo bench --no-run --workspace

# cargo install cargo-workspaces
bump-version: check-syntax
	cargo ws version --no-individual-tags

check-resource-config:
	cargo expand --lib -p m6-resource-config

clean-ice:
	@ rm -f rustc-ice*.txt

clean: clean-ice
	@ rm -f *.dot
