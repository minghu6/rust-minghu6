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

