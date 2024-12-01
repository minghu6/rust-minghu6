//! IEEE 754

use std::{
    fmt::{Debug, Display},
    marker::ConstParamTy,
};

use binpack::{pack_msb, Pack, Unpack};

////////////////////////////////////////////////////////////////////////////////
//// Constants

const K_P_ASSOC: [(ParameterK, usize); 4] =
    [(Half, 11), (Single, 24), (Double, 53), (Quadruple, 113)];


////////////////////////////////////////////////////////////////////////////////
//// Macros

macro_rules! define_and_impl {
    ({ $struct_name:ident, $k:path, $uint:ty, $sint:ty, $float:ty }) => {
        #[allow(unused)]
        #[derive(Clone, Copy)]
        #[repr(transparent)]
        struct $struct_name($uint);

        impl BinaryInterchangeFormat<{ $k }> for $struct_name {}

        #[allow(unused)]
        impl $struct_name {
            /// 1 bit
            ///
            /// signed
            #[allow(non_snake_case)]
            pub fn S(self) -> $uint {
                self.0.extract_msb(1..=1)
            }

            /// w bit
            ///
            /// biased exponent
            #[allow(non_snake_case)]
            pub fn E(self) -> $uint {
                self.0.extract_msb(2..=1 + Self::w())
            }

            /// t bit
            ///
            /// mantisa (significand precision)
            #[allow(non_snake_case)]
            pub fn T(self) -> $uint {
                self.0.extract_msb(Self::k() - Self::t() + 1..=Self::k())
            }

            /// true for negative
            pub fn signed(self) -> bool {
                self.S() != 0
            }

            pub fn exponent(self) -> $sint {
                self.E() as $sint - Self::bias() as $sint
            }

            pub fn trailing(self) -> $uint {
                self.T() >> (Self::k() - Self::t())
            }

            pub fn from_be_bytes(bytes: [u8; $k as usize / 8]) -> Self {
                Self(<$uint>::from_be_bytes(bytes))
            }

            pub fn from_le_bytes(bytes: [u8; $k as usize / 8]) -> Self {
                Self(<$uint>::from_le_bytes(bytes))
            }

            pub fn from_ne_bytes(bytes: [u8; $k as usize / 8]) -> Self {
                Self(<$uint>::from_ne_bytes(bytes))
            }

            pub fn from_components(
                signed: bool,
                exponent: $sint,
                trailing: $uint,
            ) -> Self {
                pack_msb! {
                    v: $uint = <{ signed as $uint }:1>
                             <
                                {
                                    if exponent < 0 {
                                        (Self::bias() as $sint + exponent) as $uint
                                    }
                                    else {
                                        Self::bias() as $uint + exponent as $uint
                                    }
                                }
                                :
                                { Self::w() }
                            >
                            <
                                {
                                    (trailing << (Self::k() - Self::t()))
                                }
                                :
                                { Self::t() }
                            >
                }

                Self(v)
            }

            pub fn explain(&self) -> FloatExplain {
                if self.E() == (1 << Self::w()) - 1 {
                    // overflow
                    if self.T() == 0 {
                        Infinity(self.signed())
                    } else {
                        // invalid operation exception
                        // IEEE-754-2008, ch-3.4, p19
                        // raw msb: 0 for signaling
                        NaN(
                            (self.T() >> (Self::t() - 1)) == 0
                        )
                    }
                }
                // underflow
                else if self.E() == 0 && self.T() == 0 {
                    SubNormal
                } else {
                    Normal
                }
            }

            pub fn to_float(&self) -> $float {
                <$float>::from_ne_bytes(self.0.to_ne_bytes())
            }
        }

        impl Debug for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let bytes = self.0.to_be_bytes();

                let mut d = f.debug_tuple(&format!("{} (MSB)", stringify!($struct_name)));

                for byte in bytes {
                    d.field_with(|fmt| write!(fmt, "{:08b}", byte));
                }

                d.finish()
            }
        }

        impl Display for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "({}) 1.({:b}) * 2^({})",
                    if self.signed() { "-" } else { "+" },
                    self.trailing(),
                    self.exponent()
                )
            }
        }

        impl From<$float> for $struct_name {
            fn from(value: $float) -> Self {
                Self::from_be_bytes(value.to_be_bytes())
            }
        }

    };

    ($( { $struct_name:ident, $k:path, $uint:ty, $sint:ty, $float:ty } )*) => {
        $(
            define_and_impl!({ $struct_name, $k, $uint, $sint, $float });
        )*
    };
}

define_and_impl! {
    { Float16,  Half,      u16,  i16,  f16  }
    { Float32,  Single,    u32,  i32,  f32  }
    { Float64,  Double,    u64,  i64,  f64  }
    { Float128, Quadruple, u128, i128, f128 }
}

////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait BinaryInterchangeFormat<const K: ParameterK> {
    /// total width in bits
    fn k() -> usize {
        K as _
    }

    /// precision in bits = trailing bits (t) + implicit leading `1` (1)
    fn p() -> usize {
        if Self::k() > 128 {
            Self::k() - 4 * Self::k().ilog2() as usize + 13
        } else {
            K_P_ASSOC.iter().find(|x| x.0 == K).unwrap().1
        }
    }

    /// exponent field width in bits
    fn w() -> usize {
        Self::k() - Self::p()
    }

    /// trailing significand field width in bits
    fn t() -> usize {
        Self::p() - 1
    }

    fn emax() -> usize {
        (1 << (Self::w() - 1)) - 1
    }

    fn bias() -> usize {
        Self::emax()
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Structures

///
/// |  k | s  |  e  |  w  |
/// |----|----|----|----|
/// |  16  | 01 | 05 | 10 |
/// |  32  | 01 | 08 | 23 |
/// |  64  | 01 | 11 | 52 |
/// | 128  | 01 | 15 | 112 |
/// | 256  | 01 | 19 | 237 |
///
#[derive(ConstParamTy, PartialEq, Eq)]
pub enum ParameterK {
    Half = 16,
    Single = 32,
    Double = 64,
    Quadruple = 128,
    Octuple = 256,
}

pub(crate) use ParameterK::*;


#[derive(Debug, PartialEq, Eq)]
pub enum FloatExplain {
    Normal,
    /// -0
    SignedZero,
    /// underflow
    SubNormal,

    /// overflow
    /// signed (true - 1 - negative)
    Infinity(bool),

    /// [quiet NaN](https://en.wikipedia.org/wiki/NaN#Quiet_NaN) \[default\]
    ///
    /// [signaling NaN](https://en.wikipedia.org/wiki/NaN#Signaling_NaN)
    NaN(bool),
}

pub(crate) use FloatExplain::*;

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl Display for FloatExplain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Normal => "N",
                SignedZero => "-0",
                SubNormal => "subN",
                Infinity(s) =>
                    if *s {
                        "-∞"
                    } else {
                        "∞"
                    },
                NaN(s) =>
                    if *s {
                        "sNaN"
                    } else {
                        "qNaN"
                    },
            }
        )
    }
}


#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use resource_config::RES;

    use super::*;

    #[test]
    fn test_float_spec() {
        assert_eq!(Float16::k(), 16);
        assert_eq!(Float16::p(), 11);
        assert_eq!(Float16::w(), 5);
        assert_eq!(Float16::t(), 10);
        assert_eq!(Float16::bias(), 15);

        assert_eq!(Float32::k(), 32);
        assert_eq!(Float32::p(), 24);
        assert_eq!(Float32::w(), 8);
        assert_eq!(Float32::t(), 23);
        assert_eq!(Float32::bias(), 127);

        assert_eq!(Float64::k(), 64);
        assert_eq!(Float64::p(), 53);
        assert_eq!(Float64::w(), 11);
        assert_eq!(Float64::t(), 52);
        assert_eq!(Float64::bias(), 1023);

        assert_eq!(Float128::k(), 128);
        assert_eq!(Float128::p(), 113);
        assert_eq!(Float128::w(), 15);
        assert_eq!(Float128::t(), 112);
        assert_eq!(Float128::bias(), 16383);
    }

    #[test]
    fn test_float_number() {
        println!("{}", Float32::from(92f32));

        let pinf16 = Float16::from_components(false, 0x10, 0);

        assert_eq!(pinf16.to_float(), f16::INFINITY, "{:?}", pinf16);

        let sNaN32 =  Float32::from_components(
            false,
            Float32::emax() as i32 + 1,
            1
        );

        println!("{}", sNaN32.to_float());

        assert_eq!(sNaN32.explain(), NaN(false),
        "{:?}, E: {:08b}, T: {:023b}", sNaN32, sNaN32.E(), sNaN32.T()
        );

        assert!(
            matches!(Float32::from(f32::NAN).explain(), NaN(_)),
        );
        assert!(
            matches!(Float32::from(f32::INFINITY).explain(), Infinity(false)),
        );
    }

    #[test]
    fn echo_plain_text_with_float() {
        const B: usize = 4;

        let mut bytes = RES.zh_en_poems_txt().load();

        for _ in bytes.len() % B + 1..=B {
            bytes.push(0);
        }

        for (i, s) in bytes.windows(B).enumerate() {
            let f = Float32::from_le_bytes(
                s.try_into().unwrap()
            );

            print!("{:#?}, ", f.to_float());

            if (i+1) % 4 == 0 {
                println!();
            }
        }

    }
}
