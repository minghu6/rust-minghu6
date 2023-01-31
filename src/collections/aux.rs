use std::{
    fmt::{Debug, Display},
    ops::Index,
};


use super::Coll;
use crate::etc::StrJoin;


////////////////////////////////////////////////////////////////////////////////
//// Macro


////////////////////////////////////////
//// Node wrapper

macro_rules! boxptr {
    ($v:expr) => {
        Box::into_raw(Box::new($v))
    };
}


macro_rules! unboxptr {
    ($ptr:expr) => {
        unsafe { *Box::from_raw($ptr) }
    };
}


macro_rules! node {
    (BST { $key:expr, $val:expr $(,$attr:ident : $attr_val:expr)* }) => {
        Node(Some(std::rc::Rc::new(std::cell::RefCell::new(Node_ {
            left: Node::none(),
            right: Node::none(),
            paren: WeakNode::none(),

            key: boxptr!($key),
            val: boxptr!($val),

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
}


macro_rules! unwrap_into {
    ($node:expr) => {
        std::rc::Rc::try_unwrap($node.0.unwrap())
            .unwrap()
            .into_inner()
    };
}


////////////////////////////////////////
//// Attr macros

macro_rules! attr {
    ($node:expr, $attr:ident) => {{
        /* to pass runtime borrow check  */
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
    ($node:expr, $attr:ident, $val:expr) => {{
        if let Some(bor) = $node.clone().0 {
            bor.as_ref().borrow_mut().$attr = $val
        }
        else {
            panic!("MAccess {} on None", stringify!($attr));
        }
    }};
}


macro_rules! def_attr_macro {
    ($($name:ident),+) => {
        $(
            macro_rules! $name {
                ($node:expr) => {
                    attr!($$node, $name)
                };
                ($node:expr, $val:expr) => {
                    attr!($$node, $name, $$val)
                };
            }
            #[allow(unused)]
            pub(crate) use $name;

        )+
    };
    (ptr | $($name:ident),+) => {
        $(
            macro_rules! $name {
                ($node:expr) => {
                    unsafe { &* attr!($$node, $name) }
                };
                ($node:expr, $val:expr) => {
                    attr!($$node, $name, $$val)
                };
            }
            #[allow(unused)]
            pub(crate) use $name;

            concat_idents::concat_idents! (name_mut = $name, _mut {
                #[allow(unused)]
                macro_rules! name_mut {
                    ($node:expr) => {
                        unsafe { &mut * attr!($$node, $name) }
                    };
                }
                #[allow(unused)]
                pub(crate) use name_mut;
            });
        )+
    };
}


////////////////////////////////////////
//// Etc.

/// Hack method convert self to self_mut
macro_rules! mut_self {
    ($self: ident) => {
         unsafe { &mut *($self as *const Self as *mut Self) }
    };
}



pub(crate) use node;
pub(crate) use attr;
pub(crate) use boxptr;
pub(crate) use unboxptr;
pub(crate) use unwrap_into;
pub(crate) use def_attr_macro;
pub(crate) use mut_self;

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
}


impl Coll for RoadMap {
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




#[macro_export]
macro_rules! roadmap {
    ($($item:expr),*) => {
        {
            use crate::collections::aux::RoadMap;

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


#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
    fn test_roadmap() {
        let rm = roadmap![0, 1, 2];

        println!("{}", rm);
    }
}
