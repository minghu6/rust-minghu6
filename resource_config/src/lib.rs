//! Mix data in code.

use lazy_static::lazy_static;

use proc_macros::resources;

resources! {
    graph: {
        sp: {
            sp_5_csv: "sp5.csv"
        }
    },
    test_suites: {
        mutable_mapping_toml: "mutable_mapping.toml",
        bpt_toml: "bpt.toml"
    },
    zh_en_poems_txt: "zh_en_poems.txt"
}

lazy_static! {
    pub static ref RES: Res = {
        Res::new()
    };
}
