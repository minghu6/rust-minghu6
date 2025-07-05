//! If the BOM is missing, RFC 2781 recommends[nb 3] that big-endian (BE) encoding be assumed.
//! In practice, due to Windows using little-endian (LE) order by default,
//! many applications assume little-endian encoding.
//! It is also reliable to detect endianness by looking for null bytes,
//! on the assumption that characters less than U+0100 are very common.
//! If more even bytes (starting at 0) are null, then it is big-endian.

pub const BOM_U16_LE: u16 = 0xFFFE;
pub const BOM_U16_BE: u16 = 0xFEFF;

use UTF16Char::*;
use UTF16EncodeError::*;
use UTF16DecodeError::*;

////////////////////////////////////////////////////////////////////////////////
//// Macros

macro_rules! split_u16 {
    ($expr:expr) => {{
        let cached_value = $expr as u16;
        [
            (cached_value & 0xFF) as u8,
            ((cached_value & 0xFF00) >> 8) as u8,
        ]
    }};
}

macro_rules! parse_2_array {
    ($bytes:expr) => {{
        let cached_value = &$bytes;

        if cached_value.len() >= 2 {
            Ok(cached_value[..2].try_into().unwrap())
        } else {
            Err(MalformedLength(cached_value.len()))
        }
    }};
}

////////////////////////////////////////////////////////////////////////////////
//// Structures

#[derive(Debug)]
pub enum UTF16EncodeError {
    Surrogate(u16),
    /// code point > 0x10_FF_FF
    Overflow(u32),
}

#[derive(Debug)]
pub enum UTF16DecodeError {
    UnpairedSurrogate(u16),
    MalformedLength(usize),
}

#[derive(Debug)]
pub enum UTF16Char {
    /// Basic Multilingual Plane
    BMP(u16),

    /// Supplementary Multilingual Plane
    ///
    /// High surrogates, Low surrogates
    SMP(u16, u16),
}

////////////////////////////////////////////////////////////////////////////////
//// Functions

pub fn encode_utf16_le(
    chars: impl Iterator<Item = char>,
) -> Result<Vec<u8>, UTF16EncodeError> {
    let mut bytes = vec![];

    for ch in chars {
        encode_utf16_cp_le(&mut bytes, ch as _)?;
    }

    Ok(bytes)
}

pub fn encode_utf16_be(
    chars: impl Iterator<Item = char>,
) -> Result<Vec<u8>, UTF16EncodeError> {
    let mut bytes = vec![];

    for ch in chars {
        encode_utf16_cp_be(&mut bytes, ch as _)?;
    }

    Ok(bytes)
}

pub fn encode_utf16_cp_le(
    bytes: &mut Vec<u8>,
    ch: u32,
) -> Result<(), UTF16EncodeError> {
    match encode_utf16_cp_(ch)? {
        BMP(w) => bytes.extend_from_slice(&split_u16!(w)),
        SMP(w1, w2) => {
            let [b0, b1] = split_u16!(w1);
            let [b2, b3] = split_u16!(w2);

            bytes.extend_from_slice(&[b0, b1, b2, b3]);
        }
    }

    Ok(())
}

pub fn encode_utf16_cp_be(
    bytes: &mut Vec<u8>,
    ch: u32,
) -> Result<(), UTF16EncodeError> {
    match encode_utf16_cp_(ch)? {
        BMP(w) => {
            let [b0, b1] = split_u16!(w);
            bytes.extend_from_slice(&[b1, b0]);
        }
        SMP(w1, w2) => {
            let [b0, b1] = split_u16!(w1);
            let [b2, b3] = split_u16!(w2);

            bytes.extend_from_slice(&[b1, b0, b3, b2]);
        }
    }

    Ok(())
}

fn encode_utf16_cp_(ch: u32) -> Result<UTF16Char, UTF16EncodeError> {
    match ch {
        ..0xD800 | 0xE000..0x01_0000 => Ok(BMP(ch as _)),
        0xD800..0xE000 => Err(Surrogate(ch as _)),
        0x01_00_00..0x11_00_00 => {
            // U' = yyyyyyyyyyxxxxxxxxxx  // U - 0x10000
            // W1 = 110110yyyyyyyyyy      // 0xD800 + yyyyyyyyyy high surrogate
            // W2 = 110111xxxxxxxxxx      // 0xDC00 + xxxxxxxxxx low surrogate

            // 0x00_00_00 - 0x0F_FF_FF
            let ch = ch - 0x01_00_00;

            // high unit 0b_1101_11_00_0000_0000, 0xDC00
            let unit_hi: u16 = 0xD800 | ((ch >> 10) & 0x3FF) as u16;

            // low unit
            // 0b_1101_10_00_0000_0000 (10 bit) ..., 0xD800
            let unit_lo: u16 = 0xDC00 | (ch & 0x3FF) as u16;

            Ok(SMP(unit_hi, unit_lo))
        }
        0x11_00_00.. => Err(Overflow(ch)),
    }
}

pub fn decode_utf16_le(bytes: &[u8]) -> Result<String, UTF16DecodeError> {
    let mut s = String::new();

    let mut ptr = bytes;

    while !ptr.is_empty() {
        let (ch, nbytes) = decode_utf16_cp_le(ptr)?;

        s.push(ch);
        ptr = &ptr[nbytes..];
    }

    Ok(s)
}

pub fn decode_utf16_be(bytes: &[u8]) -> Result<String, UTF16DecodeError> {
    let mut s = String::new();

    let mut ptr = bytes;

    while !ptr.is_empty() {
        let (ch, nbytes) = decode_utf16_cp_be(ptr)?;

        s.push(ch);
        ptr = &ptr[nbytes..];
    }

    Ok(s)
}

pub fn decode_utf16_cp_le(
    bytes: &[u8],
) -> Result<(char, usize), UTF16DecodeError> {
    let w = u16::from_le_bytes(
        parse_2_array!(bytes)?,
    );

    Ok(match w {
        ..0xD800 | 0xE000.. => (decode_utf16_cp_(BMP(w)), 2),
        // high surrogate
        0xD800..0xDC00 => {
            let w2 =
                u16::from_le_bytes(parse_2_array!(bytes[2..])?);

            (decode_utf16_cp_(SMP(w, w2)), 4)
        }
        // low surrogate
        0xDC00..0xE000 => Err(UnpairedSurrogate(w))?,
    })
}

pub fn decode_utf16_cp_be(
    bytes: &[u8],
) -> Result<(char, usize), UTF16DecodeError> {
    let w = u16::from_be_bytes(
        parse_2_array!(bytes)?,
    );

    Ok(match w {
        ..0xD800 | 0xE000.. => (decode_utf16_cp_(BMP(w)), 2),
        // high surrogate
        0xD800..0xDC00 => {
            let w2 =
                u16::from_be_bytes(parse_2_array!(bytes[2..])?);

            (decode_utf16_cp_(SMP(w, w2)), 4)
        }
        // low surrogate
        0xDC00..0xE000 => Err(UnpairedSurrogate(w))?,
    })
}

fn decode_utf16_cp_(utf16char: UTF16Char) -> char {
    match utf16char {
        BMP(w) => w as u32,
        SMP(w1, w2) => {
            (((w1 & 0x3FF) << 10) | (w2 & 0x3FF)) as u32 + 0x01_00_00
        }
    }
    .try_into()
    .unwrap()
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf16_le() {
        assert_eq!(
            encode_utf16_le(['\u{10437}'].into_iter()).unwrap(),
            vec![0x01, 0xD8, 0x37, 0xDC]
        );
        assert_eq!(
            encode_utf16_be(['\u{10437}'].into_iter()).unwrap(),
            vec![0xD8, 0x01, 0xDC, 0x37]
        );

        let s = "你好，世界！\u{1f6e6}";

        assert_eq!(
            s,
            String::from_utf16le_lossy(&encode_utf16_le(s.chars()).unwrap()),
            "{s:?}"
        );

        assert_eq!(
            s,
            String::from_utf16be_lossy(&encode_utf16_be(s.chars()).unwrap()),
            "{s:?}"
        );

        assert_eq!(
            s,
            decode_utf16_le(&encode_utf16_le(s.chars()).unwrap()).unwrap(),
            "{s:?}"
        );
        assert_eq!(
            s,
            decode_utf16_be(&encode_utf16_be(s.chars()).unwrap()).unwrap(),
            "{s:?}"
        );
    }
}
