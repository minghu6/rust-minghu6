use std::{fmt::{Debug, Display}, ops::Index};

use crate::etc::StrJoin;

use super::Collection;


#[derive(Clone)]
pub(crate) struct RoadMap {
    data: Vec<i32>,
}

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

impl Collection for RoadMap {
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
        write!(f, "{}", (&mut self.data.iter() as &mut dyn Iterator<Item=&i32>).strjoin("-"))
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

        let rm  = roadmap![0, 1, 2];

        println!("{}", rm);
    }

}