

## Test
`cargo test --test test_name -- modname::some_test`


## Bench

`cargo bench --bench bench_name -- modname::some_benchmark`

By default, the bench profile inherits the settings from the **release** profile.

Adjust it: https://doc.rust-lang.org/cargo/reference/profiles.html#bench
