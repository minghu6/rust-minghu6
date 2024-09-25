//! Universal Coded Character Set (UCS Unicode)
//!
//! UCS-4 alias as UTF-32


use common::BitLen;

#[derive(Debug, Clone, Copy)]
pub enum UTF8CharSt {
    First { partial: u8, nbyte: u8 },
    Continue(u8),
}

#[derive(Debug)]
pub enum UTF8DecodeError {
    /// Invalid Unicode
    UTF16Surrogates(u16),

    /// > 1111_0xxx
    OutRange(u8),

    /// 10xx_xxxx
    TrailAtCodePoint(u8),

    ///
    UnexpectedCodePoint {
        expect: u8,
        found: u8,
    },

    UnexpectedEOF {
        expect: u8,
        found: u8,
    },
}

#[derive(Debug)]
pub enum UTF8EncodeError {
    /// Invalid Unicode
    UTF16Surrogates(u16)
}

pub struct UTF8Str {
    inner: Vec<u8>,
}


impl TryFrom<u8> for UTF8CharSt {
    type Error = UTF8DecodeError;

    fn try_from(c: u8) -> Result<Self, Self::Error> {
        Ok(if c < 0x007F {
            UTF8CharSt::First {
                partial: c,
                nbyte: 1,
            }
        }
        // 10xx_xxxx, 0x80 - 0xBF
        else if 0x80 <= c && c <= 0xBF {
            UTF8CharSt::Continue(c & 0x3F) // 0011_1111
        }
        // 110x_xxxx, 0xC0 - 0xDF
        else if 0xC0 <= c && c <= 0xDF {
            UTF8CharSt::First {
                partial: c & 0x1F, // 0001_1111
                nbyte: 2,
            }
        }
        // 1110_xxxx, 0xE0 - 0xEF
        else if 0xE0 <= c && c <= 0xEF {
            UTF8CharSt::First {
                partial: c & 0x0F, // 0000_1111
                nbyte: 3,
            }
        }
        // 1111_0xxx, 0xF0 - 0xF7
        else if 0xF0 <= c && c <= 0xF7 {
            UTF8CharSt::First {
                partial: c & 0x07, // 0000_0111
                nbyte: 4,
            }
        } else {
            return Err(UTF8DecodeError::OutRange(c));
        })
    }
}


impl UTF8Str {
    pub fn push_char(&mut self, _c: char) {}


    pub fn to_string(&self) -> String {
        unsafe { String::from_utf8_unchecked(self.inner.clone()) }
    }
}


/// Write order (Big endian)
/// -> (step, char)
pub fn next_utf8_unicode(bytes: &[u8]) -> Result<(u8, char), UTF8DecodeError> {
    let ch_1st = bytes[0];
    #[allow(unused)]
    let mut acc = 0u32;
    let n;

    match UTF8CharSt::try_from(ch_1st)? {
        UTF8CharSt::First { partial, nbyte } => {
            acc = partial as u32;
            n = nbyte;
        }
        UTF8CharSt::Continue(_partial) => {
            return Err(UTF8DecodeError::TrailAtCodePoint(ch_1st));
        }
    }

    if bytes.len() < n as usize {
        return Err(UTF8DecodeError::UnexpectedEOF {
            expect: n,
            found: bytes.len() as u8,
        });
    }

    for i in 1..n as usize {
        match UTF8CharSt::try_from(bytes[i])? {
            UTF8CharSt::First { partial: _, nbyte: _ } => {
                return Err(UTF8DecodeError::UnexpectedCodePoint {
                    expect: n,
                    found: i as u8,
                });
            }
            UTF8CharSt::Continue(partial) => {
                acc = (acc << 6) | partial as u32;
            }
        }
    }

    if 0xD800 <= acc && acc <= 0xDFFF {
        return Err(UTF8DecodeError::UTF16Surrogates(acc as u16))
    }

    let ch = unsafe { char::from_u32_unchecked(acc) };

    Ok((n, ch))
}

pub fn decode_utf8_bytes(bytes: &[u8]) -> Result<Vec<char>, UTF8DecodeError> {
    let mut step = 0;
    let mut chars = vec![];

    loop {
        let (nbytes, c) = next_utf8_unicode(&bytes[step..])?;

        step += nbytes as usize;
        chars.push(c);

        if step == bytes.len() { break Ok(chars); }
    }
}

pub fn encode_utf8_char(bytes: &mut Vec<u8>, ch: u32) -> Result<(), UTF8EncodeError> {

    if 0xD800 <= ch && ch <= 0xDFFF {
        return Err(UTF8EncodeError::UTF16Surrogates(ch as u16))
    }

    let bitlen = ch.bit_len();

    // 7
    if bitlen <= 7 {
        return Ok(bytes.push(ch as u8));
    }

    // let mut bitph: u32 = if bitlen <= 11 { 11 }
    // else if bitlen <= 16 { 16 }
    // else if bitlen <= 21 { 21 }
    // else { unreachable!() };

    // while bitph > 5 {
    //     let shf = bitph - bitph % 6;

    //     let byte = ((ch >> shf) & 0x3F) as u8 | 0x80;

    //     bytes.push(byte);
    // }


    // 5 + 6 = 11
    if bitlen <= 11 {
        // + 0b10xx_xxxx
        let byte_2nd = (ch & 0x3F) as u8 | 0x80;

        // + 0b110x_xxxx, patch 5 bit, or 0xC0
        let byte_1st = ((ch >> 6) & 0x1F) as u8 | 0xC0;

        bytes.push(byte_1st);
        bytes.push(byte_2nd);
    }

    // 4 + 6 * 2 = 16
    else if bitlen <= 16 {
        // + 0b10xx_xxxx
        let byte_3rd = (ch & 0x3F) as u8 | 0x80;

        // + 0b10xx_xxxx
        let byte_2nd = ((ch >> 6) & 0x3F) as u8 | 0x80;

        // + 0b1110_xxxx, patch 4 bit, or 0xE0
        let byte_1st = ((ch >> 12) & 0xF) as u8 | 0xE0;

        bytes.push(byte_1st);
        bytes.push(byte_2nd);
        bytes.push(byte_3rd);
    }

    // 3 + 6 * 3 = 21
    else if bitlen <= 21 {
        // + 0b10xx_xxxx
        let byte_4th = (ch & 0x3F) as u8 | 0x80;

        // + 0b10xx_xxxx
        let byte_3rd = ((ch >> 6) & 0x3F) as u8 | 0x80;

        // + 0b10xx_xxxx
        let byte_2nd = ((ch >> 12) & 0x3F) as u8 | 0x80;

        // + 0b1111_1xxx, patch 4 bit, or 0xE0
        let byte_1st = ((ch >> 18) & 0x7) as u8 | 0xF8;

        bytes.push(byte_1st);
        bytes.push(byte_2nd);
        bytes.push(byte_3rd);
        bytes.push(byte_4th);
    }
    else {
        unreachable!("ch: {}, ({})", ch, bitlen)
    }

    Ok(())
}




#[cfg(test)]
mod tests {

    use super::{UTF8CharSt, decode_utf8_bytes, encode_utf8_char};


    #[test]
    fn test_str_index() {
        let s = String::from_utf8(vec![0xD8, 0x82]).unwrap();
        let _ss = s.as_str();
        let c = char::from_u32(0xD7FF).unwrap();

        println!("{}, {}", s, c);
    }

    #[test]
    fn test_utf8() {
        let s = "你好，世界！";

        for c in s.as_bytes().into_iter() {
            let st = UTF8CharSt::try_from(*c).unwrap();

            println!("{}: {:?}", c, st);
        }

        let chars = decode_utf8_bytes(s.as_bytes()).unwrap();
        println!("{:?}", chars);

        let mut bytes = vec![];
        for ch in chars.into_iter() {
            encode_utf8_char(&mut bytes, ch as u32).unwrap();
        }

        let s = String::from_utf8(bytes).unwrap();
        println!("{}", s);

    }
}

