
test-flatbpt:
	RUST_LOG=info cargo test flatbpt -- --nocapture

trace-flatbpt:
	RUST_LOG=trace cargo test flatbpt -- --nocapture

test-rb:
	cargo test test_bst_rb -- --nocapture

bench-dict-get:
	cargo bench dict_get -- --nocapture
	# cargo bench --bench dict_get

bench-dict-bulk-build:
	cargo bench dict_bulk_build -- --nocapture

bench-dict-insert:
	cargo bench dict_insert -- --nocapture

bench-dict-remove:
	cargo bench dict_remove -- --nocapture

bench-dict-insert-remove:
	cargo bench bench_dict_ --bench dict_insert_remove -- --nocapture

bench-dict-range:
	cargo bench dict_range -- --nocapture

bench-dict-all:
	cargo bench dict_all -- --nocapture
