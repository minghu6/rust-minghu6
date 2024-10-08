//! IEEE 754

use std::fmt::{Debug, Display};

use binpack::{pack_msb, Pack, Unpack};

pub enum ParameterK {
    Half = 16,
    Single = 32,
    Double = 64,
    Quadruple = 128,
    Octuple = 256
}

pub struct BinaryInterchangeFormat;


/// MSB(u16)
///
/// s:  1
///
/// e:  5
///
/// t: 10
///
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Float16(u16);

#[derive(Debug)]
pub enum FloatExplain {
    Normal,
    /// -0
    SignedZero,
    /// underflow
    SubNormal,

    /// overflow
    /// signed (true - 1 - negative)
    Infinite(bool),

    ///
    ///
    /// [quiet NaN](https://en.wikipedia.org/wiki/NaN#Quiet_NaN) \[default\]
    ///
    /// [signaling NaN](https://en.wikipedia.org/wiki/NaN#Signaling_NaN)
    NaN(bool)
}

use FloatExplain::*;

impl Float16 {
    /// 1 bit
    ///
    /// signed
    pub fn s(self) -> u16 {
        self.0.extract_msb(1..=1)
    }

    /// 5 bit
    ///
    /// biased exponent
    pub fn e(self) -> u16 {
        self.0.extract_msb(2..=6)
    }

    pub fn e_bias() -> u16 {
        (1 << 4) - 1
    }

    /// 10 bit
    ///
    /// mantisa (significand precision)
    pub fn t(self) -> u16 {
        self.0.extract_msb(7..=16)
    }

    /// true for negative
    pub fn signed(self) -> bool {
        self.s() != 0
    }

    pub fn exponent(self) -> i16 {
        self.e() as i16 - Self::e_bias() as i16
    }

    pub fn trailing(self) -> u16 {
        self.t().reverse_bits() >> 6
    }

    ///
    pub fn from_be_bytes(bytes: [u8; 2]) -> Self {
        Self(u16::from_be_bytes(bytes))
    }

    pub fn from_components(
        signed: bool,
        exponent: i16,
        trailing: u16,
    ) -> Self {
        // debug_assert!(exponent < (1 << 4), "exponent overflow {exponent:0b}");
        // debug_assert!(
        //     exponent > -(1 << 4),
        //     "exponent -overflow {exponent:0b}"
        // );
        // debug_assert!(trailing < (1 << 11), "trailing overflow {trailing:0b}");

        // true to 1
        // let s = (signed as u16) << (size_of::<Self>() * 8 - 1);

        let e = if exponent < 0 {
             (Self::e_bias() as i16 + exponent) as u16
        }
        else {
            Self::e_bias() + exponent as u16
        };

        println!("e: {e:0b}");

        // let t = (trailing << 6).reverse_bits();

        // println!("{s:01b}, {e:05b}, {t:010b}");

        pack_msb! {
            v: u16 = <signed as u16:1>
                     <e:5>
                     <(trailing << 6).reverse_bits():10>
        }

        Self(v)
    }

    pub fn explain(&self) -> FloatExplain {
        if self.e() == (1 << 5) - 1 {
            // overflow
            if self.t() == 0 {
                FloatExplain::Infinite(self.signed())
            }
            else {
                // invalid operation exception
                // IEEE-754-2008
                FloatExplain::NaN(self.t() & 1 == 0)
            }
        }
        // underflow
        else if self.e() == 0 && self.t() == 0 {
            FloatExplain::SubNormal
        }
        else {
            FloatExplain::Normal
        }
    }

    pub fn to_f16(&self) -> f16 {
        f16::from_ne_bytes(self.0.to_ne_bytes())
    }
}

impl Debug for Float16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.0.to_be_bytes();

        f.debug_tuple("Float16 (MSB)")
            .field_with(|fmt| write!(fmt, "{:08b}", bytes[0]))
            .field_with(|fmt| write!(fmt, "{:08b}", bytes[1]))
            .finish()
    }
}

impl Display for Float16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}) 1.({}) * 2^({})",
            if self.signed() { "-" } else { "+" },
            self.trailing(),
            self.exponent()
        )
    }
}

impl From<f16> for Float16 {
    fn from(value: f16) -> Self {
        Self::from_be_bytes(value.to_be_bytes())
    }
}


impl Display for FloatExplain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Normal => "N",
            SignedZero => "-0",
            SubNormal => "subN",
            Infinite(s) => if *s { "-∞" } else { "∞" } ,
            NaN(s) => if *s { "sNaN" } else { "qNaN" },
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float16() {
        let inf = Float16::from_components(false, 0x10, 0);

        println!("{:?}", Float16::from_components(false, 0x10, 0x20).to_f16());
        assert_eq!(inf.to_f16(), f16::INFINITY);

        println!("{}", ParameterK::Half as u8);
    }
}
