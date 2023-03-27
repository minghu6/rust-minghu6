uninstall:
	cargo uninstall

install-lc:
	cargo install --path . --example srcstats

install-pkcheat:
	cargo install --path . --example pkcheat

bump-version:
	# cargo install cargo-workspaces
	cargo ws version --no-git-tag

clean:
	@ rm *.dot
