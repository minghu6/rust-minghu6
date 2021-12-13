use either::Either;



////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait Reverse<T> {
    fn reverse(&self) -> T;
}


pub trait Conjugation<T> {
    fn adjoint(&self, baseline: &str) -> T;
}



////////////////////////////////////////////////////////////////////////////////
//// Implements

impl Reverse<Either<(), ()>> for Either<(), ()> {
    fn reverse(&self) -> Either<(), ()> {
        if self.is_left() {
            Either::Right(())
        } else {
            Either::Left(())
        }
    }
}




////////////////////////////////////////////////////////////////////////////////
//// Declare Macro

/// (literal | expr | ident): 3x3
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
