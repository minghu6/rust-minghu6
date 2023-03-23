#![feature(box_syntax)]
#![feature(default_free_fn)]
#![allow(unused_imports)]

use std::{
    convert::TryFrom,
    default::default,
    fs::{read, File},
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
};

use clap::Parser;
use serde_json::{self, Map, Value};
use configparser::ini::Ini;

use common::{Itertools, error_code::ErrorCode};


macro_rules! value_name {
    ($v: expr) => {
        match $v {
            Value::Null => "Null",
            Value::Object(_) => "Object",
            Value::Array(_) => "Array",
            Value::Number(_) => "Number",
            Value::String(_) => "String",
            Value::Bool(_) => "Bool",
        }
    };
}


macro_rules! flat2tupleiter {
    ($iter:expr) => {
        {
            let mut _coll = vec![];
            for (x0, x1) in $iter.into_iter() {
                _coll.push(x0);
                _coll.push(x1);
            }
            _coll
        }
    };
}



const AIWU_CHEAT_LIST_NAME: &str = "aiWuCheatList";
const AIWU_CHEAT_CUSTOM_LIST_NAME: &str = "customCheatList";
const AIWU_CHEAT_SUB_NAME: &str = "children";
const AIWU_CHEAT_CODE_NAME: &str = "cheatCode";
const AIWU_CHEAT_DESC_NAME: &str = "desc";



#[derive(Debug)]
pub struct CheatPeace {
    pub title: String,
    pub desc: String,
    /// 8byte String
    pub code: Vec<String>,
    pub is_turned: bool,
}

#[derive(Debug)]
struct CommonCheats(Vec<CheatPeace>);



impl CommonCheats {
    fn try_from_aiwu_confpath<P: AsRef<Path>>(p: P) -> Result<Self, ErrorCode> {
        let file = File::open(p).map_err(|err| ErrorCode::Open(err))?;
        let buf_file = BufReader::new(file);

        let jsonobj: Value = serde_json::from_reader(buf_file)
            .map_err(|_err| ErrorCode::MalformedJson)?;

        let mut flat_cheats = vec![];
        match jsonobj {
            Value::Object(mut kvmap) => {
                let mut maybe_nested_cheats_coll = vec![];

                if let Some(maybe_nested_cheats) =
                    kvmap.remove(AIWU_CHEAT_LIST_NAME)
                {
                    maybe_nested_cheats_coll.push(maybe_nested_cheats);
                }
                if let Some(maybe_nested_cheats) =
                    kvmap.remove(AIWU_CHEAT_CUSTOM_LIST_NAME)
                {
                    maybe_nested_cheats_coll.push(maybe_nested_cheats);
                }

                for maybe_nested_cheats in maybe_nested_cheats_coll {
                    match maybe_nested_cheats {
                        Value::Array(arr) => {
                            for maybe_nested_cheat in arr {
                                flat_cheats.extend(flat_maybe_nested_cheat(
                                    maybe_nested_cheat,
                                )?);
                            }
                        }
                        other => {
                            return Err(ErrorCode::UnmatchedJsonField {
                                expect: format!("root(Object) -> Array"),
                                found: format!("{:#?}", value_name!(other)),
                            })
                        }
                    }
                }
            }
            other => {
                return Err(ErrorCode::UnmatchedJsonField {
                    expect: format!("root(Object)"),
                    found: format!("{}", value_name!(other)),
                })
            }
        };

        let mut cheat_peaces = vec![];
        for cheatv in flat_cheats {
            cheat_peaces.push(
                CheatPeace::try_from_aiwu_jsonobj(cheatv)?
            );
        }

        Ok(CommonCheats(cheat_peaces))
    }


    fn try_into_drastic_confpath<P: AsRef<Path>>(&self, p: P) -> Result<(), ErrorCode> {
        let mut config = Ini::new();

        for piece in self.0.iter() {
            let mut key = String::new();
            for i in (0..piece.code.len()).step_by(2) {
                key.push_str(&format!("{} {}\n", piece.code[i], piece.code[i+1]));
            }

            config.set(&piece.title, &key, None);
        }

        config.write(p).map_err(|err| ErrorCode::Write(err))
    }
}


impl CheatPeace {
    fn try_from_aiwu_jsonobj(cheatv: Value) -> Result<Self, ErrorCode> {
        match cheatv {
            Value::Object(mut kvmap) => {
                let title =
                    if let Some(descv) = kvmap.remove(AIWU_CHEAT_DESC_NAME) {
                        if let Value::String(s) = descv {
                            // Escape control char '[', ']'
                            s
                                .replace("[", "【")
                                .replace("]", "】")
                                .chars()
                                .take(30)
                                .collect()

                        } else {
                            return Err(ErrorCode::UnmatchedJsonField {
                                expect: format!(
                                    "{}: String",
                                    AIWU_CHEAT_DESC_NAME
                                ),
                                found: format!("{}", value_name!(descv)),
                            });
                        }
                    } else {
                        String::new()
                    };

                let code = if let Some(codev) =
                    kvmap.remove(AIWU_CHEAT_CODE_NAME)
                {
                    if let Value::String(s) = codev {
                        flat2tupleiter!(
                            s.split(',')
                            .map(|subs| subs.trim().split_at(8))
                            .map(|(x0, x1)| (x0.to_owned(), x1.to_owned()))
                        )
                    } else {
                        return Err(ErrorCode::UnmatchedJsonField {
                            expect: format!(
                                "{}: String",
                                AIWU_CHEAT_CODE_NAME
                            ),
                            found: format!("{}", value_name!(codev)),
                        });
                    }
                } else {
                    return Err(ErrorCode::UnmatchedJsonField {
                        expect: format!("{}: String", AIWU_CHEAT_CODE_NAME),
                        found: format!("field not exist"),
                    });
                };

                Ok(CheatPeace {
                    title,
                    desc: default(),
                    code,
                    is_turned: false,
                })
            }
            other => Err(ErrorCode::UnmatchedJsonField {
                expect: format!("root(Object) -> Array -> Object"),
                found: format!("{}", value_name!(other)),
            }),
        }
    }
}




fn flat_maybe_nested_cheat(
    mut itemv: Value,
) -> Result<Box<dyn Iterator<Item = Value>>, ErrorCode> {
    match itemv {
        // looks like Haskell, cool!
        Value::Object(ref mut kvmap) => {
            if kvmap.contains_key(AIWU_CHEAT_SUB_NAME) {
                let maybe_nested_cheats =
                    kvmap.remove(AIWU_CHEAT_SUB_NAME).unwrap();
                /* Cheak extra cheat code field is not exist */
                if let Some(cheatv) = kvmap.get(AIWU_CHEAT_CODE_NAME) {
                    if let Some(arr) = cheatv.as_array() {
                        if !arr.is_empty() {
                            return Err(ErrorCode::UnmatchedJsonField {
                                expect: format!(
                                    "Empty cheat code field for subdir"
                                ),
                                found: format!("Nonnull cheat code field"),
                            });
                        }
                    }
                }

                match maybe_nested_cheats {
                    Value::Array(arr) => Ok(box arr.into_iter()),
                    other => Err(ErrorCode::UnmatchedJsonField {
                        expect: format!("subdir for list"),
                        found: format!("{}", value_name!(other)),
                    }),
                }
            } else if kvmap.contains_key(AIWU_CHEAT_CODE_NAME) {
                Ok(box std::iter::once(itemv))
            } else {
                Err(ErrorCode::UnmatchedJsonField {
                    expect: format!("cheat code field"),
                    found: format!("no cheat code field"),
                })
            }
        }
        other => Err(ErrorCode::UnmatchedJsonField {
            expect: format!("root(Object) -> Array -> Object"),
            found: format!("{}", value_name!(other)),
        }),
    }
}


/// PKconfig converter
#[derive(Parser)]
#[clap()]
struct Args {
    patten: PathBuf,
    // #[clap(default_value=".")]
    // target_dir: PathBuf
}


fn main() -> Result<(), ErrorCode> {
    let args = Args::parse();

    let plains = CommonCheats::try_from_aiwu_confpath(&args.patten)?;
    plains.try_into_drastic_confpath(args.patten.with_extension("ini"))?;
    // println!("{:?}", plains);

    Ok(())
}
