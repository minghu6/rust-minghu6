use chardet::{detect, charset2encoding};
use encoding::label::encoding_from_whatwg_label;
use encoding::DecoderTrap;

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;


pub fn bytes2utf8(raw_contents: Vec<u8>) -> Box<String> {

    let detected_encoding_result = detect(&raw_contents);

    let encoding_label = charset2encoding(&detected_encoding_result.0);

    let coder = encoding_from_whatwg_label(&encoding_label);
    let contents = coder.unwrap().decode(&raw_contents, DecoderTrap::Ignore).expect("Error");

    Box::new(contents)
}

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
