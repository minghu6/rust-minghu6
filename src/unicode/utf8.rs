use UTF8DecodeError::*;
use UTF8EncodeError::*;

////////////////////////////////////////////////////////////////////////////////
//// Macros

macro_rules! parse_array {
    ($bytes:expr, $n:expr) => {{
        let cached_value = &$bytes;
        let n = $n;

        if cached_value.len() >= n {
            Ok(cached_value[..n].try_into().unwrap())
        } else {
            Err(UnfinishedByteSeq {
                expect: n,
                found: cached_value.len(),
            })
        }
    }};
}

macro_rules! check_continuation_byte {
    ($b:expr, expect=$expect:literal, found=$found:literal) => {
        {
            let b = $b;

            if b >= 0x80 && b < 0xC0 {
                Ok(())
            }
            else {
                Err(UnfinishedByteSeq {
                    expect: $expect,
                    found: $found,
                })
            }
        }
    };
}

////////////////////////////////////////////////////////////////////////////////
//// Structures

#[derive(Debug)]
pub enum UTF8DecodeError {
    InvalidStartByte(u8),
    UnfinishedByteSeq { expect: usize, found: usize },
}

#[derive(Debug)]
pub enum UTF8EncodeError {
    Overflow(u32),
}

////////////////////////////////////////////////////////////////////////////////
//// Functions

pub fn encode_utf8(
    chars: impl Iterator<Item = char>,
) -> Result<Vec<u8>, UTF8EncodeError> {
    let mut bytes = vec![];

    for ch in chars {
        encode_utf8_cp(&mut bytes, ch as _)?;
    }

    Ok(bytes)
}

pub fn encode_utf8_cp(
    bytes: &mut Vec<u8>,
    ch: u32,
) -> Result<(), UTF8EncodeError> {
    // for U+uvwxyz
    Ok(match ch {
        ..0x0080 => bytes.push(ch as _),
        0x0080..0x0800 => {
            // 110xxxyy 10yyzzzz
            // 5 + 6 bit

            let b2 = 0x80 | (ch & 0x3F) as u8;
            let b1 = 0xC0 | ((ch >> 6) & 0x1F) as u8;

            bytes.extend_from_slice(&[b1, b2]);
        }
        0x00_0800..0x01_0000 => {
            // 1110wwww 10xxxxyy 10yyzzzz
            // 4 + 6 * 2 = 16

            let b3 = 0x80 | (ch & 0x3F) as u8;
            let b2 = 0x80 | ((ch >> 6) & 0x3F) as u8;
            let b1 = 0xE0 | ((ch >> 12) & 0x0F) as u8;

            bytes.extend_from_slice(&[b1, b2, b3]);
        }
        0x01_0000..0x10_0000 => {
            // 11110uvv 10vvwwww 10xxxxyy 10yyzzzz
            // 3 + 6 * 3 = 21

            let b4 = 0x80 | (ch & 0x3F) as u8;
            let b3 = 0x80 | ((ch >> 6) & 0x3F) as u8;
            let b2 = 0x80 | ((ch >> 12) & 0x3F) as u8;
            let b1 = 0xF0 | ((ch >> 18) & 0x07) as u8;

            bytes.extend_from_slice(&[b1, b2, b3, b4]);
        }
        0x10_0000.. => Err(Overflow(ch))?,
    })
}

pub fn decode_utf8(bytes: &[u8]) -> Result<String, UTF8DecodeError> {
    let mut s = String::new();

    let mut ptr = bytes;

    while !ptr.is_empty() {
        let (ch, nbytes) = decode_utf8_cp(ptr)?;

        s.push(ch);
        ptr = &ptr[nbytes..];
    }

    Ok(s)
}

pub fn decode_utf8_cp(bytes: &[u8]) -> Result<(char, usize), UTF8DecodeError> {
    let [b1] = parse_array!(bytes, 1)?;

    Ok(match b1 {
        // 0yyyzzzz
        ..0x80 => (b1 as _, 1),
        0x80..0xC0 => Err(InvalidStartByte(b1))?,
        // 110xxxyy 10yyzzzz
        0xC0..0xE0 => {
            let [b2] = parse_array!(bytes[1..], 1)?;

            check_continuation_byte!(b2, expect=2, found=1)?;

            let ch = ((b1 as u32 & 0x1F) << 6) | (b2 as u32 & 0x3F);

            (ch.try_into().unwrap(), 2)
        }
        // 1110wwww 10xxxxyy 10yyzzzz
        0xE0..0xF0 => {
            let [b2, b3] = parse_array!(bytes[1..], 2)?;

            check_continuation_byte!(b2, expect=3, found=1)?;
            check_continuation_byte!(b3, expect=3, found=2)?;

            let ch = (b1 as u32 & 0x0F) << 12
                | (b2 as u32 & 0x3F) << 6
                | b3 as u32 & 0x3F;

            (ch.try_into().unwrap(), 3)
        }
        // 11110uvv 10vvwwww 10xxxxyy 10yyzzzz
        0xF0..0xF8 => {
            let [b2, b3, b4] = parse_array!(bytes[1..], 3)?;

            check_continuation_byte!(b2, expect=4, found=1)?;
            check_continuation_byte!(b3, expect=4, found=2)?;
            check_continuation_byte!(b4, expect=4, found=3)?;

            let ch = (b1 as u32 & 0x07) << 18
                | (b2 as u32 & 0x3F) << 12
                | (b3 as u32 & 0x3F) << 6
                | b4 as u32 & 0x3F;

            (ch.try_into().unwrap(), 4)
        }
        0xF8.. => Err(InvalidStartByte(b1))?,
    })
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_utf8() {
        let s = "你好，世界！\u{1f6e6}";

        assert_eq!(
            s,
            String::from_utf8_lossy(&encode_utf8(s.chars()).unwrap()),
            "{s:?}"
        );
        assert_eq!(
            s,
            decode_utf8(&encode_utf8(s.chars()).unwrap()).unwrap(),
            "{s:?}"
        );
    }
}
