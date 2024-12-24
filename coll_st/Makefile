
test-flatbpt:
	RUST_LOG=info cargo test flatbpt -- --nocapture

trace-flatbpt:
	RUST_LOG=trace cargo test flatbpt -- --nocapture

test-rb:
	cargo test test_bst_rb -- --nocapture

bench-dict-get:
	cargo bench dict_get
	# cargo bench --bench dict_get

bench-dict-bulk-build:
	cargo bench dict_bulk_build

bench-dict-insert:
	cargo bench dict_insert

bench-dict-remove:
	cargo bench dict_remove

bench-dict-range:
	cargo bench dict_range

bench-dict-all:
	cargo bench dict_all
