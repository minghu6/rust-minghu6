
uninstall:
	cargo uninstall

install-lc:
	cargo install --path . --example srcstats

install-pkcheat:
	cargo install --path . --example pkcheat

clean:
	@ rm *.dot
