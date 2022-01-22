// use num_format::{ Locale, ToFormattedString };
// use std::fmt::{ Debug, self };
use std::mem::size_of;

use either::Either;


////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait Reverse {
    fn reverse(&self) -> Self;
}


pub trait Conjugation<T> {
    fn adjoint(&self, baseline: &str) -> T;
}

pub trait BitLen {
    fn bit_len(&self) -> usize;
}

// pub trait NumENDebug: Debug {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
// }


////////////////////////////////////////////////////////////////////////////////
//// Implements

impl Reverse for Either<(), ()> {
    fn reverse(&self) -> Either<(), ()> {
        if self.is_left() {
            Either::Right(())
        } else {
            Either::Left(())
        }
    }
}


impl BitLen for usize {
    fn bit_len(&self) -> usize {
        size_of::<usize>() * 8 - self.leading_zeros() as usize
    }
}

impl BitLen for u32 {
    fn bit_len(&self) -> usize {
        size_of::<u32>() - self.leading_zeros() as usize
    }
}


// impl NumENDebug for usize {

//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.to_formatted_string(&Locale::en))
//     }
// }


////////////////////////////////////////////////////////////////////////////////
//// Declare Macro

#[macro_export]
macro_rules! ht {
    ( $head_expr:expr, $tail_expr:expr ) => {
        {
            let head = $head_expr;
            let tail = $tail_expr;

            let mut _vec = vec![head];
            _vec.extend(tail.iter().cloned());
            _vec
        }
    };
    ( $head:expr) => {
        {
            ht!($head, vec![])
        }
    };

}


/// should be used inner function which return Result<(), Box<dyn Error>>
#[macro_export]
macro_rules! should {
    ($cond:expr $(,)?) => {{
        use crate::XXXError;

        if !$cond {
            return Err(XXXError::new_box_err(
                ""
            ))
        }

     }};
    ($cond:expr, $args:tt) => {{
        use crate::XXXError;

        if !$cond {
            return Err(XXXError::new_box_err(
                format!($args).as_str()
            ))
        }

     }};
}


#[cfg(test)]
mod tests {

    use crate::*;
    // use super::NumENDebug;
    use std::error::Error;

    #[test]
    fn test_should() -> Result<(), Box<dyn Error>> {

        should!(2 < 2, "2 shoud lt 2");

        Ok(())
    }

    // #[test]
    // fn test_debug_num() {

    //     println!("{:?}", 10000000usize);
    // }

}