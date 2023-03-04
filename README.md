

## Test
`cargo test --test test_name -- modname::some_test`


## Bench

`cargo bench --bench bench_name -- modname::some_benchmark`

By default, the bench profile inherits the settings from the **release** profile.

Adjust it: https://doc.rust-lang.org/cargo/reference/profiles.html#bench


## Pass Custom CFG Flag

`RUSTFLAGS="--cfg=tprofile" cargo test bench_bp_remove --bench dict_remove --release -- --nocapture`
