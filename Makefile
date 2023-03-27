uninstall:
	cargo uninstall

install-lc:
	cargo install --path . --example srcstats

install-pkcheat:
	cargo install --path . --example pkcheat

# cargo install cargo-workspaces
bump-version:
	cargo ws version --no-individual-tags

clean:
	@ rm *.dot
