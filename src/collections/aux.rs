use std::{
    fmt::{Debug, Display},
    ops::Index,
};

use super::Coll;
use crate::etc::StrJoin;



////////////////////////////////////////////////////////////////////////////////
//// Macro

#[macro_export]
macro_rules! rc_eq {
    ($x:expr,$y:expr) => {{
        std::rc::Rc::ptr_eq(&$x.clone().unwrap(), &$y.clone().unwrap())
    }};
}

/// Clone attr
#[macro_export]
macro_rules! attr {
    ($node:expr, $attr:ident) => {{
        let _unr = $node.clone().unwrap();
        let _bor = _unr.as_ref().borrow();
        let _attr = _bor.$attr.clone();
        drop(_bor);
        _attr
    }}; // $node.clone().unwrap().as_ref().borrow().$attr};
}

#[macro_export]
macro_rules! refattr {
    ($node:expr, $attr:ident) => {
        &$node.clone().unwrap().as_ref().borrow().$attr
    };
}

#[macro_export]
macro_rules! mattr {
    ($node:expr, $attr:ident) => {
        $node.clone().unwrap().as_ref().borrow_mut().$attr
    };
}

#[macro_export]
macro_rules! justinto {
    ($node:expr) => {
        std::rc::Rc::try_unwrap($node.unwrap())
            .unwrap()
            .into_inner()
    };
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
    Fail
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
