uninstall:
	cargo uninstall

install-lc:
	cargo install --path . --example srcstats

install-pkcheat:
	cargo install --path . --example pkcheat

check-syntax:
	cargo build
	cargo test --no-run --workspace
	cargo bench --no-run --workspace

# cargo install cargo-workspaces
bump-version: check-syntax
	cargo ws version --no-individual-tags

clean:
	@ rm *.dot
