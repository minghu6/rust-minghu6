

/// (literal | expr | ident): 3x3
#[macro_export]
macro_rules! ht {
    ( $head_lit:literal | $tail_lit:expr ) => {
        {
            let mut _vec = vec![$head_lit];
            _vec.extend($tail_lit.iter().cloned());
            _vec
        }
    };
    ( $head_lit:literal | $tail:ident ) => {
        {
            let mut _vec = vec![$head_lit];
            _vec.extend($tail.iter().cloned());
            _vec
        }
    };
    ( $head:ident | $tail_lit:literal ) => {
        {
            let tail = $tail_lit;
            $ht![$head | $tail]
        }
    };
    ( $head:ident | $tail:ident ) => {
        {
            let mut _vec = vec![$head];
            _vec.extend($tail.iter().cloned());
            _vec
        }
    };
    ( $head:ident | $tail:expr ) => {
        {
            let mut _vec = vec![$head];
            _vec.extend($tail.iter().cloned());
            _vec
        }
    };
    ( $head:expr, | $tail:expr ) => {
        {
            let mut _vec = vec![$head];
            _vec.extend($tail.iter().cloned());
            _vec
        }
    };
    ( $head: ident) => {
        {
            vec![$head]
        }
    };
    ( $head: expr) => {
        {
            vec![$head]
        }
    };
    ( $head: ident) => {
        {
            vec![$head]
        }
    }

}
