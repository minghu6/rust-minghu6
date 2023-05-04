use std::{
    fmt::{Debug, Display},
    ops::Index,
};

use common::StrJoin;


////////////////////////////////////////////////////////////////////////////////
//// Macro


////////////////////////////////////////
//// Node wrapper

/// Define node wrapper
#[macro_export]
macro_rules! impl_node {
    () => {
        impl_node!(pub(self));
    };
    ($vis:vis) => {
        impl_node!($vis <K, V>, rc);
    };
    ($vis:vis <$($g:ident),+>, rc) => {
        impl_node!(
            $vis <$($g),+>,
            std::rc::Rc<std::cell::RefCell<Node_<$($g),+>>>,
            std::rc::Weak<std::cell::RefCell<Node_<$($g),+>>>
        );

        #[allow(unused)]
        impl<$($g),+> Node<$($g),+> {
            fn as_ptr(&self) -> *mut Node_<$($g),+> {
                match self.0 {
                    Some(ref rc) => rc.as_ptr(),
                    None => std::ptr::null_mut(),
                }
            }
        }

        #[allow(unused)]
        macro_rules! aux_node {
            ({ $$($attr:ident : $attr_val:expr),* $$(,)? }) => {
                Node(Some(std::rc::Rc::new(std::cell::RefCell::new(Node_ {
                    $$(
                        $attr: $attr_val
                    ),*
                }))))
            };
            (ENUM $ty:ident { $$($attr:ident : $attr_val:expr),* $$(,)? }) => {
                Node(Some(std::rc::Rc::new(std::cell::RefCell::new(Node_::$ty {
                    $$(
                        $attr: $attr_val
                    ),*
                }))))
            };
        }

        #[allow(unused)]
        macro_rules! unwrap_into {
            ($node:expr) => {
                std::rc::Rc::try_unwrap($node.0.unwrap())
                    .unwrap()
                    .into_inner()
            };
        }
    };
    ($vis:vis <$($g:ident),+>, arc) => {
        impl_node!(
            $vis <$($g),+>,
            std::sync::Arc<UnsafeSendSync<Node_<$($g),+>>>,
            std::sync::Weak<UnsafeSendSync<Node_<$($g),+>>>
        );
        macro_rules! aux_node {
            ({ $$($attr:ident : $attr_val:expr),* }) => {
                Node(Some(std::sync::Arc::new(UnsafeSendSync::new(Node_ {
                    $$(
                        $attr: $attr_val
                    ),*
                }))))
            };
            (ENUM $ty:ident { $$($attr:ident : $attr_val:expr),* }) => {
                Node(Some(std::sync::Arc::new(UnsafeSendSync::new(Node_::$ty {
                    $$(
                        $attr: $attr_val
                    ),*
                }))))
            };
        }
    };
    ($vis:vis <$($g:ident),+>, $rc:ty, $wk:ty) => {
        $vis struct Node<$($g),+>(
            Option<$rc>,
        );

        /// Used for reverse reference to avoid circular-reference
        ///
        /// So we can easy auto drop
        struct WeakNode<$($g),+>(
            Option<$wk>,
        );

        #[allow(unused)]
        impl<$($g),+> Node<$($g),+> {
            fn downgrade(&self) -> WeakNode<$($g),+> {
                WeakNode(
                    self.0.clone().map(|ref rc| <$rc>::downgrade(rc)),
                )
            }

            fn none() -> Self {
                Self(None)
            }

            fn is_some(&self) -> bool {
                self.0.is_some()
            }

            fn is_none(&self) -> bool {
                self.0.is_none()
            }

            fn rc_eq(&self, other: &Self) -> bool {
                match self.0 {
                    Some(ref rc1) => {
                        if let Some(ref rc2) = other.0 {
                            <$rc>::ptr_eq(rc1, rc2)
                        } else {
                            false
                        }
                    }
                    None => other.is_none(),
                }
            }
        }


        impl<$($g),+> Default for Node<$($g),+> {
            fn default() -> Self {
                Self::none()
            }
        }


        impl<$($g),+> Clone for Node<$($g),+> {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }


        #[allow(unused)]
        impl<$($g),+> WeakNode<$($g),+> {
            fn upgrade(&self) -> Node<$($g),+> {
                Node(self.0.clone().map(|weak| {
                    if let Some(strong) = weak.upgrade() {
                        strong
                    } else {
                        unreachable!("weak node upgrade failed")
                    }
                }))
            }

            fn none() -> Self {
                Self(None)
            }

            fn is_none(&self) -> bool {
                self.0.is_none()
            }

            fn is_some(&self) -> bool {
                self.0.is_some()
            }

            fn replace(&mut self, oth: Self) {
                self.0 = oth.0;
            }
        }


        impl<$($g),+> Clone for WeakNode<$($g),+> {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }
    }
}

////////////////////////////////////////
//// Node Implementation


/// Node_ heap data field access
#[macro_export]
macro_rules! def_node__heap_access {
    (internal, $name:ident, $ret:ty) => {
        #[inline]
        fn $name(&self) -> &$ret {
            match self {
                Internal { $name, .. } => $name,
                Leaf {..} => panic!(
                    "Get `{}` on leaf",
                    stringify!($name)
                )
            }
        }
        coll::paste!(
            #[inline]
            fn [<$name _mut>](&mut self) -> &mut $ret {
                match self {
                    Internal { $name, .. } => $name,
                    Leaf {..} => panic!(
                        "Get `{}` on leaf",
                        stringify!($name)
                    )
                }
            }
        );
    };
    (leaf, $name:ident, $ret:ty) => {
        #[inline]
        fn $name(&self) -> &$ret {
            match self {
                Internal {..} => panic!(
                    "Get `{}` on internal node",
                    stringify!($name)
                ),
                Leaf { $name, ..} => $name
            }
        }
        coll::paste!(
            #[inline]
            fn [<$name _mut>](&mut self) -> &mut $ret {
                match self {
                    Internal {..} => panic!(
                        "Get `{}` on internal node",
                        stringify!($name)
                    ),
                    Leaf { $name, ..} => $name
                }
            }
        );
    };
    (both, $name:ident, $ret:ty) => {
        #[inline]
        fn $name(&self) -> &$ret {
            match self {
                Internal { $name, .. } => $name,
                Leaf {$name, ..} => $name
            }
        }
        coll::paste!(
            #[inline]
            fn [<$name _mut>](&mut self) -> &mut $ret {
                match self {
                    Internal { $name, .. } => $name,
                    Leaf {$name, ..} => $name
                }
            }
        );
    };
}


/// Node_ WeakNode field access
#[macro_export]
macro_rules! def_node__wn_access {
    (both, $name:ident) => {
        fn $name(&self) -> Node<K, V> {
            match self {
                Internal { $name, .. } => $name,
                Leaf { $name, .. } => $name,
            }
            .upgrade()
        }
        coll::paste!(
            fn [<set_ $name>](&mut self, x: Node<K, V>) {
                match self {
                    Internal { $name, .. } => $name,
                    Leaf { $name, .. } => $name,
                }
                .replace(x.downgrade());
            }
        );
    };
    (leaf, $name:ident) => {
        fn $name(&self) -> Node<K, V> {
            match self {
                Internal {..} => panic!(
                    "Get `{}` on internal node",
                    stringify!($name)
                ),
                Leaf { $name, .. } => $name,
            }
            .upgrade()
        }
        coll::paste!(
            fn [<set_ $name>](&mut self, x: Node<K, V>) {
                match self {
                    Internal {..} => panic!(
                        "Get `{}` on internal node",
                        stringify!($name)
                    ),
                    Leaf { $name, .. } => $name,
                }
                .replace(x.downgrade());
            }
        );
    };
}


////////////////////////////////////////
//// Attr macros

/// Evil hack for Rc<RefCell<T>>
#[macro_export]
macro_rules! attr {
    (ref_mut | $node:expr, $attr:ident, $ty:ty) => {{
        if let Some(ref _unr) = $node.0 {
            let _bor = _unr.as_ref().borrow();
            let _attr = (&_bor.$attr) as *const $ty as *mut $ty;
            drop(_bor);
            unsafe { &mut *_attr }
        }
        else {
            panic!("Access {} on None", stringify!($attr));
        }
    }};
    (ref | $node:expr, $attr:ident, $ty:ty) => {{
        if let Some(ref _unr) = $node.0 {
            let _bor = _unr.as_ref().borrow();
            let _attr = (&_bor.$attr) as *const $ty;
            drop(_bor);
            unsafe { &*_attr }
        }
        else {
            panic!("Access {} on None", stringify!($attr));
        }
    }};
    (clone | $node:expr, $attr:ident) => {{
        if let Some(ref _unr) = $node.0 {
            let _bor = _unr.as_ref().borrow();
            let _attr = _bor.$attr.clone();
            drop(_bor);
            _attr
        }
        else {
            panic!("Access {} on None", stringify!($attr));
        }
    }};
    ($node:expr, $attr:ident, $val:expr) => {{
        if let Some(ref _unr) = $node.0 {
            _unr.as_ref().borrow_mut().$attr = $val
        }
        else {
            panic!("MAccess {} on None", stringify!($attr));
        }
    }};
    (self | $node:expr) => {{
        if let Some(ref _unr) = $node.0 {
            unsafe { &* _unr.as_ref().as_ptr() }
        }
        else {
            panic!("Call {} on None", stringify!($attr));
        }
    }};
    (self_mut | $node:expr) => {{
        if let Some(ref _unr) = $node.0 {
            unsafe { &mut * _unr.as_ref().as_ptr() }
        }
        else {
            panic!("Call {} on None", stringify!($attr));
        }
    }};
    (self_unsafe_sync | $node:expr) => {{
        if let Some(ref _unr) = $node.0 {
            unsafe { &* _unr.as_ref().as_ptr() }
        }
        else {
            panic!("Call {} on None", stringify!($attr));
        }
    }};
    (self_unsafe_sync_mut | $node:expr) => {{
        if let Some(ref _unr) = $node.0 {
            unsafe { &mut* _unr.as_ref().as_ref_mut_ptr() }
        }
        else {
            panic!("MAccess {} on None", stringify!($attr));
        }
    }};
}


#[macro_export]
macro_rules! def_attr_macro {
    (ref | $(($name:ident,$ty:ty)),+) => {
        $(
            macro_rules! $name {
                ($node:expr) => {
                    attr!(ref | $$node, $name, $ty)
                };
                ($node:expr, $val:expr) => {
                    attr!($$node, $name, $$val)
                };
            }
            #[allow(unused)]
            pub(crate) use $name;

            coll::paste!(
                #[allow(unused)]
                macro_rules! [<$name _mut>] {
                    ($node:expr) => {
                        attr!(ref_mut | $$node, $name, $ty)
                    };
                }
                #[allow(unused)]
                pub(crate) use [<$name _mut>];
            );
        )+
    };
    (clone | $($name:ident),+) => {
        $(
            macro_rules! $name {
                ($node:expr) => {
                    attr!(clone | $$node, $name)
                };
                ($node:expr, $val:expr) => {
                    attr!($$node, $name, $$val)
                };
            }
            #[allow(unused)]
            pub(crate) use $name;

        )+
    };
    (call | $($name:ident),+) => {
        $(
            coll::paste!(
                macro_rules! $name {
                    ($node:expr) => {
                        attr!(self | $$node).$name()
                    };
                    ($node:expr, $val:expr) => {
                        attr!(self_mut | $$node).[<set_ $name>]($$val)
                    };
                }
            );

            coll::paste!(
                #[allow(unused)]
                macro_rules! [<$name _mut>] {
                    ($node:expr) => {
                        attr!(self_mut | $$node).[<$name _mut>]()
                    };
                }
            );
        )+
    };
    (call_unsafe_sync | $($name:ident),+) => {
        $(
            coll::paste!(
                macro_rules! $name {
                    ($node:expr) => {
                        attr!(self_unsafe_sync | $$node).$name()
                    };
                }
            );

            coll::paste!(
                #[allow(unused)]
                macro_rules! [<$name _mut>] {
                    ($node:expr) => {
                        attr!(self_unsafe_sync_mut | $$node).[<$name _mut>]()
                    };
                }
            );
        )+
    }
}




////////////////////////////////////////
//// Etc.

/// Hack method convert self to self_mut
#[macro_export]
macro_rules! mut_self {
    ($self: ident) => {
         unsafe { &mut *($self as *const Self as *mut Self) }
    };
}


#[macro_export]
macro_rules! swap {
    (node | $x:expr, $y:expr, $attr:ident, $ty:ty) => {
        {
            let x = $x.clone();
            let y = $y.clone();

            let x_attr = attr!(ref_mut| x, $attr, $ty);
            let y_attr = attr!(ref_mut| y, $attr, $ty);

            std::mem::swap(x_attr, y_attr);
        }
    };
}


#[macro_export]
macro_rules! boxptr {
    ($v:expr) => {
        Box::into_raw(Box::new($v))
    };
}


#[macro_export]
#[allow(unused)]
macro_rules! unboxptr {
    ($ptr:expr) => {
        unsafe { *Box::from_raw($ptr) }
    };
}


#[allow(unused)]
#[macro_export]
macro_rules! roadmap {
    ($($item:expr),*) => {
        {
            use crate::aux::RoadMap;

            #[allow(unused_mut)]
            let mut _roadmap = RoadMap::empty();

            $(
                let item = $item;
                _roadmap.push(item);
            )*

            _roadmap
        }
    }

}


#[macro_export]
macro_rules! parse_range {
    ($range:expr, $len:expr) => {{
        use std::ops::Bound::*;

        let range = $range;
        let len = $len;

        let l;
        let r;

        match range.start_bound() {
            Included(v) => l = *v,
            Excluded(v) => l = *v + 1,
            Unbounded => l = 0,
        }

        match range.end_bound() {
            Included(v) => r = *v,
            Excluded(v) => {
                assert!(*v > 0, "range upper is invalid (=0)");
                r = *v - 1
            }
            Unbounded => r = len - 1,
        }

        (l, r)
    }};
}


////////////////////////////////////////////////////////////////////////////////
//// Structure

#[derive(Clone)]
pub(crate) struct RoadMap {
    data: Vec<i32>,
}


pub type VerifyResult = Result<(), VerifyError>;

#[derive(PartialEq, Eq, Debug)]
pub enum VerifyError {
    Inv(String),
    Fail(String)
}


////////////////////////////////////////////////////////////////////////////////
//// Implmentation

#[allow(unused)]
impl RoadMap {
    pub(crate) fn empty() -> Self {
        Self { data: Vec::new() }
    }

    pub(crate) fn push(&mut self, pos: i32) {
        self.data.push(pos);
    }

    pub(crate) fn ppush(&self, pos: i32) -> Self {
        let mut roadmap = self.clone();
        roadmap.push(pos);
        roadmap
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}


impl Default for RoadMap {
    fn default() -> Self {
        Self::empty()
    }
}

impl Debug for RoadMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            (&mut self.data.iter() as &mut dyn Iterator<Item = &i32>)
                .strjoin("-")
        )
    }
}

impl Display for RoadMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Index<usize> for RoadMap {
    type Output = i32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}


pub fn vec_ordered_insert<T: Ord>(vec: &mut Vec<T>, x: T) -> Option<T> {
    match vec.binary_search(&x) {
        Ok(oldidx) => {
            Some(std::mem::replace(&mut vec[oldidx], x))
        },
        Err(inseridx) => {
            vec.insert(inseridx, x);
            None
        },
    }
}



#[cfg(test)]
mod tests {

    #[test]
    fn test_roadmap() {
        let rm = roadmap![0, 1, 2];

        println!("{}", rm);
    }
}
