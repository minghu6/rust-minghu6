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
        $vis struct Node<K, V>(
            Option<std::rc::Rc<std::cell::RefCell<Node_<K, V>>>>,
        );

        /// Used for reverse reference to avoid circular-reference
        ///
        /// So we can easy auto drop
        struct WeakNode<K, V>(
            Option<std::rc::Weak<std::cell::RefCell<Node_<K, V>>>>,
        );

        #[allow(unused)]
        impl<K, V> Node<K, V> {
            fn downgrade(&self) -> WeakNode<K, V> {
                WeakNode(
                    self.0.clone().map(|ref rc| std::rc::Rc::downgrade(rc)),
                )
            }

            fn as_ptr(&self) -> *mut Node_<K, V> {
                match self.0 {
                    Some(ref rc) => rc.as_ptr(),
                    None => std::ptr::null_mut(),
                }
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
                            std::rc::Rc::ptr_eq(rc1, rc2)
                        } else {
                            false
                        }
                    }
                    None => other.is_none(),
                }
            }
        }


        impl<K, V> Default for Node<K, V> {
            fn default() -> Self {
                Self::none()
            }
        }


        impl<K, V> Clone for Node<K, V> {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }


        #[allow(unused)]
        impl<K, V> WeakNode<K, V> {
            fn upgrade(&self) -> Node<K, V> {
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


        impl<K, V> Clone for WeakNode<K, V> {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
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

#[macro_export]
macro_rules! node {
    (BST { $key:expr, $val:expr $(,$attr:ident : $attr_val:expr)* }) => {
        Node(Some(std::rc::Rc::new(std::cell::RefCell::new(Node_ {
            left: Node::none(),
            right: Node::none(),
            paren: WeakNode::none(),

            key: $key,
            val: $val,

            $(
                $attr: $attr_val
            ),*
        }))))
    };
    (FREE { $($attr:ident : $attr_val:expr),* }) => {
        Node(Some(std::rc::Rc::new(std::cell::RefCell::new(Node_ {
            $(
                $attr: $attr_val
            ),*
        }))))
    };
    (FREE-ENUM $ty:ident { $($attr:ident : $attr_val:expr),* }) => {
        Node(Some(std::rc::Rc::new(std::cell::RefCell::new(Node_::$ty {
            $(
                $attr: $attr_val
            ),*
        }))))
    };
}

#[macro_export]
macro_rules! unwrap_into {
    ($node:expr) => {
        std::rc::Rc::try_unwrap($node.0.unwrap())
            .unwrap()
            .into_inner()
    };
}


////////////////////////////////////////
//// Attr macros

/// Evil hack for Rc<RefCell<T>>
#[macro_export]
macro_rules! attr {
    (ref_mut | $node:expr, $attr:ident, $ty:ty) => {{
        if let Some(_unr) = $node.clone().0 {
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
        if let Some(_unr) = $node.clone().0 {
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
        if let Some(_unr) = $node.clone().0 {
            let _bor = _unr.as_ref().borrow();
            let _attr = _bor.$attr.clone();
            drop(_bor);
            _attr
        }
        else {
            panic!("Access {} on None", stringify!($attr));
        }
    }};
    (self | $node:expr) => {{
        if let Some(_unr) = $node.clone().0 {
            unsafe { &* _unr.as_ref().as_ptr() }
        }
        else {
            panic!("Call {} on None", stringify!($attr));
        }
    }};
    (self_mut | $node:expr) => {{
        if let Some(_unr) = $node.clone().0 {
            unsafe { &mut * _unr.as_ref().as_ptr() }
        }
        else {
            panic!("Call {} on None", stringify!($attr));
        }
    }};
    ($node:expr, $attr:ident, $val:expr) => {{
        if let Some(bor) = $node.clone().0 {
            bor.as_ref().borrow_mut().$attr = $val
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