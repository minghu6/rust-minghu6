//! If the BOM is missing, RFC 2781 recommends[nb 3] that big-endian (BE) encoding be assumed.
//! In practice, due to Windows using little-endian (LE) order by default, many applications assume little-endian encoding.
//! It is also reliable to detect endianness by looking for null bytes, on the assumption that characters less than U+0100 are very common.
//! If more even bytes (starting at 0) are null, then it is big-endian.

pub const BOM_U16_LE: u16 = 0xFFFE;
pub const BOM_U16_BE: u16 = 0xFEFF;

macro_rules! split_u16 {
    ($name:ident) => {
        (($name & 0xFF) as u8, ($name & 0xFF00) as u8)
    };
}


#[derive(Debug)]
pub enum UTF16EncodeError {
    /// Invalid Unicode
    UTF16Surrogates(u16),
    Overflow(u32),
}

#[derive(Debug)]
pub enum UTF16DecodeError {
    Overflow(u32),
    UnexpectedEOF,
    UnfinishedUnits,
    ExpectHiFoundLow,
    ExpectLowFoundHi,
    ExpectHiFoundBMP
}

#[derive(Debug)]
pub enum UTF16UnitSt {
    /// Basic Multilingual Plane
    BMP(u16),

    /// Low surrogates, second unit, W2
    Lo(u16),

    /// Hign surrogates, first unit, W1
    Hi(u16)
}

impl TryFrom<u16> for UTF16UnitSt {
    type Error = UTF16DecodeError;

    fn try_from(unit: u16) -> Result<Self, Self::Error> {
        Ok(if unit <= 0xD7FF {
            UTF16UnitSt::BMP(unit)
        }
        else if 0xD800 <= unit && unit <= 0xDFFF {
            if unit <= 0xDBFF {  // 0b_1101_10xx_xxxx_xxxx
                UTF16UnitSt::Lo(unit & 0x03FF)
            }
            else {               // 0b_1101_11xx_xxxx_xxxx
                UTF16UnitSt::Hi(unit & 0x03FF)
            }
        }
        else {  // 0xE000 <= unit
            UTF16UnitSt::BMP(unit)
        })

    }
}

pub fn encode_utf16_le(
    bytes: &mut Vec<u8>,
    ch: u32,
) -> Result<(), UTF16EncodeError> {
    if 0xD800 <= ch && ch <= 0xDFFF {
        return Err(UTF16EncodeError::UTF16Surrogates(ch as u16));
    }

    Ok(if ch < 0x01_00_00 {
        let byte_1st = (ch & 0xFF) as u8;
        let byte_2nd = ((ch >> 8) & 0xFF) as u8;

        bytes.push(byte_1st);
        bytes.push(byte_2nd);
    } else if 0x1_00_00 <= ch && ch <= 0x10_FF_FF {
        // U' = yyyyyyyyyyxxxxxxxxxx  // U - 0x10000
        // W1 = 110110yyyyyyyyyy      // 0xD800 + yyyyyyyyyy
        // W2 = 110111xxxxxxxxxx      // 0xDC00 + xxxxxxxxxx

        // low bits
        // 0b_1101_10_00_0000_0000 (10 bit) ..., 0xD800
        let unit_1st: u16 = 0xDC00 | (ch & 0x3FF) as u16;

        // high bits 0b_1101_11_00_0000_0000, 0xDC00
        let unit_2nd: u16 = 0xD800 | (ch >> 10 & 0x3FF) as u16;

        let (b0, b1) = split_u16!(unit_1st);
        let (b2, b3) = split_u16!(unit_2nd);

        bytes.extend_from_slice(&[b0, b1, b2, b3]);
    } else {
        return Err(UTF16EncodeError::Overflow(ch));
    })
}


fn next_u16_unit(bytes: &[u8]) -> Result<UTF16UnitSt, UTF16DecodeError> {
    if bytes.len() < 2 {
        return Err(UTF16DecodeError::UnexpectedEOF);
    }

    let (b0, b1) = (bytes[0], bytes[1]);

    let unit = ((b1 as u16) << 8) | b0 as u16;

    UTF16UnitSt::try_from(unit)
}

/// -> (nbytes, utf16st)
pub fn next_u16_le_unicode(bytes: &[u8]) -> Result<(u8, char), UTF16DecodeError> {

    Ok(match next_u16_unit(bytes)? {
        UTF16UnitSt::BMP(ch) => {
            (2, char::from_u32(ch as u32).unwrap())
        },
        UTF16UnitSt::Lo(lo) => {
            match next_u16_unit(&bytes[2..])? {
                UTF16UnitSt::BMP(_) => {
                    return Err(UTF16DecodeError::ExpectHiFoundBMP);
                },
                UTF16UnitSt::Lo(_) => {
                    return Err(UTF16DecodeError::ExpectHiFoundLow);
                },
                UTF16UnitSt::Hi(hi) => {
                    (4, char::from_u32((hi << 8 | lo) as u32).unwrap())
                },
            }
        },
        UTF16UnitSt::Hi(_) => {
            return Err(UTF16DecodeError::ExpectLowFoundHi);
        },
    })

}

pub fn decode_utf16_le(
    bytes: &[u8],
) -> Result<Vec<char>, UTF16DecodeError> {
    let mut step = 0;
    let mut chars = vec![];

    loop {
        let (nbytes, c) = next_u16_le_unicode(&bytes[step..])?;

        step += nbytes as usize;
        chars.push(c);

        if step == bytes.len() { break Ok(chars); }
    }
}



#[cfg(test)]
mod tests {
    use crate::etc::utf16::{encode_utf16_le, decode_utf16_le};


    #[test]
    fn test_utf16_le() {
        let s = "你好，世界！";
        let mut bytes = vec![];

        for c in s.chars() {
            encode_utf16_le(&mut bytes, c as u32).unwrap();
        }
        println!("{:?}", bytes);

        let chars = decode_utf16_le(&bytes).unwrap();
        println!("{:?}", chars);
    }
}

